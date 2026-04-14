//! Aggregation of recent `webhook.request_attempt` rows — the first step
//! of the evaluation pipeline.

use chrono::{DateTime, Utc};
use sqlx::query_as;
use uuid::Uuid;

/// One row per subscription from the request_attempt scan: how many attempts
/// completed since the cursor, split into total vs failed.
#[derive(Debug)]
pub struct RequestAttemptAggregate {
    pub subscription_id: Uuid,
    pub total: i64,
    pub failed: i64,
    /// The latest completion timestamp in this batch — used to advance the
    /// cursor after the transaction commits so the next tick skips these rows.
    pub max_completed_at: Option<DateTime<Utc>>,
}

/// Scans `webhook.request_attempt` for rows completed after the cursor and
/// aggregates them per subscription.
///
/// The LIMIT caps the batch size to avoid long-running queries on high-traffic
/// instances — any remaining rows will be picked up on the next tick.
pub async fn aggregate_recent_request_attempts(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    cursor: DateTime<Utc>,
    max_rows: u32,
) -> Result<Vec<RequestAttemptAggregate>, sqlx::Error> {
    let max_rows = i64::from(max_rows);
    query_as!(
        RequestAttemptAggregate,
        r#"
            with new_request_attempts_window as (
                select subscription__id, failed_at, succeeded_at
                from webhook.request_attempt
                where coalesce(succeeded_at, failed_at) > $1
                  and (succeeded_at is not null or failed_at is not null)
                order by coalesce(succeeded_at, failed_at)
                limit $2
            )
            select subscription__id as "subscription_id!",
                   count(*) as "total!",
                   count(failed_at) as "failed!",
                   max(coalesce(succeeded_at, failed_at)) as "max_completed_at"
            from new_request_attempts_window
            group by subscription__id
        "#,
        cursor,
        max_rows,
    )
    .fetch_all(&mut **tx)
    .await
}
