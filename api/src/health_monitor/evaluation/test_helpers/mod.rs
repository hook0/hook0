//! Shared test fixtures for the `evaluation/` sub-module tests.
//!
//! All items are `pub(in crate::health_monitor::evaluation)` so the sibling
//! test modules can reach them but nothing outside of `evaluation/` can.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::health_monitor::HealthMonitorConfig;
use crate::health_monitor::errors::HealthMonitorError;
use crate::health_monitor::queries;
use crate::health_monitor::queries::SubscriptionHealth;
use crate::health_monitor::state_machine::{PlannedAction, plan_for_subscription};
use crate::health_monitor::types::{HealthEventCause, HealthStatus};

mod fixtures;

pub(in crate::health_monitor::evaluation) use fixtures::insert_test_fixtures;

/// Test-only convenience wrapper: runs the pure state machine and applies its
/// `PlannedAction`s to the database inside `tx`, returning the planned actions
/// so tests can assert on what the state machine decided.
pub(in crate::health_monitor::evaluation) async fn process_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
    subscription: &SubscriptionHealth,
) -> Result<Vec<PlannedAction>, HealthMonitorError> {
    let now = Utc::now();
    let planned = plan_for_subscription(config, subscription, now);
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

pub(in crate::health_monitor::evaluation) fn test_config() -> HealthMonitorConfig {
    HealthMonitorConfig {
        interval: Duration::from_secs(60),
        warning_failure_percent: 50,
        disable_failure_percent: 90,
        time_window: Duration::from_secs(3600),
        min_sample_size: 1,
        warning_cooldown: Duration::from_secs(3600),
        retention_period_days: 30,
        bucket_duration: Duration::from_secs(300),
        bucket_max_messages: 100,
        bucket_retention_days: 30,
        max_delta_rows_per_tick: 50_000,
    }
}

/// Sets the cursor inside the given transaction.
pub(in crate::health_monitor::evaluation) async fn set_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ts: DateTime<Utc>,
) {
    sqlx::query!(
        "UPDATE webhook.health_monitor_cursor SET last_processed_at = $1 WHERE cursor__id = 1",
        ts,
    )
    .execute(&mut **tx)
    .await
    .unwrap();
}
