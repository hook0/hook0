//! Bucket lifecycle queries: upsert delivery deltas into open buckets and
//! close buckets that exceed the configured duration or message count.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::SubscriptionHealthConfig;
use super::DeltaRow;

/// Adds new delivery results to each subscription's current open bucket.
///
/// Two-step approach:
/// 1. Fetch each subscription's currently open bucket (if any) in one query
/// 2. Bulk upsert all delivery counts using QueryBuilder::push_values
///
/// For subscriptions without an open bucket, a new one is created starting now.
/// The ON CONFLICT clause adds counts to existing buckets rather than replacing.
pub async fn upsert_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    deltas: &[DeltaRow],
) -> Result<(), sqlx::Error> {
    // Nothing to insert — skip to avoid an empty VALUES clause
    if deltas.is_empty() {
        return Ok(());
    }

    let sub_ids: Vec<Uuid> = deltas.iter().map(|d| d.subscription_id).collect();
    let now = Utc::now();

    // Step 1: Find open buckets for all subscriptions in one query
    let open_rows = sqlx::query!(
        r#"
        SELECT subscription__id AS "subscription_id!", bucket_start AS "bucket_start!"
        FROM webhook.subscription_health_bucket
        WHERE subscription__id = ANY($1)
          AND bucket_end IS NULL
        "#,
        &sub_ids,
    )
    .fetch_all(&mut **tx)
    .await?;

    let open_buckets: std::collections::HashMap<Uuid, DateTime<Utc>> = open_rows
        .into_iter()
        .map(|r| (r.subscription_id, r.bucket_start))
        .collect();

    // Step 2: Bulk upsert — one INSERT with multiple VALUES rows
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        "INSERT INTO webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count) ",
    );

    qb.push_values(deltas, |mut b, d| {
        let bucket_start = open_buckets.get(&d.subscription_id).copied().unwrap_or(now);
        b.push_bind(d.subscription_id)
            .push_bind(bucket_start)
            .push_bind(d.total.min(i32::MAX as i64) as i32)
            .push_bind(d.failed.min(i32::MAX as i64) as i32);
    });

    qb.push(
        " ON CONFLICT (subscription__id, bucket_start) DO UPDATE SET \
         total_count = subscription_health_bucket.total_count + EXCLUDED.total_count, \
         failed_count = subscription_health_bucket.failed_count + EXCLUDED.failed_count",
    );

    qb.build().execute(&mut **tx).await?;

    Ok(())
}

/// Closes buckets that are "full" — either they've been open too long (exceeded
/// the configured duration) or they contain too many deliveries (exceeded the
/// configured max message count).
///
/// A closed bucket (bucket_end IS NOT NULL) is frozen: no more delivery results
/// will be added to it. New deliveries for that subscription will go into a
/// fresh bucket on the next tick.
///
/// Why close buckets? Without closing, a single bucket would grow indefinitely.
/// Closing creates discrete time windows that let us compute failure rates
/// over a sliding window (e.g., "failure rate in the last hour" = sum of
/// the last 12 five-minute buckets).
pub async fn close_full_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthConfig,
) -> Result<u64, sqlx::Error> {
    let bucket_duration_secs = config.bucket_duration.as_secs() as f64;
    let bucket_max_messages = config.bucket_max_messages as i32;

    let result = sqlx::query!(
        r#"
        UPDATE webhook.subscription_health_bucket
        SET bucket_end = now()
        WHERE bucket_end IS NULL
          AND (bucket_start < now() - make_interval(secs => $1)
               OR total_count >= $2)
        "#,
        bucket_duration_secs,
        bucket_max_messages,
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}
