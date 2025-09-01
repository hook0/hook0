use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Row, query, query_scalar};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use crate::iam::{Action, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct OperationalEndpoint {
    pub operational_endpoint_id: Uuid,
    pub application_id: Uuid,
    #[validate(url)]
    pub url: String,
    pub description: Option<String>,
    pub headers: HashMap<String, String>,
    pub secret: Uuid,
    pub is_enabled: bool,
    pub filter_types: Vec<String>,
    pub rate_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct CreateOperationalEndpoint {
    pub application_id: Uuid,
    #[validate(url)]
    pub url: String,
    pub description: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub filter_types: Vec<String>,
    pub rate_limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct UpdateOperationalEndpoint {
    #[validate(url)]
    pub url: Option<String>,
    pub description: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub is_enabled: Option<bool>,
    pub filter_types: Option<Vec<String>>,
    pub rate_limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct OperationalEvent {
    pub operational_event_id: Uuid,
    pub application_id: Uuid,
    pub event_type_name: String,
    pub payload: Value,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct OperationalEventType {
    pub event_type_name: String,
    pub description: String,
    pub schema: Value,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct MessageStats {
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_messages: i32,
    pub successful_messages: i32,
    pub failed_messages: i32,
    pub pending_messages: i32,
    pub avg_delivery_time_ms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub after: Option<Uuid>,
    pub application_id: Option<Uuid>,
}

#[api_v2_operation(
    summary = "List operational webhook endpoints",
    description = "List all operational webhook endpoints for the authenticated user's applications",
    operation_id = "operational_webhooks.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Operational Webhooks")
)]
pub async fn list(
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<sqlx::PgPool>,
    query: Query<ListQuery>,
) -> Result<Json<Vec<OperationalEndpoint>>, Hook0Problem> {
    let limit = query.limit.unwrap_or(100).min(100);

    let mut sql_query = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            operational_endpoint__id as operational_endpoint_id,
            application__id as application_id,
            url,
            description,
            headers,
            secret,
            is_enabled,
            filter_types,
            rate_limit,
            created_at,
            updated_at
        FROM webhook.operational_endpoint
        WHERE deleted_at IS NULL
        "#,
    );

    if let Some(app_id) = query.application_id {
        authorize_for_application(
            db.as_ref(),
            &biscuit,
            Action::OperationalWebhookList {
                application_id: &app_id,
            },
            5000,
        )
        .await
        .map_err(|e| {
            error!("Authorization failed: {e}");
            Hook0Problem::Forbidden
        })?;
        sql_query.push(" AND application__id = ");
        sql_query.push_bind(app_id);
    }

    if let Some(after) = query.after {
        sql_query.push(" AND operational_endpoint__id > ");
        sql_query.push_bind(after);
    }

    sql_query.push(" ORDER BY operational_endpoint__id ASC LIMIT ");
    sql_query.push_bind(limit);

    let endpoints_query = sql_query.build();
    let rows = endpoints_query.fetch_all(db.as_ref()).await.map_err(|e| {
        error!("Failed to list operational endpoints: {e}");
        Hook0Problem::InternalServerError
    })?;

    let endpoints: Vec<OperationalEndpoint> = rows
        .into_iter()
        .map(|row| OperationalEndpoint {
            operational_endpoint_id: row.get("operational_endpoint_id"),
            application_id: row.get("application_id"),
            url: row.get("url"),
            description: row.get("description"),
            headers: serde_json::from_value(row.get("headers")).unwrap_or_default(),
            secret: row.get("secret"),
            is_enabled: row.get("is_enabled"),
            filter_types: serde_json::from_value(row.get("filter_types")).unwrap_or_default(),
            rate_limit: row.get("rate_limit"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(Json(endpoints))
}

#[api_v2_operation(
    summary = "Create operational webhook endpoint",
    description = "Create a new operational webhook endpoint to receive notifications about webhook system events",
    operation_id = "operational_webhooks.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Operational Webhooks")
)]
pub async fn create(
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<sqlx::PgPool>,
    Json(payload): Json<CreateOperationalEndpoint>,
) -> Result<CreatedJson<OperationalEndpoint>, Hook0Problem> {
    payload.validate().map_err(Hook0Problem::Validation)?;

    authorize_for_application(
        db.as_ref(),
        &biscuit,
        Action::OperationalWebhookCreate {
            application_id: &payload.application_id,
        },
        5000,
    )
    .await
    .map_err(|e| {
        error!("Authorization failed: {e}");
        Hook0Problem::Forbidden
    })?;

    let headers_json =
        serde_json::to_value(&payload.headers).map_err(|_| Hook0Problem::InternalServerError)?;

    let filter_types_json = serde_json::to_value(&payload.filter_types)
        .map_err(|_| Hook0Problem::InternalServerError)?;

    let endpoint: OperationalEndpoint = query!(
        r#"
        INSERT INTO webhook.operational_endpoint (
            application__id,
            url,
            description,
            headers,
            filter_types,
            rate_limit
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING 
            operational_endpoint__id,
            application__id,
            url,
            description,
            headers,
            secret,
            is_enabled,
            filter_types,
            rate_limit,
            created_at,
            updated_at
        "#,
        payload.application_id,
        payload.url,
        payload.description,
        headers_json,
        filter_types_json,
        payload.rate_limit
    )
    .fetch_one(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to create operational endpoint: {e}");
        Hook0Problem::InternalServerError
    })
    .map(|row| OperationalEndpoint {
        operational_endpoint_id: row.operational_endpoint__id,
        application_id: row.application__id,
        url: row.url,
        description: row.description,
        headers: serde_json::from_value(row.headers).unwrap_or_default(),
        secret: row.secret,
        is_enabled: row.is_enabled,
        filter_types: serde_json::from_value(row.filter_types).unwrap_or_default(),
        rate_limit: row.rate_limit,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })?;

    Ok(CreatedJson(endpoint))
}

#[api_v2_operation(
    summary = "Get operational webhook endpoint",
    description = "Get details of a specific operational webhook endpoint",
    operation_id = "operational_webhooks.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Operational Webhooks")
)]
pub async fn get(
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<sqlx::PgPool>,
    path: Path<Uuid>,
) -> Result<Json<OperationalEndpoint>, Hook0Problem> {
    let endpoint_id = path.into_inner();

    let row = query!(
        r#"
        SELECT 
            operational_endpoint__id,
            application__id,
            url,
            description,
            headers,
            secret,
            is_enabled,
            filter_types,
            rate_limit,
            created_at,
            updated_at
        FROM webhook.operational_endpoint
        WHERE operational_endpoint__id = $1
          AND deleted_at IS NULL
        "#,
        endpoint_id
    )
    .fetch_optional(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get operational endpoint: {e}");
        Hook0Problem::InternalServerError
    })?
    .ok_or(Hook0Problem::NotFound)?;

    let endpoint = OperationalEndpoint {
        operational_endpoint_id: row.operational_endpoint__id,
        application_id: row.application__id,
        url: row.url,
        description: row.description,
        headers: serde_json::from_value(row.headers).unwrap_or_default(),
        secret: row.secret,
        is_enabled: row.is_enabled,
        filter_types: serde_json::from_value(row.filter_types).unwrap_or_default(),
        rate_limit: row.rate_limit,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    authorize_for_application(
        db.as_ref(),
        &biscuit,
        Action::OperationalWebhookGet {
            application_id: &endpoint.application_id,
        },
        5000,
    )
    .await
    .map_err(|e| {
        error!("Authorization failed: {e}");
        Hook0Problem::Forbidden
    })?;

    Ok(Json(endpoint))
}

#[api_v2_operation(
    summary = "Update operational webhook endpoint",
    description = "Update an existing operational webhook endpoint",
    operation_id = "operational_webhooks.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Operational Webhooks")
)]
pub async fn update(
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<sqlx::PgPool>,
    path: Path<Uuid>,
    Json(payload): Json<UpdateOperationalEndpoint>,
) -> Result<Json<OperationalEndpoint>, Hook0Problem> {
    payload.validate().map_err(Hook0Problem::Validation)?;

    let endpoint_id = path.into_inner();

    // First, get the endpoint to check authorization
    let existing = query_scalar!(
        r#"
        SELECT application__id
        FROM webhook.operational_endpoint
        WHERE operational_endpoint__id = $1
          AND deleted_at IS NULL
        "#,
        endpoint_id
    )
    .fetch_optional(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get operational endpoint: {e}");
        Hook0Problem::InternalServerError
    })?
    .ok_or(Hook0Problem::NotFound)?;

    authorize_for_application(
        db.as_ref(),
        &biscuit,
        Action::OperationalWebhookUpdate {
            application_id: &existing,
        },
        5000,
    )
    .await
    .map_err(|e| {
        error!("Authorization failed: {e}");
        Hook0Problem::Forbidden
    })?;

    // First get current endpoint data
    let current = query!(
        r#"
        SELECT 
            operational_endpoint__id,
            application__id,
            url,
            description,
            headers,
            secret,
            is_enabled,
            filter_types,
            rate_limit,
            created_at,
            updated_at
        FROM webhook.operational_endpoint
        WHERE operational_endpoint__id = $1
          AND deleted_at IS NULL
        "#,
        endpoint_id
    )
    .fetch_one(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get operational endpoint: {e}");
        Hook0Problem::InternalServerError
    })?;

    // Apply updates
    let new_url = payload.url.unwrap_or(current.url);
    let new_description = payload.description.or(current.description);
    let new_headers = if let Some(headers) = payload.headers {
        serde_json::to_value(&headers).map_err(|_| Hook0Problem::InternalServerError)?
    } else {
        current.headers
    };
    let new_is_enabled = payload.is_enabled.unwrap_or(current.is_enabled);
    let new_filter_types = if let Some(filter_types) = payload.filter_types {
        serde_json::to_value(&filter_types).map_err(|_| Hook0Problem::InternalServerError)?
    } else {
        current.filter_types
    };
    let new_rate_limit = payload.rate_limit.or(current.rate_limit);

    // Update the endpoint
    let row = query!(
        r#"
        UPDATE webhook.operational_endpoint
        SET url = $2,
            description = $3,
            headers = $4,
            is_enabled = $5,
            filter_types = $6,
            rate_limit = $7,
            updated_at = statement_timestamp()
        WHERE operational_endpoint__id = $1
        RETURNING 
            operational_endpoint__id,
            application__id,
            url,
            description,
            headers,
            secret,
            is_enabled,
            filter_types,
            rate_limit,
            created_at,
            updated_at
        "#,
        endpoint_id,
        new_url,
        new_description,
        new_headers,
        new_is_enabled,
        new_filter_types,
        new_rate_limit
    )
    .fetch_one(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to update operational endpoint: {e}");
        Hook0Problem::InternalServerError
    })?;

    let endpoint = OperationalEndpoint {
        operational_endpoint_id: row.operational_endpoint__id,
        application_id: row.application__id,
        url: row.url,
        description: row.description,
        headers: serde_json::from_value(row.headers).unwrap_or_default(),
        secret: row.secret,
        is_enabled: row.is_enabled,
        filter_types: serde_json::from_value(row.filter_types).unwrap_or_default(),
        rate_limit: row.rate_limit,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(Json(endpoint))
}

#[api_v2_operation(
    summary = "Delete operational webhook endpoint",
    description = "Delete an operational webhook endpoint",
    operation_id = "operational_webhooks.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Operational Webhooks")
)]
pub async fn delete(
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<sqlx::PgPool>,
    path: Path<Uuid>,
) -> Result<NoContent, Hook0Problem> {
    let endpoint_id = path.into_inner();

    // First, get the endpoint to check authorization
    let existing = query_scalar!(
        r#"
        SELECT application__id
        FROM webhook.operational_endpoint
        WHERE operational_endpoint__id = $1
          AND deleted_at IS NULL
        "#,
        endpoint_id
    )
    .fetch_optional(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get operational endpoint: {e}");
        Hook0Problem::InternalServerError
    })?
    .ok_or(Hook0Problem::NotFound)?;

    authorize_for_application(
        db.as_ref(),
        &biscuit,
        Action::OperationalWebhookDelete {
            application_id: &existing,
        },
        5000,
    )
    .await
    .map_err(|e| {
        error!("Authorization failed: {e}");
        Hook0Problem::Forbidden
    })?;

    query!(
        r#"
        UPDATE webhook.operational_endpoint
        SET deleted_at = statement_timestamp()
        WHERE operational_endpoint__id = $1
        "#,
        endpoint_id
    )
    .execute(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to delete operational endpoint: {e}");
        Hook0Problem::InternalServerError
    })?;

    Ok(NoContent)
}

#[api_v2_operation(
    summary = "List operational event types",
    description = "List all available operational event types",
    operation_id = "operational_webhooks.event_types",
    consumes = "application/json",
    produces = "application/json",
    tags("Operational Webhooks")
)]
pub async fn event_types(
    _: OaBiscuit,
    _biscuit: ReqData<Biscuit>,
    db: Data<sqlx::PgPool>,
) -> Result<Json<Vec<OperationalEventType>>, Hook0Problem> {
    let rows = query!(
        r#"
        SELECT 
            event_type__name,
            description,
            schema
        FROM webhook.operational_event_type
        ORDER BY event_type__name
        "#
    )
    .fetch_all(db.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to list operational event types: {e}");
        Hook0Problem::InternalServerError
    })?;

    let types: Vec<OperationalEventType> = rows
        .into_iter()
        .map(|row| OperationalEventType {
            event_type_name: row.event_type__name,
            description: row.description,
            schema: row.schema,
        })
        .collect();

    Ok(Json(types))
}

#[api_v2_operation(
    summary = "Get message statistics",
    description = "Get delivery statistics for messages",
    operation_id = "operational_webhooks.stats",
    consumes = "application/json",
    produces = "application/json",
    tags("Operational Webhooks")
)]
pub async fn stats(
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<sqlx::PgPool>,
    query: Query<ListQuery>,
) -> Result<Json<Vec<MessageStats>>, Hook0Problem> {
    let limit = query.limit.unwrap_or(100).min(100);

    let mut sql_query = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            application__id as application_id,
            subscription__id as subscription_id,
            period_start,
            period_end,
            total_messages,
            successful_messages,
            failed_messages,
            pending_messages,
            avg_delivery_time_ms
        FROM webhook.message_stats
        WHERE 1=1
        "#,
    );

    if let Some(app_id) = query.application_id {
        authorize_for_application(
            db.as_ref(),
            &biscuit,
            Action::OperationalWebhookStats {
                application_id: &app_id,
            },
            5000,
        )
        .await
        .map_err(|e| {
            error!("Authorization failed: {e}");
            Hook0Problem::Forbidden
        })?;
        sql_query.push(" AND application__id = ");
        sql_query.push_bind(app_id);
    }

    sql_query.push(" ORDER BY period_start DESC LIMIT ");
    sql_query.push_bind(limit);

    let stats_query = sql_query.build();
    let rows = stats_query.fetch_all(db.as_ref()).await.map_err(|e| {
        error!("Failed to get message stats: {e}");
        Hook0Problem::InternalServerError
    })?;

    let stats: Vec<MessageStats> = rows
        .into_iter()
        .map(|row| MessageStats {
            application_id: row.get("application_id"),
            subscription_id: row.get("subscription_id"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            total_messages: row.get("total_messages"),
            successful_messages: row.get("successful_messages"),
            failed_messages: row.get("failed_messages"),
            pending_messages: row.get("pending_messages"),
            avg_delivery_time_ms: row.get("avg_delivery_time_ms"),
        })
        .collect();

    Ok(Json(stats))
}
