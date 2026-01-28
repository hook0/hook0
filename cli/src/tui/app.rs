use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use tokio::sync::mpsc;

use crate::tunnel::Inspector;
use super::events::EventsWidget;
use super::details::DetailsWidget;

/// TUI application state
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    inspector: Inspector,
    webhook_url: String,
    target_url: String,
    selected_index: usize,
    show_details: bool,
    scroll_offset: usize,
    start_time: Instant,
    should_quit: bool,
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new(inspector: Inspector, webhook_url: String, target_url: String) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            inspector,
            webhook_url,
            target_url,
            selected_index: 0,
            show_details: false,
            scroll_offset: 0,
            start_time: Instant::now(),
            should_quit: false,
        })
    }

    /// Run the TUI event loop
    pub async fn run(&mut self, update_rx: &mut mpsc::Receiver<()>) -> Result<()> {
        loop {
            // Prepare draw data
            let draw_data = DrawData {
                webhook_url: self.webhook_url.clone(),
                target_url: self.target_url.clone(),
                selected_index: self.selected_index,
                show_details: self.show_details,
                scroll_offset: self.scroll_offset,
                uptime: self.format_uptime(),
                inspector: self.inspector.clone(),
            };

            // Draw UI
            self.terminal.draw(|f| draw_ui(f, &draw_data))?;

            // Handle events with timeout
            tokio::select! {
                // Check for terminal events
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if event::poll(Duration::from_millis(0))? {
                        if let Event::Key(key) = event::read()? {
                            self.handle_key(key.code, key.modifiers);
                        }
                    }
                }
                // Check for updates from stream
                _ = update_rx.recv() => {
                    // Requests updated, will redraw on next loop
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Restore terminal
        self.restore_terminal()?;

        Ok(())
    }

    fn restore_terminal(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            // Quit
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if self.show_details {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                } else {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.show_details {
                    self.scroll_offset += 1;
                } else {
                    let count = self.inspector.count();
                    if count > 0 && self.selected_index < count - 1 {
                        self.selected_index += 1;
                    }
                }
            }
            KeyCode::PageUp => {
                if self.show_details {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                } else {
                    self.selected_index = self.selected_index.saturating_sub(10);
                }
            }
            KeyCode::PageDown => {
                if self.show_details {
                    self.scroll_offset += 10;
                } else {
                    let count = self.inspector.count();
                    self.selected_index = (self.selected_index + 10).min(count.saturating_sub(1));
                }
            }
            KeyCode::Home => {
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
            KeyCode::End => {
                let count = self.inspector.count();
                self.selected_index = count.saturating_sub(1);
            }

            // Show/hide details
            KeyCode::Enter | KeyCode::Char('d') => {
                self.show_details = !self.show_details;
                self.scroll_offset = 0;
            }
            KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                    self.scroll_offset = 0;
                }
            }

            // Replay
            KeyCode::Char('r') => {
                // TODO: Implement replay
            }

            // Open in browser
            KeyCode::Char('o') => {
                // TODO: Open event in dashboard
            }

            _ => {}
        }
    }

    fn format_uptime(&self) -> String {
        let elapsed = self.start_time.elapsed();
        let secs = elapsed.as_secs();
        let mins = secs / 60;
        let hours = mins / 60;

        if hours > 0 {
            format!("{}h {}m", hours, mins % 60)
        } else if mins > 0 {
            format!("{}m {}s", mins, secs % 60)
        } else {
            format!("{}s", secs)
        }
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        let _ = self.restore_terminal();
    }
}

/// Data needed for drawing
struct DrawData {
    webhook_url: String,
    target_url: String,
    selected_index: usize,
    show_details: bool,
    scroll_offset: usize,
    uptime: String,
    inspector: Inspector,
}

/// Draw the UI
fn draw_ui(frame: &mut Frame, data: &DrawData) {
    let size = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header
            Constraint::Min(10),    // Events list or details
            Constraint::Length(2),  // Footer
        ])
        .split(size);

    // Draw header
    draw_header(frame, chunks[0], data);

    // Draw events list or details
    if data.show_details {
        draw_details(frame, chunks[1], data);
    } else {
        draw_events(frame, chunks[1], data);
    }

    // Draw footer
    draw_footer(frame, chunks[2], data);
}

fn draw_header(frame: &mut Frame, area: Rect, data: &DrawData) {
    let total = data.inspector.count();
    let events_info = format!(" | Events: {} | Uptime: {}", total, data.uptime);

    let header_text = vec![
        Line::from(vec![
            Span::styled("Hook0 Local Listener", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Webhook URL: "),
            Span::styled(data.webhook_url.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Forwarding to: "),
            Span::styled(data.target_url.clone(), Style::default().fg(Color::Green)),
            Span::raw(" | Status: "),
            Span::styled("Connected", Style::default().fg(Color::Green)),
            Span::raw(events_info),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Hook0"));

    frame.render_widget(header, area);
}

fn draw_events(frame: &mut Frame, area: Rect, data: &DrawData) {
    let requests = data.inspector.list();
    let events_widget = EventsWidget::new(&requests, data.selected_index);
    frame.render_widget(events_widget, area);
}

fn draw_details(frame: &mut Frame, area: Rect, data: &DrawData) {
    let requests = data.inspector.list();
    if let Some(request) = requests.get(data.selected_index) {
        let details_widget = DetailsWidget::new(request, data.scroll_offset);
        frame.render_widget(details_widget, area);
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, data: &DrawData) {
    let shortcuts = if data.show_details {
        "[Esc] Back  [j/k] Scroll  [r] Replay  [o] Open Dashboard  [q] Quit"
    } else {
        "[j/k/Up/Down] Navigate  [Enter/d] Details  [r] Replay  [o] Open Dashboard  [i] Info  [q] Quit"
    };

    let footer = Paragraph::new(shortcuts)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, area);
}
