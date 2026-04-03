use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use super::HealthMonitorConfig;
use super::errors::HealthMonitorError;
use super::evaluation::SubscriptionHealth;
use super::notifications::{HealthAction, HealthActionInfo};
use super::types::{HealthEventSource, HealthStatus};

/// Evaluates a single subscription's health and determines state transitions.
///
/// State machine:
///   - `None` / `resolved` + high failure → insert `warning` event
///   - `warning` + even higher failure → disable subscription
///   - `warning` + low failure → insert `resolved` event
///   - `disabled` → no-op (manual re-enable required)
///   - `resolved` within cooldown → no-op (prevent email spam)
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

    // Persist current failure_percent to subscription table for frontend display
    sqlx::query("UPDATE webhook.subscription SET failure_percent = $1 WHERE subscription__id = $2")
        .bind(failure_percent)
        .bind(subscription.subscription_id)
        .execute(&mut **transaction)
        .await?;

    match last_status {
        Some(HealthStatus::Disabled) => {}

        Some(HealthStatus::Resolved)
            if last_at.is_some_and(|at| {
                (Utc::now() - at)
                    < chrono::Duration::from_std(config.warning_cooldown).unwrap_or_default()
            }) => {}

        Some(HealthStatus::Warning)
            if failure_percent >= warning_percent && failure_percent < disable_percent => {}

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

        Some(HealthStatus::Warning) => {
            let disabled_at = disable_subscription(transaction, subscription).await?;
            if let Some(at) = disabled_at {
                actions.push(HealthAction::Disabled(HealthActionInfo::from_subscription(
                    subscription,
                    Some(at),
                )));
            }
        }

        _ if failure_percent >= disable_percent => {
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
            let disabled_at = disable_subscription(transaction, subscription).await?;
            if let Some(at) = disabled_at {
                actions.push(HealthAction::Disabled(HealthActionInfo::from_subscription(
                    subscription,
                    Some(at),
                )));
            }
        }

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
/// Returns `Some(disabled_at)` if it actually disabled, `None` if already disabled.
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
