//! Resolves the event payload for a given event.
//!
//! Payloads live in the DB `event.event.payload` column initially, but large or
//! old payloads are offloaded to S3-compatible object storage.  This module
//! tries the DB first; if the column is NULL it falls back to object storage.
//! Returns `None` when neither source has the data (payload expired / purged),
//! which the caller should treat as a user-visible error (410 Gone).

use aws_sdk_s3::error::DisplayErrorContext;
use chrono::{DateTime, Utc};
use hook0_sentry_integration::log_object_storage_error_with_context;
use std::time::Duration;
use uuid::Uuid;

use crate::ObjectStorageConfig;

/// Upper bound for the entire S3 round-trip (connect + response + body read).
/// 10s is well above typical S3 latency (<500ms) but short enough that the
/// user gets a fast error when S3 is unreachable.
const S3_TIMEOUT: Duration = Duration::from_secs(10);

/// Resolves the original event payload needed to (re-)deliver a webhook.
///
/// Checks the DB first (cheap), then falls back to S3 with a timeout.
/// Returns `None` when the payload is unavailable — callers should surface
/// `Hook0Problem::EventPayloadUnavailable`.
pub async fn fetch_event_payload(
    db_payload: Option<Vec<u8>>,
    object_storage: Option<&ObjectStorageConfig>,
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
) -> Option<Vec<u8>> {
    if let Some(p) = db_payload {
        return Some(p);
    }

    let os = object_storage?;
    let key = format!(
        "{}/event/{}/{event_id}",
        application_id,
        received_at.naive_utc().date(),
    );

    match fetch_from_s3(os, &key).await {
        Ok(bytes) => Some(bytes),
        Err(S3Error::Timeout) => {
            log_object_storage_error_with_context!(
                "S3 GET OBJECT timed out",
                error_chain = format!("no response within {}s", S3_TIMEOUT.as_secs()),
                object_key = &key,
            );
            None
        }
        Err(S3Error::GetObject(e)) => {
            log_object_storage_error_with_context!(
                "S3 GET OBJECT failed",
                error_chain = DisplayErrorContext(&e).to_string(),
                object_key = &key,
            );
            None
        }
        Err(S3Error::BodyCollect(e)) => {
            log_object_storage_error_with_context!(
                "S3 GET OBJECT body collect failed",
                error_chain = format!("{e}"),
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

/// Fetches an object from S3 with a single timeout covering the entire
/// round-trip (request + body download).
async fn fetch_from_s3(os: &ObjectStorageConfig, key: &str) -> Result<Vec<u8>, S3Error> {
    let fut = async {
        let obj = os
            .client
            .get_object()
            .bucket(&os.bucket)
            .key(key)
            .send()
            .await
            .map_err(S3Error::GetObject)?;

        obj.body
            .collect()
            .await
            .map(|ab| ab.to_vec())
            .map_err(S3Error::BodyCollect)
    };

    tokio::time::timeout(S3_TIMEOUT, fut)
        .await
        .map_err(|_| S3Error::Timeout)?
}
