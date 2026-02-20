use anyhow::bail;
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use log::{debug, error, info, trace, warn};
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
use tokio::time::{Instant, interval_at, sleep};
use tokio::{select, spawn};
use tokio_util::task::TaskTracker;
use uuid::Uuid;

use crate::opentelemetry::{
    end_request_attempt_span, gather_pulsar_consumer_metrics, start_request_attempt_span,
};
use crate::work::work;
use crate::{
    Config, ObjectStorageConfig, PulsarConfig, RequestAttempt, RequestAttemptWithOptionalPayload,
    compute_next_retry,
};
use hook0_protobuf::ObjectStorageResponse;
use hook0_sentry_integration::log_object_storage_error_with_context;

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
            };
            producer
                .create_message()
                .event_time(request_attempt.created_at.timestamp_micros() as u64)
                .with_content(request_attempt)
                .send_non_blocking()
                .await?;
            counter += 1;
        } else {
            warn!("Could not get payload for event {}", ra.event_id);
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

    let mut stats_interval = interval_at(
        Instant::now() + config.pulsar_consumer_stats_interval,
        config.pulsar_consumer_stats_interval,
    );

    loop {
        // We prepare a future to acquire a permit from the semaphore and then get a message from the Pulsar consumer
        // This future is not awaited yet!
        let next_msg = async {
            let permit = semaphore.clone().try_acquire_owned()?;
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
            _ = stats_interval.tick() => {
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
            Ok((permit, msg_opt)) = next_msg, if !task_tracker.is_closed() => {
                if let Some(msg) = msg_opt {
                    let ack_tx_for_error = ack_tx.clone();
                    let msg_id = msg.message_id().to_owned();

                    let ack_tx = ack_tx.clone();
                    let c = config.clone();
                    let po = pool.clone();
                    let os = object_storage.clone();
                    let wn = worker_name.clone();
                    let wv = worker_version.clone();
                    let rp = retry_producer.clone();

                    // We handle the request attempt in a new Tokio task
                    task_tracker.spawn(async move {
                        if let Err(e) = handle_message(
                            &c,
                            &po,
                            &os,
                            &wn,
                            &wv,
                            &rp,
                            msg,
                            permit,
                            ack_tx,
                        )
                        .await
                        {
                            // If the request attempt handling failed, we NACK the message
                            error!("Error while handling message: {e}");
                            ack_tx_for_error.send((msg_id, None, false)).await.unwrap();
                        }
                    });
                }
            },
            else => break,
        }
    }

    if task_tracker.is_closed() {
        Ok(())
    } else {
        bail!("Pulsar consumer crashed");
    }
}

#[derive(Debug, Clone, Copy)]
enum RequestAttemptStatus {
    Ready,
    AlreadyDone,
    Cancelled,
    NotFound,
}

#[allow(clippy::too_many_arguments)]
async fn handle_message(
    config: &Config,
    pool: &PgPool,
    object_storage: &Arc<Option<ObjectStorageConfig>>,
    worker_name: &str,
    worker_version: &str,
    retry_producer: &Mutex<Producer<TokioExecutor>>,
    msg: Message<RequestAttempt>,
    permit: OwnedSemaphorePermit,
    ack_tx: Sender<AckMessage>,
) -> anyhow::Result<()> {
    let picked_at = Utc::now();

    match msg.deserialize() {
        Ok(attempt) => {
            // Check if request attempt must still be done
            struct RawRequestAttemptStatus {
                not_cancelled: bool,
                not_done: bool,
            }
            let request_attempt_status = match query_as!(
                RawRequestAttemptStatus,
                r#"
                    SELECT
                        (s.is_enabled AND a.deleted_at IS NULL) AS "not_cancelled!",
                        (ra.succeeded_at IS NULL AND ra.failed_at IS NULL) AS "not_done!"
                    FROM webhook.request_attempt AS ra
                    INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                    INNER JOIN event.application AS a ON a.application__id = s.application__id
                    INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                    WHERE ra.request_attempt__id = $1
                "#,
                attempt.request_attempt_id,
            )
            .fetch_optional(pool)
            .await?
            {
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: true,
                }) => RequestAttemptStatus::Ready,
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: false,
                }) => RequestAttemptStatus::AlreadyDone,
                Some(RawRequestAttemptStatus {
                    not_cancelled: false,
                    not_done: _,
                }) => RequestAttemptStatus::Cancelled,
                None => RequestAttemptStatus::NotFound,
            };

            match request_attempt_status {
                RequestAttemptStatus::Ready => {
                    // Start OpenTelemetry span
                    let span = start_request_attempt_span(&attempt);

                    // Work
                    let response = work(config, &attempt).await;
                    trace!(
                        "Got a response for request attempt {} in {} ms",
                        &attempt.request_attempt_id,
                        &response.elapsed_time_ms()
                    );

                    // Open DB transaction
                    let mut tx = pool.begin().await?;

                    // Store response
                    trace!(
                        "Storing response for request attempt {}",
                        &attempt.request_attempt_id
                    );
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

                    if response.is_success() {
                        // Mark attempt as completed
                        trace!("Completing request attempt {}", &attempt.request_attempt_id);
                        query!(
                            "
                                UPDATE webhook.request_attempt
                                SET worker_name = $1,
                                    worker_version = $2,
                                    picked_at = $3,
                                    response__id = $4,
                                    succeeded_at = statement_timestamp()
                                WHERE request_attempt__id = $5
                            ",
                            worker_name,
                            worker_version,
                            picked_at,
                            response_id,
                            attempt.request_attempt_id,
                        )
                        .execute(&mut *tx)
                        .await?;

                        debug!(
                            "Request attempt {} was completed sucessfully",
                            &attempt.request_attempt_id
                        );
                    } else {
                        // Mark attempt as failed
                        trace!("Failing request attempt {}", &attempt.request_attempt_id);
                        query!(
                            "
                                UPDATE webhook.request_attempt
                                SET worker_name = $1,
                                    worker_version = $2,
                                    picked_at = $3,
                                    response__id = $4,
                                    failed_at = statement_timestamp()
                                WHERE request_attempt__id = $5
                            ",
                            worker_name,
                            worker_version,
                            picked_at,
                            response_id,
                            attempt.request_attempt_id,
                        )
                        .execute(&mut *tx)
                        .await?;

                        // Creating a retry request or giving up
                        if let Some(retry_in) = compute_next_retry(
                            &mut tx,
                            &attempt,
                            &response,
                            config.max_fast_retries,
                            config.max_slow_retries,
                        )
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
                                    INSERT INTO webhook.request_attempt (application__id, event__id, subscription__id, delay_until, retry_count)
                                    VALUES ($1, $2, $3, $4, $5)
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

                            debug!(
                                "Request attempt {} failed; retry #{next_retry_count} created as {} to be picked in {}s",
                                &attempt.request_attempt_id,
                                retry.request_attempt__id,
                                &retry_in.as_secs(),
                            );

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
                                    ..attempt
                                })
                                .send_non_blocking()
                                .await?;
                        } else {
                            debug!(
                                "Request attempt {} failed after {} attempts; giving up",
                                &attempt.request_attempt_id, &attempt.retry_count,
                            );
                        }
                    }

                    tx.commit().await?;

                    // End OpenTelemetry span
                    end_request_attempt_span(span, &response);

                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                RequestAttemptStatus::AlreadyDone => {
                    trace!(
                        "Request attempt {} was already done according to database",
                        &attempt.request_attempt_id
                    );
                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                RequestAttemptStatus::Cancelled => {
                    trace!(
                        "Request attempt {} was cancelled according to database",
                        &attempt.request_attempt_id
                    );
                    ack_tx
                        .send((msg.message_id().clone(), Some(permit), true))
                        .await?;

                    Ok(())
                }
                RequestAttemptStatus::NotFound => {
                    if attempt.created_at + config.request_attempt_db_commit_grace_period
                        >= Utc::now()
                    {
                        trace!(
                            "Request attempt {} was not found in database; as it was created recently it may not have been committed into database yet so let's retry a bit later",
                            &attempt.request_attempt_id
                        );

                        ack_tx
                            .send((msg.message_id().clone(), Some(permit), false))
                            .await?;
                    } else {
                        trace!(
                            "Request attempt {} was not found in database; it was not created recently so let's drop it",
                            &attempt.request_attempt_id
                        );

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
