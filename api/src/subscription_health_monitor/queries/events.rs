//! Health event writes and resolved-event cleanup.

use sqlx::{PgPool, query};
use uuid::Uuid;

use super::super::runner::SubscriptionHealthMonitorConfig;
use super::super::types::{HealthEventCause, HealthStatus};

/// Inserts a health event row for a subscription.
///
/// `cause`: `Auto` = automatic (subscription health monitor), `Manual` = API
/// action (user, service token, or application secret). When cause is
/// `Manual` and user_id is `None`, the action was performed via a service
/// token or application secret.
pub async fn insert_health_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    status: HealthStatus,
    cause: HealthEventCause,
    user_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
            insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
            values ($1, $2, $3, $4)
        "#,
        subscription_id,
        status as HealthStatus,
        cause as HealthEventCause,
        user_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Removes resolved health events older than the configured retention, while
/// keeping at least the latest event per subscription.
///
/// Example: a subscription with events at -100d (resolved), -80d (resolved),
/// -10d (warning) only deletes the -100d row; the -80d row is kept because
/// -10d is newer.
pub async fn cleanup_resolved_health_events(
    db: &PgPool,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_secs = config.resolved_event_retention.as_secs_f64();

    let result = query!(
        r#"
            delete from webhook.subscription_health_event d
            where d.created_at < now() - make_interval(secs => $1)
              and d.status = 'resolved'
              and exists (
                  select 1 from webhook.subscription_health_event newer
                  where newer.subscription__id = d.subscription__id
                    and newer.created_at > d.created_at
              )
        "#,
        retention_secs,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
