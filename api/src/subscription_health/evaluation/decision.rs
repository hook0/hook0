//! Decision side-effects of the evaluation pipeline.
//!
//! These are the writes produced after the pipeline has made up its mind:
//! advancing the processing cursor, and clearing the `failure_percent` column
//! on subscriptions that are no longer suspects (so the UI badge does not
//! linger on a now-healthy endpoint).

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Advances the cursor to the given timestamp.
///
/// The cursor is a singleton row (exactly one row in the table). The WHERE clause
/// ensures we never move the cursor backwards — if two ticks overlap, the later
/// timestamp wins.
pub async fn advance_cursor(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    max_completed_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE webhook.subscription_health_monitor_cursor
        SET last_processed_at = $1
        WHERE cursor__id = 1
          AND $1 > last_processed_at
        "#,
        max_completed_at,
    )
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        tracing::debug!("Cursor not advanced (already at or past target)");
    }

    Ok(())
}

/// Resets failure_percent to NULL for subscriptions that are NOT suspects.
///
/// Why? The frontend displays failure_percent as a badge. If a subscription
/// was briefly unhealthy but recovered, we clear its stale failure data so
/// the UI doesn't show an outdated red badge on a now-healthy endpoint.
pub async fn reset_healthy_failure_percent(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    suspect_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE webhook.subscription
        SET failure_percent = NULL
        WHERE failure_percent IS NOT NULL
          AND subscription__id NOT IN (SELECT unnest($1::uuid[]))
        "#,
        suspect_ids,
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sqlx::PgPool;

    use super::super::fetch_subscription_health_stats;
    use super::super::test_helpers::{insert_test_fixtures, set_cursor, test_config};
    use super::{advance_cursor, reset_healthy_failure_percent};

    /// Cursor advances after a health tick.
    #[sqlx::test(migrations = "./migrations")]
    async fn test_cursor_advances(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, _app_id, _sub_id) = insert_test_fixtures(&mut tx, 2, 1, now).await;

        // Read cursor before
        let cursor_before = sqlx::query_scalar!(
            "SELECT last_processed_at FROM webhook.subscription_health_monitor_cursor WHERE cursor__id = 1",
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(cursor_before, cursor_past);

        // Run health eval + advance cursor
        let (_subs, max_completed) = fetch_subscription_health_stats(&mut tx, &config)
            .await
            .unwrap();
        if let Some(ts) = max_completed {
            advance_cursor(&mut tx, ts).await.unwrap();
        }

        // Read cursor after
        let cursor_after = sqlx::query_scalar!(
            "SELECT last_processed_at FROM webhook.subscription_health_monitor_cursor WHERE cursor__id = 1",
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        assert!(
            cursor_after > cursor_before,
            "cursor should have advanced: before={cursor_before}, after={cursor_after}"
        );
        // tx dropped -- automatic rollback
    }

    /// reset_healthy_failure_percent clears failure_percent for non-suspects.
    #[sqlx::test(migrations = "./migrations")]
    async fn test_reset_failure_percent_for_non_suspects(pool: PgPool) {
        let now = Utc::now();

        let mut tx = pool.begin().await.unwrap();
        let (_org_id, _app_id, sub_id) = insert_test_fixtures(&mut tx, 5, 0, now).await;

        // Manually set failure_percent on the subscription
        sqlx::query!(
            "UPDATE webhook.subscription SET failure_percent = 50.0 WHERE subscription__id = $1",
            sub_id,
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        let fp_before = sqlx::query_scalar!(
            "SELECT failure_percent FROM webhook.subscription WHERE subscription__id = $1",
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(fp_before, Some(50.0));

        // Call reset with empty suspect list
        let rows = reset_healthy_failure_percent(&mut tx, &[]).await.unwrap();
        assert!(rows >= 1, "should have reset at least 1 subscription");

        let fp_after = sqlx::query_scalar!(
            "SELECT failure_percent FROM webhook.subscription WHERE subscription__id = $1",
            sub_id,
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(fp_after, None, "failure_percent should be NULL after reset");
        // tx dropped -- automatic rollback
    }
}
