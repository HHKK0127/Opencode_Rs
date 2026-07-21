//! Editor component — code editor with inline completions (ghost text).
//!
//! Provides:
//! - Line buffer (`Vec<String>`) with cursor movement
//! - Insert / Normal mode switching
//! - Inline suggestion (ghost text) display
//! - Scrolling for files larger than viewport

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

/// Editor mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
}

/// Visual focus of the split panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Editor,
    Chat,
}

/// A suggestion (inline completion) from the LLM.
#[derive(Debug, Clone, Default)]
pub struct Suggestion {
    /// The suggested text to insert at the cursor.
    pub text: String,
    /// Whether the suggestion is currently visible.
    pub visible: bool,
}

/// Full editor state.
#[derive(Debug, Clone)]
pub struct EditorState {
    /// Lines of text in the buffer.
    pub lines: Vec<String>,
    /// Current cursor row (0-indexed).
    pub row: usize,
    /// Current cursor column (0-indexed, byte offset).
    pub col: usize,
    /// Scroll offset (how many lines from top are hidden).
    pub scroll_offset: usize,
    /// Editor mode.
    pub mode: EditorMode,
    /// Whether the editor is visible.
    pub visible: bool,
    /// Current suggestion (inline completion).
    pub suggestion: Suggestion,
    /// Line number width for gutter.
    pub line_number_width: usize,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            row: 0,
            col: 0,
            scroll_offset: 0,
            mode: EditorMode::Normal,
            visible: false,
            suggestion: Suggestion::default(),
            line_number_width: 3,
        }
    }
}

impl EditorState {
    /// Insert a character at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        let line = &mut self.lines[self.row];
        line.insert(self.col.min(line.len()), c);
        self.col += c.len_utf8();
    }

    /// Insert a newline at the current cursor position.
    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.row];
        let rest = line.split_off(self.col.min(line.len()));
        self.lines.insert(self.row + 1, rest);
        self.row += 1;
        self.col = 0;
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(&mut self) {
        if self.col > 0 {
            let line = &mut self.lines[self.row];
            let prev = line[..self.col].chars().next_back().unwrap();
            let len = prev.len_utf8();
            line.drain(self.col - len..self.col);
            self.col -= len;
        } else if self.row > 0 {
            // Join with previous line
            let prev_line = self.lines.remove(self.row);
            self.row -= 1;
            let new_col = self.lines[self.row].len();
            self.lines[self.row].push_str(&prev_line);
            self.col = new_col;
        }
    }

    /// Delete the character at the cursor (delete).
    pub fn delete(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        let line_len = self.lines[self.row].len();
        if self.col < line_len {
            let next = self.lines[self.row][self.col..].chars().next().unwrap();
            let len = next.len_utf8();
            self.lines[self.row].drain(self.col..self.col + len);
        } else if self.row + 1 < self.lines.len() {
            // Merge with next line
            let next_line = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next_line);
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.col > 0 {
            let prev = self.lines[self.row][..self.col]
                .chars()
                .next_back()
                .unwrap();
            self.col -= prev.len_utf8();
        } else if self.row > 0 {
            self.row -= 1;
            self.col = self.lines[self.row].len();
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        if self.col < self.lines[self.row].len() {
            self.col += self.lines[self.row][self.col..]
                .chars()
                .next()
                .unwrap()
                .len_utf8();
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0;
        }
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(self.lines[self.row].len());
        }
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = self.col.min(self.lines[self.row].len());
        }
    }

    /// Move to start of line.
    pub fn home(&mut self) {
        self.col = 0;
    }

    /// Move to end of line.
    pub fn end(&mut self) {
        self.col = self.lines[self.row].len();
    }

    /// Accept the current suggestion (insert full text at cursor).
    pub fn accept_suggestion(&mut self) {
        if !self.suggestion.visible || self.suggestion.text.is_empty() {
            return;
        }
        let text = self.suggestion.text.clone();
        // Insert the suggestion text at the current cursor position
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        let line = &mut self.lines[self.row];
        line.insert_str(self.col.min(line.len()), &text);
        self.col += text.len();
        self.suggestion.visible = false;
        self.suggestion.text.clear();
    }

    /// Ensure cursor is visible (adjust scroll offset).
    pub fn ensure_cursor_visible(&mut self, view_height: usize) {
        if view_height == 0 {
            return;
        }
        if self.row < self.scroll_offset {
            self.scroll_offset = self.row;
        } else if self.row >= self.scroll_offset + view_height {
            self.scroll_offset = self.row - view_height + 1;
        }
    }

    /// Get the visible slice of lines based on scroll offset and view height.
    pub fn visible_lines(&self, view_height: usize) -> &[String] {
        let start = self.scroll_offset.min(self.lines.len().saturating_sub(1));
        let end = (start + view_height).min(self.lines.len());
        if start >= end {
            &self.lines[start..start]
        } else {
            &self.lines[start..end]
        }
    }

    /// Render the editor into the given area.
    pub fn render(&self, frame: &mut Frame, area: Rect, theme_color: &super::MarkdownTheme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Editor [{}] ",
                match self.mode {
                    EditorMode::Normal => "NORMAL",
                    EditorMode::Insert => "INSERT",
                }
            ))
            .border_style(Style::default().fg(theme_color.border));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let view_height = inner.height as usize;
        let width = inner.width as usize;
        if width == 0 {
            return;
        }

        let line_number_w = self.line_number_width;
        let content_width = width.saturating_sub(line_number_w + 2);

        let mut lines: Vec<Line> = Vec::new();
        let visible = self.visible_lines(view_height);
        let total_lines = self.lines.len();

        for (i, text) in visible.iter().enumerate() {
            let global_line = self.scroll_offset + i;
            let line_num = format!("{:>width$} ", global_line + 1, width = line_number_w);

            let is_cursor_line = global_line == self.row;
            let line_style = if is_cursor_line && self.mode == EditorMode::Insert {
                Style::default().fg(theme_color.accent)
            } else {
                Style::default().fg(theme_color.text)
            };

            // Truncate or wrap
            let display_text = if text.len() > content_width {
                format!("{}…", &text[..content_width.saturating_sub(1)])
            } else {
                text.clone()
            };

            // Build line with gutter
            let used = display_text.len();
            let spans = vec![
                Span::styled(line_num, Style::default().fg(theme_color.text_muted)),
                Span::raw("│"),
                Span::styled(display_text, line_style),
            ];

            // If this is the cursor line and there's a suggestion, show ghost text
            if is_cursor_line && self.suggestion.visible && !self.suggestion.text.is_empty() {
                let ghost_text = &self.suggestion.text;
                // Only show ghost if it fits on screen
                let available = content_width.saturating_sub(used);
                if available > 2 {
                    let ghost = if ghost_text.len() > available {
                        format!("{}…", &ghost_text[..available.saturating_sub(1)])
                    } else {
                        ghost_text.clone()
                    };
                    // Add ghost text with dim style
                    let mut full_spans = spans;
                    full_spans.push(Span::styled(
                        ghost,
                        Style::default()
                            .fg(theme_color.text_muted)
                            .add_modifier(Modifier::DIM),
                    ));
                    lines.push(Line::from(full_spans));
                    continue;
                }
            }

            lines.push(Line::from(spans));
        }

        // Fill remaining lines
        while lines.len() < view_height {
            lines.push(Line::from(Span::raw("")));
        }

        // Show mode indicator at bottom of editor
        if view_height > 1 {
            let mode_str = match self.mode {
                EditorMode::Normal => "NORMAL",
                EditorMode::Insert => "INSERT",
            };
            let mode_indicator = format!(
                " {}  |  Ln {}  Col {}  |  {} lines ",
                mode_str,
                self.row + 1,
                self.col + 1,
                total_lines,
            );
            // Replace last line with mode indicator
            let indicator_style = Style::default()
                .fg(theme_color.text_muted)
                .add_modifier(Modifier::DIM);
            if !lines.is_empty() {
                *lines.last_mut().unwrap() =
                    Line::from(Span::styled(mode_indicator, indicator_style));
            }
        }

        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}
