//! SQL layer for the health monitor — cursor management, delta ingestion,
//! bucket upserts, suspect identification, and failure rate computation.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::HealthMonitorConfig;
use super::types::{HealthEventSource, HealthStatus};

/// One row per subscription from the delta scan: how many deliveries completed
/// since the cursor, split into total vs failed.
#[derive(Debug)]
pub struct DeltaRow {
    pub subscription_id: Uuid,
    pub total: i64,
    pub failed: i64,
    /// The latest completion timestamp in this batch — used to advance the cursor
    /// after the transaction commits so the next tick skips these rows.
    pub max_completed_at: Option<DateTime<Utc>>,
}

/// All the data the state machine needs to decide whether to warn, disable,
/// or resolve a subscription: its failure rate, its latest health event,
/// and its retry schedule (included so notification emails can reference it).
#[derive(Debug)]
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
    // Selected for potential use in notification personalization — suppress warning until wired up
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

/// Adds new delivery results to each subscription's current open bucket.
///
/// Two-step approach:
/// 1. Fetch each subscription's currently open bucket (if any) in one query
/// 2. Bulk upsert all delivery counts using QueryBuilder::push_values
///
/// For subscriptions without an open bucket, a new one is created starting now.
/// The ON CONFLICT clause adds counts to existing buckets rather than replacing.
pub async fn upsert_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    deltas: &[DeltaRow],
) -> Result<(), sqlx::Error> {
    // Nothing to insert — skip to avoid an empty VALUES clause
    if deltas.is_empty() {
        return Ok(());
    }

    let sub_ids: Vec<Uuid> = deltas.iter().map(|d| d.subscription_id).collect();
    let now = Utc::now();

    // Step 1: Find open buckets for all subscriptions in one query
    let open_rows = sqlx::query!(
        r#"
        SELECT subscription__id AS "subscription_id!", bucket_start AS "bucket_start!"
        FROM webhook.subscription_health_bucket
        WHERE subscription__id = ANY($1)
          AND bucket_end IS NULL
        "#,
        &sub_ids,
    )
    .fetch_all(&mut **tx)
    .await?;

    let open_buckets: std::collections::HashMap<Uuid, DateTime<Utc>> = open_rows
        .into_iter()
        .map(|r| (r.subscription_id, r.bucket_start))
        .collect();

    // Step 2: Bulk upsert — one INSERT with multiple VALUES rows
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count) ",
    );

    qb.push_values(deltas, |mut b, d| {
        let bucket_start = open_buckets.get(&d.subscription_id).copied().unwrap_or(now);
        b.push_bind(d.subscription_id)
            .push_bind(bucket_start)
            .push_bind(d.total.min(i32::MAX as i64) as i32)
            .push_bind(d.failed.min(i32::MAX as i64) as i32);
    });

    qb.push(
        " ON CONFLICT (subscription__id, bucket_start) DO UPDATE SET \
         total_count = subscription_health_bucket.total_count + EXCLUDED.total_count, \
         failed_count = subscription_health_bucket.failed_count + EXCLUDED.failed_count",
    );

    qb.build().execute(&mut **tx).await?;

    Ok(())
}

/// Closes buckets that are "full" — either they've been open too long (exceeded
/// the configured duration) or they contain too many deliveries (exceeded the
/// configured max message count).
///
/// A closed bucket (bucket_end IS NOT NULL) is frozen: no more delivery results
/// will be added to it. New deliveries for that subscription will go into a
/// fresh bucket on the next tick.
///
/// Why close buckets? Without closing, a single bucket would grow indefinitely.
/// Closing creates discrete time windows that let us compute failure rates
/// over a sliding window (e.g., "failure rate in the last hour" = sum of
/// the last 12 five-minute buckets).
pub async fn close_full_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let bucket_duration_secs = config.bucket_duration.as_secs() as f64;
    let bucket_max_messages = config.bucket_max_messages as i32;

    let result = sqlx::query!(
        r#"
        UPDATE webhook.subscription_health_bucket
        SET bucket_end = now()
        WHERE bucket_end IS NULL
          AND (bucket_start < now() - make_interval(secs => $1)
               OR total_count >= $2)
        "#,
        bucket_duration_secs,
        bucket_max_messages,
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
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
            lh.source AS "last_health_source?: HealthEventSource",
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
            SELECT she.status, she.created_at, she.source, she.user__id
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
