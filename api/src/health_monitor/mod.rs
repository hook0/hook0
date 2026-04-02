mod evaluation;
mod notifications;
mod state_machine;

use std::time::{Duration, Instant};

use sqlx::PgPool;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use hook0_client::Hook0Client;

use crate::mailer::Mailer;

/// Arbitrary unique ID for pg_try_advisory_xact_lock — must not conflict with other advisory locks in the application.
const ADVISORY_LOCK_ID: i64 = 42_000_001;

#[derive(Clone)]
pub struct HealthMonitorConfig {
    pub interval: Duration,
    pub warning_failure_percent: u8,
    pub disable_failure_percent: u8,
    pub time_window: Duration,
    #[allow(dead_code)] // kept for CLI backward compatibility
    pub message_window: u32,
    pub min_sample_size: u32,
    pub warning_cooldown: Duration,
    pub retention_period_days: u32,
    pub bucket_duration: Duration,
    pub bucket_max_messages: u32,
    pub bucket_retention_days: u32,
}

/// Runs the health monitor loop.
///
/// Uses BOTH housekeeping_semaphore (intra-process mutual exclusion with
/// other housekeeping tasks) AND pg_try_advisory_xact_lock (inter-instance
/// mutual exclusion across Kubernetes replicas). The semaphore prevents
/// overloading the 3-connection housekeeping pool; the advisory lock
/// prevents duplicate emails/events from concurrent API instances.
pub async fn run_health_monitor(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    mailer: &Mailer,
    hook0_client: &Option<Hook0Client>,
    config: &HealthMonitorConfig,
) {
    info!(
        "Health monitor started (interval: {:?}, warning: {}%, disable: {}%)",
        config.interval, config.warning_failure_percent, config.disable_failure_percent
    );

    const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 1 day

    let mut last_cleanup: Option<Instant> = None;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = run_health_check(db, mailer, hook0_client, config).await {
            error!("Health monitor error: {e}");
        }

        if last_cleanup.is_none() || last_cleanup.unwrap().elapsed() > CLEANUP_INTERVAL {
            match evaluation::cleanup_resolved_health_events(db, config).await {
                Ok(n) => {
                    if n > 0 {
                        info!("Health monitor: cleaned up {n} resolved health events");
                    } else {
                        debug!("Health monitor: cleanup tick, no events to remove");
                    }
                }
                Err(e) => warn!("Health monitor: cleanup error: {e}"),
            }
            match evaluation::cleanup_old_buckets(db, config).await {
                Ok(n) if n > 0 => info!("Health monitor: cleaned up {n} old health buckets"),
                Ok(_) => debug!("Health monitor: bucket cleanup tick, none to remove"),
                Err(e) => warn!("Health monitor: bucket cleanup error: {e}"),
            }
            last_cleanup = Some(Instant::now());
        }

        drop(permit);
        // Sleep between ticks. Total cycle time = check duration + interval.
        actix_web::rt::time::sleep(config.interval).await;
    }

    warn!("Health monitor stopped (semaphore closed)");
}

/// Acquires the advisory lock, evaluates all subscriptions, then fires side-effects (emails, Hook0 events).
async fn run_health_check(
    db: &PgPool,
    mailer: &Mailer,
    hook0_client: &Option<Hook0Client>,
    config: &HealthMonitorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Phase 1: transaction — evaluate health, insert events, disable subscriptions.
    let actions = {
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

        let mut actions = Vec::new();
        for subscription in &subscriptions {
            match state_machine::process_subscription(&mut transaction, config, subscription).await
            {
                Ok(mut subscription_actions) => actions.append(&mut subscription_actions),
                Err(e) => {
                    warn!(
                        "Health monitor: error processing subscription {}: {e}",
                        subscription.subscription_id
                    );
                }
            }
        }

        // Reset failure_percent for non-suspect subscriptions
        let suspect_ids: Vec<uuid::Uuid> = subscriptions
            .iter()
            .map(|s| s.subscription_id)
            .collect();
        if let Err(e) =
            evaluation::reset_healthy_failure_percent(&mut transaction, &suspect_ids).await
        {
            warn!("Health monitor: failed to reset healthy failure_percent: {e}");
        }

        // Advance watermark if we processed any deltas
        if let Some(wm) = max_completed_at
            && let Err(e) = evaluation::advance_watermark(&mut transaction, wm).await
        {
            warn!("Health monitor: failed to advance watermark: {e}");
        }

        transaction.commit().await?;
        actions
    };

    // Phase 2: best-effort side-effects (no transaction held).
    notifications::dispatch_health_actions(&actions, mailer, db, hook0_client, config).await;

    Ok(())
}
