//! Background loop that evaluates subscription health on a recurring tick.
//!
//! **What it does**: detects unhealthy webhook subscriptions (high failure rate)
//! and automatically warns or disables them.
//!
//! **How it works**:
//!   1. Acquire an advisory lock so only one API instance runs the check.
//!   2. In a transaction: evaluate every subscription's failure rate, insert
//!      health events, disable broken subscriptions.
//!   3. Periodically run a cleanup cycle to remove stale health data (once/day).

mod cleanup;
pub mod errors;
mod evaluation;
mod queries;
mod state_machine;
pub mod types;

use std::time::{Duration, Instant};

use sqlx::PgPool;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use evaluation::SubscriptionHealth;
use state_machine::PlannedAction;
use types::{HealthEventCause, HealthStatus};

/// Arbitrary unique ID for pg_try_advisory_xact_lock — must not conflict with other advisory locks in the application.
const ADVISORY_LOCK_ID: i64 = 42_000_001;

/// Tuning knobs for the health monitor — thresholds, bucketing, retention.
#[derive(Clone)]
pub struct SubscriptionHealthConfig {
    pub interval: Duration,
    pub warning_failure_percent: u8,
    pub disable_failure_percent: u8,
    pub time_window: Duration,
    pub min_sample_size: u32,
    pub warning_cooldown: Duration,
    pub retention_period_days: u32,
    pub bucket_duration: Duration,
    pub bucket_max_messages: u32,
    pub bucket_retention_days: u32,
    pub max_delta_rows_per_tick: u32,
}

/// Runs the health monitor loop.
///
/// Uses BOTH housekeeping_semaphore (intra-process mutual exclusion with
/// other housekeeping tasks) AND pg_try_advisory_xact_lock (inter-process
/// mutual exclusion across replicas). The semaphore prevents overloading the
/// 3-connection housekeeping pool; the advisory lock prevents duplicate
/// health events from concurrent API instances.
pub async fn run_subscription_health_monitor(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    config: &SubscriptionHealthConfig,
) {
    info!(
        "Health monitor started (interval: {:?}, warning: {}%, disable: {}%)",
        config.interval, config.warning_failure_percent, config.disable_failure_percent
    );

    // Cleanup runs once per day (not every tick) to keep the tables lean without adding per-tick overhead
    const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

    let mut last_cleanup: Option<Instant> = None;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = run_health_check(db, config).await {
            error!("Health monitor error: {e}");
        }

        if last_cleanup.is_none_or(|t| t.elapsed() > CLEANUP_INTERVAL) {
            match cleanup::cleanup_resolved_health_events(db, config).await {
                Ok(n) if n > 0 => {
                    info!("Health monitor: cleaned up {n} resolved health events");
                }
                Ok(_) => debug!("Health monitor: cleanup tick, no events to remove"),
                Err(e) => warn!("Health monitor: cleanup error: {e}"),
            }
            match cleanup::cleanup_old_buckets(db, config).await {
                Ok(n) if n > 0 => info!("Health monitor: cleaned up {n} old health buckets"),
                Ok(_) => debug!("Health monitor: bucket cleanup tick, none to remove"),
                Err(e) => warn!("Health monitor: bucket cleanup error: {e}"),
            }
            last_cleanup = Some(Instant::now());
        }

        drop(permit);
        // Note: total cycle time = check duration + sleep, not exactly config.interval
        actix_web::rt::time::sleep(config.interval).await;
    }

    warn!("Health monitor stopped (semaphore closed)");
}

/// Acquires the advisory lock and evaluates all subscriptions inside a single transaction.
async fn run_health_check(
    db: &PgPool,
    config: &SubscriptionHealthConfig,
) -> Result<(), errors::SubscriptionHealthError> {
    let mut transaction = db.begin().await?;

    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(ADVISORY_LOCK_ID)
        .fetch_one(&mut *transaction)
        .await?;

    if !acquired {
        info!("Health monitor: another instance holds the lock, skipping");
        return Ok(());
    }

    let (subscriptions, max_completed_at) =
        evaluation::fetch_subscription_health_stats(&mut transaction, config).await?;
    info!(
        "Health monitor: evaluated {} subscriptions",
        subscriptions.len()
    );

    let now = chrono::Utc::now();
    for subscription in &subscriptions {
        let planned = state_machine::plan_for_subscription(config, subscription, now);
        if let Err(e) = apply_planned_actions(&mut transaction, subscription, &planned).await {
            warn!(
                "Health monitor: error processing subscription {}: {e}",
                subscription.subscription_id
            );
        }
    }

    // Reset failure_percent for non-suspect subscriptions. Without this, a subscription
    // that spiked briefly and recovered would keep its stale rate cached forever —
    // consumers of webhook.subscription.failure_percent would read outdated data.
    let suspect_ids: Vec<uuid::Uuid> = subscriptions.iter().map(|s| s.subscription_id).collect();
    evaluation::reset_healthy_failure_percent(&mut transaction, &suspect_ids).await?;

    // Advance the cursor so the next tick only looks at newer deliveries
    if let Some(ts) = max_completed_at {
        evaluation::advance_cursor(&mut transaction, ts).await?;
    }

    transaction.commit().await?;
    Ok(())
}

/// Applies the list of `PlannedAction`s decided by the pure state machine to
/// the database (within the caller's transaction).
///
/// This is where the only side effects live: every branch maps a single
/// `PlannedAction` to one or more `queries::*` calls. No decision logic here.
async fn apply_planned_actions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription: &SubscriptionHealth,
    planned: &[PlannedAction],
) -> Result<(), errors::SubscriptionHealthError> {
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
                // ticks), it returns None and we skip — no duplicate event row.
                let _ = queries::disable_subscription(tx, subscription.subscription_id).await?;
            }
        }
    }
    Ok(())
}
