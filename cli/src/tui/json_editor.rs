//! Inline JSON editor widget with cursor state and syntax highlighting.

use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// State for a multi-line JSON text editor.
#[derive(Debug, Clone)]
pub struct JsonEditorState {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub dirty: bool,
}

impl JsonEditorState {
    /// Create a new editor from a JSON string.
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(String::from).collect()
        };
        // If the text ends with a newline, lines() won't include the trailing empty line
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            dirty: false,
        }
    }

    /// Get the full text content.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, ch: char) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.lines[row].len());
        self.lines[row].insert(col, ch);
        self.cursor_col = col + ch.len_utf8();
        self.dirty = true;
    }

    /// Insert a newline at the cursor position, splitting the current line.
    pub fn insert_newline(&mut self) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.lines[row].len());
        let remainder = self.lines[row][col..].to_string();
        self.lines[row].truncate(col);
        self.cursor_row = row + 1;
        self.lines.insert(self.cursor_row, remainder);
        self.cursor_col = 0;
        self.dirty = true;
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(&mut self) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.lines[row].len());

        if col > 0 {
            // Find the byte position of the previous character
            let prev_char_start = self.lines[row][..col]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.lines[row].remove(prev_char_start);
            self.cursor_col = prev_char_start;
            self.dirty = true;
        } else if row > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(row);
            self.cursor_row = row - 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
            self.dirty = true;
        }
    }

    /// Delete the character at the cursor position.
    pub fn delete(&mut self) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.lines[row].len());

        if col < self.lines[row].len() {
            self.lines[row].remove(col);
            self.dirty = true;
        } else if row + 1 < self.lines.len() {
            // Merge next line into current
            let next_line = self.lines.remove(row + 1);
            self.lines[row].push_str(&next_line);
            self.dirty = true;
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            // Find the start of the previous character
            let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
            let col = self.cursor_col.min(self.lines[row].len());
            self.cursor_col = self.lines[row][..col]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.lines[row].len());
        if col < self.lines[row].len() {
            // Find the start of the next character
            self.cursor_col = self.lines[row][col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| col + i)
                .unwrap_or(self.lines[row].len());
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    /// Move cursor to beginning of line.
    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line.
    pub fn move_end(&mut self) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        self.cursor_col = self.lines[row].len();
    }

    /// Adjust scroll offset so the cursor row is visible within the given viewport height.
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        } else if self.cursor_row >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor_row - viewport_height + 1;
        }
    }

    /// Number of visible lines from the current scroll position.
    pub fn visible_lines(&self, viewport_height: usize) -> impl Iterator<Item = (usize, &str)> {
        self.lines
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(viewport_height)
            .map(|(i, line)| (i, line.as_str()))
    }
}

// ── JSON Syntax Highlighting ────────────────────────────────────────

/// State machine for JSON syntax coloring within a single line.
#[derive(Clone, Copy, PartialEq, Eq)]
enum JsonTokenState {
    /// Outside any token
    Normal,
    /// Inside a string that is a key (before the colon)
    Key,
    /// Inside a string that is a value (after the colon)
    StringValue,
    /// Accumulating a number
    Number,
    /// Accumulating a keyword (true, false, null)
    Keyword,
}

/// Highlight a single line of JSON, returning styled spans.
pub fn highlight_json_line(line: &str) -> Vec<Span<'static>> {
    if line.is_empty() {
        return vec![Span::raw(String::new())];
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut state = JsonTokenState::Normal;
    let mut current = String::new();
    let mut saw_colon = false;
    let mut escaped = false;

    let flush = |spans: &mut Vec<Span<'static>>, current: &mut String, state: JsonTokenState| {
        if !current.is_empty() {
            let style = match state {
                JsonTokenState::Key => Style::default().fg(Color::Cyan),
                JsonTokenState::StringValue => Style::default().fg(Color::Green),
                JsonTokenState::Number => Style::default().fg(Color::Yellow),
                JsonTokenState::Keyword => Style::default().fg(Color::Magenta),
                JsonTokenState::Normal => Style::default().fg(Color::White),
            };
            spans.push(Span::styled(std::mem::take(current), style));
        }
    };

    for ch in line.chars() {
        match state {
            JsonTokenState::Key | JsonTokenState::StringValue => {
                current.push(ch);
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    flush(&mut spans, &mut current, state);
                    state = JsonTokenState::Normal;
                }
            }
            JsonTokenState::Number => {
                if ch.is_ascii_digit()
                    || ch == '.'
                    || ch == 'e'
                    || ch == 'E'
                    || ch == '+'
                    || ch == '-'
                {
                    current.push(ch);
                } else {
                    flush(&mut spans, &mut current, state);
                    state = JsonTokenState::Normal;
                    // Re-process this character in Normal state
                    match ch {
                        '"' => {
                            state = if saw_colon {
                                JsonTokenState::StringValue
                            } else {
                                JsonTokenState::Key
                            };
                            current.push(ch);
                        }
                        ':' => {
                            saw_colon = true;
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default().fg(Color::White),
                            ));
                        }
                        ',' => {
                            saw_colon = false;
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default().fg(Color::White),
                            ));
                        }
                        '{' | '}' | '[' | ']' => {
                            saw_colon = false;
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default().fg(Color::White),
                            ));
                        }
                        _ => {
                            current.push(ch);
                        }
                    }
                }
            }
            JsonTokenState::Keyword => {
                if ch.is_ascii_alphabetic() {
                    current.push(ch);
                } else {
                    // Check what keyword we have
                    let kw_state = match current.as_str() {
                        "true" | "false" => JsonTokenState::Keyword,
                        "null" => JsonTokenState::Keyword,
                        _ => JsonTokenState::Normal,
                    };
                    flush(&mut spans, &mut current, kw_state);
                    state = JsonTokenState::Normal;
                    // Re-process this character
                    match ch {
                        '"' => {
                            state = if saw_colon {
                                JsonTokenState::StringValue
                            } else {
                                JsonTokenState::Key
                            };
                            current.push(ch);
                        }
                        ':' => {
                            saw_colon = true;
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default().fg(Color::White),
                            ));
                        }
                        ',' => {
                            saw_colon = false;
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default().fg(Color::White),
                            ));
                        }
                        '{' | '}' | '[' | ']' => {
                            saw_colon = false;
                            spans.push(Span::styled(
                                ch.to_string(),
                                Style::default().fg(Color::White),
                            ));
                        }
                        _ => {
                            current.push(ch);
                        }
                    }
                }
            }
            JsonTokenState::Normal => match ch {
                '"' => {
                    flush(&mut spans, &mut current, state);
                    state = if saw_colon {
                        JsonTokenState::StringValue
                    } else {
                        JsonTokenState::Key
                    };
                    current.push(ch);
                }
                ':' => {
                    flush(&mut spans, &mut current, state);
                    saw_colon = true;
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::White),
                    ));
                }
                ',' => {
                    flush(&mut spans, &mut current, state);
                    saw_colon = false;
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::White),
                    ));
                }
                '{' | '}' | '[' | ']' => {
                    flush(&mut spans, &mut current, state);
                    saw_colon = false;
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::White),
                    ));
                }
                c if c.is_ascii_digit() || c == '-' => {
                    flush(&mut spans, &mut current, state);
                    state = JsonTokenState::Number;
                    current.push(ch);
                }
                't' | 'f' | 'n' => {
                    flush(&mut spans, &mut current, state);
                    state = JsonTokenState::Keyword;
                    current.push(ch);
                }
                _ => {
                    current.push(ch);
                }
            },
        }
    }

    // Flush remaining content
    let final_state = match state {
        JsonTokenState::Keyword => match current.as_str() {
            "true" | "false" | "null" => JsonTokenState::Keyword,
            _ => JsonTokenState::Normal,
        },
        other => other,
    };
    flush(&mut spans, &mut current, final_state);

    if spans.is_empty() {
        spans.push(Span::raw(String::new()));
    }

    spans
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── JsonEditorState ─────────────────────────────────────────

    #[test]
    fn from_text_empty() {
        let ed = JsonEditorState::from_text("");
        assert_eq!(ed.lines, vec![""]);
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.cursor_col, 0);
        assert!(!ed.dirty);
    }

    #[test]
    fn from_text_multiline() {
        let ed = JsonEditorState::from_text("{\n  \"key\": 1\n}");
        assert_eq!(ed.lines.len(), 3);
        assert_eq!(ed.lines[0], "{");
        assert_eq!(ed.lines[1], "  \"key\": 1");
        assert_eq!(ed.lines[2], "}");
    }

    #[test]
    fn insert_char_at_beginning() {
        let mut ed = JsonEditorState::from_text("hello");
        ed.insert_char('X');
        assert_eq!(ed.lines[0], "Xhello");
        assert_eq!(ed.cursor_col, 1);
        assert!(ed.dirty);
    }

    #[test]
    fn insert_char_at_end() {
        let mut ed = JsonEditorState::from_text("hello");
        ed.cursor_col = 5;
        ed.insert_char('!');
        assert_eq!(ed.lines[0], "hello!");
        assert_eq!(ed.cursor_col, 6);
    }

    #[test]
    fn insert_char_in_middle() {
        let mut ed = JsonEditorState::from_text("hllo");
        ed.cursor_col = 1;
        ed.insert_char('e');
        assert_eq!(ed.lines[0], "hello");
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut ed = JsonEditorState::from_text("hello world");
        ed.cursor_col = 5;
        ed.insert_newline();
        assert_eq!(ed.lines.len(), 2);
        assert_eq!(ed.lines[0], "hello");
        assert_eq!(ed.lines[1], " world");
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn insert_newline_at_end() {
        let mut ed = JsonEditorState::from_text("hello");
        ed.cursor_col = 5;
        ed.insert_newline();
        assert_eq!(ed.lines.len(), 2);
        assert_eq!(ed.lines[0], "hello");
        assert_eq!(ed.lines[1], "");
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn backspace_deletes_previous_char() {
        let mut ed = JsonEditorState::from_text("hello");
        ed.cursor_col = 3;
        ed.backspace();
        assert_eq!(ed.lines[0], "helo");
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn backspace_at_line_start_merges_lines() {
        let mut ed = JsonEditorState::from_text("hello\nworld");
        ed.cursor_row = 1;
        ed.cursor_col = 0;
        ed.backspace();
        assert_eq!(ed.lines.len(), 1);
        assert_eq!(ed.lines[0], "helloworld");
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.cursor_col, 5);
    }

    #[test]
    fn backspace_at_beginning_of_first_line_does_nothing() {
        let mut ed = JsonEditorState::from_text("hello");
        ed.cursor_col = 0;
        ed.backspace();
        assert_eq!(ed.lines[0], "hello");
        assert_eq!(ed.cursor_col, 0);
        assert!(!ed.dirty);
    }

    #[test]
    fn delete_at_cursor() {
        let mut ed = JsonEditorState::from_text("hello");
        ed.cursor_col = 2;
        ed.delete();
        assert_eq!(ed.lines[0], "helo");
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn delete_at_end_merges_next_line() {
        let mut ed = JsonEditorState::from_text("hello\nworld");
        ed.cursor_col = 5;
        ed.delete();
        assert_eq!(ed.lines.len(), 1);
        assert_eq!(ed.lines[0], "helloworld");
    }

    #[test]
    fn delete_at_end_of_last_line_does_nothing() {
        let mut ed = JsonEditorState::from_text("hello");
        ed.cursor_col = 5;
        ed.delete();
        assert_eq!(ed.lines[0], "hello");
    }

    #[test]
    fn move_left_right() {
        let mut ed = JsonEditorState::from_text("abc");
        ed.cursor_col = 1;
        ed.move_left();
        assert_eq!(ed.cursor_col, 0);
        ed.move_right();
        assert_eq!(ed.cursor_col, 1);
        ed.move_right();
        assert_eq!(ed.cursor_col, 2);
        ed.move_right();
        assert_eq!(ed.cursor_col, 3);
    }

    #[test]
    fn move_left_wraps_to_previous_line() {
        let mut ed = JsonEditorState::from_text("abc\ndef");
        ed.cursor_row = 1;
        ed.cursor_col = 0;
        ed.move_left();
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.cursor_col, 3);
    }

    #[test]
    fn move_right_wraps_to_next_line() {
        let mut ed = JsonEditorState::from_text("abc\ndef");
        ed.cursor_col = 3;
        ed.move_right();
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn move_up_down() {
        let mut ed = JsonEditorState::from_text("abc\ndef\nghi");
        ed.cursor_row = 0;
        ed.cursor_col = 2;
        ed.move_down();
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 2);
        ed.move_down();
        assert_eq!(ed.cursor_row, 2);
        ed.move_up();
        assert_eq!(ed.cursor_row, 1);
    }

    #[test]
    fn move_up_clamps_at_top() {
        let mut ed = JsonEditorState::from_text("abc");
        ed.move_up();
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn move_down_clamps_at_bottom() {
        let mut ed = JsonEditorState::from_text("abc");
        ed.move_down();
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn move_down_clamps_col_to_shorter_line() {
        let mut ed = JsonEditorState::from_text("long line\nhi");
        ed.cursor_col = 8;
        ed.move_down();
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 2); // "hi" is only 2 chars
    }

    #[test]
    fn move_home_end() {
        let mut ed = JsonEditorState::from_text("hello world");
        ed.cursor_col = 5;
        ed.move_home();
        assert_eq!(ed.cursor_col, 0);
        ed.move_end();
        assert_eq!(ed.cursor_col, 11);
    }

    #[test]
    fn ensure_visible_scrolls_down() {
        let mut ed = JsonEditorState::from_text("a\nb\nc\nd\ne\nf");
        ed.cursor_row = 5;
        ed.scroll_offset = 0;
        ed.ensure_visible(3);
        assert_eq!(ed.scroll_offset, 3); // row 5 visible in a 3-line viewport
    }

    #[test]
    fn ensure_visible_scrolls_up() {
        let mut ed = JsonEditorState::from_text("a\nb\nc\nd\ne\nf");
        ed.cursor_row = 1;
        ed.scroll_offset = 4;
        ed.ensure_visible(3);
        assert_eq!(ed.scroll_offset, 1);
    }

    #[test]
    fn ensure_visible_no_change_when_visible() {
        let mut ed = JsonEditorState::from_text("a\nb\nc");
        ed.cursor_row = 1;
        ed.scroll_offset = 0;
        ed.ensure_visible(3);
        assert_eq!(ed.scroll_offset, 0);
    }

    #[test]
    fn text_roundtrip() {
        let original = "{\n  \"key\": \"value\"\n}";
        let ed = JsonEditorState::from_text(original);
        assert_eq!(ed.text(), original);
    }

    #[test]
    fn utf8_insert_and_backspace() {
        let mut ed = JsonEditorState::from_text("");
        ed.insert_char('é');
        assert_eq!(ed.lines[0], "é");
        assert_eq!(ed.cursor_col, 2); // 'é' is 2 bytes in UTF-8
        ed.backspace();
        assert_eq!(ed.lines[0], "");
        assert_eq!(ed.cursor_col, 0);
    }

    // ── highlight_json_line ─────────────────────────────────────

    fn spans_text(spans: &[Span]) -> String {
        spans.iter().map(|s| s.content.to_string()).collect()
    }

    #[test]
    fn highlight_empty_line() {
        let spans = highlight_json_line("");
        assert_eq!(spans_text(&spans), "");
    }

    #[test]
    fn highlight_key_value_string() {
        let spans = highlight_json_line("\"name\": \"hook0\"");
        let text = spans_text(&spans);
        assert_eq!(text, "\"name\": \"hook0\"");
        // "name" should be cyan (key)
        assert_eq!(spans[0].style.fg, Some(Color::Cyan));
        // "hook0" should be green (string value)
        let value_span = spans.iter().find(|s| s.content.contains("hook0")).unwrap();
        assert_eq!(value_span.style.fg, Some(Color::Green));
    }

    #[test]
    fn highlight_number() {
        let spans = highlight_json_line("\"count\": 42");
        let num_span = spans.iter().find(|s| s.content.contains("42")).unwrap();
        assert_eq!(num_span.style.fg, Some(Color::Yellow));
    }

    #[test]
    fn highlight_boolean_true() {
        let spans = highlight_json_line("\"active\": true");
        let bool_span = spans.iter().find(|s| s.content.contains("true")).unwrap();
        assert_eq!(bool_span.style.fg, Some(Color::Magenta));
    }

    #[test]
    fn highlight_boolean_false() {
        let spans = highlight_json_line("\"active\": false");
        let bool_span = spans.iter().find(|s| s.content.contains("false")).unwrap();
        assert_eq!(bool_span.style.fg, Some(Color::Magenta));
    }

    #[test]
    fn highlight_null() {
        let spans = highlight_json_line("\"value\": null");
        let null_span = spans.iter().find(|s| s.content.contains("null")).unwrap();
        assert_eq!(null_span.style.fg, Some(Color::Magenta));
    }

    #[test]
    fn highlight_braces() {
        let spans = highlight_json_line("{");
        let text = spans_text(&spans);
        assert_eq!(text, "{");
        assert_eq!(spans[0].style.fg, Some(Color::White));
    }

    #[test]
    fn highlight_nested_object() {
        let spans = highlight_json_line("{\"a\": {\"b\": 1}}");
        let text = spans_text(&spans);
        assert_eq!(text, "{\"a\": {\"b\": 1}}");
    }

    #[test]
    fn highlight_escaped_quotes() {
        let spans = highlight_json_line("\"say \\\"hello\\\"\"");
        let text = spans_text(&spans);
        assert!(text.contains("\\\"hello\\\""));
    }

    #[test]
    fn highlight_negative_number() {
        let spans = highlight_json_line("\"val\": -3.14");
        let num_span = spans.iter().find(|s| s.content.contains("-3.14")).unwrap();
        assert_eq!(num_span.style.fg, Some(Color::Yellow));
    }
}
