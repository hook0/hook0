//! Background subsystem that auto-warns and auto-disables unhealthy webhook
//! subscriptions based on their rolling failure rate.
//!
//! Module graph:
//!
//!   ┌──────────┐     ┌──────────────┐     ┌───────────┐
//!   │  runner  │────▶│  evaluation  │────▶│  queries  │
//!   └────┬─────┘     └──────────────┘     └───────────┘
//!        │                                       ▲
//!        │           ┌───────────────┐           │
//!        └──────────▶│ state_machine │           │
//!                    └───────┬───────┘           │
//!                            │                   │
//!                            ▼                   │
//!                    ┌───────────────┐           │
//!                    │     types     │◀──────────┘
//!                    └───────────────┘
//!
//! - [`run_subscription_health_monitor`]: the public entry point — a
//!   background loop that wakes up on the housekeeping semaphore, chains
//!   evaluation ticks while the pipeline is catching up on a backlog, runs
//!   the daily cleanup pass, then sleeps. Lives here (not in `runner`)
//!   because it's the one thing `main.rs` imports from this subsystem.
//! - [`runner`]: per-tick orchestrator + `PlannedAction` dispatch to the DB
//!   layer. Holds the transaction scope for one evaluation tick.
//! - [`evaluation`]: pipeline façade (`run_subscription_health_monitor_tick`)
//!   that produces the list of subscriptions the state machine should judge.
//! - [`queries`]: SQL layer, split by domain (buckets, cursor, deltas,
//!   events, subscription_state).
//! - [`state_machine`]: pure decision function — takes a subscription's
//!   current state, returns a list of `PlannedAction`. No I/O.
//! - [`types`]: `HealthStatus` and `HealthEventCause` enums shared across
//!   the subsystem and the API layer.

mod evaluation;
mod queries;
mod runner;
mod state_machine;
pub mod types;

use std::time::Instant;

use sqlx::PgPool;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

pub use runner::SubscriptionHealthMonitorConfig;

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
        for _ in 0..runner::MAX_CHAINED_TICKS {
            match runner::run_health_check(db, config).await {
                Ok(outcome) if outcome.hit_cap => continue,
                Ok(_) => break,
                Err(e) => {
                    error!("Subscription health monitor error: {e}");
                    break;
                }
            }
        }

        if last_cleanup.is_none_or(|t| t.elapsed() > runner::CLEANUP_INTERVAL) {
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
