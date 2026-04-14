//! Adaptive-window state machine flow tests: two-pass warning then
//! recovery / cooldown evaluated across the `time_window`.
//!
//! These tests exercise the orchestrator
//! [`super::fetch_subscription_health_stats`] together with the
//! [`crate::health_monitor::state_machine`] end-to-end. See the parent
//! [`super`] module for the high-level narrative.

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sqlx::PgPool;

    use super::super::fetch_subscription_health_stats;
    use super::super::test_helpers::{
        insert_test_fixtures, process_subscription, set_cursor, test_config,
    };

    /// Recovery triggers a resolved health event.
    #[sqlx::test(migrations = "./migrations")]
    async fn test_recovery_triggers_resolved_event(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;

        // 2 succeeded + 5 failed = 71% failure (>50% warning, <90% disable)
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

        // Phase 1: fetch + process -> warning (not disable, since 71% < 90%)
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();
        let sub = subs
            .iter()
            .find(|s| s.subscription_id == sub_id)
            .expect("subscription should be suspect");
        let actions = process_subscription(&mut tx, &config, sub).await.unwrap();
        if let Some(ts) = max_completed {
            super::super::advance_cursor(&mut tx, ts).await.unwrap();
        }
        assert!(
            actions.iter().any(|a| matches!(
                a,
                crate::health_monitor::notifications::HealthAction::Warning(_)
            )),
            "first pass should produce a Warning action"
        );
        assert!(
            !actions.iter().any(|a| matches!(
                a,
                crate::health_monitor::notifications::HealthAction::Disabled(_)
            )),
            "first pass should NOT disable (71% < 90%)"
        );

        let warning_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM webhook.subscription_health_event WHERE subscription__id = $1 AND status = 'warning'"#,
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(warning_count, 1, "should have exactly one warning event");

        // Phase 2: replace buckets with low failure rate
        sqlx::query!(
            "DELETE FROM webhook.subscription_health_bucket WHERE subscription__id = $1",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count) VALUES ($1, now(), 20, 1)",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        let (subs2, _) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();
        let sub2 = subs2
            .iter()
            .find(|s| s.subscription_id == sub_id)
            .expect("warned sub should still be suspect");
        assert!(
            sub2.failure_percent < 50.0,
            "failure rate should be low now"
        );
        let actions2 = process_subscription(&mut tx, &config, sub2).await.unwrap();

        let resolved_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM webhook.subscription_health_event WHERE subscription__id = $1 AND status = 'resolved' AND cause = 'auto'"#,
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(
            resolved_count, 1,
            "should have a resolved event with cause=auto"
        );
        assert!(
            actions2.iter().any(|a| matches!(
                a,
                crate::health_monitor::notifications::HealthAction::Recovered(_)
            )),
            "second pass should produce a Recovered action"
        );
    }

    /// Cooldown prevents re-warning after a recent resolved event.
    #[sqlx::test(migrations = "./migrations")]
    async fn test_cooldown_prevents_rewarning(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;

        // 2 succeeded + 5 failed = 71% (warning range, not disable)
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

        // Phase 1: trigger warning
        let (subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();
        let sub = subs.iter().find(|s| s.subscription_id == sub_id).unwrap();
        let _ = process_subscription(&mut tx, &config, sub).await.unwrap();
        if let Some(ts) = max_completed {
            super::super::advance_cursor(&mut tx, ts).await.unwrap();
        }

        // Insert a resolved event with created_at = now() (within cooldown)
        sqlx::query!(
            "INSERT INTO webhook.subscription_health_event (subscription__id, status, cause) VALUES ($1, 'resolved', 'auto')",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        // Phase 2: buckets still have high failure rate, but cooldown blocks re-warning
        let (subs2, _) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();
        let sub2 = subs2
            .iter()
            .find(|s| s.subscription_id == sub_id)
            .expect("subscription should still be suspect via bucket failure count");
        let actions = process_subscription(&mut tx, &config, sub2).await.unwrap();

        assert!(
            !actions.iter().any(|a| matches!(
                a,
                crate::health_monitor::notifications::HealthAction::Warning(_)
            )),
            "cooldown should prevent re-warning"
        );

        let warning_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM webhook.subscription_health_event WHERE subscription__id = $1 AND status = 'warning'"#,
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(
            warning_count, 1,
            "no new warning event should have been inserted during cooldown"
        );
    }
}
