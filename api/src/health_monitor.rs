use std::str::FromStr;
use std::time::{Duration, Instant};

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
    info!(
        "Health monitor started (interval: {:?}, warning: {}%, disable: {}%)",
        config.interval, config.warning_failure_percent, config.disable_failure_percent
    );

    // Force cleanup on first iteration by backdating the timestamp.
    let mut last_cleanup = Instant::now() - Duration::from_secs(86400);

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = run_health_check(db, mailer, hook0_client, config).await {
            error!("Health monitor error: {e}");
        }

        if last_cleanup.elapsed() > Duration::from_secs(86400) {
            match cleanup_old_health_events(db).await {
                Ok(n) => {
                    if n > 0 {
                        info!("Health monitor: cleaned up {n} old health events");
                    }
                }
                Err(e) => warn!("Health monitor: cleanup error: {e}"),
            }
            last_cleanup = Instant::now();
        }

        drop(permit);
        actix_web::rt::time::sleep(config.interval).await;
    }

    warn!("Health monitor stopped (semaphore closed)");
}

/// Removes resolved health events older than 90 days,
/// keeping the latest event per subscription regardless of age.
///
/// Example: a subscription with events at -100d (resolved), -80d (resolved), -10d (warning)
/// deletes only the -100d row; the -80d row is kept because -10d is newer.
async fn cleanup_old_health_events(db: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM webhook.subscription_health_event d
        WHERE d.created_at < now() - interval '90 days'
          AND d.status = 'resolved'
          AND EXISTS (
            SELECT 1 FROM webhook.subscription_health_event newer
            WHERE newer.subscription__id = d.subscription__id
              AND newer.created_at > d.created_at
          )
        "#,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

#[derive(Debug, sqlx::FromRow)]
#[allow(non_snake_case, dead_code)]
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
    last_health_source: Option<String>,
    last_health_user_id: Option<Uuid>,
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
                              and ra2.created_at > now() - make_interval(secs => $1)
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
            lh.source as last_health_source,
            lh.user__id as last_health_user_id,
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
            select she.status, she.created_at, she.source, she.user__id
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

/// Describes a side-effect (email / Hook0 event) to perform after the
/// transaction has been committed.
enum HealthAction {
    Warning(HealthActionInfo),
    Disabled(HealthActionInfo),
    Recovered(HealthActionInfo),
}

/// Data needed to send emails and Hook0 events outside the transaction.
struct HealthActionInfo {
    subscription_id: Uuid,
    organization_id: Uuid,
    application_id: Uuid,
    application_name: Option<String>,
    description: Option<String>,
    target_url: String,
    failure_percent: f64,
    disabled_at: Option<DateTime<Utc>>,
    retry_schedule: Option<RetrySchedulePayload>,
}

impl HealthActionInfo {
    fn from_sub(sub: &SubscriptionHealth, disabled_at: Option<DateTime<Utc>>) -> Self {
        Self {
            subscription_id: sub.subscription__id,
            organization_id: sub.organization__id,
            application_id: sub.application__id,
            application_name: sub.application_name.clone(),
            description: sub.description.clone(),
            target_url: sub.target_url.clone(),
            failure_percent: sub.failure_percent,
            disabled_at,
            retry_schedule: sub.retry_schedule__id.map(|id| RetrySchedulePayload {
                retry_schedule_id: id,
                name: sub.retry_schedule_name.clone().unwrap_or_default(),
                strategy: sub.retry_strategy.clone().unwrap_or_default(),
                max_retries: sub.retry_max_retries.unwrap_or(0),
                custom_intervals: sub.retry_custom_intervals.clone(),
                linear_delay: sub.retry_linear_delay,
            }),
        }
    }
}

async fn run_health_check(
    db: &PgPool,
    mailer: &Mailer,
    hook0_client: &Option<Hook0Client>,
    config: &HealthMonitorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Phase 1: transaction — evaluate health, insert events, disable subs.
    let actions = {
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

        let mut actions = Vec::new();
        for sub in &subscriptions {
            match process_subscription(&mut tx, config, sub).await {
                Ok(mut sub_actions) => actions.append(&mut sub_actions),
                Err(e) => {
                    warn!(
                        "Health monitor: error processing subscription {}: {e}",
                        sub.subscription__id
                    );
                }
            }
        }

        tx.commit().await?;
        actions
    };

    // Phase 2: best-effort side-effects (no transaction held).
    for action in &actions {
        let (info, kind) = match action {
            HealthAction::Warning(info) => (info, EmailKind::Warning),
            HealthAction::Disabled(info) => (info, EmailKind::Disabled),
            HealthAction::Recovered(info) => (info, EmailKind::Recovered),
        };

        send_email_best_effort(mailer, db, info, kind, config).await;

        if let HealthAction::Disabled(info) = action {
            if let Some(client) = hook0_client {
                let disabled_at = info.disabled_at.unwrap_or_else(Utc::now);
                let event = EventSubscriptionDisabled {
                    subscription: SubscriptionDisabledPayload {
                        subscription_id: info.subscription_id,
                        organization_id: info.organization_id,
                        application_id: info.application_id,
                        description: info.description.clone(),
                        target: info.target_url.clone(),
                        disabled_at,
                    },
                    retry_schedule: info.retry_schedule.clone(),
                };
                let hook0_event: Hook0ClientEvent = event.into();
                if let Err(e) = client.send_event(&hook0_event.mk_hook0_event()).await {
                    warn!("Health monitor: failed to send subscription.disabled Hook0 event: {e}");
                }
            }
        }
    }

    Ok(())
}

enum EmailKind {
    Warning,
    Disabled,
    Recovered,
}

async fn process_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
    sub: &SubscriptionHealth,
) -> Result<Vec<HealthAction>, Box<dyn std::error::Error>> {
    let ratio = sub.failure_percent;
    let warning_pct = config.warning_failure_percent as f64;
    let disable_pct = config.disable_failure_percent as f64;
    let last_status = sub.last_health_status.as_deref();
    let last_at = sub.last_health_at;

    let mut actions = Vec::new();

    // Persist current failure_percent to subscription table for frontend display
    sqlx::query(
        "UPDATE webhook.subscription SET failure_percent = $1 WHERE subscription__id = $2",
    )
    .bind(ratio)
    .bind(sub.subscription__id)
    .execute(&mut **tx)
    .await?;

    match last_status {
        Some("disabled") => {}

        Some("resolved")
            if last_at.map_or(false, |at| {
                (Utc::now() - at)
                    < chrono::Duration::from_std(config.warning_cooldown).unwrap_or_default()
            }) => {}

        Some("warning") if ratio >= warning_pct && ratio < disable_pct => {}

        Some("warning") if ratio < warning_pct => {
            // Skip recovery email if the last event was a manual user action (re-enable via API) —
            // the user already knows about it. Only send recovery email for system-originated events.
            insert_health_event(tx, sub.subscription__id, "resolved", "system", None).await?;
            if sub.last_health_source.as_deref() != Some("user") {
                actions.push(HealthAction::Recovered(HealthActionInfo::from_sub(sub, None)));
            }
        }

        Some("warning") => {
            let disabled_at = disable_subscription(tx, sub).await?;
            if let Some(at) = disabled_at {
                actions.push(HealthAction::Disabled(HealthActionInfo::from_sub(sub, Some(at))));
            }
        }

        _ if ratio >= disable_pct => {
            insert_health_event(tx, sub.subscription__id, "warning", "system", None).await?;
            actions.push(HealthAction::Warning(HealthActionInfo::from_sub(sub, None)));
            let disabled_at = disable_subscription(tx, sub).await?;
            if let Some(at) = disabled_at {
                actions.push(HealthAction::Disabled(HealthActionInfo::from_sub(sub, Some(at))));
            }
        }

        _ if ratio >= warning_pct => {
            insert_health_event(tx, sub.subscription__id, "warning", "system", None).await?;
            actions.push(HealthAction::Warning(HealthActionInfo::from_sub(sub, None)));
        }

        _ => {}
    }

    Ok(actions)
}

/// source: "system" = automatic (health monitor), "user" = manual (API PUT).
/// When source is "user" and user_id is None, the action was via a service token.
async fn insert_health_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    status: &str,
    source: &str,
    user_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO webhook.subscription_health_event (subscription__id, status, source, user__id) VALUES ($1, $2, $3, $4)",
    )
    .bind(subscription_id)
    .bind(status)
    .bind(source)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Disables a subscription and inserts a 'disabled' health event atomically.
/// Returns `Some(disabled_at)` if it actually disabled, `None` if already disabled.
async fn disable_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sub: &SubscriptionHealth,
) -> Result<Option<DateTime<Utc>>, Box<dyn std::error::Error>> {
    let disabled_at: Option<DateTime<Utc>> = sqlx::query_scalar(
        r#"
        WITH updated AS (
            UPDATE webhook.subscription
            SET is_enabled = false
            WHERE subscription__id = $1 AND is_enabled = true
            RETURNING subscription__id
        ),
        inserted AS (
            INSERT INTO webhook.subscription_health_event (subscription__id, status, source, user__id)
            SELECT subscription__id, 'disabled', 'system', NULL FROM updated
            RETURNING created_at
        )
        SELECT created_at FROM inserted
        "#,
    )
    .bind(sub.subscription__id)
    .fetch_optional(&mut **tx)
    .await?;

    if disabled_at.is_some() {
        info!(
            "Health monitor: disabled subscription {}",
            sub.subscription__id
        );
    }

    Ok(disabled_at)
}

async fn send_email_best_effort(
    mailer: &Mailer,
    db: &PgPool,
    info: &HealthActionInfo,
    kind: EmailKind,
    config: &HealthMonitorConfig,
) {
    let description = info
        .description
        .clone()
        .unwrap_or_else(|| info.subscription_id.to_string());
    let app_name = info
        .application_name
        .clone()
        .unwrap_or_else(|| info.application_id.to_string());
    let evaluation_window = humantime::format_duration(config.time_window).to_string();

    let mail = match kind {
        EmailKind::Warning => Mail::SubscriptionWarning {
            application_name: app_name,
            subscription_description: description,
            subscription_id: info.subscription_id,
            target_url: info.target_url.clone(),
            failure_percent: info.failure_percent,
            evaluation_window,
        },
        EmailKind::Disabled => Mail::SubscriptionDisabled {
            application_name: app_name,
            subscription_description: description,
            subscription_id: info.subscription_id,
            target_url: info.target_url.clone(),
            failure_percent: info.failure_percent,
            evaluation_window,
            disabled_at: info
                .disabled_at
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
        },
        EmailKind::Recovered => Mail::SubscriptionRecovered {
            application_name: app_name,
            subscription_description: description,
            subscription_id: info.subscription_id,
            target_url: info.target_url.clone(),
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
    .bind(info.organization_id)
    .fetch_all(db)
    .await
    {
        Ok(users) => users,
        Err(e) => {
            warn!(
                "Health monitor: failed to query org users for email (org {}): {e}",
                info.organization_id
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
