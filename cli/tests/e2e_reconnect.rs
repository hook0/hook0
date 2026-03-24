//! E2E tests for WebSocket reconnection behavior:
//! - Reconnect after server-side disconnect (via TCP proxy severing)
//! - Token collision detection and regeneration
//! - Reconnect after connection drop with ping-based detection
//!
//! These tests are fully black-box: the CLI is spawned as a child process,
//! the hooks server runs in-process, and the echo server is a real HTTP server.

mod helpers;

use axum::http::StatusCode;
use std::time::Duration;
use tokio::time::timeout;

use helpers::{
    spawn_cli, start_echo_server, start_hooks_server, wait_for_connection, wait_for_new_connection,
    wait_for_webhook_with_response, EchoConfig, TcpProxy,
};

// ============================================================================
// Reconnection Tests
// ============================================================================

/// Test: CLI reconnects after TCP-level disconnection.
/// Uses a TCP proxy between CLI and hooks server. Shutting down the proxy
/// severs the connection; restarting it allows the CLI to reconnect.
///
/// 1. Start hooks server + echo server + TCP proxy + CLI (through proxy)
/// 2. Send webhook → verify forwarded (baseline works)
/// 3. Shut down TCP proxy → severs connection
/// 4. Restart TCP proxy on same port
/// 5. Wait for CLI to reconnect
/// 6. Send another webhook → verify forwarded (reconnection works)
#[tokio::test]
async fn test_reconnect_after_server_disconnect() {
    // 1. Start hooks server
    let (hooks_addr, state) = start_hooks_server().await;
    let base_url = format!("http://{}", hooks_addr);

    // 2. Start echo server
    let echo_config = EchoConfig {
        status: StatusCode::OK,
        body: r#"{"echo":"ok"}"#.to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
    };
    let echo_addr = start_echo_server(echo_config).await;

    // 3. Start TCP proxy in front of hooks server
    let proxy = TcpProxy::start(hooks_addr).await;
    let proxy_addr = proxy.addr;

    // 4. Spawn CLI connecting through the proxy
    let relay_url = format!("ws://{}/ws", proxy_addr);
    let mut child = spawn_cli(echo_addr.port(), &relay_url, &[]);

    // 5. Wait for CLI to connect
    let token = timeout(
        Duration::from_secs(10),
        wait_for_connection(&state, Duration::from_secs(10)),
    )
    .await
    .expect("CLI connection timed out");

    // 6. Send a webhook to verify baseline works
    let client = reqwest::Client::new();
    let webhook_url = format!("{}/in/{}/", base_url, token);

    let send_resp = client
        .post(&webhook_url)
        .header("content-type", "application/json")
        .body(r#"{"test":"before_disconnect"}"#)
        .send()
        .await
        .expect("Failed to send webhook");

    assert_eq!(send_resp.status(), 200);

    // Wait for webhook to be processed
    let webhook = wait_for_webhook_with_response(&base_url, &token, Duration::from_secs(10)).await;
    assert_eq!(webhook["forwarded"], true);
    assert_eq!(webhook["response"]["status"], 200);

    // 7. Shut down TCP proxy → severs the connection on both sides
    proxy.shutdown();

    // Give both sides time to detect the severed connection
    tokio::time::sleep(Duration::from_secs(1)).await;

    // The server should have detected the broken connection and cleaned up
    // Wait for server to remove the old connection
    let cleanup_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while !state.connections.is_empty() {
        if tokio::time::Instant::now() >= cleanup_deadline {
            // Force cleanup if server hasn't detected it
            state.connections.remove(&token);
            state.storage.set_disconnected(&token);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 8. Restart TCP proxy on the same port
    let _proxy = TcpProxy::restart_on(proxy_addr, hooks_addr).await;

    // 9. Wait for CLI to reconnect
    let new_token = timeout(Duration::from_secs(15), async {
        loop {
            if !state.connections.is_empty() {
                let t = state
                    .connections
                    .iter()
                    .next()
                    .expect("should have connection")
                    .key()
                    .clone();
                return t;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .expect("CLI did not reconnect after server disconnect");

    // 10. Send another webhook after reconnection
    let webhook_url_new = format!("{}/in/{}/", base_url, new_token);

    // Delete previous webhooks to avoid confusion
    let _ = client
        .delete(&format!("{}/api/tokens/{}/webhooks", base_url, new_token))
        .send()
        .await;

    let send_resp2 = client
        .post(&webhook_url_new)
        .header("content-type", "application/json")
        .body(r#"{"test":"after_reconnect"}"#)
        .send()
        .await
        .expect("Failed to send webhook after reconnect");

    assert_eq!(send_resp2.status(), 200);

    // 11. Verify the webhook was forwarded after reconnection
    let webhook2 =
        wait_for_webhook_with_response(&base_url, &new_token, Duration::from_secs(10)).await;
    assert_eq!(webhook2["forwarded"], true);
    assert_eq!(webhook2["response"]["status"], 200);

    // Cleanup
    child.kill().await.ok();
}

/// Test: CLI detects token collision and regenerates a new token.
/// 1. Start hooks server + echo server
/// 2. Pre-register a token in `state.storage` as connected (simulate collision)
/// 3. Spawn CLI with `--token` set to the colliding token
/// 4. Wait for CLI to connect with a DIFFERENT token
/// 5. Send webhook to the new token → verify forwarded
#[tokio::test]
async fn test_token_collision_regenerates_token() {
    // 1. Start hooks server
    let (hooks_addr, state) = start_hooks_server().await;
    let base_url = format!("http://{}", hooks_addr);

    // 2. Pre-register a token as "connected" to simulate collision
    // Token format: "c_" + exactly 27 base62 chars
    let colliding_token = "c_CollisionTestToken123456789";
    // Create a dummy sender to mark the token as "in use"
    let (dummy_tx, _dummy_rx) = tokio::sync::mpsc::channel::<String>(1);
    state
        .connections
        .insert(colliding_token.to_string(), dummy_tx);
    state
        .storage
        .set_connected(colliding_token, Some("127.0.0.1".to_string()));

    // 3. Start echo server
    let echo_config = EchoConfig {
        status: StatusCode::OK,
        body: r#"{"echo":"ok"}"#.to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
    };
    let echo_addr = start_echo_server(echo_config).await;

    // 4. Spawn CLI with the colliding token
    let relay_url = format!("ws://{}/ws", hooks_addr);
    let mut child = spawn_cli(echo_addr.port(), &relay_url, &["--token", colliding_token]);

    // 5. Wait for CLI to connect with a different token (collision forced regeneration)
    let new_token = timeout(
        Duration::from_secs(15),
        wait_for_new_connection(&state, colliding_token, Duration::from_secs(15)),
    )
    .await
    .expect("CLI did not connect with a new token after collision");

    assert_ne!(
        new_token, colliding_token,
        "CLI should have generated a new token after collision"
    );

    // 6. Send webhook to the new token
    let client = reqwest::Client::new();
    let webhook_url = format!("{}/in/{}/", base_url, new_token);

    let send_resp = client
        .post(&webhook_url)
        .header("content-type", "application/json")
        .body(r#"{"test":"after_collision"}"#)
        .send()
        .await
        .expect("Failed to send webhook to new token");

    assert_eq!(send_resp.status(), 200);

    // 7. Verify forwarded
    let webhook =
        wait_for_webhook_with_response(&base_url, &new_token, Duration::from_secs(10)).await;
    assert_eq!(webhook["forwarded"], true);
    assert_eq!(webhook["response"]["status"], 200);

    // Cleanup
    child.kill().await.ok();
}

/// Test: CLI reconnects after connection drop detected via ping failure.
/// Uses a TCP proxy and short ping interval to speed up detection.
///
/// 1. Start hooks server + echo server + TCP proxy + CLI (with short ping interval)
/// 2. Wait for CLI to connect
/// 3. Verify baseline webhook works
/// 4. Shut down proxy → connection drops → CLI detects via failed ping
/// 5. Restart proxy → CLI reconnects
/// 6. Verify webhooks work after reconnection
#[tokio::test]
async fn test_reconnect_after_connection_drop() {
    // 1. Start hooks server
    let (hooks_addr, state) = start_hooks_server().await;
    let base_url = format!("http://{}", hooks_addr);

    // 2. Start echo server
    let echo_config = EchoConfig {
        status: StatusCode::OK,
        body: r#"{"echo":"ok"}"#.to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
    };
    let echo_addr = start_echo_server(echo_config).await;

    // 3. Start TCP proxy
    let proxy = TcpProxy::start(hooks_addr).await;
    let proxy_addr = proxy.addr;

    // 4. Spawn CLI with short ping interval to speed up detection
    let relay_url = format!("ws://{}/ws", proxy_addr);
    let mut child = spawn_cli(echo_addr.port(), &relay_url, &["--ping-interval", "2"]);

    // 5. Wait for CLI to connect
    let token = timeout(
        Duration::from_secs(10),
        wait_for_connection(&state, Duration::from_secs(10)),
    )
    .await
    .expect("CLI connection timed out");

    // 6. Verify baseline works
    let client = reqwest::Client::new();
    let webhook_url = format!("{}/in/{}/", base_url, token);

    let send_resp = client
        .post(&webhook_url)
        .header("content-type", "application/json")
        .body(r#"{"test":"before_drop"}"#)
        .send()
        .await
        .expect("Failed to send webhook");
    assert_eq!(send_resp.status(), 200);

    let webhook = wait_for_webhook_with_response(&base_url, &token, Duration::from_secs(10)).await;
    assert_eq!(webhook["forwarded"], true);

    // 7. Shut down proxy → severs connection
    proxy.shutdown();

    // Give time for detection
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Wait for server to clean up old connection
    let cleanup_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while !state.connections.is_empty() {
        if tokio::time::Instant::now() >= cleanup_deadline {
            state.connections.remove(&token);
            state.storage.set_disconnected(&token);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 8. Restart proxy on the same port
    let _proxy = TcpProxy::restart_on(proxy_addr, hooks_addr).await;

    // 9. Wait for CLI to reconnect
    let new_token = timeout(Duration::from_secs(30), async {
        loop {
            if !state.connections.is_empty() {
                let t = state
                    .connections
                    .iter()
                    .next()
                    .expect("should have connection")
                    .key()
                    .clone();
                return t;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .expect("CLI did not reconnect after connection drop");

    // 10. Verify webhooks work after reconnection
    let webhook_url_new = format!("{}/in/{}/", base_url, new_token);

    // Delete previous webhooks
    let _ = client
        .delete(&format!("{}/api/tokens/{}/webhooks", base_url, new_token))
        .send()
        .await;

    let send_resp2 = client
        .post(&webhook_url_new)
        .header("content-type", "application/json")
        .body(r#"{"test":"after_drop_reconnect"}"#)
        .send()
        .await
        .expect("Failed to send webhook after reconnect");
    assert_eq!(send_resp2.status(), 200);

    let webhook2 =
        wait_for_webhook_with_response(&base_url, &new_token, Duration::from_secs(10)).await;
    assert_eq!(webhook2["forwarded"], true);
    assert_eq!(webhook2["response"]["status"], 200);

    // Cleanup
    child.kill().await.ok();
}
