//! Evaluation cursor — the singleton bookmark that tells the pipeline
//! "where did I stop processing request_attempts last time?".

use chrono::{DateTime, Utc};
use sqlx::{query, query_scalar};
use tracing::debug;

/// Reads the cursor timestamp. Everything newer than this value is "new work"
/// for the current tick.
///
/// The cursor lives in a singleton row (exactly one row, enforced by a CHECK
/// constraint on the primary key). It starts at '-infinity' so the first tick
/// picks up all historical deliveries.
pub async fn read_evaluation_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<DateTime<Utc>, sqlx::Error> {
    query_scalar!(
        "select last_processed_at from webhook.subscription_health_monitor_cursor where cursor__id = 1",
    )
    .fetch_one(&mut **tx)
    .await
}

/// Advances the cursor to the given timestamp.
///
/// The WHERE clause guarantees the cursor never moves backwards: if two ticks
/// overlap (e.g. a long tick held back by lock contention, then a quick one),
/// the later timestamp wins.
pub async fn advance_evaluation_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    max_completed_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let result = query!(
        r#"
            update webhook.subscription_health_monitor_cursor
            set last_processed_at = $1
            where cursor__id = 1
              and $1 > last_processed_at
        "#,
        max_completed_at,
    )
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        debug!("Evaluation cursor not advanced (already at or past target)");
    }

    Ok(())
}
