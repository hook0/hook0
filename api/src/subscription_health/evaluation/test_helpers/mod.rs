//! Shared test fixtures for the evaluation sub-module tests.
//!
//! All items are `pub(in crate::subscription_health::evaluation)` so the
//! sibling test modules can reach them but nothing outside of `evaluation/`
//! can.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::subscription_health::SubscriptionHealthConfig;
use crate::subscription_health::errors::SubscriptionHealthError;
use crate::subscription_health::queries;
use crate::subscription_health::queries::SubscriptionHealth;
use crate::subscription_health::state_machine::{PlannedAction, evaluate_health_transition};
use crate::subscription_health::types::{HealthEventCause, HealthStatus};

mod fixtures;

pub(in crate::subscription_health::evaluation) use fixtures::insert_test_fixtures;

/// Test-only convenience wrapper: runs the pure state machine and applies
/// its `PlannedAction`s to the database inside `tx`, returning the planned
/// actions so tests can assert on what the state machine decided.
pub(in crate::subscription_health::evaluation) async fn process_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthConfig,
    subscription: &SubscriptionHealth,
) -> Result<Vec<PlannedAction>, SubscriptionHealthError> {
    let now = Utc::now();
    let planned = evaluate_health_transition(subscription, config, now);
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

pub(in crate::subscription_health::evaluation) fn test_config() -> SubscriptionHealthConfig {
    SubscriptionHealthConfig {
        interval: Duration::from_secs(60),
        warning_failure_percent: 50,
        disable_failure_percent: 90,
        failure_rate_evaluation_window: Duration::from_secs(3600),
        min_deliveries_for_evaluation: 1,
        anti_flap_window: Duration::from_secs(3600),
        resolved_event_retention: Duration::from_secs(30 * 86_400),
        bucket_duration: Duration::from_secs(300),
        bucket_max_messages: 100,
        bucket_retention_days: 30,
        max_request_attempts_scanned_per_tick: 50_000,
    }
}

/// Sets the cursor inside the given transaction.
pub(in crate::subscription_health::evaluation) async fn set_cursor(
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
