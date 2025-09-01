use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Request;
use std::sync::Arc;
use uuid::Uuid;

use crate::authentication::{
    config::{AuthenticationType, BearerTokenConfig},
    encryption::SecretEncryption,
};

use super::AuthenticationProvider;

/// Bearer token authentication provider
pub struct BearerTokenProvider {
    config: BearerTokenConfig,
    application_id: Uuid,
    encryption: Arc<SecretEncryption>,
}

impl BearerTokenProvider {
    /// Create a new bearer token provider
    pub fn new(
        config: BearerTokenConfig,
        application_id: Uuid,
        encryption: Arc<SecretEncryption>,
    ) -> Self {
        Self {
            config,
            application_id,
            encryption,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for BearerTokenProvider {
    async fn authenticate(&self, request: &mut Request) -> Result<()> {
        // Resolve the token (from env or encrypted storage)
        let token = self
            .encryption
            .resolve_secret(&self.config.token, &self.application_id)
            .await?;

        // Build the header value
        let header_value = if self.config.prefix.is_empty() {
            token
        } else {
            format!("{} {}", self.config.prefix, token)
        };

        // Add the authentication header
        use reqwest::header::HeaderName;
        let header_name = HeaderName::from_bytes(self.config.header_name.as_bytes())
            .map_err(|e| anyhow!("Invalid header name: {}", e))?;
        request.headers_mut().insert(
            header_name,
            header_value
                .parse()
                .map_err(|e| anyhow!("Invalid header value: {}", e))?,
        );

        Ok(())
    }

    async fn refresh_if_needed(&self) -> Result<()> {
        // Bearer tokens don't need refresh
        Ok(())
    }

    fn get_type(&self) -> AuthenticationType {
        AuthenticationType::Bearer
    }
}
