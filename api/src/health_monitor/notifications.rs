//! Health monitor side-effects: Hook0 events for state changes.
//!
//! Dispatched after the database transaction commits so a rollback never
//! leaves phantom events. All side-effects are best-effort — failures are
//! logged but never propagated.

use chrono::{DateTime, Utc};
use tracing::warn;
use uuid::Uuid;

use hook0_client::Hook0Client;

use crate::hook0_client::{
    EventSubscriptionDisabled, Hook0ClientEvent, RetrySchedulePayload, SubscriptionDisabledPayload,
};

use super::evaluation::SubscriptionHealth;

/// Describes a state transition produced by the state machine. Only `Disabled`
/// currently produces a Hook0 event side-effect; `Warning` and `Recovered` are
/// retained so the evaluation tests can assert state-machine behaviour.
#[allow(dead_code)]
pub enum HealthAction {
    /// Emitted when a subscription's failure rate crosses warning_failure_percent
    /// for the first time (or again after cooldown expires).
    Warning(HealthActionInfo),
    /// Emitted when a subscription's failure rate crosses disable_failure_percent —
    /// the subscription has been disabled in the same transaction.
    Disabled(HealthActionInfo),
    /// Emitted when a previously warned subscription's failure rate drops back
    /// below warning_failure_percent — the endpoint recovered on its own.
    Recovered(HealthActionInfo),
}

/// Data needed to dispatch Hook0 events outside the transaction.
#[allow(dead_code)]
pub struct HealthActionInfo {
    pub subscription_id: Uuid,
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub application_name: Option<String>,
    pub description: Option<String>,
    pub target_url: String,
    pub failure_percent: f64,
    /// `Some` only for Disabled — the timestamp when is_enabled was flipped to false.
    pub disabled_at: Option<DateTime<Utc>>,
    /// Included so consumers can see the retry config (schedule name, strategy, delays).
    pub retry_schedule: Option<RetrySchedulePayload>,
}

impl HealthActionInfo {
    /// Builds a `HealthActionInfo` from a `SubscriptionHealth` row and an optional disabled timestamp.
    pub fn from_subscription(
        subscription: &SubscriptionHealth,
        disabled_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            subscription_id: subscription.subscription_id,
            organization_id: subscription.organization_id,
            application_id: subscription.application_id,
            application_name: subscription.application_name.clone(),
            description: subscription.description.clone(),
            target_url: subscription.target_url.clone(),
            failure_percent: subscription.failure_percent,
            disabled_at,
            retry_schedule: subscription
                .retry_schedule_id
                .map(|id| RetrySchedulePayload {
                    retry_schedule_id: id,
                    name: subscription.retry_schedule_name.clone().unwrap_or_default(),
                    strategy: subscription.retry_strategy.clone().unwrap_or_default(),
                    max_retries: subscription.retry_max_retries.unwrap_or(0),
                    custom_intervals: subscription.retry_custom_intervals.clone(),
                    linear_delay: subscription.retry_linear_delay,
                    increasing_base_delay: subscription.retry_increasing_base_delay,
                    increasing_wait_factor: subscription.retry_increasing_wait_factor,
                }),
        }
    }
}

/// Dispatches Hook0 events for a list of health actions.
/// Called after the database transaction has been committed.
/// Only Disabled actions emit an event — warnings and recoveries stay internal
/// to avoid noise on transient fluctuations.
/// Failures are logged but never propagated — all side-effects are best-effort.
pub async fn dispatch_health_actions(actions: &[HealthAction], hook0_client: &Option<Hook0Client>) {
    for action in actions {
        if let HealthAction::Disabled(action_info) = action
            && let Some(client) = hook0_client
        {
            let disabled_at = action_info.disabled_at.unwrap_or_else(Utc::now);
            let event = EventSubscriptionDisabled {
                subscription: SubscriptionDisabledPayload {
                    subscription_id: action_info.subscription_id,
                    organization_id: action_info.organization_id,
                    application_id: action_info.application_id,
                    description: action_info.description.clone(),
                    target: action_info.target_url.clone(),
                    disabled_at,
                },
                retry_schedule: action_info.retry_schedule.clone(),
            };
            let hook0_event: Hook0ClientEvent = event.into();
            if let Err(e) = client.send_event(&hook0_event.mk_hook0_event()).await {
                warn!("Health monitor: failed to send subscription.disabled Hook0 event: {e}");
            }
        }
    }
}
