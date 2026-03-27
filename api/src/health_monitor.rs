use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use lettre::{Address, message::Mailbox};
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
    mailer: &Mailer,
    hook0_client: &Option<Hook0Client>,
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

    for sub in &subscriptions {
        if let Err(e) = process_subscription(&mut tx, mailer, hook0_client, config, sub).await {
            warn!(
                "Health monitor: error processing subscription {}: {e}",
                sub.subscription__id
            );
        }
    }

    tx.commit().await?;
    Ok(())
}

enum EmailKind {
    Warning,
    Disabled,
    Recovered,
}

async fn process_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    mailer: &Mailer,
    hook0_client: &Option<Hook0Client>,
    config: &HealthMonitorConfig,
    sub: &SubscriptionHealth,
) -> Result<(), Box<dyn std::error::Error>> {
    let ratio = sub.failure_percent;
    let warning_pct = config.warning_failure_percent as f64;
    let disable_pct = config.disable_failure_percent as f64;
    let last_status = sub.last_health_status.as_deref();
    let last_at = sub.last_health_at;

    match last_status {
        Some("disabled") => {}

        Some("resolved")
            if last_at.map_or(false, |at| {
                (Utc::now() - at)
                    < chrono::Duration::from_std(config.warning_cooldown).unwrap_or_default()
            }) => {}

        Some("warning") if ratio >= warning_pct && ratio < disable_pct => {}

        Some("warning") if ratio < warning_pct => {
            insert_health_event(tx, sub.subscription__id, "resolved").await?;
            send_email_best_effort(mailer, tx, sub, EmailKind::Recovered, config).await;
        }

        Some("warning") => {
            disable_subscription(tx, mailer, hook0_client, sub, config).await?;
        }

        _ if ratio >= disable_pct => {
            insert_health_event(tx, sub.subscription__id, "warning").await?;
            send_email_best_effort(mailer, tx, sub, EmailKind::Warning, config).await;
            disable_subscription(tx, mailer, hook0_client, sub, config).await?;
        }

        _ if ratio >= warning_pct => {
            insert_health_event(tx, sub.subscription__id, "warning").await?;
            send_email_best_effort(mailer, tx, sub, EmailKind::Warning, config).await;
        }

        _ => {}
    }

    Ok(())
}

async fn insert_health_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO webhook.subscription_health_event (subscription__id, status) VALUES ($1, $2)",
    )
    .bind(subscription_id)
    .bind(status)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn disable_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    mailer: &Mailer,
    hook0_client: &Option<Hook0Client>,
    sub: &SubscriptionHealth,
    config: &HealthMonitorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = sqlx::query(
        r#"
        WITH updated AS (
            UPDATE webhook.subscription
            SET is_enabled = false
            WHERE subscription__id = $1 AND is_enabled = true
            RETURNING subscription__id
        )
        INSERT INTO webhook.subscription_health_event (subscription__id, status)
        SELECT subscription__id, 'disabled' FROM updated
        "#,
    )
    .bind(sub.subscription__id)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(());
    }

    info!(
        "Health monitor: disabled subscription {}",
        sub.subscription__id
    );

    // Best-effort email
    send_email_best_effort(mailer, tx, sub, EmailKind::Disabled, config).await;

    // Best-effort Hook0 event
    if let Some(client) = hook0_client {
        let event = EventSubscriptionDisabled {
            subscription: SubscriptionDisabledPayload {
                subscription_id: sub.subscription__id,
                organization_id: sub.organization__id,
                application_id: sub.application__id,
                description: sub.description.clone(),
                target: sub.target_url.clone(),
                disabled_at: Utc::now(),
            },
            retry_schedule: sub.retry_schedule__id.map(|id| RetrySchedulePayload {
                retry_schedule_id: id,
                name: sub.retry_schedule_name.clone().unwrap_or_default(),
                strategy: sub.retry_strategy.clone().unwrap_or_default(),
                max_retries: sub.retry_max_retries.unwrap_or(0),
                custom_intervals: sub.retry_custom_intervals.clone(),
                linear_delay: sub.retry_linear_delay,
            }),
        };
        let hook0_event: Hook0ClientEvent = event.into();
        if let Err(e) = client.send_event(&hook0_event.mk_hook0_event()).await {
            warn!("Health monitor: failed to send subscription.disabled Hook0 event: {e}");
        }
    }

    Ok(())
}

async fn send_email_best_effort(
    mailer: &Mailer,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sub: &SubscriptionHealth,
    kind: EmailKind,
    config: &HealthMonitorConfig,
) {
    let description = sub
        .description
        .clone()
        .unwrap_or_else(|| sub.subscription__id.to_string());
    let app_name = sub
        .application_name
        .clone()
        .unwrap_or_else(|| sub.application__id.to_string());
    let evaluation_window = format!("{}h", config.time_window.as_secs() / 3600);

    let mail = match kind {
        EmailKind::Warning => Mail::SubscriptionWarning {
            application_name: app_name,
            subscription_description: description,
            subscription_id: sub.subscription__id,
            target_url: sub.target_url.clone(),
            failure_percent: sub.failure_percent,
            evaluation_window,
        },
        EmailKind::Disabled => Mail::SubscriptionDisabled {
            application_name: app_name,
            subscription_description: description,
            subscription_id: sub.subscription__id,
            target_url: sub.target_url.clone(),
            failure_percent: sub.failure_percent,
            evaluation_window,
            disabled_at: Utc::now().to_rfc3339(),
        },
        EmailKind::Recovered => Mail::SubscriptionRecovered {
            application_name: app_name,
            subscription_description: description,
            subscription_id: sub.subscription__id,
            target_url: sub.target_url.clone(),
        },
    };

    #[derive(sqlx::FromRow)]
    struct OrgUser {
        first_name: String,
        last_name: String,
        email: String,
    }

    let users = match sqlx::query_as::<_, OrgUser>(
        r#"
        SELECT u.first_name, u.last_name, u.email
        FROM iam.user u
        INNER JOIN iam.user__organization ou ON u.user__id = ou.user__id
        WHERE ou.organization__id = $1
        "#,
    )
    .bind(sub.organization__id)
    .fetch_all(&mut **tx)
    .await
    {
        Ok(users) => users,
        Err(e) => {
            warn!(
                "Health monitor: failed to query org users for email (org {}): {e}",
                sub.organization__id
            );
            return;
        }
    };

    for user in users {
        let address = match Address::from_str(&user.email) {
            Ok(a) => a,
            Err(e) => {
                warn!(
                    "Health monitor: invalid email address {}: {e}",
                    user.email
                );
                continue;
            }
        };

        let recipient = Mailbox::new(
            Some(format!("{} {}", user.first_name, user.last_name)),
            address,
        );

        if let Err(e) = mailer.send_mail(mail.clone(), recipient).await {
            warn!(
                "Health monitor: failed to send email to {}: {e}",
                user.email
            );
        }
    }
}
