use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::audit;
use crate::relay::{is_valid_token, ServerMessage};
use crate::sanitize;
use crate::storage::WebhookStorage;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct WebhookQuery {
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// Receive a webhook and either:
/// 1. Forward to connected CLI client via WebSocket
/// 2. Store for later inspection if no client connected
pub async fn webhook_receiver(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<WebhookQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    // Check if IP is blocked due to invalid token attempts
    if !state.invalid_token_tracker.check_allowed(&ip) {
        audit::log_audit(
            audit::AuditEvent::InvalidTokenAttempt,
            Some(&token),
            Some(&ip),
            "blocked",
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "too_many_invalid_attempts",
                "message": "Too many invalid token attempts. Temporarily blocked."
            })),
        )
            .into_response();
    }

    // Validate token format
    if !is_valid_token(&token) {
        state.invalid_token_tracker.record_invalid(&ip);
        audit::log_audit(
            audit::AuditEvent::InvalidTokenAttempt,
            Some(&token),
            Some(&ip),
            "invalid_format",
        );
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "invalid_token",
                "message": "Token not found or invalid format"
            })),
        )
            .into_response();
    }

    // Rate limiting: per-IP
    if let Err(retry_after) = state.rate_limiter_ip.check(&ip) {
        audit::log_audit(
            audit::AuditEvent::RateLimited,
            Some(&token),
            Some(&ip),
            &format!("per_ip, retry_after={}s", retry_after),
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "rate_limited",
                "message": format!("Rate limit exceeded. Retry after {} seconds", retry_after),
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    // Rate limiting: per-token
    if let Err(retry_after) = state.rate_limiter_token.check(&token) {
        audit::log_audit(
            audit::AuditEvent::RateLimited,
            Some(&token),
            Some(&ip),
            &format!("per_token, retry_after={}s", retry_after),
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "rate_limited",
                "message": format!("Rate limit exceeded. Retry after {} seconds", retry_after),
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    // Rate limiting: global
    if let Err(retry_after) = state.rate_limiter_global.check("global") {
        audit::log_audit(
            audit::AuditEvent::RateLimited,
            Some(&token),
            Some(&ip),
            &format!("global, retry_after={}s", retry_after),
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "rate_limited",
                "message": format!("Rate limit exceeded. Retry after {} seconds", retry_after),
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    // Check payload size limit
    if body.len() > state.limits.max_payload_size {
        audit::log_audit(
            audit::AuditEvent::PayloadTooLarge,
            Some(&token),
            Some(&ip),
            &format!("{} bytes", body.len()),
        );
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": "payload_too_large",
                "message": format!("Payload size {} bytes exceeds maximum of {} bytes",
                    body.len(), state.limits.max_payload_size)
            })),
        )
            .into_response();
    }

    // Convert headers to HashMap and sanitize
    let raw_headers: HashMap<String, String> = headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
        .collect();

    let headers_map = match sanitize::sanitize_headers(raw_headers) {
        Ok(h) => h,
        Err(e) => {
            audit::log_audit(
                audit::AuditEvent::HeaderSanitizationFailed,
                Some(&token),
                Some(&ip),
                &e.to_string(),
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_headers",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
    };

    // Build query string
    let query_string = if query.extra.is_empty() {
        None
    } else {
        Some(
            query
                .extra
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"),
        )
    };

    process_webhook(state, &token, method, "/", query_string, headers_map, body).await
}

/// Receive webhook with additional path segments
pub async fn webhook_receiver_with_path(
    State(state): State<Arc<AppState>>,
    Path((token, path)): Path<(String, String)>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<WebhookQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    // Check if IP is blocked due to invalid token attempts
    if !state.invalid_token_tracker.check_allowed(&ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "too_many_invalid_attempts",
                "message": "Too many invalid token attempts. Temporarily blocked."
            })),
        )
            .into_response();
    }

    // Validate token format
    if !is_valid_token(&token) {
        state.invalid_token_tracker.record_invalid(&ip);
        audit::log_audit(
            audit::AuditEvent::InvalidTokenAttempt,
            Some(&token),
            Some(&ip),
            "invalid_format",
        );
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "invalid_token",
                "message": "Token not found or invalid format"
            })),
        )
            .into_response();
    }

    // Rate limiting: per-IP
    if let Err(retry_after) = state.rate_limiter_ip.check(&ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "rate_limited",
                "message": format!("Rate limit exceeded. Retry after {} seconds", retry_after),
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    // Rate limiting: per-token
    if let Err(retry_after) = state.rate_limiter_token.check(&token) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "rate_limited",
                "message": format!("Rate limit exceeded. Retry after {} seconds", retry_after),
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    // Rate limiting: global
    if let Err(retry_after) = state.rate_limiter_global.check("global") {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "rate_limited",
                "message": format!("Rate limit exceeded. Retry after {} seconds", retry_after),
                "retry_after": retry_after
            })),
        )
            .into_response();
    }

    // Check payload size limit
    if body.len() > state.limits.max_payload_size {
        audit::log_audit(
            audit::AuditEvent::PayloadTooLarge,
            Some(&token),
            Some(&ip),
            &format!("{} bytes", body.len()),
        );
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": "payload_too_large",
                "message": format!("Payload size {} bytes exceeds maximum of {} bytes",
                    body.len(), state.limits.max_payload_size)
            })),
        )
            .into_response();
    }

    // Convert headers to HashMap and sanitize
    let raw_headers: HashMap<String, String> = headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
        .collect();

    let headers_map = match sanitize::sanitize_headers(raw_headers) {
        Ok(h) => h,
        Err(e) => {
            audit::log_audit(
                audit::AuditEvent::HeaderSanitizationFailed,
                Some(&token),
                Some(&ip),
                &e.to_string(),
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_headers",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
    };

    // Build query string
    let query_string = if query.extra.is_empty() {
        None
    } else {
        Some(
            query
                .extra
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"),
        )
    };

    let full_path = format!("/{}", path);
    process_webhook(
        state,
        &token,
        method,
        &full_path,
        query_string,
        headers_map,
        body,
    )
    .await
}

/// Common webhook processing logic
async fn process_webhook(
    state: Arc<AppState>,
    token: &str,
    method: Method,
    path: &str,
    query_string: Option<String>,
    headers_map: HashMap<String, String>,
    body: Bytes,
) -> axum::response::Response {
    // Create webhook record
    let webhook = WebhookStorage::create_webhook(
        token,
        method.as_str(),
        path,
        query_string.as_deref(),
        headers_map.clone(),
        body.to_vec(),
    );

    let webhook_id = webhook.id.clone();

    // Store the webhook
    state.storage.store_webhook(webhook.clone());

    audit::log_audit(
        audit::AuditEvent::WebhookReceived,
        Some(token),
        None,
        &format!("{} {}", method, path),
    );

    // Check if there's a connected client for this token
    if let Some(sender) = state.connections.get(token) {
        // Mark as forwarded
        state.storage.mark_forwarded(token, &webhook_id);

        // Create the request message
        let request_msg = ServerMessage::request(
            webhook_id.clone(),
            method.to_string(),
            path.to_string(),
            body.to_vec(),
            headers_map,
            query_string,
        );

        // Send to connected client
        let msg_json = serde_json::to_string(&request_msg).unwrap_or_default();

        if sender.send(msg_json).await.is_ok() {
            audit::log_audit(
                audit::AuditEvent::WebhookForwarded,
                Some(token),
                None,
                &webhook_id,
            );

            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": webhook_id,
                    "status": "forwarded",
                    "message": "Webhook forwarded to connected client"
                })),
            )
                .into_response();
        }
    }

    // No connected client - store only
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id": webhook_id,
            "status": "stored",
            "message": "Webhook stored. No client connected.",
            "view_url": format!("{}/view/{}", state.base_url, token)
        })),
    )
        .into_response()
}
