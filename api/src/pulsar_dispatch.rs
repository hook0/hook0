//! Publishes a `RequestAttempt` to its target Pulsar topic.
//!
//! This module owns the *protocol* knowledge: topic name format, producer
//! lock acquisition, and timeout policy. Centralizing it here is what lets
//! callers stay agnostic of how Pulsar publishing actually works — and keeps
//! a single place to change if the topic format or timeout ever evolves.
//!
//! Callers are responsible for resolving which `worker_id` to use (single-row
//! query for one-shot dispatches, batch JOIN for the event ingestion path).

use std::sync::Arc;
use std::time::Duration;

use actix_web::rt::time::timeout;
use tracing::error;
use uuid::Uuid;

use crate::PulsarConfig;
use crate::problems::Hook0Problem;
use hook0_protobuf::RequestAttempt;

/// Upper bound on how long we wait to acquire the Pulsar producer lock.
/// If the lock is held for longer than this, the producer is likely stuck
/// or the broker is unreachable — failing fast is better than hanging the
/// caller's request thread.
const PULSAR_PRODUCER_LOCK_TIMEOUT: Duration = Duration::from_secs(3);

/// Sends a single `RequestAttempt` to the Pulsar topic served by `worker_id`.
///
/// The topic name follows the convention `persistent://{tenant}/{namespace}/{worker_id}.request_attempt`,
/// so each worker has its own dedicated topic.
///
/// Errors map to `InternalServerError`: a Pulsar publish failure is an
/// infrastructure problem, not a user-facing condition.
pub async fn publish_attempt(
    pulsar: &Arc<PulsarConfig>,
    worker_id: Uuid,
    attempt: RequestAttempt,
) -> Result<(), Hook0Problem> {
    let mut producer = timeout(
        PULSAR_PRODUCER_LOCK_TIMEOUT,
        pulsar.request_attempts_producer.lock(),
    )
    .await
    .map_err(|_| {
        error!("Timed out while waiting access to Pulsar producer");
        Hook0Problem::InternalServerError
    })?;

    producer
        .send_non_blocking(
            format!(
                "persistent://{}/{}/{}.request_attempt",
                &pulsar.tenant, &pulsar.namespace, worker_id,
            ),
            attempt,
        )
        .await
        .map_err(|error| {
            error!("Failed to send attempt to Pulsar: {error}");
            Hook0Problem::InternalServerError
        })?;

    Ok(())
}
