//! E2E integration tests proving the full webhook pipeline works:
//! Webhook sender → POST /in/<token>/ → [hooks server] → WebSocket → [CLI listen] → [local HTTP server]
//!                                                      ← ← Response flows back via WebSocket ← ← ←
//!
//! These tests are fully black-box: the CLI is spawned as a child process,
//! the hooks server runs in-process, and the echo server is a real HTTP server.

mod helpers;

use axum::http::StatusCode;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

use helpers::{
    spawn_cli, start_echo_server, start_hooks_server, wait_for_connection,
    wait_for_n_webhooks_with_response, wait_for_webhook_with_response, EchoConfig,
};

// ============================================================================
// E2E Tests
// ============================================================================

/// Test 1: Full round-trip — webhook is forwarded to local server and response flows back.
#[tokio::test]
async fn test_webhook_forwarded_to_local_server() {
    // 1. Start hooks server
    let (hooks_addr, state) = start_hooks_server().await;
    let base_url = format!("http://{}", hooks_addr);

    // 2. Start echo server that returns 200 + echoes body
    let echo_config = EchoConfig {
        status: StatusCode::OK,
        body: r#"{"echo":"ok"}"#.to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
    };
    let echo_addr = start_echo_server(echo_config).await;

    // 3. Spawn CLI binary
    let relay_url = format!("ws://{}/ws", hooks_addr);
    let mut child = spawn_cli(echo_addr.port(), &relay_url, &[]);

    // 4. Wait for CLI to connect
    let token = timeout(
        Duration::from_secs(10),
        wait_for_connection(&state, Duration::from_secs(10)),
    )
    .await
    .expect("CLI connection timed out");

    // 5. Send a webhook to hooks server
    let client = reqwest::Client::new();
    let webhook_url = format!("{}/in/{}/", base_url, token);

    let send_resp = client
        .post(&webhook_url)
        .header("content-type", "application/json")
        .body(r#"{"test":"data"}"#)
        .send()
        .await
        .expect("Failed to send webhook");

    assert_eq!(send_resp.status(), 200);
    let send_body: Value = send_resp
        .json()
        .await
        .expect("Failed to parse webhook response");
    assert_eq!(send_body["status"], "forwarded");

    // 6. Poll inspection API until response is available
    let webhook = wait_for_webhook_with_response(&base_url, &token, Duration::from_secs(10)).await;

    // 7. Assert: webhook was forwarded and response is 200
    assert_eq!(webhook["forwarded"], true);
    assert_eq!(webhook["response"]["status"], 200);

    // Cleanup
    child.kill().await.ok();
}

/// Test 2: Custom response (status, headers, body) flows back through the tunnel.
#[tokio::test]
async fn test_local_server_custom_response_flows_back() {
    // 1. Start hooks server
    let (hooks_addr, state) = start_hooks_server().await;
    let base_url = format!("http://{}", hooks_addr);

    // 2. Start echo server with custom response
    let echo_config = EchoConfig {
        status: StatusCode::CREATED,
        body: r#"{"result":"ok"}"#.to_string(),
        headers: vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("x-custom".to_string(), "test-value".to_string()),
        ],
    };
    let echo_addr = start_echo_server(echo_config).await;

    // 3. Spawn CLI
    let relay_url = format!("ws://{}/ws", hooks_addr);
    let mut child = spawn_cli(echo_addr.port(), &relay_url, &[]);

    // 4. Wait for connection
    let token = timeout(
        Duration::from_secs(10),
        wait_for_connection(&state, Duration::from_secs(10)),
    )
    .await
    .expect("CLI connection timed out");

    // 5. Send webhook
    let client = reqwest::Client::new();
    let webhook_url = format!("{}/in/{}/", base_url, token);

    let send_resp = client
        .post(&webhook_url)
        .body(r#"{"hello":"world"}"#)
        .send()
        .await
        .expect("Failed to send webhook");

    assert_eq!(send_resp.status(), 200);

    // 6. Poll for response
    let webhook = wait_for_webhook_with_response(&base_url, &token, Duration::from_secs(10)).await;

    // 7. Assert: status 201, custom header, body contains "result"
    assert_eq!(webhook["response"]["status"], 201);

    let response_headers = webhook["response"]["headers"]
        .as_object()
        .expect("response headers should be an object");
    assert_eq!(response_headers["x-custom"], "test-value");

    // Decode base64 body
    let body_b64 = webhook["response"]["body"]
        .as_str()
        .expect("response body should be a string");
    let body_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body_b64)
        .expect("response body should be valid base64");
    let body_str = String::from_utf8(body_bytes).expect("response body should be UTF-8");
    assert!(
        body_str.contains("result"),
        "response body should contain 'result', got: {}",
        body_str
    );

    // Cleanup
    child.kill().await.ok();
}

/// Test 3: Multiple HTTP methods (GET, POST, PUT, DELETE) are correctly forwarded.
#[tokio::test]
async fn test_multiple_methods_forwarded() {
    // 1. Start hooks server
    let (hooks_addr, state) = start_hooks_server().await;
    let base_url = format!("http://{}", hooks_addr);

    // 2. Start echo server
    let echo_config = EchoConfig {
        status: StatusCode::OK,
        body: r#"{"method_test":"ok"}"#.to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
    };
    let echo_addr = start_echo_server(echo_config).await;

    // 3. Spawn CLI
    let relay_url = format!("ws://{}/ws", hooks_addr);
    let mut child = spawn_cli(echo_addr.port(), &relay_url, &[]);

    // 4. Wait for connection
    let token = timeout(
        Duration::from_secs(10),
        wait_for_connection(&state, Duration::from_secs(10)),
    )
    .await
    .expect("CLI connection timed out");

    // 5. Send 4 webhooks with different methods
    let client = reqwest::Client::new();
    let webhook_url = format!("{}/in/{}/", base_url, token);

    let methods = [
        reqwest::Method::GET,
        reqwest::Method::POST,
        reqwest::Method::PUT,
        reqwest::Method::DELETE,
    ];

    for method in &methods {
        let resp = client
            .request(method.clone(), &webhook_url)
            .body(format!(r#"{{"method":"{}"}}"#, method))
            .send()
            .await
            .expect("Failed to send webhook");

        assert_eq!(
            resp.status(),
            200,
            "Webhook send failed for method {}",
            method
        );

        let body: Value = resp.json().await.expect("Failed to parse response");
        assert_eq!(
            body["status"], "forwarded",
            "Webhook not forwarded for method {}",
            method
        );

        // Small delay between sends to avoid race conditions
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // 6. Wait for all 4 webhook responses
    let webhooks =
        wait_for_n_webhooks_with_response(&base_url, &token, 4, Duration::from_secs(15)).await;

    // 7. Assert: all 4 are forwarded with 200 responses
    assert_eq!(webhooks.len(), 4);
    for wh in &webhooks {
        assert_eq!(wh["forwarded"], true);
        assert_eq!(wh["response"]["status"], 200);
    }

    // Verify we got all 4 methods
    let received_methods: Vec<String> = webhooks
        .iter()
        .map(|wh| wh["method"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(
        received_methods.contains(&"GET".to_string()),
        "Missing GET method"
    );
    assert!(
        received_methods.contains(&"POST".to_string()),
        "Missing POST method"
    );
    assert!(
        received_methods.contains(&"PUT".to_string()),
        "Missing PUT method"
    );
    assert!(
        received_methods.contains(&"DELETE".to_string()),
        "Missing DELETE method"
    );

    // Cleanup
    child.kill().await.ok();
}
