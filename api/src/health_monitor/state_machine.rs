//! Subscription health state machine.
//!
//! Evaluates a subscription's current failure rate against its previous health
//! status and decides what action to take. The health monitor calls this once
//! per suspect subscription on each tick.
//!
//! State transitions:
//!   No previous state + high failure       -> emit Warning
//!   No previous state + very high failure   -> Disable (skip Warning, endpoint is already broken)
//!   Warning + still failing (same level)    -> do nothing (already warned)
//!   Warning + even higher failure           -> Disable
//!   Warning + recovered (failure dropped)   -> Resolved
//!   Disabled                                -> do nothing (user must re-enable manually)
//!   Resolved + within cooldown              -> do nothing (avoid email spam)

use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use super::HealthMonitorConfig;
use super::errors::HealthMonitorError;
use super::evaluation::SubscriptionHealth;
use super::notifications::{HealthAction, HealthActionInfo};
use super::types::{HealthEventSource, HealthStatus};

/// Evaluates a single subscription's health and determines what actions to take.
///
/// This is the core decision function. It looks at:
/// - The subscription's current failure_percent (from bucket aggregation)
/// - Its last health event (warning / disabled / resolved / none)
/// - The configured thresholds (warning_failure_percent, disable_failure_percent)
///
/// Side-effects (within the transaction):
/// - Caches failure_percent on the subscription table (avoids recomputing from buckets on every API read)
/// - Inserts health events (warning, disabled, resolved)
/// - Disables the subscription if failure is extreme
///
/// Returns a list of HealthActions that the caller dispatches as side-effects
/// (emails, Hook0 events) after the transaction commits.
pub async fn evaluate_health_transition(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
    subscription: &SubscriptionHealth,
) -> Result<Vec<HealthAction>, HealthMonitorError> {
    let failure_percent = subscription.failure_percent;
    let warning_percent = config.warning_failure_percent as f64;
    let disable_percent = config.disable_failure_percent as f64;
    let last_status = subscription.last_health_status;
    let last_at = subscription.last_health_at;

    let mut actions = Vec::new();

    // Always persist the current failure rate so the frontend can show it
    sqlx::query("UPDATE webhook.subscription SET failure_percent = $1 WHERE subscription__id = $2")
        .bind(failure_percent)
        .bind(subscription.subscription_id)
        .execute(&mut **transaction)
        .await?;

    match last_status {
        // Already disabled by the health monitor — user must re-enable manually.
        // We don't touch it again to avoid overriding a deliberate user action.
        Some(HealthStatus::Disabled) => {}

        // Recently resolved (within cooldown period) — skip to avoid spamming
        // the user with warning -> resolved -> warning -> resolved emails.
        Some(HealthStatus::Resolved)
            if last_at.is_some_and(|at| {
                (Utc::now() - at)
                    < chrono::Duration::from_std(config.warning_cooldown).unwrap_or_default()
            }) => {}

        // Already warned and failure rate is in the same range (still bad but
        // not bad enough to disable) — nothing new to tell the user.
        Some(HealthStatus::Warning)
            if failure_percent >= warning_percent && failure_percent < disable_percent => {}

        // Was warned but failure rate dropped below the warning threshold — the
        // endpoint recovered. Insert a "resolved" event and notify the user.
        Some(HealthStatus::Warning) if failure_percent < warning_percent => {
            // Skip recovery email if the last event was a manual user action (re-enable via API) —
            // the user already knows about it. Only send recovery email for system-originated events.
            insert_health_event(
                transaction,
                subscription.subscription_id,
                HealthStatus::Resolved,
                HealthEventSource::System,
                None,
            )
            .await?;
            if subscription.last_health_source != Some(HealthEventSource::User) {
                actions.push(HealthAction::Recovered(
                    HealthActionInfo::from_subscription(subscription, None),
                ));
            }
        }

        // Was warned and failure rate climbed above the disable threshold — shut it down.
        Some(HealthStatus::Warning) => {
            let disabled_at = disable_subscription(transaction, subscription).await?;
            if let Some(at) = disabled_at {
                actions.push(HealthAction::Disabled(HealthActionInfo::from_subscription(
                    subscription,
                    Some(at),
                )));
            }
        }

        // No previous health state (or resolved outside cooldown) and failure rate
        // is extremely high — disable immediately (no warning step, the endpoint is
        // already too broken to wait).
        _ if failure_percent >= disable_percent => {
            let disabled_at = disable_subscription(transaction, subscription).await?;
            if let Some(at) = disabled_at {
                actions.push(HealthAction::Disabled(HealthActionInfo::from_subscription(
                    subscription,
                    Some(at),
                )));
            }
        }

        // No previous health state and failure rate crossed the warning threshold —
        // send a warning email so the user can investigate before we disable.
        _ if failure_percent >= warning_percent => {
            insert_health_event(
                transaction,
                subscription.subscription_id,
                HealthStatus::Warning,
                HealthEventSource::System,
                None,
            )
            .await?;
            actions.push(HealthAction::Warning(HealthActionInfo::from_subscription(
                subscription,
                None,
            )));
        }

        // Failure rate below warning threshold (either no prior state or resolved outside cooldown) — healthy, nothing to do
        _ => {}
    }

    Ok(actions)
}

/// Inserts a health event row for a subscription.
///
/// `source`: `System` = automatic (health monitor), `User` = manual (API PUT).
/// When source is `User` and user_id is `None`, the action was via a service token.
pub async fn insert_health_event(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    status: HealthStatus,
    source: HealthEventSource,
    user_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO webhook.subscription_health_event (subscription__id, status, source, user__id) VALUES ($1, $2, $3, $4)",
    )
    .bind(subscription_id)
    .bind(status)
    .bind(source)
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Disables a subscription and inserts a 'disabled' health event atomically.
///
/// Uses a single CTE so that if the subscription was already disabled (e.g. by
/// the user between ticks), we don't insert a duplicate event. Returns
/// `Some(disabled_at)` only if we actually flipped is_enabled from true to false.
async fn disable_subscription(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription: &SubscriptionHealth,
) -> Result<Option<DateTime<Utc>>, HealthMonitorError> {
    let disabled_at: Option<DateTime<Utc>> = sqlx::query_scalar(
        r#"
        WITH updated AS (
            UPDATE webhook.subscription
            SET is_enabled = false
            WHERE subscription__id = $1 AND is_enabled = true
            RETURNING subscription__id
        ),
        inserted AS (
            INSERT INTO webhook.subscription_health_event (subscription__id, status, source, user__id)
            SELECT subscription__id, 'disabled', 'system', NULL FROM updated
            RETURNING created_at
        )
        SELECT created_at FROM inserted
        "#,
    )
    .bind(subscription.subscription_id)
    .fetch_optional(&mut **transaction)
    .await?;

    if disabled_at.is_some() {
        info!(
            "Health monitor: disabled subscription {}",
            subscription.subscription_id
        );
    }

    Ok(disabled_at)
}
