//! Subscription state writes and the `SubscriptionHealth` aggregate that the
//! state machine consumes.

use chrono::{DateTime, Utc};
use sqlx::{query, query_as, query_scalar};
use tracing::info;
use uuid::Uuid;

use super::super::runner::SubscriptionHealthMonitorConfig;
use super::super::types::{HealthEventCause, HealthStatus};

/// All the data the state machine needs to decide whether to warn, disable,
/// or resolve a subscription: its computed failure rate and its latest
/// health event.
#[derive(Debug)]
pub struct SubscriptionHealth {
    pub subscription_id: Uuid,
    pub failure_percent: f64,
    pub last_health_status: Option<HealthStatus>,
    pub last_health_at: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub last_health_cause: Option<HealthEventCause>,
    #[allow(dead_code)]
    pub last_health_user_id: Option<Uuid>,
}

/// Caches the current failure rate on the subscription row so API consumers
/// can read it without recomputing from buckets on every request.
pub async fn update_subscription_failure_percent(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    failure_percent: f64,
) -> Result<(), sqlx::Error> {
    query!(
        "update webhook.subscription set failure_percent = $1 where subscription__id = $2",
        failure_percent,
        subscription_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Clears `webhook.subscription.failure_percent` for subscriptions that are
/// no longer candidates for evaluation.
///
/// The column caches the latest computed failure rate; without this reset, a
/// subscription that spiked briefly and recovered would keep its stale rate
/// forever — we'd advertise a state that no longer matches reality.
pub async fn reset_healthy_failure_percent(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    active_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    let result = query!(
        r#"
            update webhook.subscription
            set failure_percent = null
            where failure_percent is not null
              and subscription__id <> all($1)
        "#,
        active_ids,
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

/// Disables a subscription and inserts a `disabled` health event atomically.
///
/// Uses a single CTE so that if the subscription was already disabled (e.g.
/// by the user between ticks), we don't insert a duplicate event. Returns
/// `Some(disabled_at)` only if we actually flipped `is_enabled` from true to
/// false.
pub async fn disable_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let disabled_at: Option<DateTime<Utc>> = query_scalar!(
        r#"
            with updated as (
                update webhook.subscription
                set is_enabled = false
                where subscription__id = $1 and is_enabled = true
                returning subscription__id
            ),
            inserted as (
                insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
                select subscription__id, 'disabled', 'auto', null from updated
                returning created_at
            )
            select created_at as "created_at!" from inserted
        "#,
        subscription_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    if disabled_at.is_some() {
        info!("Subscription health monitor: disabled subscription {subscription_id}");
    }

    Ok(disabled_at)
}

/// Computes each candidate subscription's failure rate over the sliding
/// evaluation window, joined with its latest health event.
///
/// Subscriptions with fewer total deliveries than `min_deliveries`
/// are excluded to avoid false positives on low-traffic endpoints.
pub async fn compute_failure_rates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    candidate_ids: &[Uuid],
    config: &SubscriptionHealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    let evaluation_window_secs = config.failure_rate_window.as_secs_f64();
    let min_deliveries = i64::from(config.min_deliveries);

    query_as!(
        SubscriptionHealth,
        r#"
            with bucket_stats as (
                select subscription__id,
                       sum(failed_count)::float8 / nullif(sum(total_count), 0) * 100.0 as failure_percent,
                       sum(total_count) as sample_size
                from webhook.subscription_health_bucket
                where subscription__id = any($1)
                  and bucket_start > now() - make_interval(secs => $2)
                group by subscription__id
                having sum(total_count) >= $3
            )
            select
                bs.subscription__id as "subscription_id!",
                bs.failure_percent as "failure_percent!",
                lh.status as "last_health_status?: HealthStatus",
                lh.created_at as "last_health_at?",
                lh.cause as "last_health_cause?: HealthEventCause",
                lh.user__id as "last_health_user_id?"
            from bucket_stats bs
            inner join webhook.subscription s using (subscription__id)
            left join lateral (
                select she.status, she.created_at, she.cause, she.user__id
                from webhook.subscription_health_event she
                where she.subscription__id = bs.subscription__id
                order by she.created_at desc
                limit 1
            ) lh on true
            where s.is_enabled = true and s.deleted_at is null
        "#,
        candidate_ids,
        evaluation_window_secs,
        min_deliveries,
    )
    .fetch_all(&mut **tx)
    .await
}
