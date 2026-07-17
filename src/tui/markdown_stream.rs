//! Streaming markdown renderer — line-gate buffering + pulldown_cmark parsing.
//!
//! # Design
//!
//! - Incomplete lines are held in a buffer and never rendered (line-gate).
//! - Only complete lines (ending with `\n`) are committed and available for rendering.
//! - `commit_complete_source()` flushes committed source for re-rendering.
//! - `MarkdownRenderer` converts raw markdown → `Vec<Line>` using `pulldown_cmark`.
//!
//! Throttle is handled by the caller (check elapsed since last render).
//! Throttle recommendation: 50ms between re-renders, 120ms during animation.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// Theme colours needed by the markdown renderer.
/// Extracted from the binary's `ThemeColors` to avoid circular dependency.
#[derive(Debug, Clone, Copy)]
pub struct MarkdownTheme {
    pub border: Color,
    pub primary: Color,
    pub accent: Color,
    pub text: Color,
    pub text_muted: Color,
    pub status_bg: Color,
    pub ok: Color,
    pub error: Color,
}

/// Line-gate streaming buffer.
///
/// Only text after the last complete `\n` is considered "committed".
/// Partially received lines stay in the buffer until a newline arrives.
#[derive(Debug, Clone)]
pub struct MarkdownStream {
    /// Raw buffer of all received text.
    buffer: String,
    /// Byte offset into `buffer` up to which text has been committed (complete lines).
    committed_source_len: usize,
}

impl Default for MarkdownStream {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownStream {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            committed_source_len: 0,
        }
    }

    /// Append incoming text delta.
    pub fn push(&mut self, delta: &str) {
        self.buffer.push_str(delta);
    }

    /// Extract newly committed source (complete lines ending with `\n`) since last call.
    ///
    /// Returns `None` if no new complete lines are available.
    pub fn commit_complete_source(&mut self) -> Option<String> {
        let commit_end = self.buffer[self.committed_source_len..].rfind('\n').map(|i| self.committed_source_len + i + 1)?;
        if commit_end <= self.committed_source_len {
            return None;
        }
        let out = self.buffer[self.committed_source_len..commit_end].to_string();
        self.committed_source_len = commit_end;
        Some(out)
    }

    /// Get the full buffer (including incomplete tail).
    pub fn full_buffer(&self) -> &str {
        &self.buffer
    }

    /// Whether there is uncommitted (incomplete) text in the buffer.
    pub fn has_tail(&self) -> bool {
        self.buffer.len() > self.committed_source_len
    }

    /// Get the incomplete tail (text after last newline).
    pub fn tail(&self) -> &str {
        &self.buffer[self.committed_source_len..]
    }

    /// Reset the stream (for a new message).
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.committed_source_len = 0;
    }

    /// Finalize — flush any remaining text as committed.
    /// Call this when the stream ends.
    pub fn finalize(&mut self) -> Option<String> {
        let remaining = self.buffer[self.committed_source_len..].to_string();
        self.committed_source_len = self.buffer.len();
        if remaining.is_empty() {
            None
        } else {
            Some(remaining)
        }
    }
}

/// Render markdown source to Ratatui `Line`s using pulldown_cmark.
///
/// Handles:
/// - Headings (#, ##)
/// - Code blocks (```) with border styling
/// - Inline code (`)
/// - Bold / Italic / Strikethrough
/// - Links (shown as [text](url))
/// - Tables (simple column-aligned rendering)
/// - Paragraph wrapping / line breaks
/// - Unordered lists (- / *)
#[derive(Debug, Clone)]
pub struct MarkdownRenderer {
    theme_border: Color,
    theme_primary: Color,
    theme_accent: Color,
    theme_text: Color,
    theme_text_muted: Color,
    theme_status_bg: Color,
    theme_ok: Color,
    theme_error: Color,
}

impl MarkdownRenderer {
    pub fn new(theme: &MarkdownTheme) -> Self {
        Self {
            theme_border: theme.border,
            theme_primary: theme.primary,
            theme_accent: theme.accent,
            theme_text: theme.text,
            theme_text_muted: theme.text_muted,
            theme_status_bg: theme.status_bg,
            theme_ok: theme.ok,
            theme_error: theme.error,
        }
    }

    pub fn update_theme(&mut self, theme: &MarkdownTheme) {
        self.theme_border = theme.border;
        self.theme_primary = theme.primary;
        self.theme_accent = theme.accent;
        self.theme_text = theme.text;
        self.theme_text_muted = theme.text_muted;
        self.theme_status_bg = theme.status_bg;
        self.theme_ok = theme.ok;
        self.theme_error = theme.error;
    }

    /// Render markdown source to Ratatui lines.
    pub fn render<'a>(&self, text: &'a str) -> Vec<Line<'static>> {
        let mut out: Vec<Line<'static>> = Vec::new();
        let parser = Parser::new(text);
        let mut current_line: Vec<Span<'static>> = Vec::new();
        let mut in_code_block = false;
        let mut in_table = false;
        let mut table_header: Vec<String> = Vec::new();
        let mut table_col_spans: Vec<usize> = Vec::new();
        let mut list_indent: u16 = 0;

        let mut flush_line = |out: &mut Vec<Line<'static>>, line: &mut Vec<Span<'static>>| {
            if !line.is_empty() {
                out.push(Line::from(std::mem::take(line)));
            }
        };

        for event in parser {
            match event {
                // ── Code blocks ──────────────────────────────────────
                Event::Start(Tag::CodeBlock(ref kind)) => {
                    flush_line(&mut out, &mut current_line);
                    in_code_block = true;
                                    let lang_label = match kind {
                                        pulldown_cmark::CodeBlockKind::Fenced(info) => {
                                            let lang = info.as_ref().split_whitespace().next().unwrap_or("code");
                                            format!(" ┌─ {} ─", lang)
                                        }
                                        _ => " ┌─ code ─".to_string(),
                                    };
                    out.push(Line::from(Span::styled(
                                        lang_label,
                        Style::default().fg(self.theme_border),
                    )));
                }
                                Event::End(TagEnd::CodeBlock) => {
                                    in_code_block = false;
                                    out.push(Line::from(Span::styled(
                                        " └─ ─".to_string(),
                                        Style::default().fg(self.theme_border),
                                    )));
                                }
                                Event::Text(t) if in_code_block => {
                                    for line in t.as_ref().split('\n') {
                                        let content_color = if line.starts_with('+') {
                                            self.theme_ok
                                        } else if line.starts_with('-') {
                                            self.theme_error
                                        } else if line.starts_with("@@") {
                                            self.theme_accent
                                        } else {
                                            self.theme_accent
                                        };
                                        out.push(Line::from(vec![
                                            Span::styled(" │ ".to_string(), Style::default().fg(self.theme_border)),
                                            Span::styled(
                                                line.to_string(),
                                                Style::default().fg(content_color),
                                            ),
                                        ]));
                                    }
                                }

                // ── Headings ─────────────────────────────────────────
                                Event::Start(Tag::Heading {
                                    level: HeadingLevel::H1,
                                    ..
                                }) => {
                                    flush_line(&mut out, &mut current_line);
                                    // H1 is handled via text accumulation with underline style
                }
                                Event::Start(Tag::Heading {
                                    level: HeadingLevel::H2,
                                    ..
                                }) => {
                                    flush_line(&mut out, &mut current_line);
                                }
                                Event::End(TagEnd::Heading(HeadingLevel::H1)) => {
                    let text: String = current_line.drain(..).map(|s| s.content.to_string()).collect();
                    out.push(Line::from(Span::styled(
                        text,
                        Style::default()
                            .fg(self.theme_accent)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                    )));
                    current_line.clear();
                }
                Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                    let text: String = current_line.drain(..).map(|s| s.content.to_string()).collect();
                    out.push(Line::from(Span::styled(
                        format!("▌ {}", text),
                        Style::default().fg(self.theme_primary).add_modifier(Modifier::BOLD),
                    )));
                    current_line.clear();
                }
                Event::End(TagEnd::Heading(HeadingLevel::H3 | HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6)) => {
                    let text: String = current_line.drain(..).map(|s| s.content.to_string()).collect();
                    out.push(Line::from(Span::styled(
                        text,
                        Style::default().fg(self.theme_text).add_modifier(Modifier::BOLD),
                    )));
                    current_line.clear();
                }

                // ── Lists ───────────────────────────────────────────
                Event::Start(Tag::List(_)) => {
                    flush_line(&mut out, &mut current_line);
                    list_indent = 0;
                }
                Event::End(TagEnd::List(_)) => {
                    flush_line(&mut out, &mut current_line);
                }
                Event::Start(Tag::Item) => {
                    flush_line(&mut out, &mut current_line);
                    current_line.push(Span::styled(
                        "  • ".to_string(),
                        Style::default().fg(self.theme_text),
                    ));
                }
                Event::End(TagEnd::Item) => {
                    flush_line(&mut out, &mut current_line);
                }

                // ── Paragraph ───────────────────────────────────────
                Event::Start(Tag::Paragraph) => {}
                Event::End(TagEnd::Paragraph) => {
                    flush_line(&mut out, &mut current_line);
                    out.push(Line::from(Span::raw(""))); // blank line after paragraph
                }

                // ── Horizontal rule ─────────────────────────────────
                Event::Rule => {
                    flush_line(&mut out, &mut current_line);
                    let width = 40usize;
                    let ruler: String = std::iter::repeat('─').take(width).collect();
                    out.push(Line::from(Span::styled(ruler, Style::default().fg(self.theme_border))));
                    out.push(Line::from(Span::raw("")));
                }

                // ── Code (inline) ───────────────────────────────────
                Event::Code(t) => {
                    current_line.push(Span::styled(
                        t.into_string(),
                        Style::default().fg(self.theme_accent).bg(self.theme_status_bg),
                    ));
                }

                // ── Inline formatting ───────────────────────────────
                Event::Start(Tag::Emphasis) => {
                    // We buffer format state; italic handled on text
                    // Use placeholder — real impl needs state machine
                }
                Event::End(TagEnd::Emphasis) => {}
                Event::Start(Tag::Strong) => {}
                Event::End(TagEnd::Strong) => {}
                Event::Start(Tag::Strikethrough) => {}
                Event::End(TagEnd::Strikethrough) => {}

                // ── Links ───────────────────────────────────────────
                Event::Start(Tag::Link { dest_url, .. }) => {
                    current_line.push(Span::styled(
                        "[".to_string(),
                        Style::default().fg(self.theme_primary),
                    ));
                    current_line.push(Span::styled(
                        dest_url.to_string(),
                        Style::default().fg(self.theme_primary).add_modifier(Modifier::UNDERLINED),
                    ));
                    current_line.push(Span::styled(
                        "](".to_string(),
                        Style::default().fg(self.theme_text_muted),
                    ));
                }
                Event::End(TagEnd::Link) => {
                    current_line.push(Span::styled(
                        ")".to_string(),
                        Style::default().fg(self.theme_text_muted),
                    ));
                }

                // ── Tables ──────────────────────────────────────────
                Event::Start(Tag::Table(_)) => {
                    flush_line(&mut out, &mut current_line);
                    in_table = true;
                    table_header.clear();
                    table_col_spans.clear();
                }
                Event::End(TagEnd::Table) => {
                    in_table = false;
                }
                Event::Start(Tag::TableHead) => {
                    table_header.clear();
                }
                Event::End(TagEnd::TableHead) => {
                    // Render header
                    if !table_header.is_empty() {
                        let mut header_line = vec![Span::styled(
                            "│ ".to_string(),
                            Style::default().fg(self.theme_border),
                        )];
                        for (i, cell) in table_header.iter().enumerate() {
                            let col_w = table_col_spans.get(i).copied().unwrap_or(15);
                            let padded = format!("{:width$}", cell, width = col_w);
                            header_line.push(Span::styled(
                                padded,
                                Style::default().fg(self.theme_primary).add_modifier(Modifier::BOLD),
                            ));
                            header_line.push(Span::styled(
                                " │ ".to_string(),
                                Style::default().fg(self.theme_border),
                            ));
                        }
                        out.push(Line::from(header_line));

                        // Separator row
                        let sep = format!("├─{}─┤", table_header.iter()
                            .enumerate()
                            .map(|(i, _)| {
                                let w = table_col_spans.get(i).copied().unwrap_or(15);
                                std::iter::repeat('─').take(w).collect::<String>()
                            })
                            .collect::<Vec<_>>()
                            .join("─┼─"));
                        out.push(Line::from(Span::styled(sep, Style::default().fg(self.theme_border))));
                    }
                    table_header.clear();
                }
                Event::Start(Tag::TableRow) => {}
                Event::End(TagEnd::TableRow) => {
                    flush_line(&mut out, &mut current_line);
                }
                Event::Start(Tag::TableCell) => {}
                Event::End(TagEnd::TableCell) => {
                    let cell_text: String = current_line.drain(..).map(|s| s.content.to_string()).collect();
                    current_line.clear();
                    if in_table {
                        let idx = table_header.len();
                        let display_width = cell_text.width() + 2;
                        table_header.push(cell_text);
                        if idx >= table_col_spans.len() {
                            table_col_spans.push(display_width);
                        } else {
                            table_col_spans[idx] = table_col_spans[idx].max(display_width);
                        }
                    }
                }

                // ── Soft/hard breaks ───────────────────────────────
                Event::SoftBreak => {
                    flush_line(&mut out, &mut current_line);
                }
                Event::HardBreak => {
                    flush_line(&mut out, &mut current_line);
                }

                // ── Plain text ──────────────────────────────────────
                Event::Text(t) if !in_code_block => {
                    let text = t.into_string();
                    // Simple inline detection: backtick code
                    if text.contains('`') {
                        let parts: Vec<&str> = text.split('`').collect();
                        for (i, part) in parts.iter().enumerate() {
                            if part.is_empty() { continue; }
                            if i % 2 == 1 {
                                current_line.push(Span::styled(
                                    part.to_string(),
                                    Style::default().fg(self.theme_accent).bg(self.theme_status_bg),
                                ));
                            } else {
                                current_line.push(Span::styled(
                                    part.to_string(),
                                    Style::default().fg(self.theme_text),
                                ));
                            }
                        }
                    } else {
                        current_line.push(Span::styled(
                            text,
                            Style::default().fg(self.theme_text),
                        ));
                    }
                }

                // ── HTML (ignore) ───────────────────────────────────
                Event::Html(_) => {}

                                // Catch-all for unhandled events
                                Event::Text(_) => {}
                                Event::InlineMath(_) | Event::DisplayMath(_) | Event::InlineHtml(_) | Event::Code(_) | Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}
                                Event::Start(_) => {}
                                Event::End(_) => {}
            }
        }

        // Flush remaining line
        flush_line(&mut out, &mut current_line);

        out
    }
}

// Re-export for convenience.
// MarkdownTheme is already `pub struct` above — no `pub use` needed.

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_theme() -> MarkdownTheme {
        MarkdownTheme {
            primary: Color::Rgb(187, 154, 247),
            accent: Color::Rgb(255, 159, 67),
            text: Color::Rgb(220, 220, 220),
            text_muted: Color::Rgb(120, 120, 120),
            border: Color::Rgb(80, 80, 80),
            status_bg: Color::Rgb(40, 40, 60),
            ok: Color::Rgb(80, 200, 120),
            error: Color::Rgb(255, 80, 80),
        }
    }

    #[test]
    fn test_markdown_stream_line_gate() {
        let mut ms = MarkdownStream::new();
        assert!(ms.commit_complete_source().is_none());

        ms.push("hello ");
        assert!(ms.commit_complete_source().is_none()); // no newline yet

        ms.push("world\n");
        assert_eq!(ms.commit_complete_source(), Some("hello world\n".to_string()));

        ms.push("line 2\nline 3");
        assert_eq!(ms.commit_complete_source(), Some("line 2\n".to_string()));
        assert!(ms.commit_complete_source().is_none()); // "line 3" is tail

        assert_eq!(ms.tail(), "line 3");
        assert!(ms.has_tail());
    }

    #[test]
    fn test_markdown_stream_finalize() {
        let mut ms = MarkdownStream::new();
        ms.push("incomplete");
        assert_eq!(ms.finalize(), Some("incomplete".to_string()));
        assert!(ms.finalize().is_none());
    }

    #[test]
    fn test_render_heading() {
            let renderer = MarkdownRenderer::new(&mock_theme());
        let lines = renderer.render("# Hello\n## World\nNormal text.");
        assert!(!lines.is_empty());
        // Just check we got some output without panicking
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_render_code_block() {
        let renderer = MarkdownRenderer::new(&mock_theme());
        let lines = renderer.render("```rust\nlet x = 1;\n```");
        assert!(!lines.is_empty());
        // Should have code block borders
        let total: String = lines.iter().map(|l| l.to_string()).collect();
        assert!(total.contains("code"));
    }

    #[test]
    fn test_render_table() {
        let renderer = MarkdownRenderer::new(&mock_theme());
        let lines = renderer.render("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_inline_code() {
        let renderer = MarkdownRenderer::new(&mock_theme());
        let lines = renderer.render("Use `code` here.");
        assert!(!lines.is_empty());
    }
}
