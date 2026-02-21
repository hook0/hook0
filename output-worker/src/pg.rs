use anyhow::anyhow;
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use log::{debug, info, trace, warn};
use sqlx::postgres::types::PgInterval;
use sqlx::{PgPool, query, query_as};
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use tokio_util::task::TaskTracker;

use crate::opentelemetry::{end_request_attempt_span, start_request_attempt_span};
use crate::work::work;
use crate::{
    Config, ObjectStorageConfig, RequestAttemptWithOptionalPayload, Worker, WorkerScope,
    compute_next_retry,
};
use hook0_protobuf::{ObjectStorageResponse, RequestAttempt};
use hook0_sentry_integration::log_object_storage_error_with_context;

/// Minimum duration to wait when there are no unprocessed items to pick
const MIN_POLLING_SLEEP: Duration = Duration::from_secs(1);

/// Maximum duration to wait when there are no unprocessed items to pick
const MAX_POLLING_SLEEP: Duration = Duration::from_secs(10);

#[allow(clippy::too_many_arguments)]
pub async fn look_for_work(
    config: &Config,
    unit_id: u16,
    pool: &PgPool,
    object_storage: &Option<ObjectStorageConfig>,
    worker: &Worker,
    worker_version: &str,
    heartbeat_tx: Option<Sender<u16>>,
    task_tracker: &TaskTracker,
) -> anyhow::Result<()> {
    info!("[unit={unit_id}] Begin looking for work");
    loop {
        trace!("[unit={unit_id}] Fetching next unprocessed request attempt...");
        let mut tx = pool.begin().await?;

        let next_attempt = match worker.scope {
            WorkerScope::Public { worker_id } => {
                // Only consider request attempts where associated subscription have no dedicated worker specified
                query_as!(
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
                            t_http.method AS http_method,
                            t_http.url AS http_url,
                            t_http.headers AS http_headers,
                            e.event_type__name AS event_type_name,
                            e.payload AS payload,
                            e.payload_content_type AS payload_content_type,
                            s.secret
                        FROM webhook.request_attempt AS ra
                        INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                        LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                        INNER JOIN event.application AS a ON a.application__id = s.application__id AND a.deleted_at IS NULL
                        INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                        LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = o.organization__id AND ow.default = true
                        INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                        INNER JOIN event.event AS e ON e.event__id = ra.event__id
                        WHERE
                            ra.succeeded_at IS NULL
                            AND ra.failed_at IS NULL
                            AND s.is_enabled
                            AND s.deleted_at IS NULL
                            AND (ra.delay_until IS NULL OR ra.delay_until <= statement_timestamp())
                            AND (COALESCE(sw.worker__id, ow.worker__id) IS NULL OR COALESCE(sw.worker__id, ow.worker__id) = $1)
                        ORDER BY ra.created_at ASC
                        LIMIT 1
                        FOR UPDATE OF ra
                        SKIP LOCKED
                    ",
                    worker_id.to_owned(),
                )
                .fetch_optional(&mut *tx)
                .await?
            }
            WorkerScope::Private { worker_id } => {
                // Only consider request attempts where associated subscription have at least the currect worker specified as dedicated worker
                query_as!(
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
                            t_http.method AS http_method,
                            t_http.url AS http_url,
                            t_http.headers AS http_headers,
                            e.event_type__name AS event_type_name,
                            e.payload AS payload,
                            e.payload_content_type AS payload_content_type,
                            s.secret
                        FROM webhook.request_attempt AS ra
                        INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                        LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                        INNER JOIN event.application AS a ON a.application__id = s.application__id AND a.deleted_at IS NULL
                        INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                        LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = o.organization__id AND ow.default = true
                        INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                        INNER JOIN event.event AS e ON e.event__id = ra.event__id
                        WHERE
                            ra.succeeded_at IS NULL
                            AND ra.failed_at IS NULL
                            AND s.is_enabled
                            AND s.deleted_at IS NULL
                            AND (ra.delay_until IS NULL OR ra.delay_until <= statement_timestamp())
                            AND (COALESCE(sw.worker__id, ow.worker__id) = $1)
                        ORDER BY ra.created_at ASC
                        LIMIT 1
                        FOR UPDATE OF ra
                        SKIP LOCKED
                    ",
                    &worker_id,
                )
                .fetch_optional(&mut *tx)
                .await?
            }
        };

        if let Some(attempt) = next_attempt {
            // Set picked_at
            trace!(
                "[unit={unit_id}] Picking request attempt {}",
                &attempt.request_attempt_id
            );
            query!(
                "
                    UPDATE webhook.request_attempt
                    SET picked_at = statement_timestamp(), worker_name = $1, worker_version = $2
                    WHERE request_attempt__id = $3
                ",
                &worker.name,
                &worker_version,
                attempt.request_attempt_id
            )
            .execute(&mut *tx)
            .await?;
            debug!(
                "[unit={unit_id}] Picked request attempt {}",
                &attempt.request_attempt_id
            );

            let payload = if let Some(p) = attempt.payload {
                Some(p)
            } else if let Some(os) = &object_storage {
                let key = format!(
                    "{}/event/{}/{}",
                    attempt.application_id,
                    attempt.event_received_at.naive_utc().date(),
                    attempt.event_id
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
                let attempt_with_payload = RequestAttempt {
                    application_id: attempt.application_id,
                    request_attempt_id: attempt.request_attempt_id,
                    event_id: attempt.event_id,
                    event_received_at: attempt.event_received_at,
                    subscription_id: attempt.subscription_id,
                    created_at: attempt.created_at,
                    retry_count: attempt.retry_count,
                    http_method: attempt.http_method,
                    http_url: attempt.http_url,
                    http_headers: attempt.http_headers,
                    event_type_name: attempt.event_type_name,
                    payload: p,
                    payload_content_type: attempt.payload_content_type,
                    secret: attempt.secret,
                };

                // Start OpenTelemetry span
                let span = start_request_attempt_span(&attempt_with_payload);

                // Work
                let response = work(config, &attempt_with_payload).await;
                trace!(
                    "[unit={unit_id}] Got a response for request attempt {} in {} ms",
                    &attempt.request_attempt_id,
                    &response.elapsed_time_ms()
                );

                // Store response
                trace!(
                    "[unit={unit_id}] Storing response for request attempt {}",
                    &attempt.request_attempt_id
                );
                let response_headers = response.headers();
                let response_contents_to_insert = if let Some(true) =
                    object_storage.as_ref().map(|object_storage| {
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

                if let Some(object_storage) = object_storage
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

                // Associate response and request attempt
                trace!(
                    "[unit={unit_id}] Associating response {response_id} with request attempt {}",
                    &attempt.request_attempt_id
                );
                query!(
                    "UPDATE webhook.request_attempt SET response__id = $1 WHERE request_attempt__id = $2",
                    response_id, attempt.request_attempt_id
                )
                .execute(&mut *tx)
                .await?;

                if response.is_success() {
                    // Mark attempt as completed
                    trace!(
                        "[unit={unit_id}] Completing request attempt {}",
                        &attempt.request_attempt_id
                    );
                    query!(
                        "UPDATE webhook.request_attempt SET succeeded_at = statement_timestamp() WHERE request_attempt__id = $1",
                        attempt.request_attempt_id
                    )
                    .execute(&mut *tx)
                    .await?;

                    debug!(
                        "[unit={unit_id}] Request attempt {} was completed sucessfully",
                        &attempt.request_attempt_id
                    );
                } else {
                    // Mark attempt as failed
                    trace!(
                        "[unit={unit_id}] Failing request attempt {}",
                        &attempt.request_attempt_id
                    );
                    query!(
                        "UPDATE webhook.request_attempt SET failed_at = statement_timestamp() WHERE request_attempt__id = $1",
                        attempt.request_attempt_id
                    )
                    .execute(&mut *tx)
                    .await?;

                    // Creating a retry request or giving up
                    if let Some(retry_in) = compute_next_retry(
                        &mut tx,
                        &attempt_with_payload,
                        &response,
                        config.max_fast_retries,
                        config.max_slow_retries,
                    )
                    .await?
                    {
                        let next_retry_count = attempt.retry_count + 1;
                        let retry_id = query!(
                            "
                                INSERT INTO webhook.request_attempt (application__id, event__id, subscription__id, delay_until, retry_count)
                                VALUES ($1, $2, $3, statement_timestamp() + $4, $5)
                                RETURNING request_attempt__id
                            ",
                            attempt.application_id,
                            attempt.event_id,
                            attempt.subscription_id,
                            PgInterval::try_from(retry_in).unwrap(),
                            next_retry_count,
                        )
                        .fetch_one(&mut *tx)
                        .await?
                        .request_attempt__id;

                        debug!(
                            "[unit={unit_id}] Request attempt {} failed; retry #{next_retry_count} created as {retry_id} to be picked in {}s",
                            &attempt.request_attempt_id,
                            &retry_in.as_secs(),
                        );
                    } else {
                        info!(
                            "[unit={unit_id}] Request attempt {} failed after {} attempts; giving up",
                            &attempt.request_attempt_id, &attempt.retry_count,
                        );
                    }
                }

                // Commit transaction
                tx.commit().await?;

                // End OpenTelemetry span
                end_request_attempt_span(span, &response);
            } else {
                warn!("Could not get payload for event {}", attempt.event_id);
                tx.rollback().await?;
            }
        } else {
            trace!("[unit={unit_id}] No unprocessed attempt found");

            // Commit transaction
            tx.commit().await?;

            wait_because_no_work(unit_id).await;
        }

        // Send monitoring heartbeat if necessary
        if let Some(ref tx) = heartbeat_tx {
            tx.send(unit_id).await?;
        }

        if task_tracker.is_closed() {
            break;
        }
    }

    if task_tracker.is_closed() {
        Ok(())
    } else {
        Err(anyhow!("Unit {unit_id} crashed"))
    }
}

async fn wait_because_no_work(unit_id: u16) {
    // In order to reduce load on the database when there is no work to do, but simultaneously keep a low latency when some work becomes available,
    // we wait a variable duration between checks:
    // - for unit 0, we wait for a short duration, so that new work gets picked up fast
    // - for units 1 and 2, we wait for a medium duration
    // - for units > 3, we wait for a long duration, to avoid unnecessary stress on the database
    // Note: units do not wait after finishing a task (they keep going as fast as possible), they wait only if there is no more work to do
    let sleep_duration = match unit_id {
        0 => MIN_POLLING_SLEEP,
        1 | 2 => (MIN_POLLING_SLEEP + MAX_POLLING_SLEEP) / 2,
        _ => MAX_POLLING_SLEEP,
    };
    sleep(sleep_duration).await;
}
