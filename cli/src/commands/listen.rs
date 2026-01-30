//! Local webhook listener command - forwards webhooks to localhost via WebSocket tunnel

use anyhow::{anyhow, Result};
use clap::Args;
use console::style;
use futures_util::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, warn};

use crate::tunnel::{
    forward_request, generate_token, reconnect_loop, ClientMessage, ConnectionInfo, ServerMessage,
    SessionEnd, READ_TIMEOUT,
};
use crate::Cli;

/// Default hooks relay server URL
const DEFAULT_RELAY_URL: &str = "wss://play.hook0.com/ws";

#[derive(Args, Debug)]
pub struct ListenArgs {
    /// Local URL or port to forward webhooks to (auto-detects if not specified)
    /// Examples: 3000, http://localhost:3000/webhooks
    #[arg()]
    pub target: Option<String>,

    /// Hooks relay server URL (WebSocket endpoint)
    #[arg(long, env = "HOOK0_RELAY_URL", default_value = DEFAULT_RELAY_URL)]
    pub relay_url: String,

    /// Token to use (if not provided, a new one will be generated)
    #[arg(long)]
    pub token: Option<String>,

    /// Ping interval in seconds
    #[arg(long, default_value = "30")]
    pub ping_interval: u64,

    /// Disable TLS certificate verification (for self-signed certs)
    #[arg(long)]
    pub insecure: bool,

    /// Allow forwarding to non-localhost targets (external URLs)
    #[arg(long)]
    pub allow_external: bool,

    /// Disable full-screen TUI mode (use plain log output instead)
    #[arg(long)]
    #[cfg(feature = "tui")]
    pub no_tui: bool,
}

const COMMON_PORTS: &[u16] = &[3000, 8000, 8080, 4000, 5000, 5173, 8888];

async fn detect_local_server() -> Option<u16> {
    for &port in COMMON_PORTS {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Some(port);
        }
    }
    None
}

pub async fn execute(_cli: &Cli, args: &ListenArgs) -> Result<()> {
    // Parse target URL, with auto-detection when not specified
    let target_url = match &args.target {
        Some(t) => parse_target_url(t)?,
        None => {
            if let Some(port) = detect_local_server().await {
                eprintln!(
                    "  {} Auto-detected local server on port {}",
                    style("✓").green(),
                    style(port).cyan()
                );
                format!("http://localhost:{}", port)
            } else {
                eprintln!(
                    "  {} No local server detected, using default port 3000",
                    style("→").dim()
                );
                "http://localhost:3000".to_string()
            }
        }
    };

    // Warn if forwarding to non-localhost
    if !is_localhost_url(&target_url) {
        if !args.allow_external {
            eprintln!(
                "  {} Forwarding to external URL: {}",
                style("⚠").yellow().bold(),
                style(&target_url).yellow()
            );
            eprintln!(
                "  {} This could be used as an SSRF vector. Use --allow-external to confirm.",
                style("⚠").yellow().bold()
            );
            return Err(anyhow!(
                "Forwarding to non-localhost target requires --allow-external flag"
            ));
        }
        eprintln!(
            "  {} Forwarding to external URL: {} (--allow-external)",
            style("⚠").yellow().bold(),
            style(&target_url).yellow()
        );
    }

    // Generate or use provided token
    let token = args.token.clone().unwrap_or_else(generate_token);

    // Determine if TUI mode is active (default when feature is enabled)
    #[cfg(feature = "tui")]
    let use_tui = !args.no_tui;
    #[cfg(not(feature = "tui"))]
    let use_tui = false;

    // Create HTTP client for forwarding
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(args.insecure)
        .build()?;

    let ping_interval = Duration::from_secs(args.ping_interval);
    let relay_url = args.relay_url.clone();

    // TUI mode
    #[cfg(feature = "tui")]
    if use_tui {
        return run_tui_with_reconnect(&relay_url, token, &http_client, &target_url, ping_interval)
            .await;
    }

    // Non-TUI mode: show header once, then reconnect loop
    if !use_tui {
        println!();
        println!(
            "  {}",
            style("Hook0 — Local Webhook Listener").bold().cyan()
        );
        println!("  {}", style("─".repeat(40)).dim());
        println!();
    }

    let target_url_clone = target_url.clone();
    reconnect_loop(&relay_url, token, |info: ConnectionInfo| {
        let http_client = http_client.clone();
        let target_url = target_url_clone.clone();
        async move { run_plain_session(info, &http_client, &target_url, ping_interval).await }
    })
    .await
}

/// Run a single non-TUI session within the reconnect loop
async fn run_plain_session(
    info: ConnectionInfo,
    http_client: &reqwest::Client,
    target_url: &str,
    ping_interval: Duration,
) -> Result<SessionEnd> {
    let tx = info.tx;
    let mut read = info.read;

    if info.is_reconnection {
        eprintln!(
            "  {} {}",
            style("●").green(),
            style(format!("Reconnected (attempt {})", info.reconnect_count)).green()
        );
    }

    // Display connection summary
    eprintln!(
        "  {} {} Connected to relay",
        style("[1/2]").dim(),
        style("●").green()
    );
    eprintln!("  {} {} Ready", style("[2/2]").dim(), style("●").green());

    println!();
    println!("  {}", style("─".repeat(40)).dim());
    println!(
        "  {}  {}",
        style("IN ").dim().bold(),
        style(&info.webhook_url).cyan().underlined()
    );
    println!(
        "  {}  {}",
        style("OUT").dim().bold(),
        style(target_url).yellow()
    );
    println!(
        "  {}  {}",
        style("WEB").dim().bold(),
        style(&info.view_url).dim()
    );
    println!("  {}", style("─".repeat(40)).dim());
    println!();

    if !info.is_reconnection {
        println!(
            "  {} {}",
            style("ℹ").blue(),
            style("No login required. Use 'hook0 login' for full API access.").dim()
        );
        println!();
    }

    println!(
        "  {}",
        style("Waiting for webhooks... (Ctrl+C to stop)").dim()
    );
    println!();

    // Setup ping interval
    let mut ping_timer = tokio::time::interval(ping_interval);
    ping_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    // Main event loop with read timeout
    loop {
        tokio::select! {
            // Handle incoming messages with read timeout
            msg = async {
                tokio::time::timeout(READ_TIMEOUT, read.next()).await
            } => {
                match msg {
                    Err(_) => {
                        // Read timeout — assume dead connection
                        warn!("Read timeout ({}s), reconnecting", READ_TIMEOUT.as_secs());
                        eprintln!(
                            "  {} {}",
                            style("⟳").dim(),
                            style("Connection stale, reconnecting...").dim()
                        );
                        return Ok(SessionEnd::Disconnected);
                    }
                    Ok(Some(Ok(Message::Text(text)))) => {
                        let server_msg: ServerMessage = serde_json::from_str(&text)?;
                        match handle_server_message(&tx, http_client, target_url, server_msg).await {
                            Ok(None) => {}
                            Ok(Some(session_end)) => return Ok(session_end),
                            Err(e) => {
                                warn!("Error handling message: {}", e);
                            }
                        }
                    }
                    Ok(Some(Ok(Message::Close(_)))) => {
                        debug!("Connection closed by server");
                        eprintln!(
                            "  {} {}",
                            style("⟳").dim(),
                            style("Server closed connection, reconnecting...").dim()
                        );
                        return Ok(SessionEnd::Disconnected);
                    }
                    Ok(Some(Ok(_))) => {}
                    Ok(Some(Err(e))) => {
                        warn!("WebSocket error: {}", e);
                        eprintln!(
                            "  {} {}",
                            style("⟳").dim(),
                            style(format!("Connection error, reconnecting... ({})", e)).dim()
                        );
                        return Ok(SessionEnd::Disconnected);
                    }
                    Ok(None) => {
                        debug!("Connection closed");
                        eprintln!(
                            "  {} {}",
                            style("⟳").dim(),
                            style("Connection lost, reconnecting...").dim()
                        );
                        return Ok(SessionEnd::Disconnected);
                    }
                }
            }
            // Send periodic pings via mpsc channel
            _ = ping_timer.tick() => {
                let ping_msg = ClientMessage::ping();
                let ping_json = serde_json::to_string(&ping_msg)?;
                if tx.send(ping_json).await.is_err() {
                    return Ok(SessionEnd::Disconnected);
                }
                debug!("Sent ping");
            }
        }
    }
}

/// Handle a message from the server (non-TUI mode).
/// Returns `None` to continue, `Some(SessionEnd)` to end the session.
async fn handle_server_message(
    tx: &mpsc::Sender<String>,
    http_client: &reqwest::Client,
    target_url: &str,
    msg: ServerMessage,
) -> Result<Option<SessionEnd>> {
    match msg {
        ServerMessage::Request { data, .. } => {
            let request_id = data.id.clone();
            let method = data.method.clone();
            let path = data.path.clone();

            // Display incoming request
            print!(
                "  {} {} {}{}",
                style("←").cyan(),
                style(&method).bold(),
                path,
                if let Some(ref q) = data.query {
                    format!("?{}", q)
                } else {
                    String::new()
                }
            );

            // Forward to local server
            match forward_request(http_client, target_url, &data).await {
                Ok(result) => {
                    let status_style = if result.status < 400 {
                        style(result.status).green()
                    } else {
                        style(result.status).red()
                    };

                    println!(
                        "  {} {} ({}ms)",
                        style("→").dim(),
                        status_style,
                        result.duration.as_millis()
                    );

                    // Send response back to server via mpsc channel
                    let response_msg = ClientMessage::response(
                        request_id,
                        result.status,
                        result.headers,
                        result.body,
                    );
                    let response_json = serde_json::to_string(&response_msg)?;
                    if tx.send(response_json).await.is_err() {
                        return Ok(Some(SessionEnd::Disconnected));
                    }
                }
                Err(e) => {
                    println!(
                        "  {} {} - {}",
                        style("→").dim(),
                        style("ERR").red().bold(),
                        e
                    );
                    warn!("Failed to forward request: {}", e);

                    // Send error response via mpsc channel
                    let error_body = format!(
                        r#"{{"error": "forwarding_failed", "message": "{}"}}"#,
                        e.to_string().replace('"', "\\\"")
                    );
                    let response_msg = ClientMessage::response(
                        request_id,
                        502,
                        std::iter::once((
                            "content-type".to_string(),
                            "application/json".to_string(),
                        ))
                        .collect(),
                        error_body.into_bytes(),
                    );
                    let response_json = serde_json::to_string(&response_msg)?;
                    if tx.send(response_json).await.is_err() {
                        return Ok(Some(SessionEnd::Disconnected));
                    }
                }
            }
        }
        ServerMessage::Error { data, .. } => {
            if data.code == "token_in_use" {
                return Ok(Some(SessionEnd::TokenCollision));
            }
            println!(
                "  {} Server error: {} - {}",
                style("✗").red(),
                data.code,
                data.message
            );
        }
        ServerMessage::Pong => {
            debug!("Received pong");
        }
        ServerMessage::Started { .. } => {
            // Already handled during connection setup
        }
    }

    Ok(None)
}

/// Run TUI mode — the TUI owns its own reconnection loop and stays alive throughout.
#[cfg(feature = "tui")]
async fn run_tui_with_reconnect(
    relay_url: &str,
    token: String,
    http_client: &reqwest::Client,
    target_url: &str,
    ping_interval: Duration,
) -> Result<()> {
    crate::tui::run_tui(relay_url, token, http_client, target_url, ping_interval).await
}

/// Parse the target URL from user input
fn parse_target_url(target: &str) -> Result<String> {
    // If it's just a port number
    if target.parse::<u16>().is_ok() {
        return Ok(format!("http://localhost:{}", target));
    }

    // If it already has a scheme
    if target.starts_with("http://") || target.starts_with("https://") {
        return Ok(target.to_string());
    }

    // Assume http:// if no scheme
    Ok(format!("http://{}", target))
}

/// Check if a URL points to localhost
fn is_localhost_url(url: &str) -> bool {
    // Strip scheme
    let host_part = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);

    // Strip path and port
    let host = host_part.split('/').next().unwrap_or(host_part);
    let host = host.split(':').next().unwrap_or(host);

    matches!(
        host,
        "localhost" | "127.0.0.1" | "::1" | "[::1]" | "0.0.0.0"
    )
}
