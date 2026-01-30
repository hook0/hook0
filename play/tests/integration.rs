//! Black-box integration tests for the hooks server and client interaction.
//!
//! These tests verify the complete webhook relay functionality including:
//! - Server endpoints (health, webhook receiver, inspection API)
//! - WebSocket connection and protocol
//! - Full E2E webhook forwarding scenarios

use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use hook0_play::{create_app, relay, AppState};

// ============================================================================
// Test Helpers
// ============================================================================

/// Start the hooks server on a random available port
async fn start_test_server() -> (SocketAddr, Arc<AppState>) {
    let port = portpicker::pick_unused_port().expect("No free port available");
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let state = Arc::new(AppState::new(format!("http://{}", addr)));
    let app = create_app(state.clone());

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    let local_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    (local_addr, state)
}

/// Protocol messages for WebSocket communication
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    Start { version: u16, data: StartData },
    Response { version: u16, data: ResponseData },
    Ping,
}

#[derive(Debug, Clone, Serialize)]
struct StartData {
    token: String,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseData {
    id: String,
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
enum ServerMessage {
    Started { version: u16, data: StartedData },
    Request { version: u16, data: RequestData },
    Error { version: u16, data: ErrorData },
    Pong,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct StartedData {
    webhook_url: String,
    view_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct RequestData {
    id: String,
    method: String,
    path: String,
    body: String,
    headers: HashMap<String, String>,
    query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct ErrorData {
    code: String,
    message: String,
}

// ============================================================================
// Server Standalone Tests
// ============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("http://{}/health", addr))
        .send()
        .await
        .expect("Health request failed");

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "OK");
}

#[tokio::test]
async fn test_webhook_receiver_invalid_token_format() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    // Invalid token format (missing prefix)
    let response = client
        .post(format!("http://{}/in/invalid_token/", addr))
        .body("test payload")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 404);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "invalid_token");
}

#[tokio::test]
async fn test_webhook_receiver_stores_webhook() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let token = relay::generate_token();
    let payload = json!({"user_id": 123, "action": "test"});

    // Send webhook
    let response = client
        .post(format!("http://{}/in/{}/", addr, token))
        .header("content-type", "application/json")
        .header("x-custom-header", "test-value")
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["status"], "stored");
    assert!(body["id"].is_string());

    // Verify webhook is stored via inspection API
    let webhooks_response = client
        .get(format!("http://{}/api/tokens/{}/webhooks", addr, token))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(webhooks_response.status(), 200);
    let webhooks: Value = webhooks_response.json().await.unwrap();
    assert_eq!(webhooks["webhooks"].as_array().unwrap().len(), 1);

    let stored_webhook = &webhooks["webhooks"][0];
    assert_eq!(stored_webhook["method"], "POST");
    assert_eq!(stored_webhook["path"], "/");
}

#[tokio::test]
async fn test_webhook_receiver_without_trailing_slash() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let token = relay::generate_token();
    let payload = json!({"event": "no_slash"});

    // Send webhook WITHOUT trailing slash — must not return 404
    let response = client
        .post(format!("http://{}/in/{}", addr, token))
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["status"], "stored");
    assert!(body["id"].is_string());

    // Verify webhook is stored via inspection API
    let webhooks_response = client
        .get(format!("http://{}/api/tokens/{}/webhooks", addr, token))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(webhooks_response.status(), 200);
    let webhooks: Value = webhooks_response.json().await.unwrap();
    assert_eq!(webhooks["webhooks"].as_array().unwrap().len(), 1);

    let stored_webhook = &webhooks["webhooks"][0];
    assert_eq!(stored_webhook["method"], "POST");
    assert_eq!(stored_webhook["path"], "/");
}

#[tokio::test]
async fn test_webhook_receiver_with_path_and_query() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let token = relay::generate_token();

    // Send webhook with path and query
    let response = client
        .put(format!(
            "http://{}/in/{}/api/users/123?format=json&verbose=true",
            addr, token
        ))
        .body("update data")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    // Verify stored webhook
    let webhooks_response = client
        .get(format!("http://{}/api/tokens/{}/webhooks", addr, token))
        .send()
        .await
        .unwrap();

    let webhooks: Value = webhooks_response.json().await.unwrap();
    let stored = &webhooks["webhooks"][0];

    assert_eq!(stored["method"], "PUT");
    assert_eq!(stored["path"], "/api/users/123");
    // Query params may be in different order
    let query = stored["query"].as_str().unwrap();
    assert!(query.contains("format=json"));
    assert!(query.contains("verbose=true"));
}

#[tokio::test]
async fn test_inspection_api_get_specific_webhook() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let token = relay::generate_token();

    // Store a webhook
    let response = client
        .post(format!("http://{}/in/{}/", addr, token))
        .body("test body")
        .send()
        .await
        .unwrap();

    let body: Value = response.json().await.unwrap();
    let webhook_id = body["id"].as_str().unwrap();

    // Get specific webhook
    let webhook_response = client
        .get(format!(
            "http://{}/api/tokens/{}/webhooks/{}",
            addr, token, webhook_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(webhook_response.status(), 200);
    let webhook: Value = webhook_response.json().await.unwrap();
    assert_eq!(webhook["id"], webhook_id);
}

#[tokio::test]
async fn test_inspection_api_webhook_not_found() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let token = relay::generate_token();

    let response = client
        .get(format!(
            "http://{}/api/tokens/{}/webhooks/nonexistent-id",
            addr, token
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn test_session_api() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let token = relay::generate_token();

    let response = client
        .get(format!("http://{}/api/tokens/{}", addr, token))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["session"]["connected"], false);
    assert_eq!(body["session"]["total_webhooks"], 0);
}

#[tokio::test]
async fn test_view_token_endpoint() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();

    let token = relay::generate_token();

    let response = client
        .get(format!("http://{}/view/{}", addr, token))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["token"], token);
    assert!(body["webhook_url"].as_str().unwrap().contains(&token));
}

// ============================================================================
// WebSocket Connection Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_connection_and_start() {
    let (addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    let ws_url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("WebSocket connect failed");

    // Send start message
    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: token.clone(),
        },
    };
    let msg_json = serde_json::to_string(&start_msg).unwrap();
    ws_stream
        .send(Message::Text(msg_json.into()))
        .await
        .unwrap();

    // Receive started confirmation
    let response = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for started message")
        .expect("Stream ended")
        .expect("WebSocket error");

    if let Message::Text(text) = response {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        match server_msg {
            ServerMessage::Started { data, .. } => {
                assert!(data.webhook_url.contains(&token));
                assert!(data.view_url.contains(&token));
            }
            _ => panic!("Expected Started message, got {:?}", server_msg),
        }
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
}

#[tokio::test]
async fn test_websocket_invalid_token() {
    let (addr, _state) = start_test_server().await;

    let ws_url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("WebSocket connect failed");

    // Send start with invalid token
    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: "invalid".to_string(),
        },
    };
    let msg_json = serde_json::to_string(&start_msg).unwrap();
    ws_stream
        .send(Message::Text(msg_json.into()))
        .await
        .unwrap();

    // Should receive error
    let response = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout")
        .expect("Stream ended")
        .expect("WebSocket error");

    if let Message::Text(text) = response {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        match server_msg {
            ServerMessage::Error { data, .. } => {
                assert_eq!(data.code, "invalid_token");
            }
            _ => panic!("Expected Error message"),
        }
    }

    ws_stream.close(None).await.ok();
}

#[tokio::test]
async fn test_websocket_token_already_in_use() {
    let (addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    let ws_url = format!("ws://{}/ws", addr);

    // First connection
    let (mut ws1, _) = connect_async(&ws_url).await.unwrap();
    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: token.clone(),
        },
    };
    ws1.send(Message::Text(
        serde_json::to_string(&start_msg).unwrap().into(),
    ))
    .await
    .unwrap();

    // Wait for started
    let _ = timeout(Duration::from_secs(5), ws1.next()).await.unwrap();

    // Second connection with same token
    let (mut ws2, _) = connect_async(&ws_url).await.unwrap();
    ws2.send(Message::Text(
        serde_json::to_string(&start_msg).unwrap().into(),
    ))
    .await
    .unwrap();

    // Should receive error
    let response = timeout(Duration::from_secs(5), ws2.next())
        .await
        .expect("Timeout")
        .expect("Stream ended")
        .expect("WebSocket error");

    if let Message::Text(text) = response {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        match server_msg {
            ServerMessage::Error { data, .. } => {
                assert_eq!(data.code, "token_in_use");
            }
            _ => panic!("Expected Error message, got {:?}", server_msg),
        }
    }

    ws1.close(None).await.ok();
    ws2.close(None).await.ok();
}

#[tokio::test]
async fn test_websocket_ping_pong() {
    let (addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    let ws_url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

    // Connect first
    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData { token },
    };
    ws_stream
        .send(Message::Text(
            serde_json::to_string(&start_msg).unwrap().into(),
        ))
        .await
        .unwrap();
    let _ = ws_stream.next().await; // Consume started

    // Send ping
    let ping_msg = ClientMessage::Ping;
    ws_stream
        .send(Message::Text(
            serde_json::to_string(&ping_msg).unwrap().into(),
        ))
        .await
        .unwrap();

    // Should receive pong
    let response = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout")
        .expect("Stream ended")
        .expect("WebSocket error");

    if let Message::Text(text) = response {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        assert!(matches!(server_msg, ServerMessage::Pong));
    }

    ws_stream.close(None).await.ok();
}

// ============================================================================
// Full E2E Integration Tests
// ============================================================================

#[tokio::test]
async fn test_e2e_webhook_forwarding() {
    let (server_addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    // Connect WebSocket client
    let ws_url = format!("ws://{}/ws", server_addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

    // Start listening
    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: token.clone(),
        },
    };
    ws_stream
        .send(Message::Text(
            serde_json::to_string(&start_msg).unwrap().into(),
        ))
        .await
        .unwrap();

    // Wait for started and verify
    let started_msg = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    if let Message::Text(text) = &started_msg {
        let msg: ServerMessage = serde_json::from_str(text).unwrap();
        assert!(matches!(msg, ServerMessage::Started { .. }));
    }

    // Spawn HTTP request in background
    let http_client = Client::new();
    let webhook_payload = json!({"event": "user.created", "user_id": 42});
    let server_addr_clone = server_addr;
    let token_clone = token.clone();

    let http_handle = tokio::spawn(async move {
        http_client
            .post(format!("http://{}/in/{}/", server_addr_clone, token_clone))
            .header("content-type", "application/json")
            .header("x-webhook-signature", "sha256=abc123")
            .json(&webhook_payload)
            .send()
            .await
    });

    // Receive forwarded request via WebSocket
    let request_msg = timeout(Duration::from_secs(10), ws_stream.next())
        .await
        .expect("Timeout waiting for request")
        .expect("Stream ended")
        .expect("WebSocket error");

    if let Message::Text(text) = request_msg {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        match server_msg {
            ServerMessage::Request { data, .. } => {
                assert_eq!(data.method, "POST");
                assert_eq!(data.path, "/");

                // Decode and verify body
                let body_bytes =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data.body)
                        .unwrap();
                let body: Value = serde_json::from_slice(&body_bytes).unwrap();
                assert_eq!(body["event"], "user.created");
                assert_eq!(body["user_id"], 42);

                // Verify headers
                assert!(data.headers.contains_key("content-type"));
                assert!(data.headers.contains_key("x-webhook-signature"));

                // Send response back
                let response_msg = ClientMessage::Response {
                    version: 1,
                    data: ResponseData {
                        id: data.id,
                        status: 200,
                        headers: HashMap::from([(
                            "content-type".to_string(),
                            "application/json".to_string(),
                        )]),
                        body: base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            b"{\"received\": true}",
                        ),
                    },
                };
                ws_stream
                    .send(Message::Text(
                        serde_json::to_string(&response_msg).unwrap().into(),
                    ))
                    .await
                    .unwrap();
            }
            _ => panic!("Expected Request message, got {:?}", server_msg),
        }
    }

    // Verify HTTP response completed
    let response = http_handle.await.unwrap().unwrap();
    assert_eq!(response.status(), 200);

    ws_stream.close(None).await.ok();
}

#[tokio::test]
async fn test_e2e_multiple_webhooks_sequence() {
    let (server_addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    // Connect WebSocket
    let ws_url = format!("ws://{}/ws", server_addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: token.clone(),
        },
    };
    ws_stream
        .send(Message::Text(
            serde_json::to_string(&start_msg).unwrap().into(),
        ))
        .await
        .unwrap();
    let _ = ws_stream.next().await; // Consume started

    let http_client = Client::new();

    // Send multiple webhooks sequentially
    for i in 0..5 {
        let server_addr_clone = server_addr;
        let token_clone = token.clone();
        let body = format!("webhook-{}", i);
        let http_client_clone = http_client.clone();

        let http_handle = tokio::spawn(async move {
            http_client_clone
                .post(format!("http://{}/in/{}/", server_addr_clone, token_clone))
                .body(body)
                .send()
                .await
        });

        // Receive and respond
        let msg = timeout(Duration::from_secs(10), ws_stream.next())
            .await
            .expect("Timeout")
            .expect("Stream ended")
            .expect("WS error");

        if let Message::Text(text) = msg {
            let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
            if let ServerMessage::Request { data, .. } = server_msg {
                let body_decoded =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data.body)
                        .unwrap();
                assert_eq!(
                    String::from_utf8(body_decoded).unwrap(),
                    format!("webhook-{}", i)
                );

                // Send response
                let response_msg = ClientMessage::Response {
                    version: 1,
                    data: ResponseData {
                        id: data.id,
                        status: 200,
                        headers: HashMap::new(),
                        body: String::new(),
                    },
                };
                ws_stream
                    .send(Message::Text(
                        serde_json::to_string(&response_msg).unwrap().into(),
                    ))
                    .await
                    .unwrap();
            }
        }

        let _ = http_handle.await;
    }

    // Verify all webhooks stored
    let webhooks_response = http_client
        .get(format!(
            "http://{}/api/tokens/{}/webhooks",
            server_addr, token
        ))
        .send()
        .await
        .unwrap();

    let webhooks: Value = webhooks_response.json().await.unwrap();
    assert_eq!(webhooks["webhooks"].as_array().unwrap().len(), 5);
    assert_eq!(webhooks["session"]["forwarded_webhooks"], 5);

    ws_stream.close(None).await.ok();
}

#[tokio::test]
async fn test_e2e_disconnect_and_store_mode() {
    let (server_addr, _state) = start_test_server().await;
    let token = relay::generate_token();
    let http_client = Client::new();

    // First, connect and verify connected state
    {
        let ws_url = format!("ws://{}/ws", server_addr);
        let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

        let start_msg = ClientMessage::Start {
            version: 1,
            data: StartData {
                token: token.clone(),
            },
        };
        ws_stream
            .send(Message::Text(
                serde_json::to_string(&start_msg).unwrap().into(),
            ))
            .await
            .unwrap();
        let _ = ws_stream.next().await;

        // Verify connected
        let session: Value = http_client
            .get(format!("http://{}/api/tokens/{}", server_addr, token))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(session["session"]["connected"], true);

        ws_stream.close(None).await.ok();
    }

    // Wait for disconnect to be processed
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify disconnected
    let session: Value = http_client
        .get(format!("http://{}/api/tokens/{}", server_addr, token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(session["session"]["connected"], false);

    // Send webhook while disconnected
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .body("disconnected webhook")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["status"], "stored"); // Should be stored, not forwarded
}

#[tokio::test]
async fn test_e2e_large_payload() {
    let (server_addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    // Connect WebSocket
    let ws_url = format!("ws://{}/ws", server_addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: token.clone(),
        },
    };
    ws_stream
        .send(Message::Text(
            serde_json::to_string(&start_msg).unwrap().into(),
        ))
        .await
        .unwrap();
    let _ = ws_stream.next().await;

    // Send large payload (1MB)
    let large_payload = "x".repeat(1024 * 1024);
    let http_client = Client::new();
    let server_addr_clone = server_addr;
    let token_clone = token.clone();
    let payload_clone = large_payload.clone();

    let http_handle = tokio::spawn(async move {
        http_client
            .post(format!("http://{}/in/{}/", server_addr_clone, token_clone))
            .body(payload_clone)
            .send()
            .await
    });

    // Receive and verify
    let msg = timeout(Duration::from_secs(30), ws_stream.next())
        .await
        .expect("Timeout")
        .expect("Stream ended")
        .expect("WS error");

    if let Message::Text(text) = msg {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        if let ServerMessage::Request { data, .. } = server_msg {
            let body =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data.body)
                    .unwrap();
            assert_eq!(body.len(), 1024 * 1024);

            // Send response
            let response_msg = ClientMessage::Response {
                version: 1,
                data: ResponseData {
                    id: data.id,
                    status: 200,
                    headers: HashMap::new(),
                    body: String::new(),
                },
            };
            ws_stream
                .send(Message::Text(
                    serde_json::to_string(&response_msg).unwrap().into(),
                ))
                .await
                .unwrap();
        }
    }

    let response = http_handle.await.unwrap().unwrap();
    assert_eq!(response.status(), 200);

    ws_stream.close(None).await.ok();
}

#[tokio::test]
async fn test_e2e_various_http_methods() {
    let (server_addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    let ws_url = format!("ws://{}/ws", server_addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: token.clone(),
        },
    };
    ws_stream
        .send(Message::Text(
            serde_json::to_string(&start_msg).unwrap().into(),
        ))
        .await
        .unwrap();
    let _ = ws_stream.next().await;

    let http_client = Client::new();
    let methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE"];

    for method in methods {
        let server_addr_clone = server_addr;
        let token_clone = token.clone();
        let http_client_clone = http_client.clone();
        let method_clone = method.to_string();

        let http_handle = tokio::spawn(async move {
            let request = match method_clone.as_str() {
                "GET" => http_client_clone
                    .get(format!("http://{}/in/{}/", server_addr_clone, token_clone)),
                "POST" => http_client_clone
                    .post(format!("http://{}/in/{}/", server_addr_clone, token_clone)),
                "PUT" => http_client_clone
                    .put(format!("http://{}/in/{}/", server_addr_clone, token_clone)),
                "PATCH" => http_client_clone
                    .patch(format!("http://{}/in/{}/", server_addr_clone, token_clone)),
                "DELETE" => http_client_clone
                    .delete(format!("http://{}/in/{}/", server_addr_clone, token_clone)),
                _ => unreachable!(),
            };
            request.send().await
        });

        let msg = timeout(Duration::from_secs(10), ws_stream.next())
            .await
            .expect("Timeout")
            .expect("Stream ended")
            .expect("WS error");

        if let Message::Text(text) = msg {
            let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
            if let ServerMessage::Request { data, .. } = server_msg {
                assert_eq!(data.method, method);

                let response_msg = ClientMessage::Response {
                    version: 1,
                    data: ResponseData {
                        id: data.id,
                        status: 200,
                        headers: HashMap::new(),
                        body: String::new(),
                    },
                };
                ws_stream
                    .send(Message::Text(
                        serde_json::to_string(&response_msg).unwrap().into(),
                    ))
                    .await
                    .unwrap();
            }
        }

        let _ = http_handle.await;
    }

    ws_stream.close(None).await.ok();
}

#[tokio::test]
async fn test_e2e_reconnection_after_disconnect() {
    let (server_addr, _state) = start_test_server().await;
    let token = relay::generate_token();

    // First connection
    {
        let ws_url = format!("ws://{}/ws", server_addr);
        let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

        let start_msg = ClientMessage::Start {
            version: 1,
            data: StartData {
                token: token.clone(),
            },
        };
        ws_stream
            .send(Message::Text(
                serde_json::to_string(&start_msg).unwrap().into(),
            ))
            .await
            .unwrap();
        let _ = ws_stream.next().await;

        ws_stream.close(None).await.ok();
    }

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Reconnect with same token (should work after disconnect)
    let ws_url = format!("ws://{}/ws", server_addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: token.clone(),
        },
    };
    ws_stream
        .send(Message::Text(
            serde_json::to_string(&start_msg).unwrap().into(),
        ))
        .await
        .unwrap();

    let msg = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    if let Message::Text(text) = msg {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        assert!(matches!(server_msg, ServerMessage::Started { .. }));
    }

    ws_stream.close(None).await.ok();
}

// ============================================================================
// Token Generation Tests
// ============================================================================

#[tokio::test]
async fn test_token_format_validation() {
    // Valid tokens
    assert!(relay::is_valid_token("c_123456789012345678901234567"));
    assert!(relay::is_valid_token("c_ABCDEFGHIJKLMNOPQRSTUVWXYZa"));
    assert!(relay::is_valid_token("c_abcdefghijklmnopqrstuvwxyz0"));

    // Invalid tokens
    assert!(!relay::is_valid_token("")); // Empty
    assert!(!relay::is_valid_token("c_")); // Too short
    assert!(!relay::is_valid_token("c_12345678901234567890123456")); // Too short (26 chars)
    assert!(!relay::is_valid_token("c_1234567890123456789012345678")); // Too long (28 chars)
    assert!(!relay::is_valid_token("x_123456789012345678901234567")); // Wrong prefix
    assert!(!relay::is_valid_token("123456789012345678901234567890")); // No prefix
    assert!(!relay::is_valid_token("c_12345678901234567890123456!")); // Invalid char
}

#[tokio::test]
async fn test_generated_tokens_are_valid() {
    for _ in 0..100 {
        let token = relay::generate_token();
        assert!(
            relay::is_valid_token(&token),
            "Generated token '{}' should be valid",
            token
        );
    }
}

#[tokio::test]
async fn test_generated_tokens_are_unique() {
    let tokens: std::collections::HashSet<String> =
        (0..1000).map(|_| relay::generate_token()).collect();
    assert_eq!(tokens.len(), 1000, "All generated tokens should be unique");
}

// ============================================================================
// Abuse Protection Tests
// ============================================================================

/// Start a test server with custom limits for abuse testing
async fn start_test_server_with_limits(
    limits: hook0_play::ServerLimits,
) -> (SocketAddr, Arc<AppState>) {
    let port = portpicker::pick_unused_port().expect("No free port available");
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let state = Arc::new(AppState::with_limits(format!("http://{}", addr), limits));
    let app = create_app(state.clone());

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    let local_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    (local_addr, state)
}

#[tokio::test]
async fn test_payload_size_limit() {
    // Create server with small payload limit (1KB)
    let limits = hook0_play::ServerLimits {
        max_payload_size: 1024, // 1KB
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let token = relay::generate_token();
    let http_client = Client::new();

    // Small payload should succeed
    let small_payload = "x".repeat(500);
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .body(small_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Large payload should be rejected
    let large_payload = "x".repeat(2048); // 2KB > 1KB limit
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .body(large_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 413); // Payload Too Large

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "payload_too_large");
}

#[tokio::test]
async fn test_payload_size_limit_with_path() {
    // Create server with small payload limit
    let limits = hook0_play::ServerLimits {
        max_payload_size: 512,
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let token = relay::generate_token();
    let http_client = Client::new();

    // Test with path segments - should also enforce limit
    let large_payload = "x".repeat(1024);
    let response = http_client
        .post(format!(
            "http://{}/in/{}/webhook/events",
            server_addr, token
        ))
        .body(large_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 413);
}

#[tokio::test]
async fn test_max_webhooks_per_token_fifo_eviction() {
    // Create server with small webhook limit for testing
    let limits = hook0_play::ServerLimits {
        max_webhooks_per_token: 3,
        ..Default::default()
    };
    let (server_addr, state) = start_test_server_with_limits(limits).await;

    let token = relay::generate_token();
    let http_client = Client::new();

    // Send 5 webhooks
    for i in 0..5 {
        let response = http_client
            .post(format!("http://{}/in/{}/", server_addr, token))
            .body(format!("webhook_{}", i))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    // Only the last 3 should be stored (FIFO eviction)
    let webhooks = state.storage.get_webhooks(&token);
    assert_eq!(webhooks.len(), 3, "Should only have 3 webhooks (max limit)");
}

#[tokio::test]
async fn test_connection_limit_per_ip() {
    // Create server with very low connection limit
    let limits = hook0_play::ServerLimits {
        max_connections_per_ip: 2,
        max_total_connections: 100, // High total to not interfere
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let ws_url = format!("ws://{}/ws", server_addr);

    // First connection should succeed
    let conn1 = connect_async(&ws_url).await;
    assert!(conn1.is_ok(), "First connection should succeed");
    let (mut ws1, _) = conn1.unwrap();

    // Second connection should succeed
    let conn2 = connect_async(&ws_url).await;
    assert!(conn2.is_ok(), "Second connection should succeed");
    let (mut ws2, _) = conn2.unwrap();

    // Third connection should fail (over limit)
    let conn3 = connect_async(&ws_url).await;
    // Connection might succeed at TCP level but get rejected at HTTP level
    // The rejection happens during upgrade
    if let Ok((mut ws3, _)) = conn3 {
        // If connection succeeded, try to send and expect error or close
        let start_msg = ClientMessage::Start {
            version: 1,
            data: StartData {
                token: relay::generate_token(),
            },
        };
        let send_result = ws3
            .send(Message::Text(
                serde_json::to_string(&start_msg).unwrap().into(),
            ))
            .await;
        // Connection should be rejected
        assert!(
            send_result.is_err()
                || timeout(Duration::from_secs(2), ws3.next())
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.is_err() || matches!(r, Ok(Message::Close(_))))
                    .unwrap_or(true),
            "Third connection should be rejected or closed"
        );
    }
    // If connection failed at TCP level, that's also acceptable

    ws1.close(None).await.ok();
    ws2.close(None).await.ok();
}

#[tokio::test]
async fn test_total_connection_limit() {
    // Create server with very low total connection limit
    let limits = hook0_play::ServerLimits {
        max_total_connections: 2,
        max_connections_per_ip: 100, // High per-IP to not interfere
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let ws_url = format!("ws://{}/ws", server_addr);

    // First two connections should succeed
    let conn1 = connect_async(&ws_url).await;
    assert!(conn1.is_ok(), "First connection should succeed");
    let (mut ws1, _) = conn1.unwrap();

    let conn2 = connect_async(&ws_url).await;
    assert!(conn2.is_ok(), "Second connection should succeed");
    let (mut ws2, _) = conn2.unwrap();

    // Third should fail
    let conn3 = connect_async(&ws_url).await;
    if let Ok((mut ws3, _)) = conn3 {
        let start_msg = ClientMessage::Start {
            version: 1,
            data: StartData {
                token: relay::generate_token(),
            },
        };
        let send_result = ws3
            .send(Message::Text(
                serde_json::to_string(&start_msg).unwrap().into(),
            ))
            .await;
        assert!(
            send_result.is_err()
                || timeout(Duration::from_secs(2), ws3.next())
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.is_err() || matches!(r, Ok(Message::Close(_))))
                    .unwrap_or(true),
            "Third connection should be rejected"
        );
    }

    ws1.close(None).await.ok();
    ws2.close(None).await.ok();
}

#[tokio::test]
async fn test_connection_slot_released_on_disconnect() {
    // Create server with low connection limit
    let limits = hook0_play::ServerLimits {
        max_connections_per_ip: 1,
        max_total_connections: 100,
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let ws_url = format!("ws://{}/ws", server_addr);

    // First connection
    {
        let (mut ws1, _) = connect_async(&ws_url).await.unwrap();
        let start_msg = ClientMessage::Start {
            version: 1,
            data: StartData {
                token: relay::generate_token(),
            },
        };
        ws1.send(Message::Text(
            serde_json::to_string(&start_msg).unwrap().into(),
        ))
        .await
        .unwrap();
        let _ = ws1.next().await;
        ws1.close(None).await.ok();
    }

    // Wait for cleanup
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Second connection should work (slot released)
    let conn2 = connect_async(&ws_url).await;
    assert!(
        conn2.is_ok(),
        "Connection should succeed after previous one disconnected"
    );

    let (mut ws2, _) = conn2.unwrap();
    let start_msg = ClientMessage::Start {
        version: 1,
        data: StartData {
            token: relay::generate_token(),
        },
    };
    ws2.send(Message::Text(
        serde_json::to_string(&start_msg).unwrap().into(),
    ))
    .await
    .unwrap();

    let msg = timeout(Duration::from_secs(5), ws2.next())
        .await
        .expect("Timeout")
        .expect("Stream ended")
        .expect("WS error");

    if let Message::Text(text) = msg {
        let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
        assert!(
            matches!(server_msg, ServerMessage::Started { .. }),
            "Should receive Started message"
        );
    }

    ws2.close(None).await.ok();
}

// ============================================================================
// Rate Limiting Tests
// ============================================================================

#[tokio::test]
async fn test_rate_limit_per_token() {
    // Create server with very low per-token rate limit
    let limits = hook0_play::ServerLimits {
        webhook_rate_limit_per_token: 3, // Only 3 per second per token
        webhook_rate_limit_per_ip: 1000, // High so it doesn't interfere
        webhook_rate_limit_global: 1000,
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let token = relay::generate_token();
    let http_client = Client::new();

    // Send requests up to the limit - should all succeed
    for _ in 0..3 {
        let response = http_client
            .post(format!("http://{}/in/{}/", server_addr, token))
            .body("test")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    // Next request should be rate limited
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .body("test")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 429);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "rate_limited");
    assert!(body["retry_after"].is_number());
}

#[tokio::test]
async fn test_rate_limit_per_ip() {
    // Create server with very low per-IP rate limit
    let limits = hook0_play::ServerLimits {
        webhook_rate_limit_per_ip: 2,       // Only 2 per second per IP
        webhook_rate_limit_per_token: 1000, // High
        webhook_rate_limit_global: 1000,
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let http_client = Client::new();

    // Use different tokens but same IP - should all count against IP limit
    for _ in 0..2 {
        let token = relay::generate_token();
        let response = http_client
            .post(format!("http://{}/in/{}/", server_addr, token))
            .body("test")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    // Third request (different token, same IP) should be rate limited
    let token = relay::generate_token();
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .body("test")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 429);
}

#[tokio::test]
async fn test_rate_limit_global() {
    // Create server with very low global rate limit
    let limits = hook0_play::ServerLimits {
        webhook_rate_limit_global: 2, // Only 2 total per second
        webhook_rate_limit_per_ip: 1000,
        webhook_rate_limit_per_token: 1000,
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let http_client = Client::new();

    // Use different tokens
    for _ in 0..2 {
        let token = relay::generate_token();
        let response = http_client
            .post(format!("http://{}/in/{}/", server_addr, token))
            .body("test")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    // Global limit exceeded
    let token = relay::generate_token();
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .body("test")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 429);
}

// ============================================================================
// Invalid Token Blocking Tests
// ============================================================================

#[tokio::test]
async fn test_invalid_token_blocking() {
    // Create server with very low invalid token threshold
    let limits = hook0_play::ServerLimits {
        max_invalid_token_attempts: 3,
        invalid_token_block_duration: Duration::from_secs(60),
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let http_client = Client::new();

    // Send requests with invalid tokens until blocked
    for i in 0..3 {
        let response = http_client
            .post(format!("http://{}/in/bad_token_{}/", server_addr, i))
            .body("test")
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            404,
            "Invalid token should return 404 for attempt {}",
            i
        );
    }

    // After exceeding the limit, even valid tokens should be blocked from this IP
    let valid_token = relay::generate_token();
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, valid_token))
        .body("test")
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        429,
        "Should be blocked after too many invalid attempts"
    );
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "too_many_invalid_attempts");
}

// ============================================================================
// Delete Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_delete_single_webhook() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();
    let token = relay::generate_token();

    // Store a webhook
    let response = client
        .post(format!("http://{}/in/{}/", addr, token))
        .body("to-delete")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    let webhook_id = body["id"].as_str().unwrap().to_string();

    // Verify it exists
    let get_response = client
        .get(format!(
            "http://{}/api/tokens/{}/webhooks/{}",
            addr, token, webhook_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(get_response.status(), 200);

    // Delete it
    let delete_response = client
        .delete(format!(
            "http://{}/api/tokens/{}/webhooks/{}",
            addr, token, webhook_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(delete_response.status(), 200);
    let delete_body: Value = delete_response.json().await.unwrap();
    assert_eq!(delete_body["status"], "deleted");

    // Verify it's gone
    let get_response = client
        .get(format!(
            "http://{}/api/tokens/{}/webhooks/{}",
            addr, token, webhook_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn test_delete_all_webhooks() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();
    let token = relay::generate_token();

    // Store multiple webhooks
    for i in 0..5 {
        let response = client
            .post(format!("http://{}/in/{}/", addr, token))
            .body(format!("webhook-{}", i))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    // Verify they exist
    let list_response = client
        .get(format!("http://{}/api/tokens/{}/webhooks", addr, token))
        .send()
        .await
        .unwrap();
    let list_body: Value = list_response.json().await.unwrap();
    assert_eq!(list_body["webhooks"].as_array().unwrap().len(), 5);

    // Delete all
    let delete_response = client
        .delete(format!("http://{}/api/tokens/{}/webhooks", addr, token))
        .send()
        .await
        .unwrap();
    assert_eq!(delete_response.status(), 200);
    let delete_body: Value = delete_response.json().await.unwrap();
    assert_eq!(delete_body["status"], "deleted");
    assert_eq!(delete_body["count"], 5);

    // Verify all gone
    let list_response = client
        .get(format!("http://{}/api/tokens/{}/webhooks", addr, token))
        .send()
        .await
        .unwrap();
    let list_body: Value = list_response.json().await.unwrap();
    assert_eq!(list_body["webhooks"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_delete_nonexistent_webhook() {
    let (addr, _state) = start_test_server().await;
    let client = Client::new();
    let token = relay::generate_token();

    let delete_response = client
        .delete(format!(
            "http://{}/api/tokens/{}/webhooks/nonexistent-id",
            addr, token
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(delete_response.status(), 404);
    let body: Value = delete_response.json().await.unwrap();
    assert_eq!(body["error"], "not_found");
}

// ============================================================================
// Encrypted Storage Tests
// ============================================================================

#[tokio::test]
async fn test_encrypted_storage_roundtrip() {
    // Create server with encryption enabled
    let limits = hook0_play::ServerLimits {
        enable_encryption: true,
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let token = relay::generate_token();
    let http_client = Client::new();

    let payload = json!({"secret": "sensitive-data", "amount": 42});

    // Store encrypted webhook
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Retrieve via API - should be decrypted transparently
    let webhooks_response = http_client
        .get(format!(
            "http://{}/api/tokens/{}/webhooks",
            server_addr, token
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(webhooks_response.status(), 200);
    let webhooks: Value = webhooks_response.json().await.unwrap();

    let stored_webhook = &webhooks["webhooks"][0];
    // Body should be readable (decrypted)
    let body_b64 = stored_webhook["body"].as_str().unwrap();
    let body_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body_b64).unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["secret"], "sensitive-data");
    assert_eq!(body["amount"], 42);
}

// ============================================================================
// WebSocket Handshake Timeout Test
// ============================================================================

#[tokio::test]
async fn test_websocket_handshake_timeout() {
    // Create server with very short handshake timeout
    let limits = hook0_play::ServerLimits {
        handshake_timeout: Duration::from_millis(200),
        ..Default::default()
    };
    let (server_addr, _state) = start_test_server_with_limits(limits).await;

    let ws_url = format!("ws://{}/ws", server_addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.unwrap();

    // Don't send Start message - just wait for timeout
    let msg = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Should receive timeout error before outer timeout");

    match msg {
        Some(Ok(Message::Text(text))) => {
            let server_msg: ServerMessage = serde_json::from_str(&text).unwrap();
            match server_msg {
                ServerMessage::Error { data, .. } => {
                    assert_eq!(data.code, "handshake_timeout");
                }
                _ => panic!("Expected handshake_timeout error, got {:?}", server_msg),
            }
        }
        Some(Ok(Message::Close(_))) => {
            // Also acceptable - server closed connection
        }
        Some(Err(_)) => {
            // Protocol error (e.g., ResetWithoutClosingHandshake) is acceptable
            // when server drops the connection on timeout
        }
        None => {
            // Stream ended - also acceptable
        }
        other => panic!("Unexpected message: {:?}", other),
    }
}

// ============================================================================
// TTL Cleanup Test
// ============================================================================

#[tokio::test]
async fn test_webhook_ttl_cleanup() {
    // Create server with very short TTL
    let limits = hook0_play::ServerLimits {
        webhook_ttl: Duration::from_millis(100), // 100ms TTL
        ..Default::default()
    };
    let (server_addr, state) = start_test_server_with_limits(limits).await;

    let token = relay::generate_token();
    let http_client = Client::new();

    // Store a webhook
    let response = http_client
        .post(format!("http://{}/in/{}/", server_addr, token))
        .body("expiring")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Verify it exists
    let webhooks = state.storage.get_webhooks(&token);
    assert_eq!(webhooks.len(), 1);

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Manually trigger cleanup (background task interval is too long for tests)
    let removed = state.storage.cleanup_expired(Duration::from_millis(100));
    assert!(removed > 0, "Should have cleaned up expired webhooks");

    // Verify it's gone
    let webhooks = state.storage.get_webhooks(&token);
    assert_eq!(webhooks.len(), 0, "Expired webhooks should be cleaned up");
}

// ============================================================================
// Rate Limiter Unit Tests (via public interface)
// ============================================================================

#[tokio::test]
async fn test_rate_limiter_cleanup() {
    use hook0_play::rate_limiter::RateLimiter;

    let limiter = RateLimiter::new(Duration::from_millis(50), 2);

    // Fill rate limit
    assert!(limiter.check("key1").is_ok());
    assert!(limiter.check("key1").is_ok());
    assert!(limiter.check("key1").is_err());

    // Wait for window to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cleanup should remove old entries
    limiter.cleanup();

    // Should be able to make requests again
    assert!(limiter.check("key1").is_ok());
}

#[tokio::test]
async fn test_invalid_token_tracker_blocking_and_recovery() {
    use hook0_play::rate_limiter::InvalidTokenTracker;

    let tracker = InvalidTokenTracker::new(
        Duration::from_millis(50),  // window
        3,                          // max attempts
        Duration::from_millis(100), // block duration
    );

    // First 3 attempts should be allowed
    for _ in 0..3 {
        assert!(tracker.check_allowed("test_ip"));
        tracker.record_invalid("test_ip");
    }

    // Should be blocked now
    assert!(!tracker.check_allowed("test_ip"));

    // Wait for block duration + window to expire
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Should be unblocked
    assert!(tracker.check_allowed("test_ip"));
}
