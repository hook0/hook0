use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::audit;
use crate::relay::is_valid_token;
use crate::storage::{StoredWebhook, TokenSession};
use crate::AppState;

#[derive(Serialize)]
struct WebhookListResponse {
    token: String,
    session: TokenSession,
    webhooks: Vec<StoredWebhook>,
    webhook_url: String,
    view_url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

/// Get all webhooks for a token
pub async fn get_webhooks(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    if !is_valid_token(&token) {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "invalid_token".to_string(),
                message: "Token not found or invalid format".to_string(),
            }),
        )
            .into_response();
    }

    audit::log_audit(
        audit::AuditEvent::InspectionApiAccess,
        Some(&token),
        None,
        "get_webhooks",
    );

    let session = state.storage.get_or_create_session(&token);
    let webhooks = state.storage.get_webhooks(&token);

    (
        StatusCode::OK,
        Json(WebhookListResponse {
            token: token.clone(),
            session,
            webhooks,
            webhook_url: format!("{}/in/{}/", state.base_url, token),
            view_url: format!("{}/view/{}", state.base_url, token),
        }),
    )
        .into_response()
}

/// Get a specific webhook by ID
pub async fn get_webhook(
    State(state): State<Arc<AppState>>,
    Path((token, webhook_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if !is_valid_token(&token) {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "invalid_token".to_string(),
                message: "Token not found or invalid format".to_string(),
            }),
        )
            .into_response();
    }

    audit::log_audit(
        audit::AuditEvent::InspectionApiAccess,
        Some(&token),
        None,
        &format!("get_webhook:{}", webhook_id),
    );

    match state.storage.get_webhook(&token, &webhook_id) {
        Some(webhook) => (StatusCode::OK, Json(webhook)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Webhook {} not found", webhook_id),
            }),
        )
            .into_response(),
    }
}

/// Get session information for a token
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    if !is_valid_token(&token) {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "invalid_token".to_string(),
                message: "Token not found or invalid format".to_string(),
            }),
        )
            .into_response();
    }

    audit::log_audit(
        audit::AuditEvent::InspectionApiAccess,
        Some(&token),
        None,
        "get_session",
    );

    let session = state.storage.get_or_create_session(&token);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "session": session,
            "webhook_url": format!("{}/in/{}/", state.base_url, token),
            "view_url": format!("{}/view/{}", state.base_url, token)
        })),
    )
        .into_response()
}

/// Delete a specific webhook
pub async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Path((token, webhook_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if !is_valid_token(&token) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "invalid_token",
                "message": "Token not found or invalid format"
            })),
        )
            .into_response();
    }

    if state.storage.delete_webhook(&token, &webhook_id) {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "deleted",
                "webhook_id": webhook_id
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "not_found",
                "message": format!("Webhook {} not found", webhook_id)
            })),
        )
            .into_response()
    }
}

/// Delete all webhooks for a token
pub async fn delete_all_webhooks(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    if !is_valid_token(&token) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "invalid_token",
                "message": "Token not found or invalid format"
            })),
        );
    }

    let count = state.storage.delete_all_webhooks(&token);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "deleted",
            "count": count
        })),
    )
}
