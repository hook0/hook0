use sqlx::PgPool;

use super::HealthMonitorConfig;

/// Removes old closed buckets beyond the configured retention period.
pub async fn cleanup_old_buckets(
    db: &PgPool,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_days = config.bucket_retention_days as i32;

    let result = sqlx::query(
        r#"
        DELETE FROM webhook.subscription_health_bucket
        WHERE bucket_start < now() - make_interval(days => $1)
        "#,
    )
    .bind(retention_days)
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

    let result = sqlx::query(
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
    )
    .bind(retention_period_days)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
