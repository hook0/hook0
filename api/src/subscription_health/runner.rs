//! Background loop + per-tick orchestrator for the subscription health
//! monitor.
//!
//! The loop:
//!   - Evaluates every subscription's recent delivery outcomes inside a
//!     single advisory-locked transaction so replicas can't stomp on each
//!     other.
//!   - Every 24 hours, runs a cleanup pass to keep the health tables lean.
//!   - Sleeps `config.interval` between ticks — total cycle time is
//!     `check duration + sleep`, not exactly `interval`.

use std::time::{Duration, Instant};

use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use super::errors::SubscriptionHealthError;
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

/// Tuning knobs for the subscription health monitor — thresholds, bucketing,
/// retention.
#[derive(Clone)]
pub struct SubscriptionHealthConfig {
    pub interval: Duration,
    pub warning_failure_percent: u8,
    pub disable_failure_percent: u8,
    pub failure_rate_evaluation_window: Duration,
    pub min_deliveries_for_evaluation: u32,
    pub anti_flap_window: Duration,
    pub resolved_event_retention: Duration,
    pub bucket_duration: Duration,
    pub bucket_max_messages: u32,
    pub bucket_retention_days: u32,
    pub max_request_attempts_scanned_per_tick: u32,
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
    config: &SubscriptionHealthConfig,
) {
    info!(
        "Subscription health monitor started (interval: {:?}, warning: {}%, disable: {}%)",
        config.interval, config.warning_failure_percent, config.disable_failure_percent
    );

    let mut last_cleanup: Option<Instant> = None;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = run_health_check(db, config).await {
            error!("Subscription health monitor error: {e}");
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
    config: &SubscriptionHealthConfig,
) -> Result<(), SubscriptionHealthError> {
    let mut transaction = db.begin().await?;

    let acquired: bool = sqlx::query_scalar("select pg_try_advisory_xact_lock($1)")
        .bind(ADVISORY_LOCK_ID)
        .fetch_one(&mut *transaction)
        .await?;

    if !acquired {
        info!("Subscription health monitor: another instance holds the lock, skipping");
        return Ok(());
    }

    let subscriptions = evaluation::run_evaluation_tick(&mut transaction, config).await?;
    info!(
        "Subscription health monitor: evaluated {} subscriptions",
        subscriptions.len()
    );

    let now = Utc::now();
    for subscription in &subscriptions {
        let planned = state_machine::evaluate_health_transition(subscription, config, now);
        if let Err(e) = apply_planned_actions(&mut transaction, subscription, &planned).await {
            warn!(
                "Subscription health monitor: error processing subscription {}: {e}",
                subscription.subscription_id
            );
        }
    }

    transaction.commit().await?;
    Ok(())
}

/// Applies the list of `PlannedAction`s decided by the pure state machine to
/// the database (within the caller's transaction). Every branch maps a
/// single action to one or more `queries::*` calls — no decision logic lives
/// here.
async fn apply_planned_actions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription: &SubscriptionHealth,
    planned: &[PlannedAction],
) -> Result<(), SubscriptionHealthError> {
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
