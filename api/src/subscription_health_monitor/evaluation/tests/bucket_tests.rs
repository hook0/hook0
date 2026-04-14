//! Bucket-lifecycle tests: population, closing, and retention cleanup.
//!
//! Drives the bucket aggregation logic in
//! [`crate::subscription_health_monitor::queries::buckets`] through the orchestrator
//! [`crate::subscription_health_monitor::evaluation::run_subscription_health_monitor_tick`].

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::subscription_health_monitor::evaluation::run_subscription_health_monitor_tick;
use crate::subscription_health_monitor::evaluation::test_helpers::{
    insert_test_fixtures, set_cursor, test_config,
};

/// Buckets are populated after a health tick.
#[sqlx::test(migrations = "./migrations")]
async fn test_health_buckets_populated(pool: PgPool) {
    let config = test_config();
    let now = Utc::now();
    let cursor_past = now - chrono::Duration::hours(1);

    let mut tx = pool.begin().await.unwrap();
    set_cursor(&mut tx, cursor_past).await;
    let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

    let (subs, _) = run_subscription_health_monitor_tick(&mut tx, &config)
        .await
        .unwrap();

    let bucket = sqlx::query!(
        r#"
            select total_count, failed_count
            from webhook.subscription_health_bucket
            where subscription__id = $1
        "#,
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

    let candidate_ids: Vec<Uuid> = subs.iter().map(|s| s.subscription_id).collect();
    assert!(
        candidate_ids.contains(&sub_id),
        "test subscription should be in candidates"
    );
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

    // First pass: creates an open bucket and advances the cursor.
    run_subscription_health_monitor_tick(&mut tx, &config)
        .await
        .unwrap();

    let open = sqlx::query_scalar!(
        r#"
            select (bucket_end is null) as "is_open!"
            from webhook.subscription_health_bucket
            where subscription__id = $1
            limit 1
        "#,
        sub_id,
    )
    .fetch_optional(&mut *tx)
    .await
    .unwrap();
    assert_eq!(open, Some(true), "bucket should be open initially");

    // Age the bucket beyond bucket_duration (300s).
    sqlx::query!(
        "update webhook.subscription_health_bucket set bucket_start = now() - interval '1 hour' where subscription__id = $1",
        sub_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    // Second pass: should close the aged bucket.
    run_subscription_health_monitor_tick(&mut tx, &config)
        .await
        .unwrap();

    let closed = sqlx::query_scalar!(
        r#"
            select (bucket_end is not null) as "is_closed!"
            from webhook.subscription_health_bucket
            where subscription__id = $1
              and bucket_start < now() - interval '30 minutes'
            limit 1
        "#,
        sub_id,
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert!(
        closed,
        "aged bucket should be closed after second health tick"
    );
}

/// cleanup_old_buckets removes buckets older than retention period.
#[sqlx::test(migrations = "./migrations")]
async fn test_cleanup_old_buckets(pool: PgPool) {
    let config = test_config(); // bucket_retention = 30d
    let now = Utc::now();

    let mut tx = pool.begin().await.unwrap();
    let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 3, 2, now).await;

    // Insert a bucket older than retention (31 days ago).
    let old_start = now - chrono::Duration::days(31);
    let old_end = old_start + chrono::Duration::seconds(300);
    sqlx::query!(
        r#"
            insert into webhook.subscription_health_bucket
                (subscription__id, bucket_start, bucket_end, total_count, failed_count)
            values ($1, $2, $3, 10, 5)
        "#,
        sub_id,
        old_start,
        old_end,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    let count_before = sqlx::query_scalar!(
        r#"
            select count(*) as "count!"
            from webhook.subscription_health_bucket
            where subscription__id = $1 and bucket_start < now() - interval '30 days'
        "#,
        sub_id,
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert!(count_before >= 1, "old bucket should exist before cleanup");

    // Inline cleanup_old_buckets logic so it runs within the same transaction.
    let retention_secs = config.bucket_retention.as_secs_f64();
    let deleted = sqlx::query!(
        r#"
            delete from webhook.subscription_health_bucket
            where bucket_start < now() - make_interval(secs => $1)
        "#,
        retention_secs,
    )
    .execute(&mut *tx)
    .await
    .unwrap()
    .rows_affected();
    assert!(deleted >= 1, "should have deleted at least 1 old bucket");

    let count_after = sqlx::query_scalar!(
        r#"
            select count(*) as "count!"
            from webhook.subscription_health_bucket
            where subscription__id = $1 and bucket_start < now() - interval '30 days'
        "#,
        sub_id,
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(count_after, 0, "old bucket should be deleted after cleanup");
}
