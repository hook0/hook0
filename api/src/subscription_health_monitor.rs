//! Subscription health monitor — watches customer webhooks and reacts when they fail too much.
//!
//! How it works:
//! 1. A background loop ticks on a fixed interval — one tick = one DB pass.
//! 2. Each tick reads new delivery attempts, aggregates per subscription, and asks the pure state
//!    machine what to do: warn, disable, or resolve.
//! 3. Decided actions apply in the same transaction — subscription row update, health event
//!    insert, or auto-disable.
//! 4. Once a day it also purges old health events and stale buckets.
//!
//! Pure decision in `state_machine`, DB I/O in `candidates`. This file is the orchestrator.

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query, query_as, query_scalar};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use strum::Display;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// --- Public Types (API / Configuration) ---

/// Operational knobs for the monitor — thresholds, scan limits, retention windows.
#[derive(Clone)]
pub struct SubscriptionHealthMonitorConfig {
    pub interval: Duration,
    pub failure_percent_for_warning: u8,
    pub failure_percent_for_disable: u8,
    pub failure_rate_window: Duration,
    pub min_request_attempts: u32,
    pub anti_flap_window: Duration,
    pub resolved_event_retention: Duration,
    pub bucket_max_duration: Duration,
    pub bucket_max_request_attempts: u32,
    pub bucket_retention: Duration,
    pub request_attempt_scan_cap_per_tick: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthStatus {
    Warning,
    Disabled,
    Resolved,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Apiv2Schema, sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum HealthEventCause {
    Auto,
    Manual,
}

#[derive(Debug)]
pub struct SubscriptionHealth {
    pub subscription_id: Uuid,
    pub failure_percent: f64,
    pub last_health_status: Option<HealthStatus>,
    pub last_health_at: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub last_health_cause: Option<HealthEventCause>,
    #[allow(dead_code)]
    pub last_health_user_id: Option<Uuid>,
}

// --- Background Daemon ---

// Purge runs once a day — health history is append-only and small, no point scanning more often.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Watches customer webhook subscriptions forever:
/// - warns when recent failures cross the warning threshold
/// - auto-disables when they cross the disable threshold
/// - emits a recovery event when a warned endpoint starts succeeding again
///
/// Returns only when the semaphore is closed (shutdown).
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

    // Shared across housekeeping tasks — holding a permit bounds DB load from concurrent jobs.
    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = run_one_tick(db, config).await {
            // Log and keep looping — transient DB errors shouldn't kill the monitor.
            // Next tick retries the same window via the cursor.
            error!("Subscription health monitor error: {e}");
        }

        // Daily cleanup for stale events and buckets
        if last_cleanup.is_none_or(|t| t.elapsed() > CLEANUP_INTERVAL) {
            let log_cleanup = |name: &str, result: Result<u64, sqlx::Error>| match result {
                Ok(n) if n > 0 => info!("Subscription health monitor: cleaned up {n} {name}"),
                Ok(_) => debug!("Subscription health monitor: nothing to remove for {name}"),
                Err(e) => warn!("Subscription health monitor: cleanup error for {name}: {e}"),
            };

            // Drop old "resolved" events only if a newer event exists for the same subscription.
            let resolved_cleanup = query!(
                r#"
                    delete from webhook.subscription_health_event d
                    where d.created_at < now() - make_interval(secs => $1)
                      and d.status = 'resolved'
                      and exists (
                          select 1 from webhook.subscription_health_event newer
                          where newer.subscription__id = d.subscription__id
                            and newer.created_at > d.created_at
                      )
                "#,
                config.resolved_event_retention.as_secs_f64(),
            )
            .execute(db)
            .await
            .map(|r| r.rows_affected());

            log_cleanup("resolved subscription_health_event rows", resolved_cleanup);

            // Purge old pre-aggregated buckets
            let buckets_cleanup = query!(
                r#"
                    delete from webhook.subscription_health_bucket
                    where bucket_start < now() - make_interval(secs => $1)
                "#,
                config.bucket_retention.as_secs_f64(),
            )
            .execute(db)
            .await
            .map(|r| r.rows_affected());

            log_cleanup("old subscription_health_bucket rows", buckets_cleanup);

            last_cleanup = Some(Instant::now());
        }

        // Release the permit before sleeping so other housekeeping tasks can run during the idle window.
        drop(permit);
        actix_web::rt::time::sleep(config.interval).await;
    }

    warn!("Subscription health monitor stopped (semaphore closed)");
}

// --- Tick Logic ---

#[derive(Debug, Clone, PartialEq)]
enum PlannedAction {
    UpdateFailurePercent,
    EmitWarning,
    EmitResolved,
    EmitDisabled,
}

// Arbitrary fixed key for `pg_try_advisory_xact_lock`. Guarantees one API node per tick.
const ADVISORY_LOCK_ID: i64 = 42_000_001;

// Hard ceiling per tick — prevents long DB locks holding up the monitor.
const TICK_STATEMENT_TIMEOUT: &str = "5min";

/// One pass of the monitor:
/// - Read new delivery attempts since the cursor
/// - Ask the state machine what to do for each subscription
/// - Apply updates and health events in a single transaction
async fn run_one_tick(
    db: &PgPool,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    // Timeout dies with the tx — no leak into the next tick's batch.
    query(&format!(
        "set local statement_timeout = '{TICK_STATEMENT_TIMEOUT}'"
    ))
    .execute(&mut *tx)
    .await?;

    // Advisory transaction lock — a Postgres mutex keyed on an arbitrary int. Tied to the tx so a
    // crashed worker can't leave it held. Non-blocking: skip if already held.
    let acquired: bool = query_scalar("select pg_try_advisory_xact_lock($1)")
        .bind(ADVISORY_LOCK_ID)
        .fetch_one(&mut *tx)
        .await?;

    if !acquired {
        info!("Subscription health monitor: another instance holds the lock, skipping");
        return Ok(());
    }

    // Advance cursor and fetch subscriptions whose failure rate has changed
    let subscriptions = list_candidates(&mut tx, config).await?;
    info!(
        "Subscription health monitor: evaluated {} subscriptions",
        subscriptions.len()
    );

    // Single `now` per batch — anti-flap windows stay consistent across the tick.
    let now = Utc::now();

    for subscription in &subscriptions {
        // Step 1: Pure state machine decision (immutable)
        let planned_actions = {
            let failure = subscription.failure_percent;
            let warning = f64::from(config.failure_percent_for_warning);
            let disable = f64::from(config.failure_percent_for_disable);

            let anti_flap =
                ChronoDuration::from_std(config.anti_flap_window).unwrap_or(ChronoDuration::zero());

            // Determine if an event must be emitted based on current status and failure rates
            let state_action = match subscription.last_health_status {
                Some(HealthStatus::Disabled) => None,

                Some(HealthStatus::Resolved)
                    if subscription
                        .last_health_at
                        .is_some_and(|at| (now - at) < anti_flap) =>
                {
                    None // Ignored during the anti-flap cooldown
                }

                Some(HealthStatus::Warning) => {
                    if failure < warning {
                        Some(PlannedAction::EmitResolved)
                    } else if failure >= disable {
                        Some(PlannedAction::EmitDisabled)
                    } else {
                        None // Still in warning zone
                    }
                }

                None | Some(HealthStatus::Resolved) => {
                    if failure >= disable {
                        Some(PlannedAction::EmitDisabled)
                    } else if failure >= warning {
                        Some(PlannedAction::EmitWarning)
                    } else {
                        None // Healthy
                    }
                }
            };

            // Every processed subscription gets its percentage updated.
            match state_action {
                Some(action) => vec![PlannedAction::UpdateFailurePercent, action],
                None => vec![PlannedAction::UpdateFailurePercent],
            }
        };

        // Step 2: Apply decided actions directly to the database
        for action in &planned_actions {
            let action_result = match action {
                PlannedAction::UpdateFailurePercent => query!(
                    "update webhook.subscription set failure_percent = $1 where subscription__id = $2",
                    subscription.failure_percent,
                    subscription.subscription_id,
                )
                .execute(&mut *tx)
                .await
                .map(|_| ()),

                PlannedAction::EmitWarning => query!(
                    r#"
                        insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
                        values ($1, 'warning', 'auto', null)
                    "#,
                    subscription.subscription_id,
                )
                .execute(&mut *tx)
                .await
                .map(|_| ()),

                PlannedAction::EmitResolved => query!(
                    r#"
                        insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
                        values ($1, 'resolved', 'auto', null)
                    "#,
                    subscription.subscription_id,
                )
                .execute(&mut *tx)
                .await
                .map(|_| ()),

                PlannedAction::EmitDisabled => {
                    // Atomic disable + event insert in a single CTE. A manual re-enable between 
                    // our read and write stays intact.
                    let disabled_at = query_scalar!(
                        r#"
                            with updated as (
                                update webhook.subscription
                                set is_enabled = false
                                where subscription__id = $1 and is_enabled = true
                                returning subscription__id
                            ),
                            inserted as (
                                insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
                                select subscription__id, 'disabled', 'auto', null from updated
                                returning created_at
                            )
                            select created_at as "created_at!" from inserted
                        "#,
                        subscription.subscription_id,
                    )
                    .fetch_optional(&mut *tx)
                    .await;

                    match disabled_at {
                        Ok(Some(_)) => {
                            info!(
                                "Subscription health monitor: disabled subscription {}",
                                subscription.subscription_id
                            );
                            Ok(())
                        }
                        // Already disabled by someone else — deliberate user action, don't touch.
                        Ok(None) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
            };

            if let Err(e) = action_result {
                // Per-subscription failure — log and keep going to avoid rolling back the whole batch.
                warn!(
                    "Subscription health monitor: error processing subscription {}: {e}",
                    subscription.subscription_id
                );
            }
        }
    }

    // Commit atomically — cursor advance and health events land together.
    // A crash mid-tick replays the exact same window on the next run.
    tx.commit().await?;
    Ok(())
}

// --- Database Candidates Fetching ---

struct RequestAttemptAggregate {
    subscription_id: Uuid,
    total: i64,
    failed: i64,
}

pub(super) async fn list_candidates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
    // 1. Fetch current position
    let cursor: DateTime<Utc> = query_scalar!(
        "select last_processed_at from webhook.subscription_health_monitor_cursor where cursor__id = 1",
    )
    .fetch_one(&mut **tx)
    .await?;

    struct AggregateRow {
        subscription_id: Option<Uuid>,
        total: Option<i64>,
        failed: Option<i64>,
        max_completed_at: Option<DateTime<Utc>>,
    }

    let scan_cap = i64::from(config.request_attempt_scan_cap_per_tick);

    // 2. Fetch and aggregate unread requests
    let rows = query_as!(
        AggregateRow,
        r#"
            with new_request_attempts as (
                select subscription__id, failed_at, succeeded_at
                from webhook.request_attempt
                where coalesce(succeeded_at, failed_at) > $1
                  and (succeeded_at is not null or failed_at is not null)
                order by coalesce(succeeded_at, failed_at)
                limit $2
            )
            select subscription__id as "subscription_id: Uuid",
                   count(*) as "total: i64",
                   count(failed_at) as "failed: i64",
                   null::timestamptz as "max_completed_at: DateTime<Utc>"
            from new_request_attempts
            group by subscription__id
            union all
            select null::uuid,
                   null::bigint,
                   null::bigint,
                   max(coalesce(succeeded_at, failed_at))
            from new_request_attempts
        "#,
        cursor,
        scan_cap,
    )
    .fetch_all(&mut **tx)
    .await?;

    let max_completed_at = rows
        .iter()
        .find(|row| row.subscription_id.is_none())
        .and_then(|row| row.max_completed_at);

    let aggregates: Vec<RequestAttemptAggregate> = rows
        .into_iter()
        .filter_map(|row| {
            Some(RequestAttemptAggregate {
                subscription_id: row.subscription_id?,
                total: row.total.unwrap_or(0),
                failed: row.failed.unwrap_or(0),
            })
        })
        .collect();

    // 3. Upsert into buckets for current window evaluation
    if !aggregates.is_empty() {
        let input_ids: Vec<Uuid> = aggregates.iter().map(|a| a.subscription_id).collect();

        let open_rows = query!(
            r#"
                select subscription__id as "subscription_id!", bucket_start as "bucket_start!"
                from webhook.subscription_health_bucket
                where subscription__id = any($1)
                  and bucket_end is null
            "#,
            &input_ids,
        )
        .fetch_all(&mut **tx)
        .await?;

        let open_buckets: HashMap<Uuid, DateTime<Utc>> = open_rows
            .into_iter()
            .map(|r| (r.subscription_id, r.bucket_start))
            .collect();
        let now = Utc::now();

        // Preparing parallel arrays for postgres `unnest` bulk insert
        // Preparing parallel arrays for postgres `unnest` bulk insert
        let subscription_ids: Vec<Uuid> = aggregates.iter().map(|a| a.subscription_id).collect();
        let bucket_starts: Vec<DateTime<Utc>> = aggregates
            .iter()
            .map(|a| open_buckets.get(&a.subscription_id).copied().unwrap_or(now))
            .collect();
        let totals: Vec<i32> = aggregates
            .iter()
            .map(|a| i32::try_from(a.total).unwrap_or(i32::MAX))
            .collect();
        let faileds: Vec<i32> = aggregates
            .iter()
            .map(|a| i32::try_from(a.failed).unwrap_or(i32::MAX))
            .collect();

        query!(
            r#"
                insert into webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count)
                select * from unnest($1::uuid[], $2::timestamptz[], $3::int4[], $4::int4[])
                on conflict (subscription__id, bucket_start) do update set
                    total_count = subscription_health_bucket.total_count + excluded.total_count,
                    failed_count = subscription_health_bucket.failed_count + excluded.failed_count
            "#,
            &subscription_ids,
            &bucket_starts,
            &totals,
            &faileds,
        )
        .execute(&mut **tx)
        .await?;
    }

    // 4. Close buckets that reached max duration or max attempts
    let bucket_max_duration_secs = config.bucket_max_duration.as_secs_f64();
    let bucket_max_request_attempts =
        i32::try_from(config.bucket_max_request_attempts).unwrap_or(i32::MAX);

    query!(
        r#"
            update webhook.subscription_health_bucket
            set bucket_end = now()
            where bucket_end is null
              and (bucket_start < now() - make_interval(secs => $1)
                   or total_count >= $2)
        "#,
        bucket_max_duration_secs,
        bucket_max_request_attempts,
    )
    .execute(&mut **tx)
    .await?;

    // 5. Evaluate overall subscription failure rate across recent buckets
    let evaluation_window_secs = config.failure_rate_window.as_secs_f64();
    let min_request_attempts = i64::from(config.min_request_attempts);

    let subscriptions = query_as!(
        SubscriptionHealth,
        r#"
            with bucket_stats as (
                select subscription__id,
                       sum(total_count) as total_count,
                       sum(failed_count) as failed_count,
                       sum(failed_count)::float8 / nullif(sum(total_count), 0) * 100.0 as failure_percent
                from webhook.subscription_health_bucket
                where bucket_start > now() - make_interval(secs => $1)
                group by subscription__id
                having sum(total_count) >= $2
            ),
            candidates as (
                select subscription__id from bucket_stats where failed_count > $2
                union
                select subscription__id
                from (
                    select distinct on (subscription__id) subscription__id, status
                    from webhook.subscription_health_event
                    order by subscription__id, created_at desc
                ) latest
                where latest.status = 'warning'
            )
            select
                bs.subscription__id as "subscription_id!",
                bs.failure_percent as "failure_percent!",
                lh.status as "last_health_status?: HealthStatus",
                lh.created_at as "last_health_at?",
                lh.cause as "last_health_cause?: HealthEventCause",
                lh.user__id as "last_health_user_id?"
            from candidates c
            inner join bucket_stats bs on bs.subscription__id = c.subscription__id
            inner join webhook.subscription s on s.subscription__id = c.subscription__id
            left join lateral (
                select she.status, she.created_at, she.cause, she.user__id
                from webhook.subscription_health_event she
                where she.subscription__id = c.subscription__id
                order by she.created_at desc
                limit 1
            ) lh on true
            where s.is_enabled = true and s.deleted_at is null
        "#,
        evaluation_window_secs,
        min_request_attempts,
    )
    .fetch_all(&mut **tx)
    .await?;

    // 6. Reset failure rate on currently inactive/healthy subscriptions
    let active_ids: Vec<Uuid> = subscriptions.iter().map(|s| s.subscription_id).collect();
    query!(
        r#"
            update webhook.subscription
            set failure_percent = null
            where failure_percent is not null
              and subscription__id <> all($1)
        "#,
        &active_ids,
    )
    .execute(&mut **tx)
    .await?;

    // 7. Finally, bump the cursor forward
    if let Some(ts) = max_completed_at {
        query!(
            r#"
                update webhook.subscription_health_monitor_cursor
                set last_processed_at = $1
                where cursor__id = 1
                  and $1 > last_processed_at
            "#,
            ts,
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(subscriptions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use sqlx::{PgPool, query};
    use std::time::Duration;
    use uuid::Uuid;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_bucket_aggregation_and_cursor(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(1);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;

        // 3 successes, 2 failures
        let (_org_id, _app_id, sub_id, _secret_token) =
            insert_test_fixtures(&mut tx, 3, 2, now).await;

        // Fills buckets and advances cursor as side effect
        let subs = list_candidates(&mut tx, &config).await.unwrap();

        // 1. Bucket holds aggregated counts
        let bucket = query!(
            r#"
            select total_count, failed_count
            from webhook.subscription_health_bucket
            where subscription__id = $1
        "#,
            sub_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .unwrap()
        .expect("bucket should exist after fetching candidates");

        assert_eq!(bucket.total_count, 5);
        assert_eq!(bucket.failed_count, 2);

        // 2. Subscription appears in candidates
        let candidate_ids: Vec<Uuid> = subs.iter().map(|s| s.subscription_id).collect();
        assert!(candidate_ids.contains(&sub_id));

        // 3. Cursor moved forward
        let cursor_after = sqlx::query_scalar!(
        "select last_processed_at from webhook.subscription_health_monitor_cursor where cursor__id = 1",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();

        assert!(
            cursor_after > cursor_past,
            "cursor should have advanced past the initial state"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_e2e_warning_lifecycle(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(2);

        // --- PHASE 1: Warning ---
        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        // 2 successes, 5 failures → ~71% (warning zone)
        let (_org_id, app_id, sub_id, secret_token) =
            insert_test_fixtures(&mut tx, 2, 5, now).await;
        tx.commit().await.unwrap(); // Commit so run_one_tick sees fixtures

        run_one_tick(&pool, &config).await.unwrap();

        let warning_event = sqlx::query_scalar!(
        "select count(*) from webhook.subscription_health_event where subscription__id = $1 and status = 'warning'",
        sub_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
        assert_eq!(
            warning_event,
            Some(1),
            "Sub should have triggered a warning event"
        );

        // --- PHASE 2: Recovery ---
        // Clear old buckets to simulate time passing
        query!(
            "delete from webhook.subscription_health_bucket where subscription__id = $1",
            sub_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        // 10 successes, 0 failures → 0% (recovery)
        insert_attempts(&mut tx, app_id, sub_id, secret_token, 10, 0, Utc::now()).await;
        tx.commit().await.unwrap();

        run_one_tick(&pool, &config).await.unwrap();

        let resolved_event = sqlx::query_scalar!(
        "select count(*) from webhook.subscription_health_event where subscription__id = $1 and status = 'resolved'",
        sub_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
        assert_eq!(
            resolved_event,
            Some(1),
            "Sub should have triggered a resolved event"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_e2e_disable_lifecycle(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        // 0 successes, 10 failures → 100% (above 90% disable threshold)
        let (_org_id, _app_id, sub_id, _secret_token) =
            insert_test_fixtures(&mut tx, 0, 10, now).await;
        tx.commit().await.unwrap();

        run_one_tick(&pool, &config).await.unwrap();

        let disabled_event = sqlx::query_scalar!(
        "select count(*) from webhook.subscription_health_event where subscription__id = $1 and status = 'disabled'",
            sub_id,
        )
    .fetch_one(&pool)
    .await
    .unwrap();
        assert_eq!(
            disabled_event,
            Some(1),
            "Sub should have triggered a disabled event"
        );

        let is_enabled = sqlx::query_scalar!(
            "select is_enabled from webhook.subscription where subscription__id = $1",
            sub_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            is_enabled, false,
            "Subscription should be actually disabled in DB"
        );
    }

    // ============================================================================
    // HELPERS & FIXTURES
    // ============================================================================

    fn test_config() -> SubscriptionHealthMonitorConfig {
        SubscriptionHealthMonitorConfig {
            interval: Duration::from_secs(60),
            failure_percent_for_warning: 50,
            failure_percent_for_disable: 90,
            failure_rate_window: Duration::from_secs(3600),
            min_request_attempts: 1,
            anti_flap_window: Duration::from_secs(3600),
            resolved_event_retention: Duration::from_secs(30 * 86_400),
            bucket_max_duration: Duration::from_secs(300),
            bucket_max_request_attempts: 100,
            bucket_retention: Duration::from_secs(30 * 86_400),
            request_attempt_scan_cap_per_tick: 50_000,
        }
    }

    async fn set_cursor(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, ts: DateTime<Utc>) {
        query!(
        "update webhook.subscription_health_monitor_cursor set last_processed_at = $1 where cursor__id = 1",
        ts,
    )
    .execute(&mut **tx)
    .await
    .unwrap();
    }

    async fn insert_test_fixtures(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        num_succeeded: i32,
        num_failed: i32,
        attempts_timestamp: DateTime<Utc>,
    ) -> (Uuid, Uuid, Uuid, Uuid) {
        let org_id = Uuid::now_v7();
        let app_id = Uuid::now_v7();
        let sub_id = Uuid::now_v7();
        let target_id = sub_id;
        let secret_token = Uuid::now_v7();

        // Relational tree: Org → App → Subscription → Webhook
        query!(
            "INSERT INTO iam.organization (organization__id, name, created_by) VALUES ($1, $2, $3)",
            org_id,
            "test-org-health",
            Uuid::nil(),
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        query!(
        "INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, $3)",
        app_id,
        org_id,
        "test-app",
    )
    .execute(&mut **tx)
    .await
    .unwrap();

        query!(
            "INSERT INTO event.service (service__name, application__id) VALUES ($1, $2)",
            "svc",
            app_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        query!(
        "INSERT INTO event.resource_type (resource_type__name, application__id, service__name) VALUES ($1, $2, $3)",
        "res",
        app_id,
        "svc",
    )
    .execute(&mut **tx)
    .await
    .unwrap();

        query!(
            "INSERT INTO event.verb (verb__name, application__id) VALUES ($1, $2)",
            "created",
            app_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        query!(
        "INSERT INTO event.event_type (application__id, service__name, resource_type__name, verb__name) VALUES ($1, $2, $3, $4)",
        app_id,
        "svc",
        "res",
        "created",
    )
    .execute(&mut **tx)
    .await
    .unwrap();

        query!(
            "INSERT INTO event.application_secret (token, application__id) VALUES ($1, $2)",
            secret_token,
            app_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        query!(
            r#"INSERT INTO webhook.subscription
               (subscription__id, application__id, target__id, is_enabled, labels)
               VALUES ($1, $2, $3, true, '{"env":"test"}'::jsonb)"#,
            sub_id,
            app_id,
            target_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        query!(
            "INSERT INTO webhook.target_http (target__id, method, url) VALUES ($1, $2, $3)",
            target_id,
            "POST",
            "https://example.com/webhook",
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        insert_attempts(
            tx,
            app_id,
            sub_id,
            secret_token,
            num_succeeded,
            num_failed,
            attempts_timestamp,
        )
        .await;

        (org_id, app_id, sub_id, secret_token)
    }

    async fn insert_attempts(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        app_id: Uuid,
        sub_id: Uuid,
        secret_token: Uuid,
        num_succeeded: i32,
        num_failed: i32,
        attempts_timestamp: DateTime<Utc>,
    ) {
        query!("ALTER TABLE event.event DISABLE TRIGGER event_dispatch")
            .execute(&mut **tx)
            .await
            .unwrap();

        for i in 0..(num_succeeded + num_failed) {
            let event_id = Uuid::now_v7();
            let attempt_id = Uuid::now_v7();
            let is_failed = i >= num_succeeded;

            query!(
                r#"INSERT INTO event.event
                   (event__id, application__id, event_type__name, payload_content_type, ip, occurred_at, application_secret__token, labels)
                   VALUES ($1, $2, 'svc.res.created', 'application/json', '127.0.0.1'::inet, $3, $4, '{"env":"test"}'::jsonb)"#,
                event_id,
                app_id,
                attempts_timestamp,
                secret_token,
            )
            .execute(&mut **tx)
            .await
            .unwrap();

            if is_failed {
                query!(
                    r#"INSERT INTO webhook.request_attempt
                       (request_attempt__id, event__id, subscription__id, application__id, failed_at)
                       VALUES ($1, $2, $3, $4, $5)"#,
                    attempt_id,
                    event_id,
                    sub_id,
                    app_id,
                    attempts_timestamp,
                )
                .execute(&mut **tx)
                .await
                .unwrap();
            } else {
                query!(
                    r#"INSERT INTO webhook.request_attempt
                       (request_attempt__id, event__id, subscription__id, application__id, succeeded_at)
                       VALUES ($1, $2, $3, $4, $5)"#,
                    attempt_id,
                    event_id,
                    sub_id,
                    app_id,
                    attempts_timestamp,
                )
                .execute(&mut **tx)
                .await
                .unwrap();
            }
        }

        query!("ALTER TABLE event.event ENABLE TRIGGER event_dispatch")
            .execute(&mut **tx)
            .await
            .unwrap();
    }
}
