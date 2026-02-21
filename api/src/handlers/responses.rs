use actix_web::web::ReqData;
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::operation::get_object::GetObjectError;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as Base64;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use log::warn;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::query_as;
use std::collections::HashMap;
use uuid::Uuid;

use crate::iam::{Action, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use hook0_protobuf::ObjectStorageResponse;
use hook0_sentry_integration::log_object_storage_error_with_context;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Response {
    pub response_id: Uuid,
    pub response_error_name: Option<String>,
    pub http_code: Option<i16>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub elapsed_time_ms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
}

#[api_v2_operation(
    summary = "Get a response by its ID",
    description = "A response is produced when a request attempt is processed. Response IDs can be obtained from request attempts details.",
    operation_id = "response.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
    response_id: Path<Uuid>,
) -> Result<Json<Response>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::ResponseGet {
            application_id: &qs.application_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    #[allow(non_snake_case)]
    struct RawResponse {
        response__id: Uuid,
        response_error__name: Option<String>,
        http_code: Option<i16>,
        headers: Option<Value>,
        body: Option<Vec<u8>>,
        elapsed_time_ms: Option<i32>,
        request_attempt_created_at: DateTime<Utc>,
    }

    let raw_response = query_as!(
        RawResponse,
        "
            SELECT
                r.response__id,
                r.response_error__name,
                r.http_code,
                r.headers,
                r.body,
                r.elapsed_time_ms,
                ra.created_at as request_attempt_created_at
            FROM webhook.response AS r
            INNER JOIN webhook.request_attempt AS ra ON ra.response__id = r.response__id
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            WHERE s.application__id = $1 AND r.response__id = $2
        ",
        &qs.application_id,
        &response_id.into_inner(),
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if let Some(rr) = raw_response {
        let object_storage_response = if let Some(object_storage) = &state.object_storage {
            let key = format!(
                "{}/response/{}/{}",
                qs.application_id,
                rr.request_attempt_created_at.naive_utc().date(),
                rr.response__id,
            );
            let payload_object = object_storage
                .client
                .get_object()
                .bucket(&object_storage.bucket)
                .key(&key)
                .send()
                .await;
            match payload_object {
                Ok(object) => object
                    .body
                    .collect()
                    .await
                    .map_err(|e| {
                        log_object_storage_error_with_context!(
                            "S3 GET OBJECT body collect failed",
                            error_chain = format!("{e}"),
                            object_key = &key,
                        );
                        Hook0Problem::InternalServerError
                    })
                    .and_then(|p| {
                        let res: Result<Option<ObjectStorageResponse>, _> =
                            p.to_vec().as_slice().try_into().map(Some).map_err(|e| {
                                log_object_storage_error_with_context!(
                                    "S3 response decode failed",
                                    error_chain = format!("{e}"),
                                    object_key = &key,
                                );
                                Hook0Problem::InternalServerError
                            });
                        res
                    }),
                Err(ref e)
                    if matches!(e.as_service_error(), Some(GetObjectError::NoSuchKey(_))) =>
                {
                    Ok(None)
                }
                Err(e) => {
                    log_object_storage_error_with_context!(
                        "S3 GET OBJECT failed",
                        error_chain = DisplayErrorContext(&e).to_string(),
                        object_key = &key,
                    );
                    Err(Hook0Problem::InternalServerError)
                }
            }
        } else {
            Ok(None)
        }?;

        let (body_bytes, raw_headers) = if let Some(os_response) = object_storage_response {
            (Some(os_response.body), Some(os_response.headers))
        } else {
            (rr.body, rr.headers)
        };

        let headers = match raw_headers {
            Some(Value::Null) => Some(HashMap::new()),
            Some(h) => match serde_json::from_value(h) {
                Ok(hashmap) => Some(hashmap),
                Err(e) => {
                    warn!("Could not deserialize response headers: {e}");
                    Some(HashMap::new())
                }
            },
            None => None,
        };
        let body = body_bytes.map(|bytes| {
            String::from_utf8(bytes.to_owned()).unwrap_or_else(|_| Base64.encode(bytes))
        });

        Ok(Json(Response {
            response_id: rr.response__id,
            response_error_name: rr.response_error__name,
            http_code: rr.http_code,
            headers,
            body,
            elapsed_time_ms: rr.elapsed_time_ms,
        }))
    } else {
        Err(Hook0Problem::NotFound)
    }
}
