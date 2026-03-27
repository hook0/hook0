# Automatic Subscription Deactivation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a periodic health monitor to the API that evaluates subscription failure ratios, sends warning/deactivation/recovery emails, disables unhealthy subscriptions, and emits Hook0 events.

**Architecture:** Background task in the API process (`actix_web::rt::spawn` + `tokio::time::sleep`), using `housekeeping_pool` + `housekeeping_semaphore` + `pg_try_advisory_xact_lock` (transaction-level, safe with connection pools). Single SQL query computes adaptive failure ratios. Append-only `subscription_health_event` table tracks state transitions. Three MJML email templates. One new Hook0 event type.

**Tech Stack:** Rust, actix-web, sqlx (Postgres), clap, lettre (SMTP), mrml (MJML), humantime, Biscuit auth

**Spec:** `docs/superpowers/specs/2026-03-26-auto-deactivation-design.md`

**Prerequisite:** Phase 1 (Configurable Retry Schedule) **must be merged** before starting. Verify `webhook.retry_schedule` table exists: `psql $DATABASE_URL -c "\d webhook.retry_schedule"`. If it does not exist, stop and merge Phase 1 first.

---

## Conventions

**Build check:** After every task, run `cargo check -p hook0-api` to verify compilation.

**Commit:** After every task, commit with the message provided. Use `git add <specific files>`.

**SQL style:** Lowercase keywords (`create table`, `alter table`, `select`), matching existing migrations.

**Anchors:** File locations use pattern anchors (e.g., "after the `QuotaEventsPerDayReached` variant"), not line numbers.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `api/migrations/20260326120000_add_subscription_health.up.sql` | Create | Migration: health event table + indexes |
| `api/migrations/20260326120000_add_subscription_health.down.sql` | Create | Rollback |
| `api/src/health_monitor.rs` | Create | Background task: evaluation, state machine, emails, Hook0 events |
| `api/src/handlers/subscriptions.rs` | Modify | Add `auto_disabled_at` to response, CTE re-activation |
| `api/src/hook0_client.rs` | Modify | Add `api.subscription.disabled` event type + nested structs |
| `api/src/mailer.rs` | Modify | Add 3 `Mail` variants |
| `api/src/mail_templates/subscriptions/warning.mjml` | Create | Warning email template |
| `api/src/mail_templates/subscriptions/disabled.mjml` | Create | Deactivation email template |
| `api/src/mail_templates/subscriptions/recovered.mjml` | Create | Recovery email template |
| `api/src/main.rs` | Modify | Config args + spawn health monitor + register module |

---

## Task 1: Database Migration

- [ ] **Step 1: Write up migration**

```sql
-- api/migrations/20260326120000_add_subscription_health.up.sql

create table webhook.subscription_health_event (
    health_event__id uuid not null default public.gen_random_uuid(),
    subscription__id uuid not null
        references webhook.subscription(subscription__id)
        on delete cascade,
    status text not null
        check (status in ('warning', 'disabled', 'resolved')),
    created_at timestamptz not null default statement_timestamp(),
    constraint subscription_health_event_pkey primary key (health_event__id)
);

create index if not exists idx_subscription_health_event_sub_id
    on webhook.subscription_health_event(subscription__id, created_at desc)
    include (status);

-- Composite index for health evaluation query (replaces the single-column subscription__id index)
create index if not exists idx_request_attempt_sub_health
    on webhook.request_attempt (subscription__id, created_at desc)
    include (succeeded_at, failed_at);

-- Drop redundant single-column index (the new composite index is a strict superset)
drop index if exists webhook.request_attempt_subscription__id_idx;
```

- [ ] **Step 2: Write down migration**

```sql
-- api/migrations/20260326120000_add_subscription_health.down.sql

-- Restore the original single-column index
create index if not exists request_attempt_subscription__id_idx
    on webhook.request_attempt (subscription__id);

drop index if exists webhook.idx_request_attempt_sub_health;
drop index if exists webhook.idx_subscription_health_event_sub_id;
drop table if exists webhook.subscription_health_event;
```

- [ ] **Step 3: Run migration and verify**

Prerequisite: migration DB must be up. `cd api && sqlx migrate run`, then verify with `\d webhook.subscription_health_event` and `\di webhook.idx_request_attempt_sub_health`.

Commit: `feat(db): add subscription_health_event table and request_attempt health index`

---

## Task 2: Hook0 Event — `api.subscription.disabled`

- [ ] **Step 1: Add event type string to `EVENT_TYPES`**

In `api/src/hook0_client.rs`, add `"api.subscription.disabled",` after `"api.retry_schedule.removed",` in the `EVENT_TYPES` array.

- [ ] **Step 2: Define nested payload structs**

After the last event struct (find `EventRetryScheduleRemoved`), add:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionDisabledPayload {
    pub subscription_id: Uuid,
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub description: Option<String>,
    pub target: String,
    pub disabled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RetrySchedulePayload {
    pub retry_schedule_id: Uuid,
    pub name: String,
    pub strategy: String,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventSubscriptionDisabled {
    pub subscription: SubscriptionDisabledPayload,
    pub retry_schedule: Option<RetrySchedulePayload>,
}

impl Event for EventSubscriptionDisabled {
    fn event_type(&self) -> &'static str {
        "api.subscription.disabled"
    }

    fn labels(&self) -> Vec<(String, String)> {
        vec![
            ("organization_id".to_owned(), self.subscription.organization_id.to_string()),
            ("application_id".to_owned(), self.subscription.application_id.to_string()),
            ("subscription_id".to_owned(), self.subscription.subscription_id.to_string()),
        ]
    }
}

impl From<EventSubscriptionDisabled> for Hook0ClientEvent {
    fn from(e: EventSubscriptionDisabled) -> Self {
        Self::SubscriptionDisabled(e)
    }
}
```

- [ ] **Step 3: Add variant to `Hook0ClientEvent` enum**

After the `SubscriptionRemoved(EventSubscriptionRemoved)` variant, add `SubscriptionDisabled(EventSubscriptionDisabled),`.

- [ ] **Step 4: Add match arm in `mk_hook0_event`**

After the `Self::SubscriptionRemoved(e) => to_event(e, None),` arm, add `Self::SubscriptionDisabled(e) => to_event(e, None),`.

Commit: `feat(api): add api.subscription.disabled Hook0 event type with nested payload`

---

## Task 3: Mailer — Three Email Templates

- [ ] **Step 1: Create MJML templates**

Create `api/src/mail_templates/subscriptions/` directory. Create three MJML templates following the structure of `quotas/events_per_day_warning.mjml`:

- `warning.mjml` — variables: `{ $application_name }`, `{ $subscription_description }`, `{ $subscription_id }`, `{ $target_url }`, `{ $failure_percent }`, `{ $evaluation_window }`
- `disabled.mjml` — same variables + `{ $disabled_at }`
- `recovered.mjml` — variables: `{ $application_name }`, `{ $subscription_description }`, `{ $subscription_id }`, `{ $target_url }`

Variable convention: MJML uses `{ $key }` (with spaces and `$` prefix). The `variables()` method returns `("key", value)` pairs **without** the `$` — the `render()` method adds it via `format!("{{ ${key} }}")`.

- [ ] **Step 2: Add `Mail` enum variants**

After the `QuotaEventsPerDayReached` variant in `api/src/mailer.rs`, add:

```rust
    SubscriptionWarning {
        application_name: String,
        subscription_description: String, // fallback to subscription_id already applied
        subscription_id: Uuid,
        target_url: String,
        failure_percent: f64,
        evaluation_window: String,
    },
    SubscriptionDisabled {
        application_name: String,
        subscription_description: String,
        subscription_id: Uuid,
        target_url: String,
        failure_percent: f64,
        evaluation_window: String,
        disabled_at: String,
    },
    SubscriptionRecovered {
        application_name: String,
        subscription_description: String,
        subscription_id: Uuid,
        target_url: String,
    },
```

- [ ] **Step 3: Add `template()` match arms**

After the `QuotaEventsPerDayReached` arm in `template()`:

```rust
Mail::SubscriptionWarning { .. } => include_str!("mail_templates/subscriptions/warning.mjml"),
Mail::SubscriptionDisabled { .. } => include_str!("mail_templates/subscriptions/disabled.mjml"),
Mail::SubscriptionRecovered { .. } => include_str!("mail_templates/subscriptions/recovered.mjml"),
```

- [ ] **Step 4: Add `subject()` match arms**

```rust
Mail::SubscriptionWarning { ref subscription_description, .. } => {
    format!("[Hook0] Subscription failing: {subscription_description}")
}
Mail::SubscriptionDisabled { ref subscription_description, .. } => {
    format!("[Hook0] Subscription disabled: {subscription_description}")
}
Mail::SubscriptionRecovered { ref subscription_description, .. } => {
    format!("[Hook0] Subscription recovered: {subscription_description}")
}
```

- [ ] **Step 5: Add `variables()` match arms**

Map each field to `("key", value)` pairs. Use `failure_percent` as `format!("{:.0}", failure_percent)` for display (rounded, no decimals). Follow the existing `QuotaEventsPerDayWarning` pattern for the HashMap construction.

Commit: `feat(api): add subscription health email templates (warning, disabled, recovered)`

---

## Task 4: Config — Health Monitor Args

- [ ] **Step 1: Add clap args to `Config` struct**

In `api/src/main.rs`, add in the housekeeping section (after `soft_deleted_applications_cleanup_grace_period`):

```rust
    /// Enable the subscription health monitor background task
    #[clap(long, env, default_value_t = false)]
    enable_health_monitor: bool,

    /// How often the health monitor runs
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "30m")]
    health_monitor_interval: Duration,

    /// Failure % threshold for warning email
    #[clap(long, env, default_value_t = 80)]
    health_monitor_warning_failure_percent: u8,

    /// Failure % threshold for deactivation
    #[clap(long, env, default_value_t = 95)]
    health_monitor_disable_failure_percent: u8,

    /// Time window for low-volume health evaluation
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "24h")]
    health_monitor_time_window: Duration,

    /// Message count window for high-volume health evaluation
    #[clap(long, env, default_value_t = 100)]
    health_monitor_message_window: u32,

    /// Minimum completed attempts in window before health evaluation applies
    #[clap(long, env, default_value_t = 5)]
    health_monitor_min_sample_size: u32,

    /// Cooldown after resolved before new warning email
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "1h")]
    health_monitor_warning_cooldown: Duration,
```

- [ ] **Step 2: Add startup validation**

After config parsing, before State construction:

```rust
if config.enable_health_monitor {
    assert!(
        config.health_monitor_warning_failure_percent < config.health_monitor_disable_failure_percent,
        "health_monitor_warning_failure_percent ({}) must be < health_monitor_disable_failure_percent ({})",
        config.health_monitor_warning_failure_percent,
        config.health_monitor_disable_failure_percent,
    );
    assert!(
        (1..=100).contains(&config.health_monitor_warning_failure_percent)
            && (1..=100).contains(&config.health_monitor_disable_failure_percent),
        "health_monitor failure percents must be in [1, 100]"
    );
}
```

Commit: `feat(api): add health monitor configuration args with enable flag`

---

## Task 5: Health Monitor — Scaffold + Advisory Lock

- [ ] **Step 1: Register module**

Add `mod health_monitor;` in `api/src/main.rs` near the other module declarations.

- [ ] **Step 2: Create entry point**

```rust
// api/src/health_monitor.rs

use std::time::Duration;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::hook0_client::{
    EventSubscriptionDisabled, Hook0Client, Hook0ClientEvent,
    SubscriptionDisabledPayload, RetrySchedulePayload,
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

/// Runs the health monitor loop. Uses `while let` to exit gracefully
/// on semaphore close (shutdown).
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
    sleep(STARTUP_GRACE_PERIOD).await;
    info!(
        "Health monitor started (interval: {:?}, warning: {}%, disable: {}%)",
        config.interval, config.warning_failure_percent, config.disable_failure_percent
    );

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = run_health_check(db, mailer, hook0_client, config).await {
            error!("Health monitor error: {e}");
        }
        drop(permit);
        sleep(config.interval).await;
    }

    warn!("Health monitor stopped (semaphore closed)");
}
```

- [ ] **Step 3: Implement health check with transaction-level advisory lock**

```rust
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
        sqlx::query_scalar("select pg_try_advisory_xact_lock($1)")
            .bind(ADVISORY_LOCK_ID)
            .fetch_one(&mut *tx)
            .await?;

    if !acquired {
        info!("Health monitor: another instance holds the lock, skipping");
        // tx drops here -> rollback -> lock not held
        return Ok(());
    }

    let subscriptions = evaluate_subscriptions(&mut tx, config).await?;
    info!("Health monitor: evaluated {} subscriptions", subscriptions.len());

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
```

Commit: `feat(api): add health monitor scaffold with advisory lock`

---

## Task 6: Health Monitor — Evaluation Query

- [ ] **Step 1: Define the `SubscriptionHealth` struct**

```rust
#[derive(Debug)]
struct SubscriptionHealth {
    subscription__id: Uuid,
    application__id: Uuid,
    organization__id: Uuid,
    application_name: Option<String>,
    description: Option<String>,
    // Target URL resolved from webhook.target_http via target__id
    target_url: String,
    failure_percent: f64,
    total: i64,
    last_health_status: Option<String>,
    last_health_at: Option<DateTime<Utc>>,
    // Retry schedule fields for Hook0 event payload
    retry_schedule__id: Option<Uuid>,
    retry_schedule_name: Option<String>,
    retry_strategy: Option<String>,
    retry_max_retries: Option<i32>,
    retry_custom_intervals: Option<Vec<i32>>,
    retry_linear_delay: Option<i32>,
}
```

- [ ] **Step 2: Implement the evaluation query**

The query computes failure ratios with adaptive windowing: for subscriptions with >= `message_window` attempts in the time window, only the last `message_window` attempts are considered.

```rust
async fn evaluate_subscriptions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    let time_window_secs = config.time_window.as_secs() as i64;

    sqlx::query_as!(
        SubscriptionHealth,
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
              and ra.created_at > now() - make_interval(secs => $1::float8)
              and (ra.succeeded_at is not null or ra.failed_at is not null)
            group by ra.subscription__id
            having count(*) >= $2
        ),
        -- For high-volume subscriptions, recompute ratio over last N attempts only
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
            w.subscription__id as "subscription__id!",
            s.application__id as "application__id!",
            app.organization__id as "organization__id!",
            app.name as application_name,
            s.description,
            coalesce(th.url, '') as "target_url!",
            w.failure_percent as "failure_percent!",
            w.total as "total!",
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
        time_window_secs as f64,  // $1
        config.min_sample_size as i64,  // $2
        config.message_window as i64,  // $3
    )
    .fetch_all(&mut **tx)
    .await
}
```

Note: the `target_url` resolution via `webhook.target_http` may need adjustment — check the actual column name (likely `url` or `target_url`). Verify with `\d webhook.target_http`.

Commit: `feat(api): add health evaluation query with adaptive windowing`

---

## Task 7: Health Monitor — State Machine + Actions

- [ ] **Step 1: Implement the state machine dispatcher**

```rust
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
        // Disabled — skip
        Some("disabled") => {}

        // Resolved within cooldown — skip entirely (no events, no emails)
        Some("resolved")
            if last_at.map_or(false, |at| {
                (Utc::now() - at)
                    < chrono::Duration::from_std(config.warning_cooldown).unwrap_or_default()
            }) => {}

        // Warning + ratio still 80-95% — no action
        Some("warning") if ratio >= warning_pct && ratio < disable_pct => {}

        // Warning + ratio < 80% — resolved + recovery email
        Some("warning") if ratio < warning_pct => {
            insert_health_event(tx, sub.subscription__id, "resolved").await?;
            send_email_best_effort(mailer, tx, sub, EmailKind::Recovered).await;
        }

        // Warning + ratio >= 95% — disable
        Some("warning") => {
            disable_subscription(tx, mailer, hook0_client, sub).await?;
        }

        // None/resolved (past cooldown) + ratio >= 95% — warning + disable
        _ if ratio >= disable_pct => {
            insert_health_event(tx, sub.subscription__id, "warning").await?;
            send_email_best_effort(mailer, tx, sub, EmailKind::Warning).await;
            disable_subscription(tx, mailer, hook0_client, sub).await?;
        }

        // None/resolved (past cooldown) + ratio >= 80% — warning
        _ if ratio >= warning_pct => {
            insert_health_event(tx, sub.subscription__id, "warning").await?;
            send_email_best_effort(mailer, tx, sub, EmailKind::Warning).await;
        }

        // Healthy — no action
        _ => {}
    }

    Ok(())
}
```

- [ ] **Step 2: Implement helper — insert health event**

```rust
async fn insert_health_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscription_id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "insert into webhook.subscription_health_event (subscription__id, status) values ($1, $2)",
        subscription_id,
        status,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
```

- [ ] **Step 3: Implement helper — disable subscription (CTE, idempotent)**

```rust
async fn disable_subscription(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    mailer: &Mailer,
    hook0_client: &Option<Hook0Client>,
    sub: &SubscriptionHealth,
) -> Result<(), Box<dyn std::error::Error>> {
    let rows = sqlx::query!(
        r#"
        with updated as (
            update webhook.subscription
            set is_enabled = false
            where subscription__id = $1 and is_enabled = true
            returning subscription__id
        )
        insert into webhook.subscription_health_event (subscription__id, status)
        select subscription__id, 'disabled' from updated
        "#,
        sub.subscription__id,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();

    if rows == 0 {
        return Ok(()); // Already disabled
    }

    // Best-effort email
    send_email_best_effort(mailer, tx, sub, EmailKind::Disabled).await;

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
            warn!("Failed to send subscription.disabled Hook0 event: {e}");
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Implement email helpers**

```rust
enum EmailKind {
    Warning,
    Disabled,
    Recovered,
}

/// Best-effort email sending. Failures are logged, not propagated.
/// This differs from the quota email pattern (which rolls back on failure).
/// Here, the DB state is the critical path; email is advisory.
async fn send_email_best_effort(
    mailer: &Mailer,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sub: &SubscriptionHealth,
    kind: EmailKind,
) {
    // Resolve org members: subscription -> application -> organization -> users
    // Same pattern as api/src/quotas.rs send_organization_email_notification()
    let recipients = match sqlx::query!(
        r#"
        select u.first_name, u.last_name, u.email
        from iam.user u
        inner join iam.user__organization ou on u.user__id = ou.user__id
        where ou.organization__id = $1
        "#,
        sub.organization__id,
    )
    .fetch_all(&mut **tx)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("Health monitor: failed to resolve email recipients: {e}");
            return;
        }
    };

    let description = sub
        .description
        .clone()
        .unwrap_or_else(|| sub.subscription__id.to_string());

    for recipient in &recipients {
        let mail = match kind {
            EmailKind::Warning => Mail::SubscriptionWarning {
                application_name: sub.application_name.clone().unwrap_or_default(),
                subscription_description: description.clone(),
                subscription_id: sub.subscription__id,
                target_url: sub.target_url.clone(),
                failure_percent: sub.failure_percent,
                evaluation_window: "TODO".to_string(), // format from config
            },
            EmailKind::Disabled => Mail::SubscriptionDisabled {
                application_name: sub.application_name.clone().unwrap_or_default(),
                subscription_description: description.clone(),
                subscription_id: sub.subscription__id,
                target_url: sub.target_url.clone(),
                failure_percent: sub.failure_percent,
                evaluation_window: "TODO".to_string(),
                disabled_at: Utc::now().to_rfc3339(),
            },
            EmailKind::Recovered => Mail::SubscriptionRecovered {
                application_name: sub.application_name.clone().unwrap_or_default(),
                subscription_description: description.clone(),
                subscription_id: sub.subscription__id,
                target_url: sub.target_url.clone(),
            },
        };

        let mailbox = match format!(
            "{} {} <{}>",
            recipient.first_name, recipient.last_name, recipient.email
        )
        .parse()
        {
            Ok(m) => m,
            Err(e) => {
                warn!("Health monitor: invalid email address {}: {e}", recipient.email);
                continue;
            }
        };

        if let Err(e) = mailer.send_mail(mail, mailbox).await {
            warn!("Health monitor: failed to send email to {}: {e}", recipient.email);
        }
    }
}
```

Note: the `evaluation_window` field should be formatted from the config (e.g., `"24h"` or `"last 100 messages"`). Refine at implementation time.

Commit: `feat(api): add health monitor state machine, actions, and email helpers`

---

## Task 8: Spawn Health Monitor

- [ ] **Step 1: Spawn in `main.rs`**

After State construction and after mailer/hook0_client are created, add (gated by `enable_health_monitor`):

```rust
if config.enable_health_monitor {
    let hm_db = housekeeping_pool.clone();
    let hm_semaphore = housekeeping_semaphore.clone();
    let hm_mailer = mailer.clone();
    let hm_hook0_client = hook0_client.clone();
    let hm_config = health_monitor::HealthMonitorConfig {
        interval: config.health_monitor_interval,
        warning_failure_percent: config.health_monitor_warning_failure_percent,
        disable_failure_percent: config.health_monitor_disable_failure_percent,
        time_window: config.health_monitor_time_window,
        message_window: config.health_monitor_message_window,
        min_sample_size: config.health_monitor_min_sample_size,
        warning_cooldown: config.health_monitor_warning_cooldown,
    };
    actix_web::rt::spawn(async move {
        health_monitor::run_health_monitor(
            &hm_semaphore,
            &hm_db,
            &hm_mailer,
            &hm_hook0_client,
            &hm_config,
        )
        .await;
    });
}
```

Verify that `mailer` and `hook0_client` are cloned **before** they are moved into the `HttpServer::new` closure. Check that both implement `Clone`.

Commit: `feat(api): spawn health monitor background task`

---

## Task 9: Subscription API — `auto_disabled_at` + Re-activation CTE

- [ ] **Step 1: Add `auto_disabled_at` to response structs**

In `api/src/handlers/subscriptions.rs`, add `pub auto_disabled_at: Option<DateTime<Utc>>` to:
- The `Subscription` response struct (after `updated_at`)
- Both `RawSubscription` structs (in `list` and `get` handlers)

- [ ] **Step 2: Add LATERAL JOIN to list and get SQL queries**

In both the `list` and `get` SQL queries, add to the outer SELECT (after the `subs` CTE):

```sql
left join lateral (
    select she.created_at as auto_disabled_at
    from webhook.subscription_health_event she
    where she.subscription__id = subs.subscription__id
      and she.status = 'disabled'
    order by she.created_at desc
    limit 1
) health on true
```

Add `health.auto_disabled_at` to the SELECT list. Map it in the response construction.

- [ ] **Step 3: Modify PUT handler for re-activation CTE**

In the `edit` handler, detect when `is_enabled` transitions from `false` to `true`. Fetch the current `is_enabled` value before the update (it's available from the existing query). When transitioning, use the CTE pattern in the same transaction:

```rust
if body.is_enabled && !current_is_enabled {
    // Re-activation: insert resolved health event atomically
    sqlx::query!(
        r#"
        insert into webhook.subscription_health_event (subscription__id, status)
        values ($1, 'resolved')
        "#,
        subscription_id,
    )
    .execute(&mut *tx)
    .await?;
}
```

This runs within the existing transaction (`tx`) in the PUT handler. No separate CTE needed — the UPDATE and INSERT are in the same transaction.

Commit: `feat(api): add auto_disabled_at to subscription response + re-activation health event`

---

## Task 10: Build Verification + sqlx Prepare

- [ ] **Step 1: Build workspace**

```bash
cargo build
```

- [ ] **Step 2: Run sqlx prepare**

```bash
cd api && cargo sqlx prepare
```

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Commit: `chore: update sqlx prepared queries for health monitor`

---

## Task 11: Data Retention Cleanup

- [ ] **Step 1: Add cleanup function to `health_monitor.rs`**

```rust
/// Removes resolved health events older than 90 days,
/// keeping the latest event per subscription regardless of age.
async fn cleanup_old_health_events(db: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        delete from webhook.subscription_health_event d
        where d.created_at < now() - interval '90 days'
          and d.status = 'resolved'
          and exists (
            select 1 from webhook.subscription_health_event newer
            where newer.subscription__id = d.subscription__id
              and newer.created_at > d.created_at
          )
        "#
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
```

- [ ] **Step 2: Call cleanup once per day from the health monitor loop**

Track `last_cleanup_at` in the loop. Run cleanup when > 24h since last run:

```rust
// In run_health_monitor, before the while let loop:
let mut last_cleanup = Instant::now() - Duration::from_secs(86400); // force first run

// Inside the loop, after run_health_check:
if last_cleanup.elapsed() > Duration::from_secs(86400) {
    match cleanup_old_health_events(db).await {
        Ok(n) => { if n > 0 { info!("Health monitor: cleaned up {n} old health events"); } }
        Err(e) => warn!("Health monitor: cleanup error: {e}"),
    }
    last_cleanup = Instant::now();
}
```

Commit: `feat(api): add health event cleanup for data retention (90 days)`
