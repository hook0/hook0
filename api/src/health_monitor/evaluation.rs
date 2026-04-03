//! Health monitor evaluation pipeline.
//!
//! **What it does**: monitors webhook delivery success/failure rates to
//! auto-warn and auto-disable unhealthy subscription endpoints.
//!
//! **How it works at a high level** (one "tick"):
//!   1. Read the cursor — a bookmark saying "I've already processed all
//!      deliveries up to this timestamp."
//!   2. Scan `request_attempt` for deliveries newer than the cursor.
//!   3. Group those deliveries into time buckets per subscription
//!      (a bucket = a group of deliveries bounded by a max duration OR a max
//!      message count, whichever comes first — e.g. "10:00–10:05, 50 ok, 3 failed").
//!   4. Close buckets that are full (exceeded duration or message count).
//!   5. Identify "suspects" — subscriptions with enough recent failures to
//!      potentially need a warning or to be disabled.
//!   6. Compute each suspect's failure rate over a sliding window (the last N
//!      minutes of buckets, configured by `time_window`).
//!
//! The caller (mod.rs) then feeds each suspect into the state machine, which
//! decides whether to warn, disable, or resolve, and finally advances the
//! cursor so the next tick only looks at newer deliveries.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::HealthMonitorConfig;
use super::queries;

pub use queries::SubscriptionHealth;

/// Runs the full evaluation pipeline for one tick: read cursor, ingest new
/// deliveries, bucket them, close full buckets, find suspects, compute failure rates.
///
/// Returns `(suspects, max_completed_at)` where `max_completed_at` is the
/// timestamp the caller should pass to `advance_cursor` after committing the
/// transaction.
pub async fn fetch_subscription_health_stats(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<(Vec<SubscriptionHealth>, Option<DateTime<Utc>>), sqlx::Error> {
    // 1. Read the cursor — "where did I stop last time?"
    let cursor = queries::read_cursor(tx).await?;

    // 2. Scan for new deliveries since the cursor (capped to avoid long queries)
    let deltas = queries::ingest_deltas(tx, cursor, config.max_delta_rows_per_tick).await?;
    let max_completed_at = deltas.iter().filter_map(|d| d.max_completed_at).max();

    // 3. Pour those delivery counts into open buckets (one bucket per subscription)
    // Skip if empty — upsert_buckets would produce an empty VALUES clause
    if !deltas.is_empty() {
        queries::upsert_buckets(tx, &deltas).await?;
    }

    // 4. Close any bucket that exceeded its time or message limit
    queries::close_full_buckets(tx, config).await?;

    // 5. Find subscriptions that might be unhealthy (or were previously warned)
    let suspect_ids = queries::find_suspects(tx, config).await?;

    // 6. Compute failure rates for each suspect
    if suspect_ids.is_empty() {
        return Ok((Vec::new(), max_completed_at));
    }
    let subscriptions = queries::compute_failure_rates(tx, &suspect_ids, config).await?;

    Ok((subscriptions, max_completed_at))
}

/// Advances the cursor to the given timestamp.
///
/// The cursor is a singleton row (exactly one row in the table). The WHERE clause
/// ensures we never move the cursor backwards — if two ticks overlap, the later
/// timestamp wins.
pub async fn advance_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    max_completed_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE webhook.health_monitor_cursor
        SET last_processed_at = $1
        WHERE cursor__id = 1
          AND $1 > last_processed_at
        "#,
    )
    .bind(max_completed_at)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        tracing::debug!("Cursor not advanced (already at or past target)");
    }

    Ok(())
}

/// Resets failure_percent to NULL for subscriptions that are NOT suspects.
///
/// Why? The frontend displays failure_percent as a badge. If a subscription
/// was briefly unhealthy but recovered, we clear its stale failure data so
/// the UI doesn't show an outdated red badge on a now-healthy endpoint.
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;

    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

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
            min_sample_size: 1,
            warning_cooldown: Duration::from_secs(3600),
            retention_period_days: 30,
            bucket_duration: Duration::from_secs(300),
            bucket_max_messages: 100,
            bucket_retention_days: 30,
            max_delta_rows_per_tick: 50_000,
        }
    }

    /// Sets the cursor inside the given transaction.
    async fn set_cursor(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, ts: DateTime<Utc>) {
        sqlx::query("UPDATE webhook.health_monitor_cursor SET last_processed_at = $1 WHERE cursor__id = 1")
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

    // -- Bucket + cursor tests ------------------------------------------------

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
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
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
        // tx dropped -- automatic rollback
    }

    /// Cursor advances after a health tick.
    #[tokio::test]
    #[ignore]
    async fn test_cursor_advances() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, _app_id, _sub_id) = insert_test_fixtures(&mut tx, 2, 1, now).await;

        // Read cursor before
        let cursor_before: DateTime<Utc> =
            sqlx::query_scalar("SELECT last_processed_at FROM webhook.health_monitor_cursor WHERE cursor__id = 1")
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert_eq!(cursor_before, cursor_past);

        // Run health eval + advance cursor
        let (_subs, max_completed) =
            fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        if let Some(ts) = max_completed {
            advance_cursor(&mut tx, ts).await.unwrap();
        }

        // Read cursor after
        let cursor_after: DateTime<Utc> =
            sqlx::query_scalar("SELECT last_processed_at FROM webhook.health_monitor_cursor WHERE cursor__id = 1")
                .fetch_one(&mut *tx)
                .await
                .unwrap();

        assert!(
            cursor_after > cursor_before,
            "cursor should have advanced: before={cursor_before}, after={cursor_after}"
        );
        // tx dropped -- automatic rollback
    }

    // -- State machine scenario tests -----------------------------------------

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
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

        // First pass: creates an open bucket
        let (_, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        if let Some(ts) = max_completed {
            advance_cursor(&mut tx, ts).await.unwrap();
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

        // Second pass: should close the aged bucket
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
        // tx dropped -- automatic rollback
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
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 0, 5, now).await;

        // First pass: ingest deltas and advance cursor
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        if let Some(ts) = max_completed {
            advance_cursor(&mut tx, ts).await.unwrap();
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

        // Second pass: cursor advanced, no re-ingestion. UNION picks up warned sub.
        let (subs2, _) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();

        let found = subs2.iter().find(|s| s.subscription_id == sub_id);
        assert!(found.is_some(), "warned subscription should still appear in suspects via UNION");
        assert!(
            found.unwrap().failure_percent < 50.0,
            "failure_percent should be low (subscription recovered)"
        );
        // tx dropped -- automatic rollback
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
        let cursor_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;

        // 2 succeeded + 5 failed = 71% failure (>50% warning, <90% disable)
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

        // Phase 1: fetch + process -> warning (not disable, since 71% < 90%)
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        let sub = subs.iter().find(|s| s.subscription_id == sub_id).expect("subscription should be suspect");
        let actions = crate::health_monitor::state_machine::evaluate_health_transition(&mut tx, &config, sub).await.unwrap();
        if let Some(ts) = max_completed {
            advance_cursor(&mut tx, ts).await.unwrap();
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
        let actions2 = crate::health_monitor::state_machine::evaluate_health_transition(&mut tx, &config, sub2).await.unwrap();

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
        // tx dropped -- automatic rollback
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
        let cursor_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;

        // 2 succeeded + 5 failed = 71% (warning range, not disable)
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

        // Phase 1: trigger warning
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config).await.unwrap();
        let sub = subs.iter().find(|s| s.subscription_id == sub_id).unwrap();
        let _ = crate::health_monitor::state_machine::evaluate_health_transition(&mut tx, &config, sub).await.unwrap();
        if let Some(ts) = max_completed {
            advance_cursor(&mut tx, ts).await.unwrap();
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
        let actions = crate::health_monitor::state_machine::evaluate_health_transition(&mut tx, &config, sub2).await.unwrap();

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
        // tx dropped -- automatic rollback
    }

    // -- Utility function tests -----------------------------------------------

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

        // Call reset with empty suspect list
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
        // tx dropped -- automatic rollback
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
        // tx dropped -- automatic rollback
    }
}
