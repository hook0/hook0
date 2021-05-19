use actix_web_middleware_keycloak_auth::UnstructuredClaims;
use log::warn;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query, ReqData},
    Apiv2Schema,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::query_as;
use std::collections::HashMap;
use uuid::Uuid;

use crate::iam::{can_access_application, Role};
use crate::problems::Hook0Problem;

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
    summary = "Get a response by its id",
    description = "A response is produced when a request attempt is processed",
    operation_id = "response.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn get(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    qs: Query<Qs>,
    response_id: Path<Uuid>,
) -> Result<Json<Response>, Hook0Problem> {
    if !can_access_application(
        &state.db,
        &unstructured_claims,
        &qs.application_id,
        &Role::Viewer,
    )
    .await
    {
        return Err(Hook0Problem::Forbidden);
    }

    #[allow(non_snake_case)]
    struct RawResponse {
        response__id: Uuid,
        response_error__name: Option<String>,
        http_code: Option<i16>,
        headers: Option<Value>,
        body: Option<String>,
        elapsed_time_ms: Option<i32>,
    }

    let raw_response = query_as!(
        RawResponse,
        "
            SELECT r.response__id, r.response_error__name, r.http_code, r.headers, r.body, r.elapsed_time_ms
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

    let response = raw_response.map(|rr| {
        let headers = match rr.headers {
            Some(h) => match serde_json::from_value(h) {
                Ok(hashmap) => Some(hashmap),
                Err(e) => {
                    warn!("Could not deserialize response headers: {}", &e);
                    Some(HashMap::new())
                }
            },
            None => None,
        };

        Response {
            response_id: rr.response__id,
            response_error_name: rr.response_error__name,
            http_code: rr.http_code,
            headers,
            body: rr.body,
            elapsed_time_ms: rr.elapsed_time_ms,
        }
    });

    match response {
        Some(r) => Ok(Json(r)),
        None => Err(Hook0Problem::NotFound),
    }
}
