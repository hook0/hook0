//! Event loop for the example TUI.
//!
//! Manages terminal lifecycle, echo server, WebSocket reconnection,
//! keyboard input, webhook sending, and UI rendering.

use anyhow::{anyhow, Result};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, warn};

use crate::tunnel::{forward_request, generate_token, ClientMessage, ServerMessage, READ_TIMEOUT};

use super::app::{ConnectionStatus, WebhookEvent};
use super::echo_server::start_echo_server;
use super::example_app::{cap_body, sorted_headers, ComposeField, ExampleApp, ViewMode};
use super::example_ui;

/// Estimated viewport height for the JSON editor (conservative minimum).
const EDITOR_VIEWPORT_ESTIMATE: usize = 8;

/// Backoff schedule (same as listen TUI).
const BACKOFF: &[Duration] = &[
    Duration::ZERO,
    Duration::from_millis(100),
    Duration::from_millis(1000),
    Duration::from_millis(5000),
];

/// Action returned by key event handlers.
enum KeyAction {
    None,
    Send,
    Quit,
}

/// Run the example TUI with its own reconnection loop.
pub async fn run_example_tui(
    relay_url: &str,
    initial_token: String,
    target_url: Option<&str>,
    ping_interval: Duration,
) -> Result<()> {
    // Initialize terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Start echo server (unless --target is provided)
    let (echo_url, echo_status_tx, echo_handle) = if let Some(target) = target_url {
        // Use provided target — no echo server
        (target.to_string(), None, None)
    } else {
        let (port, status_tx, handle) = start_echo_server();
        (
            format!("http://127.0.0.1:{}", port),
            Some(status_tx),
            Some(handle),
        )
    };

    let mut app = ExampleApp::new(echo_url.clone());
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client");

    let result = run_example_reconnect_loop(
        &mut terminal,
        &mut app,
        relay_url,
        initial_token,
        &http_client,
        &echo_url,
        ping_interval,
        &echo_status_tx,
    )
    .await;

    // Cleanup
    if let Some(handle) = echo_handle {
        handle.abort();
    }
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

// ── Reconnect loop ──────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn run_example_reconnect_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut ExampleApp,
    relay_url: &str,
    initial_token: String,
    http_client: &reqwest::Client,
    target_url: &str,
    ping_interval: Duration,
    echo_status_tx: &Option<watch::Sender<u16>>,
) -> Result<()> {
    let mut token = initial_token;
    let mut backoff_index: usize = 0;
    let mut last_connected_at: Option<Instant> = None;

    // Initial render
    terminal.draw(|f| example_ui::render(f, app))?;

    loop {
        if app.should_quit {
            return Ok(());
        }

        // Apply backoff delay while keeping TUI alive
        let delay = BACKOFF[backoff_index.min(BACKOFF.len() - 1)];
        if !delay.is_zero() {
            app.status = ConnectionStatus::Reconnecting {
                attempt: app.reconnect_count,
            };
            terminal.draw(|f| example_ui::render(f, app))?;

            if wait_with_example_tui(terminal, app, delay).await? {
                return Ok(()); // user quit
            }
        }

        // Attempt handshake
        app.status = ConnectionStatus::Reconnecting {
            attempt: app.reconnect_count,
        };
        terminal.draw(|f| example_ui::render(f, app))?;

        let handshake_result = handshake_with_example_tui(terminal, app, relay_url, &token).await;

        let (write, read, webhook_url, _view_url) = match handshake_result {
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

        // Reset backoff if last connection was long-lived
        if let Some(last_t) = last_connected_at {
            if last_t.elapsed() > Duration::from_secs(10) {
                backoff_index = 0;
            }
        }
        last_connected_at = Some(Instant::now());

        // Update app state
        app.webhook_url = webhook_url;
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

        // Run session
        let session_result = run_example_session(
            terminal,
            app,
            tx,
            read,
            http_client,
            target_url,
            ping_interval,
            echo_status_tx,
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
                terminal.draw(|f| example_ui::render(f, app))?;
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

// ── Wait with TUI ───────────────────────────────────────────────────

async fn wait_with_example_tui(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut ExampleApp,
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
                    Some(Ok(Event::Key(key))) => {
                        if let KeyAction::Quit = handle_example_key_event(app, key) {
                            return Ok(true);
                        }
                    }
                    Some(Err(_)) | None => return Ok(true),
                    _ => {}
                }
                terminal.draw(|f| example_ui::render(f, app))?;
            }
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| example_ui::render(f, app))?;
            }
            _ = tokio::time::sleep_until(deadline) => {
                return Ok(false);
            }
        }
    }
}

// ── Handshake ───────────────────────────────────────────────────────

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

async fn handshake_with_example_tui(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut ExampleApp,
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
    let mut ui_tick = tokio::time::interval(Duration::from_millis(250));
    ui_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut event_stream = EventStream::new();

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
                    Some(Ok(Event::Key(key))) => {
                        if let KeyAction::Quit = handle_example_key_event(app, key) {
                            return Err(HandshakeOutcome::Quit);
                        }
                    }
                    Some(Err(_)) | None => return Err(HandshakeOutcome::Quit),
                    _ => {}
                }
                terminal.draw(|f| example_ui::render(f, app))?;
            }
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| example_ui::render(f, app))?;
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

    // Wait for Started confirmation
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
                    Some(Ok(Event::Key(key))) => {
                        if let KeyAction::Quit = handle_example_key_event(app, key) {
                            return Err(HandshakeOutcome::Quit);
                        }
                    }
                    Some(Err(_)) | None => return Err(HandshakeOutcome::Quit),
                    _ => {}
                }
                terminal.draw(|f| example_ui::render(f, app))?;
            }
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| example_ui::render(f, app))?;
            }
        }
    }
}

// ── Session ─────────────────────────────────────────────────────────

enum SessionResult {
    Disconnected,
    TokenCollision,
    Quit,
}

#[allow(clippy::too_many_arguments)]
async fn run_example_session(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut ExampleApp,
    tx: mpsc::Sender<String>,
    mut read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    http_client: &reqwest::Client,
    target_url: &str,
    ping_interval: Duration,
    echo_status_tx: &Option<watch::Sender<u16>>,
) -> Result<SessionResult> {
    let mut ping_timer = tokio::time::interval(ping_interval);
    ping_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let mut ui_tick = tokio::time::interval(Duration::from_millis(250));
    ui_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut event_stream = EventStream::new();

    // Initial render
    terminal.draw(|f| example_ui::render(f, app))?;

    loop {
        if app.should_quit {
            return Ok(SessionResult::Quit);
        }

        tokio::select! {
            // Terminal events
            term_event = event_stream.next() => {
                match term_event {
                    Some(Ok(Event::Key(key))) => {
                        match handle_example_key_event(app, key) {
                            KeyAction::Quit => return Ok(SessionResult::Quit),
                            KeyAction::Send => {
                                // Send webhook in background
                                send_example_webhook(http_client, app).await;
                            }
                            KeyAction::None => {}
                        }
                        // Sync echo server status code with compose form
                        if let Some(ref tx) = echo_status_tx {
                            tx.send_replace(app.current_status_code());
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {}
                    Some(Err(_)) => return Ok(SessionResult::Quit),
                    None => return Ok(SessionResult::Quit),
                    _ => {}
                }
                terminal.draw(|f| example_ui::render(f, app))?;
            }
            // WebSocket messages
            ws_msg = async {
                tokio::time::timeout(READ_TIMEOUT, read.next()).await
            } => {
                match ws_msg {
                    Err(_) => {
                        warn!("Read timeout ({}s), reconnecting", READ_TIMEOUT.as_secs());
                        return Ok(SessionResult::Disconnected);
                    }
                    Ok(Some(Ok(Message::Text(text)))) => {
                        let server_msg: ServerMessage = serde_json::from_str(&text)?;
                        match handle_server_message_example(app, &tx, http_client, target_url, server_msg).await {
                            Ok(None) => {}
                            Ok(Some(SessionResult::Disconnected)) => return Ok(SessionResult::Disconnected),
                            Ok(Some(SessionResult::TokenCollision)) => return Ok(SessionResult::TokenCollision),
                            Ok(Some(SessionResult::Quit)) => return Ok(SessionResult::Quit),
                            Err(e) => {
                                warn!("Error handling message: {}", e);
                            }
                        }
                        terminal.draw(|f| example_ui::render(f, app))?;
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
            // Ping timer
            _ = ping_timer.tick() => {
                let ping_msg = ClientMessage::ping();
                let ping_json = serde_json::to_string(&ping_msg)?;
                if tx.send(ping_json).await.is_err() {
                    return Ok(SessionResult::Disconnected);
                }
                debug!("Sent ping");
            }
            // UI tick
            _ = ui_tick.tick() => {
                app.tick();
                terminal.draw(|f| example_ui::render(f, app))?;
            }
        }
    }
}

// ── Key handling ────────────────────────────────────────────────────

fn handle_example_key_event(app: &mut ExampleApp, key: KeyEvent) -> KeyAction {
    // Ctrl+C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return KeyAction::Quit;
    }

    // Ctrl+S always sends (from any field)
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if app.is_connected() && app.view_mode == ViewMode::Compose {
            return KeyAction::Send;
        }
        return KeyAction::None;
    }

    match app.view_mode {
        ViewMode::Compose => handle_compose_key(app, key),
        ViewMode::Inspect => handle_inspect_key(app, key),
    }
}

fn handle_compose_key(app: &mut ExampleApp, key: KeyEvent) -> KeyAction {
    // If JSON editor is focused, handle editor-specific keys first
    if app.focused_field == ComposeField::JsonEditor {
        match key.code {
            KeyCode::Esc => {
                // Exit editor focus
                app.focused_field = ComposeField::EventType;
                return KeyAction::None;
            }
            KeyCode::Tab => {
                app.next_field();
                return KeyAction::None;
            }
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+H = toggle headers from within editor
                app.toggle_headers();
                return KeyAction::None;
            }
            KeyCode::Up => {
                app.current_payload_mut().move_up();
                app.current_payload_mut()
                    .ensure_visible(EDITOR_VIEWPORT_ESTIMATE);
                return KeyAction::None;
            }
            KeyCode::Down => {
                app.current_payload_mut().move_down();
                app.current_payload_mut()
                    .ensure_visible(EDITOR_VIEWPORT_ESTIMATE);
                return KeyAction::None;
            }
            KeyCode::Left => {
                app.current_payload_mut().move_left();
                app.current_payload_mut()
                    .ensure_visible(EDITOR_VIEWPORT_ESTIMATE);
                return KeyAction::None;
            }
            KeyCode::Right => {
                app.current_payload_mut().move_right();
                app.current_payload_mut()
                    .ensure_visible(EDITOR_VIEWPORT_ESTIMATE);
                return KeyAction::None;
            }
            KeyCode::Home => {
                app.current_payload_mut().move_home();
                return KeyAction::None;
            }
            KeyCode::End => {
                app.current_payload_mut().move_end();
                return KeyAction::None;
            }
            KeyCode::Enter => {
                app.current_payload_mut().insert_newline();
                app.current_payload_mut()
                    .ensure_visible(EDITOR_VIEWPORT_ESTIMATE);
                return KeyAction::None;
            }
            KeyCode::Backspace => {
                app.current_payload_mut().backspace();
                app.current_payload_mut()
                    .ensure_visible(EDITOR_VIEWPORT_ESTIMATE);
                return KeyAction::None;
            }
            KeyCode::Delete => {
                app.current_payload_mut().delete();
                return KeyAction::None;
            }
            KeyCode::Char(ch) => {
                app.current_payload_mut().insert_char(ch);
                return KeyAction::None;
            }
            _ => return KeyAction::None,
        }
    }

    // Non-editor compose keys
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            KeyAction::Quit
        }
        KeyCode::Tab => {
            app.next_field();
            KeyAction::None
        }
        KeyCode::Char('h') => {
            app.toggle_headers();
            KeyAction::None
        }
        KeyCode::Enter => {
            if app.is_connected() {
                KeyAction::Send
            } else {
                KeyAction::None
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.focused_field {
                ComposeField::EventType => app.prev_event_type(),
                ComposeField::StatusCode => app.prev_status(),
                _ => {}
            }
            KeyAction::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match app.focused_field {
                ComposeField::EventType => app.next_event_type(),
                ComposeField::StatusCode => app.next_status(),
                _ => {}
            }
            KeyAction::None
        }
        KeyCode::Left => {
            if app.focused_field == ComposeField::StatusCode {
                app.prev_status();
            }
            KeyAction::None
        }
        KeyCode::Right => {
            if app.focused_field == ComposeField::StatusCode {
                app.next_status();
            }
            KeyAction::None
        }
        _ => KeyAction::None,
    }
}

fn handle_inspect_key(app: &mut ExampleApp, key: KeyEvent) -> KeyAction {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            KeyAction::Quit
        }
        // Navigation
        KeyCode::Down => {
            app.select_next();
            KeyAction::None
        }
        KeyCode::Up => {
            app.select_prev();
            KeyAction::None
        }
        KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.select_next();
            KeyAction::None
        }
        KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.select_prev();
            KeyAction::None
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.select_first();
            KeyAction::None
        }
        KeyCode::End | KeyCode::Char('G') => {
            app.select_last();
            KeyAction::None
        }
        // Tab toggle
        KeyCode::Tab => {
            app.toggle_tab();
            KeyAction::None
        }
        // Body scroll
        KeyCode::Char('J') => {
            app.scroll_body_down(1);
            KeyAction::None
        }
        KeyCode::Char('K') => {
            app.scroll_body_up(1);
            KeyAction::None
        }
        KeyCode::PageDown => {
            app.scroll_body_down(10);
            KeyAction::None
        }
        KeyCode::PageUp => {
            app.scroll_body_up(10);
            KeyAction::None
        }
        // New compose
        KeyCode::Char('n') => {
            app.new_compose();
            KeyAction::None
        }
        // Edit + resend
        KeyCode::Char('e') => {
            app.edit_and_resend();
            KeyAction::None
        }
        _ => KeyAction::None,
    }
}

// ── Webhook sending ─────────────────────────────────────────────────

/// Send a webhook from the compose form to the play server's webhook URL.
async fn send_example_webhook(http_client: &reqwest::Client, app: &ExampleApp) {
    let event_type = app.current_event_type().to_string();
    let payload = app.current_payload().text();
    let webhook_url = app.webhook_url.clone();

    if webhook_url.is_empty() {
        return;
    }

    let mut request = http_client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header("X-Hook0-Event-Type", &event_type)
        .header("X-Hook0-Example", "true");

    // Add custom headers
    for (key, value) in &app.custom_headers {
        request = request.header(key, value);
    }

    // Fire and forget — the response comes back via WebSocket
    let _ = request.body(payload).send().await;
}

// ── Server message handling ─────────────────────────────────────────

async fn handle_server_message_example(
    app: &mut ExampleApp,
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

            let request_headers = sorted_headers(&data.headers);

            let request_body_raw =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data.body)
                    .unwrap_or_default();
            let request_body = cap_body(request_body_raw);

            match forward_request(http_client, target_url, &data).await {
                Ok(result) => {
                    let duration_ms = result.duration.as_millis() as u64;
                    let is_error = result.status >= 400;
                    app.record_event(duration_ms, is_error);

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

                    // Auto-switch to inspect mode after receiving response
                    app.enter_inspect_mode();

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

                    app.enter_inspect_mode();

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
