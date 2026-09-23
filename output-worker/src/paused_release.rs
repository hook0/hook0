//! Release of paused request attempts.
//!
//! When subscription health is enforced, the request attempts of degraded or recovering
//! subscriptions are created paused (see `webhook.effective_subscription_health` and the
//! `event.dispatch` trigger). Every period, this task unpauses some of the paused request attempts
//! of this worker's subscriptions, so that each of them gets at most a target number of request
//! attempts per minute (processed + waiting), the target depending on the subscription's health.

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use futures::{StreamExt, stream};
use pulsar::{Producer, ProducerOptions, TokioExecutor};
use rand::seq::SliceRandom;
use sqlx::postgres::types::PgInterval;
use sqlx::{Connection, PgConnection, PgPool, query, query_as, query_scalar};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use strum::{EnumString, IntoStaticStr};
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::{Instant, MissedTickBehavior, interval};
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::opentelemetry::{
    report_paused_request_attempts_released, report_subscriptions_with_paused_request_attempts,
};
use crate::pulsar::{await_receipt, enqueue};
use crate::{
    Config, MissingPayload, ObjectStorageConfig, PulsarConfig, RequestAttempt,
    RequestAttemptWithOptionalPayload, Worker, fetch_event_payload, give_up_on_missing_payload,
};

/// Number of paused request attempts a Pulsar worker releases at once
const PULSAR_CHUNK_SIZE: i64 = 100;

/// Added to the HTTP timeout to get how long request attempts paused again after a Pulsar failure are
/// held before being released again (see `release_with_pulsar`)
const REPAUSE_HOLD_MARGIN: Duration = Duration::from_secs(60);

/// Number of payloads a Pulsar worker fetches concurrently from object storage
const PAYLOAD_FETCH_CONCURRENCY: usize = 16;

/// Health of a subscription as it must affect deliveries (as returned by `webhook.effective_subscription_health`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum Health {
    Healthy,
    Degraded,
    Recovering,
}

impl Health {
    fn target_per_minute(self, config: &Config) -> u32 {
        match self {
            Self::Healthy => config.paused_request_attempts_release_target_healthy,
            Self::Degraded => config.paused_request_attempts_release_target_degraded,
            Self::Recovering => config.paused_request_attempts_release_target_recovering,
        }
    }
}

/// Number of paused request attempts to release so that the request attempts processed during the
/// window plus the ones waiting to be processed reach the per-minute target scaled to the window.
fn release_quota(target_per_minute: u32, window: Duration, processed: i64, waiting: i64) -> i64 {
    let target = u128::from(target_per_minute) * window.as_millis() / 60_000;
    i64::try_from(target)
        .unwrap_or(i64::MAX)
        .saturating_sub(processed.saturating_add(waiting))
        .max(0)
}

pub async fn run_paused_release(
    config: &Config,
    pool: &PgPool,
    worker: &Worker,
    object_storage: &Option<ObjectStorageConfig>,
    pulsar: &Option<Arc<PulsarConfig>>,
    task_tracker: &TaskTracker,
) {
    let mut ticker = interval(config.paused_request_attempts_release_period);
    // A pass can last up to a whole period; the next one must not start right after it
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker.tick().await; // skip the first immediate tick

    loop {
        select! {
            biased;
            _ = task_tracker.wait() => break,
            _ = ticker.tick() => {
                if !task_tracker.is_closed()
                    && let Err(e) = release_pass(config, pool, worker, object_storage, pulsar, task_tracker).await
                {
                    error!("Could not release paused request attempts: {e}");
                }
            }
        }
    }

    debug!("Paused request attempts release task terminated");
}

async fn release_pass(
    config: &Config,
    pool: &PgPool,
    worker: &Worker,
    object_storage: &Option<ObjectStorageConfig>,
    pulsar: &Option<Arc<PulsarConfig>>,
    task_tracker: &TaskTracker,
) -> anyhow::Result<()> {
    // Unassigned subscriptions are shared by all public PG workers, whereas Pulsar routing always
    // needs an explicit assignment
    let is_public = pulsar.is_none() && worker.scope.is_public();
    let worker_id = worker.scope.worker_id();

    // Only one process at a time releases the paused request attempts of a set of subscriptions:
    // otherwise, processes interleaving their counts and their releases would each release a
    // whole quota.
    // The lock belongs to a transaction of its own, while the releases below are committed as they go
    // on another connection: the lock is released when this transaction ends, whatever happens (a
    // dropped transaction is rolled back when its connection returns to the pool)
    let lock_key = match (is_public, worker_id) {
        (false, Some(id)) => id.to_string(),
        _ => "public".to_owned(),
    };
    let mut lock_tx = pool.begin().await?;
    // This transaction stays idle during the whole pass: it must not be killed, losing the lock
    query!("SET LOCAL idle_in_transaction_session_timeout = 0")
        .execute(&mut *lock_tx)
        .await?;
    let locked = query_scalar!(
        r#"SELECT pg_try_advisory_xact_lock(hashtext('hook0.paused_request_attempts_release'), hashtext($1)) AS "locked!""#,
        lock_key,
    )
    .fetch_one(&mut *lock_tx)
    .await?;

    if locked {
        // Housekeeping pool, so that a main pool saturated by busy work units cannot starve this task
        let mut conn = pool.acquire().await?;

        struct PausedSubscription {
            subscription_id: Uuid,
            health: String,
        }
        // Performance-critical query shape; do NOT simplify into a `SELECT DISTINCT`, which reads one
        // index entry per paused request attempt (a degraded subscription can pile up millions of them).
        // This recursive CTE is a loose index scan over `request_attempt_paused_idx`: each step jumps to
        // the next subscription with a single index descent, so the cost grows with the number of
        // subscriptions instead of the number of paused request attempts.
        // Both branches repeat the index predicate exactly, and order by `(subscription__id, created_at)`,
        // which only this index can provide without sorting: ordering by `subscription__id` alone lets the
        // generic plan pick `request_attempt_subscription__id_idx` when paused request attempts are most of
        // the table, and check them one by one against the heap.
        // The recursion ends with a NULL row, which the inner join on `webhook.subscription` drops.
        let subscriptions = query_as!(
            PausedSubscription,
            r#"
                WITH RECURSIVE paused AS (
                    (
                        SELECT subscription__id
                        FROM webhook.request_attempt
                        WHERE paused AND succeeded_at IS NULL AND failed_at IS NULL
                        ORDER BY subscription__id, created_at
                        LIMIT 1
                    )
                    UNION ALL
                    SELECT (
                        SELECT ra.subscription__id
                        FROM webhook.request_attempt AS ra
                        WHERE ra.paused AND ra.succeeded_at IS NULL AND ra.failed_at IS NULL
                            AND ra.subscription__id > p.subscription__id
                        ORDER BY ra.subscription__id, ra.created_at
                        LIMIT 1
                    )
                    FROM paused AS p
                    WHERE p.subscription__id IS NOT NULL
                )
                SELECT
                    s.subscription__id AS "subscription_id!",
                    webhook.effective_subscription_health(s.subscription__id, s.application__id) AS "health!"
                FROM paused AS p
                INNER JOIN webhook.subscription AS s ON s.subscription__id = p.subscription__id AND s.is_enabled AND s.deleted_at IS NULL
                INNER JOIN event.application AS a ON a.application__id = s.application__id AND a.deleted_at IS NULL
                LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = a.organization__id AND ow.default = true
                WHERE ($2 AND COALESCE(sw.worker__id, ow.worker__id) IS NULL)
                    OR COALESCE(sw.worker__id, ow.worker__id) = $1
            "#,
            worker_id,
            is_public,
        )
        .fetch_all(&mut *conn)
        .await?;
        report_subscriptions_with_paused_request_attempts(subscriptions.len() as u64);

        if !subscriptions.is_empty() {
            let window = config.paused_request_attempts_release_window;
            let window_interval = PgInterval::try_from(window).map_err(|e| {
                anyhow!("Could not convert release window ({window:?}) to a PG interval: {e}")
            })?;
            let subscription_ids: Vec<Uuid> =
                subscriptions.iter().map(|s| s.subscription_id).collect();

            // Processed: request attempts completed during the window, found through their
            // response, whose UUIDv7 primary key is time-ordered (bounded on both sides so that
            // older UUIDv4 keys are left out)
            // Waiting: request attempts that are neither paused nor completed, and due
            struct Counts {
                subscription_id: Uuid,
                processed: i64,
                waiting: i64,
            }
            let counts: HashMap<Uuid, (i64, i64)> = query_as!(
                Counts,
                r#"
                    WITH processed AS (
                        SELECT ra.subscription__id, count(*) AS n
                        FROM webhook.response AS r
                        INNER JOIN webhook.request_attempt AS ra ON ra.response__id = r.response__id
                        WHERE r.response__id BETWEEN (SELECT uuidv7(-$2::interval)) AND (SELECT uuidv7())
                            AND ra.subscription__id = ANY($1)
                            AND COALESCE(ra.succeeded_at, ra.failed_at) >= statement_timestamp() - $2::interval
                        GROUP BY ra.subscription__id
                    ), waiting AS (
                        SELECT subscription__id, count(*) AS n
                        FROM webhook.request_attempt
                        WHERE subscription__id = ANY($1)
                            AND NOT paused AND succeeded_at IS NULL AND failed_at IS NULL
                            AND (delay_until IS NULL OR delay_until <= statement_timestamp())
                        GROUP BY subscription__id
                    )
                    SELECT
                        s.subscription__id AS "subscription_id!",
                        COALESCE(p.n, 0) AS "processed!",
                        COALESCE(w.n, 0) AS "waiting!"
                    FROM unnest($1::uuid[]) AS s(subscription__id)
                    LEFT JOIN processed AS p ON p.subscription__id = s.subscription__id
                    LEFT JOIN waiting AS w ON w.subscription__id = s.subscription__id
                "#,
                &subscription_ids,
                window_interval,
            )
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|c| (c.subscription_id, (c.processed, c.waiting)))
            .collect();

            let mut to_release: Vec<(Uuid, Health, i64)> = subscriptions
                .into_iter()
                .filter_map(|s| {
                    let health = Health::from_str(&s.health).unwrap_or(Health::Healthy);
                    let (processed, waiting) =
                        counts.get(&s.subscription_id).copied().unwrap_or((0, 0));
                    let quota =
                        release_quota(health.target_per_minute(config), window, processed, waiting);
                    (quota > 0).then_some((s.subscription_id, health, quota))
                })
                .collect();
            // If the time budget runs out, the subscriptions that were not reached wait for the
            // next pass: shuffling avoids starving the same ones every time
            to_release.shuffle(&mut rand::rng());

            // Built only when there is something to send, and once per pass so that a broken
            // producer is replaced by the next pass
            let producer = match (pulsar, worker_id, to_release.is_empty()) {
                (Some(pulsar), Some(worker_id), false) => Some(Mutex::new(
                    pulsar
                        .pulsar
                        .producer()
                        // Paused request attempts are first attempts (see `event.dispatch`), which
                        // the API always sends to the high priority topic
                        .with_topic(format!(
                            "persistent://{}/{}/{}.request_attempt",
                            pulsar.tenant, pulsar.namespace, worker_id,
                        ))
                        .with_name(format!(
                            "hook0-output-worker.{worker_id}.paused-request-attempts-release.{}",
                            Uuid::now_v7()
                        ))
                        .with_options(ProducerOptions {
                            block_queue_if_full: true,
                            ..Default::default()
                        })
                        .build()
                        .await?,
                )),
                _ => None,
            };

            // The time budget is only checked between units of work, so that no request attempt
            // is left unpaused without having been sent to Pulsar (unless the process crashes)
            let deadline = Instant::now() + config.paused_request_attempts_release_period;
            let mut released_total = 0u64;
            let mut released_subscriptions = 0u64;
            for (subscription_id, health, quota) in to_release {
                if Instant::now() < deadline && !task_tracker.is_closed() {
                    let released = if let Some(producer) = &producer {
                        release_with_pulsar(
                            &mut conn,
                            config,
                            object_storage,
                            producer,
                            subscription_id,
                            quota,
                            deadline,
                            task_tracker,
                        )
                        .await?
                    } else {
                        release_with_pg(&mut conn, subscription_id, quota).await?
                    };
                    if released > 0 {
                        report_paused_request_attempts_released(health.into(), released);
                        released_total += released;
                        released_subscriptions += 1;
                    }
                }
            }

            if released_total > 0 {
                info!(
                    released = released_total,
                    subscriptions = released_subscriptions,
                    "Released paused request attempts"
                );
            }
        }
    } else {
        debug!(
            "Another process is already releasing the paused request attempts of this worker; skipping this pass"
        );
    }

    // The releases are already committed; a failure here only means that this connection is broken,
    // which releases the lock too
    if let Err(e) = lock_tx.rollback().await {
        warn!(
            "Could not end the transaction holding the paused request attempts release lock: {e}"
        );
    }

    Ok(())
}

async fn release_with_pg(
    conn: &mut PgConnection,
    subscription_id: Uuid,
    quota: i64,
) -> Result<u64, sqlx::Error> {
    // `FOR UPDATE` checks the conditions again on the latest version of each row once it is locked,
    // so a request attempt that was released or completed in the meantime is not counted twice.
    // Request attempts held after a Pulsar failure (see `release_with_pulsar`) are skipped too.
    // Request attempts locked by the API's cancellation statements (subscription disabled or
    // deleted, application deleted) are skipped instead of waited for, which also rules out
    // deadlocks with them; the ones that are not cancelled in the end are released by a next pass.
    Ok(query!(
        "
            UPDATE webhook.request_attempt
            SET paused = false, delay_until = statement_timestamp()
            WHERE request_attempt__id IN (
                SELECT request_attempt__id
                FROM webhook.request_attempt
                WHERE subscription__id = $1 AND paused AND succeeded_at IS NULL AND failed_at IS NULL
                    AND (delay_until IS NULL OR delay_until <= statement_timestamp())
                ORDER BY created_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
        ",
        subscription_id,
        quota,
    )
    .execute(conn)
    .await?
    .rows_affected())
}

#[allow(clippy::too_many_arguments)]
async fn release_with_pulsar(
    conn: &mut PgConnection,
    config: &Config,
    object_storage: &Option<ObjectStorageConfig>,
    producer: &Mutex<Producer<TokioExecutor>>,
    subscription_id: Uuid,
    quota: i64,
    deadline: Instant,
    task_tracker: &TaskTracker,
) -> anyhow::Result<u64> {
    let mut released = 0i64;
    // Keyset cursor over `(created_at, request_attempt__id)`, so that request attempts left paused
    // (payload unavailable) are not selected again during this pass
    let mut cursor = (DateTime::<Utc>::UNIX_EPOCH, Uuid::nil());
    let mut running = true;

    while running && released < quota && Instant::now() < deadline && !task_tracker.is_closed() {
        let rows = query_as!(
            RequestAttemptWithOptionalPayload,
            "
                SELECT
                    e.application__id AS application_id,
                    ra.request_attempt__id AS request_attempt_id,
                    ra.event__id AS event_id,
                    e.received_at AS event_received_at,
                    ra.subscription__id AS subscription_id,
                    ra.created_at,
                    ra.retry_count,
                    ra.delay_until,
                    t_http.method as http_method,
                    t_http.url as http_url,
                    t_http.headers as http_headers,
                    e.event_type__name AS event_type_name,
                    e.payload,
                    e.payload_content_type,
                    s.secret
                FROM webhook.request_attempt AS ra
                INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                INNER JOIN event.event AS e ON e.event__id = ra.event__id
                WHERE ra.subscription__id = $1
                    AND ra.paused AND ra.succeeded_at IS NULL AND ra.failed_at IS NULL
                    AND (ra.delay_until IS NULL OR ra.delay_until <= statement_timestamp())
                    AND (ra.created_at, ra.request_attempt__id) > ($2::timestamptz, $3::uuid)
                ORDER BY ra.created_at ASC, ra.request_attempt__id ASC
                LIMIT $4::bigint
            ",
            subscription_id,
            cursor.0,
            cursor.1,
            (quota - released).min(PULSAR_CHUNK_SIZE),
        )
        .fetch_all(&mut *conn)
        .await?;

        if let Some(last) = rows.last() {
            cursor = (last.created_at, last.request_attempt_id);

            // Payloads are fetched while the request attempts are still paused
            let fetched: Vec<_> = stream::iter(rows)
                .map(|mut ra| async move {
                    let payload = match ra.payload.take() {
                        Some(p) => Ok(p),
                        None => {
                            fetch_event_payload(
                                object_storage,
                                ra.application_id,
                                ra.event_received_at,
                                ra.event_id,
                            )
                            .await
                        }
                    };
                    (ra, payload)
                })
                .buffered(PAYLOAD_FETCH_CONCURRENCY)
                .collect()
                .await;

            let mut ready = Vec::with_capacity(fetched.len());
            for (ra, payload) in fetched {
                match payload {
                    Ok(p) => ready.push((ra, p)),
                    Err(MissingPayload::Gone) => {
                        warn!(
                            event_id = %ra.event_id,
                            request_attempt_id = %ra.request_attempt_id,
                            "Payload object is missing from object storage; giving up on this paused request attempt"
                        );
                        let mut tx = conn.begin().await?;
                        give_up_on_missing_payload(&mut tx, ra.request_attempt_id).await?;
                        tx.commit().await?;
                    }
                    Err(MissingPayload::Unavailable) => {
                        warn!(
                            event_id = %ra.event_id,
                            request_attempt_id = %ra.request_attempt_id,
                            "Could not get payload for event; leaving the request attempt paused"
                        );
                    }
                }
            }

            // Request attempts locked by the API's cancellations are skipped: see `release_with_pg`.
            // Accepted, rare: if this unpauses a request attempt that was just created, before the
            // API reads it back to send it to Pulsar after committing the event, the API sends it
            // too; delivery is at-least-once anyway.
            let ready_ids: Vec<Uuid> = ready.iter().map(|(ra, _)| ra.request_attempt_id).collect();
            let unpaused: HashSet<Uuid> = query_scalar!(
                "
                    UPDATE webhook.request_attempt
                    SET paused = false, delay_until = statement_timestamp()
                    WHERE request_attempt__id IN (
                        SELECT request_attempt__id
                        FROM webhook.request_attempt
                        WHERE request_attempt__id = ANY($1)
                            AND paused AND succeeded_at IS NULL AND failed_at IS NULL
                        FOR UPDATE SKIP LOCKED
                    )
                    RETURNING request_attempt__id
                ",
                &ready_ids,
            )
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .collect();

            // From here on, every unpaused request attempt must either be sent to Pulsar or be paused again
            let mut sending = Vec::with_capacity(unpaused.len());
            let mut failed = Vec::new();
            for (ra, payload) in ready
                .into_iter()
                .filter(|(ra, _)| unpaused.contains(&ra.request_attempt_id))
            {
                let request_attempt_id = ra.request_attempt_id;
                let created_at = ra.created_at;
                let request_attempt = RequestAttempt {
                    application_id: ra.application_id,
                    request_attempt_id: ra.request_attempt_id,
                    event_id: ra.event_id,
                    event_received_at: ra.event_received_at,
                    subscription_id: ra.subscription_id,
                    created_at: ra.created_at,
                    retry_count: ra.retry_count,
                    http_method: ra.http_method,
                    http_url: ra.http_url,
                    http_headers: ra.http_headers,
                    event_type_name: ra.event_type_name,
                    payload,
                    payload_content_type: ra.payload_content_type,
                    secret: ra.secret,
                };
                match enqueue(
                    producer,
                    request_attempt,
                    created_at,
                    None,
                    config.pulsar_send_receipt_timeout,
                )
                .await
                {
                    Ok(send_future) => sending.push((request_attempt_id, send_future)),
                    Err(e) => {
                        error!(%request_attempt_id, "Could not enqueue a released request attempt into Pulsar: {e}");
                        failed.push(request_attempt_id);
                    }
                }
            }
            let receipts = join_all(sending.into_iter().map(
                |(request_attempt_id, send_future)| async move {
                    (
                        request_attempt_id,
                        await_receipt(
                            send_future,
                            config.pulsar_send_receipt_timeout,
                            request_attempt_id,
                        )
                        .await,
                    )
                },
            ))
            .await;
            failed.extend(
                receipts
                    .into_iter()
                    .filter(|(_, receipt)| receipt.is_err())
                    .map(|(request_attempt_id, _)| request_attempt_id),
            );

            released += (unpaused.len() - failed.len()) as i64;
            if !failed.is_empty() {
                // A message whose receipt failed may still have reached a consumer, which may be
                // delivering it right now (Pulsar workers only write `picked_at` when they complete
                // a request attempt, so this cannot be checked). Held for longer than a delivery can
                // take, these request attempts cannot be released and sent a second time while that
                // delivery is still going on.
                let hold = config.timeout + REPAUSE_HOLD_MARGIN;
                // Accepted, rare: unlike the unpause statements, this one must not skip any row, so
                // it waits for the API's cancellations and can deadlock with them (both lock several
                // request attempts of a subscription in no particular order). If PostgreSQL aborts
                // this one, the API goes on to cancel these request attempts anyway; if it aborts the
                // API's, that request fails with a 500 and can be retried.
                query!(
                    "
                        UPDATE webhook.request_attempt
                        SET paused = true, delay_until = statement_timestamp() + make_interval(secs => $2)
                        WHERE request_attempt__id = ANY($1)
                            AND succeeded_at IS NULL AND failed_at IS NULL
                    ",
                    &failed,
                    hold.as_secs_f64(),
                )
                .execute(&mut *conn)
                .await?;
                warn!(
                    count = failed.len(),
                    "Could not send some released request attempts to Pulsar; they were paused again"
                );
                // Pulsar is probably unavailable; a later pass will try again
                running = false;
            }
        } else {
            running = false;
        }
    }

    Ok(released as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: Duration = Duration::from_secs(60);

    #[test]
    fn release_quota_fills_up_to_the_target() {
        assert_eq!(release_quota(50, MINUTE, 0, 0), 50);
        assert_eq!(release_quota(50, MINUTE, 20, 10), 20);
    }

    #[test]
    fn release_quota_never_goes_negative() {
        assert_eq!(release_quota(50, MINUTE, 40, 40), 0);
        assert_eq!(release_quota(50, MINUTE, i64::MAX, i64::MAX), 0);
    }

    #[test]
    fn release_quota_is_scaled_to_the_window() {
        assert_eq!(release_quota(50, Duration::from_secs(30), 0, 0), 25);
        assert_eq!(release_quota(500, Duration::from_secs(120), 100, 0), 900);
        assert_eq!(release_quota(5000, Duration::from_secs(1), 0, 0), 83);
    }

    #[test]
    fn a_zero_target_releases_nothing() {
        assert_eq!(release_quota(0, MINUTE, 0, 0), 0);
    }

    #[test]
    fn health_is_parsed_from_the_database_values() {
        assert_eq!(Health::from_str("healthy"), Ok(Health::Healthy));
        assert_eq!(Health::from_str("degraded"), Ok(Health::Degraded));
        assert_eq!(Health::from_str("recovering"), Ok(Health::Recovering));
        assert_eq!(<&'static str>::from(Health::Recovering), "recovering");
    }
}
