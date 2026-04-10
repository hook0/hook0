//! Dispatches individual request attempts to Pulsar topics.
//!
//! Resolves the worker routing (subscription → worker → topic) and publishes
//! the message.  Best-effort: if Pulsar is unavailable or no Pulsar worker is
//! configured for the subscription, the PG poller picks up the attempt instead.

use std::sync::Arc;
use std::time::Duration;

use actix_web::rt::time::timeout;
use sqlx::query_as;
use tracing::error;
use uuid::Uuid;

use crate::PulsarConfig;
use crate::problems::Hook0Problem;
use hook0_protobuf::RequestAttempt;

/// Timeout for acquiring the Pulsar producer Mutex lock.  If the lock is held
/// longer than 3s, the producer is likely stuck and we should fail fast rather
/// than block the request thread.
const PULSAR_PRODUCER_LOCK_TIMEOUT: Duration = Duration::from_secs(3);

/// Sends a single pre-built `RequestAttempt` to the appropriate Pulsar topic.
///
/// Resolves the worker routing (subscription → worker → Pulsar topic) via a
/// DB query, then publishes the message.  Returns `Ok(())` even if no Pulsar
/// worker is configured — the PG poller will find the un-finalized row.
pub async fn send_single_attempt_to_pulsar(
    db: &sqlx::PgPool,
    pulsar: &Arc<PulsarConfig>,
    attempt: RequestAttempt,
) -> Result<(), Hook0Problem> {
    #[allow(non_snake_case)]
    struct WorkerRoute {
        worker_id: Option<Uuid>,
        worker_queue_type: Option<String>,
    }

    let route = query_as!(
        WorkerRoute,
        "
            SELECT
                COALESCE(sw.worker__id, ow.worker__id) AS worker_id,
                COALESCE(w1.queue_type, w2.queue_type) AS worker_queue_type
            FROM webhook.subscription AS s
            INNER JOIN event.application AS a ON a.application__id = s.application__id
            LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
            LEFT JOIN infrastructure.worker AS w1 ON w1.worker__id = sw.worker__id
            LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = a.organization__id AND ow.default = true
            LEFT JOIN infrastructure.worker AS w2 ON w2.worker__id = ow.worker__id
            WHERE s.subscription__id = $1
        ",
        &attempt.subscription_id,
    )
    .fetch_optional(db)
    .await
    .map_err(Hook0Problem::from)?;

    if let Some(route) = route
        && let Some(worker_id) = route.worker_id
        && route.worker_queue_type.as_deref() == Some("pulsar")
    {
        let mut producer = timeout(
            PULSAR_PRODUCER_LOCK_TIMEOUT,
            pulsar.request_attempts_producer.lock(),
        )
        .await
        .map_err(|_| {
            error!("Timed out while waiting access to Pulsar producer");
            Hook0Problem::InternalServerError
        })?;

        // Best-effort: if send fails, the PG poller will find the row.
        if let Err(error) = producer
            .send_non_blocking(
                format!(
                    "persistent://{}/{}/{}.request_attempt",
                    &pulsar.tenant, &pulsar.namespace, worker_id,
                ),
                attempt,
            )
            .await
        {
            error!("Failed to send attempt to Pulsar: {error}");
        }
    }

    Ok(())
}
