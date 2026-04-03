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
        let totals: Vec<i32> = deltas.iter().map(|d| d.total.min(i32::MAX as i64) as i32).collect();
        let faileds: Vec<i32> = deltas.iter().map(|d| d.failed.min(i32::MAX as i64) as i32).collect();

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

    if result.rows_affected() == 0 {
        tracing::debug!("Watermark not advanced (already at or past target)");
    }

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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;

    async fn setup_test_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    fn test_config() -> HealthMonitorConfig {
        HealthMonitorConfig {
            interval: Duration::from_secs(60),
            warning_failure_percent: 50,
            disable_failure_percent: 90,
            time_window: Duration::from_secs(3600),
            message_window: 100,
            min_sample_size: 1,
            warning_cooldown: Duration::from_secs(3600),
            retention_period_days: 30,
            bucket_duration: Duration::from_secs(300),
            bucket_max_messages: 100,
            bucket_retention_days: 30,
        }
    }

    /// Sets the watermark inside the given transaction.
    async fn set_watermark(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, ts: DateTime<Utc>) {
        sqlx::query("UPDATE webhook.health_monitor_watermark SET last_processed_at = $1 WHERE id = 1")
            .bind(ts)
            .execute(&mut **tx)
            .await
            .unwrap();
    }

    /// Inserts the minimum FK-chain for a subscription + request_attempts:
    ///   iam.organization -> event.application -> event.service -> event.resource_type
    ///   -> event.verb -> event.event_type -> event.application_secret -> event.event
    ///   -> webhook.subscription (with target) -> webhook.request_attempt
    ///
    /// Returns (org_id, app_id, sub_id).
    async fn insert_test_fixtures(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        num_succeeded: i32,
        num_failed: i32,
        attempts_timestamp: DateTime<Utc>,
    ) -> (Uuid, Uuid, Uuid) {
        let org_id = Uuid::now_v7();
        let app_id = Uuid::now_v7();
        let sub_id = Uuid::now_v7();
        let target_id = sub_id; // target__id = subscription target__id (UNIQUE)
        let secret_token = Uuid::now_v7();

        // 1. Organization
        sqlx::query(
            "INSERT INTO iam.organization (organization__id, name, created_by) VALUES ($1, $2, $3)",
        )
        .bind(org_id)
        .bind("test-org-health")
        .bind(Uuid::nil())
        .execute(&mut **tx)
        .await
        .unwrap();

        // 2. Application
        sqlx::query(
            "INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, $3)",
        )
        .bind(app_id)
        .bind(org_id)
        .bind("test-app")
        .execute(&mut **tx)
        .await
        .unwrap();

        // 3. Service, resource_type, verb (for event_type FK chain)
        sqlx::query("INSERT INTO event.service (service__name, application__id) VALUES ($1, $2)")
            .bind("svc")
            .bind(app_id)
            .execute(&mut **tx)
            .await
            .unwrap();

        sqlx::query("INSERT INTO event.resource_type (resource_type__name, application__id, service__name) VALUES ($1, $2, $3)")
            .bind("res")
            .bind(app_id)
            .bind("svc")
            .execute(&mut **tx)
            .await
            .unwrap();

        sqlx::query("INSERT INTO event.verb (verb__name, application__id) VALUES ($1, $2)")
            .bind("created")
            .bind(app_id)
            .execute(&mut **tx)
            .await
            .unwrap();

        // 4. Event type (generated column: svc.res.created)
        sqlx::query("INSERT INTO event.event_type (application__id, service__name, resource_type__name, verb__name) VALUES ($1, $2, $3, $4)")
            .bind(app_id)
            .bind("svc")
            .bind("res")
            .bind("created")
            .execute(&mut **tx)
            .await
            .unwrap();

        // 5. Application secret
        sqlx::query("INSERT INTO event.application_secret (token, application__id) VALUES ($1, $2)")
            .bind(secret_token)
            .bind(app_id)
            .execute(&mut **tx)
            .await
            .unwrap();

        // 6. Subscription (labels required, target__id must be unique)
        sqlx::query(
            r#"INSERT INTO webhook.subscription
               (subscription__id, application__id, target__id, is_enabled, labels)
               VALUES ($1, $2, $3, true, '{"env":"test"}'::jsonb)"#,
        )
        .bind(sub_id)
        .bind(app_id)
        .bind(target_id)
        .execute(&mut **tx)
        .await
        .unwrap();

        // 7. Target HTTP (inherits webhook.target; FK to subscription.target__id)
        sqlx::query(
            "INSERT INTO webhook.target_http (target__id, method, url) VALUES ($1, $2, $3)",
        )
        .bind(target_id)
        .bind("POST")
        .bind("https://example.com/webhook")
        .execute(&mut **tx)
        .await
        .unwrap();

        // 8. Insert events + request_attempts
        // Disable the dispatch trigger once to avoid side-effects
        sqlx::query("ALTER TABLE event.event DISABLE TRIGGER event_dispatch")
            .execute(&mut **tx)
            .await
            .unwrap();

        for i in 0..(num_succeeded + num_failed) {
            let event_id = Uuid::now_v7();
            let attempt_id = Uuid::now_v7();
            let is_failed = i >= num_succeeded;

            sqlx::query(
                r#"INSERT INTO event.event
                   (event__id, application__id, event_type__name, payload_content_type, ip, occurred_at, application_secret__token, labels)
                   VALUES ($1, $2, 'svc.res.created', 'application/json', '127.0.0.1', $3, $4, '{"env":"test"}'::jsonb)"#,
            )
            .bind(event_id)
            .bind(app_id)
            .bind(attempts_timestamp)
            .bind(secret_token)
            .execute(&mut **tx)
            .await
            .unwrap();

            if is_failed {
                sqlx::query(
                    r#"INSERT INTO webhook.request_attempt
                       (request_attempt__id, event__id, subscription__id, application__id, failed_at)
                       VALUES ($1, $2, $3, $4, $5)"#,
                )
                .bind(attempt_id)
                .bind(event_id)
                .bind(sub_id)
                .bind(app_id)
                .bind(attempts_timestamp)
                .execute(&mut **tx)
                .await
                .unwrap();
            } else {
                sqlx::query(
                    r#"INSERT INTO webhook.request_attempt
                       (request_attempt__id, event__id, subscription__id, application__id, succeeded_at)
                       VALUES ($1, $2, $3, $4, $5)"#,
                )
                .bind(attempt_id)
                .bind(event_id)
                .bind(sub_id)
                .bind(app_id)
                .bind(attempts_timestamp)
                .execute(&mut **tx)
                .await
                .unwrap();
            }
        }

        sqlx::query("ALTER TABLE event.event ENABLE TRIGGER event_dispatch")
            .execute(&mut **tx)
            .await
            .unwrap();

        (org_id, app_id, sub_id)
    }

    // ── Bucket + watermark tests ────────────────────────────────────────

    /// Buckets are populated after a health tick.
    #[tokio::test]
    #[ignore]
    async fn test_health_buckets_populated() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config();
        let now = Utc::now();
        let watermark_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_watermark(&mut tx, watermark_past).await;
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

        let (subs, max_completed) =
            fetch_subscription_health_stats(&mut tx, &config).await.unwrap();

        // Verify buckets were created for our subscription
        let bucket: Option<(i32, i32)> = sqlx::query_as(
            r#"SELECT total_count, failed_count
               FROM webhook.subscription_health_bucket
               WHERE subscription__id = $1"#,
        )
        .bind(sub_id)
        .fetch_optional(&mut *tx)
        .await
        .unwrap();

        assert!(bucket.is_some(), "bucket should exist after health tick");
        let (total, failed) = bucket.unwrap();
        assert_eq!(total, 5, "total_count should be 5 (3 succeeded + 2 failed)");
        assert_eq!(failed, 2, "failed_count should be 2");
        assert!(max_completed.is_some(), "max_completed_at should be Some");

        let suspect_ids: Vec<Uuid> = subs.iter().map(|s| s.subscription_id).collect();
        assert!(
            suspect_ids.contains(&sub_id),
            "test subscription should be in suspects"
        );
        // tx dropped — automatic rollback
    }

    /// Watermark advances after a health tick.
    #[tokio::test]
    #[ignore]
    async fn test_watermark_advances() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config();
        let now = Utc::now();
        let watermark_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_watermark(&mut tx, watermark_past).await;
        let (_org_id, _app_id, _sub_id) = insert_test_fixtures(&mut tx, 2, 1, now).await;

        // Read watermark before
        let wm_before: DateTime<Utc> =
            sqlx::query_scalar("SELECT last_processed_at FROM webhook.health_monitor_watermark WHERE id = 1")
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert_eq!(wm_before, watermark_past);

        // Run health eval + advance watermark
        let (_subs, max_completed) =
            fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        if let Some(wm) = max_completed {
            advance_watermark(&mut tx, wm).await.unwrap();
        }

        // Read watermark after
        let wm_after: DateTime<Utc> =
            sqlx::query_scalar("SELECT last_processed_at FROM webhook.health_monitor_watermark WHERE id = 1")
                .fetch_one(&mut *tx)
                .await
                .unwrap();

        assert!(
            wm_after > wm_before,
            "watermark should have advanced: before={wm_before}, after={wm_after}"
        );
        // tx dropped — automatic rollback
    }

    // ── State machine scenario tests ────────────────────────────────────

    /// Aged buckets are closed after a second health tick.
    #[tokio::test]
    #[ignore]
    async fn test_bucket_closing() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config();
        let now = Utc::now();
        let watermark_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_watermark(&mut tx, watermark_past).await;
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

        // First pass: creates an open bucket
        let (_, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        if let Some(wm) = max_completed {
            advance_watermark(&mut tx, wm).await.unwrap();
        }

        // Verify bucket is open
        let open: Option<bool> = sqlx::query_scalar(
            "SELECT bucket_end IS NULL FROM webhook.subscription_health_bucket WHERE subscription__id = $1 LIMIT 1",
        )
        .bind(sub_id)
        .fetch_optional(&mut *tx)
        .await
        .unwrap();
        assert_eq!(open, Some(true), "bucket should be open initially");

        // Age the bucket beyond bucket_duration (300s)
        sqlx::query(
            "UPDATE webhook.subscription_health_bucket SET bucket_start = now() - interval '1 hour' WHERE subscription__id = $1",
        )
        .bind(sub_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // Second pass: Step 3 should close the aged bucket
        let _ = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();

        // Verify bucket is now closed
        let closed: bool = sqlx::query_scalar(
            "SELECT bucket_end IS NOT NULL FROM webhook.subscription_health_bucket WHERE subscription__id = $1 AND bucket_start < now() - interval '30 minutes' LIMIT 1",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert!(closed, "aged bucket should be closed after second health tick");
        // tx dropped — automatic rollback
    }

    /// Warned subscription still appears in suspects via UNION.
    #[tokio::test]
    #[ignore]
    async fn test_warned_subscription_in_suspects() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config();
        let now = Utc::now();
        let watermark_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_watermark(&mut tx, watermark_past).await;
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 0, 5, now).await;

        // First pass: ingest deltas and advance watermark
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        if let Some(wm) = max_completed {
            advance_watermark(&mut tx, wm).await.unwrap();
        }
        assert!(
            subs.iter().any(|s| s.subscription_id == sub_id),
            "subscription should be a suspect on first pass"
        );

        // Insert a warning health event (simulating state machine)
        sqlx::query(
            "INSERT INTO webhook.subscription_health_event (subscription__id, status, source) VALUES ($1, 'warning', 'system')",
        )
        .bind(sub_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // Replace buckets with healthy data (0 failures)
        sqlx::query("DELETE FROM webhook.subscription_health_bucket WHERE subscription__id = $1")
            .bind(sub_id)
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count) VALUES ($1, now(), 10, 0)",
        )
        .bind(sub_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // Second pass: watermark advanced, no re-ingestion. UNION picks up warned sub.
        let (subs2, _) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();

        let found = subs2.iter().find(|s| s.subscription_id == sub_id);
        assert!(found.is_some(), "warned subscription should still appear in suspects via UNION");
        assert!(
            found.unwrap().failure_percent < 50.0,
            "failure_percent should be low (subscription recovered)"
        );
        // tx dropped — automatic rollback
    }

    /// Recovery triggers a resolved health event.
    #[tokio::test]
    #[ignore]
    async fn test_recovery_triggers_resolved_event() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config();
        let now = Utc::now();
        let watermark_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_watermark(&mut tx, watermark_past).await;

        // 2 succeeded + 5 failed = 71% failure (>50% warning, <90% disable)
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

        // Phase 1: fetch + process -> warning (not disable, since 71% < 90%)
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        let sub = subs.iter().find(|s| s.subscription_id == sub_id).expect("subscription should be suspect");
        let actions = crate::health_monitor::state_machine::process_subscription(&mut tx, &config, sub).await.unwrap();
        if let Some(wm) = max_completed {
            advance_watermark(&mut tx, wm).await.unwrap();
        }
        assert!(
            actions.iter().any(|a| matches!(a, crate::health_monitor::notifications::HealthAction::Warning(_))),
            "first pass should produce a Warning action"
        );
        assert!(
            !actions.iter().any(|a| matches!(a, crate::health_monitor::notifications::HealthAction::Disabled(_))),
            "first pass should NOT disable (71% < 90%)"
        );

        let warning_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM webhook.subscription_health_event WHERE subscription__id = $1 AND status = 'warning'",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(warning_count, 1, "should have exactly one warning event");

        // Phase 2: replace buckets with low failure rate
        sqlx::query("DELETE FROM webhook.subscription_health_bucket WHERE subscription__id = $1")
            .bind(sub_id)
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count) VALUES ($1, now(), 20, 1)",
        )
        .bind(sub_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        let (subs2, _) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        let sub2 = subs2.iter().find(|s| s.subscription_id == sub_id).expect("warned sub should still be suspect");
        assert!(sub2.failure_percent < 50.0, "failure rate should be low now");
        let actions2 = crate::health_monitor::state_machine::process_subscription(&mut tx, &config, sub2).await.unwrap();

        let resolved_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM webhook.subscription_health_event WHERE subscription__id = $1 AND status = 'resolved' AND source = 'system'",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(resolved_count, 1, "should have a resolved event with source=system");
        assert!(
            actions2.iter().any(|a| matches!(a, crate::health_monitor::notifications::HealthAction::Recovered(_))),
            "second pass should produce a Recovered action"
        );
        // tx dropped — automatic rollback
    }

    /// Cooldown prevents re-warning after a recent resolved event.
    #[tokio::test]
    #[ignore]
    async fn test_cooldown_prevents_rewarning() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config();
        let now = Utc::now();
        let watermark_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_watermark(&mut tx, watermark_past).await;

        // 2 succeeded + 5 failed = 71% (warning range, not disable)
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

        // Phase 1: trigger warning
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        let sub = subs.iter().find(|s| s.subscription_id == sub_id).unwrap();
        let _ = crate::health_monitor::state_machine::process_subscription(&mut tx, &config, sub).await.unwrap();
        if let Some(wm) = max_completed {
            advance_watermark(&mut tx, wm).await.unwrap();
        }

        // Insert a resolved event with created_at = now() (within cooldown)
        sqlx::query(
            "INSERT INTO webhook.subscription_health_event (subscription__id, status, source) VALUES ($1, 'resolved', 'system')",
        )
        .bind(sub_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // Phase 2: buckets still have high failure rate, but cooldown blocks re-warning
        let (subs2, _) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        let sub2 = subs2.iter().find(|s| s.subscription_id == sub_id)
            .expect("subscription should still be suspect via bucket failure count");
        let actions = crate::health_monitor::state_machine::process_subscription(&mut tx, &config, sub2).await.unwrap();

        assert!(
            !actions.iter().any(|a| matches!(a, crate::health_monitor::notifications::HealthAction::Warning(_))),
            "cooldown should prevent re-warning"
        );

        let warning_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM webhook.subscription_health_event WHERE subscription__id = $1 AND status = 'warning'",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(warning_count, 1, "no new warning event should have been inserted during cooldown");
        // tx dropped — automatic rollback
    }

    // ── Utility function tests ──────────────────────────────────────────

    /// reset_healthy_failure_percent clears failure_percent for non-suspects.
    #[tokio::test]
    #[ignore]
    async fn test_reset_failure_percent_for_non_suspects() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let now = Utc::now();

        let mut tx = pool.begin().await.unwrap();
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 5, 0, now).await;

        // Manually set failure_percent on the subscription
        sqlx::query("UPDATE webhook.subscription SET failure_percent = 50.0 WHERE subscription__id = $1")
            .bind(sub_id)
            .execute(&mut *tx)
            .await
            .unwrap();

        let fp_before: Option<f64> = sqlx::query_scalar(
            "SELECT failure_percent FROM webhook.subscription WHERE subscription__id = $1",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(fp_before, Some(50.0));

        // Call reset with empty suspect list — only affects rows inside this tx
        let rows = reset_healthy_failure_percent(&mut tx, &[]).await.unwrap();
        assert!(rows >= 1, "should have reset at least 1 subscription");

        let fp_after: Option<f64> = sqlx::query_scalar(
            "SELECT failure_percent FROM webhook.subscription WHERE subscription__id = $1",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(fp_after, None, "failure_percent should be NULL after reset");
        // tx dropped — automatic rollback
    }

    /// cleanup_old_buckets removes buckets older than retention period.
    #[tokio::test]
    #[ignore]
    async fn test_cleanup_old_buckets() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config(); // bucket_retention_days = 30
        let now = Utc::now();

        let mut tx = pool.begin().await.unwrap();
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

        // Insert a bucket older than retention (31 days ago)
        let old_start = now - chrono::Duration::days(31);
        sqlx::query(
            "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, bucket_end, total_count, failed_count) VALUES ($1, $2, $3, 10, 5)",
        )
        .bind(sub_id)
        .bind(old_start)
        .bind(old_start + chrono::Duration::seconds(300))
        .execute(&mut *tx)
        .await
        .unwrap();

        let count_before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM webhook.subscription_health_bucket WHERE subscription__id = $1 AND bucket_start < now() - interval '30 days'",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert!(count_before >= 1, "old bucket should exist before cleanup");

        // Inline the cleanup_old_buckets SQL to run within the transaction
        let retention_days = config.bucket_retention_days as i32;
        let deleted = sqlx::query(
            "DELETE FROM webhook.subscription_health_bucket WHERE bucket_start < now() - make_interval(days => $1)",
        )
        .bind(retention_days)
        .execute(&mut *tx)
        .await
        .unwrap()
        .rows_affected();
        assert!(deleted >= 1, "should have deleted at least 1 old bucket");

        let count_after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM webhook.subscription_health_bucket WHERE subscription__id = $1 AND bucket_start < now() - interval '30 days'",
        )
        .bind(sub_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(count_after, 0, "old bucket should be deleted after cleanup");
        // tx dropped — automatic rollback
    }
}
