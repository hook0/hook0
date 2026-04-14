//! Bucket lifecycle: upsert delivery aggregates into open buckets, close
//! buckets that exceed their duration or message-count limit, and drop
//! buckets older than the retention window.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, query};
use uuid::Uuid;

use super::super::runner::SubscriptionHealthMonitorConfig;
use super::deltas::RequestAttemptAggregate;

/// Adds new delivery aggregates to each subscription's currently-open bucket.
///
/// Two steps:
///   1. Pre-fetch each subscription's open bucket start (one query).
///   2. Run a single compile-time-checked INSERT using UNNEST over four
///      parallel arrays (subscription_ids, bucket_starts, totals, faileds).
///
/// Subscriptions without an open bucket get a fresh one starting `now()`.
/// The ON CONFLICT clause adds counts to existing buckets rather than
/// replacing — this matters when the same (subscription, bucket_start) pair
/// appears multiple times across consecutive ticks.
pub async fn upsert_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    aggregates: &[RequestAttemptAggregate],
) -> Result<(), sqlx::Error> {
    if aggregates.is_empty() {
        return Ok(());
    }

    let input_ids: Vec<Uuid> = aggregates.iter().map(|a| a.subscription_id).collect();

    // Step 1: find each subscription's currently-open bucket start.
    let open_rows = query!(
        r#"
            select subscription__id as "subscription_id!", bucket_start as "bucket_start!"
            from webhook.subscription_health_bucket
            where subscription__id = any($1)
              and bucket_end is null
        "#,
        &input_ids,
    )
    .fetch_all(&mut **tx)
    .await?;

    let open_buckets: HashMap<Uuid, DateTime<Utc>> = open_rows
        .into_iter()
        .map(|r| (r.subscription_id, r.bucket_start))
        .collect();
    let now = Utc::now();

    // Step 2: build four parallel vectors for UNNEST.
    let len = aggregates.len();
    let mut sub_ids: Vec<Uuid> = Vec::with_capacity(len);
    let mut bucket_starts: Vec<DateTime<Utc>> = Vec::with_capacity(len);
    let mut totals: Vec<i32> = Vec::with_capacity(len);
    let mut faileds: Vec<i32> = Vec::with_capacity(len);

    for agg in aggregates {
        let start = open_buckets
            .get(&agg.subscription_id)
            .copied()
            .unwrap_or(now);
        sub_ids.push(agg.subscription_id);
        bucket_starts.push(start);
        totals.push(i32::try_from(agg.total).unwrap_or(i32::MAX));
        faileds.push(i32::try_from(agg.failed).unwrap_or(i32::MAX));
    }

    query!(
        r#"
            insert into webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count)
            select * from unnest($1::uuid[], $2::timestamptz[], $3::int4[], $4::int4[])
            on conflict (subscription__id, bucket_start) do update set
                total_count = subscription_health_bucket.total_count + excluded.total_count,
                failed_count = subscription_health_bucket.failed_count + excluded.failed_count
        "#,
        &sub_ids,
        &bucket_starts,
        &totals,
        &faileds,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Closes buckets that are "full" — either they've been open too long
/// (exceeded `bucket_duration`) or they contain too many deliveries
/// (exceeded `bucket_max_messages`).
///
/// A closed bucket (`bucket_end IS NOT NULL`) is frozen: no more delivery
/// results will be added to it. New deliveries for that subscription will go
/// into a fresh bucket on the next tick.
///
/// Why close buckets? Without closing, a single bucket would grow indefinitely.
/// Closing creates discrete time windows that let us compute failure rates
/// over a sliding window (e.g. "failure rate in the last hour" = sum of the
/// last 12 five-minute buckets).
pub async fn close_full_buckets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let bucket_duration_secs = config.bucket_duration.as_secs_f64();
    let bucket_max_messages = i32::try_from(config.bucket_max_messages).unwrap_or(i32::MAX);

    let result = query!(
        r#"
            update webhook.subscription_health_bucket
            set bucket_end = now()
            where bucket_end is null
              and (bucket_start < now() - make_interval(secs => $1)
                   or total_count >= $2)
        "#,
        bucket_duration_secs,
        bucket_max_messages,
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

/// Drops buckets (open or closed) older than `bucket_retention`.
///
/// Runs once per day from the monitor loop. The
/// `idx_subscription_health_bucket_start` index makes this an index scan,
/// not a full table scan, even as the table grows.
pub async fn cleanup_old_buckets(
    db: &PgPool,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_secs = config.bucket_retention.as_secs_f64();

    let result = query!(
        r#"
            delete from webhook.subscription_health_bucket
            where bucket_start < now() - make_interval(secs => $1)
        "#,
        retention_secs,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
