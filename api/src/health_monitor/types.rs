use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use strum::Display;

/// Health status of a subscription, stored in `subscription_health_event.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthStatus {
    Warning,
    Disabled,
    Resolved,
}

/// Source of a health event: automatic (system/health monitor) or manual (user/API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthEventSource {
    System,
    User,
}
