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
use tokio::time::timeout;
use uuid::Uuid;

use crate::ObjectStorageConfig;

/// Upper bound for a single S3 GET call (connect + first byte + body read).
/// Keeps the retry handler responsive — if S3 is slow we'd rather return
/// "payload unavailable" than block the request for minutes.  10s is well
/// above typical S3 latency (<500ms) but short enough that the user gets
/// a fast error.
const S3_TIMEOUT: Duration = Duration::from_secs(10);

/// Resolves the original event payload needed to (re-)deliver a webhook.
///
/// Hook0 stores small/recent payloads inline in the DB.  When a retention
/// policy moves them to S3, the DB column becomes NULL.  This function
/// checks the DB first (cheap) then falls back to S3 with a 10-second
/// timeout.  If neither source has the data — the event has been purged
/// or S3 is unreachable — it returns `None`.  Callers should surface
/// `Hook0Problem::EventPayloadUnavailable` so the user knows the retry
/// cannot proceed.
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

    if let Some(os) = object_storage {
        let key = format!(
            "{}/event/{}/{event_id}",
            application_id,
            received_at.naive_utc().date(),
        );
        match timeout(
            S3_TIMEOUT,
            os.client
                .get_object()
                .bucket(&os.bucket)
                .key(&key)
                .send(),
        )
        .await
        {
            Ok(Ok(obj)) => match timeout(S3_TIMEOUT, obj.body.collect()).await {
                Ok(Ok(ab)) => return Some(ab.to_vec()),
                Ok(Err(e)) => {
                    log_object_storage_error_with_context!(
                        "S3 GET OBJECT body collect failed",
                        error_chain = format!("{e}"),
                        object_key = &key,
                    );
                }
                Err(_) => {
                    log_object_storage_error_with_context!(
                        "S3 GET OBJECT body collect timed out",
                        error_chain = "timeout".to_string(),
                        object_key = &key,
                    );
                }
            },
            Ok(Err(e)) => {
                log_object_storage_error_with_context!(
                    "S3 GET OBJECT failed",
                    error_chain = DisplayErrorContext(&e).to_string(),
                    object_key = &key,
                );
            }
            Err(_) => {
                log_object_storage_error_with_context!(
                    "S3 GET OBJECT timed out",
                    error_chain = "timeout".to_string(),
                    object_key = &key,
                );
            }
        }
    }

    None
}
