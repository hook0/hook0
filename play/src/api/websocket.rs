use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::audit;
use crate::relay::{is_valid_token, ClientMessage, ServerMessage};
use crate::sanitize;
use crate::storage::StoredResponse;
use crate::AppState;

/// WebSocket upgrade handler for CLI connections
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    // Check connection limits before upgrading
    if let Err(e) = state.can_accept_connection(&ip) {
        audit::log_audit(
            audit::AuditEvent::ConnectionRejected,
            None,
            Some(&ip),
            &e.to_string(),
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "connection_limit_exceeded",
                "message": e.to_string()
            })),
        )
            .into_response();
    }

    // Register the connection
    state.register_connection(&ip);

    ws.on_upgrade(move |socket| handle_socket(socket, state, ip))
        .into_response()
}

/// Handle a WebSocket connection from a CLI client
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, client_ip: String) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Channel for sending messages to this client
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Track the token for this connection
    let mut connected_token: Option<String> = None;

    // Connection start time for session timeout
    let connection_start = tokio::time::Instant::now();

    // Handshake timeout: must receive Start message within this time
    let handshake_timeout = state.limits.handshake_timeout;
    let session_timeout = state.limits.session_timeout;
    let idle_timeout = state.limits.idle_timeout;

    // Spawn task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Track last activity for idle timeout
    let mut last_activity = tokio::time::Instant::now();
    let mut handshake_completed = false;

    // Process incoming messages with timeout checks
    loop {
        // Calculate the next timeout to apply
        let timeout_duration = if !handshake_completed {
            handshake_timeout
        } else {
            // Check session timeout
            let elapsed_since_start = connection_start.elapsed();
            if elapsed_since_start >= session_timeout {
                if let Some(ref token) = connected_token {
                    audit::log_audit(
                        audit::AuditEvent::SessionTimedOut,
                        Some(token),
                        Some(&client_ip),
                        &format!("after {}s", elapsed_since_start.as_secs()),
                    );
                    let error = ServerMessage::error(
                        "session_timeout",
                        "Maximum session duration exceeded",
                    );
                    let _ = tx
                        .send(serde_json::to_string(&error).unwrap_or_default())
                        .await;
                }
                break;
            }

            // Check idle timeout
            let idle_elapsed = last_activity.elapsed();
            if idle_elapsed >= idle_timeout {
                if let Some(ref token) = connected_token {
                    audit::log_audit(
                        audit::AuditEvent::SessionIdleTimeout,
                        Some(token),
                        Some(&client_ip),
                        &format!("idle for {}s", idle_elapsed.as_secs()),
                    );
                    let error =
                        ServerMessage::error("idle_timeout", "Connection idle for too long");
                    let _ = tx
                        .send(serde_json::to_string(&error).unwrap_or_default())
                        .await;
                }
                break;
            }

            // Use the smaller of remaining session time or idle timeout
            let remaining_session = session_timeout - elapsed_since_start;
            let remaining_idle = idle_timeout;
            remaining_session.min(remaining_idle)
        };

        let msg_result = tokio::time::timeout(timeout_duration, ws_receiver.next()).await;

        let msg = match msg_result {
            Ok(Some(result)) => {
                last_activity = tokio::time::Instant::now();
                result
            }
            Ok(None) => break, // Stream ended
            Err(_) => {
                // Timeout
                if !handshake_completed {
                    warn!("WebSocket handshake timeout for IP {}", client_ip);
                    let error = ServerMessage::error(
                        "handshake_timeout",
                        "Handshake not completed in time",
                    );
                    let _ = tx
                        .send(serde_json::to_string(&error).unwrap_or_default())
                        .await;
                }
                // Session/idle timeouts are already handled above
                break;
            }
        };

        let msg = match msg {
            Ok(Message::Text(text)) => text.to_string(),
            Ok(Message::Binary(data)) => match String::from_utf8(data.to_vec()) {
                Ok(s) => s,
                Err(_) => continue,
            },
            Ok(Message::Ping(_)) => {
                // Pings are handled automatically by axum
                continue;
            }
            Ok(Message::Pong(_)) => continue,
            Ok(Message::Close(_)) => break,
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        };

        // Parse the message
        let client_msg: ClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                warn!("Invalid message format: {}", e);
                let error = ServerMessage::error("invalid_message", "Could not parse message");
                let _ = tx
                    .send(serde_json::to_string(&error).unwrap_or_default())
                    .await;
                continue;
            }
        };

        match client_msg {
            ClientMessage::Start { data, .. } => {
                let token = data.token;

                // Validate token
                if !is_valid_token(&token) {
                    let error = ServerMessage::error("invalid_token", "Invalid token format");
                    let _ = tx
                        .send(serde_json::to_string(&error).unwrap_or_default())
                        .await;
                    continue;
                }

                // Check if token is already in use
                if state.storage.is_connected(&token) {
                    let error =
                        ServerMessage::error("token_in_use", "This token is already connected");
                    let _ = tx
                        .send(serde_json::to_string(&error).unwrap_or_default())
                        .await;
                    continue;
                }

                // Register the connection
                state.connections.insert(token.clone(), tx.clone());
                state.storage.set_connected(&token, Some(client_ip.clone()));
                connected_token = Some(token.clone());
                handshake_completed = true;

                info!("Client connected with token: {}", token);

                // Send confirmation
                let webhook_url = format!("{}/in/{}/", state.base_url, token);
                let view_url = format!("{}/view/{}", state.base_url, token);
                let started = ServerMessage::started(webhook_url, view_url);
                let _ = tx
                    .send(serde_json::to_string(&started).unwrap_or_default())
                    .await;
            }

            ClientMessage::Response { data, .. } => {
                if let Some(ref token) = connected_token {
                    // Validate status code
                    if !sanitize::is_valid_status_code(data.status) {
                        audit::log_audit(
                            audit::AuditEvent::ResponseValidationFailed,
                            Some(token),
                            Some(&client_ip),
                            &format!("invalid status code: {}", data.status),
                        );
                        let error = ServerMessage::error(
                            "invalid_status_code",
                            &format!("Invalid HTTP status code: {}", data.status),
                        );
                        let _ = tx
                            .send(serde_json::to_string(&error).unwrap_or_default())
                            .await;
                        continue;
                    }

                    // Check response body size
                    let body_bytes = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        &data.body,
                    )
                    .unwrap_or_default();

                    if body_bytes.len() > state.limits.max_response_body_size {
                        audit::log_audit(
                            audit::AuditEvent::ResponseValidationFailed,
                            Some(token),
                            Some(&client_ip),
                            &format!(
                                "response too large: {} bytes (max: {})",
                                body_bytes.len(),
                                state.limits.max_response_body_size
                            ),
                        );
                        let error = ServerMessage::error(
                            "response_too_large",
                            &format!(
                                "Response body too large: {} bytes (max: {} bytes)",
                                body_bytes.len(),
                                state.limits.max_response_body_size
                            ),
                        );
                        let _ = tx
                            .send(serde_json::to_string(&error).unwrap_or_default())
                            .await;
                        continue;
                    }

                    // Store the response
                    let response = StoredResponse {
                        status: data.status,
                        headers: data.headers,
                        body: data.body.clone(),
                        body_size: body_bytes.len(),
                        received_at: chrono::Utc::now(),
                        response_time_ms: 0,
                    };

                    state
                        .storage
                        .update_webhook_response(token, &data.id, response);
                }
            }

            ClientMessage::Ping => {
                let pong = ServerMessage::pong();
                let _ = tx
                    .send(serde_json::to_string(&pong).unwrap_or_default())
                    .await;
            }
        }
    }

    // Cleanup on disconnect
    if let Some(token) = connected_token {
        info!("Client disconnected: {}", token);
        state.connections.remove(&token);
        state.storage.set_disconnected(&token);
    }

    // Unregister the connection (always, regardless of token)
    state.unregister_connection(&client_ip);

    // Cancel the send task
    send_task.abort();
}
