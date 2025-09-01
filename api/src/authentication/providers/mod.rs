pub mod basic;
pub mod bearer;
pub mod certificate;
pub mod oauth2;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Request;
use uuid::Uuid;

use crate::authentication::config::AuthenticationType;

/// Trait for authentication providers
#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
    /// Apply authentication to an HTTP request
    async fn authenticate(&self, request: &mut Request) -> Result<()>;

    /// Refresh authentication if needed (e.g., OAuth2 token refresh)
    async fn refresh_if_needed(&self) -> Result<()>;

    /// Get the authentication type
    fn get_type(&self) -> AuthenticationType;

    /// Check if authentication needs refresh
    fn needs_refresh(&self) -> bool {
        false
    }
}

/// Authentication context for audit logging
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub subscription_id: Option<Uuid>,
    pub request_attempt_id: Option<Uuid>,
    pub auth_type: AuthenticationType,
}
