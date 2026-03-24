//! Shared test helpers for E2E tests.
//! Provides: hooks server startup, echo server, CLI spawning, TCP proxy, and polling utilities.

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::any,
    Router,
};
use serde_json::Value;
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use hook0_play::{create_app, AppState};

/// Start the hooks server on a random available port.
/// Returns (address, shared state for polling).
pub async fn start_hooks_server() -> (SocketAddr, Arc<AppState>) {
    let port = portpicker::pick_unused_port().expect("No free port available");
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let state = Arc::new(AppState::new(format!("http://{}", addr)));
    let app = create_app(state.clone());

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind hooks server");
    let local_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // Wait for server readiness
    tokio::time::sleep(Duration::from_millis(100)).await;

    (local_addr, state)
}

/// Configuration for the echo server response.
#[derive(Clone)]
pub struct EchoConfig {
    pub status: StatusCode,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

/// Start an echo HTTP server on a random port.
/// The server responds to any request with the configured status/body/headers.
pub async fn start_echo_server(config: EchoConfig) -> SocketAddr {
    let shared = Arc::new(config);

    let app = Router::new()
        .route("/{*path}", any(echo_handler))
        .route("/", any(echo_handler))
        .with_state(shared);

    let port = portpicker::pick_unused_port().expect("No free port available");
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind echo server");
    let local_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Wait for server readiness
    tokio::time::sleep(Duration::from_millis(50)).await;

    local_addr
}

/// Echo handler: responds with configured status/body/headers.
async fn echo_handler(
    State(config): State<Arc<EchoConfig>>,
    _method: Method,
    _headers: HeaderMap,
    _body: Bytes,
) -> impl IntoResponse {
    let mut response = (config.status, config.body.clone()).into_response();

    for (name, value) in &config.headers {
        response.headers_mut().insert(
            name.parse::<axum::http::HeaderName>()
                .expect("valid header name"),
            value.parse().expect("valid header value"),
        );
    }

    response
}

/// A TCP proxy that forwards connections to a target address.
/// Shutting down the proxy severs all active connections (RST/FIN).
pub struct TcpProxy {
    pub addr: SocketAddr,
    shutdown_tx: broadcast::Sender<()>,
}

impl TcpProxy {
    /// Start a TCP proxy that forwards connections to `target_addr`.
    pub async fn start(target_addr: SocketAddr) -> Self {
        let port = portpicker::pick_unused_port().expect("No free port available");
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let listener = TcpListener::bind(addr)
            .await
            .expect("Failed to bind TCP proxy");
        let local_addr = listener.local_addr().unwrap();

        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        let shutdown = shutdown_tx.clone();

        tokio::spawn(async move {
            loop {
                let mut shutdown_rx = shutdown.subscribe();
                tokio::select! {
                    result = listener.accept() => {
                        let (client, _) = result.expect("Accept failed");
                        let mut conn_shutdown = shutdown.subscribe();
                        let target = target_addr;
                        tokio::spawn(async move {
                            let server = match tokio::net::TcpStream::connect(target).await {
                                Ok(s) => s,
                                Err(_) => return,
                            };
                            let (mut cr, mut cw) = client.into_split();
                            let (mut sr, mut sw) = server.into_split();

                            tokio::select! {
                                _ = tokio::io::copy(&mut cr, &mut sw) => {}
                                _ = tokio::io::copy(&mut sr, &mut cw) => {}
                                _ = conn_shutdown.recv() => {
                                    // Shutdown: close both sides
                                    let _ = sw.shutdown().await;
                                    let _ = cw.shutdown().await;
                                }
                            }
                        });
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Wait for proxy readiness
        tokio::time::sleep(Duration::from_millis(50)).await;

        Self {
            addr: local_addr,
            shutdown_tx,
        }
    }

    /// Shut down the proxy, severing all active connections.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Restart the proxy on a specific address (after the previous proxy was shut down).
    /// Returns a new TcpProxy at the given address.
    pub async fn restart_on(addr: SocketAddr, target_addr: SocketAddr) -> Self {
        // Wait for the old listener to be released
        tokio::time::sleep(Duration::from_millis(200)).await;

        let listener = TcpListener::bind(addr)
            .await
            .expect("Failed to rebind TCP proxy");
        let local_addr = listener.local_addr().unwrap();

        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        let shutdown = shutdown_tx.clone();

        tokio::spawn(async move {
            loop {
                let mut shutdown_rx = shutdown.subscribe();
                tokio::select! {
                    result = listener.accept() => {
                        let (client, _) = result.expect("Accept failed");
                        let mut conn_shutdown = shutdown.subscribe();
                        let target = target_addr;
                        tokio::spawn(async move {
                            let server = match tokio::net::TcpStream::connect(target).await {
                                Ok(s) => s,
                                Err(_) => return,
                            };
                            let (mut cr, mut cw) = client.into_split();
                            let (mut sr, mut sw) = server.into_split();

                            tokio::select! {
                                _ = tokio::io::copy(&mut cr, &mut sw) => {}
                                _ = tokio::io::copy(&mut sr, &mut cw) => {}
                                _ = conn_shutdown.recv() => {
                                    let _ = sw.shutdown().await;
                                    let _ = cw.shutdown().await;
                                }
                            }
                        });
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Wait for proxy readiness
        tokio::time::sleep(Duration::from_millis(50)).await;

        Self {
            addr: local_addr,
            shutdown_tx,
        }
    }
}

/// Spawn the CLI binary as a child process (fully black-box).
/// Accepts extra arguments after the standard listen args.
pub fn spawn_cli(echo_port: u16, relay_url: &str, extra_args: &[&str]) -> tokio::process::Child {
    let bin = env!("CARGO_BIN_EXE_hook0");
    let mut cmd = tokio::process::Command::new(bin);
    cmd.args([
        "listen",
        &echo_port.to_string(),
        "--relay-url",
        relay_url,
        "--no-tui",
    ]);
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI")
}

/// Wait until the hooks server has an active WebSocket connection.
/// Returns the token of the connected client.
pub async fn wait_for_connection(state: &AppState, max_wait: Duration) -> String {
    let deadline = tokio::time::Instant::now() + max_wait;

    loop {
        // Check DashMap for any connection
        if !state.connections.is_empty() {
            let token = state
                .connections
                .iter()
                .next()
                .expect("connections should not be empty")
                .key()
                .clone();
            return token;
        }

        if tokio::time::Instant::now() >= deadline {
            panic!("Timed out waiting for CLI to connect to hooks server");
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Wait until the hooks server has an active WebSocket connection with a different token.
/// Returns the new token.
pub async fn wait_for_new_connection(
    state: &AppState,
    old_token: &str,
    max_wait: Duration,
) -> String {
    let deadline = tokio::time::Instant::now() + max_wait;

    loop {
        for entry in state.connections.iter() {
            if entry.key() != old_token {
                return entry.key().clone();
            }
        }

        if tokio::time::Instant::now() >= deadline {
            panic!("Timed out waiting for new connection with different token");
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Poll the inspection API until a webhook has a response attached.
/// Returns the webhook JSON value.
pub async fn wait_for_webhook_with_response(
    base_url: &str,
    token: &str,
    max_wait: Duration,
) -> Value {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tokens/{}/webhooks", base_url, token);
    let deadline = tokio::time::Instant::now() + max_wait;

    loop {
        let resp = client
            .get(&url)
            .send()
            .await
            .expect("Failed to call inspection API");

        let json: Value = resp
            .json()
            .await
            .expect("Failed to parse inspection API response");

        let webhooks = json["webhooks"]
            .as_array()
            .expect("webhooks should be an array");

        // Look for a webhook that has a response
        for wh in webhooks {
            if wh["response"].is_object() {
                return wh.clone();
            }
        }

        if tokio::time::Instant::now() >= deadline {
            panic!(
                "Timed out waiting for webhook response. Current webhooks: {}",
                serde_json::to_string_pretty(&json).unwrap_or_default()
            );
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// Poll the inspection API until we have N webhooks with responses.
pub async fn wait_for_n_webhooks_with_response(
    base_url: &str,
    token: &str,
    n: usize,
    max_wait: Duration,
) -> Vec<Value> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tokens/{}/webhooks", base_url, token);
    let deadline = tokio::time::Instant::now() + max_wait;

    loop {
        let resp = client
            .get(&url)
            .send()
            .await
            .expect("Failed to call inspection API");

        let json: Value = resp
            .json()
            .await
            .expect("Failed to parse inspection API response");

        let webhooks = json["webhooks"]
            .as_array()
            .expect("webhooks should be an array");

        let with_response: Vec<Value> = webhooks
            .iter()
            .filter(|wh| wh["response"].is_object())
            .cloned()
            .collect();

        if with_response.len() >= n {
            return with_response;
        }

        if tokio::time::Instant::now() >= deadline {
            panic!(
                "Timed out waiting for {} webhook responses (got {}). Current: {}",
                n,
                with_response.len(),
                serde_json::to_string_pretty(&json).unwrap_or_default()
            );
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
