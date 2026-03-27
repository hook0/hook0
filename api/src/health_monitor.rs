use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use hook0_client::Hook0Client;

use crate::hook0_client::{
    EventSubscriptionDisabled, Hook0ClientEvent, RetrySchedulePayload,
    SubscriptionDisabledPayload,
};
use crate::mailer::{Mail, Mailer};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(60);
const ADVISORY_LOCK_ID: i64 = 42_000_001;

#[derive(Clone)]
pub struct HealthMonitorConfig {
    pub interval: Duration,
    pub warning_failure_percent: u8,
    pub disable_failure_percent: u8,
    pub time_window: Duration,
    pub message_window: u32,
    pub min_sample_size: u32,
    pub warning_cooldown: Duration,
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
    actix_web::rt::time::sleep(STARTUP_GRACE_PERIOD).await;
    info!(
        "Health monitor started (interval: {:?}, warning: {}%, disable: {}%)",
        config.interval, config.warning_failure_percent, config.disable_failure_percent
    );

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = run_health_check(db, mailer, hook0_client, config).await {
            error!("Health monitor error: {e}");
        }
        drop(permit);
        actix_web::rt::time::sleep(config.interval).await;
    }

    warn!("Health monitor stopped (semaphore closed)");
}

async fn run_health_check(
    db: &PgPool,
    _mailer: &Mailer,
    _hook0_client: &Option<Hook0Client>,
    _config: &HealthMonitorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Transaction-level advisory lock: auto-released on commit/rollback.
    // Safe with connection pools (no lock leak on error).
    let mut tx = db.begin().await?;

    let acquired: bool =
        sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
            .bind(ADVISORY_LOCK_ID)
            .fetch_one(&mut *tx)
            .await?;

    if !acquired {
        info!("Health monitor: another instance holds the lock, skipping");
        return Ok(());
    }

    // TODO: evaluate_subscriptions (Task 6)
    // TODO: process_subscription state machine (Task 7)

    tx.commit().await?;
    Ok(())
}
