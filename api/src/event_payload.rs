//! Resolves the event payload for a given event.
//!
//! Payloads live in the DB `event.event.payload` column initially, but large or
//! old payloads are offloaded to S3-compatible object storage.  This module
//! provides two entry points:
//! - `fetch_event_payload`: tries DB first, falls back to S3 — the standard path.
//! - `fetch_s3_event_payload`: S3-only, for callers that already checked the DB.

use aws_sdk_s3::error::DisplayErrorContext;
use chrono::{DateTime, Utc};
use hook0_sentry_integration::log_object_storage_error_with_context;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::ObjectStorageConfig;

/// Upper bound for the entire S3 round-trip (connect + response + body read).
/// 10s is well above typical S3 latency (<500ms) but short enough that the
/// user gets a fast error when S3 is unreachable.
const S3_TIMEOUT: Duration = Duration::from_secs(10);

/// Resolves the event payload by checking the DB inline column first, then
/// falling back to S3 if the column is NULL (payload offloaded by retention).
///
/// Returns `None` when neither source has the data (payload expired / purged).
/// Callers should surface `Hook0Problem::EventPayloadUnavailable`.
pub async fn fetch_event_payload(
    db: &PgPool,
    object_storage: Option<&ObjectStorageConfig>,
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
) -> Option<Vec<u8>> {
    // Try the DB inline column first — cheap, no network call.
    #[allow(non_snake_case)]
    struct EventPayloadRow {
        payload: Option<Vec<u8>>,
    }
    let row = sqlx::query_as!(
        EventPayloadRow,
        "SELECT payload FROM event.event WHERE event__id = $1",
        &event_id,
    )
    .fetch_optional(db)
    .await
    .ok()??;

    if let Some(payload) = row.payload {
        return Some(payload);
    }

    // DB column is NULL — the payload was offloaded to S3 by the retention policy.
    let object_storage = object_storage?;
    fetch_s3_event_payload(object_storage, application_id, event_id, received_at).await
}

/// Fetches the event payload directly from S3-compatible object storage.
///
/// Called when the DB `payload` column is NULL.  Returns `None` if S3 is
/// unreachable, the object is missing, or the timeout expires.
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
