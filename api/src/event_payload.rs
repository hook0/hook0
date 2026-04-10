use aws_sdk_s3::error::DisplayErrorContext;
use chrono::{DateTime, Utc};
use hook0_sentry_integration::log_object_storage_error_with_context;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use crate::ObjectStorageConfig;

const S3_TIMEOUT: Duration = Duration::from_secs(10);

/// Fetch event payload from the DB column or fall back to object storage.
/// Returns `None` if the payload is unavailable (expired or storage error).
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
