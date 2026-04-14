//! Periodic cleanup of stale health data — old buckets and resolved events.
//!
//! Called from `mod.rs` once per day (not every tick) to keep tables lean.
//! Both queries use dedicated indexes (`idx_subscription_health_bucket_start`
//! and `idx_subscription_health_event_cleanup`) so they're index scans, not
//! full table scans.

use sqlx::PgPool;

use super::HealthMonitorConfig;

/// Removes all buckets (open or closed) older than the configured retention period.
///
/// This includes both closed buckets (`bucket_end IS NOT NULL`) and open ones
/// that have gone stale. The `idx_subscription_health_bucket_start` index
/// on `bucket_start` makes this an index scan, not a full table scan.
pub async fn cleanup_old_buckets(
    db: &PgPool,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_days = config.bucket_retention_days as i32;

    let result = sqlx::query!(
        r#"
        DELETE FROM webhook.subscription_health_bucket
        WHERE bucket_start < now() - make_interval(days => $1)
        "#,
        retention_days,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

/// Removes resolved health events older than the configured retention period,
/// keeping at least the latest event per subscription.
///
/// Example: a subscription with events at -100d (resolved), -80d (resolved), -10d (warning)
/// deletes only the -100d row; the -80d row is kept because -10d is newer.
pub async fn cleanup_resolved_health_events(
    db: &PgPool,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_period_days = config.retention_period_days as i32;

    let result = sqlx::query!(
        r#"
        DELETE FROM webhook.subscription_health_event d
        WHERE d.created_at < now() - make_interval(days => $1)
          AND d.status = 'resolved'
          AND EXISTS (
            SELECT 1 FROM webhook.subscription_health_event newer
            WHERE newer.subscription__id = d.subscription__id
              AND newer.created_at > d.created_at
          )
        "#,
        retention_period_days,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
