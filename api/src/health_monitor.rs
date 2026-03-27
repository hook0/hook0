use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use uuid::Uuid;

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

#[derive(Debug, sqlx::FromRow)]
#[allow(non_snake_case)]
struct SubscriptionHealth {
    subscription__id: Uuid,
    application__id: Uuid,
    organization__id: Uuid,
    application_name: Option<String>,
    description: Option<String>,
    target_url: String,
    failure_percent: f64,
    total: i64,
    last_health_status: Option<String>,
    last_health_at: Option<DateTime<Utc>>,
    retry_schedule__id: Option<Uuid>,
    retry_schedule_name: Option<String>,
    retry_strategy: Option<String>,
    retry_max_retries: Option<i32>,
    retry_custom_intervals: Option<Vec<i32>>,
    retry_linear_delay: Option<i32>,
}

async fn evaluate_subscriptions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as f64;
    let min_sample = config.min_sample_size as i64;
    let msg_window = config.message_window as i64;

    sqlx::query_as::<_, SubscriptionHealth>(
        r#"
        with attempt_stats as (
            select
                ra.subscription__id,
                count(*) as total,
                count(*) filter (where ra.failed_at is not null) as failed
            from webhook.request_attempt ra
            inner join webhook.subscription s on s.subscription__id = ra.subscription__id
            where s.is_enabled = true
              and s.deleted_at is null
              and ra.created_at > now() - make_interval(secs => $1)
              and (ra.succeeded_at is not null or ra.failed_at is not null)
            group by ra.subscription__id
            having count(*) >= $2
        ),
        windowed_stats as (
            select
                a.subscription__id,
                case
                    when a.total >= $3 then (
                        select count(*) filter (where sub.failed_at is not null)::float8
                             / $3::float8 * 100.0
                        from (
                            select ra2.failed_at
                            from webhook.request_attempt ra2
                            where ra2.subscription__id = a.subscription__id
                              and (ra2.succeeded_at is not null or ra2.failed_at is not null)
                            order by ra2.created_at desc
                            limit $3
                        ) sub
                    )
                    else (a.failed::float8 / a.total::float8 * 100.0)
                end as failure_percent,
                a.total
            from attempt_stats a
        )
        select
            w.subscription__id,
            s.application__id,
            app.organization__id,
            app.name as application_name,
            s.description,
            coalesce(th.url, '') as target_url,
            w.failure_percent,
            w.total,
            lh.status as last_health_status,
            lh.created_at as last_health_at,
            s.retry_schedule__id,
            rs.name as retry_schedule_name,
            rs.strategy as retry_strategy,
            rs.max_retries as retry_max_retries,
            rs.custom_intervals as retry_custom_intervals,
            rs.linear_delay as retry_linear_delay
        from windowed_stats w
        inner join webhook.subscription s using (subscription__id)
        inner join event.application app on app.application__id = s.application__id
        left join lateral (
            select she.status, she.created_at
            from webhook.subscription_health_event she
            where she.subscription__id = w.subscription__id
            order by she.created_at desc
            limit 1
        ) lh on true
        left join webhook.retry_schedule rs on rs.retry_schedule__id = s.retry_schedule__id
        left join webhook.target_http th on th.target__id = s.target__id
        "#,
    )
    .bind(time_window_secs)
    .bind(min_sample)
    .bind(msg_window)
    .fetch_all(&mut **tx)
    .await
}

async fn run_health_check(
    db: &PgPool,
    _mailer: &Mailer,
    _hook0_client: &Option<Hook0Client>,
    config: &HealthMonitorConfig,
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

    let subscriptions = evaluate_subscriptions(&mut tx, config).await?;
    info!(
        "Health monitor: evaluated {} subscriptions",
        subscriptions.len()
    );

    // TODO: process_subscription state machine (Task 7)
    for sub in &subscriptions {
        info!(
            "Health monitor: subscription {} failure_percent={:.1}% (last_status={:?})",
            sub.subscription__id, sub.failure_percent, sub.last_health_status
        );
    }

    tx.commit().await?;
    Ok(())
}
