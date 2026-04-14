//! Subscription state write tests — moved here from the old decision.rs
//! inline tests.

use chrono::Utc;
use sqlx::PgPool;

use crate::subscription_health_monitor::evaluation::test_helpers::insert_test_fixtures;
use crate::subscription_health_monitor::queries::reset_healthy_failure_percent;

/// `reset_healthy_failure_percent` clears failure_percent for subscriptions
/// that are not in the active set.
#[sqlx::test(migrations = "./migrations")]
async fn test_reset_failure_percent_for_non_active_subscriptions(pool: PgPool) {
    let now = Utc::now();

    let mut tx = pool.begin().await.unwrap();
    let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 5, 0, now).await;

    // Manually set failure_percent on the subscription.
    sqlx::query!(
        "update webhook.subscription set failure_percent = 50.0 where subscription__id = $1",
        sub_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    let fp_before = sqlx::query_scalar!(
        "select failure_percent from webhook.subscription where subscription__id = $1",
        sub_id,
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(fp_before, Some(50.0));

    // Call reset with empty active list — everyone gets cleared.
    let rows = reset_healthy_failure_percent(&mut tx, &[]).await.unwrap();
    assert!(rows >= 1, "should have reset at least 1 subscription");

    let fp_after = sqlx::query_scalar!(
        "select failure_percent from webhook.subscription where subscription__id = $1",
        sub_id,
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(fp_after, None, "failure_percent should be NULL after reset");
}
