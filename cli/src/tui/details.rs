use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::api::models::base64_decode;
use crate::tunnel::{InspectedRequest, RequestStatus};

/// Widget for displaying request details
pub struct DetailsWidget<'a> {
    request: &'a InspectedRequest,
    scroll_offset: usize,
}

impl<'a> DetailsWidget<'a> {
    pub fn new(request: &'a InspectedRequest, scroll_offset: usize) -> Self {
        Self {
            request,
            scroll_offset,
        }
    }

    fn format_headers(headers: &std::collections::HashMap<String, String>) -> Vec<Line<'static>> {
        let mut lines: Vec<Line> = headers
            .iter()
            .map(|(k, v)| {
                Line::from(vec![
                    Span::styled(format!("{}: ", k), Style::default().fg(Color::Cyan)),
                    Span::raw(v.clone()),
                ])
            })
            .collect();

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                "(no headers)",
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }

    fn format_status(status: &RequestStatus) -> Line<'static> {
        match status {
            RequestStatus::Pending => Line::from(Span::styled(
                "Pending...",
                Style::default().fg(Color::Yellow),
            )),
            RequestStatus::Forwarding => Line::from(Span::styled(
                "Forwarding...",
                Style::default().fg(Color::Blue),
            )),
            RequestStatus::Success { status_code, elapsed_ms } => {
                let color = if *status_code >= 400 {
                    Color::Red
                } else if *status_code >= 300 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                Line::from(vec![
                    Span::styled(format!("{}", status_code), Style::default().fg(color)),
                    Span::raw(format!(" ({} ms)", elapsed_ms)),
                ])
            }
            RequestStatus::Failed { error, elapsed_ms } => Line::from(vec![
                Span::styled("Failed", Style::default().fg(Color::Red)),
                Span::raw(format!(" ({} ms): {}", elapsed_ms, error)),
            ]),
        }
    }
}

impl<'a> Widget for DetailsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Request Details (Esc to go back, j/k to scroll)");

        let inner = block.inner(area);
        block.render(area, buf);

        // Layout: split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // Info
                Constraint::Length(8),  // Request headers
                Constraint::Min(10),    // Payload
                Constraint::Length(6),  // Response
            ])
            .split(inner);

        // Info section
        let info_lines = vec![
            Line::from(vec![
                Span::styled("Event ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.request.event_id.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Event Type: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.request.event_type.clone()),
            ]),
            Line::from(vec![
                Span::styled("Received At: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.request.received_at.to_rfc3339()),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Self::format_status(&self.request.status),
        ];

        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::BOTTOM).title("Info"));
        info.render(chunks[0], buf);

        // Request headers section
        let mut header_lines = vec![Line::from(Span::styled(
            "Request Headers",
            Style::default().add_modifier(Modifier::BOLD),
        ))];
        header_lines.extend(Self::format_headers(&self.request.headers));

        let headers_widget = Paragraph::new(header_lines)
            .block(Block::default().borders(Borders::BOTTOM));
        headers_widget.render(chunks[1], buf);

        // Payload section
        let payload_decoded = base64_decode(&self.request.payload)
            .unwrap_or_else(|_| self.request.payload.clone());

        let payload_formatted = format_json(&payload_decoded);
        let payload_lines: Vec<Line> = payload_formatted
            .lines()
            .skip(self.scroll_offset)
            .map(|line| Line::from(line.to_string()))
            .collect();

        let payload_widget = Paragraph::new(payload_lines)
            .block(Block::default().borders(Borders::BOTTOM).title("Payload"))
            .wrap(Wrap { trim: false });
        payload_widget.render(chunks[2], buf);

        // Response section (if available)
        if let Some(resp_headers) = &self.request.response_headers {
            let mut resp_lines = vec![Line::from(Span::styled(
                "Response Headers",
                Style::default().add_modifier(Modifier::BOLD),
            ))];
            resp_lines.extend(Self::format_headers(resp_headers));

            if let Some(body) = &self.request.response_body {
                resp_lines.push(Line::from(""));
                resp_lines.push(Line::from(Span::styled(
                    "Response Body:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                for line in body.lines().take(5) {
                    resp_lines.push(Line::from(line.to_string()));
                }
                if body.lines().count() > 5 {
                    resp_lines.push(Line::from(Span::styled(
                        "... (truncated)",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }

            let response_widget = Paragraph::new(resp_lines)
                .block(Block::default().title("Response"));
            response_widget.render(chunks[3], buf);
        }
    }
}

/// Format JSON for display (pretty print if valid JSON)
fn format_json(s: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| s.to_string()),
        Err(_) => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_request() -> InspectedRequest {
        InspectedRequest::new(
            "req-1".to_string(),
            Uuid::new_v4(),
            "user.account.created".to_string(),
            r#"eyJ1c2VyX2lkIjogMTIzfQ=="#.to_string(), // {"user_id": 123}
            HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            Utc::now(),
        )
    }

    #[test]
    fn test_format_json_valid() {
        let json = r#"{"key":"value"}"#;
        let formatted = format_json(json);
        assert!(formatted.contains("\"key\""));
        assert!(formatted.contains("\"value\""));
    }

    #[test]
    fn test_format_json_invalid() {
        let invalid = "not json";
        assert_eq!(format_json(invalid), "not json");
    }

    #[test]
    fn test_details_widget_creation() {
        let request = create_test_request();
        let widget = DetailsWidget::new(&request, 0);
        assert_eq!(widget.scroll_offset, 0);
    }
}
