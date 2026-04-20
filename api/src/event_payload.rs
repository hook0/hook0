//! Resolves the event payload for a given event.
//!
//! Payloads live in the DB `event.event.payload` column initially, but large or
//! old payloads are offloaded to S3-compatible object storage.  This module
//! provides two entry points:
//! - `fetch_event_payload`: tries DB first, falls back to S3 — the standard path.
//! - `fetch_s3_event_payload`: S3-only, for callers that already have the DB result.

use aws_sdk_s3::error::DisplayErrorContext;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query_scalar};
use uuid::Uuid;

use crate::ObjectStorageConfig;
use crate::problems::Hook0Problem;
use hook0_sentry_integration::log_object_storage_error_with_context;

/// Resolves the event payload by checking the DB inline column first, then
/// falling back to S3 if the column is NULL (payload offloaded by retention).
///
/// Returns `Ok(None)` when the payload is genuinely unavailable (S3 miss,
/// expired).  Returns `Err` on technical failures (DB down, missing event).
pub async fn fetch_event_payload(
    db: &PgPool,
    object_storage: Option<&ObjectStorageConfig>,
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
) -> Result<Option<Vec<u8>>, Hook0Problem> {
    // Try the DB inline column first — cheap.
    let db_payload: Option<Option<Vec<u8>>> = query_scalar!(
        "SELECT payload FROM event.event WHERE event__id = $1",
        &event_id,
    )
    .fetch_optional(db)
    .await
    .map_err(Hook0Problem::from)?;

    match db_payload {
        // Payload is inline in the DB — return it directly.
        Some(Some(payload)) => Ok(Some(payload)),
        // Column is NULL — payload was offloaded to S3 by the retention policy.
        Some(None) => match object_storage {
            // No S3 configured but the DB column is NULL — the payload is lost.
            None => Ok(None),
            Some(object_storage) => {
                Ok(
                    fetch_s3_event_payload(object_storage, application_id, event_id, received_at)
                        .await,
                )
            }
        },
        // Event row not found — should not happen (we just resolved it via
        // request_attempt), indicates a data integrity issue.
        None => Err(Hook0Problem::NotFound),
    }
}

/// Fetches the event payload directly from S3-compatible object storage.
///
/// Called when the DB `payload` column is NULL.
/// Returns `None` if S3 is unreachable, the object is missing, or an SDK-level
/// timeout fires (after emitting a log message and a Sentry error).
pub async fn fetch_s3_event_payload(
    object_storage: &ObjectStorageConfig,
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
) -> Option<Vec<u8>> {
    let key = format!(
        "{application_id}/event/{}/{event_id}",
        received_at.naive_utc().date(),
    );

    match object_storage
        .client
        .get_object()
        .bucket(&object_storage.bucket)
        .key(&key)
        .send()
        .await
    {
        Ok(response) => match response.body.collect().await {
            Ok(aggregated_bytes) => Some(aggregated_bytes.to_vec()),
            Err(error) => {
                log_object_storage_error_with_context!(
                    "S3 GET OBJECT body collect failed",
                    error_chain = format!("{error}"),
                    object_key = &key,
                );
                None
            }
        },
        Err(error) => {
            log_object_storage_error_with_context!(
                "S3 GET OBJECT failed",
                error_chain = DisplayErrorContext(&error).to_string(),
                object_key = &key,
            );
            None
        }
    }
}
