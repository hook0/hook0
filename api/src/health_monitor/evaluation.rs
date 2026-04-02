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

/// Delta row returned by the watermark-based scan of request_attempt.
#[derive(Debug, sqlx::FromRow)]
struct DeltaRow {
    subscription_id: Uuid,
    total: i64,
    failed: i64,
    max_completed_at: Option<DateTime<Utc>>,
}

/// Implements the 7-step bucketed health evaluation flow.
///
/// Returns `(subscriptions, max_completed_at)` where max_completed_at is used
/// by the caller to advance the watermark after committing.
pub async fn fetch_subscription_health_stats(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<(Vec<SubscriptionHealth>, Option<DateTime<Utc>>), sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as f64;
    let min_sample = config.min_sample_size as i64;
    let bucket_duration_secs = config.bucket_duration.as_secs() as f64;
    let bucket_max_messages = config.bucket_max_messages as i32;

    // ── Step 1: Read watermark and ingest deltas ─────────────────────────
    let watermark: DateTime<Utc> =
        sqlx::query_scalar("SELECT last_processed_at FROM webhook.health_monitor_watermark WHERE id = 1")
            .fetch_one(&mut **tx)
            .await?;

    let deltas = sqlx::query_as::<_, DeltaRow>(
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
    .await?;

    // Compute the overall max_completed_at across all subscriptions
    let max_completed_at: Option<DateTime<Utc>> = deltas
        .iter()
        .filter_map(|d| d.max_completed_at)
        .max();

    // ── Step 2: Upsert deltas into open buckets ──────────────────────────
    if !deltas.is_empty() {
        // Build parallel arrays for the unnest-based bulk upsert
        let sub_ids: Vec<Uuid> = deltas.iter().map(|d| d.subscription_id).collect();
        let totals: Vec<i32> = deltas.iter().map(|d| d.total as i32).collect();
        let faileds: Vec<i32> = deltas.iter().map(|d| d.failed as i32).collect();

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
    }

    // ── Step 3: Close full buckets ───────────────────────────────────────
    sqlx::query(
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

    // ── Step 4: Detect suspects ──────────────────────────────────────────
    let suspect_ids: Vec<Uuid> = sqlx::query_scalar(
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
    .await?;

    if suspect_ids.is_empty() {
        return Ok((Vec::new(), max_completed_at));
    }

    // ── Step 5 + 6: Compute failure rate for suspects and join details ───
    let subscriptions = sqlx::query_as::<_, SubscriptionHealth>(
        r#"
        WITH bucket_stats AS (
            SELECT subscription__id,
                   SUM(failed_count)::float8 / SUM(total_count) * 100.0 AS failure_percent,
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
    .bind(&suspect_ids)
    .bind(time_window_secs)
    .bind(min_sample)
    .fetch_all(&mut **tx)
    .await?;

    Ok((subscriptions, max_completed_at))
}

/// Advances the watermark to the given timestamp.
/// Asserts that exactly one row was updated (singleton table invariant).
pub async fn advance_watermark(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    max_completed_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE webhook.health_monitor_watermark
        SET last_processed_at = $1
        WHERE id = 1
          AND $1 > last_processed_at
        "#,
    )
    .bind(max_completed_at)
    .execute(&mut **tx)
    .await?;

    assert_eq!(
        result.rows_affected(),
        1,
        "health_monitor_watermark singleton row missing or watermark did not advance"
    );

    Ok(())
}

/// Resets failure_percent to NULL for subscriptions that are NOT in the suspect list
/// but currently have a non-null failure_percent.
pub async fn reset_healthy_failure_percent(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    suspect_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE webhook.subscription
        SET failure_percent = NULL
        WHERE failure_percent IS NOT NULL
          AND subscription__id NOT IN (SELECT unnest($1::uuid[]))
        "#,
    )
    .bind(suspect_ids)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

/// Removes old closed buckets beyond the configured retention period.
pub async fn cleanup_old_buckets(
    db: &PgPool,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_days = config.bucket_retention_days as i32;

    let result = sqlx::query(
        r#"
        DELETE FROM webhook.subscription_health_bucket
        WHERE bucket_start < now() - make_interval(days => $1)
        "#,
    )
    .bind(retention_days)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
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
