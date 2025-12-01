use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::{
    GlobalKeyExtractor, Governor, GovernorConfig, GovernorConfigBuilder, KeyExtractor,
};
use actix_web::HttpMessage;
use actix_web::middleware::Condition;
use actix_web::rt::time::sleep;
use biscuit_auth::Biscuit;
use log::{debug, trace, warn};
use std::net::IpAddr;
use std::time::Duration;

use crate::opentelemetry::report_rate_limiters_metrics;
use crate::problems::Hook0Problem;

#[derive(Debug, Clone)]
pub struct Hook0RateLimiters {
    disable_api_rate_limiting: bool,
    disable_api_rate_limiting_global: bool,
    disable_api_rate_limiting_ip: bool,
    disable_api_rate_limiting_token: bool,
    global: GovernorConfig<GlobalKeyExtractor, NoOpMiddleware>,
    ip: GovernorConfig<UserIpKeyExtractor, NoOpMiddleware>,
    token: GovernorConfig<TokenKeyExtractor, NoOpMiddleware>,
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
    ) -> Self {
        let global = GovernorConfigBuilder::default()
            .key_extractor(GlobalKeyExtractor)
            .burst_size(api_rate_limiting_global_burst_size)
            .milliseconds_per_request(api_rate_limiting_global_replenish_period_in_ms)
            .finish()
            .expect("Could not build global rate limiter; check configuration");
        let ip = GovernorConfigBuilder::default()
            .key_extractor(UserIpKeyExtractor)
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
        }

        Self {
            disable_api_rate_limiting,
            disable_api_rate_limiting_global,
            disable_api_rate_limiting_ip,
            disable_api_rate_limiting_token,
            global,
            ip,
            token,
        }
    }

    pub fn global(&self) -> Condition<Governor<GlobalKeyExtractor, NoOpMiddleware>> {
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

    pub fn spawn_housekeeping_task(&self, interval: Duration) {
        let self_clone = self.clone();
        actix_web::rt::spawn(async move {
            loop {
                sleep(interval).await;

                trace!("Removing old entries from rate limiters...");
                self_clone.ip.limiter().retain_recent();
                self_clone.token.limiter().retain_recent();

                trace!("Shrinking rate limiters internal's structures...");
                self_clone.ip.limiter().shrink_to_fit();
                self_clone.token.limiter().shrink_to_fit();

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

                report_rate_limiters_metrics(&dbg!([
                    ("ip", self_clone.ip.limiter().len()),
                    ("token", self_clone.token.limiter().len()),
                ]));
            }
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UserIpKeyExtractor;

impl KeyExtractor for UserIpKeyExtractor {
    type Key = IpAddr;
    type KeyExtractionError = Hook0Problem;

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        req.extensions()
            .get::<IpAddr>()
            .copied()
            .ok_or(Hook0Problem::InternalServerError)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TokenKeyExtractor;

impl KeyExtractor for TokenKeyExtractor {
    type Key = Vec<Vec<u8>>;
    type KeyExtractionError = Hook0Problem;

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        req.extensions()
            .get::<Biscuit>()
            .map(|biscuit| biscuit.revocation_identifiers())
            .ok_or(Hook0Problem::InternalServerError)
    }
}
