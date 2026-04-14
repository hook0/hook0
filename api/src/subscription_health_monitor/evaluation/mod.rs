//! Per-tick evaluation pipeline for the subscription health monitor.
//!
//! [`snapshot_subscription_healths`] is the single entry point the runner
//! calls — it ingests recent delivery outcomes, folds them into buckets,
//! picks the subscriptions worth judging, and returns their current failure
//! rates. The internal stages are implementation details, documented here
//! so readers can drill in when needed but hidden from callers elsewhere:
//!
//!   1. Read the evaluation cursor (bookmark from the previous tick).
//!   2. Aggregate `webhook.request_attempt` rows newer than the cursor,
//!      capped at `max_request_attempts_per_tick` per tick.
//!   3. Upsert the new per-subscription counts into the currently-open
//!      bucket for each subscription.
//!   4. Close buckets that exceeded `bucket_duration` or `bucket_max_messages`.
//!   5. In ONE query, compute the candidate set AND their failure rates
//!      over `failure_rate_window` (candidates = subs with enough recent
//!      failures ∪ subs currently in warning).
//!   6. Reset `webhook.subscription.failure_percent` for every row that is
//!      NOT in the current candidate set, so the API's cached rate never
//!      shows stale data on recovered subs.
//!   7. Advance the evaluation cursor to the latest processed timestamp.
//!
//! The whole function runs inside a single transaction owned by the caller
//! in [`super::runner`]. The state machine + side-effect dispatch happen
//! after this function returns, still inside the same transaction, so a
//! crash between any two stages rolls the whole tick back.

use super::queries::{self, SubscriptionHealth};
use super::runner::SubscriptionHealthMonitorConfig;

#[cfg(test)]
mod tests;

/// Point-in-time snapshot of every subscription the state machine should
/// judge this tick.
///
/// Returns:
/// - the subscriptions with their computed failure rate (caller dispatches
///   the resulting `PlannedAction`s);
/// - a `hit_cap` flag that's true when the `request_attempt` scan reached
///   `max_request_attempts_per_tick`, signalling the monitor loop to chain
///   another tick immediately instead of sleeping.
pub(super) async fn snapshot_subscription_healths(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<(Vec<SubscriptionHealth>, bool), sqlx::Error> {
    let cursor = queries::read_evaluation_cursor(tx).await?;

    let aggregates = queries::aggregate_recent_request_attempts(
        tx,
        cursor,
        config.max_request_attempts_per_tick,
    )
    .await?;
    let max_completed_at = aggregates.iter().filter_map(|a| a.max_completed_at).max();

    // Sum of `total` across aggregates = number of request_attempt rows pulled
    // into the capped CTE. When it equals `max_request_attempts_per_tick`, the
    // LIMIT clipped us and there's still backlog — the caller will chain
    // another tick without sleeping.
    let total_scanned: i64 = aggregates.iter().map(|a| a.total).sum();
    let hit_cap = total_scanned >= i64::from(config.max_request_attempts_per_tick);

    if !aggregates.is_empty() {
        queries::upsert_buckets(tx, &aggregates).await?;
    }
    queries::close_full_buckets(tx, config).await?;

    let subscriptions = queries::compute_candidate_healths(tx, config).await?;

    let active_ids: Vec<_> = subscriptions.iter().map(|s| s.subscription_id).collect();
    queries::reset_healthy_failure_percent(tx, &active_ids).await?;

    if let Some(ts) = max_completed_at {
        queries::advance_evaluation_cursor(tx, ts).await?;
    }

    Ok((subscriptions, hit_cap))
}
