//! Per-tick evaluation pipeline for the subscription health monitor.
//!
//! Dataflow:
//!
//!   ┌────────────────────────────┐
//!   │   read_evaluation_cursor   │  "where did I stop last time?"
//!   └─────────────┬──────────────┘
//!                 ▼
//!   ┌──────────────────────────────────────┐
//!   │  aggregate_recent_request_attempts   │  scan new rows, capped per tick
//!   └─────────────────┬────────────────────┘
//!                     ▼
//!   ┌─────────────────┐
//!   │  upsert_buckets │  one open bucket per subscription
//!   └────────┬────────┘
//!            ▼
//!   ┌──────────────────────┐
//!   │  close_full_buckets  │  freeze buckets past duration/count limit
//!   └──────────┬───────────┘
//!              ▼
//!   ┌──────────────────────────────────────────────┐
//!   │ find_subscriptions_pending_health_evaluation │  failing ∪ currently-warned
//!   └─────────────────────┬────────────────────────┘
//!                         ▼
//!   ┌────────────────────────┐
//!   │  compute_failure_rates │  Vec<SubscriptionHealth> for the state machine
//!   └────────────┬───────────┘
//!                ▼
//!   ┌───────────────────────────────┐
//!   │ reset_healthy_failure_percent │  clear the cached rate on recovered subs
//!   └─────────────┬─────────────────┘
//!                 ▼
//!   ┌────────────────────────────┐
//!   │  advance_evaluation_cursor │  bookmark progress for the next tick
//!   └────────────────────────────┘
//!
//! The whole pipeline runs inside a single transaction owned by the caller
//! in [`super::runner`]. The state machine + side-effect dispatch happen
//! after this function returns, still inside the same transaction, so a
//! crash between any two stages rolls the whole tick back.

use super::queries::{self, SubscriptionHealth};
use super::runner::SubscriptionHealthConfig;

#[cfg(test)]
pub(super) mod test_helpers;

#[cfg(test)]
mod tests;

/// One full evaluation tick. Returns the subscriptions the state machine
/// should judge — the caller is responsible for running the state machine
/// and dispatching the resulting `PlannedAction`s.
pub async fn run_evaluation_tick(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    let cursor = queries::read_evaluation_cursor(tx).await?;

    let aggregates = queries::aggregate_recent_request_attempts(
        tx,
        cursor,
        config.max_request_attempts_scanned_per_tick,
    )
    .await?;
    let max_completed_at = aggregates.iter().filter_map(|a| a.max_completed_at).max();

    if !aggregates.is_empty() {
        queries::upsert_buckets(tx, &aggregates).await?;
    }
    queries::close_full_buckets(tx, config).await?;

    let candidate_ids = queries::find_subscriptions_pending_health_evaluation(tx, config).await?;
    let subscriptions = if candidate_ids.is_empty() {
        Vec::new()
    } else {
        queries::compute_failure_rates(tx, &candidate_ids, config).await?
    };

    let active_ids: Vec<_> = subscriptions.iter().map(|s| s.subscription_id).collect();
    queries::reset_healthy_failure_percent(tx, &active_ids).await?;

    if let Some(ts) = max_completed_at {
        queries::advance_evaluation_cursor(tx, ts).await?;
    }

    Ok(subscriptions)
}
