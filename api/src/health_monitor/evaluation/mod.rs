//! Health monitor evaluation pipeline.
//!
//! **What it does**: monitors webhook delivery success/failure rates to
//! auto-warn and auto-disable unhealthy subscription endpoints.
//!
//! **How it works at a high level** (one "tick"):
//!   1. Read the cursor — a bookmark saying "I've already processed all
//!      deliveries up to this timestamp."
//!   2. Scan `request_attempt` for deliveries newer than the cursor.
//!   3. Group those deliveries into time buckets per subscription
//!      (a bucket = a group of deliveries bounded by a max duration OR a max
//!      message count, whichever comes first — e.g. "10:00–10:05, 50 ok, 3 failed").
//!   4. Close buckets that are full (exceeded duration or message count).
//!   5. Identify "suspects" — subscriptions with enough recent failures to
//!      potentially need a warning or to be disabled.
//!   6. Compute each suspect's failure rate over a sliding window (the last N
//!      minutes of buckets, configured by `time_window`).
//!
//! The caller (mod.rs) then feeds each suspect into the state machine, which
//! decides whether to warn, disable, or resolve, and finally advances the
//! cursor so the next tick only looks at newer deliveries.
//!
//! Production code lives in two sub-modules:
//! - [`decision`]: cursor advancement and failure-percent reset side effects
//! - the orchestrator [`fetch_subscription_health_stats`] in this file, which
//!   stitches together [`crate::health_monitor::queries`] (bucket lifecycle,
//!   suspect detection, failure-rate computation) and the
//!   [`crate::health_monitor::state_machine`] (warn / disable / recover /
//!   cooldown transitions)
//!
//! The black-box integration tests are split by behavioral focus rather than
//! by file structure:
//! - [`bucket_tests`]: bucket population, closing, and retention cleanup
//!   (exercising [`crate::health_monitor::queries`] `upsert_buckets` /
//!   `close_full_buckets`)
//! - [`window_tests`]: adaptive-window flow — two-pass warning then
//!   recovery / cooldown evaluated across the `time_window` (exercising the
//!   [`crate::health_monitor::state_machine`] end-to-end)
//! - [`threshold_tests`]: threshold-driven suspect tracking — the UNION
//!   behavior in [`crate::health_monitor::queries::find_suspects`] that keeps
//!   a previously-warned subscription in the suspect set even after its
//!   bucket failure rate drops, which is what lets the state machine fire the
//!   Recovered transition

use chrono::{DateTime, Utc};

use super::HealthMonitorConfig;
use super::queries;

pub use queries::SubscriptionHealth;

mod decision;

pub use decision::{advance_cursor, reset_healthy_failure_percent};

#[cfg(test)]
mod bucket_tests;
#[cfg(test)]
mod threshold_tests;
#[cfg(test)]
mod window_tests;

#[cfg(test)]
pub(super) mod test_helpers;

/// Runs the full evaluation pipeline for one tick: read cursor, ingest new
/// deliveries, bucket them, close full buckets, find suspects, compute failure rates.
///
/// Returns `(suspects, max_completed_at)` where `max_completed_at` is the
/// timestamp the caller should pass to `advance_cursor` after committing the
/// transaction.
pub async fn fetch_subscription_health_stats(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<(Vec<SubscriptionHealth>, Option<DateTime<Utc>>), sqlx::Error> {
    // 1. Read the cursor — "where did I stop last time?"
    let cursor = queries::read_cursor(tx).await?;

    // 2. Scan for new deliveries since the cursor (capped to avoid long queries)
    let deltas = queries::ingest_deltas(tx, cursor, config.max_delta_rows_per_tick).await?;
    let max_completed_at = deltas.iter().filter_map(|d| d.max_completed_at).max();

    // 3. Pour those delivery counts into open buckets (one bucket per subscription)
    // Skip if empty — upsert_buckets would produce an empty VALUES clause
    if !deltas.is_empty() {
        queries::upsert_buckets(tx, &deltas).await?;
    }

    // 4. Close any bucket that exceeded its time or message limit
    queries::close_full_buckets(tx, config).await?;

    // 5. Find subscriptions that might be unhealthy (or were previously warned)
    let suspect_ids = queries::find_suspects(tx, config).await?;

    // 6. Compute failure rates for each suspect
    if suspect_ids.is_empty() {
        return Ok((Vec::new(), max_completed_at));
    }
    let subscriptions = queries::compute_failure_rates(tx, &suspect_ids, config).await?;

    Ok((subscriptions, max_completed_at))
}
