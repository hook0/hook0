//! Audit logging for data access and security events

use tracing::info;

/// Audit event types
#[derive(Debug, Clone, Copy)]
pub enum AuditEvent {
    WebhookReceived,
    WebhookForwarded,
    WebhookViewed,
    WebhookDeleted,
    WebhookExpired,
    SessionCreated,
    SessionDisconnected,
    SessionTimedOut,
    SessionIdleTimeout,
    RateLimited,
    InvalidTokenAttempt,
    ConnectionRejected,
    InspectionApiAccess,
    PayloadTooLarge,
    HeaderSanitizationFailed,
    ResponseValidationFailed,
}

impl std::fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WebhookReceived => write!(f, "webhook_received"),
            Self::WebhookForwarded => write!(f, "webhook_forwarded"),
            Self::WebhookViewed => write!(f, "webhook_viewed"),
            Self::WebhookDeleted => write!(f, "webhook_deleted"),
            Self::WebhookExpired => write!(f, "webhook_expired"),
            Self::SessionCreated => write!(f, "session_created"),
            Self::SessionDisconnected => write!(f, "session_disconnected"),
            Self::SessionTimedOut => write!(f, "session_timed_out"),
            Self::SessionIdleTimeout => write!(f, "session_idle_timeout"),
            Self::RateLimited => write!(f, "rate_limited"),
            Self::InvalidTokenAttempt => write!(f, "invalid_token_attempt"),
            Self::ConnectionRejected => write!(f, "connection_rejected"),
            Self::InspectionApiAccess => write!(f, "inspection_api_access"),
            Self::PayloadTooLarge => write!(f, "payload_too_large"),
            Self::HeaderSanitizationFailed => write!(f, "header_sanitization_failed"),
            Self::ResponseValidationFailed => write!(f, "response_validation_failed"),
        }
    }
}

/// Log an audit event with structured data
pub fn log_audit(event: AuditEvent, token: Option<&str>, ip: Option<&str>, details: &str) {
    info!(
        audit = true,
        event = %event,
        token = token.unwrap_or("-"),
        ip = ip.unwrap_or("-"),
        details = details,
        "AUDIT"
    );
}
