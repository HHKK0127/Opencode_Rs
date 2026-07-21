//! Integration tests for markdown_stream module.
//! Standalone so they don't depend on lib features (postgres, etc.).

use opencode_poc::tui::markdown_stream::{MarkdownRenderer, MarkdownStream, MarkdownTheme};
use ratatui::style::Color;

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
    assert_eq!(
        ms.commit_complete_source(),
        Some("hello world\n".to_string())
    );

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
