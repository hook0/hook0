use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sqlx::{PgPool, query, query_as, query_scalar};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Tuning knobs for one running health monitor instance.
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

/// Subscription health verdict emitted by monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
enum HealthStatus {
    Warning,
    Disabled,
    Resolved,
}

/// Origin of health event: monitor or operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
enum HealthEventCause {
    Auto,
    Manual,
}

/// Evaluated subscription with failure rate and last health event.
#[derive(Debug)]
struct SubscriptionHealth {
    subscription_id: Uuid,
    failure_percent: f64,
    last_health_status: Option<HealthStatus>,
    last_health_at: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    last_health_cause: Option<HealthEventCause>,
    #[allow(dead_code)]
    last_health_user_id: Option<Uuid>,
}

const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Evaluates subscription failure rates, emits health events.
///
/// Operator sees warning/disabled/resolved per subscription in logs.
/// Cleans up stale resolved events and old buckets daily.
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
        // run_one_tick is where we do the main work
        if let Err(e) = run_one_tick(db, config).await {
            error!("Subscription health monitor error: {e}");
        }

        if last_cleanup.is_none_or(|t| t.elapsed() > CLEANUP_INTERVAL) {
            let log_cleanup = |name: &str, result: Result<u64, sqlx::Error>| match result {
                Ok(n) if n > 0 => info!("Subscription health monitor: cleaned up {n} {name}"),
                Ok(_) => debug!("Subscription health monitor: nothing to remove for {name}"),
                Err(e) => warn!("Subscription health monitor: cleanup error for {name}: {e}"),
            };

            // Keeps latest resolved event per subscription; older purged.
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

        // Release permit before sleep; other tasks run during wait.
        drop(permit);
        actix_web::rt::time::sleep(config.interval).await;
    }

    warn!("Subscription health monitor stopped (semaphore closed)");
}

/// Side effect scheduled for one subscription during tick.
#[derive(Debug, Clone, PartialEq)]
enum SubscriptionAction {
    UpdateFailurePercent,
    EmitWarning,
    EmitResolved,
    EmitDisabled,
}

// Prevents concurrent ticks across API replicas.
const ADVISORY_LOCK_ID: i64 = 42_000_001;

// Safety cap; kills runaway tick queries.
const TICK_STATEMENT_TIMEOUT: &str = "5min";

/// Single evaluation pass over all active subscriptions.
///
/// Operator sees state transitions logged per subscription.
/// Entire tick runs in one transaction with advisory lock.
async fn run_one_tick(
    db: &PgPool,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    // Scoped to this transaction; resets automatically on commit/rollback.
    query(&format!(
        "set local statement_timeout = '{TICK_STATEMENT_TIMEOUT}'"
    ))
    .execute(&mut *tx)
    .await?;

    let acquired: bool = query_scalar("select pg_try_advisory_xact_lock($1)")
        .bind(ADVISORY_LOCK_ID)
        .fetch_one(&mut *tx)
        .await?;

    if !acquired {
        info!("Subscription health monitor: another instance holds the lock, skipping");
        return Ok(());
    }

    let candidates_subscriptions =
        list_health_evaluation_subscriptions_candidates(&mut tx, config).await?;
    info!(
        "Subscription health monitor: evaluated {} subscriptions",
        candidates_subscriptions.len()
    );

    // Shared `now` for anti-flap comparison across all subscriptions.
    let now = Utc::now();

    for candidate_subscription in &candidates_subscriptions {
        let candidate_subscription_actions = {
            let failure_percent = candidate_subscription.failure_percent;
            let warning_threshold = f64::from(config.failure_percent_for_warning);
            let disable_threshold = f64::from(config.failure_percent_for_disable);

            // Overflowed chrono range → zero disables anti-flap safely.
            let anti_flap =
                ChronoDuration::from_std(config.anti_flap_window).unwrap_or(ChronoDuration::zero());

            let in_anti_flap = candidate_subscription
                .last_health_at
                .is_some_and(|at| (now - at) < anti_flap);

            // 1. Determine new status based on metrics
            let new_status = if failure_percent >= disable_threshold {
                HealthStatus::Disabled
            } else if failure_percent >= warning_threshold {
                HealthStatus::Warning
            } else {
                HealthStatus::Resolved
            };

            let state_transition_action =
                match (candidate_subscription.last_health_status, new_status) {
                    // Disabled state is terminal
                    (Some(HealthStatus::Disabled), _) => None,

                    // Anti-flap protection: suppress state changes
                    (Some(HealthStatus::Resolved), _) if in_anti_flap => None,

                    // No actual state change
                    (Some(old), new) if old == new => None,
                    (None, HealthStatus::Resolved) => None, // None is implicitly Resolved

                    // Valid transitions to a new state
                    (_, HealthStatus::Disabled) => Some(SubscriptionAction::EmitDisabled),
                    (_, HealthStatus::Warning) => Some(SubscriptionAction::EmitWarning),
                    (_, HealthStatus::Resolved) => Some(SubscriptionAction::EmitResolved),
                };

            // 3. Assemble actions
            let mut actions = vec![SubscriptionAction::UpdateFailurePercent];
            if let Some(new_action) = state_transition_action {
                actions.push(new_action);
            }
            actions
        };

        for new_action in &candidate_subscription_actions {
            let action_result = match new_action {
                SubscriptionAction::UpdateFailurePercent => query!(
                    "update webhook.subscription set failure_percent = $1 where subscription__id = $2",
                    candidate_subscription.failure_percent,
                    candidate_subscription.subscription_id,
                )
                .execute(&mut *tx)
                .await
                .map(|_| ()),

                SubscriptionAction::EmitWarning => query!(
                    r#"
                        insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
                        values ($1, 'warning', 'auto', null)
                    "#,
                    candidate_subscription.subscription_id,
                )
                .execute(&mut *tx)
                .await
                .map(|_| ()),

                SubscriptionAction::EmitResolved => query!(
                    r#"
                        insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
                        values ($1, 'resolved', 'auto', null)
                    "#,
                    candidate_subscription.subscription_id,
                )
                .execute(&mut *tx)
                .await
                .map(|_| ()),

                // Atomic disable + event insert. Prevents re-enable race.
                SubscriptionAction::EmitDisabled => {
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
                        candidate_subscription.subscription_id,
                    )
                    .fetch_optional(&mut *tx)
                    .await;

                    match disabled_at {
                        Ok(Some(_)) => {
                            info!(
                                "Subscription health monitor: disabled subscription {}",
                                candidate_subscription.subscription_id
                            );
                            Ok(())
                        }
                        Ok(None) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
            };

            if let Err(e) = action_result {
                warn!(
                    "Subscription health monitor: error processing subscription {}: {e}",
                    candidate_subscription.subscription_id
                );
            }
        }
    }

    tx.commit().await?;
    Ok(())
}

/// Success/failure counts for one subscription in current scan.
struct RequestAttemptAggregate {
    subscription_id: Uuid,
    total: i64,
    failed: i64,
}

/// Identifies subscriptions whose failure rate crossed threshold.
///
/// Returns evaluated subscriptions with failure percent and last health event.
/// - Advances cursor through request_attempt incrementally.
/// - Accumulates subscription counts into time-bounded buckets.
/// - Closes expired/full buckets, evaluates subscriptions over sliding window.
/// - Clears failure_percent on subscriptions no longer evaluation candidates.
async fn list_health_evaluation_subscriptions_candidates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &SubscriptionHealthMonitorConfig,
) -> Result<Vec<SubscriptionHealth>, sqlx::Error> {
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

    // Single round-trip: per-subscription request attempt aggregates + max timestamp.
    // Sentinel row (subscription_id NULL) carries max_completed_at.
    let request_attempt_aggregate_rows = query_as!(
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

    // Extract sentinel row for cursor advance.
    let max_completed_at = request_attempt_aggregate_rows
        .iter()
        .find(|row| row.subscription_id.is_none())
        .and_then(|row| row.max_completed_at);

    let request_attempts_aggregates: Vec<RequestAttemptAggregate> = request_attempt_aggregate_rows
        .into_iter()
        .filter_map(|row| {
            Some(RequestAttemptAggregate {
                subscription_id: row.subscription_id?,
                total: row.total.unwrap_or(0),
                failed: row.failed.unwrap_or(0),
            })
        })
        .collect();

    // New request attempts found — upsert into buckets, then close full/expired ones.
    if !request_attempts_aggregates.is_empty() {
        let subscriptions_with_new_attempts: Vec<Uuid> = request_attempts_aggregates
            .iter()
            .map(|a| a.subscription_id)
            .collect();

        let open_bucket_rows = query!(
            r#"
                select subscription__id as "subscription_id!", bucket_start as "bucket_start!"
                from webhook.subscription_health_bucket
                where subscription__id = any($1)
                  and bucket_end is null
            "#,
            &subscriptions_with_new_attempts,
        )
        .fetch_all(&mut **tx)
        .await?;

        let open_buckets: HashMap<Uuid, DateTime<Utc>> = open_bucket_rows
            .into_iter()
            .map(|r| (r.subscription_id, r.bucket_start))
            .collect();
        let now = Utc::now();

        let bucket_starts: Vec<DateTime<Utc>> = request_attempts_aggregates
            .iter()
            // No open bucket → start new one at current time.
            .map(|a| open_buckets.get(&a.subscription_id).copied().unwrap_or(now))
            .collect();
        // Saturating cast; values beyond i32 capped, not truncated.
        let total_counts: Vec<i32> = request_attempts_aggregates
            .iter()
            .map(|a| i32::try_from(a.total).unwrap_or(i32::MAX))
            .collect();
        let failed_counts: Vec<i32> = request_attempts_aggregates
            .iter()
            .map(|a| i32::try_from(a.failed).unwrap_or(i32::MAX))
            .collect();

        // Upsert: new buckets created, existing accumulate counts.
        query!(
            r#"
                insert into webhook.subscription_health_bucket (subscription__id, bucket_start, total_count, failed_count)
                select * from unnest($1::uuid[], $2::timestamptz[], $3::int4[], $4::int4[])
                on conflict (subscription__id, bucket_start) do update set
                    total_count = subscription_health_bucket.total_count + excluded.total_count,
                    failed_count = subscription_health_bucket.failed_count + excluded.failed_count
            "#,
            &subscriptions_with_new_attempts,
            &bucket_starts,
            &total_counts,
            &failed_counts,
        )
        .execute(&mut **tx)
        .await?;
    }

    // Close buckets exceeding age or attempt cap.
    // Closed buckets become immutable for evaluation.
    query!(
        r#"
            update webhook.subscription_health_bucket
            set bucket_end = now()
            where bucket_end is null
              and (bucket_start < now() - make_interval(secs => $1)
                   or total_count >= $2)
        "#,
        config.bucket_max_duration.as_secs_f64(),
        i32::try_from(config.bucket_max_request_attempts).unwrap_or(i32::MAX),
    )
    .execute(&mut **tx)
    .await?;

    // bucket_stats: subscription failure rate over sliding window.
    // candidates: subscriptions above minimum failures OR currently in warning.
    // Warning-state subscriptions included for possible resolve transition.
    let candidates_subscriptions = query_as!(
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
        config.failure_rate_window.as_secs_f64(),
        i64::from(config.min_request_attempts),
    )
    .fetch_all(&mut **tx)
    .await?;

    // Reset failure_percent on subscriptions no longer evaluation candidates.
    // Prevents stale percentage display in API responses.
    let candidate_ids: Vec<Uuid> = candidates_subscriptions
        .iter()
        .map(|s| s.subscription_id)
        .collect();
    query!(
        r#"
            update webhook.subscription
            set failure_percent = null
            where failure_percent is not null
              and subscription__id <> all($1)
        "#,
        &candidate_ids,
    )
    .execute(&mut **tx)
    .await?;

    // Forward-only cursor. Guards against rewind double-counting.
    if let Some(cursor_timestamp) = max_completed_at {
        query!(
            r#"
                update webhook.subscription_health_monitor_cursor
                set last_processed_at = $1
                where cursor__id = 1
                  and $1 > last_processed_at
            "#,
            cursor_timestamp,
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(candidates_subscriptions)
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

        let (_org_id, _app_id, sub_id, _secret_token) =
            insert_test_fixtures(&mut tx, 3, 2, now).await;

        let candidates_subscriptions =
            list_health_evaluation_subscriptions_candidates(&mut tx, &config)
                .await
                .unwrap();

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

        let candidate_ids: Vec<Uuid> = candidates_subscriptions
            .iter()
            .map(|s| s.subscription_id)
            .collect();
        assert!(candidate_ids.contains(&sub_id));

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

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, app_id, sub_id, secret_token) =
            insert_test_fixtures(&mut tx, 2, 5, now).await;
        tx.commit().await.unwrap();

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

        query!(
            "delete from webhook.subscription_health_bucket where subscription__id = $1",
            sub_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
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

        query!(
            "delete from webhook.subscription_health_bucket where subscription__id = $1",
            sub_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        insert_attempts(&mut tx, app_id, sub_id, secret_token, 2, 5, Utc::now()).await;
        tx.commit().await.unwrap();

        run_one_tick(&pool, &config).await.unwrap();

        let warning_count = sqlx::query_scalar!(
            "select count(*) from webhook.subscription_health_event where subscription__id = $1 and status = 'warning'",
            sub_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            warning_count,
            Some(1),
            "Anti-flap: no new warning during cooldown"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_e2e_disable_lifecycle(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, app_id, sub_id, secret_token) =
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
        assert!(
            !is_enabled,
            "Subscription should be actually disabled in DB"
        );

        let mut tx = pool.begin().await.unwrap();
        insert_attempts(&mut tx, app_id, sub_id, secret_token, 0, 5, Utc::now()).await;
        tx.commit().await.unwrap();

        run_one_tick(&pool, &config).await.unwrap();

        let disabled_count = sqlx::query_scalar!(
            "select count(*) from webhook.subscription_health_event where subscription__id = $1 and status = 'disabled'",
            sub_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            disabled_count,
            Some(1),
            "Disabled subscription must not emit more events"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_e2e_warning_to_disable_escalation(pool: PgPool) {
        let config = test_config();
        let now = Utc::now();
        let cursor_past = now - chrono::Duration::hours(2);

        let mut tx = pool.begin().await.unwrap();
        set_cursor(&mut tx, cursor_past).await;
        let (_org_id, app_id, sub_id, secret_token) =
            insert_test_fixtures(&mut tx, 2, 5, now).await;
        tx.commit().await.unwrap();

        run_one_tick(&pool, &config).await.unwrap();

        let warning_count = sqlx::query_scalar!(
            "select count(*) from webhook.subscription_health_event where subscription__id = $1 and status = 'warning'",
            sub_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(warning_count, Some(1), "Should enter warning first");

        query!(
            "delete from webhook.subscription_health_bucket where subscription__id = $1",
            sub_id
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        insert_attempts(&mut tx, app_id, sub_id, secret_token, 0, 10, Utc::now()).await;
        tx.commit().await.unwrap();

        run_one_tick(&pool, &config).await.unwrap();

        let disabled_count = sqlx::query_scalar!(
            "select count(*) from webhook.subscription_health_event where subscription__id = $1 and status = 'disabled'",
            sub_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            disabled_count,
            Some(1),
            "Should escalate from warning to disabled"
        );

        let is_enabled = sqlx::query_scalar!(
            "select is_enabled from webhook.subscription where subscription__id = $1",
            sub_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!is_enabled, "Subscription should be disabled in DB");
    }

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
