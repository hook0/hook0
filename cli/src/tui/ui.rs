//! TUI rendering — sidebar + tabbed detail panel layout
//!
//! Layout:
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │  HOOK0 LISTEN                  Listening on https://play.hook0.com/in/… │
//! │  ● Connected                   Target: http://localhost:3000             │
//! ├──────────────┬───────────────────────────────────────────────────────────┤
//! │ Messages  3  │  [Request]  Response                                      │
//! │              │                                                           │
//! │ >POST abc12 │  ┌─ Details ─────────────┐  ┌─ Request Headers ─────────┐ │
//! │  29 Jan 22:40│  │ URL     /in/c_xxx/    │  │ ACCEPT       */*          │ │
//! │              │  │ METHOD  POST          │  │ USER-AGENT   curl/8.15.0  │ │
//! │  GET  def34 │  │ DATE    29 Jan 22:40  │  │ CONTENT-TYPE app/json     │ │
//! │  29 Jan 22:39│  │ STATUS  200           │  │                           │ │
//! │              │  │ DURATION 45ms         │  │                           │ │
//! │  PUT  ghi56 │  └───────────────────────┘  └───────────────────────────┘ │
//! │  29 Jan 22:38│                                                           │
//! │              │  ┌─ Body ──────────────────────────────────────────────┐  │
//! │              │  │ {                                                    │  │
//! │              │  │   "test": "data"                                    │  │
//! │              │  │ }                                                    │  │
//! │              │  └────────────────────────────────────────────────────┘  │
//! ├──────────────┴───────────────────────────────────────────────────────────┤
//! │  ↑↓ Navigate  Tab Switch  J/K Scroll body  q Quit               1/3    │
//! └──────────────────────────────────────────────────────────────────────────┘

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::app::{ConnectionStatus, TuiApp};
use super::shared;

pub fn render(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header: branding + status + URLs
            Constraint::Min(10),   // Main: sidebar + detail
            Constraint::Length(1), // Footer: shortcuts + position
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_main_content(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let spinner_frame = shared::SPINNER[app.tick as usize % shared::SPINNER.len()];

    let status_spans: Vec<Span<'_>> = match &app.status {
        ConnectionStatus::Connected => {
            let dots = ["●", "●", "◉", "●"];
            let dot = dots[(app.tick / 2) as usize % dots.len()];
            vec![
                Span::styled(dot, Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled(
                    "Connected",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]
        }
        ConnectionStatus::Reconnecting { attempt } => {
            let label = if *attempt == 0 {
                "Connecting...".to_string()
            } else {
                format!("Connection lost, reconnecting... (attempt {})", attempt)
            };
            vec![
                Span::styled(
                    spinner_frame,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    label,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]
        }
    };

    let uptime = app.uptime_str();
    let stats = if app.event_count > 0 {
        let avg_ms = format!("{:.0}ms", app.avg_duration_ms);
        format!(
            "  {}ok {}err {}avg  up {}",
            app.success_count, app.error_count, avg_ms, uptime
        )
    } else {
        format!("  up {uptime}")
    };

    let mut first_line_spans = vec![
        Span::styled(
            " HOOK0 LISTEN ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];
    first_line_spans.extend(status_spans);
    first_line_spans.push(Span::styled(stats, Style::default().fg(Color::DarkGray)));

    // Flow diagram: 🌐 webhook_url ━━⚡━━▸ 🖥 target_url
    let mut flow_spans: Vec<Span<'_>> = Vec::new();
    flow_spans.push(Span::styled(" 🌐 ", Style::default().fg(Color::Cyan)));
    let webhook_display = if app.webhook_url.is_empty() {
        "...".to_string()
    } else {
        app.webhook_url.clone()
    };
    flow_spans.push(Span::styled(
        webhook_display,
        Style::default().fg(Color::White),
    ));
    flow_spans.push(Span::raw(" "));
    flow_spans.extend(shared::tunnel_segment(app.tick, app.is_connected()));
    flow_spans.push(Span::raw(" "));
    flow_spans.push(Span::styled("🖥  ", Style::default().fg(Color::Yellow)));
    flow_spans.push(Span::styled(
        &app.target_url,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));

    let lines = vec![Line::from(first_line_spans), Line::from(flow_spans)];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_main_content(f: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar
            Constraint::Min(40),    // Detail panel
        ])
        .split(area);

    shared::render_sidebar(
        f,
        "Messages",
        &app.events,
        app.selected,
        app.tick,
        "Waiting...",
        chunks[0],
    );
    render_detail_panel(f, app, chunks[1]);
}

fn render_detail_panel(f: &mut Frame, app: &TuiApp, area: Rect) {
    if app.events.is_empty() {
        let frame = shared::SPINNER[app.tick as usize % shared::SPINNER.len()];
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {frame}  Waiting for webhooks..."),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Send a request to your webhook URL to see it here.",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                format!(
                    "  curl -H 'Content-Type: application/json' -d '{{\"event\":\"test\"}}' {}",
                    app.webhook_url
                ),
                Style::default().fg(Color::DarkGray),
            )]),
        ];
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(9), // Info panels (Details + Headers)
            Constraint::Min(4),    // Body
        ])
        .split(area);

    shared::render_tab_bar(f, app.active_tab, chunks[0]);

    if let Some(evt) = app.events.get(app.selected) {
        shared::render_info_panels(f, evt, app.active_tab, chunks[1]);
        shared::render_body_panel(f, evt, app.active_tab, app.body_scroll_offset, chunks[2]);
    }
}

fn render_footer(f: &mut Frame, app: &TuiApp, area: Rect) {
    let mut spans: Vec<Span<'_>> = vec![
        Span::styled(
            " ↑↓ ",
            Style::default().fg(Color::Black).bg(Color::DarkGray),
        ),
        Span::raw(" Navigate  "),
        Span::styled(
            " Tab ",
            Style::default().fg(Color::Black).bg(Color::DarkGray),
        ),
        Span::raw(" Switch  "),
        Span::styled(
            " J/K ",
            Style::default().fg(Color::Black).bg(Color::DarkGray),
        ),
        Span::raw(" Scroll body  "),
        Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::raw(" Quit"),
    ];

    if !app.events.is_empty() {
        // Right-align position counter
        let position = format!("  {}/{}  ", app.selected + 1, app.events.len());
        // Calculate padding
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (area.width as usize).saturating_sub(used + position.len());
        spans.push(Span::raw(" ".repeat(remaining)));
        spans.push(Span::styled(position, Style::default().fg(Color::DarkGray)));
    }

    let paragraph = Paragraph::new(Line::from(spans));
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::shared::{format_body, method_color, truncate_str, tunnel_segment};
    use ratatui::style::Color;

    // ── tunnel_segment ──────────────────────────────────────────

    #[test]
    fn tunnel_segment_disconnected_shows_static_cross() {
        let spans = tunnel_segment(0, false);
        let text: String = spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains('✗'), "disconnected segment should show ✗");
        // Should NOT contain the bolt
        assert!(
            !text.contains('⚡'),
            "disconnected segment should not show ⚡"
        );
    }

    #[test]
    fn tunnel_segment_disconnected_is_stable_across_ticks() {
        let text_0: String = tunnel_segment(0, false)
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        let text_99: String = tunnel_segment(99, false)
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(text_0, text_99, "disconnected segment should not animate");
    }

    #[test]
    fn tunnel_segment_connected_shows_bolt() {
        let spans = tunnel_segment(0, true);
        let text: String = spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains('⚡'), "connected segment should show ⚡");
        assert!(
            text.contains('▸'),
            "connected segment should end with arrow"
        );
    }

    #[test]
    fn tunnel_segment_connected_animates_across_ticks() {
        // Collect span texts for several ticks — the bolt position should change
        let texts: Vec<String> = (0..14)
            .map(|tick| {
                tunnel_segment(tick, true)
                    .iter()
                    .map(|s| s.content.to_string())
                    .collect()
            })
            .collect();

        // Not all frames should be identical (the bolt bounces)
        let unique_count = texts.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(
            unique_count > 1,
            "connected tunnel should animate (got {} unique frames out of 14)",
            unique_count
        );
    }

    #[test]
    fn tunnel_segment_connected_bolt_stays_within_bounds() {
        // WIDTH=8, cycle=(8-1)*2=14 — test a full cycle + extra
        for tick in 0..30 {
            let spans = tunnel_segment(tick, true);
            // The total character width should be consistent
            let text: String = spans.iter().map(|s| s.content.to_string()).collect();
            assert!(text.contains('⚡'), "tick {} should contain ⚡", tick);
            assert!(text.contains('▸'), "tick {} should end with ▸", tick);
        }
    }

    // ── format_body ─────────────────────────────────────────────

    #[test]
    fn format_body_json_pretty_prints() {
        let data = br#"{"name":"hook0","version":1}"#;
        let lines = format_body(data);
        // Pretty-printed JSON should produce multiple lines
        assert!(
            lines.len() > 1,
            "JSON should be pretty-printed into multiple lines"
        );
        // First line should contain opening brace
        let first: String = lines[0]
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert!(
            first.contains('{'),
            "first line should contain opening brace"
        );
    }

    #[test]
    fn format_body_plain_text() {
        let data = b"Hello, world!\nSecond line";
        let lines = format_body(data);
        assert_eq!(lines.len(), 2);
        let first: String = lines[0]
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert!(first.contains("Hello, world!"));
    }

    #[test]
    fn format_body_binary_hex_dump() {
        // Invalid UTF-8 bytes: 0x80..0xA0 are continuation bytes without start bytes
        let data: Vec<u8> = (0x80..0xA0).collect();
        let lines = format_body(&data);
        // Should produce hex dump lines
        assert!(!lines.is_empty());
        let first: String = lines[0]
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        // Hex dump should contain hex values like "80 81 82"
        assert!(
            first.contains("80"),
            "hex dump should contain hex values, got: {}",
            first
        );
    }

    #[test]
    fn format_body_binary_large_shows_truncation_notice() {
        // Use invalid UTF-8 to force hex dump path (0x80..0xFF repeated)
        let data: Vec<u8> = (0..512).map(|i| 0x80 + (i % 64) as u8).collect();
        let lines = format_body(&data);
        let last: String = lines
            .last()
            .unwrap()
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert!(
            last.contains("512 bytes total"),
            "large binary body should show truncation notice, got: {}",
            last
        );
    }

    #[test]
    fn format_body_empty_json_object() {
        let data = b"{}";
        let lines = format_body(data);
        assert!(!lines.is_empty());
    }

    #[test]
    fn format_body_json_array() {
        let data = br#"[1,2,3]"#;
        let lines = format_body(data);
        // Should pretty-print the array across multiple lines
        assert!(lines.len() > 1);
    }

    // ── truncate_str ────────────────────────────────────────────

    #[test]
    fn truncate_str_short_string_unchanged() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn truncate_str_exact_length_unchanged() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn truncate_str_over_length_adds_ellipsis() {
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }

    #[test]
    fn truncate_str_very_small_max_no_ellipsis() {
        // max_len <= 3 means no room for ellipsis, just truncate
        assert_eq!(truncate_str("hello", 3), "hel");
        assert_eq!(truncate_str("hello", 2), "he");
        assert_eq!(truncate_str("hello", 1), "h");
    }

    #[test]
    fn truncate_str_max_4_adds_ellipsis() {
        // max_len=4 > 3, so ellipsis applies: 1 char + "..."
        assert_eq!(truncate_str("hello", 4), "h...");
    }

    #[test]
    fn truncate_str_empty_string() {
        assert_eq!(truncate_str("", 10), "");
        assert_eq!(truncate_str("", 0), "");
    }

    // ── method_color ────────────────────────────────────────────

    #[test]
    fn method_color_known_methods() {
        // GET -> Green
        let get_style = method_color("GET");
        assert_eq!(get_style.fg, Some(Color::Green));

        // POST -> Blue
        let post_style = method_color("POST");
        assert_eq!(post_style.fg, Some(Color::Blue));

        // PUT -> Yellow
        let put_style = method_color("PUT");
        assert_eq!(put_style.fg, Some(Color::Yellow));

        // PATCH -> Yellow
        let patch_style = method_color("PATCH");
        assert_eq!(patch_style.fg, Some(Color::Yellow));

        // DELETE -> Red
        let delete_style = method_color("DELETE");
        assert_eq!(delete_style.fg, Some(Color::Red));
    }

    #[test]
    fn method_color_err_has_background() {
        let err_style = method_color("ERR");
        assert_eq!(err_style.fg, Some(Color::Red));
        assert_eq!(err_style.bg, Some(Color::DarkGray));
    }

    #[test]
    fn method_color_unknown_defaults_to_white() {
        let style = method_color("OPTIONS");
        assert_eq!(style.fg, Some(Color::White));
    }

    #[test]
    fn method_color_case_insensitive() {
        // Should match regardless of case
        let get_lower = method_color("get");
        assert_eq!(get_lower.fg, Some(Color::Green));

        let post_mixed = method_color("Post");
        assert_eq!(post_mixed.fg, Some(Color::Blue));
    }
}
