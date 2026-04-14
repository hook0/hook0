//! Health event writes, suspect detection, and resolved-event cleanup.

use sqlx::{PgPool, query, query_scalar};
use uuid::Uuid;

use super::super::runner::SubscriptionHealthConfig;
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

/// Finds subscriptions that might need a health warning or to be disabled:
/// those with enough recent failures to cross the `min_deliveries_for_evaluation`
/// bar UNION those currently in warning status (so we can check if they've
/// recovered).
///
/// The UNION with warned subscriptions is critical: without it, a subscription
/// that was warned but then stopped failing would never get a "resolved" event
/// because it wouldn't appear in the bucket-based query anymore.
pub async fn find_subscriptions_pending_health_evaluation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthConfig,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let evaluation_window_secs = config.failure_rate_evaluation_window.as_secs_f64();
    let min_deliveries = i64::from(config.min_deliveries_for_evaluation);

    query_scalar!(
        r#"
            -- Subscriptions with enough delivery data to evaluate — only active, non-deleted ones.
            select shb.subscription__id as "subscription_id!"
            from webhook.subscription_health_bucket shb
            inner join webhook.subscription s on s.subscription__id = shb.subscription__id
            where shb.bucket_start > now() - make_interval(secs => $1)
              and s.is_enabled and s.deleted_at is null
            group by shb.subscription__id
            having sum(shb.failed_count) > $2
            union
            -- Subscriptions currently in 'warning' state — re-evaluated to detect recovery or escalation.
            select subscription__id
            from (
                select distinct on (subscription__id) subscription__id, status
                from webhook.subscription_health_event
                order by subscription__id, created_at desc
            ) latest
            where latest.status = 'warning'
        "#,
        evaluation_window_secs,
        min_deliveries,
    )
    .fetch_all(&mut **tx)
    .await
}

/// Removes resolved health events older than the configured retention, while
/// keeping at least the latest event per subscription.
///
/// Example: a subscription with events at -100d (resolved), -80d (resolved),
/// -10d (warning) only deletes the -100d row; the -80d row is kept because
/// -10d is newer.
pub async fn cleanup_resolved_health_events(
    db: &PgPool,
    config: &SubscriptionHealthConfig,
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
