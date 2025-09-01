use anyhow::{Result, anyhow};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use reqwest::Request;
use std::sync::Arc;
use uuid::Uuid;

use crate::authentication::{
    config::{AuthenticationType, BasicAuthConfig},
    encryption::SecretEncryption,
};

use super::AuthenticationProvider;

/// Basic authentication provider
pub struct BasicAuthProvider {
    config: BasicAuthConfig,
    application_id: Uuid,
    encryption: Arc<SecretEncryption>,
}

impl BasicAuthProvider {
    /// Create a new basic auth provider
    pub fn new(
        config: BasicAuthConfig,
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
impl AuthenticationProvider for BasicAuthProvider {
    async fn authenticate(&self, request: &mut Request) -> Result<()> {
        // Resolve the password (from env or encrypted storage)
        let password = self
            .encryption
            .resolve_secret(&self.config.password, &self.application_id)
            .await?;

        // Create Basic auth header value
        let credentials = format!("{}:{}", self.config.username, password);
        let encoded = BASE64.encode(credentials.as_bytes());
        let header_value = format!("Basic {}", encoded);

        // Add the Authorization header
        request.headers_mut().insert(
            "Authorization",
            header_value
                .parse()
                .map_err(|e| anyhow!("Invalid header value: {}", e))?,
        );

        Ok(())
    }

    async fn refresh_if_needed(&self) -> Result<()> {
        // Basic auth doesn't need refresh
        Ok(())
    }

    fn get_type(&self) -> AuthenticationType {
        AuthenticationType::Basic
    }
}
