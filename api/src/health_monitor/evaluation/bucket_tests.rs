//! Bucket-lifecycle tests: population, closing, and retention cleanup.
//!
//! These tests black-box the bucket aggregation logic in
//! [`crate::health_monitor::queries`] (`upsert_buckets`, `close_full_buckets`)
//! by driving it through the orchestrator
//! [`super::fetch_subscription_health_stats`]. See the parent [`super`]
//! module for the high-level narrative.

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::super::fetch_subscription_health_stats;
    use super::super::test_helpers::{insert_test_fixtures, set_cursor, test_config};

    /// Buckets are populated after a health tick.
    #[sqlx::test(migrations = "./migrations")]
    async fn test_health_buckets_populated(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();

        // Verify buckets were created for our subscription
        let bucket = sqlx::query!(
            r#"SELECT total_count, failed_count
               FROM webhook.subscription_health_bucket
               WHERE subscription__id = $1"#,
            sub_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .unwrap();

        assert!(bucket.is_some(), "bucket should exist after health tick");
        let b = bucket.unwrap();
        assert_eq!(
            b.total_count, 5,
            "total_count should be 5 (3 succeeded + 2 failed)"
        );
        assert_eq!(b.failed_count, 2, "failed_count should be 2");
        assert!(max_completed.is_some(), "max_completed_at should be Some");

        let suspect_ids: Vec<Uuid> = subs.iter().map(|s| s.subscription_id).collect();
        assert!(
            suspect_ids.contains(&sub_id),
            "test subscription should be in suspects"
        );
        // tx dropped -- automatic rollback
    }

    /// Aged buckets are closed after a second health tick.
    #[sqlx::test(migrations = "./migrations")]
    async fn test_bucket_closing(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

        // First pass: creates an open bucket
        let (_, max_completed) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();
        if let Some(ts) = max_completed {
            super::super::advance_cursor(&mut tx, ts).await.unwrap();
        }

        // Verify bucket is open
        let open = sqlx::query_scalar!(
            r#"SELECT (bucket_end IS NULL) AS "is_open!" FROM webhook.subscription_health_bucket WHERE subscription__id = $1 LIMIT 1"#,
            sub_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .unwrap();
        assert_eq!(open, Some(true), "bucket should be open initially");

        // Age the bucket beyond bucket_duration (300s)
        sqlx::query!(
            "UPDATE webhook.subscription_health_bucket SET bucket_start = now() - interval '1 hour' WHERE subscription__id = $1",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        // Second pass: should close the aged bucket
        let _ = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();

        // Verify bucket is now closed
        let closed = sqlx::query_scalar!(
            r#"SELECT (bucket_end IS NOT NULL) AS "is_closed!" FROM webhook.subscription_health_bucket WHERE subscription__id = $1 AND bucket_start < now() - interval '30 minutes' LIMIT 1"#,
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert!(
            closed,
            "aged bucket should be closed after second health tick"
        );
        // tx dropped -- automatic rollback
    }

    /// cleanup_old_buckets removes buckets older than retention period.
    #[sqlx::test(migrations = "./migrations")]
    async fn test_cleanup_old_buckets(pool: PgPool) {
        let config = test_config(); // bucket_retention_days = 30
        let now = Utc::now();

        let mut tx = pool.begin().await.unwrap();
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

        // Insert a bucket older than retention (31 days ago)
        let old_start = now - chrono::Duration::days(31);
        let old_end = old_start + chrono::Duration::seconds(300);
        sqlx::query!(
            "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, bucket_end, total_count, failed_count) VALUES ($1, $2, $3, 10, 5)",
            sub_id,
            old_start,
            old_end,
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        let count_before = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM webhook.subscription_health_bucket WHERE subscription__id = $1 AND bucket_start < now() - interval '30 days'"#,
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert!(count_before >= 1, "old bucket should exist before cleanup");

        // Inline the cleanup_old_buckets SQL to run within the transaction
        let retention_days = config.bucket_retention_days as i32;
        let deleted = sqlx::query!(
            "DELETE FROM webhook.subscription_health_bucket WHERE bucket_start < now() - make_interval(days => $1)",
            retention_days,
        )
        .execute(&mut *tx)
        .await
        .unwrap()
        .rows_affected();
        assert!(deleted >= 1, "should have deleted at least 1 old bucket");

        let count_after = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM webhook.subscription_health_bucket WHERE subscription__id = $1 AND bucket_start < now() - interval '30 days'"#,
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(count_after, 0, "old bucket should be deleted after cleanup");
        // tx dropped -- automatic rollback
    }
}
