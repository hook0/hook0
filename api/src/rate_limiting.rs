use actix_governor::{
    GlobalKeyExtractor, Governor, GovernorConfig, GovernorConfigBuilder, KeyExtractor,
};
use actix_web::middleware::Condition;
use actix_web::HttpMessage;
use ipnetwork::IpNetwork;
use log::warn;

use crate::iam::{AuthProof, Hook0Claims};
use crate::problems::Hook0Problem;

#[derive(Debug, Clone)]
pub struct Hook0RateLimiters {
    disable_api_rate_limiting: bool,
    disable_api_rate_limiting_global: bool,
    disable_api_rate_limiting_ip: bool,
    disable_api_rate_limiting_token: bool,
    global: GovernorConfig<GlobalKeyExtractor>,
    ip: GovernorConfig<UserIpKeyExtractor>,
    token: GovernorConfig<TokenKeyExtractor>,
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
            .per_millisecond(api_rate_limiting_global_replenish_period_in_ms)
            .finish()
            .expect("Could not build global rate limiter; check configuration");
        let ip = GovernorConfigBuilder::default()
            .key_extractor(UserIpKeyExtractor)
            .burst_size(api_rate_limiting_ip_burst_size)
            .per_millisecond(api_rate_limiting_ip_replenish_period_in_ms)
            .finish()
            .expect("Could not build per-IP rate limiter; check configuration");
        let token = GovernorConfigBuilder::default()
            .key_extractor(TokenKeyExtractor)
            .burst_size(api_rate_limiting_token_burst_size)
            .per_millisecond(api_rate_limiting_token_replenish_period_in_ms)
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

    pub fn global(&self) -> Condition<Governor<GlobalKeyExtractor>> {
        Condition::new(
            !self.disable_api_rate_limiting && !self.disable_api_rate_limiting_global,
            Governor::new(&self.global),
        )
    }

    pub fn ip(&self) -> Condition<Governor<UserIpKeyExtractor>> {
        Condition::new(
            !self.disable_api_rate_limiting && !self.disable_api_rate_limiting_ip,
            Governor::new(&self.ip),
        )
    }

    pub fn token(&self) -> Condition<Governor<TokenKeyExtractor>> {
        Condition::new(
            !self.disable_api_rate_limiting && !self.disable_api_rate_limiting_token,
            Governor::new(&self.token),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UserIpKeyExtractor;

impl KeyExtractor for UserIpKeyExtractor {
    type Key = IpNetwork;
    type KeyExtractionError = Hook0Problem;

    fn name(&self) -> &'static str {
        "user IP"
    }

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        req.extensions()
            .get::<IpNetwork>()
            .copied()
            .ok_or(Hook0Problem::InternalServerError)
    }

    fn key_name(&self, key: &Self::Key) -> Option<String> {
        Some(key.to_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TokenKeyExtractor;

impl KeyExtractor for TokenKeyExtractor {
    type Key = AuthProof;
    type KeyExtractionError = Hook0Problem;

    fn name(&self) -> &'static str {
        "token"
    }

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        req.extensions()
            .get::<AuthProof>()
            .map(|proof| proof.to_owned())
            .ok_or(Hook0Problem::InternalServerError)
    }

    fn key_name(&self, key: &Self::Key) -> Option<String> {
        Some(match key {
            AuthProof::Jwt {
                claims:
                    Hook0Claims {
                        sub,
                        groups: _,
                        email: _,
                        given_name: _,
                        family_name: _,
                    },
            } => format!("jwt:{}", sub),
            AuthProof::ApplicationSecret {
                application_id,
                name: _,
                secret: _,
            } => format!("application_secret:{}", application_id),
        })
    }
}
