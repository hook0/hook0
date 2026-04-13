//! Shared test fixtures for the `evaluation/` sub-module tests.
//!
//! All items are `pub(in crate::health_monitor::evaluation)` so the sibling
//! test modules can reach them but nothing outside of `evaluation/` can.

use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::health_monitor::HealthMonitorConfig;

mod fixtures;

pub(in crate::health_monitor::evaluation) use fixtures::insert_test_fixtures;

pub(in crate::health_monitor::evaluation) async fn setup_test_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&url).await.ok()
}

pub(in crate::health_monitor::evaluation) fn test_config() -> HealthMonitorConfig {
    HealthMonitorConfig {
        interval: Duration::from_secs(60),
        warning_failure_percent: 50,
        disable_failure_percent: 90,
        time_window: Duration::from_secs(3600),
        min_sample_size: 1,
        warning_cooldown: Duration::from_secs(3600),
        retention_period_days: 30,
        bucket_duration: Duration::from_secs(300),
        bucket_max_messages: 100,
        bucket_retention_days: 30,
        max_delta_rows_per_tick: 50_000,
    }
}

/// Sets the cursor inside the given transaction.
pub(in crate::health_monitor::evaluation) async fn set_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ts: DateTime<Utc>,
) {
    sqlx::query!(
        "UPDATE webhook.health_monitor_cursor SET last_processed_at = $1 WHERE cursor__id = 1",
        ts,
    )
    .execute(&mut **tx)
    .await
    .unwrap();
}
