//! Retrieves event payloads from the database or object storage (S3).
//!
//! Events may have their payload stored inline in the DB (`event.payload`) or offloaded
//! to object storage. This module provides a single entry point that checks both locations,
//! falling back to S3 when the DB column is NULL.

use aws_sdk_s3::error::DisplayErrorContext;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::ObjectStorageConfig;
use hook0_sentry_integration::log_object_storage_error_with_context;

/// Fetch an event's payload bytes — first from the DB inline column, then from object storage.
/// Returns `None` if the payload is unavailable in both locations (e.g., expired/purged).
pub async fn fetch_event_payload(
    db_payload: Option<Vec<u8>>,
    object_storage: Option<&ObjectStorageConfig>,
    application_id: &Uuid,
    received_at: &DateTime<Utc>,
    event_id: &Uuid,
) -> Option<Vec<u8>> {
    // Fast path: payload stored inline in the event row
    if let Some(payload) = db_payload {
        return Some(payload);
    }

    // Slow path: attempt to retrieve from object storage (S3)
    let storage = object_storage?;
    let key = format!(
        "{}/event/{}/{event_id}",
        application_id,
        received_at.naive_utc().date(),
    );

    match storage
        .client
        .get_object()
        .bucket(&storage.bucket)
        .key(&key)
        .send()
        .await
    {
        Ok(response) => match response.body.collect().await {
            Ok(aggregated_bytes) => Some(aggregated_bytes.to_vec()),
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
}
