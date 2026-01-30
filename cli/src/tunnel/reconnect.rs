//! Resilient WebSocket reconnection engine.
//!
//! Provides auto-reconnection with exponential backoff, read timeout (watchdog),
//! token collision handling, and mpsc channel decoupling for reader/writer.

use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, warn};

use super::message::{ClientMessage, ServerMessage};
use super::token::generate_token;

/// Why a connection session ended
#[derive(Debug)]
pub enum SessionEnd {
    /// Server closed or network error — reconnect with same token
    Disconnected,
    /// Server says token_in_use — regenerate token and reconnect
    TokenCollision,
    /// User requested quit (q key, Ctrl+C)
    Quit,
}

/// Information about the current connection, passed to the session function
pub struct ConnectionInfo {
    /// Write channel sender — send serialized JSON strings here
    pub tx: mpsc::Sender<String>,
    /// Read half of the WebSocket
    pub read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    /// Webhook URL assigned by server
    pub webhook_url: String,
    /// View URL assigned by server
    pub view_url: String,
    /// The token used for this connection
    pub token: String,
    /// Whether this is a reconnection (not the first attempt)
    pub is_reconnection: bool,
    /// The current reconnect count (0 = first connection)
    pub reconnect_count: u32,
}

/// Backoff schedule
const BACKOFF: &[Duration] = &[
    Duration::ZERO,
    Duration::from_millis(100),
    Duration::from_millis(1000),
    Duration::from_millis(5000),
];

/// Read timeout: if no WS message for this long, assume dead connection
pub const READ_TIMEOUT: Duration = Duration::from_secs(45);

/// Perform WebSocket handshake: connect, send Start, wait for Started.
/// Returns (write_half, read_half, webhook_url, view_url) or SessionEnd on failure.
async fn handshake(
    relay_url: &str,
    token: &str,
) -> Result<
    (
        futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        String,
        String,
    ),
    HandshakeError,
> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(relay_url)
        .await
        .map_err(|e| HandshakeError::ConnectionFailed(e.into()))?;

    let (mut write, mut read) = ws_stream.split();

    // Send Start message
    let start_msg = ClientMessage::start(token.to_string());
    let start_json =
        serde_json::to_string(&start_msg).map_err(|e| HandshakeError::Fatal(e.into()))?;
    write
        .send(Message::Text(start_json.into()))
        .await
        .map_err(|e| HandshakeError::ConnectionFailed(e.into()))?;

    // Wait for Started confirmation (with timeout)
    let handshake_timeout = Duration::from_secs(10);
    let deadline = tokio::time::Instant::now() + handshake_timeout;

    loop {
        let remaining = deadline - tokio::time::Instant::now();
        let msg = tokio::time::timeout(remaining, read.next())
            .await
            .map_err(|_| {
                HandshakeError::ConnectionFailed(anyhow!("Handshake timed out waiting for Started"))
            })?
            .ok_or_else(|| {
                HandshakeError::ConnectionFailed(anyhow!(
                    "Connection closed before receiving Started"
                ))
            })?
            .map_err(|e| HandshakeError::ConnectionFailed(e.into()))?;

        if let Message::Text(text) = msg {
            let server_msg: ServerMessage = serde_json::from_str(&text)
                .map_err(|e| HandshakeError::Fatal(anyhow!("Invalid server message: {}", e)))?;
            match server_msg {
                ServerMessage::Started { data, .. } => {
                    return Ok((write, read, data.webhook_url, data.view_url));
                }
                ServerMessage::Error { data, .. } => {
                    if data.code == "token_in_use" {
                        return Err(HandshakeError::TokenCollision);
                    }
                    return Err(HandshakeError::ConnectionFailed(anyhow!(
                        "Server error: {} - {}",
                        data.code,
                        data.message
                    )));
                }
                _ => continue,
            }
        }
    }
}

/// Errors during handshake
#[derive(Debug)]
enum HandshakeError {
    /// Transient error — should retry
    ConnectionFailed(anyhow::Error),
    /// Token already in use — regenerate token
    TokenCollision,
    /// Unrecoverable error
    Fatal(anyhow::Error),
}

/// Run the outer reconnection loop.
/// Calls `session_fn` for each successful connection.
/// Handles backoff, token regeneration, and reconnection.
pub async fn reconnect_loop<F, Fut>(
    relay_url: &str,
    initial_token: String,
    session_fn: F,
) -> Result<()>
where
    F: Fn(ConnectionInfo) -> Fut,
    Fut: std::future::Future<Output = Result<SessionEnd>>,
{
    let mut token = initial_token;
    let mut backoff_index: usize = 0;
    let mut last_connected_at: Option<Instant> = None;
    let mut reconnect_count: u32 = 0;

    loop {
        // Apply backoff delay
        let delay = BACKOFF[backoff_index.min(BACKOFF.len() - 1)];
        if !delay.is_zero() {
            debug!("Reconnecting in {:?} (attempt {})", delay, reconnect_count);
            tokio::time::sleep(delay).await;
        }

        // Attempt handshake
        let handshake_result = handshake(relay_url, &token).await;

        let (write, read, webhook_url, view_url) = match handshake_result {
            Ok(result) => result,
            Err(HandshakeError::TokenCollision) => {
                token = generate_token();
                backoff_index = 0;
                warn!("Token collision detected, regenerated token");
                continue;
            }
            Err(HandshakeError::ConnectionFailed(e)) => {
                warn!("Connection failed: {}", e);
                backoff_index = (backoff_index + 1).min(BACKOFF.len() - 1);
                reconnect_count += 1;
                continue;
            }
            Err(HandshakeError::Fatal(e)) => {
                return Err(e).context("Fatal handshake error");
            }
        };

        // Reset backoff if last connection was long-lived (>10s)
        if let Some(last_t) = last_connected_at {
            if last_t.elapsed() > Duration::from_secs(10) {
                backoff_index = 0;
            }
        }
        last_connected_at = Some(Instant::now());

        // Create mpsc channel for write decoupling
        let (tx, mut rx) = mpsc::channel::<String>(64);

        // Spawn writer task
        let writer_task = tokio::spawn(async move {
            let mut write = write;
            while let Some(msg) = rx.recv().await {
                if write.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        });

        let is_reconnection = reconnect_count > 0;
        let info = ConnectionInfo {
            tx,
            read,
            webhook_url,
            view_url,
            token: token.clone(),
            is_reconnection,
            reconnect_count,
        };

        // Run the session
        let session_result = session_fn(info).await;

        // Abort the writer task
        writer_task.abort();

        match session_result {
            Ok(SessionEnd::Disconnected) => {
                backoff_index = (backoff_index + 1).min(BACKOFF.len() - 1);
                reconnect_count += 1;
                debug!("Session disconnected, will reconnect");
                continue;
            }
            Ok(SessionEnd::TokenCollision) => {
                token = generate_token();
                backoff_index = 0;
                reconnect_count += 1;
                warn!("Token collision during session, regenerated token");
                continue;
            }
            Ok(SessionEnd::Quit) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e).context("Session error");
            }
        }
    }
}
