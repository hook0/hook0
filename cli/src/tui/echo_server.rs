//! Built-in echo server for the example TUI.
//!
//! Starts an axum HTTP server on a free port. Responds to any request with
//! an enriched JSON body containing the received payload metadata.
//! The response status code is configurable via a `watch` channel.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::any;
use axum::Router;
use tokio::sync::watch;
use tokio::task::JoinHandle;

/// Shared state for the echo handler.
#[derive(Clone)]
struct EchoState {
    status_rx: watch::Receiver<u16>,
}

/// Start the echo server on a free port.
///
/// Returns `(port, status_tx, server_handle)`:
/// - `port`: the port the server is listening on
/// - `status_tx`: send a new status code to change the echo response status
/// - `server_handle`: abort this to stop the server
pub fn start_echo_server() -> (u16, watch::Sender<u16>, JoinHandle<()>) {
    let port = portpicker::pick_unused_port().expect("no free port available");
    let (status_tx, status_rx) = watch::channel(200u16);

    let state = EchoState { status_rx };

    let app = Router::new()
        .route("/", any(echo_handler))
        .route("/{*path}", any(echo_handler))
        .with_state(state);

    let listener_addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    let handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(listener_addr)
            .await
            .expect("failed to bind echo server");
        axum::serve(listener, app)
            .await
            .expect("echo server failed");
    });

    (port, status_tx, handle)
}

/// Echo handler: returns enriched JSON with request metadata.
async fn echo_handler(
    State(state): State<EchoState>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let status_code = *state.status_rx.borrow();
    let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK);

    let event_type = headers
        .get("x-hook0-event-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let response_body = serde_json::json!({
        "received": true,
        "event_type": event_type,
        "method": method.as_str(),
        "payload_size": body.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "echo_status": status_code,
    });

    let response_json = serde_json::to_string_pretty(&response_body)
        .unwrap_or_else(|_| r#"{"received":true}"#.to_string());

    (
        status,
        [("content-type", "application/json")],
        response_json,
    )
}
