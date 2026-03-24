//! Shared rendering functions for TUI panels.
//! Used by both the listen TUI and the example TUI.

use std::collections::VecDeque;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::app::{DetailTab, HeaderMap, WebhookEvent};

/// Braille spinner frames for animations.
pub const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// ── Sidebar ─────────────────────────────────────────────────────────

/// Render the sidebar with webhook event history.
pub fn render_sidebar(
    f: &mut Frame,
    title_prefix: &str,
    events: &VecDeque<WebhookEvent>,
    selected: usize,
    tick: u64,
    empty_label: &str,
    area: Rect,
) {
    let title = format!(" {}  {} ", title_prefix, events.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if events.is_empty() {
        let frame = SPINNER[tick as usize % SPINNER.len()];
        let lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(" {frame} {empty_label}"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )]),
        ];
        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    // Each event takes 2 lines: method+id, timestamp
    let inner_height = area.height.saturating_sub(2) as usize;
    let lines_per_event = 2usize;

    // Calculate scroll offset to keep selected item visible
    let selected_start = selected * lines_per_event;
    let selected_end = selected_start + lines_per_event;

    let scroll_offset = if selected_end > inner_height {
        (selected_end - inner_height) as u16
    } else {
        0
    };

    let mut lines: Vec<Line<'_>> = Vec::with_capacity(events.len() * lines_per_event);
    for (i, evt) in events.iter().enumerate() {
        let is_selected = i == selected;
        let pointer = if is_selected { ">" } else { " " };

        let method_style = method_color(&evt.method);
        let bg = if is_selected {
            Color::Rgb(30, 30, 50)
        } else {
            Color::Reset
        };

        // Line 1: pointer + method + short request_id
        let short_id = if evt.request_id.len() > 5 {
            &evt.request_id[..5]
        } else {
            &evt.request_id
        };

        let status_indicator = if evt.error.is_some() {
            Span::styled(" ✗", Style::default().fg(Color::Red))
        } else if evt.status >= 400 {
            Span::styled(" !", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{pointer}{} ", evt.method.to_uppercase()),
                method_style.bg(bg).add_modifier(if is_selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ),
            Span::styled(
                short_id.to_string(),
                Style::default().fg(Color::White).bg(bg),
            ),
            status_indicator,
        ]));

        // Line 2: timestamp
        lines.push(Line::from(vec![Span::styled(
            format!(" {}", evt.timestamp.format("%d %b %H:%M")),
            Style::default().fg(Color::DarkGray).bg(bg),
        )]));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_offset, 0));
    f.render_widget(paragraph, area);
}

// ── Tab bar ─────────────────────────────────────────────────────────

/// Render the Request/Response tab bar.
pub fn render_tab_bar(f: &mut Frame, active_tab: DetailTab, area: Rect) {
    let req_style = if active_tab == DetailTab::Request {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let resp_style = if active_tab == DetailTab::Response {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled("Request", req_style),
        Span::raw("    "),
        Span::styled("Response", resp_style),
    ]);

    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}

// ── Info panels (Details + Headers) ─────────────────────────────────

/// Render the info panels (Details + Headers) side by side.
pub fn render_info_panels(f: &mut Frame, event: &WebhookEvent, active_tab: DetailTab, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_details_block(f, event, chunks[0]);

    let (headers, tab_label) = match active_tab {
        DetailTab::Request => (&event.request_headers, "Request Headers"),
        DetailTab::Response => (&event.response_headers, "Response Headers"),
    };
    render_headers_block(f, headers, tab_label, chunks[1]);
}

/// Render the details block (URL, METHOD, DATE, STATUS, DURATION, ID, ERROR).
pub fn render_details_block(f: &mut Frame, evt: &WebhookEvent, area: Rect) {
    let url = if let Some(ref q) = evt.query {
        format!("{}?{}", evt.path, q)
    } else {
        evt.path.clone()
    };

    let status_str = if evt.status == 0 {
        "ERR".to_string()
    } else {
        format!("{}", evt.status)
    };

    let status_color = if evt.error.is_some() || evt.status >= 500 {
        Color::Red
    } else if evt.status >= 300 {
        Color::Yellow
    } else {
        Color::Green
    };

    let mut lines: Vec<Line<'_>> = vec![
        detail_kv("URL", &url),
        detail_kv("METHOD", &evt.method.to_uppercase()),
        detail_kv("DATE", &evt.timestamp.format("%d %b %H:%M:%S").to_string()),
        Line::from(vec![
            Span::styled(
                format!(" {:<10}", "STATUS"),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(status_str, Style::default().fg(status_color)),
        ]),
        detail_kv("DURATION", &format!("{}ms", evt.duration_ms)),
        detail_kv("ID", &evt.request_id),
    ];

    if let Some(ref err) = evt.error {
        lines.push(Line::from(vec![
            Span::styled(
                " ERROR     ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                truncate_str(err, area.width.saturating_sub(13) as usize),
                Style::default().fg(Color::Red),
            ),
        ]));
    }

    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Render the headers block.
pub fn render_headers_block(f: &mut Frame, headers: &HeaderMap, tab_label: &str, area: Rect) {
    let max_value_width = area.width.saturating_sub(20) as usize;

    let lines: Vec<Line<'_>> = headers
        .iter()
        .map(|(key, value)| {
            Line::from(vec![
                Span::styled(
                    format!(" {:<14}", key.to_uppercase()),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    truncate_str(value, max_value_width),
                    Style::default().fg(Color::White),
                ),
            ])
        })
        .collect();

    let title = format!(" {} ", tab_label);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = if lines.is_empty() {
        Paragraph::new(vec![Line::from(vec![Span::styled(
            " (no headers)",
            Style::default().fg(Color::DarkGray),
        )])])
        .block(block)
    } else {
        Paragraph::new(lines).block(block)
    };

    f.render_widget(paragraph, area);
}

// ── Body panel ──────────────────────────────────────────────────────

/// Render the body panel with scroll support.
pub fn render_body_panel(
    f: &mut Frame,
    event: &WebhookEvent,
    active_tab: DetailTab,
    scroll_offset: u16,
    area: Rect,
) {
    let body = match active_tab {
        DetailTab::Request => &event.request_body,
        DetailTab::Response => &event.response_body,
    };

    let tab_label = match active_tab {
        DetailTab::Request => "Request Body",
        DetailTab::Response => "Response Body",
    };

    let title = format!(" {} ", tab_label);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let lines: Vec<Line<'_>> = match body {
        None => {
            vec![Line::from(vec![Span::styled(
                " (empty)",
                Style::default().fg(Color::DarkGray),
            )])]
        }
        Some(data) => format_body(data),
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_offset, 0));
    f.render_widget(paragraph, area);
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Animated tunnel segment: a ⚡ bounces left-right when connected,
/// a static ✗ is shown when disconnected.
pub fn tunnel_segment(tick: u64, connected: bool) -> Vec<Span<'static>> {
    const WIDTH: usize = 8;

    if !connected {
        let left = "━".repeat(WIDTH / 2);
        let right = "━".repeat(WIDTH / 2);
        return vec![
            Span::styled(left, Style::default().fg(Color::DarkGray)),
            Span::styled(
                " ✗ ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{right} "), Style::default().fg(Color::DarkGray)),
        ];
    }

    // Bouncing ⚡: position oscillates 0..WIDTH-1..0
    let cycle = (WIDTH - 1) * 2;
    let raw_pos = (tick as usize) % cycle;
    let pos = if raw_pos < WIDTH {
        raw_pos
    } else {
        cycle - raw_pos
    };

    let left = "━".repeat(pos);
    let right = "━".repeat(WIDTH - 1 - pos);
    vec![
        Span::styled(left, Style::default().fg(Color::DarkGray)),
        Span::styled(
            "⚡",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{right}▸"), Style::default().fg(Color::DarkGray)),
    ]
}

/// Format body bytes for display: try JSON pretty-print, then UTF-8, then hex dump.
pub fn format_body(data: &[u8]) -> Vec<Line<'static>> {
    // Try JSON pretty-print
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(text) {
            if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
                return pretty
                    .lines()
                    .map(|line| {
                        Line::from(vec![Span::styled(
                            format!(" {line}"),
                            Style::default().fg(Color::White),
                        )])
                    })
                    .collect();
            }
        }

        // Plain UTF-8 text
        return text
            .lines()
            .map(|line| {
                Line::from(vec![Span::styled(
                    format!(" {line}"),
                    Style::default().fg(Color::White),
                )])
            })
            .collect();
    }

    // Hex dump for binary data (first 256 bytes)
    let limit = data.len().min(256);
    let mut lines = Vec::new();
    for chunk in data[..limit].chunks(16) {
        let hex: String = chunk.iter().map(|b| format!("{b:02x} ")).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {:<49}", hex),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(ascii, Style::default().fg(Color::White)),
        ]));
    }
    if data.len() > 256 {
        lines.push(Line::from(vec![Span::styled(
            format!(" ... ({} bytes total)", data.len()),
            Style::default().fg(Color::DarkGray),
        )]));
    }
    lines
}

/// Render a key-value detail line.
pub fn detail_kv<'a>(label: &str, value: &str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!(" {:<10}", label),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

/// Return a style for an HTTP method.
pub fn method_color(method: &str) -> Style {
    match method.to_uppercase().as_str() {
        "GET" => Style::default().fg(Color::Green),
        "POST" => Style::default().fg(Color::Blue),
        "PUT" | "PATCH" => Style::default().fg(Color::Yellow),
        "DELETE" => Style::default().fg(Color::Red),
        "ERR" => Style::default().fg(Color::Red).bg(Color::DarkGray),
        _ => Style::default().fg(Color::White),
    }
}

/// Truncate a string to a maximum length, adding ellipsis if needed.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}
