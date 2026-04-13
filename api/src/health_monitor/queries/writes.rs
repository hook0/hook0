//! State-machine side effects: insert health events, disable subscriptions,
//! and cache the computed failure percent on the subscription row.

use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use super::super::types::{HealthEventCause, HealthStatus};

/// Caches the current failure rate on the subscription row so the frontend
/// can read it without recomputing from buckets on every API call.
pub async fn update_subscription_failure_percent(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    failure_percent: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE webhook.subscription SET failure_percent = $1 WHERE subscription__id = $2",
        failure_percent,
        subscription_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Inserts a health event row for a subscription.
///
/// `cause`: `Auto` = automatic (health monitor), `Manual` = API action (user,
/// service token, or application secret). When cause is `Manual` and user_id
/// is `None`, the action was via a service token or application secret.
pub async fn insert_health_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    status: HealthStatus,
    cause: HealthEventCause,
    user_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO webhook.subscription_health_event (subscription__id, status, cause, user__id) VALUES ($1, $2, $3, $4)",
        subscription_id,
        status.to_string(),
        cause.to_string(),
        user_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Disables a subscription and inserts a 'disabled' health event atomically.
///
/// Uses a single CTE so that if the subscription was already disabled (e.g.
/// by the user between ticks), we don't insert a duplicate event. Returns
/// `Some(disabled_at)` only if we actually flipped is_enabled from true to
/// false — the caller uses this to decide whether to send the disabled email.
pub async fn disable_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let disabled_at: Option<DateTime<Utc>> = sqlx::query_scalar!(
        r#"
        WITH updated AS (
            UPDATE webhook.subscription
            SET is_enabled = false
            WHERE subscription__id = $1 AND is_enabled = true
            RETURNING subscription__id
        ),
        inserted AS (
            INSERT INTO webhook.subscription_health_event (subscription__id, status, cause, user__id)
            SELECT subscription__id, 'disabled', 'auto', NULL FROM updated
            RETURNING created_at
        )
        SELECT created_at AS "created_at!" FROM inserted
        "#,
        subscription_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    if disabled_at.is_some() {
        info!("Health monitor: disabled subscription {subscription_id}");
    }

    Ok(disabled_at)
}
