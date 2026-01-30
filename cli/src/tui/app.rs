//! TUI application state and main event loop
//!
//! The TUI owns the full lifecycle: terminal init, reconnection loop, and cleanup.
//! The terminal stays alive during reconnection attempts so the user always sees
//! a status (Connected / Reconnecting) instead of raw log lines.

use anyhow::{anyhow, Result};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, warn};

use crate::tunnel::{forward_request, generate_token, ClientMessage, ServerMessage, READ_TIMEOUT};

use super::ui;

/// Ordered list of (header_name, header_value)
pub type HeaderMap = Vec<(String, String)>;

/// Maximum body size to store (64 KB)
const MAX_BODY_SIZE: usize = 64 * 1024;

/// Backoff schedule (same as reconnect.rs)
const BACKOFF: &[Duration] = &[
    Duration::ZERO,
    Duration::from_millis(100),
    Duration::from_millis(1000),
    Duration::from_millis(5000),
];

/// A single webhook event record for display
pub struct WebhookEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub request_id: String,
    pub error: Option<String>,
    pub query: Option<String>,
    pub request_headers: HeaderMap,
    pub request_body: Option<Vec<u8>>,
    pub response_headers: HeaderMap,
    pub response_body: Option<Vec<u8>>,
}

/// Which detail tab is active
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailTab {
    Request,
    Response,
}

/// Connection status displayed in the TUI header
#[derive(Clone)]
pub enum ConnectionStatus {
    Connected,
    Reconnecting { attempt: u32 },
}

/// Application state for the TUI
pub struct TuiApp {
    pub webhook_url: String,
    pub target_url: String,
    pub view_url: String,
    pub events: VecDeque<WebhookEvent>,
    pub selected: usize,
    pub active_tab: DetailTab,
    pub body_scroll_offset: u16,
    pub status: ConnectionStatus,
    pub should_quit: bool,
    pub event_count: u64,
    /// Tick counter for animations (wraps around)
    pub tick: u64,
    /// Uptime since TUI started
    pub started_at: Instant,
    /// Success/error counters
    pub success_count: u64,
    pub error_count: u64,
    /// Average response time (running average)
    pub avg_duration_ms: f64,
    /// Number of reconnections
    pub reconnect_count: u32,
}

impl TuiApp {
    const MAX_EVENTS: usize = 500;

    pub(crate) fn new(target_url: String) -> Self {
        Self {
            webhook_url: String::new(),
            target_url,
            view_url: String::new(),
            events: VecDeque::new(),
            selected: 0,
            active_tab: DetailTab::Request,
            body_scroll_offset: 0,
            status: ConnectionStatus::Reconnecting { attempt: 0 },
            should_quit: false,
            event_count: 0,
            tick: 0,
            started_at: Instant::now(),
            success_count: 0,
            error_count: 0,
            avg_duration_ms: 0.0,
            reconnect_count: 0,
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.status, ConnectionStatus::Connected)
    }

    pub fn uptime_str(&self) -> String {
        let secs = self.started_at.elapsed().as_secs();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if h > 0 {
            format!("{h}h{m:02}m{s:02}s")
        } else if m > 0 {
            format!("{m}m{s:02}s")
        } else {
            format!("{s}s")
        }
    }

    pub fn record_event(&mut self, duration_ms: u64, is_error: bool) {
        self.event_count += 1;
        if is_error {
            self.error_count += 1;
        } else {
            self.success_count += 1;
        }
        // Running average
        let n = self.event_count as f64;
        self.avg_duration_ms = self.avg_duration_ms * ((n - 1.0) / n) + (duration_ms as f64 / n);
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn push_event(&mut self, event: WebhookEvent) {
        if self.events.len() >= Self::MAX_EVENTS {
            self.events.pop_front();
            self.selected = self.selected.saturating_sub(1);
        }
        self.events.push_back(event);
        self.selected = self.events.len().saturating_sub(1);
        self.body_scroll_offset = 0;
    }

    fn select_next(&mut self) {
        if !self.events.is_empty() && self.selected < self.events.len() - 1 {
            self.selected += 1;
            self.body_scroll_offset = 0;
        }
    }

    fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.body_scroll_offset = 0;
        }
    }

    fn select_first(&mut self) {
        self.selected = 0;
        self.body_scroll_offset = 0;
    }

    fn select_last(&mut self) {
        if !self.events.is_empty() {
            self.selected = self.events.len() - 1;
            self.body_scroll_offset = 0;
        }
    }

    fn toggle_tab(&mut self) {
        self.active_tab = match self.active_tab {
            DetailTab::Request => DetailTab::Response,
            DetailTab::Response => DetailTab::Request,
        };
        self.body_scroll_offset = 0;
    }

    fn scroll_body_down(&mut self, amount: u16) {
        self.body_scroll_offset = self.body_scroll_offset.saturating_add(amount);
    }

    fn scroll_body_up(&mut self, amount: u16) {
        self.body_scroll_offset = self.body_scroll_offset.saturating_sub(amount);
    }
}

/// Run the TUI with its own reconnection loop.
/// The terminal stays alive during reconnection so the user always sees status updates.
pub async fn run_tui(
    relay_url: &str,
    initial_token: String,
    http_client: &reqwest::Client,
    target_url: &str,
    ping_interval: Duration,
) -> Result<()> {
    // Initialize terminal once
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = TuiApp::new(target_url.to_string());

    let result = run_tui_reconnect_loop(
        &mut terminal,
        &mut app,
        relay_url,
        initial_token,
        http_client,
        target_url,
        ping_interval,
    )
    .await;

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

/// Inner reconnection loop that keeps the TUI alive.
async fn run_tui_reconnect_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut TuiApp,
    relay_url: &str,
    initial_token: String,
    http_client: &reqwest::Client,
    target_url: &str,
    ping_interval: Duration,
) -> Result<()> {
    let mut token = initial_token;
    let mut backoff_index: usize = 0;
    let mut last_connected_at: Option<Instant> = None;

    // Initial render
    terminal.draw(|f| ui::render(f, app))?;

    loop {
        if app.should_quit {
            return Ok(());
        }

        // Apply backoff delay while keeping TUI alive (animations + keyboard)
        let delay = BACKOFF[backoff_index.min(BACKOFF.len() - 1)];
        if !delay.is_zero() {
            app.status = ConnectionStatus::Reconnecting {
                attempt: app.reconnect_count,
            };
            terminal.draw(|f| ui::render(f, app))?;

            if wait_with_tui(terminal, app, delay).await? {
                return Ok(()); // user quit
            }
        }

        // Attempt handshake
        app.status = ConnectionStatus::Reconnecting {
            attempt: app.reconnect_count,
        };
        terminal.draw(|f| ui::render(f, app))?;

        let handshake_result = handshake_with_tui(terminal, app, relay_url, &token).await;

        let (write, read, webhook_url, view_url) = match handshake_result {
            Ok(result) => result,
            Err(HandshakeOutcome::Quit) => return Ok(()),
            Err(HandshakeOutcome::TokenCollision) => {
                token = generate_token();
                backoff_index = 0;
                continue;
            }
            Err(HandshakeOutcome::ConnectionFailed) => {
                backoff_index = (backoff_index + 1).min(BACKOFF.len() - 1);
                app.reconnect_count += 1;
                continue;
            }
            Err(HandshakeOutcome::Fatal(e)) => return Err(e),
        };

        // Reset backoff if last connection was long-lived (>10s)
        if let Some(last_t) = last_connected_at {
            if last_t.elapsed() > Duration::from_secs(10) {
                backoff_index = 0;
            }
        }
        last_connected_at = Some(Instant::now());

        // Update app state with connection info
        app.webhook_url = webhook_url;
        app.view_url = view_url;
        app.status = ConnectionStatus::Connected;

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

        // Run the active session
        let session_result = run_tui_session(
            terminal,
            app,
            tx,
            read,
            http_client,
            target_url,
            ping_interval,
        )
        .await;

        writer_task.abort();

        match session_result {
            Ok(SessionResult::Disconnected) => {
                app.status = ConnectionStatus::Reconnecting {
                    attempt: app.reconnect_count,
                };
                backoff_index = (backoff_index + 1).min(BACKOFF.len() - 1);
                app.reconnect_count += 1;
                terminal.draw(|f| ui::render(f, app))?;
                continue;
            }
            Ok(SessionResult::TokenCollision) => {
                token = generate_token();
                backoff_index = 0;
                app.reconnect_count += 1;
                continue;
            }
            Ok(SessionResult::Quit) => return Ok(()),
            Err(e) => return Err(e),
        }
    }
}

/// Wait for a duration while keeping the TUI alive (animations + keyboard).
/// Returns true if the user quit during the wait.
async fn wait_with_tui(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut TuiApp,
    duration: Duration,
) -> Result<bool> {
    let deadline = tokio::time::Instant::now() + duration;
    let mut ui_tick = tokio::time::interval(Duration::from_millis(250));
    ui_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut event_stream = EventStream::new();

    loop {
        if app.should_quit {
            return Ok(true);
        }
        if tokio::time::Instant::now() >= deadline {
            return Ok(false);
        }

        tokio::select! {
            term_event = event_stream.next() => {
                match term_event {
                    Some(Ok(Event::Key(key))) => handle_key_event(app, key),
                    Some(Err(_)) | None => return Ok(true),
                    _ => {}
                }
                terminal.draw(|f| ui::render(f, app))?;
            }
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| ui::render(f, app))?;
            }
            _ = tokio::time::sleep_until(deadline) => {
                return Ok(false);
            }
        }
    }
}

enum HandshakeOutcome {
    Quit,
    ConnectionFailed,
    TokenCollision,
    Fatal(anyhow::Error),
}

impl From<std::io::Error> for HandshakeOutcome {
    fn from(e: std::io::Error) -> Self {
        HandshakeOutcome::Fatal(e.into())
    }
}

/// Perform WebSocket handshake while keeping TUI alive.
async fn handshake_with_tui(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut TuiApp,
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
    HandshakeOutcome,
> {
    // Connect with TUI-aware timeout
    let mut ui_tick = tokio::time::interval(Duration::from_millis(250));
    ui_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut event_stream = EventStream::new();

    // Start the async connect
    let connect_fut = tokio_tungstenite::connect_async(relay_url);
    tokio::pin!(connect_fut);

    let ws_stream = loop {
        if app.should_quit {
            return Err(HandshakeOutcome::Quit);
        }
        tokio::select! {
            result = &mut connect_fut => {
                match result {
                    Ok((ws, _)) => break ws,
                    Err(e) => {
                        warn!("Connection failed: {}", e);
                        return Err(HandshakeOutcome::ConnectionFailed);
                    }
                }
            }
            term_event = event_stream.next() => {
                match term_event {
                    Some(Ok(Event::Key(key))) => handle_key_event(app, key),
                    Some(Err(_)) | None => return Err(HandshakeOutcome::Quit),
                    _ => {}
                }
                terminal.draw(|f| ui::render(f, app))?;
            }
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| ui::render(f, app))?;
            }
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Send Start message
    let start_msg = ClientMessage::start(token.to_string());
    let start_json =
        serde_json::to_string(&start_msg).map_err(|e| HandshakeOutcome::Fatal(e.into()))?;
    write
        .send(Message::Text(start_json.into()))
        .await
        .map_err(|_| HandshakeOutcome::ConnectionFailed)?;

    // Wait for Started confirmation with TUI-aware timeout
    let handshake_deadline = tokio::time::Instant::now() + Duration::from_secs(10);

    loop {
        if app.should_quit {
            return Err(HandshakeOutcome::Quit);
        }
        if tokio::time::Instant::now() >= handshake_deadline {
            warn!("Handshake timed out");
            return Err(HandshakeOutcome::ConnectionFailed);
        }

        tokio::select! {
            ws_msg = read.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        let server_msg: ServerMessage = serde_json::from_str(&text)
                            .map_err(|e| HandshakeOutcome::Fatal(anyhow!("Invalid server message: {}", e)))?;
                        match server_msg {
                            ServerMessage::Started { data, .. } => {
                                return Ok((write, read, data.webhook_url, data.view_url));
                            }
                            ServerMessage::Error { data, .. } => {
                                if data.code == "token_in_use" {
                                    return Err(HandshakeOutcome::TokenCollision);
                                }
                                warn!("Server error: {} - {}", data.code, data.message);
                                return Err(HandshakeOutcome::ConnectionFailed);
                            }
                            _ => continue,
                        }
                    }
                    Some(Ok(_)) => continue,
                    Some(Err(_)) | None => {
                        return Err(HandshakeOutcome::ConnectionFailed);
                    }
                }
            }
            term_event = event_stream.next() => {
                match term_event {
                    Some(Ok(Event::Key(key))) => handle_key_event(app, key),
                    Some(Err(_)) | None => return Err(HandshakeOutcome::Quit),
                    _ => {}
                }
                terminal.draw(|f| ui::render(f, app))?;
            }
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| ui::render(f, app))?;
            }
        }
    }
}

/// Session result (internal — not exposed like SessionEnd from tunnel)
enum SessionResult {
    Disconnected,
    TokenCollision,
    Quit,
}

/// Inner TUI session loop (connected state)
async fn run_tui_session(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut TuiApp,
    tx: mpsc::Sender<String>,
    mut read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    http_client: &reqwest::Client,
    target_url: &str,
    ping_interval: Duration,
) -> Result<SessionResult> {
    let mut ping_timer = tokio::time::interval(ping_interval);
    ping_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    // UI tick for animations (4 fps)
    let mut ui_tick = tokio::time::interval(Duration::from_millis(250));
    ui_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut event_stream = EventStream::new();

    // Initial render
    terminal.draw(|f| ui::render(f, app))?;

    loop {
        if app.should_quit {
            return Ok(SessionResult::Quit);
        }

        tokio::select! {
            // Terminal events (keyboard, resize)
            term_event = event_stream.next() => {
                match term_event {
                    Some(Ok(Event::Key(key))) => {
                        handle_key_event(app, key);
                    }
                    Some(Ok(Event::Resize(_, _))) => {}
                    Some(Err(_)) => return Ok(SessionResult::Quit),
                    None => return Ok(SessionResult::Quit),
                    _ => {}
                }
                terminal.draw(|f| ui::render(f, app))?;
            }
            // WebSocket messages with read timeout
            ws_msg = async {
                tokio::time::timeout(READ_TIMEOUT, read.next()).await
            } => {
                match ws_msg {
                    Err(_) => {
                        // Read timeout — assume dead connection
                        warn!("Read timeout ({}s), reconnecting", READ_TIMEOUT.as_secs());
                        return Ok(SessionResult::Disconnected);
                    }
                    Ok(Some(Ok(Message::Text(text)))) => {
                        let server_msg: ServerMessage = serde_json::from_str(&text)?;
                        match handle_server_message_tui(app, &tx, http_client, target_url, server_msg).await {
                            Ok(None) => {}
                            Ok(Some(SessionResult::Disconnected)) => return Ok(SessionResult::Disconnected),
                            Ok(Some(SessionResult::TokenCollision)) => return Ok(SessionResult::TokenCollision),
                            Ok(Some(SessionResult::Quit)) => return Ok(SessionResult::Quit),
                            Err(e) => {
                                warn!("Error handling message: {}", e);
                            }
                        }
                        terminal.draw(|f| ui::render(f, app))?;
                    }
                    Ok(Some(Ok(Message::Close(_)))) | Ok(None) => {
                        return Ok(SessionResult::Disconnected);
                    }
                    Ok(Some(Err(_))) => {
                        return Ok(SessionResult::Disconnected);
                    }
                    Ok(Some(Ok(_))) => {}
                }
            }
            // Ping timer via mpsc channel
            _ = ping_timer.tick() => {
                let ping_msg = ClientMessage::ping();
                let ping_json = serde_json::to_string(&ping_msg)?;
                if tx.send(ping_json).await.is_err() {
                    return Ok(SessionResult::Disconnected);
                }
                debug!("Sent ping");
            }
            // UI animation tick
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| ui::render(f, app))?;
            }
        }
    }
}

pub(crate) fn handle_key_event(app: &mut TuiApp, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true
        }
        // Navigation: arrow keys and lowercase j/k
        KeyCode::Down => app.select_next(),
        KeyCode::Up => app.select_prev(),
        KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::SHIFT) => app.select_next(),
        KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::SHIFT) => app.select_prev(),
        // First/last
        KeyCode::Home | KeyCode::Char('g') => app.select_first(),
        KeyCode::End | KeyCode::Char('G') => app.select_last(),
        // Tab toggle between Request/Response
        KeyCode::Tab => app.toggle_tab(),
        // Body scroll: shift+J / shift+K
        KeyCode::Char('J') => app.scroll_body_down(1),
        KeyCode::Char('K') => app.scroll_body_up(1),
        // Page scroll
        KeyCode::PageDown => app.scroll_body_down(10),
        KeyCode::PageUp => app.scroll_body_up(10),
        _ => {}
    }
}

/// Cap a body to MAX_BODY_SIZE
pub(crate) fn cap_body(body: Vec<u8>) -> Option<Vec<u8>> {
    if body.is_empty() {
        return None;
    }
    if body.len() > MAX_BODY_SIZE {
        Some(body[..MAX_BODY_SIZE].to_vec())
    } else {
        Some(body)
    }
}

/// Convert a HashMap to a sorted HeaderMap
pub(crate) fn sorted_headers(map: &std::collections::HashMap<String, String>) -> HeaderMap {
    let mut headers: HeaderMap = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    headers.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    headers
}

async fn handle_server_message_tui(
    app: &mut TuiApp,
    tx: &mpsc::Sender<String>,
    http_client: &reqwest::Client,
    target_url: &str,
    msg: ServerMessage,
) -> Result<Option<SessionResult>> {
    match msg {
        ServerMessage::Request { data, .. } => {
            let request_id = data.id.clone();
            let method = data.method.clone();
            let path = data.path.clone();
            let query = data.query.clone();

            // Capture request headers (sorted)
            let request_headers = sorted_headers(&data.headers);

            // Decode request body from base64
            let request_body_raw =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data.body)
                    .unwrap_or_default();
            let request_body = cap_body(request_body_raw);

            match forward_request(http_client, target_url, &data).await {
                Ok(result) => {
                    let duration_ms = result.duration.as_millis() as u64;
                    let is_error = result.status >= 400;
                    app.record_event(duration_ms, is_error);

                    // Capture response headers (sorted)
                    let response_headers = sorted_headers(&result.headers);
                    let response_body = cap_body(result.body.clone());

                    app.push_event(WebhookEvent {
                        timestamp: chrono::Utc::now(),
                        method,
                        path,
                        status: result.status,
                        duration_ms,
                        request_id: request_id.clone(),
                        error: None,
                        query,
                        request_headers,
                        request_body,
                        response_headers,
                        response_body,
                    });

                    let response_msg = ClientMessage::response(
                        request_id,
                        result.status,
                        result.headers,
                        result.body,
                    );
                    let response_json = serde_json::to_string(&response_msg)?;
                    if tx.send(response_json).await.is_err() {
                        return Ok(Some(SessionResult::Disconnected));
                    }
                }
                Err(e) => {
                    app.record_event(0, true);

                    let error_body_str = format!(
                        r#"{{"error": "forwarding_failed", "message": "{}"}}"#,
                        e.to_string().replace('"', "\\\"")
                    );
                    let error_body_bytes = error_body_str.clone().into_bytes();

                    let error_response_headers =
                        vec![("content-type".to_string(), "application/json".to_string())];

                    app.push_event(WebhookEvent {
                        timestamp: chrono::Utc::now(),
                        method,
                        path,
                        status: 502,
                        duration_ms: 0,
                        request_id: request_id.clone(),
                        error: Some(e.to_string()),
                        query,
                        request_headers,
                        request_body,
                        response_headers: error_response_headers,
                        response_body: cap_body(error_body_bytes),
                    });

                    let response_msg = ClientMessage::response(
                        request_id,
                        502,
                        std::iter::once((
                            "content-type".to_string(),
                            "application/json".to_string(),
                        ))
                        .collect(),
                        error_body_str.into_bytes(),
                    );
                    let response_json = serde_json::to_string(&response_msg)?;
                    if tx.send(response_json).await.is_err() {
                        return Ok(Some(SessionResult::Disconnected));
                    }
                }
            }
        }
        ServerMessage::Error { data, .. } => {
            if data.code == "token_in_use" {
                return Ok(Some(SessionResult::TokenCollision));
            }
            app.record_event(0, true);
            app.push_event(WebhookEvent {
                timestamp: chrono::Utc::now(),
                method: "ERR".to_string(),
                path: format!("{}: {}", data.code, data.message),
                status: 0,
                duration_ms: 0,
                request_id: String::new(),
                error: Some(data.message),
                query: None,
                request_headers: Vec::new(),
                request_body: None,
                response_headers: Vec::new(),
                response_body: None,
            });
        }
        ServerMessage::Pong => {
            debug!("Received pong");
        }
        ServerMessage::Started { .. } => {}
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_event(method: &str, status: u16, request_id: &str) -> WebhookEvent {
        WebhookEvent {
            timestamp: chrono::Utc::now(),
            method: method.to_string(),
            path: "/test".to_string(),
            status,
            duration_ms: 42,
            request_id: request_id.to_string(),
            error: None,
            query: None,
            request_headers: vec![("content-type".to_string(), "application/json".to_string())],
            request_body: Some(b"hello".to_vec()),
            response_headers: vec![],
            response_body: None,
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_shift(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
    }

    // ── TuiApp::new ──────────────────────────────────────────────

    #[test]
    fn new_app_starts_in_reconnecting_state() {
        let app = TuiApp::new("http://localhost:3000".into());
        assert!(matches!(
            app.status,
            ConnectionStatus::Reconnecting { attempt: 0 }
        ));
        assert!(!app.is_connected());
        assert_eq!(app.events.len(), 0);
        assert_eq!(app.selected, 0);
        assert_eq!(app.event_count, 0);
        assert!(!app.should_quit);
    }

    // ── push_event / MAX_EVENTS cap ──────────────────────────────

    #[test]
    fn push_event_selects_latest() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.push_event(make_event("POST", 200, "aaa"));
        assert_eq!(app.events.len(), 1);
        assert_eq!(app.selected, 0);

        app.push_event(make_event("GET", 200, "bbb"));
        assert_eq!(app.events.len(), 2);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn push_event_resets_body_scroll() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.push_event(make_event("POST", 200, "aaa"));
        app.body_scroll_offset = 10;
        app.push_event(make_event("GET", 200, "bbb"));
        assert_eq!(app.body_scroll_offset, 0);
    }

    #[test]
    fn push_event_caps_at_max() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        for i in 0..TuiApp::MAX_EVENTS + 50 {
            app.push_event(make_event("POST", 200, &format!("id_{i}")));
        }
        assert_eq!(app.events.len(), TuiApp::MAX_EVENTS);
        // The latest event should be selected
        assert_eq!(app.selected, TuiApp::MAX_EVENTS - 1);
        // The oldest events should have been evicted; first remaining is id_50
        assert_eq!(app.events.front().unwrap().request_id, "id_50");
        assert_eq!(
            app.events.back().unwrap().request_id,
            format!("id_{}", TuiApp::MAX_EVENTS + 49)
        );
    }

    // ── Navigation ───────────────────────────────────────────────

    #[test]
    fn select_next_prev() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.push_event(make_event("POST", 200, "a"));
        app.push_event(make_event("GET", 200, "b"));
        app.push_event(make_event("PUT", 200, "c"));
        assert_eq!(app.selected, 2);

        // select_prev walks backward
        handle_key_event(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected, 1);
        handle_key_event(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected, 0);
        // clamped at 0
        handle_key_event(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected, 0);

        // select_next walks forward
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected, 1);
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected, 2);
        // clamped at len-1
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn select_first_last() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        for i in 0..10 {
            app.push_event(make_event("POST", 200, &format!("id_{i}")));
        }
        assert_eq!(app.selected, 9);

        handle_key_event(&mut app, key(KeyCode::Home));
        assert_eq!(app.selected, 0);

        handle_key_event(&mut app, key(KeyCode::End));
        assert_eq!(app.selected, 9);

        // 'g' = first, 'G' = last
        handle_key_event(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.selected, 0);
        handle_key_event(&mut app, key(KeyCode::Char('G')));
        assert_eq!(app.selected, 9);
    }

    #[test]
    fn vim_jk_navigation() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.push_event(make_event("POST", 200, "a"));
        app.push_event(make_event("GET", 200, "b"));
        assert_eq!(app.selected, 1);

        handle_key_event(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.selected, 0);

        handle_key_event(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn navigation_on_empty_list_does_not_panic() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        handle_key_event(&mut app, key(KeyCode::Down));
        handle_key_event(&mut app, key(KeyCode::Up));
        handle_key_event(&mut app, key(KeyCode::Home));
        handle_key_event(&mut app, key(KeyCode::End));
        assert_eq!(app.selected, 0);
    }

    // ── Tab toggle ───────────────────────────────────────────────

    #[test]
    fn toggle_tab_switches_request_response() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        assert_eq!(app.active_tab, DetailTab::Request);

        handle_key_event(&mut app, key(KeyCode::Tab));
        assert_eq!(app.active_tab, DetailTab::Response);

        handle_key_event(&mut app, key(KeyCode::Tab));
        assert_eq!(app.active_tab, DetailTab::Request);
    }

    #[test]
    fn toggle_tab_resets_body_scroll() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.body_scroll_offset = 5;
        handle_key_event(&mut app, key(KeyCode::Tab));
        assert_eq!(app.body_scroll_offset, 0);
    }

    // ── Body scroll ──────────────────────────────────────────────

    #[test]
    fn shift_jk_scrolls_body() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        assert_eq!(app.body_scroll_offset, 0);

        handle_key_event(&mut app, key_shift('J'));
        assert_eq!(app.body_scroll_offset, 1);
        handle_key_event(&mut app, key_shift('J'));
        assert_eq!(app.body_scroll_offset, 2);

        handle_key_event(&mut app, key_shift('K'));
        assert_eq!(app.body_scroll_offset, 1);
        handle_key_event(&mut app, key_shift('K'));
        assert_eq!(app.body_scroll_offset, 0);
        // clamped at 0
        handle_key_event(&mut app, key_shift('K'));
        assert_eq!(app.body_scroll_offset, 0);
    }

    #[test]
    fn page_up_down_scrolls_by_10() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        handle_key_event(&mut app, key(KeyCode::PageDown));
        assert_eq!(app.body_scroll_offset, 10);
        handle_key_event(&mut app, key(KeyCode::PageDown));
        assert_eq!(app.body_scroll_offset, 20);
        handle_key_event(&mut app, key(KeyCode::PageUp));
        assert_eq!(app.body_scroll_offset, 10);
    }

    #[test]
    fn navigation_resets_body_scroll() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.push_event(make_event("POST", 200, "a"));
        app.push_event(make_event("GET", 200, "b"));
        app.body_scroll_offset = 5;

        handle_key_event(&mut app, key(KeyCode::Up));
        assert_eq!(app.body_scroll_offset, 0);
    }

    // ── Quit ─────────────────────────────────────────────────────

    #[test]
    fn q_sets_should_quit() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        assert!(!app.should_quit);
        handle_key_event(&mut app, key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn esc_sets_should_quit() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        handle_key_event(&mut app, key(KeyCode::Esc));
        assert!(app.should_quit);
    }

    #[test]
    fn ctrl_c_sets_should_quit() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(app.should_quit);
    }

    // ── record_event / stats ─────────────────────────────────────

    #[test]
    fn record_event_tracks_success_and_errors() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.record_event(100, false);
        app.record_event(200, false);
        app.record_event(300, true);

        assert_eq!(app.event_count, 3);
        assert_eq!(app.success_count, 2);
        assert_eq!(app.error_count, 1);
    }

    #[test]
    fn record_event_running_average() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.record_event(100, false);
        assert!((app.avg_duration_ms - 100.0).abs() < 0.01);

        app.record_event(200, false);
        assert!((app.avg_duration_ms - 150.0).abs() < 0.01);

        app.record_event(300, false);
        assert!((app.avg_duration_ms - 200.0).abs() < 0.01);
    }

    // ── ConnectionStatus ─────────────────────────────────────────

    #[test]
    fn is_connected_returns_correct_value() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        assert!(!app.is_connected());

        app.status = ConnectionStatus::Connected;
        assert!(app.is_connected());

        app.status = ConnectionStatus::Reconnecting { attempt: 3 };
        assert!(!app.is_connected());
    }

    // ── cap_body ─────────────────────────────────────────────────

    #[test]
    fn cap_body_returns_none_for_empty() {
        assert!(cap_body(vec![]).is_none());
    }

    #[test]
    fn cap_body_returns_data_under_limit() {
        let data = vec![1u8; 100];
        let result = cap_body(data.clone());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn cap_body_truncates_over_limit() {
        let data = vec![42u8; MAX_BODY_SIZE + 1000];
        let result = cap_body(data).unwrap();
        assert_eq!(result.len(), MAX_BODY_SIZE);
    }

    // ── sorted_headers ───────────────────────────────────────────

    #[test]
    fn sorted_headers_sorts_case_insensitive() {
        let mut map = std::collections::HashMap::new();
        map.insert("Content-Type".to_string(), "application/json".to_string());
        map.insert("Accept".to_string(), "*/*".to_string());
        map.insert("x-custom".to_string(), "value".to_string());

        let headers = sorted_headers(&map);
        assert_eq!(headers[0].0, "Accept");
        assert_eq!(headers[1].0, "Content-Type");
        assert_eq!(headers[2].0, "x-custom");
    }

    #[test]
    fn sorted_headers_handles_empty() {
        let map = std::collections::HashMap::new();
        let headers = sorted_headers(&map);
        assert!(headers.is_empty());
    }

    // ── tick ─────────────────────────────────────────────────────

    #[test]
    fn tick_increments() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        assert_eq!(app.tick, 0);
        app.tick();
        assert_eq!(app.tick, 1);
        app.tick();
        assert_eq!(app.tick, 2);
    }

    #[test]
    fn tick_wraps_around() {
        let mut app = TuiApp::new("http://localhost:3000".into());
        app.tick = u64::MAX;
        app.tick();
        assert_eq!(app.tick, 0);
    }
}
