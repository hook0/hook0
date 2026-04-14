//! Threshold-driven candidate tracking tests.
//!
//! Exercises the UNION behavior in
//! [`crate::subscription_health_monitor::queries::find_subscriptions_pending_health_evaluation`]
//! that keeps a previously-warned subscription in the candidate set even
//! after its bucket failure rate drops — that's what lets the state machine
//! fire the Resolved transition.

use chrono::Utc;
use sqlx::PgPool;

use crate::subscription_health_monitor::evaluation::run_subscription_health_monitor_tick;
use crate::subscription_health_monitor::evaluation::test_helpers::{
    insert_test_fixtures, set_cursor, test_config,
};

/// Warned subscription still appears in candidates via UNION.
#[sqlx::test(migrations = "./migrations")]
async fn test_warned_subscription_in_candidates(pool: PgPool) {
    let config = test_config();
    let now = Utc::now();
    let cursor_past = now - chrono::Duration::hours(1);

    let mut tx = pool.begin().await.unwrap();
    set_cursor(&mut tx, cursor_past).await;
    let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 0, 5, now).await;

    // First pass: ingests deltas and advances the cursor.
    let (subs, _) = run_subscription_health_monitor_tick(&mut tx, &config)
        .await
        .unwrap();
    assert!(
        subs.iter().any(|s| s.subscription_id == sub_id),
        "subscription should be a candidate on first pass"
    );

    // Insert a warning health event (simulating state machine output).
    sqlx::query!(
        r#"
            insert into webhook.subscription_health_event (subscription__id, status, cause)
            values ($1, 'warning', 'auto')
        "#,
        sub_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    // Replace buckets with healthy data (0 failures).
    sqlx::query!(
        "delete from webhook.subscription_health_bucket where subscription__id = $1",
        sub_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query!(
        r#"
            insert into webhook.subscription_health_bucket
                (subscription__id, bucket_start, total_count, failed_count)
            values ($1, now(), 10, 0)
        "#,
        sub_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    // Second pass: cursor advanced, no re-ingestion. UNION picks up the
    // warned subscription even though its buckets are now healthy.
    let (subs2, _) = run_subscription_health_monitor_tick(&mut tx, &config)
        .await
        .unwrap();

    let found = subs2.iter().find(|s| s.subscription_id == sub_id);
    assert!(
        found.is_some(),
        "warned subscription should still appear in candidates via UNION"
    );
    assert!(
        found.unwrap().failure_percent < 50.0,
        "failure_percent should be low (subscription recovered)"
    );
}
