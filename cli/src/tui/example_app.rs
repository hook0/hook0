//! Application state for the example TUI.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use super::app::{ConnectionStatus, DetailTab, HeaderMap, WebhookEvent};
use super::json_editor::JsonEditorState;

/// Event types available in the compose form.
pub const EVENT_TYPES: &[&str] = &[
    "user.account.created",
    "order.completed",
    "payment.received",
];

/// Status codes available in the compose form.
pub const STATUS_OPTIONS: &[(u16, &str)] = &[
    (200, "OK"),
    (201, "Created"),
    (204, "No Content"),
    (400, "Bad Request"),
    (401, "Unauthorized"),
    (403, "Forbidden"),
    (404, "Not Found"),
    (500, "Internal Server Error"),
    (502, "Bad Gateway"),
    (503, "Service Unavailable"),
];

/// Maximum number of events to keep in history.
const MAX_EVENTS: usize = 500;

/// Maximum body size to store (64 KB).
const MAX_BODY_SIZE: usize = 64 * 1024;

/// Current view mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Compose,
    Inspect,
}

/// Which field is focused in compose mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposeField {
    EventType,
    StatusCode,
    JsonEditor,
    CustomHeaders,
}

/// Application state for the example TUI.
pub struct ExampleApp {
    // ── Connection ──────────────────────────────────────────────
    pub webhook_url: String,
    pub echo_server_url: String,
    pub status: ConnectionStatus,
    pub should_quit: bool,
    pub tick: u64,
    pub started_at: Instant,
    pub reconnect_count: u32,

    // ── View mode ───────────────────────────────────────────────
    pub view_mode: ViewMode,

    // ── History (shared with listen) ────────────────────────────
    pub events: VecDeque<WebhookEvent>,
    pub selected: usize,
    pub active_tab: DetailTab,
    pub body_scroll_offset: u16,
    pub event_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub avg_duration_ms: f64,

    // ── Compose form ────────────────────────────────────────────
    pub focused_field: ComposeField,
    pub selected_event_type: usize,
    pub selected_status: usize,
    pub headers_visible: bool,
    /// Per-event-type payload editors.
    pub payloads: HashMap<String, JsonEditorState>,
    pub custom_headers: Vec<(String, String)>,
}

impl ExampleApp {
    pub fn new(echo_server_url: String) -> Self {
        let mut payloads = HashMap::new();
        // Seed default payloads for each event type
        for event_type in EVENT_TYPES {
            let payload = sample_payload(event_type);
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            payloads.insert(event_type.to_string(), JsonEditorState::from_text(&pretty));
        }

        Self {
            webhook_url: String::new(),
            echo_server_url,
            status: ConnectionStatus::Reconnecting { attempt: 0 },
            should_quit: false,
            tick: 0,
            started_at: Instant::now(),
            reconnect_count: 0,

            view_mode: ViewMode::Compose,

            events: VecDeque::new(),
            selected: 0,
            active_tab: DetailTab::Request,
            body_scroll_offset: 0,
            event_count: 0,
            success_count: 0,
            error_count: 0,
            avg_duration_ms: 0.0,

            focused_field: ComposeField::EventType,
            selected_event_type: 0,
            selected_status: 0,
            headers_visible: false,
            payloads,
            custom_headers: Vec::new(),
        }
    }

    // ── Connection ──────────────────────────────────────────────

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

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    // ── Stats ───────────────────────────────────────────────────

    pub fn record_event(&mut self, duration_ms: u64, is_error: bool) {
        self.event_count += 1;
        if is_error {
            self.error_count += 1;
        } else {
            self.success_count += 1;
        }
        let n = self.event_count as f64;
        self.avg_duration_ms = self.avg_duration_ms * ((n - 1.0) / n) + (duration_ms as f64 / n);
    }

    // ── Event history ───────────────────────────────────────────

    pub fn push_event(&mut self, event: WebhookEvent) {
        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
            self.selected = self.selected.saturating_sub(1);
        }
        self.events.push_back(event);
        self.selected = self.events.len().saturating_sub(1);
        self.body_scroll_offset = 0;
    }

    // ── View mode transitions ───────────────────────────────────

    pub fn enter_inspect_mode(&mut self) {
        self.view_mode = ViewMode::Inspect;
        self.body_scroll_offset = 0;
    }

    pub fn new_compose(&mut self) {
        self.view_mode = ViewMode::Compose;
        self.focused_field = ComposeField::EventType;
        self.body_scroll_offset = 0;
    }

    pub fn edit_and_resend(&mut self) {
        if let Some(evt) = self.events.get(self.selected) {
            // Pre-fill the compose form from the selected event
            // Try to find the event type from the request headers
            let event_type = evt
                .request_headers
                .iter()
                .find(|(k, _)| k.to_lowercase() == "x-hook0-event-type")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| EVENT_TYPES[0].to_string());

            // Set the event type selector
            if let Some(idx) = EVENT_TYPES.iter().position(|&t| t == event_type) {
                self.selected_event_type = idx;
            }

            // Pre-fill the payload from the request body
            if let Some(ref body) = evt.request_body {
                if let Ok(text) = std::str::from_utf8(body) {
                    // Try to pretty-print the JSON
                    let pretty = serde_json::from_str::<serde_json::Value>(text)
                        .ok()
                        .and_then(|v| serde_json::to_string_pretty(&v).ok())
                        .unwrap_or_else(|| text.to_string());
                    self.payloads
                        .insert(event_type.clone(), JsonEditorState::from_text(&pretty));
                }
            }
        }
        self.view_mode = ViewMode::Compose;
        self.focused_field = ComposeField::JsonEditor;
    }

    // ── Navigation (inspect mode) ───────────────────────────────

    pub fn select_next(&mut self) {
        if !self.events.is_empty() && self.selected < self.events.len() - 1 {
            self.selected += 1;
            self.body_scroll_offset = 0;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.body_scroll_offset = 0;
        }
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
        self.body_scroll_offset = 0;
    }

    pub fn select_last(&mut self) {
        if !self.events.is_empty() {
            self.selected = self.events.len() - 1;
            self.body_scroll_offset = 0;
        }
    }

    pub fn toggle_tab(&mut self) {
        self.active_tab = match self.active_tab {
            DetailTab::Request => DetailTab::Response,
            DetailTab::Response => DetailTab::Request,
        };
        self.body_scroll_offset = 0;
    }

    pub fn scroll_body_down(&mut self, amount: u16) {
        self.body_scroll_offset = self.body_scroll_offset.saturating_add(amount);
    }

    pub fn scroll_body_up(&mut self, amount: u16) {
        self.body_scroll_offset = self.body_scroll_offset.saturating_sub(amount);
    }

    // ── Compose form helpers ────────────────────────────────────

    /// Get the currently selected event type string.
    pub fn current_event_type(&self) -> &str {
        EVENT_TYPES[self.selected_event_type]
    }

    /// Get the current status code.
    pub fn current_status_code(&self) -> u16 {
        STATUS_OPTIONS[self.selected_status].0
    }

    /// Get the current status label (e.g. "200 OK").
    pub fn current_status_label(&self) -> String {
        let (code, label) = STATUS_OPTIONS[self.selected_status];
        format!("{} {}", code, label)
    }

    /// Get the current JSON editor state for the selected event type.
    pub fn current_payload(&self) -> &JsonEditorState {
        let event_type = self.current_event_type();
        self.payloads
            .get(event_type)
            .expect("payload must exist for event type")
    }

    /// Get a mutable reference to the current JSON editor state.
    pub fn current_payload_mut(&mut self) -> &mut JsonEditorState {
        let event_type = self.current_event_type().to_string();
        self.payloads
            .entry(event_type)
            .or_insert_with(|| JsonEditorState::from_text("{}"))
    }

    /// Cycle to the next event type.
    pub fn next_event_type(&mut self) {
        self.selected_event_type = (self.selected_event_type + 1) % EVENT_TYPES.len();
    }

    /// Cycle to the previous event type.
    pub fn prev_event_type(&mut self) {
        if self.selected_event_type == 0 {
            self.selected_event_type = EVENT_TYPES.len() - 1;
        } else {
            self.selected_event_type -= 1;
        }
    }

    /// Cycle to the next status code.
    pub fn next_status(&mut self) {
        self.selected_status = (self.selected_status + 1) % STATUS_OPTIONS.len();
    }

    /// Cycle to the previous status code.
    pub fn prev_status(&mut self) {
        if self.selected_status == 0 {
            self.selected_status = STATUS_OPTIONS.len() - 1;
        } else {
            self.selected_status -= 1;
        }
    }

    /// Cycle focus to the next compose field.
    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            ComposeField::EventType => ComposeField::StatusCode,
            ComposeField::StatusCode => ComposeField::JsonEditor,
            ComposeField::JsonEditor => {
                if self.headers_visible {
                    ComposeField::CustomHeaders
                } else {
                    ComposeField::EventType
                }
            }
            ComposeField::CustomHeaders => ComposeField::EventType,
        };
    }

    /// Toggle custom headers visibility.
    pub fn toggle_headers(&mut self) {
        self.headers_visible = !self.headers_visible;
        if !self.headers_visible && self.focused_field == ComposeField::CustomHeaders {
            self.focused_field = ComposeField::EventType;
        }
    }
}

/// Cap a body to MAX_BODY_SIZE.
pub fn cap_body(body: Vec<u8>) -> Option<Vec<u8>> {
    if body.is_empty() {
        return None;
    }
    if body.len() > MAX_BODY_SIZE {
        Some(body[..MAX_BODY_SIZE].to_vec())
    } else {
        Some(body)
    }
}

/// Convert a HashMap to a sorted HeaderMap.
pub fn sorted_headers(map: &std::collections::HashMap<String, String>) -> HeaderMap {
    let mut headers: HeaderMap = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    headers.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    headers
}

/// Generate a sample JSON payload for a given event type.
fn sample_payload(event_type: &str) -> serde_json::Value {
    match event_type {
        "user.account.created" => serde_json::json!({
            "user": {
                "id": "usr_abc123",
                "email": "jane.doe@example.com",
                "name": "Jane Doe",
                "created_at": "2025-01-15T10:30:00Z"
            }
        }),
        "order.completed" => serde_json::json!({
            "order": {
                "id": "ord_789xyz",
                "amount": 49.99,
                "currency": "USD",
                "items": [
                    {"name": "Widget Pro", "quantity": 2, "price": 24.99}
                ],
                "completed_at": "2025-01-15T14:22:00Z"
            }
        }),
        "payment.received" => serde_json::json!({
            "payment": {
                "id": "pay_def456",
                "amount": 99.00,
                "currency": "EUR",
                "method": "card",
                "status": "succeeded",
                "received_at": "2025-01-15T09:15:00Z"
            }
        }),
        _ => serde_json::json!({
            "event_type": event_type,
            "data": {
                "id": "evt_sample_001",
                "message": "This is a sample webhook event",
                "timestamp": "2025-01-15T12:00:00Z"
            }
        }),
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
            request_headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                (
                    "x-hook0-event-type".to_string(),
                    "user.account.created".to_string(),
                ),
            ],
            request_body: Some(br#"{"user":{"id":"usr_abc123"}}"#.to_vec()),
            response_headers: vec![],
            response_body: None,
        }
    }

    #[test]
    fn new_app_starts_in_compose_mode() {
        let app = ExampleApp::new("http://localhost:9876".into());
        assert_eq!(app.view_mode, ViewMode::Compose);
        assert_eq!(app.focused_field, ComposeField::EventType);
        assert!(!app.is_connected());
        assert_eq!(app.events.len(), 0);
    }

    #[test]
    fn new_app_has_payloads_for_all_event_types() {
        let app = ExampleApp::new("http://localhost:9876".into());
        for event_type in EVENT_TYPES {
            assert!(
                app.payloads.contains_key(*event_type),
                "missing payload for {}",
                event_type
            );
        }
    }

    #[test]
    fn push_event_and_enter_inspect() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        app.push_event(make_event("POST", 200, "aaa"));
        app.enter_inspect_mode();
        assert_eq!(app.view_mode, ViewMode::Inspect);
        assert_eq!(app.events.len(), 1);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn push_event_caps_at_max() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        for i in 0..MAX_EVENTS + 50 {
            app.push_event(make_event("POST", 200, &format!("id_{i}")));
        }
        assert_eq!(app.events.len(), MAX_EVENTS);
        assert_eq!(app.selected, MAX_EVENTS - 1);
    }

    #[test]
    fn event_type_cycling() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        assert_eq!(app.current_event_type(), "user.account.created");
        app.next_event_type();
        assert_eq!(app.current_event_type(), "order.completed");
        app.next_event_type();
        assert_eq!(app.current_event_type(), "payment.received");
        app.next_event_type();
        assert_eq!(app.current_event_type(), "user.account.created"); // wraps
        app.prev_event_type();
        assert_eq!(app.current_event_type(), "payment.received"); // wraps back
    }

    #[test]
    fn event_type_switching_preserves_edits() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        // Edit the first event type payload
        app.current_payload_mut().lines = vec!["edited".to_string()];
        app.current_payload_mut().dirty = true;

        // Switch to another type
        app.next_event_type();
        assert_ne!(app.current_payload().lines[0], "edited");

        // Switch back
        app.prev_event_type();
        assert_eq!(app.current_payload().lines[0], "edited");
        assert!(app.current_payload().dirty);
    }

    #[test]
    fn status_cycling() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        assert_eq!(app.current_status_code(), 200);
        app.next_status();
        assert_eq!(app.current_status_code(), 201);
        app.prev_status();
        assert_eq!(app.current_status_code(), 200);
        app.prev_status(); // wraps
        assert_eq!(
            app.current_status_code(),
            STATUS_OPTIONS[STATUS_OPTIONS.len() - 1].0
        );
    }

    #[test]
    fn field_cycling() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        assert_eq!(app.focused_field, ComposeField::EventType);
        app.next_field();
        assert_eq!(app.focused_field, ComposeField::StatusCode);
        app.next_field();
        assert_eq!(app.focused_field, ComposeField::JsonEditor);
        app.next_field(); // headers not visible → wraps to EventType
        assert_eq!(app.focused_field, ComposeField::EventType);
    }

    #[test]
    fn field_cycling_with_headers() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        app.headers_visible = true;
        app.focused_field = ComposeField::JsonEditor;
        app.next_field();
        assert_eq!(app.focused_field, ComposeField::CustomHeaders);
        app.next_field();
        assert_eq!(app.focused_field, ComposeField::EventType);
    }

    #[test]
    fn toggle_headers_resets_focus_if_hidden() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        app.headers_visible = true;
        app.focused_field = ComposeField::CustomHeaders;
        app.toggle_headers();
        assert!(!app.headers_visible);
        assert_eq!(app.focused_field, ComposeField::EventType);
    }

    #[test]
    fn new_compose_resets_state() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        app.view_mode = ViewMode::Inspect;
        app.focused_field = ComposeField::JsonEditor;
        app.body_scroll_offset = 10;
        app.new_compose();
        assert_eq!(app.view_mode, ViewMode::Compose);
        assert_eq!(app.focused_field, ComposeField::EventType);
        assert_eq!(app.body_scroll_offset, 0);
    }

    #[test]
    fn edit_and_resend_prefills_from_event() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        app.push_event(make_event("POST", 200, "aaa"));
        app.enter_inspect_mode();
        app.edit_and_resend();
        assert_eq!(app.view_mode, ViewMode::Compose);
        assert_eq!(app.focused_field, ComposeField::JsonEditor);
        assert_eq!(app.selected_event_type, 0); // user.account.created
    }

    #[test]
    fn navigation_inspect_mode() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        app.push_event(make_event("POST", 200, "a"));
        app.push_event(make_event("GET", 200, "b"));
        app.push_event(make_event("PUT", 200, "c"));
        assert_eq!(app.selected, 2);

        app.select_prev();
        assert_eq!(app.selected, 1);
        app.select_first();
        assert_eq!(app.selected, 0);
        app.select_last();
        assert_eq!(app.selected, 2);
        app.select_next(); // clamped
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn record_event_stats() {
        let mut app = ExampleApp::new("http://localhost:9876".into());
        app.record_event(100, false);
        app.record_event(200, true);
        assert_eq!(app.event_count, 2);
        assert_eq!(app.success_count, 1);
        assert_eq!(app.error_count, 1);
        assert!((app.avg_duration_ms - 150.0).abs() < 0.01);
    }

    #[test]
    fn status_label_format() {
        let app = ExampleApp::new("http://localhost:9876".into());
        assert_eq!(app.current_status_label(), "200 OK");
    }

    #[test]
    fn cap_body_empty() {
        assert!(cap_body(vec![]).is_none());
    }

    #[test]
    fn cap_body_within_limit() {
        let data = vec![1u8; 100];
        let result = cap_body(data.clone());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn cap_body_over_limit() {
        let data = vec![42u8; MAX_BODY_SIZE + 1000];
        let result = cap_body(data).unwrap();
        assert_eq!(result.len(), MAX_BODY_SIZE);
    }

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
}
