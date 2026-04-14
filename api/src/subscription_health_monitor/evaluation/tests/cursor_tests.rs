//! Cursor advancement tests — moved here from the old decision.rs inline tests.

use chrono::Utc;
use sqlx::PgPool;

use super::helpers::{insert_test_fixtures, set_cursor, test_config};
use crate::subscription_health_monitor::evaluation::run_subscription_health_monitor_tick;

/// Cursor advances after a health tick.
#[sqlx::test(migrations = "./migrations")]
async fn test_cursor_advances(pool: PgPool) {
    let config = test_config();
    let now = Utc::now();
    let cursor_past = now - chrono::Duration::hours(1);

    let mut tx = pool.begin().await.unwrap();
    set_cursor(&mut tx, cursor_past).await;
    let (_org_id, _app_id, _sub_id) = insert_test_fixtures(&mut tx, 2, 1, now).await;

    let cursor_before = sqlx::query_scalar!(
        "select last_processed_at from webhook.subscription_health_monitor_cursor where cursor__id = 1",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(cursor_before, cursor_past);

    run_subscription_health_monitor_tick(&mut tx, &config)
        .await
        .unwrap();

    let cursor_after = sqlx::query_scalar!(
        "select last_processed_at from webhook.subscription_health_monitor_cursor where cursor__id = 1",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();

    assert!(
        cursor_after > cursor_before,
        "cursor should have advanced: before={cursor_before}, after={cursor_after}"
    );
}
