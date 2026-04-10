//! Resolves the event payload for a given event.
//!
//! Payloads live in the DB `event.event.payload` column initially, but large or
//! old payloads are offloaded to S3-compatible object storage.  This module
//! provides two entry points:
//! - `fetch_event_payload`: tries DB first, falls back to S3 — the standard path.
//! - `fetch_s3_event_payload`: S3-only, for callers that already have the DB result.

use aws_sdk_s3::error::DisplayErrorContext;
use chrono::{DateTime, Utc};
use hook0_sentry_integration::log_object_storage_error_with_context;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::ObjectStorageConfig;
use crate::problems::Hook0Problem;

/// Upper bound for the entire S3 round-trip (connect + response + body read).
/// 10s is well above typical S3 latency (<500ms) but short enough that the
/// user gets a fast error when S3 is unreachable.
const S3_TIMEOUT: Duration = Duration::from_secs(10);

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
    // Try the DB inline column first — cheap, no network call.
    let db_payload: Option<Option<Vec<u8>>> = sqlx::query_scalar!(
        "SELECT payload FROM event.event WHERE event__id = $1",
        &event_id,
    )
    .fetch_optional(db)
    .await
    .map_err(Hook0Problem::from)?;

    match db_payload {
        // Payload is inline in the DB — return it directly.
        Some(Some(payload)) => return Ok(Some(payload)),
        // Column is NULL — payload was offloaded to S3 by the retention policy.
        Some(None) => {}
        // Event row not found — should not happen (we just resolved it via
        // request_attempt), indicates a data integrity issue.
        None => return Err(Hook0Problem::NotFound),
    }

    // No S3 configured but the DB column is NULL — the payload is lost.
    let Some(object_storage) = object_storage else {
        return Ok(None);
    };

    Ok(fetch_s3_event_payload(object_storage, application_id, event_id, received_at).await)
}

/// Fetches the event payload directly from S3-compatible object storage.
///
/// Called when the DB `payload` column is NULL (offloaded by retention).
/// Returns `None` if S3 is unreachable, the object is missing, or the timeout
/// expires.
pub async fn fetch_s3_event_payload(
    object_storage: &ObjectStorageConfig,
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
) -> Option<Vec<u8>> {
    let key = format!(
        "{}/event/{}/{event_id}",
        application_id,
        received_at.naive_utc().date(),
    );

    match fetch_from_s3(object_storage, &key).await {
        Ok(bytes) => Some(bytes),
        Err(S3Error::Timeout) => {
            log_object_storage_error_with_context!(
                "S3 GET OBJECT timed out",
                error_chain = format!("no response within {}s", S3_TIMEOUT.as_secs()),
                object_key = &key,
            );
            None
        }
        Err(S3Error::GetObject(error)) => {
            log_object_storage_error_with_context!(
                "S3 GET OBJECT failed",
                error_chain = DisplayErrorContext(&error).to_string(),
                object_key = &key,
            );
            None
        }
        Err(S3Error::BodyCollect(error)) => {
            log_object_storage_error_with_context!(
                "S3 GET OBJECT body collect failed",
                error_chain = format!("{error}"),
                object_key = &key,
            );
            None
        }
    }
}

enum S3Error {
    Timeout,
    GetObject(aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>),
    BodyCollect(aws_sdk_s3::primitives::ByteStreamError),
}

/// Single timeout covering the entire S3 round-trip (request + body download).
async fn fetch_from_s3(
    object_storage: &ObjectStorageConfig,
    key: &str,
) -> Result<Vec<u8>, S3Error> {
    let fut = async {
        let response = object_storage
            .client
            .get_object()
            .bucket(&object_storage.bucket)
            .key(key)
            .send()
            .await
            .map_err(S3Error::GetObject)?;

        response
            .body
            .collect()
            .await
            .map(|aggregated_bytes| aggregated_bytes.to_vec())
            .map_err(S3Error::BodyCollect)
    };

    tokio::time::timeout(S3_TIMEOUT, fut)
        .await
        .map_err(|_| S3Error::Timeout)?
}
