use anyhow::{Context, bail};
use hickory_resolver::TokioResolver;
use hickory_resolver::config::{LookupIpStrategy, ResolveHosts, ResolverOpts};
use hickory_resolver::net::{DnsError, NetError};
use hickory_resolver::proto::op::ResponseCode;
use hickory_resolver::proto::rr::Name;
use reqwest::Url;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, trace, warn};
use url::Host;

use crate::work::ResponseError;

/// Smallest accepted budget for DNS resolution.
///
/// Below ~2.223ms the 1ms floor in `pool_deadline` starts binding, and one pool send's
/// worst case (`2 * opts.timeout`) escapes the budget — see
/// `pool_deadline_stays_under_the_budget`. No real lookup completes that fast anyway.
pub const MIN_BUDGET: Duration = Duration::from_millis(3);

#[derive(Debug, Clone, Copy)]
pub struct DnsResolverOptions {
    /// Wall-clock bound on resolving a single target, timeout included.
    pub budget: Duration,
    pub positive_max_ttl: Duration,
    pub negative_max_ttl: Duration,
    pub ip_strategy: LookupIpStrategy,
    pub append_search_domains: bool,
}

/// An asynchronous DNS resolver shared by every request attempt in the process.
#[derive(Debug, Clone)]
pub struct DnsResolver {
    inner: TokioResolver,
    budget: Duration,
    append_search_domains: bool,
}

/// Deadline for one hickory name-server pool send, derived from the caller's total budget.
pub fn pool_deadline(budget: Duration) -> Duration {
    (budget / 20 * 9).max(Duration::from_millis(1))
}

/// Applies the worker's DNS policy on top of whatever /etc/resolv.conf asked for.
fn apply_options(opts: &mut ResolverOpts, options: &DnsResolverOptions) {
    opts.ip_strategy = options.ip_strategy;
    opts.use_hosts_file = ResolveHosts::Always;
    opts.positive_max_ttl = Some(options.positive_max_ttl);
    opts.negative_max_ttl = Some(options.negative_max_ttl);
    opts.attempts = 0;
    opts.timeout = pool_deadline(options.budget);
}

impl DnsResolver {
    pub fn new(options: DnsResolverOptions) -> anyhow::Result<Self> {
        if options.budget < MIN_BUDGET {
            bail!("DNS budget must be at least {:?}", MIN_BUDGET);
        }

        // `builder_tokio` reads /etc/resolv.conf (the registry on Windows) and overwrites the
        // whole `ResolverOpts` struct with what it finds, so our own tweaks must come after it.
        let mut builder = TokioResolver::builder_tokio()
            .context("Could not read the system DNS configuration (/etc/resolv.conf)")?;
        apply_options(builder.options_mut(), &options);

        let inner = builder
            .build()
            .context("Could not build the DNS resolver")?;
        Ok(Self {
            inner,
            budget: options.budget,
            append_search_domains: options.append_search_domains,
        })
    }

    /// Resolves a webhook target to the set of addresses it is allowed to be dialed on.
    ///
    /// The returned addresses should then be pinned into the HTTP client, so the connection
    /// can only go to something that was vetted here.
    pub async fn resolve_target(
        &self,
        url: &Url,
        allow_forbidden_ips: bool,
    ) -> Result<Vec<SocketAddr>, ResolveError> {
        let (host, port) = target_endpoint(url)?;

        let ips = match host {
            Host::Ipv4(ip) => vec![IpAddr::V4(ip)],
            Host::Ipv6(ip) => vec![IpAddr::V6(ip)],
            Host::Domain(domain) => {
                // Parse the name up front. `lookup_ip` does this internally and reports a bad
                // name as `NetError::Proto` — the same variant it uses for a malformed response
                // from the server.
                let name = target_name(domain, self.append_search_domains)?;
                trace!(target_http_url = %url, dns_name = %name, "Resolving target hostname");
                let lookup = timeout(self.budget, self.inner.lookup_ip(name))
                    .await
                    .map_err(|_| ResolveError::Timeout)?
                    .map_err(map_net_error)?;
                lookup.iter().collect::<Vec<IpAddr>>()
            }
        };

        trace!(target_http_url = %url, ?ips, "Resolved target");
        vet_addresses(ips, port, allow_forbidden_ips)
    }
}

/// Splits a URL into the host to resolve and the port to connect to.
///
/// This mirrors `Url::socket_addrs(|| None)`, including the order of its two failure modes: it
/// reports a missing host before a missing port.
fn target_endpoint(url: &Url) -> Result<(Host<&str>, u16), ResolveError> {
    let host = url.host().ok_or(ResolveError::NoHost)?;
    let port = url.port_or_known_default().ok_or(ResolveError::NoPort)?;
    Ok((host, port))
}

/// Parses a URL host into the exact DNS name to query.
fn target_name(domain: &str, append_search_domains: bool) -> Result<Name, ResolveError> {
    let mut name =
        Name::from_utf8(domain).map_err(|_| ResolveError::InvalidName(domain.to_owned()))?;
    if !append_search_domains {
        name.set_fqdn(true);
    }
    Ok(name)
}

/// Checks resolved addresses against the target-IP guard and pairs the survivors with `port`.
fn vet_addresses(
    ips: Vec<IpAddr>,
    port: u16,
    allow_forbidden: bool,
) -> Result<Vec<SocketAddr>, ResolveError> {
    if ips.is_empty() {
        return Err(ResolveError::NoAddress);
    }

    // Reject if *any* resolved address is forbidden: a hostname that resolves to a mix of public and internal addresses must not pass.
    let has_forbidden_ip = ips.iter().any(|ip| is_forbidden_ip(*ip));

    if has_forbidden_ip && !allow_forbidden {
        debug!(
            ?ips,
            forbidden_ips = ?ips.iter().filter(|ip| is_forbidden_ip(**ip)).collect::<Vec<_>>(),
            "Target rejected: it resolves to at least one IP that is not globally reachable"
        );

        // Unlike glibc's `getaddrinfo`, hickory has no `AI_ADDRCONFIG`, so AAAA records are
        // returned even on a host with no global IPv6 address. A target whose A record is fine but
        // whose AAAA record points somewhere internal used to be delivered and is now rejected.
        // Log that case distinctly so its blast radius is measurable before anyone reports it.
        //
        // This only concerns *resolved* addresses: a URL with an IPv6 literal host takes the
        // `Host::Ipv6` branch in `resolve_target`, never reaches the resolver, and so is not
        // affected by `DNS_IP_STRATEGY` at all -- `is_forbidden_ip` is its only guard.
        if rejected_only_because_of_ipv6(&ips) {
            warn!(
                ?ips,
                "Target rejected only because of its IPv6 addresses; its IPv4 addresses are globally reachable (set DNS_IP_STRATEGY=ipv4-only to ignore AAAA records -- but not if this worker reaches the Internet through NAT64, where IPv4-only targets are reachable only via their synthesized AAAA records)"
            );
        }
        return Err(ResolveError::ForbiddenIp);
    }

    if has_forbidden_ip {
        debug!(
            "Target URL resolves to a forbidden IP but this is allowed in the worker's configuration"
        );
    }

    Ok(ips
        .into_iter()
        .map(|ip| SocketAddr::new(ip, port))
        .collect())
}

/// Whether every address that failed the guard is an IPv6 one, while a usable IPv4 one remains.
///
/// This is exactly the set of targets that used to be delivered on an IPv4-only host, where
/// `getaddrinfo`'s `AI_ADDRCONFIG` hid the AAAA records that are now visible.
fn rejected_only_because_of_ipv6(ips: &[IpAddr]) -> bool {
    let mut offenders = ips.iter().filter(|ip| is_forbidden_ip(**ip)).peekable();
    // `all` is vacuously true on an empty iterator, so check that something was rejected at all.
    let all_offenders_are_benign_ipv6 = offenders.peek().is_some()
        && offenders.all(|ip| match ip {
            // An IPv6 address that names an IPv4 one is not a benign "this target's AAAA record
            // is not globally reachable" case, and `ipv4-only` is not its fix: following that
            // advice would hide an SSRF attempt instead of fixing a misconfiguration.
            IpAddr::V6(ip) => match classify_ipv4_carrier(ip) {
                Ipv4Carrier::NotIpv4Carrying => true,
                Ipv4Carrier::Nat64WellKnown(_) | Ipv4Carrier::NeverAValidTarget => false,
            },
            IpAddr::V4(_) => false,
        });
    let has_usable_ipv4 = ips.iter().any(|ip| ip.is_ipv4() && !is_forbidden_ip(*ip));

    all_offenders_are_benign_ipv6 && has_usable_ipv4
}

/// Translates a hickory failure into the taxonomy the rest of the worker reasons about.
fn map_net_error(error: NetError) -> ResolveError {
    match error {
        NetError::Timeout => ResolveError::Timeout,
        NetError::Dns(DnsError::NoRecordsFound(_)) => ResolveError::NoAddress,
        NetError::Dns(DnsError::ResponseCode(code)) => ResolveError::ServerFailure(code),
        // `NetError` and `DnsError` are `#[non_exhaustive]`, so this arm has to stay. A wire
        // error, an I/O error and "no usable name server" all mean the same thing to us: no
        // answer, for a reason that is not the target URL's fault.
        other => ResolveError::Internal {
            detail: other.to_string(),
        },
    }
}

/// A DNS-layer failure, classified by fault.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResolveError {
    #[error("No host name in the URL")]
    NoHost,

    #[error("No port number in the URL")]
    NoPort,

    #[error("Could not resolve URL: {0} is not a valid hostname")]
    InvalidName(String),

    #[error("URL did not resolve to any IP address")]
    NoAddress,

    #[error("URL resolves to a forbidden IP")]
    ForbiddenIp,

    #[error("Timed out resolving target's IP using DNS")]
    Timeout,

    #[error("Could not resolve URL: DNS server returned {0}")]
    ServerFailure(ResponseCode),

    /// `detail` is deliberately NOT interpolated into the message: `Display` is stored as the
    /// customer-visible response body, while `detail` is for operators only. See
    /// [`ResolveError::detail`]. Do not "fix" this by adding `{detail}`.
    #[error("Could not resolve URL: DNS resolution failed")]
    Internal { detail: String },
}

impl ResolveError {
    /// The code the user sees. Every DNS-layer failure reports `E_DNS`, whether we could
    /// not reach a server or a server answered with an error: those cannot be told apart by
    /// fault, and `E_INVALID_TARGET` is plainly wrong for a syntactically valid URL.
    pub fn response_error(&self) -> ResponseError {
        match self {
            Self::Timeout | Self::ServerFailure(_) | Self::Internal { .. } => ResponseError::Dns,
            Self::NoHost
            | Self::NoPort
            | Self::InvalidName(_)
            | Self::NoAddress
            | Self::ForbiddenIp => ResponseError::InvalidTarget,
        }
    }

    /// The underlying cause, for operators. Excluded from `Display` — and so from the stored,
    /// customer-visible body — on purpose. Its presence in a log line is what distinguishes
    /// "we never reached a server" from "a server said no".
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Internal { detail } => Some(detail),
            _ => None,
        }
    }
}

/// How an IPv6 address relates to an IPv4 one, and whether it can be a webhook target at all.
///
/// Returned by [`classify_ipv4_carrier`]. Every caller must `match` on this exhaustively: the
/// whole point of the enum is that [`Ipv4Carrier::Nat64WellKnown`] cannot be silently folded
/// into the refuse-outright path by a later edit, because doing so would break every webhook
/// delivery on a NAT64 deployment.
enum Ipv4Carrier {
    /// NAT64 well-known prefix (`64:ff9b::/96`, RFC 6052 section 2.1). The verdict comes from
    /// the IPv4 address carried, *not* from the prefix.
    ///
    /// This prefix cannot be refused wholesale: under NAT64/DNS64, DNS64 synthesizes AAAA
    /// records here for every IPv4-only target, so `example.com` legitimately resolves to
    /// `64:ff9b::5db8:d822` and refusing the prefix would stop all delivery. IANA marks it
    /// globally reachable for that reason. But nothing stops the carried address from being
    /// 127.0.0.1, so the IPv4 rules are applied to it. RFC 6052 section 3.1 forbids embedding
    /// non-global IPv4 addresses here anyway, so this refuses nothing legal.
    Nat64WellKnown(Ipv4Addr),

    /// A transition format, or a reserved block those formats live in, that is never a
    /// legitimate webhook target -- whatever it encodes, and whether or not it encodes an
    /// IPv4 address at all.
    NeverAValidTarget,

    /// Neither an IPv4-carrying transition format nor a block that hosts one; the ordinary
    /// IPv6 rules decide.
    NotIpv4Carrying,
}

/// Classifies the IPv6 transition formats that let an IPv6 address name an IPv4 address, plus
/// the reserved block those formats live in.
///
/// The well-known prefix is only ever a /96 (RFC 6052 section 3.1), so its IPv4 address is
/// unambiguously the last 32 bits. Prefixes of other lengths place it elsewhere and cannot be
/// recognized from the address alone (RFC 6052 section 2.2), which is why the local-use
/// `64:ff9b:1::/48` (RFC 8215) is refused outright instead of decoded.
///
/// The `::/8` arm refuses a *block*, not an encoding, on purpose. Enumerating the encodings is
/// what let `::ffff:0:7f00:1` (SIIT) and `::5efe:7f00:1` (ISATAP) through: each is one more
/// spelling of 127.0.0.1 that the previous arm's `segments[5] == 0 || segments[5] == 0xffff`
/// test happened not to cover. Refusing the reserved block ends that class of miss.
///
/// **Arm order below is load-bearing.** `64:ff9b::/96` is itself inside `::/8`, so the
/// `Nat64WellKnown` arm must stay first; moving it below the block arm would refuse the whole
/// well-known prefix and break every webhook delivery on a NAT64/DNS64 deployment. Two things
/// catch that: the block arm is written as an unguarded range, so rustc reports the hoisted
/// `Nat64WellKnown` arm as an `unreachable_patterns` warning, and the
/// `the_nat64_well_known_prefix_inherits_the_verdict_of_the_ipv4_it_carries` proptest fails
/// loudly if the warning is ignored.
fn classify_ipv4_carrier(ip: &Ipv6Addr) -> Ipv4Carrier {
    match ip.segments() {
        // IPv4-IPv6 Translat. well-known prefix (`64:ff9b::/96`). Must stay first: this prefix
        // is inside the `::/8` block the last arm refuses outright.
        [0x64, 0xff9b, 0, 0, 0, 0, high, low] => {
            Ipv4Carrier::Nat64WellKnown(Ipv4Addr::from((u32::from(high) << 16) | u32::from(low)))
        }
        // Rest of IPv4-IPv6 Translat. (`64:ff9b::/32`): RFC 8215's local-use `64:ff9b:1::/48`
        // plus the unallocated remainder
        [0x64, 0xff9b, _, _, _, _, _, _]
        // 6to4 (`2002::/16`, RFC 3056), deprecated by RFC 7526: bits 16 to 47 are an arbitrary
        // IPv4 address and that is where the traffic ends up
        | [0x2002, _, _, _, _, _, _, _] => Ipv4Carrier::NeverAValidTarget,
        // Reserved by the IETF (`::/8`, RFC 4291 section 4). Global unicast is `2000::/3`, so
        // nothing routable was ever allocated here -- but every API-level way of writing an IPv4
        // address in IPv6 was: IPv4-Compatible (`::/96`, deprecated by RFC 4291 section 2.5.5.1),
        // IPv4-mapped (`::ffff:0:0/96`, RFC 4291 section 2.5.5.2), IPv4-translated
        // (`::ffff:0:0:0/96`, RFC 2765 SIIT, removed by RFC 6145) and ISATAP (`::5efe:0:0/96`,
        // RFC 5214). Each is malformed in an AAAA record and a redundant spelling of a plain
        // IPv4 URL, so the block goes rather than the list.
        //
        // Written as an unguarded range rather than `if s0 <= 0xff` so that the arm order is
        // checked by the compiler: a guarded arm never makes a later arm unreachable, so with a
        // guard here, hoisting this block above `Nat64WellKnown` would silently swallow the NAT64
        // carve-out. As a range it earns an `unreachable_patterns` warning instead. Hoisting it
        // above the `64:ff9b::/32` arm only flags a redundancy this block already subsumes.
        //
        // `::` and `::1` are refused here and nowhere else -- `is_forbidden_ip` dropped its
        // `is_unspecified` / `is_loopback` checks once this arm subsumed them. Any new carve-out
        // inside this block must re-check both, or it reopens loopback.
        [0x0000..=0x00ff, _, _, _, _, _, _, _] => Ipv4Carrier::NeverAValidTarget,
        _ => Ipv4Carrier::NotIpv4Carrying,
    }
}

/// Returns `true` when the given IP address must not be targeted by a webhook (loopback, private, link-local, shared, cloud-metadata, and other non-globally-reachable ranges).
fn is_forbidden_ip(ip: IpAddr) -> bool {
    // This should be replaced by https://doc.rust-lang.org/nightly/core/net/enum.IpAddr.html#method.is_global when it becomes stable
    //
    // ...with deliberate departures that must survive that replacement. `is_global` follows the
    // IANA special-purpose registry, where the NAT64 well-known prefix is globally reachable and
    // 6to4 is "N/A", so it answers `true` for both `64:ff9b::7f00:1` and `2002:7f00:1::1` -- two
    // ways of spelling 127.0.0.1. See [`classify_ipv4_carrier`].
    //
    // The whole of `::/8` is a departure too. `is_global`'s V6 arm excludes exactly one prefix
    // inside it -- IPv4-mapped `::ffff:0:0/96` -- so it answers `true` for `::7f00:1`
    // (IPv4-Compatible, `::/96`), `::ffff:0:7f00:1` (IPv4-translated, `::ffff:0:0:0/96`,
    // RFC 2765) and `::5efe:7f00:1` (ISATAP, RFC 5214): three more ways of spelling 127.0.0.1.
    // The block is reserved by the IETF (RFC 4291 section 4) and holds nothing routable, so
    // [`classify_ipv4_carrier`] refuses all of it except the `64:ff9b::/96` carve-out.
    //
    // Multicast and `192.88.99.0/24` are departures for the same reason: neither is on the list
    // `is_global` derives from. `224.0.0.0/4` and `ff00::/8` live in the multicast registries
    // rather than the special-purpose one, and `192.88.99.0/24` is marked deprecated there rather
    // than not-globally-reachable.

    // v4
    fn is_shared(ip: &Ipv4Addr) -> bool {
        ip.octets()[0] == 100 && (ip.octets()[1] & 0b1100_0000 == 0b0100_0000)
    }
    fn is_benchmarking(ip: &Ipv4Addr) -> bool {
        ip.octets()[0] == 198 && (ip.octets()[1] & 0xfe) == 18
    }
    fn is_reserved(ip: &Ipv4Addr) -> bool {
        ip.octets()[0] & 240 == 240 && !ip.is_broadcast()
    }

    // v6
    fn is_documentation(ip: &Ipv6Addr) -> bool {
        // Documentation (`2001:db8::/32`)
        ((ip.segments()[0] == 0x2001) && (ip.segments()[1] == 0xdb8))
            // Documentation (`3fff::/20`, RFC 9637)
            || ((ip.segments()[0] == 0x3fff) && (ip.segments()[1] < 0x1000))
    }
    fn is_unique_local(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] & 0xfe00) == 0xfc00
    }
    fn is_unicast_link_local(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] & 0xffc0) == 0xfe80
    }
    /// Site-Local Unicast (`fec0::/10`), deprecated by RFC 3879 but still used as internal
    /// addressing by legacy networks, and never globally reachable. Not covered by
    /// `is_unicast_link_local`: the same /10 mask yields `0xfec0`, not `0xfe80`.
    fn is_unicast_site_local(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] & 0xffc0) == 0xfec0
    }

    match ip {
        // Checked against the IANA IPv4 Special-Purpose Address Registry: no range it marks as
        // not globally reachable is missing below. This arm over-blocks in places, so that is a
        // superset claim rather than an exact match -- but it is what makes a future registry
        // addition re-verifiable instead of guessed at.
        IpAddr::V4(ip) => {
            ip.octets()[0] == 0 // "This network"
                || ip.is_private()
                || is_shared(&ip)
                || ip.is_loopback()
                || ip.is_link_local()
                // addresses reserved for future protocols (`192.0.0.0/24`)
                ||(ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 0)
                // 6to4 Relay Anycast (`192.88.99.0/24`, RFC 3068), deprecated by RFC 7526 -- the
                // IPv4 half of the `2002::/16` prefix `classify_ipv4_carrier` refuses
                || (ip.octets()[0] == 192 && ip.octets()[1] == 88 && ip.octets()[2] == 99)
                || ip.is_documentation()
                || is_benchmarking(&ip)
                || is_reserved(&ip)
                // Multicast (`224.0.0.0/4`, RFC 5771): never a unicast webhook target, mirroring
                // the `ff00::/8` rule in the V6 arm
                || ip.is_multicast()
                || ip.is_broadcast()
        }
        // A NAT64 well-known-prefix address *is* the IPv4 address it carries, so it inherits that
        // address's verdict. This recurses exactly once: the argument is an `IpAddr::V4` and the
        // V4 arm above never calls back in.
        IpAddr::V6(ip) => match classify_ipv4_carrier(&ip) {
            Ipv4Carrier::Nat64WellKnown(embedded) => is_forbidden_ip(IpAddr::V4(embedded)),
            Ipv4Carrier::NeverAValidTarget => true,
            // No `is_unspecified` or `is_loopback` check here: `::` and `::1` are both inside the
            // `::/8` block `classify_ipv4_carrier` refuses outright, so they never reach this
            // branch. Adding them back would be dead code that reads like the rule that stops
            // them, sending the next reader to the wrong place. The constraint that keeps this
            // true is recorded on that arm in `classify_ipv4_carrier`.
            Ipv4Carrier::NotIpv4Carrying => {
                // Discard-Only Address Block (`100::/64`)
                matches!(ip.segments(), [0x100, 0, 0, 0, _, _, _, _])
                // IETF Protocol Assignments (`2001::/23`)
                || (matches!(ip.segments(), [0x2001, b, _, _, _, _, _, _] if b < 0x200)
                    && !(
                        // Port Control Protocol Anycast (`2001:1::1`)
                        u128::from_be_bytes(ip.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
                        // Traversal Using Relays around NAT Anycast (`2001:1::2`)
                        || u128::from_be_bytes(ip.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
                        // AMT (`2001:3::/32`)
                        || matches!(ip.segments(), [0x2001, 3, _, _, _, _, _, _])
                        // AS112-v6 (`2001:4:112::/48`)
                        || matches!(ip.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
                        // ORCHIDv2 (`2001:20::/28`)
                        || matches!(ip.segments(), [0x2001, b, _, _, _, _, _, _] if (0x20..=0x2F).contains(&b))
                    ))
                // Segment Routing (SRv6) SIDs (`5f00::/16`, RFC 9602)
                || matches!(ip.segments(), [0x5f00, _, _, _, _, _, _, _])
                || is_documentation(&ip)
                || is_unique_local(&ip)
                || is_unicast_link_local(&ip)
                || is_unicast_site_local(&ip)
                // Multicast (`ff00::/8`, RFC 4291 section 2.7): never a unicast webhook target
                || ip.is_multicast()
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hickory_resolver::Resolver;
    use hickory_resolver::config::{NameServerConfig, ResolverConfig};
    use hickory_resolver::net::NoRecords;
    use hickory_resolver::net::runtime::TokioRuntimeProvider;
    use hickory_resolver::proto::ProtoError;
    use hickory_resolver::proto::op::{Query, ResponseCode};
    use hickory_resolver::proto::rr::{Name, RecordType};
    use proptest::prelude::*;
    use std::str::FromStr;
    use std::time::Instant;

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("invalid test IP")
    }

    fn url(s: &str) -> Url {
        Url::parse(s).expect("invalid test URL")
    }

    /// Builds a resolver whose queries can never leave the process: `ResolverConfig::default()`
    /// has an empty name server list, so any actual lookup fails immediately.
    fn offline_resolver(budget: Duration) -> DnsResolver {
        let inner = Resolver::builder_with_config(
            ResolverConfig::default(),
            TokioRuntimeProvider::default(),
        )
        .build()
        .expect("could not build the offline test resolver");
        DnsResolver {
            inner,
            budget,
            append_search_domains: false,
        }
    }

    fn test_options(budget: Duration) -> DnsResolverOptions {
        DnsResolverOptions {
            budget,
            positive_max_ttl: Duration::from_secs(300),
            negative_max_ttl: Duration::from_secs(30),
            ip_strategy: LookupIpStrategy::Ipv4AndIpv6,
            append_search_domains: false,
        }
    }

    #[test]
    fn pool_deadline_stays_under_the_budget() {
        assert_eq!(
            pool_deadline(Duration::from_secs(5)),
            Duration::from_millis(2250)
        );

        // The property that matters: one pool send costs up to `2 * opts.timeout`, and that has
        // to land strictly inside the budget, so hickory's own deadline fires before
        // `resolve_target`'s and the slow server's RTT is recorded instead of being discarded
        // with the cancelled future.
        //
        // Two caveats, both deliberate:
        //  - It holds for every budget at or above ~2.223ms, which is where the floor in
        //    `pool_deadline` stops binding (`floor(b_ns / 20) * 9 >= 1_000_000` <=>
        //    `b_ns >= 2_222_240`). Below that the floor wins and the outer `resolve_target`
        //    timeout is the only bound — no real lookup completes in under 2ms anyway.
        //  - It assumes one `lookup_ip` costs one pool send, so it describes the default
        //    configuration only. See `DNS_APPEND_SEARCH_DOMAINS`.
        for budget in [
            MIN_BUDGET,
            Duration::from_millis(10),
            Duration::from_millis(200),
            Duration::from_secs(5),
            Duration::from_secs(60),
        ] {
            assert!(
                pool_deadline(budget) * 2 < budget,
                "pool_deadline({budget:?}) = {:?} leaves no room for the 2x pool bound",
                pool_deadline(budget)
            );
        }
    }

    #[test]
    fn pool_deadline_is_never_zero() {
        // `Duration` division truncates, so an unclamped `budget / 20 * 9` collapses to zero
        // below 20ns. hickory reads a zero `opts.timeout` as `Instant::now() + 0`, i.e. an
        // already-expired deadline, so every query would fail instantly rather than being
        // unbounded.
        for budget in [
            Duration::from_nanos(1),
            Duration::from_nanos(20),
            Duration::from_millis(1),
        ] {
            assert!(!pool_deadline(budget).is_zero(), "budget {budget:?}");
        }
    }

    #[test]
    fn target_names_are_fully_qualified_by_default() {
        // A relative name gets expanded against resolv.conf's `search`, turning one `lookup_ip`
        // into one pool send *per suffix* and breaking `pool_deadline`'s budget. hickory's
        // `build_names` only short-circuits to a single name when `is_fqdn()`.
        assert!(target_name("hooks.example.com", false).unwrap().is_fqdn());

        // The dangerous case: otherwise `kubernetes` becomes
        // `kubernetes.default.svc.cluster.local`.
        assert!(target_name("kubernetes", false).unwrap().is_fqdn());

        // `https://example.com./` is a valid URL and `Url::domain()` keeps the trailing dot, so
        // already-qualified input must not gain an empty label.
        let already = target_name("example.com.", false).unwrap();
        assert!(already.is_fqdn());
        assert_eq!(already.num_labels(), 2); // root is not counted
        assert_eq!(already.to_string(), "example.com.");

        // The escape hatch leaves the name relative so hickory applies the search list.
        assert!(!target_name("hooks.example.com", true).unwrap().is_fqdn());

        // Either way the error carries the *original* host, so the customer-visible message is
        // unchanged and `an_unparseable_hostname_never_reaches_the_network` keeps passing.
        let host = ["averyveryverylonglabelusedtopadthisname"; 8].join(".");
        assert_eq!(
            target_name(&host, false).unwrap_err(),
            ResolveError::InvalidName(host)
        );
    }

    #[test]
    // `ResolverOpts` is `#[non_exhaustive]`, so it cannot be built with struct update syntax
    // from this crate; field reassignment is the only way to stage a hostile starting point.
    #[allow(clippy::field_reassign_with_default)]
    fn the_resolver_policy_fits_inside_the_budget() {
        // Stands in for a badly-tuned resolv.conf. `attempts` reissues the whole pool send and
        // `timeout` bounds one round, so honoring these would let one lookup run for minutes.
        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(30);
        opts.attempts = 5;

        let budget = Duration::from_secs(5);
        apply_options(&mut opts, &test_options(budget));

        // A single pool send, so `2 * opts.timeout` is the whole resolver worst case.
        assert_eq!(opts.attempts, 0);
        assert!(
            opts.timeout * 2 < budget,
            "resolver worst case {:?} must stay inside the {budget:?} budget",
            opts.timeout * 2
        );
        // resolv.conf's own timeout must not survive: the budget is the only input.
        assert_eq!(opts.timeout, pool_deadline(budget));
    }

    #[test]
    fn target_endpoint_uses_the_scheme_default_port() {
        assert_eq!(target_endpoint(&url("https://x.com/")).unwrap().1, 443);
        assert_eq!(target_endpoint(&url("http://x.com/")).unwrap().1, 80);
        assert_eq!(target_endpoint(&url("http://x.com:8443/")).unwrap().1, 8443);
    }

    #[test]
    fn target_endpoint_recognizes_ip_literals() {
        let v4 = url("http://127.0.0.1:8080/");
        let (host, port) = target_endpoint(&v4).unwrap();
        assert_eq!(host, Host::<&str>::Ipv4("127.0.0.1".parse().unwrap()));
        assert_eq!(port, 8080);

        // The IPv6 serialization is bracketed, so a `host_str()`-based implementation would try to
        // resolve "[::1]" as a DNS name and fail.
        let v6 = url("http://[::1]:8080/");
        let (host, port) = target_endpoint(&v6).unwrap();
        assert_eq!(host, Host::<&str>::Ipv6("::1".parse().unwrap()));
        assert_eq!(port, 8080);
    }

    #[test]
    fn target_endpoint_reports_a_missing_host_before_a_missing_port() {
        assert_eq!(
            target_endpoint(&url("foo://x.com/")).unwrap_err(),
            ResolveError::NoPort
        );
        assert_eq!(
            target_endpoint(&url("mailto:someone@example.com")).unwrap_err(),
            ResolveError::NoHost
        );
    }

    #[test]
    fn vet_addresses_rejects_an_empty_answer() {
        assert_eq!(
            vet_addresses(vec![], 443, false).unwrap_err(),
            ResolveError::NoAddress
        );
    }

    #[test]
    fn vet_addresses_applies_the_port_to_every_address() {
        let addrs = vet_addresses(vec![ip("1.1.1.1"), ip("2606:4700:4700::1111")], 8443, false)
            .expect("public addresses must pass");

        assert_eq!(
            addrs,
            vec![
                SocketAddr::new(ip("1.1.1.1"), 8443),
                SocketAddr::new(ip("2606:4700:4700::1111"), 8443),
            ]
        );
    }

    #[test]
    fn vet_addresses_rejects_a_mix_of_public_and_forbidden() {
        // The security invariant: one internal address poisons the whole answer.
        assert_eq!(
            vet_addresses(vec![ip("1.1.1.1"), ip("127.0.0.1")], 443, false).unwrap_err(),
            ResolveError::ForbiddenIp
        );
        assert_eq!(
            vet_addresses(vec![ip("1.1.1.1"), ip("fe80::1")], 443, false).unwrap_err(),
            ResolveError::ForbiddenIp
        );
        assert_eq!(
            vet_addresses(vec![ip("2606:4700:4700::1111"), ip("10.0.0.1")], 443, false)
                .unwrap_err(),
            ResolveError::ForbiddenIp
        );
    }

    #[test]
    fn detects_a_rejection_caused_only_by_ipv6() {
        // The `AI_ADDRCONFIG` regression: good A record, internal AAAA record.
        assert!(rejected_only_because_of_ipv6(&[
            ip("1.1.1.1"),
            ip("fc00::1")
        ]));

        // An internal IPv4 address means the target would have been rejected either way.
        assert!(!rejected_only_because_of_ipv6(&[
            ip("10.0.0.1"),
            ip("fc00::1")
        ]));
        // No usable IPv4 to fall back to, so dropping IPv6 would not help.
        assert!(!rejected_only_because_of_ipv6(&[ip("fc00::1")]));
        // Nothing was rejected at all.
        assert!(!rejected_only_because_of_ipv6(&[ip("1.1.1.1")]));

        // An AAAA record that *names an IPv4 address* is not the `AI_ADDRCONFIG` regression: it is
        // how an attacker spells an internal IPv4 address in IPv6. Reporting it as an IPv6-only
        // rejection would advertise `DNS_IP_STRATEGY=ipv4-only` as the fix, and following that
        // advice would hide the attempt rather than stop it.
        assert!(!rejected_only_because_of_ipv6(&[
            ip("1.1.1.1"),
            ip("64:ff9b::7f00:1")
        ]));
        assert!(!rejected_only_because_of_ipv6(&[
            ip("1.1.1.1"),
            ip("2002:a9fe:a9fe::1")
        ]));
        assert!(!rejected_only_because_of_ipv6(&[
            ip("1.1.1.1"),
            ip("::ffff:169.254.169.254")
        ]));
    }

    #[test]
    fn vet_addresses_keeps_every_address_when_the_check_is_disabled() {
        let addrs = vet_addresses(vec![ip("1.1.1.1"), ip("127.0.0.1")], 443, true)
            .expect("the check is disabled");

        // Including the forbidden one: this matches what the guard did before it moved here.
        assert_eq!(
            addrs,
            vec![
                SocketAddr::new(ip("1.1.1.1"), 443),
                SocketAddr::new(ip("127.0.0.1"), 443),
            ]
        );
    }

    #[test]
    fn map_net_error_separates_target_problems_from_ours() {
        let no_records = NoRecords::new(
            Query::query(Name::from_str("x.com.").unwrap(), RecordType::A),
            ResponseCode::NXDomain,
        );
        // A real answer from a server: the target genuinely has no address.
        assert_eq!(
            map_net_error(NetError::Dns(DnsError::NoRecordsFound(no_records))),
            ResolveError::NoAddress
        );

        // A server answered, but with an error code. The code is kept for the customer.
        assert_eq!(
            map_net_error(NetError::Dns(DnsError::ResponseCode(
                ResponseCode::ServFail
            ))),
            ResolveError::ServerFailure(ResponseCode::ServFail)
        );

        assert_eq!(map_net_error(NetError::Timeout), ResolveError::Timeout);

        // We never reached a server. Asserted with `matches!` rather than `assert_eq!` so the
        // test is not coupled to hickory's `Display` text for these variants.
        assert!(matches!(
            map_net_error(NetError::NoConnections),
            ResolveError::Internal { .. }
        ));

        // A malformed response is now unambiguously a wire error: `resolve_target` parses the
        // name itself, so `Proto` can no longer mean "the customer typed a bad hostname".
        assert!(matches!(
            map_net_error(NetError::Proto(ProtoError::from("malformed response"))),
            ResolveError::Internal { .. }
        ));
    }

    #[test]
    fn our_failures_report_e_dns() {
        // The regression test for the whole fix: a DNS-layer failure must not be reported as a
        // problem with the customer's URL.
        for e in [
            ResolveError::Timeout,
            ResolveError::ServerFailure(ResponseCode::ServFail),
            ResolveError::ServerFailure(ResponseCode::Refused),
            ResolveError::Internal {
                detail: "no connections available".to_owned(),
            },
        ] {
            assert_eq!(e.response_error().to_string(), "E_DNS", "{e:?}");
        }

        for e in [
            ResolveError::NoHost,
            ResolveError::NoPort,
            ResolveError::InvalidName("x y".to_owned()),
            ResolveError::NoAddress,
            ResolveError::ForbiddenIp,
        ] {
            assert_eq!(e.response_error().to_string(), "E_INVALID_TARGET", "{e:?}");
        }
    }

    #[test]
    fn detail_is_never_customer_visible() {
        let e = ResolveError::Internal {
            detail: "udp connect failed: host unreachable".to_owned(),
        };
        assert_eq!(e.detail(), Some("udp connect failed: host unreachable"));
        // The stored response body must not leak our resolver's internals.
        assert!(!e.to_string().contains("udp connect failed"));

        // Every other variant has nothing to hide, so a present `detail` field in a log line
        // unambiguously means "we could not reach a name server".
        for other in [
            ResolveError::Timeout,
            ResolveError::ServerFailure(ResponseCode::ServFail),
            ResolveError::NoAddress,
            ResolveError::ForbiddenIp,
            ResolveError::InvalidName("x y".to_owned()),
        ] {
            assert_eq!(other.detail(), None, "{other:?}");
        }
    }

    #[test]
    fn resolve_error_messages_are_stable() {
        // These strings are persisted as the response body of a failed request attempt, so they
        // are part of the API surface customers see.
        assert_eq!(ResolveError::NoHost.to_string(), "No host name in the URL");
        assert_eq!(
            ResolveError::NoPort.to_string(),
            "No port number in the URL"
        );
        assert_eq!(
            ResolveError::NoAddress.to_string(),
            "URL did not resolve to any IP address"
        );
        assert_eq!(
            ResolveError::ForbiddenIp.to_string(),
            "URL resolves to a forbidden IP"
        );
        assert_eq!(
            ResolveError::Timeout.to_string(),
            "Timed out resolving target's IP using DNS"
        );
        assert_eq!(
            ResolveError::InvalidName("x y".to_owned()).to_string(),
            "Could not resolve URL: x y is not a valid hostname"
        );
        assert_eq!(
            ResolveError::ServerFailure(ResponseCode::ServFail).to_string(),
            "Could not resolve URL: DNS server returned Server Failure"
        );
        // Generic on purpose: we cannot honestly attribute this failure, so we do not try.
        assert_eq!(
            ResolveError::Internal {
                detail: "boom".to_owned()
            }
            .to_string(),
            "Could not resolve URL: DNS resolution failed"
        );
    }

    #[tokio::test]
    async fn ip_literal_targets_do_not_hit_dns() {
        // The resolver has no name servers, so this only passes if the literal short-circuits.
        let resolver = offline_resolver(Duration::from_secs(5));

        let addrs = resolver
            .resolve_target(&url("http://93.184.216.34:8080/"), false)
            .await
            .expect("an IP-literal target needs no DNS");
        assert_eq!(addrs, vec![SocketAddr::new(ip("93.184.216.34"), 8080)]);

        let addrs = resolver
            .resolve_target(&url("http://[2606:4700:4700::1111]:443/"), false)
            .await
            .expect("an IPv6-literal target needs no DNS");
        assert_eq!(
            addrs,
            vec![SocketAddr::new(ip("2606:4700:4700::1111"), 443)]
        );

        // And the guard still applies to literals.
        assert_eq!(
            resolver
                .resolve_target(&url("http://127.0.0.1:8080/"), false)
                .await
                .unwrap_err(),
            ResolveError::ForbiddenIp
        );

        // A literal skips DNS entirely, so `DNS_IP_STRATEGY` cannot filter it out however it is
        // set: `is_forbidden_ip` is the only thing between the customer's URL and the metadata
        // service.
        assert_eq!(
            resolver
                .resolve_target(&url("http://[64:ff9b::a9fe:a9fe]:8080/"), false)
                .await
                .unwrap_err(),
            ResolveError::ForbiddenIp
        );
        assert_eq!(
            resolver
                .resolve_target(&url("http://[2002:7f00:1::1]:8080/"), false)
                .await
                .unwrap_err(),
            ResolveError::ForbiddenIp
        );
        assert_eq!(
            resolver
                .resolve_target(&url("http://[::ffff:0:7f00:1]:8080/"), false)
                .await
                .unwrap_err(),
            ResolveError::ForbiddenIp
        );

        // ...while the same prefix around a *public* IPv4 address stays a legitimate target,
        // because that is what DNS64 synthesizes for every IPv4-only target.
        let addrs = resolver
            .resolve_target(&url("http://[64:ff9b::5db8:d822]:443/"), false)
            .await
            .expect("the well-known prefix around a public IPv4 is a legitimate target");
        assert_eq!(addrs, vec![SocketAddr::new(ip("64:ff9b::5db8:d822"), 443)]);
    }

    #[tokio::test]
    async fn an_unreachable_resolver_fails_within_the_budget() {
        // This is the regression test for the bug that motivated the whole module: point the
        // resolver at TEST-NET-1, which is guaranteed unroutable, and check that the lookup gives
        // up on its own rather than hanging. A sandbox that refuses outbound UDP surfaces an I/O
        // error rather than a timeout, but both are now `E_DNS`, so the reported code can be
        // asserted directly even though the variant is environment-dependent.
        let budget = Duration::from_millis(200);
        let config =
            ResolverConfig::from_parts(None, vec![], vec![NameServerConfig::udp(ip("192.0.2.1"))]);
        let mut builder = Resolver::builder_with_config(config, TokioRuntimeProvider::default());
        // Configured exactly the way `DnsResolver::new` configures the real one, so this
        // exercises the shipped policy rather than a parallel one.
        apply_options(builder.options_mut(), &test_options(budget));
        let resolver = DnsResolver {
            inner: builder.build().expect("could not build the test resolver"),
            budget,
            append_search_domains: false,
        };

        let started = Instant::now();
        let result = timeout(
            Duration::from_secs(5),
            resolver.resolve_target(&url("http://example.com/"), false),
        )
        .await
        .expect("resolve_target must give up on its own, not hang until the outer timeout");

        let e = result.expect_err("an unroutable resolver cannot answer");
        assert_eq!(
            e.response_error().to_string(),
            "E_DNS",
            "our own resolver being unreachable must not be blamed on the target ({e:?})"
        );
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "resolution took {:?}, which is far beyond the {budget:?} budget",
            started.elapsed()
        );
    }

    #[tokio::test]
    async fn an_unparseable_hostname_never_reaches_the_network() {
        // `Url::parse` does not enforce the DNS length limits (it uses `DnsLength::Ignore`), so a
        // name over 255 bytes reaches us intact and only `Name::from_utf8` rejects it. The
        // resolver has no name servers, so this can only pass if the name is parsed *before* any
        // query is attempted.
        let resolver = offline_resolver(Duration::from_secs(5));
        let host = ["averyveryverylonglabelusedtopadthisname"; 8].join(".");
        let too_long = url(&format!("http://{host}/"));

        assert_eq!(
            resolver.resolve_target(&too_long, false).await.unwrap_err(),
            ResolveError::InvalidName(host)
        );
    }

    #[test]
    fn forbids_non_globally_reachable_ips() {
        // IPv4
        assert!(is_forbidden_ip(ip("0.0.0.0")));
        assert!(is_forbidden_ip(ip("127.0.0.1")));
        assert!(is_forbidden_ip(ip("10.0.0.1")));
        assert!(is_forbidden_ip(ip("172.16.5.4")));
        assert!(is_forbidden_ip(ip("192.168.1.1")));
        assert!(is_forbidden_ip(ip("100.64.0.1"))); // shared (CGNAT)
        assert!(is_forbidden_ip(ip("169.254.1.1"))); // link-local
        assert!(is_forbidden_ip(ip("169.254.169.254"))); // cloud metadata
        assert!(is_forbidden_ip(ip("192.88.99.1"))); // 6to4 relay anycast (RFC 7526)
        assert!(is_forbidden_ip(ip("224.0.0.1"))); // all-hosts multicast
        assert!(is_forbidden_ip(ip("239.255.255.250"))); // SSDP, administratively-scoped multicast
        assert!(is_forbidden_ip(ip("255.255.255.255"))); // broadcast
        // IPv6
        // `::/8` is refused as a block, so these are caught by that arm rather than by
        // `is_loopback` / `is_unspecified` -- which is why neither test lives in `is_forbidden_ip`
        assert!(is_forbidden_ip(ip("::1"))); // loopback
        assert!(is_forbidden_ip(ip("::"))); // unspecified
        assert!(is_forbidden_ip(ip("1::1"))); // unallocated remainder of `::/8`
        assert!(is_forbidden_ip(ip("0:0:0:1::1"))); // ditto, just past the `::/64` encodings
        assert!(is_forbidden_ip(ip("ff:ffff::1"))); // top of `::/8`
        assert!(is_forbidden_ip(ip("fc00::1"))); // unique local
        assert!(is_forbidden_ip(ip("fe80::1"))); // link-local
        assert!(is_forbidden_ip(ip("::ffff:127.0.0.1"))); // IPv4-mapped loopback
        assert!(is_forbidden_ip(ip("::ffff:169.254.169.254"))); // IPv4-mapped metadata
        assert!(is_forbidden_ip(ip("64:ff9b:1::1"))); // IPv4/IPv6 translation
        assert!(is_forbidden_ip(ip("100::1"))); // discard-only block
        assert!(is_forbidden_ip(ip("fec0::1"))); // deprecated site-local unicast
        assert!(is_forbidden_ip(ip("5f00::1"))); // SRv6 SIDs (RFC 9602)
        assert!(is_forbidden_ip(ip("ff02::1"))); // link-local all-nodes multicast
        assert!(is_forbidden_ip(ip("ff0e::1"))); // global-scope multicast is not a unicast target
        // IETF Protocol Assignments (`2001::/23`), excluding the globally-reachable carve-outs
        assert!(is_forbidden_ip(ip("2001::1"))); // generic 2001::/23 (Teredo region)
        assert!(is_forbidden_ip(ip("2001:1ff::1"))); // top of the /23 (b == 0x1ff)
        assert!(is_forbidden_ip(ip("2001:db8::1"))); // documentation
        assert!(is_forbidden_ip(ip("3fff::1"))); // documentation (RFC 9637)
        assert!(is_forbidden_ip(ip("3fff:fff::1"))); // top of `3fff::/20`
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        // `64:ff9b::<v4>` is exactly as forbidden as `<v4>` itself, over the whole IPv4 space.
        // Both halves of that equality matter: refusing the prefix outright would break every
        // delivery on a NAT64/DNS64 deployment, and allowing it outright lets an IPv6 literal
        // name 169.254.169.254.
        #[test]
        fn the_nat64_well_known_prefix_inherits_the_verdict_of_the_ipv4_it_carries(bits in any::<u32>()) {
            let v4 = Ipv4Addr::from(bits);
            let wkp = Ipv6Addr::from_bits((0x0064_ff9b_u128 << 96) | u128::from(bits));
            prop_assert_eq!(
                is_forbidden_ip(IpAddr::V6(wkp)),
                is_forbidden_ip(IpAddr::V4(v4)),
                "{} must inherit the verdict of {}",
                wkp,
                v4
            );
        }

        // The three blocks `classify_ipv4_carrier` refuses outright, one test each so a shrunk
        // counterexample names the invariant it broke. Hand-picked assertions record which
        // spelling maps to which IPv4 address; these record that no spelling was missed --
        // which is the property that enumerating encodings kept failing to deliver.

        // Every address reserved by the IETF (`::/8`, RFC 4291 section 2.4) is refused, whatever
        // encoding it happens to be: IPv4-Compatible, IPv4-mapped, SIIT IPv4-translated, ISATAP,
        // or none of them.
        #[test]
        fn every_address_in_the_ietf_reserved_block_is_refused(bits in any::<u128>().prop_map(|b| b >> 8)) {
            let ip = Ipv6Addr::from_bits(bits);
            // The block is not uniformly forbidden: `64:ff9b::/96` lives inside it and inherits
            // the verdict of the IPv4 address it carries, so `64:ff9b::5db8:d822` is allowed.
            prop_assume!(!matches!(ip.segments(), [0x64, 0xff9b, 0, 0, 0, 0, _, _]));
            prop_assert!(
                is_forbidden_ip(IpAddr::V6(ip)),
                "{} is inside `::/8` and must be refused",
                ip
            );
        }

        // Everything in `64:ff9b::/32` that is *not* the well-known /96 is refused: RFC 6052
        // section 2.2 puts the embedded IPv4 address at a position the address alone does not
        // reveal, so there is nothing to decode and inherit from.
        #[test]
        fn the_rest_of_the_nat64_prefix_is_refused(low in any::<u128>().prop_map(|b| b >> 32)) {
            prop_assume!(low >> 32 != 0); // that would be the well-known `64:ff9b::/96`
            let ip = Ipv6Addr::from_bits((0x0064_ff9b_u128 << 96) | low);
            prop_assert!(
                is_forbidden_ip(IpAddr::V6(ip)),
                "{} is in `64:ff9b::/32` but not the well-known prefix, so it must be refused",
                ip
            );
        }

        // 6to4 (`2002::/16`, RFC 3056) is refused unconditionally: bits 16 to 47 are an arbitrary
        // IPv4 address and that is where the traffic ends up, public or not.
        #[test]
        fn every_6to4_address_is_refused(low in any::<u128>().prop_map(|b| b >> 16)) {
            let ip = Ipv6Addr::from_bits((0x2002_u128 << 112) | low);
            prop_assert!(
                is_forbidden_ip(IpAddr::V6(ip)),
                "{} is inside `2002::/16` and must be refused",
                ip
            );
        }
    }

    /// The IPv6 transition formats let an IPv6 address name an arbitrary IPv4 address, so the
    /// IPv4 rules have to apply to whatever they carry. Otherwise `64:ff9b::7f00:1` and
    /// `2002:7f00:1::1` are two spellings of 127.0.0.1 that walk straight past the guard.
    #[test]
    fn forbids_ipv6_transition_addresses_that_carry_a_forbidden_ipv4() {
        // NAT64 well-known prefix (`64:ff9b::/96`): the carried IPv4 address decides
        assert!(is_forbidden_ip(ip("64:ff9b::a9fe:a9fe"))); // carries cloud metadata
        assert!(is_forbidden_ip(ip("64:ff9b::7f00:1"))); // carries loopback
        assert!(is_forbidden_ip(ip("64:ff9b::a00:1"))); // carries RFC 1918
        assert!(is_forbidden_ip(ip("64:ff9b::e000:1"))); // carries 224.0.0.1
        assert!(is_forbidden_ip(ip("64:ff9b::c058:6301"))); // carries 192.88.99.1
        assert!(is_forbidden_ip(ip("64:ff9b::"))); // carries 0.0.0.0, so the decoder fails closed
        // The rest of `64:ff9b::/32` is not the well-known prefix, so it is not decodable: the
        // embedded IPv4's position depends on a prefix length the address does not carry
        assert!(is_forbidden_ip(ip("64:ff9b:0:1::1"))); // unallocated remainder of the /32
        assert!(is_forbidden_ip(ip("64:ff9b:1::1"))); // RFC 8215 local use
        // 6to4 (`2002::/16`) carries an IPv4 address in bits 16 to 47 and is refused outright
        assert!(is_forbidden_ip(ip("2002:a9fe:a9fe::1"))); // carries cloud metadata
        assert!(is_forbidden_ip(ip("2002:7f00:1::1"))); // carries loopback
        assert!(is_forbidden_ip(ip("2002:5db8:d822::1"))); // refused even around a public IPv4
        // IPv4-Compatible Address (`::/96`), deprecated by RFC 4291
        assert!(is_forbidden_ip(ip("::7f00:1"))); // carries loopback
        assert!(is_forbidden_ip(ip("::a9fe:a9fe"))); // carries cloud metadata
        // IPv4-translated Address (`::ffff:0:0:0/96`, RFC 2765 SIIT, removed by RFC 6145):
        // refused with the rest of `::/8`, so what it carries is irrelevant
        assert!(is_forbidden_ip(ip("::ffff:0:7f00:1"))); // carries loopback
        assert!(is_forbidden_ip(ip("::ffff:0:a9fe:a9fe"))); // carries cloud metadata
        assert!(is_forbidden_ip(ip("::ffff:0:5db8:d822"))); // refused even around a public IPv4
        // ISATAP (`::5efe:0:0/96`, RFC 5214) over the `::/64` prefix
        assert!(is_forbidden_ip(ip("::5efe:7f00:1"))); // carries loopback
        assert!(is_forbidden_ip(ip("::5efe:a9fe:a9fe"))); // carries cloud metadata
    }

    #[test]
    fn allows_public_ips() {
        assert!(!is_forbidden_ip(ip("1.1.1.1")));
        assert!(!is_forbidden_ip(ip("8.8.8.8")));
        assert!(!is_forbidden_ip(ip("93.184.216.34"))); // example.com
        assert!(!is_forbidden_ip(ip("192.88.98.255"))); // just below `192.88.99.0/24`
        assert!(!is_forbidden_ip(ip("192.88.100.1"))); // just above `192.88.99.0/24`
        assert!(!is_forbidden_ip(ip("223.255.255.255"))); // last address below `224.0.0.0/4`
        assert!(!is_forbidden_ip(ip("2606:4700:4700::1111"))); // Cloudflare DNS
        assert!(!is_forbidden_ip(ip("2001:4860:4860::8888"))); // Google DNS
        // Just above `::/8`, so the block arm must not reach it. `100::/64` itself is the
        // discard-only block, which makes this the first address above `::/8` nothing forbids.
        assert!(!is_forbidden_ip(ip("100:0:0:1::1")));
        // The NAT64 carve-out *inside* `::/8`: this is what breaks if the `Nat64WellKnown` arm
        // ever stops being matched before the block arm.
        assert!(!is_forbidden_ip(ip("64:ff9b::5db8:d822")));
        // Globally-reachable carve-outs inside `2001::/23`
        assert!(!is_forbidden_ip(ip("2001:1::1"))); // Port Control Protocol Anycast
        assert!(!is_forbidden_ip(ip("2001:1::2"))); // TURN Anycast
        assert!(!is_forbidden_ip(ip("2001:3::1"))); // AMT
        assert!(!is_forbidden_ip(ip("2001:4:112::1"))); // AS112-v6
        assert!(!is_forbidden_ip(ip("2001:20::1"))); // ORCHIDv2 (low)
        assert!(!is_forbidden_ip(ip("2001:2f::1"))); // ORCHIDv2 (high)
        // First block just above `2001::/23` (b == 0x200) is globally reachable
        assert!(!is_forbidden_ip(ip("2001:200::1")));
        // First block just above `3fff::/20` (b == 0x1000) is globally reachable
        assert!(!is_forbidden_ip(ip("3fff:1000::1")));
        // Refusing the whole NAT64 well-known prefix would break every delivery on a NAT64/DNS64
        // deployment, where DNS64 synthesizes an address under it for every IPv4-only target.
        // Only the IPv4 address it carries may decide.
        assert!(!is_forbidden_ip(ip("64:ff9b::5db8:d822"))); // DNS64-synthesized example.com
        assert!(!is_forbidden_ip(ip("64:ff9b::101:101"))); // DNS64-synthesized 1.1.1.1
    }
}
