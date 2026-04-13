//! Threshold-driven suspect tracking tests.
//!
//! The threshold comparison logic itself (failure_percent vs
//! `warning_failure_percent` / `disable_failure_percent`) lives in
//! [`crate::health_monitor::queries::find_suspects`]. This sub-module tests
//! the UNION behavior that keeps a previously-warned subscription in the
//! suspect set even after its bucket failure rate drops — that is what lets
//! the state machine fire the Recovered transition.

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::super::fetch_subscription_health_stats;
    use super::super::test_helpers::{
        insert_test_fixtures, set_cursor, setup_test_pool, test_config,
    };

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
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();
        if let Some(ts) = max_completed {
            super::super::advance_cursor(&mut tx, ts).await.unwrap();
        }
        assert!(
            subs.iter().any(|s| s.subscription_id == sub_id),
            "subscription should be a suspect on first pass"
        );

        // Insert a warning health event (simulating state machine)
        sqlx::query!(
            "INSERT INTO webhook.subscription_health_event (subscription__id, status, source) VALUES ($1, 'warning', 'system')",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        // Replace buckets with healthy data (0 failures)
        sqlx::query!(
            "DELETE FROM webhook.subscription_health_bucket WHERE subscription__id = $1",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count) VALUES ($1, now(), 10, 0)",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        // Second pass: cursor advanced, no re-ingestion. UNION picks up warned sub.
        let (subs2, _) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();

        let found = subs2.iter().find(|s| s.subscription_id == sub_id);
        assert!(
            found.is_some(),
            "warned subscription should still appear in suspects via UNION"
        );
        assert!(
            found.unwrap().failure_percent < 50.0,
            "failure_percent should be low (subscription recovered)"
        );
        // tx dropped -- automatic rollback
    }
}
