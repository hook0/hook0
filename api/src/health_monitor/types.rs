//! Shared types used across evaluation, state machine, and notifications.

use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use strum::Display;

/// Health status of a subscription, stored in `subscription_health_event.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthStatus {
    /// Failure rate crossed the warning threshold — user has been notified but
    /// the subscription is still enabled.
    Warning,
    /// Failure rate crossed the disable threshold — the subscription's is_enabled
    /// has been set to false. Only a manual user action can re-enable it.
    Disabled,
    /// A previously warned subscription's failure rate dropped back below the
    /// warning threshold — the endpoint recovered on its own.
    Resolved,
}

/// Source of a health event: automatic (system/health monitor) or manual (user/API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthEventSource {
    /// Emitted by the health monitor background loop during automatic evaluation.
    System,
    /// Emitted when a user manually re-enables or disables a subscription via the API.
    User,
}
