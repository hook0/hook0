use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::HealthMonitorConfig;

/// Subscription health data fetched from the database, including failure rate,
/// latest health event status, and retry schedule configuration.
#[derive(Debug, sqlx::FromRow)]
pub struct SubscriptionHealth {
    pub subscription_id: Uuid,
    pub application_id: Uuid,
    pub organization_id: Uuid,
    pub application_name: Option<String>,
    pub description: Option<String>,
    pub target_url: String,
    pub failure_percent: f64,
    #[allow(dead_code)]
    pub total: i64,
    pub last_health_status: Option<String>,
    pub last_health_at: Option<DateTime<Utc>>,
    pub last_health_source: Option<String>,
    #[allow(dead_code)]
    pub last_health_user_id: Option<Uuid>,
    pub retry_schedule_id: Option<Uuid>,
    pub retry_schedule_name: Option<String>,
    pub retry_strategy: Option<String>,
    pub retry_max_retries: Option<i32>,
    pub retry_custom_intervals: Option<Vec<i32>>,
    pub retry_linear_delay: Option<i32>,
    pub retry_increasing_base_delay: Option<i32>,
    pub retry_increasing_wait_factor: Option<f64>,
}

/// Fetches all enabled subscriptions with their failure rates and latest health event status.
///
/// The query computes a failure percentage using either:
/// - A sliding window of the last N messages (if enough messages exist)
/// - The full time window (if fewer than N messages)
///
/// Only subscriptions with at least `min_sample_size` attempts are evaluated.
pub async fn fetch_subscription_health_stats(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as f64;
    let min_sample = config.min_sample_size as i64;
    let message_window = config.message_window as i64;

    sqlx::query_as::<_, SubscriptionHealth>(
        r#"
        -- Step 1: Rank attempts per subscription (most recent first) within the time window
        with ranked_attempts as (
            select
                ra.subscription__id,
                ra.failed_at,
                row_number() over (partition by ra.subscription__id order by ra.created_at desc) as rn
            from webhook.request_attempt ra
            inner join webhook.subscription s on s.subscription__id = ra.subscription__id
            where s.is_enabled = true
              and s.deleted_at is null
              and ra.created_at > now() - make_interval(secs => $1)
              and (ra.succeeded_at is not null or ra.failed_at is not null)
        ),
        -- Step 2: Aggregate per subscription, compute failure % over the sliding window (last N) or full window
        windowed_stats as (
            select
                subscription__id,
                count(*) as total,
                case
                    when count(*) >= $3 then
                        count(*) filter (where failed_at is not null and rn <= $3)::float8
                        / $3::float8 * 100.0
                    else
                        count(*) filter (where failed_at is not null)::float8
                        / count(*)::float8 * 100.0
                end as failure_percent
            from ranked_attempts
            group by subscription__id
            having count(*) >= $2
        )
        -- Step 3: Join with subscription details, latest health event, retry schedule, and target URL
        select
            w.subscription__id as subscription_id,
            s.application__id as application_id,
            app.organization__id as organization_id,
            app.name as application_name,
            s.description,
            coalesce(th.url, '') as target_url,
            w.failure_percent,
            w.total,
            lh.status as last_health_status,
            lh.created_at as last_health_at,
            lh.source as last_health_source,
            lh.user__id as last_health_user_id,
            s.retry_schedule__id as retry_schedule_id,
            rs.name as retry_schedule_name,
            rs.strategy as retry_strategy,
            rs.max_retries as retry_max_retries,
            rs.custom_intervals as retry_custom_intervals,
            rs.linear_delay as retry_linear_delay,
            rs.increasing_base_delay as retry_increasing_base_delay,
            rs.increasing_wait_factor as retry_increasing_wait_factor
        from windowed_stats w
        inner join webhook.subscription s using (subscription__id)
        inner join event.application app on app.application__id = s.application__id
        left join lateral (
            select she.status, she.created_at, she.source, she.user__id
            from webhook.subscription_health_event she
            where she.subscription__id = w.subscription__id
            order by she.created_at desc
            limit 1
        ) lh on true
        left join webhook.retry_schedule rs on rs.retry_schedule__id = s.retry_schedule__id
        left join webhook.target_http th on th.target__id = s.target__id
        "#,
    )
    .bind(time_window_secs)
    .bind(min_sample)
    .bind(message_window)
    .fetch_all(&mut **tx)
    .await
}

/// Removes resolved health events older than the configured retention period,
/// keeping at least the latest event per subscription.
///
/// Example: a subscription with events at -100d (resolved), -80d (resolved), -10d (warning)
/// deletes only the -100d row; the -80d row is kept because -10d is newer.
pub async fn cleanup_resolved_health_events(
    db: &PgPool,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_period_days = config.retention_period_days as i32;

    let result = sqlx::query(
        r#"
        DELETE FROM webhook.subscription_health_event d
        WHERE d.created_at < now() - make_interval(days => $1)
          AND d.status = 'resolved'
          AND EXISTS (
            SELECT 1 FROM webhook.subscription_health_event newer
            WHERE newer.subscription__id = d.subscription__id
              AND newer.created_at > d.created_at
          )
        "#,
    )
    .bind(retention_period_days)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
