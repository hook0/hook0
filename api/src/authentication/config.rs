use chrono::{DateTime, Utc};
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Authentication type enum
#[derive(Debug, Clone, Serialize, Deserialize, Apiv2Schema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationType {
    OAuth2,
    Bearer,
    Certificate,
    Basic,
    Custom,
}

impl AuthenticationType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthenticationType::OAuth2 => "oauth2",
            AuthenticationType::Bearer => "bearer",
            AuthenticationType::Certificate => "certificate",
            AuthenticationType::Basic => "basic",
            AuthenticationType::Custom => "custom",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "oauth2" => Some(AuthenticationType::OAuth2),
            "bearer" => Some(AuthenticationType::Bearer),
            "certificate" => Some(AuthenticationType::Certificate),
            "basic" => Some(AuthenticationType::Basic),
            "custom" => Some(AuthenticationType::Custom),
            _ => None,
        }
    }
}

/// Authentication configuration stored in the database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct AuthenticationConfig {
    #[sqlx(rename = "authentication_config__id")]
    #[serde(rename = "authentication_config__id")]
    pub authentication_config_id: Uuid,
    #[sqlx(rename = "application__id")]
    #[serde(rename = "application__id")]
    pub application_id: Uuid,
    #[sqlx(rename = "subscription__id")]
    #[serde(rename = "subscription__id")]
    pub subscription_id: Option<Uuid>,
    #[sqlx(rename = "authentication_type__id")]
    #[serde(rename = "authentication_type__id")]
    pub authentication_type_id: Uuid,
    pub config: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Apiv2Schema)]
pub struct OAuth2Config {
    #[serde(rename = "grant_type")]
    pub grant_type: OAuth2GrantType,

    #[validate(length(min = 1, max = 255))]
    pub client_id: String,

    /// Either env://VARIABLE_NAME or encrypted value
    pub client_secret: String,

    #[validate(url)]
    pub token_endpoint: String,

    pub scopes: Option<Vec<String>>,

    /// Seconds before expiration to refresh token
    #[serde(default = "default_refresh_threshold")]
    pub token_refresh_threshold: u32,

    pub custom_headers: Option<serde_json::Value>,
}

fn default_refresh_threshold() -> u32 {
    300 // 5 minutes
}

#[derive(Debug, Clone, Serialize, Deserialize, Apiv2Schema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2GrantType {
    ClientCredentials,
    AuthorizationCode,
    Password,
}

/// Bearer token configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Apiv2Schema)]
pub struct BearerTokenConfig {
    /// Either env://VARIABLE_NAME or encrypted value
    pub token: String,

    #[serde(default = "default_header_name")]
    #[validate(length(min = 1, max = 100))]
    pub header_name: String,

    #[serde(default = "default_bearer_prefix")]
    #[validate(length(min = 0, max = 50))]
    pub prefix: String,
}

fn default_header_name() -> String {
    "Authorization".to_string()
}

fn default_bearer_prefix() -> String {
    "Bearer".to_string()
}

/// Certificate authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Apiv2Schema)]
pub struct CertificateConfig {
    /// Either env://VARIABLE_NAME or encrypted value
    pub client_cert: String,

    /// Either env://VARIABLE_NAME or encrypted value
    pub client_key: String,

    /// Optional CA certificate
    pub ca_cert: Option<String>,

    #[serde(default = "default_true")]
    pub verify_hostname: bool,

    #[serde(default = "default_true")]
    pub mtls: bool,
}

fn default_true() -> bool {
    true
}

/// Basic authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Apiv2Schema)]
pub struct BasicAuthConfig {
    #[validate(length(min = 1, max = 255))]
    pub username: String,

    /// Either env://VARIABLE_NAME or encrypted value
    pub password: String,
}

/// Custom authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Apiv2Schema)]
pub struct CustomAuthConfig {
    /// Custom headers to add
    pub headers: serde_json::Value,

    /// Custom query parameters to add
    pub query_params: Option<serde_json::Value>,
}

/// Encrypted secret stored in the database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, Apiv2Schema)]
pub struct EncryptedSecret {
    #[sqlx(rename = "encrypted_secret__id")]
    #[serde(rename = "encrypted_secret__id")]
    pub encrypted_secret_id: Uuid,
    #[sqlx(rename = "application__id")]
    #[serde(rename = "application__id")]
    pub application_id: Uuid,
    pub name: String,
    pub encrypted_value: String,
    pub nonce: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

/// OAuth token cache entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OAuthTokenCache {
    #[sqlx(rename = "oauth_token_cache__id")]
    #[serde(rename = "oauth_token_cache__id")]
    pub oauth_token_cache_id: Uuid,
    #[sqlx(rename = "authentication_config__id")]
    #[serde(rename = "authentication_config__id")]
    pub authentication_config_id: Uuid,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
}

/// Authentication audit log entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuthenticationAuditLog {
    #[sqlx(rename = "authentication_audit_log__id")]
    #[serde(rename = "authentication_audit_log__id")]
    pub authentication_audit_log_id: Uuid,
    #[sqlx(rename = "subscription__id")]
    #[serde(rename = "subscription__id")]
    pub subscription_id: Option<Uuid>,
    #[sqlx(rename = "request_attempt__id")]
    #[serde(rename = "request_attempt__id")]
    pub request_attempt_id: Option<Uuid>,
    pub authentication_type: String,
    pub is_success: bool,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// API request/response types
/// Authentication configuration for API requests
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Apiv2Schema)]
pub struct AuthenticationConfigRequest {
    #[serde(rename = "type")]
    pub auth_type: AuthenticationType,

    pub config: serde_json::Value,
}

/// Application authentication update request
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Apiv2Schema)]
pub struct ApplicationAuthenticationUpdate {
    pub authentication: Option<AuthenticationConfigRequest>,
}

/// Subscription authentication update request
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Apiv2Schema)]
pub struct SubscriptionAuthenticationUpdate {
    pub authentication: Option<AuthenticationConfigRequest>,
}
