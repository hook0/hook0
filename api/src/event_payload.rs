//! Fetch event payload from DB or object storage — shared by replay and manual retry handlers.

use aws_sdk_s3::error::DisplayErrorContext;
use chrono::{DateTime, Utc};
use hook0_sentry_integration::log_object_storage_error_with_context;
use uuid::Uuid;

/// Fetch event payload from the DB column or fall back to object storage.
/// Returns None if the payload is unavailable (expired from DB and storage fetch failed or not configured).
pub(crate) async fn fetch_event_payload(
    db_payload: Option<Vec<u8>>,
    object_storage: Option<&crate::ObjectStorageConfig>,
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
) -> Option<Vec<u8>> {
    // If the payload is still in the DB column, use it directly
    if let Some(p) = db_payload {
        return Some(p);
    }

    // Otherwise, try fetching from S3 — the payload may have been offloaded after ingestion
    let os = object_storage?;
    let key = format!(
        "{application_id}/event/{}/{event_id}",
        received_at.naive_utc().date(),
    );

    match os.client.get_object().bucket(&os.bucket).key(&key).send().await {
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
}
