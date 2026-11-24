use actix_governor::governor::NotUntil;
use actix_governor::governor::clock::QuantaInstant;
use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::{Governor, GovernorConfig, GovernorConfigBuilder, KeyExtractor};
use actix_web::middleware::Condition;
use actix_web::rt::time::sleep;
use actix_web::{HttpMessage, HttpResponse, HttpResponseBuilder};
use http_api_problem::{HttpApiProblem, PROBLEM_JSON_MEDIA_TYPE};
use ipnetwork::Ipv6Network;
use std::net::IpAddr;
use std::time::Duration;
use tracing::{debug, trace, warn};

use crate::opentelemetry::report_rate_limiters_metrics;
use crate::problems::Hook0Problem;

/// Answer of every rate limiter once its quota is exhausted.
///
/// These limiters sit outside the handlers, so their response never goes through
/// [`Hook0Problem`]'s own [`actix_web::ResponseError`] implementation and has to be built here to
/// stay the RFC 7807 body every other Hook0 error answers with. The `Retry-After` header the
/// middleware already put on the builder is what tells a client how long to wait.
fn rate_limited_response(mut response: HttpResponseBuilder) -> HttpResponse {
    let problem: HttpApiProblem = Hook0Problem::RateLimited.into();
    response
        .content_type(PROBLEM_JSON_MEDIA_TYPE)
        .body(problem.json_bytes())
}

/// Keys every request the same way, so a single quota covers the whole instance.
///
/// [`actix_governor::GlobalKeyExtractor`] does exactly this, but a foreign type cannot be taught
/// to answer Hook0's problem body, and an SDK cannot type an error it is the only one not to
/// receive in that shape.
#[derive(Debug, Clone, Copy)]
pub struct InstanceKeyExtractor;

impl KeyExtractor for InstanceKeyExtractor {
    type Key = ();
    type KeyExtractionError = Hook0Problem;

    fn extract(
        &self,
        _req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        Ok(())
    }

    fn exceed_rate_limit_response(
        &self,
        _negative: &NotUntil<QuantaInstant>,
        response: HttpResponseBuilder,
    ) -> HttpResponse {
        rate_limited_response(response)
    }
}

#[derive(Debug, Clone)]
pub struct Hook0RateLimiters {
    disable_api_rate_limiting: bool,
    disable_api_rate_limiting_global: bool,
    disable_api_rate_limiting_ip: bool,
    disable_api_rate_limiting_token: bool,
    disable_api_rate_limiting_email: bool,
    global: GovernorConfig<InstanceKeyExtractor, NoOpMiddleware>,
    ip: GovernorConfig<UserIpKeyExtractor, NoOpMiddleware>,
    token: GovernorConfig<TokenKeyExtractor, NoOpMiddleware>,
    email: GovernorConfig<UserIpKeyExtractor, NoOpMiddleware>,
}

impl Hook0RateLimiters {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        disable_api_rate_limiting: bool,
        disable_api_rate_limiting_global: bool,
        api_rate_limiting_global_burst_size: u32,
        api_rate_limiting_global_replenish_period_in_ms: u64,
        disable_api_rate_limiting_ip: bool,
        api_rate_limiting_ip_burst_size: u32,
        api_rate_limiting_ip_replenish_period_in_ms: u64,
        disable_api_rate_limiting_token: bool,
        api_rate_limiting_token_burst_size: u32,
        api_rate_limiting_token_replenish_period_in_ms: u64,
        disable_api_rate_limiting_email: bool,
        api_rate_limiting_email_burst_size: u32,
        api_rate_limiting_email_replenish_period_in_ms: u64,
        api_rate_limiting_ipv6_prefix_length: u8,
    ) -> Self {
        let user_ip_key_extractor = UserIpKeyExtractor {
            ipv6_prefix_length: api_rate_limiting_ipv6_prefix_length,
        };
        let global = GovernorConfigBuilder::default()
            .key_extractor(InstanceKeyExtractor)
            .burst_size(api_rate_limiting_global_burst_size)
            .milliseconds_per_request(api_rate_limiting_global_replenish_period_in_ms)
            .finish()
            .expect("Could not build global rate limiter; check configuration");
        let ip = GovernorConfigBuilder::default()
            .key_extractor(user_ip_key_extractor)
            .burst_size(api_rate_limiting_ip_burst_size)
            .milliseconds_per_request(api_rate_limiting_ip_replenish_period_in_ms)
            .finish()
            .expect("Could not build per-IP rate limiter; check configuration");
        let token = GovernorConfigBuilder::default()
            .key_extractor(TokenKeyExtractor)
            .burst_size(api_rate_limiting_token_burst_size)
            .milliseconds_per_request(api_rate_limiting_token_replenish_period_in_ms)
            .finish()
            .expect("Could not build per-token rate limiter; check configuration");
        let email = GovernorConfigBuilder::default()
            .key_extractor(user_ip_key_extractor)
            .burst_size(api_rate_limiting_email_burst_size)
            .milliseconds_per_request(api_rate_limiting_email_replenish_period_in_ms)
            .finish()
            .expect("Could not build per-IP rate limiter for mail-sending endpoints; check configuration");

        if disable_api_rate_limiting {
            warn!("API rate limiting is disabled");
        } else {
            if disable_api_rate_limiting_global {
                warn!("Global API rate limiting is disabled");
            }
            if disable_api_rate_limiting_ip {
                warn!("Per-IP API rate limiting is disabled");
            }
            if disable_api_rate_limiting_token {
                warn!("Per-token API rate limiting is disabled");
            }
            if disable_api_rate_limiting_email {
                warn!("Per-IP API rate limiting of mail-sending endpoints is disabled");
            }
        }

        Self {
            disable_api_rate_limiting,
            disable_api_rate_limiting_global,
            disable_api_rate_limiting_ip,
            disable_api_rate_limiting_token,
            disable_api_rate_limiting_email,
            global,
            ip,
            token,
            email,
        }
    }

    pub fn global(&self) -> Condition<Governor<InstanceKeyExtractor, NoOpMiddleware>> {
        Condition::new(
            !self.disable_api_rate_limiting && !self.disable_api_rate_limiting_global,
            Governor::new(&self.global),
        )
    }

    pub fn ip(&self) -> Condition<Governor<UserIpKeyExtractor, NoOpMiddleware>> {
        Condition::new(
            !self.disable_api_rate_limiting && !self.disable_api_rate_limiting_ip,
            Governor::new(&self.ip),
        )
    }

    pub fn token(&self) -> Condition<Governor<TokenKeyExtractor, NoOpMiddleware>> {
        Condition::new(
            !self.disable_api_rate_limiting && !self.disable_api_rate_limiting_token,
            Governor::new(&self.token),
        )
    }

    /// Quota for the handful of endpoints that put a message in a mailbox the
    /// caller names. The whole `/api/v1` scope is already covered per IP, but
    /// that quota is sized for API traffic; an address-enumeration sweep is a
    /// few hundred requests, well under it, and every one of them can cost a
    /// mail. The per-account quotas in the database bound what one mailbox
    /// receives, but they see nothing of a caller walking thousands of distinct
    /// addresses — which is exactly what enumeration is. This is the quota that
    /// does.
    ///
    /// It answers 429, so it is visible to a caller. That is deliberate and
    /// costs nothing: what it reveals is the source address's own rate, never
    /// whether any account exists.
    pub fn email(&self) -> Condition<Governor<UserIpKeyExtractor, NoOpMiddleware>> {
        Condition::new(
            !self.disable_api_rate_limiting && !self.disable_api_rate_limiting_email,
            Governor::new(&self.email),
        )
    }

    pub fn spawn_housekeeping_task(&self, interval: Duration) {
        let self_clone = self.clone();
        actix_web::rt::spawn(async move {
            loop {
                sleep(interval).await;

                trace!("Removing old entries from rate limiters...");
                self_clone.ip.limiter().retain_recent();
                self_clone.token.limiter().retain_recent();
                self_clone.email.limiter().retain_recent();

                trace!("Shrinking rate limiters internal's structures...");
                self_clone.ip.limiter().shrink_to_fit();
                self_clone.token.limiter().shrink_to_fit();
                self_clone.email.limiter().shrink_to_fit();

                debug!("Rate limiters housekeeping done");
            }
        });
    }

    pub fn spawn_metrics_task(&self) {
        const INTERVAL: Duration = Duration::from_secs(15);
        let self_clone = self.clone();
        actix_web::rt::spawn(async move {
            loop {
                sleep(INTERVAL).await;

                report_rate_limiters_metrics(&[
                    ("ip", self_clone.ip.limiter().len()),
                    ("token", self_clone.token.limiter().len()),
                    ("email", self_clone.email.limiter().len()),
                ]);
            }
        });
    }
}

/// Keys a request on the address of its caller, as the user IP middleware resolved it.
#[derive(Debug, Clone, Copy)]
pub struct UserIpKeyExtractor {
    /// Length of the network prefix an IPv6 caller is counted on.
    ipv6_prefix_length: u8,
}

impl UserIpKeyExtractor {
    /// The key the quota of a caller at `ip` is held under.
    ///
    /// An IPv4 address is its own key. An IPv6 caller is keyed on its network prefix instead of
    /// its address: a single subscriber is routinely handed a /64 or more, and keying on the full
    /// address would give it one fresh quota per address it cares to use. An IPv4-mapped address
    /// is read as the IPv4 address it carries first, because the prefix those share would
    /// otherwise put every IPv4 caller of a dual-stack listener under one key.
    fn key(&self, ip: IpAddr) -> IpAddr {
        match ip.to_canonical() {
            IpAddr::V6(address) => Ipv6Network::new(address, self.ipv6_prefix_length)
                .map(|network| IpAddr::V6(network.network()))
                .unwrap_or(IpAddr::V6(address)),
            address => address,
        }
    }
}

impl KeyExtractor for UserIpKeyExtractor {
    type Key = IpAddr;
    type KeyExtractionError = Hook0Problem;

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        req.extensions()
            .get::<IpAddr>()
            .map(|ip| self.key(*ip))
            .ok_or(Hook0Problem::InternalServerError)
    }

    fn exceed_rate_limit_response(
        &self,
        _negative: &NotUntil<QuantaInstant>,
        response: HttpResponseBuilder,
    ) -> HttpResponse {
        rate_limited_response(response)
    }
}

/// Represents the key used for per-token rate limiting.
/// This allows different authentication methods to specify their rate limit identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateLimiterTokenKey {
    /// For regular Biscuit tokens - uses only the root revocation identifier
    /// (first block) to prevent bypassing rate limits via token attenuation
    BiscuitRootRevocationId(Vec<u8>),

    #[cfg(feature = "application-secret-compatibility")]
    /// For application secrets - uses the secret's UUID for stable identity
    ApplicationSecret(uuid::Uuid),

    /// For master API key - unique across all requests using it
    MasterApiKey,
}

#[derive(Debug, Clone, Copy)]
pub struct TokenKeyExtractor;

impl KeyExtractor for TokenKeyExtractor {
    type Key = RateLimiterTokenKey;
    type KeyExtractionError = Hook0Problem;

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        req.extensions()
            .get::<RateLimiterTokenKey>()
            .cloned()
            .ok_or(Hook0Problem::InternalServerError)
    }

    fn exceed_rate_limit_response(
        &self,
        _negative: &NotUntil<QuantaInstant>,
        response: HttpResponseBuilder,
    ) -> HttpResponse {
        rate_limited_response(response)
    }

    fn whitelisted_keys(&self) -> Vec<Self::Key> {
        vec![RateLimiterTokenKey::MasterApiKey]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(ipv6_prefix_length: u8, ip: &str) -> IpAddr {
        UserIpKeyExtractor { ipv6_prefix_length }.key(ip.parse().expect("a valid test address"))
    }

    fn address(ip: &str) -> IpAddr {
        ip.parse().expect("a valid test address")
    }

    #[test]
    fn ipv6_callers_of_one_network_share_a_key() {
        assert_eq!(
            key(64, "2001:db8:1:2::1"),
            key(64, "2001:db8:1:2:ffff:ffff:ffff:9")
        );
        assert_eq!(key(64, "2001:db8:1:2::1"), address("2001:db8:1:2::"));
        assert_eq!(key(56, "2001:db8:1:100::1"), key(56, "2001:db8:1:1ff::1"));
    }

    #[test]
    fn ipv6_callers_of_different_networks_do_not() {
        assert_ne!(key(64, "2001:db8:1:2::1"), key(64, "2001:db8:1:3::1"));
        assert_ne!(key(56, "2001:db8:1:100::1"), key(56, "2001:db8:1:200::1"));
    }

    #[test]
    fn a_full_length_prefix_keys_each_address_on_its_own() {
        assert_eq!(key(128, "2001:db8:1:2::1"), address("2001:db8:1:2::1"));
    }

    /// Masked as IPv6, every mapped address would fall under `::`, the one key all IPv4 callers of
    /// a dual-stack listener would then share.
    #[test]
    fn an_ipv4_mapped_address_is_keyed_as_the_ipv4_address_it_carries() {
        assert_eq!(key(64, "::ffff:198.51.100.1"), address("198.51.100.1"));
        assert_ne!(
            key(64, "::ffff:198.51.100.1"),
            key(64, "::ffff:198.51.100.2")
        );
    }

    #[test]
    fn an_ipv4_address_is_its_own_key() {
        assert_eq!(key(64, "198.51.100.1"), address("198.51.100.1"));
    }
}
