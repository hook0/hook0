use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::HealthMonitorConfig;
use super::types::{HealthEventSource, HealthStatus};

/// Delta row returned by the watermark-based scan of request_attempt.
#[derive(Debug, sqlx::FromRow)]
pub struct DeltaRow {
    pub subscription_id: Uuid,
    pub total: i64,
    pub failed: i64,
    pub max_completed_at: Option<DateTime<Utc>>,
}

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
    pub last_health_status: Option<HealthStatus>,
    pub last_health_at: Option<DateTime<Utc>>,
    pub last_health_source: Option<HealthEventSource>,
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

/// Reads the current watermark timestamp from the singleton table.
pub async fn read_watermark(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT last_processed_at FROM webhook.health_monitor_watermark WHERE watermark__id = 1",
    )
    .fetch_one(&mut **tx)
    .await
}

/// Ingests deltas from request_attempt since the given watermark (capped at 50 000 rows).
pub async fn ingest_deltas(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    watermark: DateTime<Utc>,
) -> Result<Vec<DeltaRow>, sqlx::Error> {
    sqlx::query_as::<_, DeltaRow>(
        r#"
        WITH capped AS (
            SELECT subscription__id, failed_at, succeeded_at
            FROM webhook.request_attempt
            WHERE COALESCE(succeeded_at, failed_at) > $1
              AND (succeeded_at IS NOT NULL OR failed_at IS NOT NULL)
            ORDER BY COALESCE(succeeded_at, failed_at)
            LIMIT 50000
        )
        SELECT subscription__id AS subscription_id,
               COUNT(*) AS total,
               COUNT(failed_at) AS failed,
               MAX(COALESCE(succeeded_at, failed_at)) AS max_completed_at
        FROM capped
        GROUP BY subscription__id
        "#,
    )
    .bind(watermark)
    .fetch_all(&mut **tx)
    .await
}

/// Upserts delta counts into open health buckets (bulk via unnest).
pub async fn upsert_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    deltas: &[DeltaRow],
) -> Result<(), sqlx::Error> {
    let sub_ids: Vec<Uuid> = deltas.iter().map(|d| d.subscription_id).collect();
    let totals: Vec<i32> = deltas
        .iter()
        .map(|d| d.total.min(i32::MAX as i64) as i32)
        .collect();
    let faileds: Vec<i32> = deltas
        .iter()
        .map(|d| d.failed.min(i32::MAX as i64) as i32)
        .collect();

    sqlx::query(
        r#"
        WITH open_buckets AS (
            SELECT subscription__id, bucket_start
            FROM webhook.subscription_health_bucket
            WHERE subscription__id = ANY($1)
              AND bucket_end IS NULL
        ),
        deltas AS (
            SELECT * FROM unnest($1::uuid[], $2::int[], $3::int[])
                AS t(subscription__id, total, failed)
        )
        INSERT INTO webhook.subscription_health_bucket
            (subscription__id, bucket_start, total_count, failed_count)
        SELECT d.subscription__id,
               COALESCE(ob.bucket_start, now()),
               d.total,
               d.failed
        FROM deltas d
        LEFT JOIN open_buckets ob USING (subscription__id)
        ON CONFLICT (subscription__id, bucket_start)
        DO UPDATE SET
            total_count = subscription_health_bucket.total_count + EXCLUDED.total_count,
            failed_count = subscription_health_bucket.failed_count + EXCLUDED.failed_count
        "#,
    )
    .bind(&sub_ids)
    .bind(&totals)
    .bind(&faileds)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Closes buckets that have exceeded their duration or message cap.
pub async fn close_full_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let bucket_duration_secs = config.bucket_duration.as_secs() as f64;
    let bucket_max_messages = config.bucket_max_messages as i32;

    let result = sqlx::query(
        r#"
        UPDATE webhook.subscription_health_bucket
        SET bucket_end = now()
        WHERE bucket_end IS NULL
          AND (bucket_start < now() - make_interval(secs => $1)
               OR total_count >= $2)
        "#,
    )
    .bind(bucket_duration_secs)
    .bind(bucket_max_messages)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

/// Finds suspect subscription IDs: those with high failure counts in recent
/// buckets UNION those currently in warning status.
pub async fn find_suspects(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as f64;
    let min_sample = config.min_sample_size as i64;

    sqlx::query_scalar(
        r#"
        SELECT subscription__id
        FROM webhook.subscription_health_bucket
        WHERE bucket_start > now() - make_interval(secs => $1)
        GROUP BY subscription__id
        HAVING SUM(failed_count) > $2
        UNION
        SELECT DISTINCT she.subscription__id
        FROM webhook.subscription_health_event she
        WHERE she.status = 'warning'
          AND NOT EXISTS (
            SELECT 1 FROM webhook.subscription_health_event newer
            WHERE newer.subscription__id = she.subscription__id
              AND newer.created_at > she.created_at
          )
        "#,
    )
    .bind(time_window_secs)
    .bind(min_sample)
    .fetch_all(&mut **tx)
    .await
}

/// Computes failure rates for suspect subscriptions, joining subscription
/// details, latest health event, and retry schedule configuration.
pub async fn compute_failure_rates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    suspect_ids: &[Uuid],
    config: &HealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as f64;
    let min_sample = config.min_sample_size as i64;

    sqlx::query_as::<_, SubscriptionHealth>(
        r#"
        WITH bucket_stats AS (
            SELECT subscription__id,
                   SUM(failed_count)::float8 / NULLIF(SUM(total_count), 0) * 100.0 AS failure_percent,
                   SUM(total_count) AS sample_size
            FROM webhook.subscription_health_bucket
            WHERE subscription__id = ANY($1)
              AND bucket_start > now() - make_interval(secs => $2)
            GROUP BY subscription__id
            HAVING SUM(total_count) >= $3
        )
        SELECT
            bs.subscription__id AS subscription_id,
            s.application__id AS application_id,
            app.organization__id AS organization_id,
            app.name AS application_name,
            s.description,
            coalesce(th.url, '') AS target_url,
            bs.failure_percent,
            lh.status AS last_health_status,
            lh.created_at AS last_health_at,
            lh.source AS last_health_source,
            lh.user__id AS last_health_user_id,
            s.retry_schedule__id AS retry_schedule_id,
            rs.name AS retry_schedule_name,
            rs.strategy AS retry_strategy,
            rs.max_retries AS retry_max_retries,
            rs.custom_intervals AS retry_custom_intervals,
            rs.linear_delay AS retry_linear_delay,
            rs.increasing_base_delay AS retry_increasing_base_delay,
            rs.increasing_wait_factor AS retry_increasing_wait_factor
        FROM bucket_stats bs
        INNER JOIN webhook.subscription s USING (subscription__id)
        INNER JOIN event.application app ON app.application__id = s.application__id
        LEFT JOIN LATERAL (
            SELECT she.status, she.created_at, she.source, she.user__id
            FROM webhook.subscription_health_event she
            WHERE she.subscription__id = bs.subscription__id
            ORDER BY she.created_at DESC
            LIMIT 1
        ) lh ON true
        LEFT JOIN webhook.retry_schedule rs ON rs.retry_schedule__id = s.retry_schedule__id
        LEFT JOIN webhook.target_http th ON th.target__id = s.target__id
        WHERE s.is_enabled = true AND s.deleted_at IS NULL
        "#,
    )
    .bind(suspect_ids)
    .bind(time_window_secs)
    .bind(min_sample)
    .fetch_all(&mut **tx)
    .await
}
