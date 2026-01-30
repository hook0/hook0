//! Rendering for the example TUI — compose panel + inspect panel.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::app::ConnectionStatus;
use super::example_app::{ComposeField, ExampleApp, ViewMode, EVENT_TYPES};
use super::json_editor::highlight_json_line;
use super::shared;

/// Render the full example TUI.
pub fn render(f: &mut Frame, app: &ExampleApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header
            Constraint::Min(10),   // Main: sidebar + panel
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_main_content(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

// ── Header ──────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &ExampleApp, area: Rect) {
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
                Span::styled(
                    format!("    Echo: {}  up {}", app.echo_server_url, app.uptime_str()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]
        }
        ConnectionStatus::Reconnecting { attempt } => {
            let label = if *attempt == 0 {
                "Connecting...".to_string()
            } else {
                format!("Reconnecting... (attempt {})", attempt)
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

    let mut first_line_spans = vec![
        Span::styled(
            " HOOK0 EXAMPLE ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];
    first_line_spans.extend(status_spans);

    // Flow diagram
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
        &app.echo_server_url,
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

// ── Main content ────────────────────────────────────────────────────

fn render_main_content(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar
            Constraint::Min(40),    // Panel
        ])
        .split(area);

    shared::render_sidebar(
        f,
        "History",
        &app.events,
        app.selected,
        app.tick,
        "(empty)",
        chunks[0],
    );

    match app.view_mode {
        ViewMode::Compose => render_compose_panel(f, app, chunks[1]),
        ViewMode::Inspect => render_inspect_panel(f, app, chunks[1]),
    }
}

// ── Compose panel ───────────────────────────────────────────────────

fn render_compose_panel(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let block = Block::default()
        .title(" Compose ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut constraints = vec![
        Constraint::Length(1), // Event Type label + Status label
        Constraint::Length(3), // Event type list + Status picker
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // "Payload" label
        Constraint::Min(4),    // JSON editor
    ];
    if app.headers_visible {
        constraints.push(Constraint::Length(5)); // Custom headers
    }
    constraints.push(Constraint::Length(1)); // Send button

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut chunk_idx = 0;

    // Row 0: Labels
    render_field_labels(f, app, chunks[chunk_idx]);
    chunk_idx += 1;

    // Row 1: Event type list + Status picker
    render_event_type_and_status(f, app, chunks[chunk_idx]);
    chunk_idx += 1;

    // Row 2: Spacer
    chunk_idx += 1;

    // Row 3: Payload label
    let payload_label = Line::from(vec![Span::styled(
        " Payload",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(Paragraph::new(payload_label), chunks[chunk_idx]);
    chunk_idx += 1;

    // Row 4: JSON editor
    render_json_editor(f, app, chunks[chunk_idx]);
    chunk_idx += 1;

    // Optional: Custom headers
    if app.headers_visible {
        render_custom_headers(f, app, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    // Last row: Send button
    render_send_button(f, app, chunks[chunk_idx]);
}

fn render_field_labels(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let et_focused = app.focused_field == ComposeField::EventType;
    let et_style = if et_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(" Event Type", et_style)])),
        chunks[0],
    );

    let sc_focused = app.focused_field == ComposeField::StatusCode;
    let sc_style = if sc_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(" Response Status", sc_style)])),
        chunks[1],
    );
}

fn render_event_type_and_status(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Event type list
    let et_focused = app.focused_field == ComposeField::EventType;
    let mut et_lines: Vec<Line<'_>> = Vec::new();
    for (i, &event_type) in EVENT_TYPES.iter().enumerate() {
        let is_selected = i == app.selected_event_type;
        let pointer = if is_selected { " ▸ " } else { "   " };
        let style = if is_selected && et_focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        et_lines.push(Line::from(vec![Span::styled(
            format!("{pointer}{event_type}"),
            style,
        )]));
    }
    f.render_widget(Paragraph::new(et_lines), chunks[0]);

    // Status code picker
    let sc_focused = app.focused_field == ComposeField::StatusCode;
    let label = app.current_status_label();
    let style = if sc_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let border_style = if sc_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);
    let status_lines = vec![Line::from(vec![
        Span::styled(" < ", Style::default().fg(Color::DarkGray)),
        Span::styled(label, style),
        Span::styled(" > ", Style::default().fg(Color::DarkGray)),
    ])];
    f.render_widget(Paragraph::new(status_lines).block(block), chunks[1]);
}

fn render_json_editor(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let is_focused = app.focused_field == ComposeField::JsonEditor;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let viewport_height = inner.height as usize;
    // Note: ensure_visible cannot be called here because app is immutable.
    // It is called in the key handler after cursor movement.
    let editor = app.current_payload();

    let mut lines: Vec<Line<'_>> = Vec::new();
    for (_line_idx, line_text) in editor.visible_lines(viewport_height) {
        let mut spans = vec![Span::raw(" ")]; // left padding
        spans.extend(highlight_json_line(line_text));
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), inner);

    // Place cursor if focused
    if is_focused {
        let cursor_y = editor.cursor_row.saturating_sub(editor.scroll_offset) as u16;
        if (cursor_y as usize) < viewport_height {
            // +1 for left padding " "
            let cursor_x = editor.cursor_col as u16 + 1;
            f.set_cursor_position((inner.x + cursor_x, inner.y + cursor_y));
        }
    }
}

fn render_custom_headers(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let is_focused = app.focused_field == ComposeField::CustomHeaders;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(" Custom Headers ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let lines: Vec<Line<'_>> = if app.custom_headers.is_empty() {
        vec![Line::from(vec![Span::styled(
            " (none — press h to toggle)",
            Style::default().fg(Color::DarkGray),
        )])]
    } else {
        app.custom_headers
            .iter()
            .map(|(k, v)| {
                Line::from(vec![
                    Span::styled(format!(" {}: ", k), Style::default().fg(Color::Cyan)),
                    Span::styled(v.clone(), Style::default().fg(Color::White)),
                ])
            })
            .collect()
    };

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_send_button(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let can_send = app.is_connected();
    let style = if can_send {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(40, 40, 40))
    };

    let label = if can_send {
        " Ctrl+S Send "
    } else {
        " Connecting... "
    };

    // Right-align the button
    let padding = (area.width as usize).saturating_sub(label.len() + 2);
    let line = Line::from(vec![
        Span::raw(" ".repeat(padding)),
        Span::styled(label, style),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

// ── Inspect panel ───────────────────────────────────────────────────

fn render_inspect_panel(f: &mut Frame, app: &ExampleApp, area: Rect) {
    if app.events.is_empty() {
        let lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No webhooks sent yet. Press 'n' to compose one.",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
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
            Constraint::Length(9), // Info panels
            Constraint::Min(4),    // Body
        ])
        .split(area);

    shared::render_tab_bar(f, app.active_tab, chunks[0]);

    if let Some(evt) = app.events.get(app.selected) {
        shared::render_info_panels(f, evt, app.active_tab, chunks[1]);
        shared::render_body_panel(f, evt, app.active_tab, app.body_scroll_offset, chunks[2]);
    }
}

// ── Footer ──────────────────────────────────────────────────────────

fn render_footer(f: &mut Frame, app: &ExampleApp, area: Rect) {
    let key_style = Style::default().fg(Color::Black).bg(Color::DarkGray);

    let spans: Vec<Span<'_>> = match app.view_mode {
        ViewMode::Compose => {
            let mut s = vec![
                Span::styled(" Tab ", key_style),
                Span::raw(" Field  "),
                Span::styled(" ↑↓ ", key_style),
                Span::raw(" Select  "),
            ];
            if app.focused_field == ComposeField::StatusCode {
                s.push(Span::styled(" ←→ ", key_style));
                s.push(Span::raw(" Status  "));
            }
            s.push(Span::styled(" h ", key_style));
            s.push(Span::raw(" Headers  "));
            s.push(Span::styled(" Ctrl+S ", key_style));
            s.push(Span::raw(" Send  "));
            s.push(Span::styled(" q ", key_style));
            s.push(Span::raw(" Quit"));
            s
        }
        ViewMode::Inspect => {
            let mut s = vec![
                Span::styled(" ↑↓ ", key_style),
                Span::raw(" Navigate  "),
                Span::styled(" Tab ", key_style),
                Span::raw(" Req/Resp  "),
                Span::styled(" J/K ", key_style),
                Span::raw(" Scroll  "),
                Span::styled(" n ", key_style),
                Span::raw(" New  "),
                Span::styled(" e ", key_style),
                Span::raw(" Edit+resend  "),
                Span::styled(" q ", key_style),
                Span::raw(" Quit"),
            ];

            if !app.events.is_empty() {
                let position = format!("  {}/{}  ", app.selected + 1, app.events.len());
                let used: usize = s.iter().map(|sp| sp.content.len()).sum();
                let remaining = (area.width as usize).saturating_sub(used + position.len());
                s.push(Span::raw(" ".repeat(remaining)));
                s.push(Span::styled(position, Style::default().fg(Color::DarkGray)));
            }

            s
        }
    };

    let paragraph = Paragraph::new(Line::from(spans));
    f.render_widget(paragraph, area);
}
