use anyhow::bail;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use log::{debug, error, info, trace};
use pulsar::consumer::{InitialPosition, Message};
use pulsar::proto::MessageIdData;
use pulsar::{Consumer, ConsumerOptions, DeserializeMessage, Executor, ProducerOptions, SubType};
use sqlx::{PgPool, query, query_as};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Sender, channel};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;
use tokio::{select, spawn};
use tokio_util::task::TaskTracker;
use uuid::Uuid;

use crate::work::work;
use crate::{Config, PulsarConfig, RequestAttempt, compute_next_retry};

type AckMessage = (MessageIdData, Option<OwnedSemaphorePermit>, bool);

pub async fn load_waiting_request_attempts_from_db(
    pool: &PgPool,
    worker_id: &Arc<Uuid>,
    pulsar: &Arc<PulsarConfig>,
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
        RequestAttempt,
        "
            SELECT
                ra.request_attempt__id AS request_attempt_id,
                ra.event__id AS event_id,
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
        producer
            .create_message()
            .event_time(ra.created_at.timestamp_micros() as u64)
            .with_content(ra)
            .send_non_blocking()
            .await?;
        counter += 1;
    }

    Ok(counter)
}

#[allow(clippy::too_many_arguments)]
pub async fn look_for_work(
    config: &Arc<Config>,
    pool: &PgPool,
    worker_id: &Arc<Uuid>,
    worker_name: &Arc<String>,
    worker_version: &Arc<String>,
    pulsar: &Arc<PulsarConfig>,
    heartbeat_tx: Option<Sender<u16>>,
    task_tracker: &TaskTracker,
) -> anyhow::Result<()> {
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
            Ok((permit, msg_opt)) = next_msg, if !task_tracker.is_closed() => {
                if let Some(msg) = msg_opt {
                    let ack_tx_for_error = ack_tx.clone();
                    let msg_id = msg.message_id().to_owned();

                    let ack_tx = ack_tx.clone();
                    let c = config.clone();
                    let po = pool.clone();
                    let wid = worker_id.clone();
                    let wn = worker_name.clone();
                    let wv = worker_version.clone();
                    let pu = pulsar.clone();
                    let t = topic.clone();

                    // We handle the request attempt in a new Tokio task
                    task_tracker.spawn(async move {
                        if let Err(e) = handle_message(
                            &c,
                            &po,
                            &wid,
                            &wn,
                            &wv,
                            pu.as_ref(),
                            &t,
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
    Ready { is_fifo: bool },
    AlreadyDone,
    Cancelled,
    NotFound,
}

#[allow(clippy::too_many_arguments)]
async fn handle_message(
    config: &Config,
    pool: &PgPool,
    worker_id: &Uuid,
    worker_name: &str,
    worker_version: &str,
    pulsar: &PulsarConfig,
    topic: &str,
    msg: Message<RequestAttempt>,
    permit: OwnedSemaphorePermit,
    ack_tx: Sender<AckMessage>,
) -> anyhow::Result<()> {
    let picked_at = Utc::now();

    match msg.deserialize() {
        Ok(attempt) => {
            // Check if request attempt must still be done and check FIFO constraints
            struct RawRequestAttemptStatus {
                not_cancelled: bool,
                not_done: bool,
                fifo_mode: bool,
                fifo_blocked: bool,
            }
            let request_attempt_status = match query_as!(
                RawRequestAttemptStatus,
                r#"
                    SELECT
                        (s.is_enabled AND a.deleted_at IS NULL) AS "not_cancelled!",
                        (ra.succeeded_at IS NULL AND ra.failed_at IS NULL) AS "not_done!",
                        s.fifo_mode AS "fifo_mode!",
                        (s.fifo_mode = true AND fss.current_request_attempt__id IS NOT NULL AND fss.current_request_attempt__id != ra.request_attempt__id) AS "fifo_blocked!"
                    FROM webhook.request_attempt AS ra
                    INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                    INNER JOIN event.application AS a ON a.application__id = s.application__id
                    INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                    LEFT JOIN webhook.fifo_subscription_state AS fss ON fss.subscription__id = s.subscription__id
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
                    fifo_mode: _,
                    fifo_blocked: true,
                }) => {
                    debug!(
                        "Request attempt {} is blocked by FIFO constraint, will retry later",
                        &attempt.request_attempt_id
                    );
                    RequestAttemptStatus::Cancelled // Will be nacked and retried
                },
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: true,
                    fifo_mode,
                    fifo_blocked: false,
                }) => {
                    // If FIFO mode, create state entry to block other requests
                    if fifo_mode {
                        debug!(
                            "[FIFO] Subscription {} entering FIFO mode, blocking request attempt {}",
                            &attempt.subscription_id,
                            &attempt.request_attempt_id
                        );
                        query!(
                            "
                                INSERT INTO webhook.fifo_subscription_state (subscription__id, current_request_attempt__id, updated_at)
                                VALUES ($1, $2, statement_timestamp())
                                ON CONFLICT (subscription__id) DO UPDATE
                                SET current_request_attempt__id = $2, updated_at = statement_timestamp()
                            ",
                            attempt.subscription_id,
                            attempt.request_attempt_id
                        )
                        .execute(pool)
                        .await?;
                    }
                    RequestAttemptStatus::Ready { is_fifo: fifo_mode }
                },
                Some(RawRequestAttemptStatus {
                    not_cancelled: true,
                    not_done: false,
                    fifo_mode: _,
                    fifo_blocked: _,
                }) => RequestAttemptStatus::AlreadyDone,
                Some(RawRequestAttemptStatus {
                    not_cancelled: false,
                    not_done: _,
                    fifo_mode: _,
                    fifo_blocked: _,
                }) => RequestAttemptStatus::Cancelled,
                None => RequestAttemptStatus::NotFound,
            };

            match request_attempt_status {
                RequestAttemptStatus::Ready { is_fifo } => {
                    // Work
                    let response = work(config, &attempt).await;
                    debug!(
                        "Got a response for request attempt {} in {} ms",
                        &attempt.request_attempt_id,
                        &response.elapsed_time_ms()
                    );
                    trace!("{response:?}");

                    // Open DB transaction
                    let mut tx = pool.begin().await?;

                    // Store response
                    debug!(
                        "Storing response for request attempt {}",
                        &attempt.request_attempt_id
                    );
                    let response_id = query!(
                        "
                            INSERT INTO webhook.response (response_error__name, http_code, headers, body, elapsed_time_ms)
                            VALUES ($1, $2, $3, $4, $5)
                            RETURNING response__id
                        ",
                        response.response_error__name(),
                        response.http_code(),
                        response.headers(),
                        response.body,
                        response.elapsed_time_ms(),
                    )
                    .fetch_one(&mut *tx)
                    .await?
                    .response__id;

                    if response.is_success() {
                        // Mark attempt as completed
                        debug!("Completing request attempt {}", &attempt.request_attempt_id);
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

                        // Clear FIFO state for this subscription
                        if is_fifo {
                            debug!(
                                "[FIFO] Subscription {} unblocked after successful request attempt {}",
                                &attempt.subscription_id,
                                &attempt.request_attempt_id
                            );
                            query!(
                                "
                                    UPDATE webhook.fifo_subscription_state
                                    SET
                                        current_request_attempt__id = NULL,
                                        last_completed_event_created_at = (SELECT e.occurred_at FROM event.event e WHERE e.event__id = $2),
                                        updated_at = statement_timestamp()
                                    WHERE subscription__id = $1
                                ",
                                attempt.subscription_id,
                                attempt.event_id
                            )
                            .execute(&mut *tx)
                            .await?;
                        }

                        info!(
                            "Request attempt {} was completed sucessfully",
                            &attempt.request_attempt_id
                        );
                    } else {
                        // Mark attempt as failed
                        debug!("Failing request attempt {}", &attempt.request_attempt_id);
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
                            &attempt.subscription_id,
                            config.max_fast_retries,
                            config.max_slow_retries,
                            attempt.retry_count,
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
                                    INSERT INTO webhook.request_attempt (event__id, subscription__id, delay_until, retry_count)
                                    VALUES ($1, $2, $3, $4)
                                    RETURNING request_attempt__id, created_at
                                ",
                                attempt.event_id,
                                attempt.subscription_id,
                                delay_until,
                                next_retry_count,
                            )
                            .fetch_one(&mut *tx)
                            .await?;

                            // Update FIFO state to point to the retry request
                            if is_fifo {
                                info!(
                                    "[FIFO] Subscription {} remains blocked, retry {} scheduled for {}s",
                                    &attempt.subscription_id,
                                    &retry.request_attempt__id,
                                    &retry_in.as_secs()
                                );
                                query!(
                                    "
                                        UPDATE webhook.fifo_subscription_state
                                        SET current_request_attempt__id = $1, updated_at = statement_timestamp()
                                        WHERE subscription__id = $2
                                    ",
                                    retry.request_attempt__id,
                                    attempt.subscription_id
                                )
                                .execute(&mut *tx)
                                .await?;
                            }

                            info!(
                                "Request attempt {} failed; retry #{next_retry_count} created as {} to be picked in {}s",
                                &attempt.request_attempt_id,
                                retry.request_attempt__id,
                                &retry_in.as_secs(),
                            );

                            pulsar
                                .pulsar
                                .producer()
                                .with_topic(topic)
                                .with_name(format!(
                                    "hook0-output-worker.{worker_id}.request-attempt-retry.{}",
                                    Uuid::now_v7()
                                ))
                                .with_options(ProducerOptions {
                                    block_queue_if_full: true,
                                    ..Default::default()
                                })
                                .build()
                                .await?
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
                            // Clear FIFO state when giving up
                            if is_fifo {
                                info!(
                                    "[FIFO] Subscription {} unblocked after exhausting retries for request attempt {}",
                                    &attempt.subscription_id,
                                    &attempt.request_attempt_id
                                );
                                query!(
                                    "
                                        UPDATE webhook.fifo_subscription_state
                                        SET
                                            current_request_attempt__id = NULL,
                                            last_completed_event_created_at = (SELECT e.occurred_at FROM event.event e WHERE e.event__id = $2),
                                            updated_at = statement_timestamp()
                                        WHERE subscription__id = $1
                                    ",
                                    attempt.subscription_id,
                                    attempt.event_id
                                )
                                .execute(&mut *tx)
                                .await?;
                            }

                            info!(
                                "Request attempt {} failed after {} attempts; giving up",
                                &attempt.request_attempt_id, &attempt.retry_count,
                            );
                        }
                    }

                    tx.commit().await?;
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
                    if attempt.created_at + Duration::from_secs(2) >= Utc::now() {
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
