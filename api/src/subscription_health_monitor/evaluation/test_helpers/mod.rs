//! Shared test fixtures for the evaluation sub-module tests.
//!
//! All items are `pub(in crate::subscription_health_monitor::evaluation)` so the
//! sibling test modules can reach them but nothing outside of `evaluation/`
//! can.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::subscription_health_monitor::SubscriptionHealthMonitorConfig;
use crate::subscription_health_monitor::errors::SubscriptionHealthMonitorError;
use crate::subscription_health_monitor::queries;
use crate::subscription_health_monitor::queries::SubscriptionHealth;
use crate::subscription_health_monitor::state_machine::{PlannedAction, plan_health_actions};
use crate::subscription_health_monitor::types::{HealthEventCause, HealthStatus};

mod fixtures;

pub(in crate::subscription_health_monitor::evaluation) use fixtures::insert_test_fixtures;

/// Test-only convenience wrapper: runs the pure state machine and applies
/// its `PlannedAction`s to the database inside `tx`, returning the planned
/// actions so tests can assert on what the state machine decided.
pub(in crate::subscription_health_monitor::evaluation) async fn process_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthMonitorConfig,
    subscription: &SubscriptionHealth,
) -> Result<Vec<PlannedAction>, SubscriptionHealthMonitorError> {
    let now = Utc::now();
    let planned = plan_health_actions(subscription, config, now);
    for action in &planned {
        match action {
            PlannedAction::UpdateFailurePercent => {
                queries::update_subscription_failure_percent(
                    tx,
                    subscription.subscription_id,
                    subscription.failure_percent,
                )
                .await?;
            }
            PlannedAction::EmitWarning => {
                queries::insert_health_event(
                    tx,
                    subscription.subscription_id,
                    HealthStatus::Warning,
                    HealthEventCause::Auto,
                    None,
                )
                .await?;
            }
            PlannedAction::EmitResolved => {
                queries::insert_health_event(
                    tx,
                    subscription.subscription_id,
                    HealthStatus::Resolved,
                    HealthEventCause::Auto,
                    None,
                )
                .await?;
            }
            PlannedAction::EmitDisabled => {
                let _ = queries::disable_subscription(tx, subscription.subscription_id).await?;
            }
        }
    }
    Ok(planned)
}

pub(in crate::subscription_health_monitor::evaluation) fn test_config()
-> SubscriptionHealthMonitorConfig {
    SubscriptionHealthMonitorConfig {
        interval: Duration::from_secs(60),
        failure_percent_for_warning: 50,
        failure_percent_for_disable: 90,
        failure_rate_window: Duration::from_secs(3600),
        min_deliveries: 1,
        anti_flap_window: Duration::from_secs(3600),
        resolved_event_retention: Duration::from_secs(30 * 86_400),
        bucket_duration: Duration::from_secs(300),
        bucket_max_messages: 100,
        bucket_retention: Duration::from_secs(30 * 86_400),
        max_request_attempts_per_tick: 50_000,
    }
}

/// Sets the cursor inside the given transaction.
pub(in crate::subscription_health_monitor::evaluation) async fn set_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ts: DateTime<Utc>,
) {
    sqlx::query!(
        "update webhook.subscription_health_monitor_cursor set last_processed_at = $1 where cursor__id = 1",
        ts,
    )
    .execute(&mut **tx)
    .await
    .unwrap();
}
