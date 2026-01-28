use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::tunnel::{InspectedRequest, RequestStatus};

/// Widget for displaying the events list
pub struct EventsWidget<'a> {
    requests: &'a [InspectedRequest],
    selected_index: usize,
}

impl<'a> EventsWidget<'a> {
    pub fn new(requests: &'a [InspectedRequest], selected_index: usize) -> Self {
        Self {
            requests,
            selected_index,
        }
    }

    fn format_request(request: &InspectedRequest, is_selected: bool) -> ListItem<'static> {
        let time = request.received_at.format("%H:%M:%S").to_string();

        let (status_text, status_color) = match &request.status {
            RequestStatus::Pending => ("...".to_string(), Color::Yellow),
            RequestStatus::Forwarding => (">>>".to_string(), Color::Blue),
            RequestStatus::Success { status_code, elapsed_ms } => {
                let color = if *status_code >= 400 {
                    Color::Red
                } else if *status_code >= 300 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                (format!("{} {}ms", status_code, elapsed_ms), color)
            }
            RequestStatus::Failed { elapsed_ms, .. } => {
                (format!("ERR {}ms", elapsed_ms), Color::Red)
            }
        };

        let labels: String = request
            .labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");

        let event_type = truncate(&request.event_type, 30);

        let mut spans = vec![
            Span::raw(if is_selected { "► " } else { "  " }),
            Span::styled(format!("{:<10}", time), Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(format!("{:<30}", event_type), Style::default()),
            Span::raw("  "),
            Span::styled(format!("{:<12}", status_text), Style::default().fg(status_color)),
        ];

        if !labels.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                truncate(&labels, 30),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let style = if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        ListItem::new(Line::from(spans)).style(style)
    }
}

impl<'a> Widget for EventsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Events (↑/↓ j/k)");

        let inner = block.inner(area);
        block.render(area, buf);

        if self.requests.is_empty() {
            let msg = "Waiting for webhooks...";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
            return;
        }

        // Header
        let header_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        buf.set_string(
            inner.x + 2,
            inner.y,
            format!(
                "{:<10}  {:<30}  {:<12}  {}",
                "TIME", "EVENT TYPE", "STATUS", "LABELS"
            ),
            header_style,
        );

        // Events
        let items: Vec<ListItem> = self
            .requests
            .iter()
            .enumerate()
            .map(|(i, r)| Self::format_request(r, i == self.selected_index))
            .collect();

        let list = List::new(items);

        // Create a sub-area for the list (below header)
        let list_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };

        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        StatefulWidget::render(list, list_area, buf, &mut state);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
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
            "payload".to_string(),
            HashMap::new(),
            Utc::now(),
        )
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is long", 10), "this is...");
    }

    #[test]
    fn test_events_widget_creation() {
        let requests = vec![create_test_request()];
        let widget = EventsWidget::new(&requests, 0);
        assert_eq!(widget.requests.len(), 1);
    }
}
