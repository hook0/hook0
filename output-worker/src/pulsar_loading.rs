use anyhow::anyhow;
use aws_sdk_s3::error::DisplayErrorContext;
use chrono::Utc;
use futures::TryStreamExt;
use futures::future::try_join_all;
use pulsar::consumer::InitialPosition;
use pulsar::proto::MessageIdData;
use pulsar::reader::Reader;
use pulsar::{ConsumerOptions, ProducerOptions, SubType, TokioExecutor};
use sqlx::{PgPool, query_as};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::select;
use tokio::time::{MissedTickBehavior, interval, timeout};
use tokio_util::task::TaskTracker;
use tracing::{error, info, trace, warn};
use uuid::Uuid;

use crate::RequestAttemptWithOptionalPayload;
use crate::pulsar::DELAY_TOLERANCE;
use crate::{ObjectStorageConfig, PulsarConfig, RequestAttempt, SlotRole};
use hook0_sentry_integration::log_object_storage_error_with_context;

#[allow(clippy::too_many_arguments)]
pub async fn load_waiting_request_attempts_from_db(
    pool: &PgPool,
    worker_id: &Arc<Uuid>,
    pulsar: &Arc<PulsarConfig>,
    object_storage: &Option<ObjectStorageConfig>,
    hp_retry_cutoff: i16,
    pulsar_send_receipt_timeout: Duration,
    ignore_ids: &HashSet<Uuid>,
    hp_only: bool,
    task_tracker: &TaskTracker,
) -> anyhow::Result<u64> {
    let hp_topic = format!(
        "persistent://{}/{}/{}.request_attempt",
        &pulsar.tenant, &pulsar.namespace, worker_id,
    );
    let mut hp_producer = pulsar
        .pulsar
        .producer()
        .with_topic(&hp_topic)
        .with_name(format!(
            "hook0-output-worker.{worker_id}.request-attempts-loading.hp.{}",
            Uuid::now_v7()
        ))
        .with_options(ProducerOptions {
            block_queue_if_full: true,
            ..Default::default()
        })
        .build()
        .await?;

    let mut lp_producer = if hp_only {
        None
    } else {
        let lp_topic = format!(
            "persistent://{}/{}/{}.request_attempt.lp",
            &pulsar.tenant, &pulsar.namespace, worker_id,
        );
        Some(
            pulsar
                .pulsar
                .producer()
                .with_topic(&lp_topic)
                .with_name(format!(
                    "hook0-output-worker.{worker_id}.request-attempts-loading.lp.{}",
                    Uuid::now_v7()
                ))
                .with_options(ProducerOptions {
                    block_queue_if_full: true,
                    ..Default::default()
                })
                .build()
                .await?,
        )
    };

    let mut request_attempts_stream = query_as!(
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
            LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = ra.subscription__id
            INNER JOIN event.application AS a ON a.application__id = s.application__id
            LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = a.organization__id AND ow.default = true
            WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL
                AND a.deleted_at IS NULL
                AND COALESCE(sw.worker__id, ow.worker__id) = $1
                AND ($2 = false OR ra.retry_count < $3)
        ",
        worker_id.as_ref(),
        hp_only,
        hp_retry_cutoff,
    )
    .fetch(pool);

    let mut counter = 0u64;
    let mut running = true;
    let mut receipt_futures = Vec::new();
    while running {
        select! {
            biased;
            _ = task_tracker.wait() => {
                running = false;
            }
            row = request_attempts_stream.try_next() => {
                match row? {
                    None => {
                        running = false;
                    }
                    Some(ra) => {
                        let kept = !ignore_ids.contains(&ra.request_attempt_id);
                        if kept {
                            let payload = if let Some(p) = ra.payload {
                                Some(p)
                            } else if let Some(os) = &object_storage {
                                let key = format!(
                                    "{}/event/{}/{}",
                                    ra.application_id,
                                    ra.event_received_at.naive_utc().date(),
                                    ra.event_id
                                );
                                match os
                                    .client
                                    .get_object()
                                    .bucket(&os.bucket)
                                    .key(&key)
                                    .send()
                                    .await
                                {
                                    Ok(obj) => match obj.body.collect().await {
                                        Ok(ab) => Some(ab.to_vec()),
                                        Err(e) => {
                                            log_object_storage_error_with_context!(
                                                "S3 GET OBJECT body collect failed",
                                                error_chain = format!("{e}"),
                                                object_key = &key,
                                            );
                                            None
                                        }
                                    },
                                    Err(e) => {
                                        log_object_storage_error_with_context!(
                                            "S3 GET OBJECT failed",
                                            error_chain = DisplayErrorContext(&e).to_string(),
                                            object_key = &key,
                                        );
                                        None
                                    }
                                }
                            } else {
                                None
                            };

                            if let Some(p) = payload {
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
                                    payload: p,
                                    payload_content_type: ra.payload_content_type,
                                    secret: ra.secret,
                                };
                                let producer = if let Some(lpp) = lp_producer.as_mut() && !SlotRole::is_hp(ra.retry_count, hp_retry_cutoff) {
                                    lpp
                                } else {
                                    &mut hp_producer
                                };
                                let mut msg_builder = producer
                                    .create_message()
                                    .event_time(request_attempt.created_at.timestamp_micros() as u64);
                                if let Some(delay_until) = ra.delay_until
                                    && delay_until > (Utc::now() + DELAY_TOLERANCE)
                                {
                                    msg_builder = msg_builder.deliver_at(delay_until.into())?;
                                }

                                let request_attempt_id = ra.request_attempt_id;
                                let send_future = msg_builder
                                    .with_content(request_attempt)
                                    .send_non_blocking()
                                    .await?;

                                receipt_futures.push(async move {
                                    timeout(pulsar_send_receipt_timeout, send_future)
                                        .await
                                        .map_err(|_| {
                                            error!(%request_attempt_id, "Pulsar broker receipt timed out");
                                            anyhow!("Pulsar broker receipt timed out")
                                        })?
                                        .map_err(|e| {
                                            error!(%request_attempt_id, error = %e, "Pulsar broker rejected message");
                                            anyhow::Error::from(e)
                                        })?;
                                    Ok::<(), anyhow::Error>(())
                                });

                                counter += 1;
                            } else {
                                warn!(event_id = %ra.event_id, "Could not get event's payload");
                            }
                        }
                    }
                }
            }
        }
    }

    if !receipt_futures.is_empty() {
        let start = Instant::now();
        let amount = receipt_futures.len();
        try_join_all(receipt_futures).await?;
        trace!(
            amount,
            elapsed_ms = start.elapsed().as_millis(),
            "Got all receipts from Pulsar"
        );
    }

    Ok(counter)
}

/// Periodically scans the HP topic and re-publishes any waiting `request_attempt`
/// rows that are missing from it. Only HP-class attempts are republished; LP
/// attempts are not refreshed by this task.
#[allow(clippy::too_many_arguments)]
pub async fn run_periodic_request_attempts_loading(
    pool: &PgPool,
    worker_id: &Arc<Uuid>,
    pulsar: &Arc<PulsarConfig>,
    object_storage: &Option<ObjectStorageConfig>,
    hp_retry_cutoff: i16,
    pulsar_send_receipt_timeout: Duration,
    interval_duration: Duration,
    task_tracker: &TaskTracker,
) {
    let mut ticker = interval(interval_duration);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    // Consume the immediate first tick so the first real run happens after one
    // full interval (no overlap with the startup one-shot).
    ticker.tick().await;

    let mut running = true;
    while running {
        select! {
            biased;
            _ = task_tracker.wait() => {
                running = false;
            }
            _ = ticker.tick() => {
                match run_one_pass(
                    pool,
                    worker_id,
                    pulsar,
                    object_storage,
                    hp_retry_cutoff,
                    pulsar_send_receipt_timeout,
                    task_tracker,
                )
                .await
                {
                    Ok(()) => {}
                    Err(e) => error!("Periodic request_attempts reload failed: {e}"),
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_one_pass(
    pool: &PgPool,
    worker_id: &Arc<Uuid>,
    pulsar: &Arc<PulsarConfig>,
    object_storage: &Option<ObjectStorageConfig>,
    hp_retry_cutoff: i16,
    pulsar_send_receipt_timeout: Duration,
    task_tracker: &TaskTracker,
) -> anyhow::Result<()> {
    let hp_topic = format!(
        "persistent://{}/{}/{}.request_attempt",
        &pulsar.tenant, &pulsar.namespace, worker_id,
    );

    let reader_start = Instant::now();

    let reader_build = pulsar
        .pulsar
        .reader()
        .with_topic(&hp_topic)
        .with_consumer_name(format!(
            "hook0-output-worker.{worker_id}.periodic-reload.hp.{}",
            Uuid::now_v7()
        ))
        .with_subscription(format!(
            "hook0-output-worker.{worker_id}.periodic-reload.{}",
            Uuid::now_v7()
        ))
        .with_subscription_type(SubType::Exclusive)
        .with_options(ConsumerOptions {
            initial_position: InitialPosition::Earliest,
            ..Default::default()
        })
        .into_reader::<RequestAttempt>();

    let reader_opt: Option<Reader<RequestAttempt, TokioExecutor>> = select! {
        biased;
        _ = task_tracker.wait() => None,
        r = reader_build => Some(r?),
    };

    if let Some(mut reader) = reader_opt {
        let last_msg_id_opt: Option<MessageIdData> = select! {
            biased;
            _ = task_tracker.wait() => None,
            r = reader.get_last_message_id() => Some(r?),
        };

        if let Some(last_msg_id) = last_msg_id_opt {
            let topic_is_empty = is_empty_sentinel(&last_msg_id);
            let mut ignore: HashSet<Uuid> = HashSet::new();
            let mut scanning = !topic_is_empty;

            while scanning {
                select! {
                    biased;
                    _ = task_tracker.wait() => {
                        scanning = false;
                    }
                    msg = reader.try_next() => {
                        match msg? {
                            None => {
                                scanning = false;
                            }
                            Some(m) => {
                                let reached_head = reached_or_past_head(&m.message_id.id, &last_msg_id);
                                match m.deserialize() {
                                    Ok(ra) => {
                                        ignore.insert(ra.request_attempt_id);
                                    }
                                    Err(e) => {
                                        warn!("Reader: failed to deserialize request attempt: {e}");
                                    }
                                }
                                if reached_head {
                                    scanning = false;
                                }
                            }
                        }
                    }
                }
            }
            drop(reader);

            let reader_elapsed = reader_start.elapsed();

            let mut loaded = 0u64;
            let mut db_elapsed = Duration::ZERO;
            if !task_tracker.is_closed() {
                let db_start = Instant::now();
                loaded = load_waiting_request_attempts_from_db(
                    pool,
                    worker_id,
                    pulsar,
                    object_storage,
                    hp_retry_cutoff,
                    pulsar_send_receipt_timeout,
                    &ignore,
                    true, // hp_only
                    task_tracker,
                )
                .await?;
                db_elapsed = db_start.elapsed();
            }

            info!(
                reader_phase_ms = reader_elapsed.as_millis() as u64,
                db_phase_ms = db_elapsed.as_millis() as u64,
                ignored_in_topic = ignore.len(),
                loaded,
                topic_was_empty = topic_is_empty,
                "Periodic request_attempts reload completed"
            );
        }
    }

    Ok(())
}

/// Returns whether the message id `current` is at or past `head` within a
/// single Pulsar partition. The ordering compares
/// `(ledger_id, entry_id, batch_index)` — a total order *only* within one
/// partition, since each partition of a partitioned topic has its own
/// independent ledger sequence.
///
/// This is safe here because the periodic reload uses a pulsar-rs `Reader`,
/// which is itself single-partition (`ConsumerBuilder::into_reader` only
/// keeps one partition). If this module is ever reworked to scan a
/// partitioned topic, this function must be replaced with a per-partition
/// head tracker; do **not** call it on ids whose `partition` fields differ.
fn reached_or_past_head(current: &MessageIdData, head: &MessageIdData) -> bool {
    (
        current.ledger_id,
        current.entry_id,
        current.batch_index.unwrap_or(-1),
    ) >= (
        head.ledger_id,
        head.entry_id,
        head.batch_index.unwrap_or(-1),
    )
}

/// Whether the broker-returned "last message id" indicates an empty topic.
/// Pulsar uses entry_id = -1 as the empty-topic sentinel; in the
/// proto-generated `u64` field it surfaces as `u64::MAX`.
fn is_empty_sentinel(id: &MessageIdData) -> bool {
    id.entry_id == u64::MAX
}
