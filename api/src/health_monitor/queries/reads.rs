//! Read-only analytical queries: cursor management, delta ingestion, suspect
//! identification, and failure rate computation.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::HealthMonitorConfig;
use super::super::types::{HealthEventCause, HealthStatus};
use super::{DeltaRow, SubscriptionHealth};

/// Reads the cursor — the timestamp of the last delivery we've already processed.
/// Everything newer than this value is "new work" for the current tick.
///
/// The cursor lives in a singleton table (exactly one row, enforced by a CHECK
/// constraint on the primary key). It starts at '-infinity' so the first tick
/// picks up all historical deliveries.
pub async fn read_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT last_processed_at FROM webhook.health_monitor_cursor WHERE cursor__id = 1",
    )
    .fetch_one(&mut **tx)
    .await
}

/// Scans request_attempt for deliveries completed after the cursor.
///
/// Groups results by subscription so we get one row per subscription with
/// (total_count, failed_count, max_completed_at). The LIMIT caps the batch
/// size to avoid long-running queries on high-traffic instances — any
/// remaining rows will be picked up on the next tick.
pub async fn ingest_deltas(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    cursor: DateTime<Utc>,
    max_rows: u32,
) -> Result<Vec<DeltaRow>, sqlx::Error> {
    let max_rows = max_rows as i64;
    sqlx::query_as!(
        DeltaRow,
        r#"
        WITH capped AS (
            SELECT subscription__id, failed_at, succeeded_at
            FROM webhook.request_attempt
            WHERE COALESCE(succeeded_at, failed_at) > $1
              AND (succeeded_at IS NOT NULL OR failed_at IS NOT NULL)
            ORDER BY COALESCE(succeeded_at, failed_at)
            LIMIT $2
        )
        SELECT subscription__id AS "subscription_id!",
               COUNT(*) AS "total!",
               COUNT(failed_at) AS "failed!",
               MAX(COALESCE(succeeded_at, failed_at)) AS "max_completed_at"
        FROM capped
        GROUP BY subscription__id
        "#,
        cursor,
        max_rows,
    )
    .fetch_all(&mut **tx)
    .await
}

/// Find subscriptions that might need a health warning or to be disabled:
/// those with high failure counts in recent buckets UNION those currently
/// in warning status (so we can check if they've recovered).
///
/// The UNION with warned subscriptions is critical: without it, a subscription
/// that was warned but then stopped failing would never get a "resolved" event
/// because it wouldn't appear in the bucket-based query anymore.
pub async fn find_suspects(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as f64;
    let min_sample = config.min_sample_size as i64;

    sqlx::query_scalar!(
        r#"
        -- Subscriptions with enough delivery data to evaluate — only active, non-deleted ones
        SELECT shb.subscription__id AS "subscription_id!"
        FROM webhook.subscription_health_bucket shb
        INNER JOIN webhook.subscription s ON s.subscription__id = shb.subscription__id
        WHERE shb.bucket_start > now() - make_interval(secs => $1)
          AND s.is_enabled AND s.deleted_at IS NULL
        GROUP BY shb.subscription__id
        HAVING SUM(shb.failed_count) > $2
        UNION
        -- Subscriptions currently in 'warning' state — re-evaluated to detect recovery or escalation
        SELECT subscription__id
        FROM (
            SELECT DISTINCT ON (subscription__id) subscription__id, status
            FROM webhook.subscription_health_event
            ORDER BY subscription__id, created_at DESC
        ) latest
        WHERE latest.status = 'warning'
        "#,
        time_window_secs,
        min_sample,
    )
    .fetch_all(&mut **tx)
    .await
}

/// Computes each suspect subscription's failure rate over the sliding window,
/// and joins in all the context needed for the state machine and notifications:
/// subscription details, latest health event, target URL, and retry schedule.
///
/// The failure rate formula: SUM(failed_count) / SUM(total_count) * 100.
/// Only buckets within the configured time_window are included.
/// Subscriptions with fewer total deliveries than min_sample_size are excluded
/// to avoid false positives on low-traffic endpoints.
pub async fn compute_failure_rates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    suspect_ids: &[Uuid],
    config: &HealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as f64;
    let min_sample = config.min_sample_size as i64;

    sqlx::query_as!(
        SubscriptionHealth,
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
            bs.subscription__id AS "subscription_id!",
            s.application__id AS "application_id!",
            app.organization__id AS "organization_id!",
            app.name AS "application_name?",
            s.description,
            coalesce(th.url, '') AS "target_url!",
            bs.failure_percent AS "failure_percent!",
            lh.status AS "last_health_status?: HealthStatus",
            lh.created_at AS "last_health_at?",
            lh.cause AS "last_health_cause?: HealthEventCause",
            lh.user__id AS "last_health_user_id?",
            -- Phase 3 will replace these NULL casts with a JOIN to webhook.retry_schedule (the table is owned by Phase 3).
            NULL::uuid AS "retry_schedule_id?",
            NULL::text AS "retry_schedule_name?",
            NULL::text AS "retry_strategy?",
            NULL::int4 AS "retry_max_retries?",
            NULL::int4[] AS "retry_custom_intervals?",
            NULL::int4 AS "retry_linear_delay?",
            NULL::int4 AS "retry_increasing_base_delay?",
            NULL::float8 AS "retry_increasing_wait_factor?"
        FROM bucket_stats bs
        INNER JOIN webhook.subscription s USING (subscription__id)
        INNER JOIN event.application app ON app.application__id = s.application__id
        LEFT JOIN LATERAL (
            SELECT she.status, she.created_at, she.cause, she.user__id
            FROM webhook.subscription_health_event she
            WHERE she.subscription__id = bs.subscription__id
            ORDER BY she.created_at DESC
            LIMIT 1
        ) lh ON true
        LEFT JOIN webhook.target_http th ON th.target__id = s.target__id
        WHERE s.is_enabled = true AND s.deleted_at IS NULL
        "#,
        suspect_ids,
        time_window_secs,
        min_sample,
    )
    .fetch_all(&mut **tx)
    .await
}
