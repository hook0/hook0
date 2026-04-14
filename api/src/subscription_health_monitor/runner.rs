//! Background loop + per-tick orchestrator for the subscription health
//! monitor.
//!
//! The loop:
//!   - Evaluates every subscription's recent delivery outcomes inside a
//!     single advisory-locked transaction so replicas can't stomp on each
//!     other.
//!   - Chains extra ticks immediately (no sleep) whenever the pipeline hits
//!     the `max_request_attempts_per_tick` cap — this lets us burn through a
//!     backlog without blocking on `interval` between scans.
//!   - Every 24 hours, runs a cleanup pass to keep the health tables lean.
//!   - Sleeps `config.interval` between normal ticks — total cycle time is
//!     `check duration + sleep`, not exactly `interval`.

use std::time::{Duration, Instant};

use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use super::errors::SubscriptionHealthMonitorError;
use super::evaluation;
use super::queries::{self, SubscriptionHealth};
use super::state_machine::{self, PlannedAction};
use super::types::{HealthEventCause, HealthStatus};

/// Arbitrary unique ID for `pg_try_advisory_xact_lock` — must not conflict
/// with any other advisory lock in the application.
const ADVISORY_LOCK_ID: i64 = 42_000_001;

/// Cleanup runs once per day (not every tick) to keep the health tables lean
/// without adding per-tick overhead. The monitor ticks are usually minutes
/// apart; cleanup scans are expensive so we amortize them over a day.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Hard cap on the number of chained ticks per wake-up. Without this, a tick
/// that consistently hits the scan cap would starve every other housekeeping
/// task (we hold `Semaphore::new(1)` across chained ticks). Ten gives the
/// monitor enough rope to burn through a modest backlog in one wake-up while
/// still yielding back to the rest of the housekeeping loop within a few
/// minutes worst case.
const MAX_CHAINED_TICKS: usize = 10;

/// Upper bound on how long a single evaluation tick may hold the advisory
/// lock + housekeeping permit before Postgres aborts it. Without this, a
/// pathological query (bad plan, huge backlog) could freeze every other
/// housekeeping task indefinitely. Set via `set local statement_timeout` at
/// the top of each tick's transaction.
const TICK_STATEMENT_TIMEOUT: &str = "5min";

/// Tuning knobs for the subscription health monitor — thresholds, bucketing,
/// retention.
#[derive(Clone)]
pub struct SubscriptionHealthMonitorConfig {
    pub interval: Duration,
    pub failure_percent_for_warning: u8,
    pub failure_percent_for_disable: u8,
    pub failure_rate_window: Duration,
    pub min_deliveries: u32,
    pub anti_flap_window: Duration,
    pub resolved_event_retention: Duration,
    pub bucket_duration: Duration,
    pub bucket_max_messages: u32,
    pub bucket_retention: Duration,
    pub max_request_attempts_per_tick: u32,
}

/// Outcome of a single `run_health_check` invocation. `hit_cap == true`
/// means the evaluation pipeline's scan reached `max_request_attempts_per_tick`
/// — a signal that there's still backlog to chew through and the loop should
/// chain another tick immediately.
#[derive(Debug, Clone, Copy)]
pub(super) struct TickOutcome {
    pub hit_cap: bool,
}

/// Runs the subscription health monitor loop.
///
/// Uses BOTH `housekeeping_semaphore` (intra-process mutual exclusion with
/// other housekeeping tasks) AND `pg_try_advisory_xact_lock` (inter-process
/// mutual exclusion across replicas). The semaphore prevents overloading
/// the 3-connection housekeeping pool; the advisory lock prevents duplicate
/// health events from concurrent API instances.
pub async fn run_subscription_health_monitor(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    config: &SubscriptionHealthMonitorConfig,
) {
    info!(
        "Subscription health monitor started (interval: {:?}, warning: {}%, disable: {}%)",
        config.interval, config.failure_percent_for_warning, config.failure_percent_for_disable
    );

    let mut last_cleanup: Option<Instant> = None;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        // Chain up to MAX_CHAINED_TICKS ticks in a row when the pipeline is
        // catching up on a backlog. A tick that hits the scan cap signals
        // "there's still more to process" — we loop without sleeping so the
        // cursor can advance. We stop chaining as soon as a tick completes
        // below the cap (normal steady state) or errors out.
        for _ in 0..MAX_CHAINED_TICKS {
            match run_health_check(db, config).await {
                Ok(outcome) if outcome.hit_cap => continue,
                Ok(_) => break,
                Err(e) => {
                    error!("Subscription health monitor error: {e}");
                    break;
                }
            }
        }

        if last_cleanup.is_none_or(|t| t.elapsed() > CLEANUP_INTERVAL) {
            log_cleanup_result(
                "resolved health events",
                queries::cleanup_resolved_health_events(db, config).await,
            );
            log_cleanup_result(
                "old health buckets",
                queries::cleanup_old_buckets(db, config).await,
            );
            last_cleanup = Some(Instant::now());
        }

        // Release the semaphore permit BEFORE sleeping so the other
        // housekeeping tasks can run during the interval. Without this
        // explicit drop, the permit is held through `sleep(config.interval)`
        // and `Semaphore::new(1)` means every other housekeeping task would
        // sit idle for up to `interval` between ticks.
        drop(permit);
        actix_web::rt::time::sleep(config.interval).await;
    }

    warn!("Subscription health monitor stopped (semaphore closed)");
}

fn log_cleanup_result(name: &str, result: Result<u64, sqlx::Error>) {
    match result {
        Ok(n) if n > 0 => info!("Subscription health monitor: cleaned up {n} {name}"),
        Ok(_) => debug!("Subscription health monitor: cleanup tick — nothing to remove for {name}"),
        Err(e) => warn!("Subscription health monitor: cleanup error for {name}: {e}"),
    }
}

/// One evaluation tick: grabs the advisory lock, runs the evaluation
/// pipeline, feeds each candidate through the state machine, and applies
/// the resulting side-effects — all inside one transaction.
async fn run_health_check(
    db: &PgPool,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<TickOutcome, SubscriptionHealthMonitorError> {
    let mut transaction = db.begin().await?;

    // Session-level safety net: no single tick may hold the advisory lock +
    // housekeeping permit longer than TICK_STATEMENT_TIMEOUT. `set local`
    // scopes the setting to the current transaction only.
    sqlx::query(&format!(
        "set local statement_timeout = '{TICK_STATEMENT_TIMEOUT}'"
    ))
    .execute(&mut *transaction)
    .await?;

    let acquired: bool = sqlx::query_scalar("select pg_try_advisory_xact_lock($1)")
        .bind(ADVISORY_LOCK_ID)
        .fetch_one(&mut *transaction)
        .await?;

    if !acquired {
        info!("Subscription health monitor: another instance holds the lock, skipping");
        return Ok(TickOutcome { hit_cap: false });
    }

    let (subscriptions, hit_cap) =
        evaluation::run_subscription_health_monitor_tick(&mut transaction, config).await?;
    info!(
        "Subscription health monitor: evaluated {} subscriptions{}",
        subscriptions.len(),
        if hit_cap {
            " (scan cap hit — chaining)"
        } else {
            ""
        }
    );

    let now = Utc::now();
    for subscription in &subscriptions {
        let planned = state_machine::plan_health_actions(subscription, config, now);
        if let Err(e) = apply_planned_actions(&mut transaction, subscription, &planned).await {
            warn!(
                "Subscription health monitor: error processing subscription {}: {e}",
                subscription.subscription_id
            );
        }
    }

    transaction.commit().await?;
    Ok(TickOutcome { hit_cap })
}

/// Applies the list of `PlannedAction`s decided by the pure state machine to
/// the database (within the caller's transaction). Every branch maps a
/// single action to one or more `queries::*` calls — no decision logic lives
/// here.
async fn apply_planned_actions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription: &SubscriptionHealth,
    planned: &[PlannedAction],
) -> Result<(), SubscriptionHealthMonitorError> {
    for action in planned {
        match action {
            PlannedAction::UpdateFailurePercent => {
                queries::update_subscription_failure_percent(
                    tx,
                    subscription.subscription_id,
                    subscription.failure_percent,
                )
                .await?;
            }
            PlannedAction::EmitWarning => {
                queries::insert_health_event(
                    tx,
                    subscription.subscription_id,
                    HealthStatus::Warning,
                    HealthEventCause::Auto,
                    None,
                )
                .await?;
            }
            PlannedAction::EmitResolved => {
                queries::insert_health_event(
                    tx,
                    subscription.subscription_id,
                    HealthStatus::Resolved,
                    HealthEventCause::Auto,
                    None,
                )
                .await?;
            }
            PlannedAction::EmitDisabled => {
                // The CTE inside disable_subscription is idempotent: if the
                // subscription was already disabled (e.g. by the user between
                // ticks) it returns None and we skip — no duplicate event row.
                let _ = queries::disable_subscription(tx, subscription.subscription_id).await?;
            }
        }
    }
    Ok(())
}
