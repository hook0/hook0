//! Shared types used across evaluation, state machine, and notifications.

use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use strum::Display;

/// Health status of a subscription, stored in `subscription_health_event.status`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type,
)]
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

/// Cause of a health event: automatic (health monitor) or manual (API action).
///
/// Note: `Manual` covers all API-initiated actions, including those performed via a
/// service token or application secret where no human user__id is recorded.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthEventCause {
    /// Emitted by the health monitor background loop during automatic evaluation.
    Auto,
    /// Emitted when a subscription is changed via the API (by a user, service token, or application secret).
    Manual,
}
