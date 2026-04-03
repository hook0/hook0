use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Health status of a subscription, stored in `subscription_health_event.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Apiv2Schema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthStatus {
    Warning,
    Disabled,
    Resolved,
}

/// Source of a health event: automatic (system/health monitor) or manual (user/API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Apiv2Schema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthEventSource {
    System,
    User,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warning => write!(f, "warning"),
            Self::Disabled => write!(f, "disabled"),
            Self::Resolved => write!(f, "resolved"),
        }
    }
}

impl fmt::Display for HealthEventSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::User => write!(f, "user"),
        }
    }
}
