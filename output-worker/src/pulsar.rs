use anyhow::bail;
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, TimeDelta, Utc};
use futures::TryStreamExt;
use pulsar::consumer::{InitialPosition, Message};
use pulsar::proto::MessageIdData;
use pulsar::{
    Consumer, ConsumerOptions, DeserializeMessage, Executor, Producer, ProducerOptions, SubType,
    TokioExecutor,
};
use sqlx::{PgPool, query, query_as};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Sender, channel};
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use tokio::time::{Instant, interval_at, sleep, timeout};
use tokio::{select, spawn};
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::opentelemetry::{
    end_request_attempt_span, gather_pulsar_consumer_metrics, start_request_attempt_span,
};
use crate::throughput_log::ThroughputStats;
use crate::work::work;
use crate::{
    Config, ObjectStorageConfig, PulsarConfig, RequestAttempt, RequestAttemptWithOptionalPayload,
    compute_next_retry,
};
use hook0_protobuf::ObjectStorageResponse;
use hook0_sentry_integration::log_object_storage_error_with_context;

const DELAY_TOLERANCE: Duration = Duration::from_secs(1);

/// Number of consecutive errors from the Pulsar consumer before giving up and restarting
const MAX_CONSECUTIVE_CONSUMER_ERRORS: u32 = 10;

/// Extra time added to the HTTP timeout when draining in-flight tasks after consumer crash
const DRAIN_TIMEOUT_MARGIN: Duration = Duration::from_secs(5);

type AckMessage = (MessageIdData, Option<OwnedSemaphorePermit>, bool);

pub async fn load_waiting_request_attempts_from_db(
    pool: &PgPool,
    worker_id: &Arc<Uuid>,
    pulsar: &Arc<PulsarConfig>,
    object_storage: &Option<ObjectStorageConfig>,
) -> anyhow::Result<u64> {
    let topic = format!(
        "persistent://{}/{}/{}.request_attempt",
        &pulsar.tenant, &pulsar.namespace, worker_id,
    );
    let mut producer = pulsar
        .pulsar
        .producer()
        .with_topic(topic)
        .with_name(format!(
            "hook0-output-worker.{worker_id}.request-attempts-initial-loading.{}",
            Uuid::now_v7()
        ))
        .with_options(ProducerOptions {
            block_queue_if_full: true,
            ..Default::default()
        })
        .build()
        .await?;

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
                s.secret,
                ra.attempt_trigger
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
        ",
        worker_id.as_ref(),
    )
    .fetch(pool);

    let mut counter = 0u64;
    while let Some(ra) = request_attempts_stream.try_next().await? {
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
                attempt_trigger: ra.attempt_trigger.parse().unwrap_or(hook0_protobuf::AttemptTrigger::Dispatch),
            };

            let mut msg_builder = producer
                .create_message()
                .event_time(request_attempt.created_at.timestamp_micros() as u64);
            if let Some(delay_until) = ra.delay_until
                && delay_until > (Utc::now() + DELAY_TOLERANCE)
            {
                msg_builder = msg_builder.deliver_at(delay_until.into())?;
            }

            msg_builder
                .with_content(request_attempt)
                .send_non_blocking()
                .await?;

            counter += 1;
        } else {
            warn!(event_id = %ra.event_id, "Could not get event's payload");
        }
    }

    Ok(counter)
}

#[allow(clippy::too_many_arguments)]
pub async fn look_for_work(
    config: &Arc<Config>,
    pool: &PgPool,
    object_storage: &Arc<Option<ObjectStorageConfig>>,
    worker_id: &Arc<Uuid>,
    worker_name: &Arc<String>,
    worker_version: &Arc<String>,
    pulsar: &Arc<PulsarConfig>,
    heartbeat_tx: Option<Sender<u16>>,
    task_tracker: &TaskTracker,
    stats: &Arc<ThroughputStats>,
) -> anyhow::Result<()> {
    info!("Begin looking for work");
    let topic = format!(
        "persistent://{}/{}/{}.request_attempt",
        &pulsar.tenant, &pulsar.namespace, worker_id,
    );

    let mut consumer = pulsar
        .pulsar
        .consumer()
        .with_topic(&topic)
        .with_consumer_name(format!(
            "hook0-output-worker.{worker_id}.consumer.{}",
            Uuid::now_v7()
        ))
        .with_subscription("hook0-output-worker.delivery")
        .with_subscription_type(SubType::Shared)
        .with_options(ConsumerOptions {
            durable: Some(true),
            initial_position: InitialPosition::Earliest,
            ..Default::default()
        })
        .build::<RequestAttempt>()
        .await?;

    // Create a single producer for retry messages, shared across all tasks
    let retry_producer = Arc::new(Mutex::new(
        pulsar
            .pulsar
            .producer()
            .with_topic(&topic)
            .with_name(format!(
                "hook0-output-worker.{worker_id}.request-attempt-retry.{}",
                Uuid::now_v7()
            ))
            .with_options(ProducerOptions {
                block_queue_if_full: true,
                ..Default::default()
            })
            .build()
            .await?,
    ));

    // This semaphore is what limits the number of inflight webhooks
    let semaphore = Arc::new(Semaphore::new(config.concurrent.into()));

    // This channel is used to bring back the semaphore permit and the message ID to properly destroy/(N)ACK them
    // This is needed because the webhook sendings happen in a Tokio task (to allow concurrency) but we need mutable access to the Pulsar consumer to (N)ACK messages
    let (ack_tx, mut ack_rx) = channel::<AckMessage>(config.concurrent.into());

    // If monitoring heartbeat is enabled, we need to spawn a task to send heartbeats in case the worker does not have any message to process
    if let Some(tx) = heartbeat_tx.clone() {
        let p = Duration::from_secs(config.monitoring_heartbeat_min_period_in_s);
        let tt = task_tracker.clone();
        spawn(async move {
            loop {
                select! {
                    biased;
                    _ = sleep(p) => tx.send(0).await.unwrap(),
                    _ = tt.wait() => break,
                }
            }
        });
    }

    let mut stats_interval = if config.pulsar_consumer_stats_interval.is_zero() {
        None
    } else {
        Some(interval_at(
            Instant::now() + config.pulsar_consumer_stats_interval,
            config.pulsar_consumer_stats_interval,
        ))
    };

    // Tracks consecutive errors from the Pulsar consumer to detect a dead connection
    let mut consecutive_errors: u32 = 0;

    loop {
        // We prepare a future to acquire a permit from the semaphore and then get a message from the Pulsar consumer
        // This future is not awaited yet!
        let next_msg = async {
            let permit = semaphore.clone().acquire_owned().await?;
            let msg_opt = consumer.try_next().await?;
            Ok::<_, anyhow::Error>((permit, msg_opt))
        };

        // We need to await 3 async operations at the same time; the first that finishes will be handled, while the others will be cancelled:
        // 1. We wait for gracefull shutdown to be asked and for inflight webhook tasks to be terminated
        // 2. We wait for at least 1 `AckMessage` to be available in the channel
        // 3. We wait for 2 sequential operations: obtaining a permit from the semaphore (= we can take a new job) and obtaining a message from Pulsar consumer (= there is a new job to take), only if gracefull shutdown was not asked
        select! {
            biased;
            _ = task_tracker.wait() => {
                debug!("Waiting for inflight webhooks to be ACKed");
                while let Ok(msg_ack) = ack_rx.try_recv() {
                    ack_message(&mut consumer, &topic, &heartbeat_tx, msg_ack).await?;
                }
                debug!("Every inflight webhook has been ACKed");
                break;
            }
            Some(msg_ack) = ack_rx.recv() => {
                ack_message(&mut consumer, &topic, &heartbeat_tx, msg_ack).await?;

                // After we have (N)ACK the first item, we check if there are more waiting so we can (N)ACK them immediately (because going back to the select! is slower)
                while let Ok(msg_ack) = ack_rx.try_recv() {
                    ack_message(&mut consumer, &topic, &heartbeat_tx, msg_ack).await?;
                }
            },
            _ = async { stats_interval.as_mut().unwrap().tick().await }, if stats_interval.is_some() => {
                // Note: get_stats() blocks the select loop, but it's a lightweight
                // binary protocol call over the existing connection.
                match consumer.get_stats().await {
                    Ok(stats) => {
                        gather_pulsar_consumer_metrics(&stats);
                    }
                    Err(e) => {
                        warn!("Could not get Pulsar consumer stats: {e}");
                    }
                }
            },
            result = next_msg, if !task_tracker.is_closed() => {
                match result {
                    Ok((permit, Some(msg))) => {
                        consecutive_errors = 0;

                        let ack_tx_for_error = ack_tx.clone();
                        let msg_id = msg.message_id().to_owned();

                        let ack_tx = ack_tx.clone();
                        let c = config.clone();
                        let po = pool.clone();
                        let os = object_storage.clone();
                        let wi = worker_id.clone();
                        let wn = worker_name.clone();
                        let wv = worker_version.clone();
                        let rp = retry_producer.clone();
                        let st = stats.clone();

                        // We handle the request attempt in a new Tokio task
                        task_tracker.spawn(async move {
                            if let Err(e) = handle_message(
                                &c,
                                &po,
                                &os,
                                &wi,
                                &wn,
                                &wv,
                                &rp,
                                msg,
                                permit,
                                ack_tx,
                                &st,
                            )
                            .await
                            {
                                // If the request attempt handling failed, we NACK the message
                                error!("Error while handling message: {e}");
                                if let Err(e) = ack_tx_for_error.send((msg_id, None, false)).await {
                                    error!("Could not send NACK for message (consumer likely restarting): {e}");
                                }
                            }
                        });
                    }
                    Ok((_permit, None)) => {
                        error!("Pulsar consumer stream ended");
                        break;
                    }
                    Err(e) => {
                        consecutive_errors += 1;
                        error!(consecutive_errors, "Pulsar consumer error: {e}");
                        if consecutive_errors >= MAX_CONSECUTIVE_CONSUMER_ERRORS {
                            error!(consecutive_errors, "Too many consecutive Pulsar consumer errors, restarting");
                            break;
                        }
                        // Brief backoff to avoid tight-loop spinning when the broker is down
                        sleep(Duration::from_millis(500)).await;
                    }
                }
            },
            else => break,
        }
    }

    if task_tracker.is_closed() {
        Ok(())
    } else {
        // Drain in-flight tasks before returning, so their ACK/NACKs are attempted.
        // The timeout is bounded by the HTTP timeout + 5s to allow the slowest request to finish.
        // If no tasks are in-flight, this returns immediately.
        drop(ack_tx);
        match timeout(config.timeout + DRAIN_TIMEOUT_MARGIN, async {
            while let Some(msg_ack) = ack_rx.recv().await {
                if let Err(e) = ack_message(&mut consumer, &topic, &heartbeat_tx, msg_ack).await {
                    error!("Failed to ACK/NACK message during drain: {e}");
                }
            }
        })
        .await
        {
            Ok(()) => debug!("All in-flight tasks drained"),
            Err(_) => error!("Timed out draining in-flight tasks"),
        }

        bail!("Pulsar consumer crashed");
    }
}

#[derive(Debug, Clone)]
enum RequestAttemptStatus {
    Ready {
        delay_until: Option<DateTime<Utc>>,
    },
    Delayed {
        delay_until: DateTime<Utc>,
        lead: TimeDelta,
    },
    AlreadyDone,
    Cancelled,
    NotForThisWorker,
    NotFound,
}

#[allow(clippy::too_many_arguments)]
async fn handle_message(
    config: &Config,
    pool: &PgPool,
    object_storage: &Arc<Option<ObjectStorageConfig>>,
    worker_id: &Uuid,
    worker_name: &str,
    worker_version: &str,
    retry_producer: &Mutex<Producer<TokioExecutor>>,
    msg: Message<RequestAttempt>,
    permit: OwnedSemaphorePermit,
    ack_tx: Sender<AckMessage>,
    stats: &ThroughputStats,
) -> anyhow::Result<()> {
    let picked_at = Utc::now();
    let _slot_guard = stats.slot_enter();

    match msg.deserialize() {
        Ok(attempt) => {
            // Check if request attempt must still be done
            struct RawRequestAttemptStatus {
                not_cancelled: bool,
                not_done: bool,
                delay_until: Option<DateTime<Utc>>,
                for_this_worker: bool,
            }
            let fetch_start = std::time::Instant::now();
            let request_attempt_status = match query_as!(
                RawRequestAttemptStatus,
                r#"
                    SELECT
                        (s.is_enabled AND a.deleted_at IS NULL) AS "not_cancelled!",
                        (ra.succeeded_at IS NULL AND ra.failed_at IS NULL) AS "not_done!",
                        ra.delay_until,
                        (
                            EXISTS (
                                SELECT 1
                                FROM webhook.subscription__worker AS sw1
                                WHERE sw1.subscription__id = ra.subscription__id
                                    AND sw1.worker__id IS NOT DISTINCT FROM $2
                            )
                            OR (
                                NOT EXISTS (
                                    SELECT 1
                                    FROM webhook.subscription__worker AS sw2
                                    WHERE sw2.subscription__id = ra.subscription__id
                                )
                                AND EXISTS (
                                    SELECT 1
                                    FROM iam.organization__worker AS ow
                                    WHERE ow.organization__id = a.organization__id
                                        AND ow.default = true
                                        AND ow.worker__id IS NOT DISTINCT FROM $2
                                )
                            )
                        ) AS "for_this_worker!"
                    FROM webhook.request_attempt AS ra
                    INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                    INNER JOIN event.application AS a ON a.application__id = s.application__id
                    WHERE ra.request_attempt__id = $1
                "#,
                attempt.request_attempt_id,
                worker_id,
            )
            .fetch_optional(pool)
            .await?
            {
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: true,
                    for_this_worker: true,
                    delay_until: Some(d),
                }) if d > (Utc::now() + DELAY_TOLERANCE) => RequestAttemptStatus::Delayed {
                    delay_until: d,
                    lead: d - Utc::now(),
                },
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: true,
                    for_this_worker: true,
                    delay_until,
                }) => RequestAttemptStatus::Ready { delay_until },
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: false,
                    ..
                }) => RequestAttemptStatus::AlreadyDone,
                Some(RawRequestAttemptStatus {
                    not_cancelled: false,
                    ..
                }) => RequestAttemptStatus::Cancelled,
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: true,
                    for_this_worker: false,
                    ..
                }) => RequestAttemptStatus::NotForThisWorker,
                None => RequestAttemptStatus::NotFound,
            };
            stats.record_db_fetch(fetch_start.elapsed());

            match request_attempt_status {
                RequestAttemptStatus::Ready { delay_until } => {
                    // Record queue lag: time between becoming eligible and pickup
                    let eligible_at = delay_until
                        .unwrap_or(attempt.created_at)
                        .max(attempt.created_at);
                    if let Ok(lag) = (picked_at - eligible_at).to_std() {
                        stats.record_lag(lag);
                    }

                    // Start OpenTelemetry span
                    let span = start_request_attempt_span(&attempt);

                    // Work
                    let response = work(config, &attempt).await;
                    trace!(request_attempt_id = %attempt.request_attempt_id, elapsed_ms = response.elapsed_time_ms(), "Got response for request attempt");

                    // Open DB transaction
                    let mut tx = pool.begin().await?;

                    // Store response
                    trace!(request_attempt_id = %attempt.request_attempt_id, "Storing response");
                    let response_headers = response.headers();
                    let response_contents_to_insert = if let Some(true) =
                        object_storage.as_ref().as_ref().map(|object_storage| {
                            object_storage.store_response_body_and_headers
                                && (object_storage
                                    .store_response_body_and_headers_only_for
                                    .is_empty()
                                    || object_storage
                                        .store_response_body_and_headers_only_for
                                        .contains(&attempt.application_id))
                                && (response.body.is_some() || response_headers.is_some())
                        }) {
                        None
                    } else {
                        Some((&response.body, &response_headers))
                    };
                    let response_id = query!(
                        "
                            INSERT INTO webhook.response (response_error__name, http_code, headers, body, elapsed_time_ms)
                            VALUES ($1, $2, $3, $4, $5)
                            RETURNING response__id
                        ",
                        response.response_error__name(),
                        response.http_code(),
                        response_contents_to_insert.map(|(_, headers)| headers.to_owned()).unwrap_or(None),
                        response_contents_to_insert.map(|(body, _)| body.to_owned()).unwrap_or(None),
                        response.elapsed_time_ms(),
                    )
                    .fetch_one(&mut *tx)
                    .await?
                    .response__id;

                    if let Some(object_storage) = object_storage.as_ref()
                        && object_storage.store_response_body_and_headers
                        && (object_storage
                            .store_response_body_and_headers_only_for
                            .is_empty()
                            || object_storage
                                .store_response_body_and_headers_only_for
                                .contains(&attempt.application_id))
                        && (response.body.is_some() || response_headers.is_some())
                    {
                        let key = format!(
                            "{}/response/{}/{response_id}",
                            attempt.application_id,
                            attempt.created_at.naive_utc().date()
                        );
                        let object: Vec<u8> = ObjectStorageResponse {
                            body: response.body.clone().unwrap_or_default(),
                            headers: response_headers.unwrap_or_default(),
                        }
                        .try_into()?;
                        object_storage
                            .client
                            .put_object()
                            .bucket(&object_storage.bucket)
                            .key(&key)
                            .content_type("application/protobuf")
                            .body(ByteStream::from(object))
                            .send()
                            .await
                            .inspect_err(|e| {
                                log_object_storage_error_with_context!(
                                    "S3 PUT OBJECT failed",
                                    error_chain = DisplayErrorContext(e).to_string(),
                                    object_key = &key,
                                );
                            })?;
                    }

                    // Between the status check and this UPDATE, another process (e.g. subscription disable or Pulsar message redelivery) may have already finalized this attempt.
                    // The UPDATE guards against this by requiring succeeded_at/failed_at to still be NULL.
                    let race_detected = if response.is_success() {
                        // Mark attempt as completed
                        trace!(request_attempt_id = %attempt.request_attempt_id, "Completing request attempt");
                        let update_result = query!(
                            "
                                UPDATE webhook.request_attempt
                                SET worker_name = $1,
                                    worker_version = $2,
                                    picked_at = $3,
                                    response__id = $4,
                                    succeeded_at = statement_timestamp()
                                WHERE request_attempt__id = $5
                                    AND succeeded_at IS NULL
                                    AND failed_at IS NULL
                            ",
                            worker_name,
                            worker_version,
                            picked_at,
                            response_id,
                            attempt.request_attempt_id,
                        )
                        .execute(&mut *tx)
                        .await?;

                        if update_result.rows_affected() == 0 {
                            warn!(request_attempt_id = %attempt.request_attempt_id, "Race detected: request attempt was already finalized by another process; skipping");
                            true
                        } else {
                            debug!(request_attempt_id = %attempt.request_attempt_id, "Request attempt completed successfully");
                            false
                        }
                    } else {
                        // Mark attempt as failed
                        trace!(request_attempt_id = %attempt.request_attempt_id, "Failing request attempt");
                        let update_result = query!(
                            "
                                UPDATE webhook.request_attempt
                                SET worker_name = $1,
                                    worker_version = $2,
                                    picked_at = $3,
                                    response__id = $4,
                                    failed_at = statement_timestamp()
                                WHERE request_attempt__id = $5
                                    AND succeeded_at IS NULL
                                    AND failed_at IS NULL
                            ",
                            worker_name,
                            worker_version,
                            picked_at,
                            response_id,
                            attempt.request_attempt_id,
                        )
                        .execute(&mut *tx)
                        .await?;

                        if update_result.rows_affected() == 0 {
                            warn!(request_attempt_id = %attempt.request_attempt_id, "Race detected: request attempt was already finalized by another process; skipping retry");
                            true
                        } else {
                            // Creating a retry request or giving up
                            if attempt.attempt_trigger == hook0_protobuf::AttemptTrigger::ManualRetry {
                                info!(request_attempt_id = %attempt.request_attempt_id, "Manual retry failed; not re-queuing (one-shot)");
                            } else if let Some(retry_in) =
                                compute_next_retry(&mut tx, &attempt, &response, config.max_retries)
                                    .await?
                            {
                                let next_retry_count = attempt.retry_count + 1;
                                let delay_until = Utc::now() + retry_in;

                                #[allow(non_snake_case)]
                                struct Retry {
                                    request_attempt__id: Uuid,
                                    created_at: DateTime<Utc>,
                                }
                                let retry = query_as!(
                                    Retry,
                                    "
                                        INSERT INTO webhook.request_attempt (application__id, event__id, subscription__id, delay_until, retry_count, attempt_trigger)
                                        VALUES ($1, $2, $3, $4, $5, 'auto_retry')
                                        RETURNING request_attempt__id, created_at
                                    ",
                                    attempt.application_id,
                                    attempt.event_id,
                                    attempt.subscription_id,
                                    delay_until,
                                    next_retry_count,
                                )
                                .fetch_one(&mut *tx)
                                .await?;

                                debug!(request_attempt_id = %attempt.request_attempt_id, retry_count = next_retry_count, retry_id = %retry.request_attempt__id, retry_in_secs = retry_in.as_secs(), "Request attempt failed; retry created");

                                retry_producer
                                    .lock()
                                    .await
                                    .create_message()
                                    .event_time(retry.created_at.timestamp_micros() as u64)
                                    .deliver_at(delay_until.into())?
                                    .with_content(RequestAttempt {
                                        request_attempt_id: retry.request_attempt__id,
                                        created_at: retry.created_at,
                                        retry_count: next_retry_count,
                                        attempt_trigger: hook0_protobuf::AttemptTrigger::AutoRetry,
                                        ..attempt
                                    })
                                    .send_non_blocking()
                                    .await?;
                            } else {
                                debug!(request_attempt_id = %attempt.request_attempt_id, retry_count = attempt.retry_count, "Request attempt failed; giving up");
                            }
                            false
                        }
                    };

                    if race_detected {
                        tx.rollback().await?;
                    } else {
                        tx.commit().await?;

                        stats.record_attempt(
                            response.is_success(),
                            attempt.retry_count,
                            response.elapsed_time,
                        );
                    }

                    // End OpenTelemetry span
                    end_request_attempt_span(span, &response);

                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                // This should never happen because delayed request attempts are sent to Pulsar with a `deliver_at` constraint
                // This process is there to make sure delayed request attempts will not be processed immediately if the Pulsar producer made a mistake
                RequestAttemptStatus::Delayed { delay_until, lead } => {
                    stats.record_not_ready();
                    trace!(request_attempt_id = %attempt.request_attempt_id, lead = ?lead, "Request attempt was scheduled for later");
                    retry_producer
                        .lock()
                        .await
                        .create_message()
                        .event_time(attempt.created_at.timestamp_micros() as u64)
                        .deliver_at(delay_until.into())?
                        .with_content(attempt)
                        .send_non_blocking()
                        .await?;
                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                RequestAttemptStatus::AlreadyDone => {
                    stats.record_not_ready();
                    trace!(request_attempt_id = %attempt.request_attempt_id, "Request attempt was already done");
                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                RequestAttemptStatus::Cancelled => {
                    stats.record_not_ready();
                    trace!(request_attempt_id = %attempt.request_attempt_id, "Request attempt was cancelled");
                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                RequestAttemptStatus::NotForThisWorker => {
                    debug!(request_attempt_id = %attempt.request_attempt_id, "Request attempt should not be handled by the present worker");

                    // Message is only ACKed and not republished into the right topic because the rightful worker is supposed to reload everything from the database when it first starts
                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                RequestAttemptStatus::NotFound => {
                    stats.record_not_ready();
                    if attempt.created_at + config.request_attempt_db_commit_grace_period
                        >= Utc::now()
                    {
                        trace!(request_attempt_id = %attempt.request_attempt_id, "Request attempt not found in database; created recently, will retry later");

                        ack_tx
                            .send((msg.message_id().clone(), Some(permit), false))
                            .await?;
                    } else {
                        trace!(request_attempt_id = %attempt.request_attempt_id, "Request attempt not found in database; not recent, dropping");

                        ack_tx
                            .send((msg.message_id().clone(), Some(permit), true))
                            .await?;
                    }

                    Ok(())
                }
            }
        }
        Err(e) => {
            error!("Could not deserialize request attempt: {e}");
            ack_tx
                .send((msg.message_id().clone(), Some(permit), false))
                .await?;

            Ok(())
        }
    }
}

async fn ack_message<T, E>(
    consumer: &mut Consumer<T, E>,
    topic: &str,
    heartbeat_tx: &Option<Sender<u16>>,
    (msg_id, permit, is_ok): AckMessage,
) -> Result<(), SendError<u16>>
where
    T: DeserializeMessage,
    E: Executor,
{
    // ACK or NACK the message in Pulsar
    if is_ok {
        let _ = consumer
            .ack_with_id(topic, msg_id)
            .await
            .inspect_err(|e| error!("Could not ACK Pulsar message: {e}"));
    } else {
        let _ = consumer
            .nack_with_id(topic, msg_id)
            .await
            .inspect_err(|e| error!("Could not NACK Pulsar message: {e}"));
    }

    // Send monitoring heartbeat if necessary
    if let Some(tx) = heartbeat_tx {
        tx.send(0).await?;
    }

    // Drop the semaphore permit
    drop(permit);

    Ok(())
}
