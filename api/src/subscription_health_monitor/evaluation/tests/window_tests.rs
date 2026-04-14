//! Adaptive-window state-machine flow tests: two-pass warning then
//! recovery / anti-flap evaluated across `failure_rate_window`.
//!
//! Exercises
//! [`crate::subscription_health_monitor::evaluation::snapshot_subscription_healths`] together
//! with [`crate::subscription_health_monitor::state_machine`] end-to-end.

use chrono::Utc;
use sqlx::PgPool;

use super::helpers::{insert_test_fixtures, process_subscription, set_cursor, test_config};
use crate::subscription_health_monitor::evaluation::snapshot_subscription_healths;
use crate::subscription_health_monitor::state_machine::PlannedAction;

/// Recovery triggers a resolved health event.
#[sqlx::test(migrations = "./migrations")]
async fn test_recovery_triggers_resolved_event(pool: PgPool) {
    let config = test_config();
    let now = Utc::now();
    let cursor_past = now - chrono::Duration::hours(2);

    let mut tx = pool.begin().await.unwrap();
    set_cursor(&mut tx, cursor_past).await;

    // 2 succeeded + 5 failed = 71% failure (>50% warning, <90% disable).
    let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

    // Phase 1: evaluate + process -> warning (not disable, since 71% < 90%).
    let (subs, _) = snapshot_subscription_healths(&mut tx, &config)
        .await
        .unwrap();
    let sub = subs
        .iter()
        .find(|s| s.subscription_id == sub_id)
        .expect("subscription should be a candidate");
    let actions = process_subscription(&mut tx, &config, sub).await.unwrap();
    assert!(
        actions.contains(&PlannedAction::EmitWarning),
        "first pass should emit Warning"
    );
    assert!(
        !actions.contains(&PlannedAction::EmitDisabled),
        "first pass should NOT disable (71% < 90%)"
    );

    let warning_count = sqlx::query_scalar!(
        r#"
            select count(*) as "count!"
            from webhook.subscription_health_event
            where subscription__id = $1 and status = 'warning'
        "#,
        sub_id,
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(warning_count, 1, "should have exactly one warning event");

    // Phase 2: replace buckets with low failure rate.
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
            values ($1, now(), 20, 1)
        "#,
        sub_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    let (subs2, _) = snapshot_subscription_healths(&mut tx, &config)
        .await
        .unwrap();
    let sub2 = subs2
        .iter()
        .find(|s| s.subscription_id == sub_id)
        .expect("warned sub should still be a candidate");
    assert!(
        sub2.failure_percent < 50.0,
        "failure rate should be low now"
    );
    let actions2 = process_subscription(&mut tx, &config, sub2).await.unwrap();

    let resolved_count = sqlx::query_scalar!(
        r#"
            select count(*) as "count!"
            from webhook.subscription_health_event
            where subscription__id = $1 and status = 'resolved' and cause = 'auto'
        "#,
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
        actions2.contains(&PlannedAction::EmitResolved),
        "second pass should emit Resolved"
    );
}

/// Anti-flap window prevents re-warning after a recent resolved event.
#[sqlx::test(migrations = "./migrations")]
async fn test_anti_flap_prevents_rewarning(pool: PgPool) {
    let config = test_config();
    let now = Utc::now();
    let cursor_past = now - chrono::Duration::hours(2);

    let mut tx = pool.begin().await.unwrap();
    set_cursor(&mut tx, cursor_past).await;

    // 2 succeeded + 5 failed = 71% (warning range, not disable).
    let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 2, 5, now).await;

    // Phase 1: trigger warning.
    let (subs, _) = snapshot_subscription_healths(&mut tx, &config)
        .await
        .unwrap();
    let sub = subs.iter().find(|s| s.subscription_id == sub_id).unwrap();
    let _ = process_subscription(&mut tx, &config, sub).await.unwrap();

    // Insert a resolved event with created_at = now() (within anti-flap).
    sqlx::query!(
        r#"
            insert into webhook.subscription_health_event (subscription__id, status, cause)
            values ($1, 'resolved', 'auto')
        "#,
        sub_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    // Phase 2: buckets still have high failure rate, but anti-flap blocks re-warning.
    let (subs2, _) = snapshot_subscription_healths(&mut tx, &config)
        .await
        .unwrap();
    let sub2 = subs2
        .iter()
        .find(|s| s.subscription_id == sub_id)
        .expect("subscription should still be a candidate via bucket failure count");
    let actions = process_subscription(&mut tx, &config, sub2).await.unwrap();

    assert!(
        !actions.contains(&PlannedAction::EmitWarning),
        "anti-flap should prevent re-warning"
    );

    let warning_count = sqlx::query_scalar!(
        r#"
            select count(*) as "count!"
            from webhook.subscription_health_event
            where subscription__id = $1 and status = 'warning'
        "#,
        sub_id,
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(
        warning_count, 1,
        "no new warning event should have been inserted during the anti-flap window"
    );
}
