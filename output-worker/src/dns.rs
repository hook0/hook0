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
        // Unlike glibc's `getaddrinfo`, hickory has no `AI_ADDRCONFIG`, so AAAA records are
        // returned even on a host with no global IPv6 address. A target whose A record is fine but
        // whose AAAA record points somewhere internal used to be delivered and is now rejected.
        // Log that case distinctly so its blast radius is measurable before anyone reports it.
        if rejected_only_because_of_ipv6(&ips) {
            warn!(
                ?ips,
                "Target rejected only because of its IPv6 addresses; its IPv4 addresses are globally reachable (set DNS_IP_STRATEGY=ipv4-only to ignore IPv6 records)"
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
    let all_offenders_are_ipv6 = offenders.peek().is_some() && offenders.all(|ip| ip.is_ipv6());
    let has_usable_ipv4 = ips.iter().any(|ip| ip.is_ipv4() && !is_forbidden_ip(*ip));

    all_offenders_are_ipv6 && has_usable_ipv4
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

/// Returns `true` when the given IP address must not be targeted by a webhook (loopback, private, link-local, shared, cloud-metadata, and other non-globally-reachable ranges).
fn is_forbidden_ip(ip: IpAddr) -> bool {
    // This should be replaced by https://doc.rust-lang.org/nightly/core/net/enum.IpAddr.html#method.is_global when it becomes stable

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
        (ip.segments()[0] == 0x2001) && (ip.segments()[1] == 0xdb8)
    }
    fn is_unique_local(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] & 0xfe00) == 0xfc00
    }
    fn is_unicast_link_local(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] & 0xffc0) == 0xfe80
    }

    match ip {
        IpAddr::V4(ip) => {
            ip.octets()[0] == 0 // "This network"
                || ip.is_private()
                || is_shared(&ip)
                || ip.is_loopback()
                || ip.is_link_local()
                // addresses reserved for future protocols (`192.0.0.0/24`)
                ||(ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 0)
                || ip.is_documentation()
                || is_benchmarking(&ip)
                || is_reserved(&ip)
                || ip.is_broadcast()
        }
        IpAddr::V6(ip) => {
            ip.is_unspecified()
                || ip.is_loopback()
                // IPv4-mapped Address (`::ffff:0:0/96`)
                || matches!(ip.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
                // IPv4-IPv6 Translat. (`64:ff9b:1::/48`)
                || matches!(ip.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
                // Discard-Only Address Block (`100::/64`)
                || matches!(ip.segments(), [0x100, 0, 0, 0, _, _, _, _])
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
                || is_documentation(&ip)
                || is_unique_local(&ip)
                || is_unicast_link_local(&ip)
        }
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
        assert!(is_forbidden_ip(ip("255.255.255.255"))); // broadcast
        // IPv6
        assert!(is_forbidden_ip(ip("::1"))); // loopback
        assert!(is_forbidden_ip(ip("::"))); // unspecified
        assert!(is_forbidden_ip(ip("fc00::1"))); // unique local
        assert!(is_forbidden_ip(ip("fe80::1"))); // link-local
        assert!(is_forbidden_ip(ip("::ffff:127.0.0.1"))); // IPv4-mapped loopback
        assert!(is_forbidden_ip(ip("::ffff:169.254.169.254"))); // IPv4-mapped metadata
        assert!(is_forbidden_ip(ip("64:ff9b:1::1"))); // IPv4/IPv6 translation
        assert!(is_forbidden_ip(ip("100::1"))); // discard-only block
        // IETF Protocol Assignments (`2001::/23`), excluding the globally-reachable carve-outs
        assert!(is_forbidden_ip(ip("2001::1"))); // generic 2001::/23 (Teredo region)
        assert!(is_forbidden_ip(ip("2001:1ff::1"))); // top of the /23 (b == 0x1ff)
        assert!(is_forbidden_ip(ip("2001:db8::1"))); // documentation
    }

    #[test]
    fn allows_public_ips() {
        assert!(!is_forbidden_ip(ip("1.1.1.1")));
        assert!(!is_forbidden_ip(ip("8.8.8.8")));
        assert!(!is_forbidden_ip(ip("93.184.216.34"))); // example.com
        assert!(!is_forbidden_ip(ip("2606:4700:4700::1111"))); // Cloudflare DNS
        assert!(!is_forbidden_ip(ip("2001:4860:4860::8888"))); // Google DNS
        // Globally-reachable carve-outs inside `2001::/23`
        assert!(!is_forbidden_ip(ip("2001:1::1"))); // Port Control Protocol Anycast
        assert!(!is_forbidden_ip(ip("2001:1::2"))); // TURN Anycast
        assert!(!is_forbidden_ip(ip("2001:3::1"))); // AMT
        assert!(!is_forbidden_ip(ip("2001:4:112::1"))); // AS112-v6
        assert!(!is_forbidden_ip(ip("2001:20::1"))); // ORCHIDv2 (low)
        assert!(!is_forbidden_ip(ip("2001:2f::1"))); // ORCHIDv2 (high)
        // First block just above `2001::/23` (b == 0x200) is globally reachable
        assert!(!is_forbidden_ip(ip("2001:200::1")));
    }
}
