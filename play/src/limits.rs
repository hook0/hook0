//! Rate limiting and abuse prevention configuration

use std::time::Duration;

/// Server limits configuration
#[derive(Debug, Clone)]
pub struct ServerLimits {
    /// Maximum payload size in bytes (default: 10MB)
    pub max_payload_size: usize,
    /// Maximum webhooks stored per token before FIFO eviction (default: 1000)
    pub max_webhooks_per_token: usize,
    /// Maximum total WebSocket connections (default: 10000)
    pub max_total_connections: usize,
    /// Maximum WebSocket connections per IP (default: 10)
    pub max_connections_per_ip: usize,
    /// Webhook TTL before automatic deletion (default: 24h)
    pub webhook_ttl: Duration,
    /// Session timeout - max duration for a WebSocket connection (default: 24h)
    pub session_timeout: Duration,
    /// Idle timeout - disconnect if no activity (default: 1h)
    pub idle_timeout: Duration,
    /// Maximum response body size in bytes (default: 10MB)
    pub max_response_body_size: usize,
    /// Per-IP webhook rate limit (requests per second) (default: 100)
    pub webhook_rate_limit_per_ip: u32,
    /// Per-token webhook rate limit (requests per second) (default: 50)
    pub webhook_rate_limit_per_token: u32,
    /// Global webhook rate limit (requests per second) (default: 10000)
    pub webhook_rate_limit_global: u32,
    /// Max invalid token attempts per IP per minute (default: 10)
    pub max_invalid_token_attempts: u32,
    /// Block duration after exceeding invalid token limit (default: 5min)
    pub invalid_token_block_duration: Duration,
    /// Enable encrypted storage (default: false)
    pub enable_encryption: bool,
    /// Encryption key (32 bytes for AES-256-GCM, auto-generated if empty)
    pub encryption_key: Option<Vec<u8>>,
    /// WebSocket handshake timeout (default: 30s)
    pub handshake_timeout: Duration,
}

impl Default for ServerLimits {
    fn default() -> Self {
        Self {
            max_payload_size: 10 * 1024 * 1024, // 10MB
            max_webhooks_per_token: 1000,
            max_total_connections: 10000,
            max_connections_per_ip: 10,
            webhook_ttl: Duration::from_secs(24 * 60 * 60), // 24h
            session_timeout: Duration::from_secs(24 * 60 * 60), // 24h
            idle_timeout: Duration::from_secs(60 * 60),     // 1h
            max_response_body_size: 10 * 1024 * 1024,       // 10MB
            webhook_rate_limit_per_ip: 100,
            webhook_rate_limit_per_token: 50,
            webhook_rate_limit_global: 10000,
            max_invalid_token_attempts: 10,
            invalid_token_block_duration: Duration::from_secs(5 * 60), // 5 min
            enable_encryption: false,
            encryption_key: None,
            handshake_timeout: Duration::from_secs(30),
        }
    }
}

/// Error response for limit violations
#[derive(Debug)]
pub enum LimitError {
    PayloadTooLarge { max: usize, actual: usize },
    TooManyConnections,
    TooManyConnectionsPerIp { ip: String, max: usize },
    RateLimited { retry_after_secs: u64 },
    InvalidTokenBlocked,
    ResponseTooLarge { max: usize, actual: usize },
    InvalidStatusCode { status: u16 },
}

impl std::fmt::Display for LimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PayloadTooLarge { max, actual } => {
                write!(
                    f,
                    "Payload too large: {} bytes (max: {} bytes)",
                    actual, max
                )
            }
            Self::TooManyConnections => {
                write!(f, "Too many connections")
            }
            Self::TooManyConnectionsPerIp { ip, max } => {
                write!(
                    f,
                    "Too many connections from IP {}: max {} allowed",
                    ip, max
                )
            }
            Self::RateLimited { retry_after_secs } => {
                write!(
                    f,
                    "Rate limit exceeded. Retry after {} seconds",
                    retry_after_secs
                )
            }
            Self::InvalidTokenBlocked => {
                write!(f, "Too many invalid token attempts. Temporarily blocked.")
            }
            Self::ResponseTooLarge { max, actual } => {
                write!(
                    f,
                    "Response body too large: {} bytes (max: {} bytes)",
                    actual, max
                )
            }
            Self::InvalidStatusCode { status } => {
                write!(f, "Invalid HTTP status code: {}", status)
            }
        }
    }
}
