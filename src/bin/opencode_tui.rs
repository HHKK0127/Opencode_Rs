#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_assignments,
    clippy::all
)]

//! OpenCode_Rs TUI — Ratatui-based chat interface for `opencode-llm`.
//!
//! Uses `ConversationRuntime` directly via `opencode-llm` crate.
//! Supports Anthropic native and OpenAI-compatible providers.

#![allow(
    dead_code,
    unused_variables,
    unused_assignments,
    unused_imports,
    unreachable_patterns
)]

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::panic;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::Terminal;

use opencode_llm::auth::AuthSource;
use opencode_llm::config::LlmConfig;
use opencode_llm::conversation::ConversationRuntime;
use opencode_llm::providers::anthropic::AnthropicClient;
use opencode_llm::providers::openai_compat::OpenAiCompatClient;
use opencode_llm::providers::Provider;

use opencode_poc::tui::component::{Component, ComponentKind, HandleResult, LayerId};
use opencode_poc::tui::compositor::Compositor;
use opencode_poc::tui::editor::{EditorMode, EditorState, PanelFocus, Suggestion};
use opencode_poc::tui::{MarkdownRenderer, MarkdownStream, MarkdownTheme};

mod slash_commands;
use slash_commands::{
    parse_slash_command, SlashAction, SlashCommandDispatcher, SlashCommandResult, BUILTIN_COMMANDS,
};

/// OpenCode ASCII logo (left-aligned).
const OPENCODE_LOGO: &str = r###"█▀▀█ █▀▀█ █▀▀█ █▀▀▄ █▀▀▀ █▀▀█ █▀▀█ █▀▀█
█  █ █  █ █▀▀▀ █  █ █    █  █ █  █ █▀▀▀
▀▀▀▀ █▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀"###;

// ---------------------------------------------------------------------------
// Chat message model (cell-based transcript)
// ---------------------------------------------------------------------------

/// A cell in the conversation transcript. Each cell carries role, content,
/// and metadata. New cell variants (e.g. tool results, image references)
/// can be added without changing the transcript storage type.
trait HistoryCell: Send {
    fn role(&self) -> MsgRole;
    fn content(&self) -> &str;
    fn timestamp(&self) -> Instant;
    fn role_label(&self) -> &'static str;
    fn role_color(&self) -> Color;
}

#[derive(Clone)]
struct ChatMsg {
    role: MsgRole,
    text: String,
    timestamp: Instant,
}

impl HistoryCell for ChatMsg {
    fn role(&self) -> MsgRole {
        self.role.clone()
    }
    fn content(&self) -> &str {
        &self.text
    }
    fn timestamp(&self) -> Instant {
        self.timestamp
    }

    fn role_label(&self) -> &'static str {
        match self.role {
            MsgRole::User => "You",
            MsgRole::Assistant => "AI",
            MsgRole::System => "System",
            MsgRole::Error => "Error",
        }
    }

    fn role_color(&self) -> Color {
        match self.role {
            MsgRole::User => Color::Cyan,
            MsgRole::Assistant => Color::Green,
            MsgRole::System => Color::Yellow,
            MsgRole::Error => Color::Red,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MsgRole {
    User,
    Assistant,
    System,
    Error,
}

// ---------------------------------------------------------------------------
// Tool call model (Hermes-inspired collapsible tool call)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum ToolStatus {
    /// Awaiting user approval before execution.
    Pending,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone)]
struct ToolCallEntry {
    id: String,
    tool_id: String,
    name: String,
    context: String,
    preview: String,
    summary: String,
    error: String,
    inline_diff: String,
    status: ToolStatus,
    started_at: Instant,
    completed_at: Option<Instant>,
    /// Whether the user has expanded this entry (None = follow default)
    expanded: Option<bool>,
}

impl ToolCallEntry {
    fn new(tool_id: &str, name: &str, context: &str) -> Self {
        Self {
            id: format!("tool-{}", tool_id),
            tool_id: tool_id.to_string(),
            name: name.to_string(),
            context: context.to_string(),
            preview: String::new(),
            summary: String::new(),
            error: String::new(),
            inline_diff: String::new(),
            status: ToolStatus::Running,
            started_at: Instant::now(),
            completed_at: None,
            expanded: None,
        }
    }

    fn elapsed_ms(&self) -> u128 {
        let end = self.completed_at.unwrap_or_else(Instant::now);
        end.duration_since(self.started_at).as_millis()
    }

    fn is_open(&self) -> bool {
        self.expanded.unwrap_or(matches!(
            self.status,
            ToolStatus::Error | ToolStatus::Pending
        ))
    }
}

// role_label() and role_color() are provided by the HistoryCell trait impl above.

// ---------------------------------------------------------------------------
// Configuration model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct TuiConfig {
    model: String,
    max_tokens: u32,
    temperature: Option<f64>,
    system_prompt: String,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 4096,
            temperature: None,
            system_prompt: "You are OpenCode, an AI coding assistant running in the terminal."
                .to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// UI screen enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum UiScreen {
    Chat,
    Config,
    Help,
    Dashboard,
    AICodeTemplate,
    ProjectMemo,
    CommandSnippet,
    UnifiedDiffViewer,
    TaskBoard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ProviderKind {
    Anthropic,
    OpenAiCompat,
}

/// Get available models for the given provider.
fn get_available_models(provider: ProviderKind) -> Vec<&'static str> {
    match provider {
        ProviderKind::Anthropic => vec![
            "claude-sonnet-4-6",
            "claude-opus-4",
            "claude-haiku-3-5",
            "claude-sonnet-4-5",
        ],
        ProviderKind::OpenAiCompat => vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-3.5-turbo",
            "o1-preview",
            "o1-mini",
        ],
    }
}

// ---------------------------------------------------------------------------
// Indicator style (Hermes-inspired)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum IndicatorStyle {
    Ascii,
    Emoji,
    Kaomoji,
    Unicode,
}

impl IndicatorStyle {
    /// Render a frame for the given tick count (monotonic counter).
    fn frame(&self, tick: u64) -> &'static str {
        match self {
            Self::Ascii => match tick % 4 {
                0 => "|",
                1 => "/",
                2 => "-",
                _ => "\\",
            },
            Self::Emoji => match tick % 6 {
                0 => "⚕",
                1 => "🌀",
                2 => "🤔",
                3 => "✨",
                4 => "🍵",
                _ => "🔮",
            },
            Self::Kaomoji => match tick % 6 {
                0 => "(•_•)",
                1 => "(╯°□°)╯",
                2 => "┐(‘～` )┌",
                3 => "ᕙ(⇀‸↼‶)ᕗ",
                4 => "(─‿‿─)",
                _ => "(◕‿◕✿)",
            },
            Self::Unicode => match tick % 4 {
                0 => "⠋",
                1 => "⠙",
                2 => "⠹",
                _ => "⠸",
            },
        }
    }

    /// Label for the /indicator slash subcommand.
    fn label(&self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::Emoji => "emoji",
            Self::Kaomoji => "kaomoji",
            Self::Unicode => "unicode",
        }
    }

    fn variants() -> &'static [IndicatorStyle] {
        &[Self::Ascii, Self::Emoji, Self::Kaomoji, Self::Unicode]
    }
}

// ---------------------------------------------------------------------------
// Theme system (Hermes-inspired)
// ---------------------------------------------------------------------------

/// Named colour palette used across the UI. Each slot maps to a theme-variable
/// name so a future skin loader can replace values by key.
#[derive(Debug, Clone)]
struct ThemeColors {
    // Foreground tones
    pub primary: Color,
    pub accent: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border: Color,
    pub label: Color,

    // Semantic colours
    pub ok: Color,
    pub error: Color,
    pub warn: Color,
    pub info: Color,

    // Status bar
    pub status_bg: Color,
    pub status_fg: Color,
    pub status_busy_bg: Color,
    pub status_demo_bg: Color,

    // Diff
    pub diff_added: Color,
    pub diff_removed: Color,

    // Prompt/input
    pub prompt_label: Color,
    pub shell_dollar: Color,

    // Model indicator bar
    pub model_bar_bg: Color,
}

/// A full theme with colours.
#[derive(Debug, Clone)]
struct Theme {
    pub color: ThemeColors,
    pub name: &'static str,
    pub is_dark: bool,
}

/// Detect whether the terminal is in light mode by checking
/// `COLORFGBG` (popular in rxvt / xterm), `FOREGROUND` / `BACKGROUND`
/// (Windows Terminal / iTerm2), or falling back to the `OSC 4` / `OSC 10`
/// approach.
fn detect_light_mode() -> bool {
    // COLORFGBG: last field after `;` is the background colour index.
    // Light backgrounds are ≥ 8.
    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(bg) = val.rsplit(';').next() {
            if let Ok(n) = bg.parse::<u8>() {
                return n >= 8;
            }
        }
    }
    // Windows Terminal / modern terminals export COLORTERM=truecolor
    // with a separate background hint.  We can't reliably detect here
    // without querying the terminal, so default to dark.
    false
}

impl Theme {
    /// Create the default theme (auto-detect light/dark).
    fn auto() -> Self {
        if detect_light_mode() {
            Self::light()
        } else {
            Self::dark()
        }
    }

    fn dark() -> Self {
        Self {
            name: "dark",
            is_dark: true,
            color: ThemeColors {
                primary: Color::Rgb(187, 154, 247), // purple
                accent: Color::Rgb(255, 159, 67),   // orange
                text: Color::Rgb(220, 220, 220),
                text_muted: Color::Rgb(120, 120, 120),
                border: Color::Rgb(80, 80, 80),
                label: Color::Rgb(100, 180, 255), // blue
                ok: Color::Rgb(80, 200, 120),
                error: Color::Rgb(255, 80, 80),
                warn: Color::Rgb(255, 200, 60),
                info: Color::Rgb(100, 180, 255),
                status_bg: Color::Rgb(40, 40, 60),
                status_fg: Color::Rgb(220, 220, 220),
                status_busy_bg: Color::Rgb(60, 50, 30),
                status_demo_bg: Color::Rgb(50, 40, 40),
                diff_added: Color::Rgb(80, 200, 120),
                diff_removed: Color::Rgb(255, 80, 80),
                prompt_label: Color::Rgb(80, 200, 120),
                shell_dollar: Color::Rgb(80, 200, 120),
                model_bar_bg: Color::Rgb(30, 30, 50),
            },
        }
    }

    fn light() -> Self {
        Self {
            name: "light",
            is_dark: false,
            color: ThemeColors {
                primary: Color::Rgb(120, 80, 200),
                accent: Color::Rgb(200, 100, 30),
                text: Color::Rgb(50, 50, 50),
                text_muted: Color::Rgb(160, 160, 160),
                border: Color::Rgb(200, 200, 200),
                label: Color::Rgb(30, 100, 200),
                ok: Color::Rgb(60, 160, 80),
                error: Color::Rgb(200, 40, 40),
                warn: Color::Rgb(180, 140, 20),
                info: Color::Rgb(30, 100, 200),
                status_bg: Color::Rgb(230, 230, 240),
                status_fg: Color::Rgb(50, 50, 50),
                status_busy_bg: Color::Rgb(240, 230, 200),
                status_demo_bg: Color::Rgb(240, 220, 220),
                diff_added: Color::Rgb(60, 160, 80),
                diff_removed: Color::Rgb(200, 40, 40),
                prompt_label: Color::Rgb(60, 160, 80),
                shell_dollar: Color::Rgb(60, 160, 80),
                model_bar_bg: Color::Rgb(220, 220, 235),
            },
        }
    }

    // --- 8 additional themes ---

    fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox-dark",
            is_dark: true,
            color: ThemeColors {
                primary: Color::Rgb(214, 153, 97),     // orange
                accent: Color::Rgb(250, 189, 47),      // yellow
                text: Color::Rgb(235, 219, 178),       // fg
                text_muted: Color::Rgb(146, 131, 116), // grey
                border: Color::Rgb(80, 73, 69),        // bg2
                label: Color::Rgb(131, 165, 152),      // aqua
                ok: Color::Rgb(142, 192, 124),         // green
                error: Color::Rgb(251, 73, 51),        // red
                warn: Color::Rgb(250, 189, 47),        // yellow
                info: Color::Rgb(131, 165, 152),       // aqua
                status_bg: Color::Rgb(50, 48, 47),     // bg1
                status_fg: Color::Rgb(235, 219, 178),
                status_busy_bg: Color::Rgb(80, 73, 69),
                status_demo_bg: Color::Rgb(60, 56, 54),
                diff_added: Color::Rgb(142, 192, 124),
                diff_removed: Color::Rgb(251, 73, 51),
                prompt_label: Color::Rgb(142, 192, 124),
                shell_dollar: Color::Rgb(142, 192, 124),
                model_bar_bg: Color::Rgb(40, 40, 39),
            },
        }
    }

    fn gruvbox_light() -> Self {
        Self {
            name: "gruvbox-light",
            is_dark: false,
            color: ThemeColors {
                primary: Color::Rgb(135, 74, 23),
                accent: Color::Rgb(181, 118, 20),
                text: Color::Rgb(60, 56, 54),
                text_muted: Color::Rgb(146, 131, 116),
                border: Color::Rgb(189, 174, 147),
                label: Color::Rgb(66, 123, 88),
                ok: Color::Rgb(66, 123, 88),
                error: Color::Rgb(204, 36, 29),
                warn: Color::Rgb(181, 118, 20),
                info: Color::Rgb(66, 123, 88),
                status_bg: Color::Rgb(235, 219, 178),
                status_fg: Color::Rgb(60, 56, 54),
                status_busy_bg: Color::Rgb(242, 229, 188),
                status_demo_bg: Color::Rgb(235, 219, 178),
                diff_added: Color::Rgb(66, 123, 88),
                diff_removed: Color::Rgb(204, 36, 29),
                prompt_label: Color::Rgb(66, 123, 88),
                shell_dollar: Color::Rgb(66, 123, 88),
                model_bar_bg: Color::Rgb(213, 196, 161),
            },
        }
    }

    fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark",
            is_dark: true,
            color: ThemeColors {
                primary: Color::Rgb(38, 139, 210),    // blue
                accent: Color::Rgb(211, 54, 130),     // magenta
                text: Color::Rgb(147, 161, 161),      // base1
                text_muted: Color::Rgb(88, 110, 117), // base01
                border: Color::Rgb(7, 54, 66),        // base02
                label: Color::Rgb(42, 161, 152),      // cyan
                ok: Color::Rgb(133, 153, 0),          // green
                error: Color::Rgb(220, 50, 47),       // red
                warn: Color::Rgb(181, 137, 0),        // yellow
                info: Color::Rgb(38, 139, 210),       // blue
                status_bg: Color::Rgb(0, 43, 54),     // base03
                status_fg: Color::Rgb(147, 161, 161),
                status_busy_bg: Color::Rgb(7, 54, 66),
                status_demo_bg: Color::Rgb(7, 54, 66),
                diff_added: Color::Rgb(133, 153, 0),
                diff_removed: Color::Rgb(220, 50, 47),
                prompt_label: Color::Rgb(133, 153, 0),
                shell_dollar: Color::Rgb(133, 153, 0),
                model_bar_bg: Color::Rgb(0, 35, 44),
            },
        }
    }

    fn solarized_light() -> Self {
        Self {
            name: "solarized-light",
            is_dark: false,
            color: ThemeColors {
                primary: Color::Rgb(38, 139, 210),
                accent: Color::Rgb(211, 54, 130),
                text: Color::Rgb(88, 110, 117),
                text_muted: Color::Rgb(147, 161, 161),
                border: Color::Rgb(207, 213, 206),
                label: Color::Rgb(42, 161, 152),
                ok: Color::Rgb(133, 153, 0),
                error: Color::Rgb(220, 50, 47),
                warn: Color::Rgb(181, 137, 0),
                info: Color::Rgb(38, 139, 210),
                status_bg: Color::Rgb(238, 232, 213), // base2
                status_fg: Color::Rgb(88, 110, 117),
                status_busy_bg: Color::Rgb(230, 223, 203),
                status_demo_bg: Color::Rgb(238, 232, 213),
                diff_added: Color::Rgb(133, 153, 0),
                diff_removed: Color::Rgb(220, 50, 47),
                prompt_label: Color::Rgb(133, 153, 0),
                shell_dollar: Color::Rgb(133, 153, 0),
                model_bar_bg: Color::Rgb(225, 218, 198),
            },
        }
    }

    fn nord() -> Self {
        Self {
            name: "nord",
            is_dark: true,
            color: ThemeColors {
                primary: Color::Rgb(136, 192, 208),  // frost - nord8
                accent: Color::Rgb(180, 142, 173),   // aurora - purple
                text: Color::Rgb(216, 222, 233),     // snow storm - nord5
                text_muted: Color::Rgb(76, 86, 106), // polar night - nord3
                border: Color::Rgb(59, 66, 82),      // polar night - nord2
                label: Color::Rgb(129, 161, 193),    // frost - nord9
                ok: Color::Rgb(163, 190, 140),       // aurora - green
                error: Color::Rgb(191, 97, 106),     // aurora - red
                warn: Color::Rgb(235, 203, 139),     // aurora - yellow
                info: Color::Rgb(136, 192, 208),     // frost - nord8
                status_bg: Color::Rgb(46, 52, 64),   // polar night - nord1
                status_fg: Color::Rgb(216, 222, 233),
                status_busy_bg: Color::Rgb(59, 66, 82),
                status_demo_bg: Color::Rgb(59, 66, 82),
                diff_added: Color::Rgb(163, 190, 140),
                diff_removed: Color::Rgb(191, 97, 106),
                prompt_label: Color::Rgb(163, 190, 140),
                shell_dollar: Color::Rgb(163, 190, 140),
                model_bar_bg: Color::Rgb(36, 42, 54),
            },
        }
    }

    fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha",
            is_dark: true,
            color: ThemeColors {
                primary: Color::Rgb(166, 227, 161),    // green
                accent: Color::Rgb(249, 226, 175),     // yellow
                text: Color::Rgb(205, 214, 244),       // text
                text_muted: Color::Rgb(108, 112, 134), // overlay1
                border: Color::Rgb(69, 71, 90),        // surface2
                label: Color::Rgb(137, 180, 250),      // blue
                ok: Color::Rgb(166, 227, 161),         // green
                error: Color::Rgb(243, 139, 168),      // red
                warn: Color::Rgb(249, 226, 175),       // yellow
                info: Color::Rgb(137, 180, 250),       // blue
                status_bg: Color::Rgb(30, 30, 46),     // base
                status_fg: Color::Rgb(205, 214, 244),
                status_busy_bg: Color::Rgb(49, 50, 68),
                status_demo_bg: Color::Rgb(49, 50, 68),
                diff_added: Color::Rgb(166, 227, 161),
                diff_removed: Color::Rgb(243, 139, 168),
                prompt_label: Color::Rgb(166, 227, 161),
                shell_dollar: Color::Rgb(166, 227, 161),
                model_bar_bg: Color::Rgb(24, 24, 37),
            },
        }
    }

    fn catppuccin_latte() -> Self {
        Self {
            name: "catppuccin-latte",
            is_dark: false,
            color: ThemeColors {
                primary: Color::Rgb(64, 160, 43),      // green
                accent: Color::Rgb(223, 142, 29),      // yellow
                text: Color::Rgb(76, 79, 105),         // text
                text_muted: Color::Rgb(156, 160, 176), // overlay1
                border: Color::Rgb(188, 192, 204),     // surface2
                label: Color::Rgb(30, 102, 245),       // blue
                ok: Color::Rgb(64, 160, 43),           // green
                error: Color::Rgb(210, 15, 57),        // red
                warn: Color::Rgb(223, 142, 29),        // yellow
                info: Color::Rgb(30, 102, 245),        // blue
                status_bg: Color::Rgb(239, 241, 245),  // base
                status_fg: Color::Rgb(76, 79, 105),
                status_busy_bg: Color::Rgb(220, 224, 232),
                status_demo_bg: Color::Rgb(220, 224, 232),
                diff_added: Color::Rgb(64, 160, 43),
                diff_removed: Color::Rgb(210, 15, 57),
                prompt_label: Color::Rgb(64, 160, 43),
                shell_dollar: Color::Rgb(64, 160, 43),
                model_bar_bg: Color::Rgb(228, 231, 237),
            },
        }
    }

    fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night",
            is_dark: true,
            color: ThemeColors {
                primary: Color::Rgb(125, 207, 255),  // blue
                accent: Color::Rgb(187, 154, 247),   // purple (mauve)
                text: Color::Rgb(192, 202, 245),     // fg
                text_muted: Color::Rgb(86, 95, 137), // comment
                border: Color::Rgb(54, 59, 83),      // bg_highlight
                label: Color::Rgb(122, 162, 247),    // blue (darker)
                ok: Color::Rgb(158, 206, 106),       // green
                error: Color::Rgb(247, 118, 142),    // red
                warn: Color::Rgb(224, 175, 104),     // yellow
                info: Color::Rgb(125, 207, 255),     // cyan
                status_bg: Color::Rgb(26, 27, 38),   // bg
                status_fg: Color::Rgb(192, 202, 245),
                status_busy_bg: Color::Rgb(41, 45, 65),
                status_demo_bg: Color::Rgb(41, 45, 65),
                diff_added: Color::Rgb(158, 206, 106),
                diff_removed: Color::Rgb(247, 118, 142),
                prompt_label: Color::Rgb(158, 206, 106),
                shell_dollar: Color::Rgb(158, 206, 106),
                model_bar_bg: Color::Rgb(22, 23, 33),
            },
        }
    }

    /// Get all available theme names.
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "dark",
            "light",
            "gruvbox-dark",
            "gruvbox-light",
            "solarized-dark",
            "solarized-light",
            "nord",
            "catppuccin-mocha",
            "catppuccin-latte",
            "tokyo-night",
        ]
    }

    /// Create a theme by name, falling back to auto-detect.
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dark" => Self::dark(),
            "light" => Self::light(),
            "gruvbox-dark" | "gruvbox" => Self::gruvbox_dark(),
            "gruvbox-light" => Self::gruvbox_light(),
            "solarized-dark" | "solarized" => Self::solarized_dark(),
            "solarized-light" => Self::solarized_light(),
            "nord" => Self::nord(),
            "catppuccin-mocha" | "catppuccin" => Self::catppuccin_mocha(),
            "catppuccin-latte" => Self::catppuccin_latte(),
            "tokyo-night" | "tokyonight" => Self::tokyo_night(),
            _ => Self::auto(),
        }
    }
}

// Status bar segment data — the top-level renderer draws segments left-to-right.
struct StatusBar {
    /// Current tick for animation (incremented each render frame).
    pub tick: u64,
    /// Elapsed seconds since app start or last reset.
    pub elapsed_secs: u64,
    /// Number of messages in transcript.
    pub msg_count: usize,
    /// Indicator style.
    pub indicator: IndicatorStyle,
    /// Whether an LLM call is in flight.
    pub busy: bool,
    /// Sub-agent count (for Hermes-style tree indicator).
    pub subagent_count: usize,
    /// Custom status message (shown in center).
    pub message: String,
    /// Token usage (used / limit) for color-coded display.
    pub token_used: usize,
    pub token_limit: usize,
    /// Current input mode label (e.g., "NORMAL", "INSERT", "MULTILINE").
    pub mode_label: String,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            tick: 0,
            elapsed_secs: 0,
            msg_count: 0,
            indicator: IndicatorStyle::Kaomoji,
            busy: false,
            subagent_count: 0,
            message: "Ready".into(),
            token_used: 0,
            token_limit: 4000,
            mode_label: "NORMAL".into(),
        }
    }
}

impl StatusBar {
    /// Left-hand side: indicator + elapsed + model info.
    fn left_side(&self, model: &str, provider: &str) -> String {
        let glyph = if self.busy {
            self.indicator.frame(self.tick)
        } else {
            match self.indicator {
                IndicatorStyle::Kaomoji => "(◕‿◕✿)",
                IndicatorStyle::Emoji => "✨",
                IndicatorStyle::Ascii => ">",
                IndicatorStyle::Unicode => "⠿",
            }
        };
        let elapsed = format_elapsed(self.elapsed_secs);
        format!(" {}  {}  {}  {} ", glyph, elapsed, model, provider)
    }

    /// Center: mode label + status message.
    fn center_side(&self) -> String {
        if self.message.is_empty() {
            format!(" [{}] ", self.mode_label)
        } else {
            format!(" [{}] {} ", self.mode_label, self.message)
        }
    }

    /// Right-hand side: message count, tokens, help hints.
    fn right_side(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if self.subagent_count > 0 {
            parts.push(format!("⊞{}", self.subagent_count));
        }

        if self.msg_count > 0 {
            parts.push(format!("{}+", self.msg_count));
        }

        // Token usage indicator (color-coded via percentage)
        if self.token_used > 0 {
            let pct = self.token_used as f64 / self.token_limit as f64;
            let symbol = if pct > 0.9 {
                "🔴"
            } else if pct > 0.7 {
                "🟡"
            } else {
                "🟢"
            };
            parts.push(format!("{}t{}", symbol, self.token_used));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!(" {} ", parts.join(" · "))
        }
    }
}

fn format_elapsed(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Generate shimmer-styled spans for a loading indicator.
/// The highlight sweeps left-to-right across the text, creating a
/// moving glow effect synchronized to the frame tick.
fn shimmer_spans(text: &str, tick: u64, base_color: Color, highlight: Color) -> Vec<Span<'static>> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if len == 0 {
        return vec![];
    }
    // Shimmer position cycles every 24 frames
    let shimmer_pos = (tick / 3) % (len as u64 + 8);
    let shimmer_width: i64 = 5;

    chars
        .iter()
        .enumerate()
        .map(|(i, &ch)| {
            let dist = (i as i64 - shimmer_pos as i64).unsigned_abs();
            let color = if dist < shimmer_width as u64 {
                // Near shimmer center: bright
                highlight
            } else if dist < shimmer_width as u64 + 3 {
                // Transition zone: base color
                base_color
            } else {
                // Far from shimmer: dim
                Color::DarkGray
            };
            Span::styled(ch.to_string(), Style::default().fg(color))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// Fuzzy search engine (subsequence scoring)
// ---------------------------------------------------------------------------

/// Score a fuzzy match between `pattern` and `text`.
/// Returns `Some(score)` if all chars in pattern appear in text in order,
/// or `None` if no match. Higher score = better match.
fn fuzzy_score(pattern: &str, text: &str) -> Option<i64> {
    if pattern.is_empty() {
        return Some(0);
    }
    let pattern_lower: Vec<char> = pattern.to_lowercase().chars().collect();
    let text_lower: Vec<char> = text.to_lowercase().chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    let mut score: i64 = 0;
    let mut pat_idx = 0;
    let mut prev_match_idx: Option<usize> = None;
    let mut consecutive_bonus: i64 = 0;

    for (i, &tc) in text_lower.iter().enumerate() {
        if pat_idx >= pattern_lower.len() {
            break;
        }
        if tc == pattern_lower[pat_idx] {
            // Base score for matching
            score += 10;

            // Bonus for consecutive matches
            if let Some(prev) = prev_match_idx {
                if i == prev + 1 {
                    consecutive_bonus += 5;
                    score += consecutive_bonus;
                } else {
                    consecutive_bonus = 0;
                }
            }

            // Bonus for matching at word boundary
            if i == 0
                || text_chars
                    .get(i - 1)
                    .is_some_and(|c| c.is_whitespace() || *c == '_' || *c == '-')
            {
                score += 15;
            }

            // Bonus for matching uppercase (camelCase)
            if text_chars.get(i).is_some_and(|c| c.is_uppercase()) {
                score += 5;
            }

            // Bonus for exact position match (pattern start == text start)
            if i == 0 {
                score += 20;
            }

            prev_match_idx = Some(i);
            pat_idx += 1;
        }
    }

    if pat_idx == pattern_lower.len() {
        // Penalty for longer texts (prefer shorter matches)
        score -= (text.len() as i64 - pattern.len() as i64) / 2;
        Some(score)
    } else {
        None
    }
}

/// Sort items by fuzzy match score (best first).
fn fuzzy_sort(pattern: &str, items: &[String]) -> Vec<(usize, i64)> {
    let mut scored: Vec<(usize, i64)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, text)| fuzzy_score(pattern, text).map(|s| (i, s)))
        .collect();
    scored.sort_by_key(|a| std::cmp::Reverse(a.1));
    scored
}

// ---------------------------------------------------------------------------
// Slash command autocomplete (Hermes/OpenCode-inspired)
// ---------------------------------------------------------------------------

/// Mention types for @-prefixed completions.
#[derive(Debug, Clone, PartialEq)]
enum MentionType {
    File,
    Code,
    History,
    Model,
}

impl MentionType {
    fn prefix(&self) -> &'static str {
        match self {
            MentionType::File => "@file",
            MentionType::Code => "@code",
            MentionType::History => "@history",
            MentionType::Model => "@model",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            MentionType::File => "Attach file content",
            MentionType::Code => "Insert code snippet",
            MentionType::History => "Search history",
            MentionType::Model => "Switch model",
        }
    }
}

/// Autocomplete state for `/`-prefixed commands and `@`-prefixed mentions.
#[derive(Debug, Clone, Default)]
struct SlashAutocomplete {
    /// Whether the popup should be visible.
    visible: bool,
    /// Filtered candidate commands matching the current input.
    candidates: Vec<usize>,
    /// Index of the currently-highlighted candidate.
    selected: usize,
    /// Whether we are in @-mention mode.
    mention_mode: bool,
    /// Filtered mention candidates.
    mention_candidates: Vec<MentionType>,
    /// Selected mention index.
    mention_selected: usize,
}

impl SlashAutocomplete {
    /// Update candidate list based on the current input.
    /// Returns `true` if the popup is visible.
    fn update(&mut self, input: &str) -> bool {
        // Check for @-mention mode
        if let Some(at_pos) = input.rfind('@') {
            // Only trigger if @ is at start or preceded by whitespace
            if at_pos == 0
                || input
                    .as_bytes()
                    .get(at_pos - 1)
                    .is_some_and(|b| b.is_ascii_whitespace())
            {
                let after_at = &input[at_pos + 1..];
                // Only trigger if no whitespace after @
                if !after_at.contains(char::is_whitespace) {
                    self.mention_mode = true;
                    let prefix = after_at.to_lowercase();
                    self.mention_candidates = vec![
                        MentionType::File,
                        MentionType::Code,
                        MentionType::History,
                        MentionType::Model,
                    ]
                    .into_iter()
                    .filter(|m| prefix.is_empty() || m.prefix()[1..].starts_with(&prefix))
                    .collect();
                    self.visible = !self.mention_candidates.is_empty();
                    if self.mention_selected >= self.mention_candidates.len() {
                        self.mention_selected = 0;
                    }
                    return self.visible;
                }
            }
        }

        // Slash command mode
        self.mention_mode = false;
        self.mention_candidates.clear();
        self.mention_selected = 0;

        if !input.starts_with('/') {
            self.visible = false;
            self.candidates.clear();
            self.selected = 0;
            return false;
        }
        // Only show popup when the slash is at start and no whitespace yet
        // (so we don't suggest inside a regular message body).
        if input.contains(char::is_whitespace) {
            self.visible = false;
            self.candidates.clear();
            self.selected = 0;
            return false;
        }
        let prefix = input[1..].to_lowercase();
        self.candidates = BUILTIN_COMMANDS
            .iter()
            .enumerate()
            .filter(|(_, c)| prefix.is_empty() || c.name.starts_with(&prefix))
            .map(|(i, _)| i)
            .collect();
        self.visible = !self.candidates.is_empty();
        if self.selected >= self.candidates.len() {
            self.selected = 0;
        }
        self.visible
    }

    /// Move selection to the next candidate.
    fn next(&mut self) {
        if self.mention_mode {
            if !self.mention_candidates.is_empty() {
                self.mention_selected = (self.mention_selected + 1) % self.mention_candidates.len();
            }
        } else if !self.candidates.is_empty() {
            self.selected = (self.selected + 1) % self.candidates.len();
        }
    }

    /// Move selection to the previous candidate.
    fn prev(&mut self) {
        if self.mention_mode {
            if !self.mention_candidates.is_empty() {
                self.mention_selected = if self.mention_selected == 0 {
                    self.mention_candidates.len() - 1
                } else {
                    self.mention_selected - 1
                };
            }
        } else if !self.candidates.is_empty() {
            self.selected = if self.selected == 0 {
                self.candidates.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Get the currently-highlighted command (or `None` if empty).
    fn selected_command(&self) -> Option<&'static slash_commands::SlashCommand> {
        self.candidates
            .get(self.selected)
            .map(|&i| &BUILTIN_COMMANDS[i])
    }

    /// Get the currently-highlighted mention (or `None` if empty).
    fn selected_mention(&self) -> Option<&MentionType> {
        self.mention_candidates.get(self.mention_selected)
    }
}

// ---------------------------------------------------------------------------
// Lightweight markdown rendering (code blocks / headings)
// ---------------------------------------------------------------------------

/// Render markdown using pulldown_cmark-based renderer (non-streaming).
///
/// # Arguments
/// * `text` - The markdown text to render.
/// * `t` - The theme colors to use.
fn render_markdown_simple<'a>(text: &'a str, t: &'a ThemeColors) -> Vec<Line<'a>> {
    let md_theme = MarkdownTheme {
        border: t.border,
        primary: t.primary,
        accent: t.accent,
        text: t.text,
        text_muted: t.text_muted,
        status_bg: t.status_bg,
        ok: t.ok,
        error: t.error,
    };
    let renderer = MarkdownRenderer::new(&md_theme);
    renderer.render(text)
}

/// Render markdown using pulldown_cmark-based renderer (non-streaming).
fn render_markdown<'a>(text: &'a str, t: &'a ThemeColors) -> Vec<Line<'a>> {
    let md_theme = MarkdownTheme {
        border: t.border,
        primary: t.primary,
        accent: t.accent,
        text: t.text,
        text_muted: t.text_muted,
        status_bg: t.status_bg,
        ok: t.ok,
        error: t.error,
    };
    let renderer = MarkdownRenderer::new(&md_theme);
    renderer.render(text)
}

/// Split a line into spans where `` `...` `` segments are styled differently.
// `inline_code_spans` removed — pulldown_cmark handles inline code natively.
// ---------------------------------------------------------------------------
// AI mode
// ---------------------------------------------------------------------------

/// AI operation mode that controls how the assistant behaves and what UI is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiMode {
    /// Standard conversational chat (default).
    Chat,
    /// Generate a plan first, then execute after approval.
    Plan,
    /// Autonomous tool execution with minimal prompting.
    Agent,
    /// Code review mode — analyse diffs and suggest fixes.
    Review,
    /// Focus on a specific file or directory scope.
    Focused,
    /// Inline edit mode (Helix-like compositor integration).
    Edit,
    /// Fully autonomous — picks goals, plans, executes, all without user input.
    Autopilot,
}

impl AiMode {
    fn label(&self) -> &'static str {
        match self {
            Self::Chat => "Chat",
            Self::Plan => "Plan",
            Self::Agent => "Agent",
            Self::Review => "Review",
            Self::Focused => "Focused",
            Self::Edit => "Edit",
            Self::Autopilot => "Autopilot",
        }
    }
    fn icon(&self) -> &'static str {
        match self {
            Self::Chat => "💬",
            Self::Plan => "📝",
            Self::Agent => "🛠️",
            Self::Review => "🔍",
            Self::Focused => "🎯",
            Self::Edit => "✏️",
            Self::Autopilot => "🤖",
        }
    }
    fn all() -> &'static [Self] {
        &[
            Self::Chat,
            Self::Plan,
            Self::Agent,
            Self::Review,
            Self::Focused,
            Self::Edit,
            Self::Autopilot,
        ]
    }
    fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|m| m == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
    fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|m| m == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }
}

// ---------------------------------------------------------------------------
// The App
// ---------------------------------------------------------------------------

/// AI code generation template
struct AICodeTemplate {
    visible: bool,
    templates: Vec<CodeTemplate>,
    selected: usize,
    search_query: String,
}

struct CodeTemplate {
    name: String,
    description: String,
    language: String,
    code: String,
    tags: Vec<String>,
}

impl AICodeTemplate {
    fn new() -> Self {
        Self {
            visible: false,
            templates: vec![
                CodeTemplate {
                    name: "REST API Handler".to_string(),
                    description: "Basic REST API endpoint".to_string(),
                    language: "rust".to_string(),
                    code: "async fn handler(req: HttpRequest) -> impl Responder { ... }"
                        .to_string(),
                    tags: vec!["api".to_string(), "web".to_string()],
                },
                CodeTemplate {
                    name: "CLI Tool".to_string(),
                    description: "Command-line tool with args".to_string(),
                    language: "rust".to_string(),
                    code: "fn main() { let args: Vec<String> = std::env::args().collect(); ... }"
                        .to_string(),
                    tags: vec!["cli".to_string()],
                },
                CodeTemplate {
                    name: "Unit Test".to_string(),
                    description: "Basic unit test template".to_string(),
                    language: "rust".to_string(),
                    code: "#[test]\nfn test_example() { assert_eq!(1+1, 2); }".to_string(),
                    tags: vec!["test".to_string()],
                },
            ],
            selected: 0,
            search_query: String::new(),
        }
    }
    fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// Project memo pad
struct ProjectMemo {
    visible: bool,
    memos: Vec<MemoEntry>,
    selected: usize,
    editing: bool,
    edit_buffer: String,
}

struct MemoEntry {
    title: String,
    content: String,
    created_at: String,
    tags: Vec<String>,
}

impl ProjectMemo {
    fn new() -> Self {
        Self {
            visible: false,
            memos: vec![MemoEntry {
                title: "Project Notes".to_string(),
                content: "Remember to update dependencies".to_string(),
                created_at: "2026-07-18".to_string(),
                tags: vec!["todo".to_string()],
            }],
            selected: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }
    fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// Command snippet library
struct CommandSnippet {
    visible: bool,
    snippets: Vec<SnippetEntry>,
    selected: usize,
    search_query: String,
}

struct SnippetEntry {
    name: String,
    command: String,
    description: String,
    category: String,
    usage_count: usize,
}

impl CommandSnippet {
    fn new() -> Self {
        Self {
            visible: false,
            snippets: vec![
                SnippetEntry {
                    name: "Build".to_string(),
                    command: "cargo build".to_string(),
                    description: "Build the project".to_string(),
                    category: "build".to_string(),
                    usage_count: 42,
                },
                SnippetEntry {
                    name: "Test".to_string(),
                    command: "cargo test".to_string(),
                    description: "Run tests".to_string(),
                    category: "test".to_string(),
                    usage_count: 35,
                },
                SnippetEntry {
                    name: "Lint".to_string(),
                    command: "cargo clippy".to_string(),
                    description: "Run linter".to_string(),
                    category: "lint".to_string(),
                    usage_count: 28,
                },
            ],
            selected: 0,
            search_query: String::new(),
        }
    }
    fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// Unified diff viewer
struct UnifiedDiffViewer {
    visible: bool,
    diff_content: Vec<DiffLine>,
    scroll: usize,
    file_path: String,
}

struct DiffLine {
    line_type: DiffLineType,
    content: String,
    line_old: Option<usize>,
    line_new: Option<usize>,
}

enum DiffLineType {
    Context,
    Added,
    Removed,
    Header,
}

impl UnifiedDiffViewer {
    fn new() -> Self {
        Self {
            visible: false,
            diff_content: vec![
                DiffLine {
                    line_type: DiffLineType::Header,
                    content: "@@ -1,5 +1,7 @@".to_string(),
                    line_old: None,
                    line_new: None,
                },
                DiffLine {
                    line_type: DiffLineType::Context,
                    content: " fn main() {".to_string(),
                    line_old: Some(1),
                    line_new: Some(1),
                },
                DiffLine {
                    line_type: DiffLineType::Removed,
                    content: "-    println!(\"Hello\");".to_string(),
                    line_old: Some(2),
                    line_new: None,
                },
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "+    println!(\"Hello, World!\");".to_string(),
                    line_old: None,
                    line_new: Some(2),
                },
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "+    // Added greeting".to_string(),
                    line_old: None,
                    line_new: Some(3),
                },
                DiffLine {
                    line_type: DiffLineType::Context,
                    content: " }".to_string(),
                    line_old: Some(3),
                    line_new: Some(4),
                },
            ],
            scroll: 0,
            file_path: "src/main.rs".to_string(),
        }
    }
    fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// Task management board (Kanban-style)
struct TaskBoard {
    visible: bool,
    columns: Vec<TaskColumn>,
    selected_col: usize,
    selected_task: usize,
}

struct TaskColumn {
    title: String,
    tasks: Vec<TaskItem>,
}

struct TaskItem {
    title: String,
    description: String,
    priority: TaskPriority,
    tags: Vec<String>,
}

enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskBoard {
    fn new() -> Self {
        Self {
            visible: false,
            columns: vec![
                TaskColumn {
                    title: "To Do".to_string(),
                    tasks: vec![
                        TaskItem {
                            title: "Implement feature X".to_string(),
                            description: "Add new feature".to_string(),
                            priority: TaskPriority::High,
                            tags: vec!["feature".to_string()],
                        },
                        TaskItem {
                            title: "Fix bug Y".to_string(),
                            description: "Fix critical bug".to_string(),
                            priority: TaskPriority::Critical,
                            tags: vec!["bug".to_string()],
                        },
                    ],
                },
                TaskColumn {
                    title: "In Progress".to_string(),
                    tasks: vec![TaskItem {
                        title: "Review PR #42".to_string(),
                        description: "Code review".to_string(),
                        priority: TaskPriority::Medium,
                        tags: vec!["review".to_string()],
                    }],
                },
                TaskColumn {
                    title: "Done".to_string(),
                    tasks: vec![TaskItem {
                        title: "Setup CI/CD".to_string(),
                        description: "Configure pipeline".to_string(),
                        priority: TaskPriority::Low,
                        tags: vec!["devops".to_string()],
                    }],
                },
            ],
            selected_col: 0,
            selected_task: 0,
        }
    }
    fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

// ---------------------------------------------------------------------------
// Change Detection (inspired by FerroCopy's change_detection.rs)
// ---------------------------------------------------------------------------

/// Lightweight change detection using hashing.
/// Tracks named values and reports when they change.
struct ChangeDetector {
    hashes: std::collections::HashMap<String, u64>,
}

impl ChangeDetector {
    fn new() -> Self {
        Self {
            hashes: std::collections::HashMap::new(),
        }
    }

    /// Returns true if the value has changed since last check.
    fn changed<T: Hash>(&mut self, name: &str, value: &T) -> bool {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();
        match self.hashes.get(name) {
            Some(&prev) if prev == hash => false,
            _ => {
                self.hashes.insert(name.to_string(), hash);
                true
            }
        }
    }

    /// Returns true if a list has changed (length + last item hash).
    fn list_changed<T: Hash>(&mut self, name: &str, items: &[T]) -> bool {
        let key = format!("{}_{}", name, items.len());
        self.changed(&key, &items.len())
    }

    /// Returns true if a count-based value changed.
    fn count_changed(&mut self, name: &str, count: usize) -> bool {
        self.changed(name, &count)
    }

    /// Force mark as changed.
    fn mark_changed(&mut self, name: &str) {
        self.hashes.remove(name);
    }

    /// Reset all tracking.
    fn reset(&mut self) {
        self.hashes.clear();
    }
}

// ---------------------------------------------------------------------------
// Crash Reporter (inspired by FerroCopy's crash_reporter.rs)
// ---------------------------------------------------------------------------

/// Minimal crash reporter that writes a dump file on panic.
struct CrashReporter {
    dump_dir: PathBuf,
}

impl CrashReporter {
    fn new(dump_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dump_dir);
        Self { dump_dir }
    }

    /// Install a panic hook that writes crash dumps.
    fn install(&self) {
        let dump_dir = self.dump_dir.clone();
        panic::set_hook(Box::new(move |info| {
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");

            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Box<dyn Any>".to_string()
            };

            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "<unknown>".to_string());

            let backtrace = std::backtrace::Backtrace::force_capture();

            let dump = format!(
                "OpenCode Crash Report\n\
                 =====================\n\
                 Thread: {}\n\
                 Panic: {}\n\
                 Location: {}\n\
                 Backtrace:\n{}\n",
                thread_name, payload, location, backtrace
            );

            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let filename = format!("crash_{}.txt", timestamp);
            let path = dump_dir.join(&filename);
            let _ = fs::write(&path, &dump);

            eprintln!("\n💥 OpenCode crashed! Crash dump: {}", path.display());
        }));
    }
}

// ---------------------------------------------------------------------------
// Signal Handler (inspired by FerroCopy's signal.rs)
// ---------------------------------------------------------------------------

/// Graceful shutdown signal.
static SHUTDOWN_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn install_signal_handler() {
    let _ = ctrlc::set_handler(|| {
        SHUTDOWN_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
    });
}

fn is_shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(std::sync::atomic::Ordering::SeqCst)
}

// ---------------------------------------------------------------------------
// Fanout Broadcasting (inspired by Fanout/Subscription pattern)
// ---------------------------------------------------------------------------

/// Lightweight fanout: broadcasts progress updates to multiple subscribers.
struct ProgressFanout {
    senders: Vec<tokio::sync::mpsc::Sender<String>>,
}

impl ProgressFanout {
    fn new() -> Self {
        Self {
            senders: Vec::new(),
        }
    }

    fn subscribe(&mut self, buffer: usize) -> tokio::sync::mpsc::Receiver<String> {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer);
        self.senders.push(tx);
        rx
    }

    async fn broadcast(&self, msg: &str) {
        for tx in &self.senders {
            let _ = tx.try_send(msg.to_string());
        }
    }
}

// ---------------------------------------------------------------------------
// Telemetry (inspired by FerroCopy's telemetry.rs)
// ---------------------------------------------------------------------------

/// Lightweight metrics collection.
struct Telemetry {
    llm_calls_total: std::sync::atomic::AtomicU64,
    llm_errors_total: std::sync::atomic::AtomicU64,
    llm_tokens_input: std::sync::atomic::AtomicU64,
    llm_tokens_output: std::sync::atomic::AtomicU64,
    ui_frames_rendered: std::sync::atomic::AtomicU64,
    session_start: Instant,
}

impl Telemetry {
    fn new() -> Self {
        Self {
            llm_calls_total: std::sync::atomic::AtomicU64::new(0),
            llm_errors_total: std::sync::atomic::AtomicU64::new(0),
            llm_tokens_input: std::sync::atomic::AtomicU64::new(0),
            llm_tokens_output: std::sync::atomic::AtomicU64::new(0),
            ui_frames_rendered: std::sync::atomic::AtomicU64::new(0),
            session_start: Instant::now(),
        }
    }

    fn record_llm_call(&self) {
        self.llm_calls_total
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_llm_error(&self) {
        self.llm_errors_total
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_tokens(&self, input: u64, output: u64) {
        self.llm_tokens_input
            .fetch_add(input, std::sync::atomic::Ordering::Relaxed);
        self.llm_tokens_output
            .fetch_add(output, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_frame(&self) {
        self.ui_frames_rendered
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            llm_calls: self
                .llm_calls_total
                .load(std::sync::atomic::Ordering::Relaxed),
            llm_errors: self
                .llm_errors_total
                .load(std::sync::atomic::Ordering::Relaxed),
            tokens_input: self
                .llm_tokens_input
                .load(std::sync::atomic::Ordering::Relaxed),
            tokens_output: self
                .llm_tokens_output
                .load(std::sync::atomic::Ordering::Relaxed),
            frames: self
                .ui_frames_rendered
                .load(std::sync::atomic::Ordering::Relaxed),
            uptime_secs: self.session_start.elapsed().as_secs(),
        }
    }
}

struct TelemetrySnapshot {
    llm_calls: u64,
    llm_errors: u64,
    tokens_input: u64,
    tokens_output: u64,
    frames: u64,
    uptime_secs: u64,
}

struct App {
    /// Conversation history (display messages)
    messages: Vec<ChatMsg>,
    /// Input buffer for user prompt
    input: String,
    /// Cursor position in input
    cursor: usize,
    /// Current screen
    screen: UiScreen,
    /// Status line
    status: String,
    /// Rich status bar (Supersedes `status` for Chat screen)
    status_bar: StatusBar,
    /// App start timestamp for elapsed time
    started_at: Instant,
    /// Scroll offset (reverse: 0 = newest)
    scroll: usize,
    /// Whether the assistant is currently responding
    is_streaming: bool,
    /// Accumulated streaming text (incremental delta)
    streaming_text: String,
    /// Whether running in demo mode (no API key)
    is_demo: bool,
    /// Configuration
    config: TuiConfig,
    /// Active provider kind
    active_provider: ProviderKind,
    /// Theme (Hermes-inspired)
    theme: Theme,
    theme_name: String,
    /// Config editing state
    config_field: usize,
    config_edit: String,
    /// Slash command autocomplete popup
    autocomplete: SlashAutocomplete,
    slash_dispatcher: SlashCommandDispatcher,
    /// Dashboard navigation selection index
    dash_selection: usize,
    /// Dashboard active tab
    dash_tab: usize,
    /// Tool call entries (Hermes-inspired collapsible)
    tool_calls: Vec<ToolCallEntry>,
    /// Whether the tool call panel is visible
    tool_panel_visible: bool,
    /// Chat sidebar state (Hermes-inspired)
    sidebar: SidebarState,
    /// Right-side data/plan/terminal/tokens menu
    right_menu: RightMenuState,
    /// Command history (previous user inputs, for ↑↓ navigation)
    cmd_history: Vec<String>,
    /// Current position in command history (None = fresh input)
    history_index: Option<usize>,
    /// Helix-inspired compositor layer stack
    compositor: Compositor,
    /// Current AI operation mode
    ai_mode: AiMode,
    /// Streaming markdown state (line-gate buffer + pulldown_cmark renderer)
    markdown_stream: MarkdownStream,
    markdown_renderer: MarkdownRenderer,
    /// Timestamp of last streaming render (for throttle)
    last_stream_render: Instant,
    /// Editor state (code editor with inline completions)
    editor: EditorState,
    /// Which panel is currently focused
    panel_focus: PanelFocus,
    /// Approval overlay selection index
    approval_selection: usize,
    /// Whether the approval overlay is showing
    showing_approval: bool,
    /// Multi-line input mode (true = Enter inserts newline, Alt+Enter sends)
    multiline_mode: bool,
    /// Streaming speed tracking
    stream_start: Option<Instant>,
    stream_token_count: usize,
    last_tokens_per_sec: f64,
    /// AI code template panel
    ai_code_template: AICodeTemplate,
    /// Project memo pad
    project_memo: ProjectMemo,
    /// Command snippet library
    command_snippet: CommandSnippet,
    /// Unified diff viewer
    unified_diff_viewer: UnifiedDiffViewer,
    /// Task management board
    task_board: TaskBoard,
    /// UI change detector for optimization
    change_detector: ChangeDetector,
    /// Telemetry metrics
    telemetry: Telemetry,
}

impl App {
    fn new() -> Self {
        let auth = AuthSource::from_env().ok();
        let config = TuiConfig::default();
        let (is_demo, status_msg) = if auth.is_some() {
            (
                false,
                "Ready — press Enter to send, Ctrl+K for config, F1 for help".to_string(),
            )
        } else {
            (
                true,
                "🔧 DEMO MODE — no API key required for UI preview".to_string(),
            )
        };

        let now = Instant::now();
        let status_bar = StatusBar {
            message: status_msg.clone(),
            ..Default::default()
        };

        // Initialize editor with sample content
        let default_theme = Theme::auto();
        let markdown_theme = MarkdownTheme {
            border: default_theme.color.border,
            primary: default_theme.color.primary,
            accent: default_theme.color.accent,
            text: default_theme.color.text,
            text_muted: default_theme.color.text_muted,
            status_bg: default_theme.color.status_bg,
            ok: default_theme.color.ok,
            error: default_theme.color.error,
        };

        let mut app = Self {
            messages: Vec::new(),
            input: String::new(),
            cursor: 0,
            screen: UiScreen::Chat,
            status: status_msg,
            status_bar,
            started_at: now,
            scroll: 0,
            is_streaming: false,
            streaming_text: String::new(),
            is_demo,
            config: TuiConfig::default(),
            active_provider: ProviderKind::Anthropic,
            theme: default_theme,
            theme_name: "auto".to_string(),
            config_field: 0,
            config_edit: String::new(),
            autocomplete: SlashAutocomplete::default(),
            slash_dispatcher: SlashCommandDispatcher::new(),
            dash_selection: 0,
            dash_tab: 0,
            tool_calls: Vec::new(),
            tool_panel_visible: false,
            sidebar: SidebarState::default(),
            right_menu: RightMenuState::default(),
            cmd_history: Vec::new(),
            history_index: None,
            ai_mode: AiMode::Chat,
            markdown_stream: MarkdownStream::new(),
            markdown_renderer: MarkdownRenderer::new(&markdown_theme),
            last_stream_render: Instant::now(),
            editor: EditorState::default(),
            panel_focus: PanelFocus::Chat,
            approval_selection: 0,
            showing_approval: false,
            multiline_mode: false,
            stream_start: None,
            stream_token_count: 0,
            last_tokens_per_sec: 0.0,
            compositor: {
                let mut c = Compositor::new();
                c.push(Box::new(ChatLayer::new()));
                c
            },
            ai_code_template: AICodeTemplate::new(),
            project_memo: ProjectMemo::new(),
            command_snippet: CommandSnippet::new(),
            unified_diff_viewer: UnifiedDiffViewer::new(),
            task_board: TaskBoard::new(),
            change_detector: ChangeDetector::new(),
            telemetry: Telemetry::new(),
        };

        // Add welcome messages
        if is_demo {
            app.add_msg(
                MsgRole::System,
                format!(
                    "{}\n\nWelcome to OpenCode TUI (Demo Mode)\n\n\
                 No API key is configured, so responses are simulated.\n\
                 Set ANTHROPIC_API_KEY or OPENAI_API_KEY env var for real LLM responses.\n\n\
                 Available commands:\n\
                 /help  — Show available commands\n\
                 /status — Show current configuration\n\
                 /tools  — List available tools\n\
                         /permissions — Show permission policies",
                    OPENCODE_LOGO
                ),
            );
        } else {
            app.add_msg(
                MsgRole::System,
                format!(
                    "{}\n\nOpenCode TUI — AI coding assistant\n\
                 Press Enter to chat, F1 for help, Ctrl+K for settings",
                    OPENCODE_LOGO
                ),
            );
        }

        // Try to detect provider from env
        if std::env::var("OPENAI_API_KEY").is_ok() {
            app.active_provider = ProviderKind::OpenAiCompat;
            app.config.model =
                std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            app.status = "OpenAI-compatible credentials detected. Ready.".to_string();
        } else if std::env::var("ANTHROPIC_API_KEY").is_ok()
            || std::env::var("ANTHROPIC_AUTH_TOKEN").is_ok()
        {
            app.config.model = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
            app.status = "Anthropic credentials detected. Ready.".to_string();
        }

        app
    }

    fn add_msg(&mut self, role: MsgRole, text: String) {
        self.messages.push(ChatMsg {
            role,
            text,
            timestamp: Instant::now(),
        });
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_add(3);
    }

    fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_sub(3);
    }

    fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    fn config_field_name(&self, i: usize) -> &'static str {
        match i {
            0 => "Provider (anthropic / openai)",
            1 => "Model name",
            2 => "Max tokens",
            3 => "Temperature (empty = default)",
            4 => "System prompt",
            5 => "Theme",
            _ => "",
        }
    }

    fn config_field_value(&self, i: usize) -> String {
        match i {
            0 => format!("{:?}", self.active_provider),
            1 => self.config.model.clone(),
            2 => self.config.max_tokens.to_string(),
            3 => self
                .config
                .temperature
                .map(|t| t.to_string())
                .unwrap_or_default(),
            4 => {
                if self.config.system_prompt.len() > 60 {
                    format!("{}...", &self.config.system_prompt[..60])
                } else {
                    self.config.system_prompt.clone()
                }
            }
            5 => self.theme_name.clone(),
            _ => String::new(),
        }
    }

    fn save_session(&self, filename: &str) -> bool {
        use std::io::Write;
        let path = std::path::Path::new(filename);
        // If relative, save in current directory.
        let path = if path.is_relative() {
            std::env::current_dir().unwrap_or_default().join(filename)
        } else {
            path.to_path_buf()
        };
        let mut file = match std::fs::File::create(&path) {
            Ok(f) => f,
            Err(_) => return false,
        };
        for msg in &self.messages {
            let line = serde_json::json!({
                "role": format!("{:?}", msg.role),
                "text": msg.text,
            });
            if writeln!(file, "{}", line).is_err() {
                return false;
            }
        }
        true
    }

    fn load_session(&mut self, filename: &str) -> bool {
        let path = std::path::Path::new(filename);
        let path = if path.is_relative() {
            std::env::current_dir().unwrap_or_default().join(filename)
        } else {
            path.to_path_buf()
        };
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        self.messages.clear();
        for line in content.lines() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                let role = val.get("role").and_then(|v| v.as_str()).unwrap_or("User");
                let text = val.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let msg_role = match role {
                    "User" | "user" => MsgRole::User,
                    "Assistant" | "assistant" => MsgRole::Assistant,
                    "System" | "system" => MsgRole::System,
                    "Error" | "error" => MsgRole::Error,
                    _ => MsgRole::System,
                };
                self.messages.push(ChatMsg {
                    role: msg_role,
                    text: text.to_string(),
                    timestamp: Instant::now(),
                });
            }
        }
        true
    }

    fn show_status(&self) -> String {
        let config = &self.config;
        format!(
            "--- Status ---\n\
             Provider: {:?}\n\
             Model: {}\n\
             Max tokens: {}\n\
             Temperature: {}\n\
             Messages: {}\n\
             System prompt: {}",
            self.active_provider,
            config.model,
            config.max_tokens,
            config
                .temperature
                .map(|t| t.to_string())
                .unwrap_or_else(|| "default".into()),
            self.messages.len(),
            if config.system_prompt.len() > 50 {
                format!("{}...", &config.system_prompt[..50])
            } else {
                config.system_prompt.clone()
            },
        )
    }
}

fn list_permissions() -> String {
    let policies = opencode_llm::permissions::PermissionEnforcer::default();
    let mut out = "--- Permission Policies ---\n".to_string();
    for p in policies.policies() {
        let level = match p.level {
            opencode_llm::permissions::PermissionLevel::AlwaysAllow => "ALLOW",
            opencode_llm::permissions::PermissionLevel::AskUser => "ASK",
            opencode_llm::permissions::PermissionLevel::AlwaysDeny => "DENY",
        };
        out.push_str(&format!("  {level:5}  {:<20}  {}\n", p.pattern, p.label));
    }
    out
}

fn list_available_tools() -> String {
    let specs = opencode_llm::tools::mvp_tool_specs();
    let mut out = "--- Available Tools ---\n".to_string();
    for spec in &specs {
        let desc = if spec.description.len() > 60 {
            format!("{}...", &spec.description[..60])
        } else {
            spec.description.clone()
        };
        out.push_str(&format!("  {:<15} {}\n", spec.name, desc));
    }
    out
}

/// Generate a simulated response for demo mode (no API key needed).
fn demo_response(text: &str, _provider: ProviderKind) -> String {
    let text = text.trim();
    format!(
        "[Demo Response]\n\n\
         You said: \"{text}\"\n\n\
         This is a simulated response. To use a real LLM, set:\n\
         - ANTHROPIC_API_KEY for Claude models, or\n\
         - OPENAI_API_KEY for OpenAI-compatible providers\n\n\
         You can also press Ctrl+K to configure settings."
    )
}

// ---------------------------------------------------------------------------
// Async command/event channels
// ---------------------------------------------------------------------------

struct SendPrompt {
    text: String,
    provider: ProviderKind,
}

enum AsyncCmd {
    SendPrompt(SendPrompt),
}

enum AsyncEvent {
    Response(String),
    Delta(String),
    Error(String),
}

// ---------------------------------------------------------------------------
// Main entry
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    // Install crash reporter (writes crash dumps on panic)
    let crash_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencode")
        .join("crash_dumps");
    let crash_reporter = CrashReporter::new(crash_dir);
    crash_reporter.install();

    // Install signal handler for graceful shutdown
    install_signal_handler();

    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::event::EnableMouseCapture,
        EnterAlternateScreen
    )?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Build runtime from environment variables and run a single prompt.
fn run_prompt_in_background(text: String, provider_kind: ProviderKind) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

    let auth = AuthSource::from_env().map_err(|e| e.to_string())?;

    match provider_kind {
        ProviderKind::OpenAiCompat => {
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            let base_url = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| opencode_llm::auth::DEFAULT_OPENAI_BASE_URL.to_string());
            let llm_config = LlmConfig::openai_compat(&model, &base_url);
            let client = OpenAiCompatClient::new(auth, llm_config).map_err(|e| e.to_string())?;
            let client_arc: Arc<OpenAiCompatClient> = Arc::new(client);
            let runtime = ConversationRuntime::<OpenAiCompatClient>::builder()
                .model(&model)
                .max_tokens(4096)
                .system("You are OpenCode, an AI coding assistant.")
                .mvp_tools()
                .build(client_arc);
            rt.block_on(runtime.run(&text)).map_err(|e| e.to_string())
        }
        ProviderKind::Anthropic => {
            let model = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
            let llm_config = LlmConfig::anthropic(&model);
            let client = AnthropicClient::new(auth, llm_config).map_err(|e| e.to_string())?;
            let client_arc: Arc<AnthropicClient> = Arc::new(client);
            let runtime = ConversationRuntime::<AnthropicClient>::builder()
                .model(&model)
                .max_tokens(4096)
                .system("You are OpenCode, an AI coding assistant.")
                .mvp_tools()
                .build(client_arc);
            rt.block_on(runtime.run(&text)).map_err(|e| e.to_string())
        }
    }
}

/// Streaming variant — emits text deltas through the channel.
fn run_prompt_streaming_in_background(
    text: String,
    provider_kind: ProviderKind,
    delta_tx: std::sync::mpsc::Sender<String>,
) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let auth = AuthSource::from_env().map_err(|e| e.to_string())?;

    // Adapt std mpsc sender into a tokio mpsc sender for stream_to_channel.
    let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    std::thread::spawn(move || {
        while let Some(delta) = tokio_rx.blocking_recv() {
            let _ = delta_tx.send(delta);
        }
    });

    match provider_kind {
        ProviderKind::OpenAiCompat => {
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            let base_url = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| opencode_llm::auth::DEFAULT_OPENAI_BASE_URL.to_string());
            let llm_config = LlmConfig::openai_compat(&model, &base_url);
            let client = OpenAiCompatClient::new(auth, llm_config).map_err(|e| e.to_string())?;
            let client_arc: Arc<OpenAiCompatClient> = Arc::new(client);
            let runtime = ConversationRuntime::<OpenAiCompatClient>::builder()
                .model(&model)
                .max_tokens(4096)
                .system("You are OpenCode, an AI coding assistant.")
                .mvp_tools()
                .build(client_arc);
            rt.block_on(runtime.stream_to_channel(&text, tokio_tx))
                .map_err(|e| e.to_string())
        }
        ProviderKind::Anthropic => {
            let model = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
            let llm_config = LlmConfig::anthropic(&model);
            let client = AnthropicClient::new(auth, llm_config).map_err(|e| e.to_string())?;
            let client_arc: Arc<AnthropicClient> = Arc::new(client);
            let runtime = ConversationRuntime::<AnthropicClient>::builder()
                .model(&model)
                .max_tokens(4096)
                .system("You are OpenCode, an AI coding assistant.")
                .mvp_tools()
                .build(client_arc);
            rt.block_on(runtime.stream_to_channel(&text, tokio_tx))
                .map_err(|e| e.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Component wrappers — each screen as a Compositor layer
// ---------------------------------------------------------------------------
// These types implement the Component trait so the compositor can manage
// them via the layer stack. They downcast `&dyn Any` to access the App.

use std::any::Any;

/// Wraps the chat screen as a compositor layer.
struct ChatLayer;
impl ChatLayer {
    fn new() -> Self {
        Self
    }
}
impl Component for ChatLayer {
    fn handle_event(&mut self, _e: &KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
    fn handle_event_with_context(&mut self, e: &KeyEvent, ctx: &mut dyn Any) -> HandleResult {
        let _ = ctx.downcast_mut::<App>();
        let _ = e;
        HandleResult::Ignored // handled by main event loop
    }
    fn render(&self, _f: &mut Frame, _a: Rect) {}
    fn render_with_context(&self, f: &mut Frame, _a: Rect, ctx: &mut dyn Any) {
        if let Some(a) = ctx.downcast_mut::<App>() {
            render_chat(f, a);
        }
    }
    fn kind(&self) -> ComponentKind {
        ComponentKind::Chat
    }
}

/// Wraps the config screen as a compositor overlay.
struct ConfigLayer;
impl ConfigLayer {
    fn new() -> Self {
        Self
    }
}
impl Component for ConfigLayer {
    fn handle_event(&mut self, _e: &KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
    fn handle_event_with_context(&mut self, e: &KeyEvent, ctx: &mut dyn Any) -> HandleResult {
        let app = match ctx.downcast_mut::<App>() {
            Some(a) => a,
            None => return HandleResult::Ignored,
        };
        match e.code {
            KeyCode::Esc => {
                app.screen = UiScreen::Chat;
                app.status = "Ready".to_string();
                HandleResult::Close
            }
            KeyCode::Tab => {
                let v = app.config_edit.clone();
                let f = app.config_field;
                app.apply_config_field(&v, f);
                app.config_field = (app.config_field + 1) % 6;
                app.config_edit = app.config_field_value(app.config_field);
                HandleResult::Consumed
            }
            KeyCode::BackTab => {
                app.config_field = (app.config_field + 5) % 6;
                app.config_edit = app.config_field_value(app.config_field);
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                let v = app.config_edit.clone();
                let f = app.config_field;
                app.apply_config_field(&v, f);
                app.screen = UiScreen::Chat;
                app.status = "Config applied".to_string();
                HandleResult::Close
            }
            KeyCode::Char(c) => {
                app.config_edit.push(c);
                HandleResult::Consumed
            }
            KeyCode::Backspace => {
                app.config_edit.pop();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
    fn render(&self, _f: &mut Frame, _a: Rect) {}
    fn render_with_context(&self, f: &mut Frame, _a: Rect, ctx: &mut dyn Any) {
        if let Some(a) = ctx.downcast_mut::<App>() {
            render_config(f, a);
        }
    }
    fn kind(&self) -> ComponentKind {
        ComponentKind::Config
    }
}

/// Wraps the help screen as a compositor overlay.
struct HelpLayer;
impl HelpLayer {
    fn new() -> Self {
        Self
    }
}
impl Component for HelpLayer {
    fn handle_event(&mut self, _e: &KeyEvent) -> HandleResult {
        HandleResult::Close
    }
    fn handle_event_with_context(&mut self, e: &KeyEvent, ctx: &mut dyn Any) -> HandleResult {
        if let Some(a) = ctx.downcast_mut::<App>() {
            a.screen = UiScreen::Chat;
            a.status = "Ready".to_string();
        }
        let _ = e;
        HandleResult::Close
    }
    fn render(&self, _f: &mut Frame, _a: Rect) {}
    fn render_with_context(&self, f: &mut Frame, _a: Rect, ctx: &mut dyn Any) {
        if let Some(a) = ctx.downcast_mut::<App>() {
            render_help(f, a);
        }
    }
    fn kind(&self) -> ComponentKind {
        ComponentKind::Help
    }
}

/// Wraps the dashboard screen as a compositor layer.
struct DashboardLayer;
impl DashboardLayer {
    fn new() -> Self {
        Self
    }
}
impl Component for DashboardLayer {
    fn handle_event(&mut self, _e: &KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
    fn handle_event_with_context(&mut self, e: &KeyEvent, ctx: &mut dyn Any) -> HandleResult {
        let app = match ctx.downcast_mut::<App>() {
            Some(a) => a,
            None => return HandleResult::Ignored,
        };
        match e.code {
            KeyCode::Char('d') if e.modifiers == KeyModifiers::CONTROL => {
                app.screen = UiScreen::Chat;
                app.status = "Ready".to_string();
                HandleResult::Switch(LayerId::Root)
            }
            KeyCode::Enter => match app.dash_selection {
                0 => {
                    app.screen = UiScreen::Chat;
                    app.status = "Switched to Chat".to_string();
                    HandleResult::Switch(LayerId::Root)
                }
                1 => HandleResult::Consumed,
                _ => HandleResult::Consumed,
            },
            KeyCode::Right | KeyCode::Char('l') => {
                app.dash_tab = (app.dash_tab + 1) % 4;
                HandleResult::Consumed
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.dash_tab = (app.dash_tab + 3) % 4;
                HandleResult::Consumed
            }
            KeyCode::Down | KeyCode::Tab => {
                app.dash_selection = (app.dash_selection + 1) % 5;
                HandleResult::Consumed
            }
            KeyCode::Up | KeyCode::BackTab => {
                app.dash_selection = (app.dash_selection + 4) % 5;
                HandleResult::Consumed
            }
            KeyCode::Char('1') => {
                app.dash_tab = 0;
                HandleResult::Consumed
            }
            KeyCode::Char('2') => {
                app.dash_tab = 1;
                HandleResult::Consumed
            }
            KeyCode::Char('3') => {
                app.dash_tab = 2;
                HandleResult::Consumed
            }
            KeyCode::Char('4') => {
                app.dash_tab = 3;
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
    fn render(&self, _f: &mut Frame, _a: Rect) {}
    fn render_with_context(&self, f: &mut Frame, _a: Rect, ctx: &mut dyn Any) {
        if let Some(a) = ctx.downcast_mut::<App>() {
            render_dashboard(f, a);
        }
    }
    fn kind(&self) -> ComponentKind {
        ComponentKind::Dashboard
    }
}

/// Wraps the approval overlay as a compositor layer.
struct ApprovalLayer;
impl ApprovalLayer {
    fn new() -> Self {
        Self
    }
}
impl Component for ApprovalLayer {
    fn handle_event(&mut self, _e: &KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
    fn handle_event_with_context(&mut self, e: &KeyEvent, ctx: &mut dyn Any) -> HandleResult {
        let app = match ctx.downcast_mut::<App>() {
            Some(a) => a,
            None => return HandleResult::Ignored,
        };
        match e.code {
            // Approve all pending tool calls
            KeyCode::Char('Y') | KeyCode::Char('y') if e.modifiers == KeyModifiers::NONE => {
                for tc in &mut app.tool_calls {
                    if tc.status == ToolStatus::Pending {
                        tc.status = ToolStatus::Running;
                    }
                }
                app.showing_approval = false;
                app.compositor.remove_layer(ComponentKind::Approval);
                app.status = "Approved all pending tool calls".to_string();
                HandleResult::Consumed
            }
            // Deny all pending tool calls
            KeyCode::Char('N') | KeyCode::Char('n') if e.modifiers == KeyModifiers::NONE => {
                app.tool_calls.retain(|tc| tc.status != ToolStatus::Pending);
                app.showing_approval = false;
                app.compositor.remove_layer(ComponentKind::Approval);
                app.status = "Denied all pending tool calls".to_string();
                HandleResult::Consumed
            }
            // Navigate selection
            KeyCode::Down | KeyCode::Tab => {
                let pending_count = app
                    .tool_calls
                    .iter()
                    .filter(|tc| tc.status == ToolStatus::Pending)
                    .count();
                if pending_count > 0 {
                    app.approval_selection = (app.approval_selection + 1) % pending_count;
                }
                HandleResult::Consumed
            }
            KeyCode::Up | KeyCode::BackTab => {
                let pending_count = app
                    .tool_calls
                    .iter()
                    .filter(|tc| tc.status == ToolStatus::Pending)
                    .count();
                if pending_count > 0 {
                    app.approval_selection =
                        (app.approval_selection + pending_count - 1) % pending_count;
                }
                HandleResult::Consumed
            }
            // Approve selected
            KeyCode::Enter => {
                let pending_indices: Vec<usize> = app
                    .tool_calls
                    .iter()
                    .enumerate()
                    .filter(|(_, tc)| tc.status == ToolStatus::Pending)
                    .map(|(i, _)| i)
                    .collect();
                if let Some(&idx) = pending_indices.get(app.approval_selection) {
                    if let Some(tc) = app.tool_calls.get_mut(idx) {
                        tc.status = ToolStatus::Running;
                    }
                }
                // Check if any remaining pending
                if !app
                    .tool_calls
                    .iter()
                    .any(|tc| tc.status == ToolStatus::Pending)
                {
                    app.showing_approval = false;
                    app.compositor.remove_layer(ComponentKind::Approval);
                    app.status = "Approved selected tool call".to_string();
                }
                HandleResult::Consumed
            }
            // Close overlay
            KeyCode::Esc | KeyCode::Char('q') => {
                app.showing_approval = false;
                app.compositor.remove_layer(ComponentKind::Approval);
                app.status = "Approval cancelled".to_string();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
    fn render(&self, _f: &mut Frame, _a: Rect) {}
    fn render_with_context(&self, f: &mut Frame, _a: Rect, ctx: &mut dyn Any) {
        if let Some(a) = ctx.downcast_mut::<App>() {
            render_approval_overlay(f, a);
        }
    }
    fn kind(&self) -> ComponentKind {
        ComponentKind::Approval
    }
}

/// Generic overlay component for modal content.
/// Can be used for any centered modal dialog.
struct GenericOverlay {
    title: String,
    content_lines: Vec<String>,
    width_pct: u16,
    height_pct: u16,
    close_on_esc: bool,
}

impl GenericOverlay {
    fn new(title: impl Into<String>, content: Vec<String>) -> Self {
        Self {
            title: title.into(),
            content_lines: content,
            width_pct: 60,
            height_pct: 60,
            close_on_esc: true,
        }
    }

    fn with_size(mut self, width_pct: u16, height_pct: u16) -> Self {
        self.width_pct = width_pct;
        self.height_pct = height_pct;
        self
    }
}

impl Component for GenericOverlay {
    fn handle_event(&mut self, e: &KeyEvent) -> HandleResult {
        if self.close_on_esc && (e.code == KeyCode::Esc || e.code == KeyCode::Char('q')) {
            HandleResult::Close
        } else {
            HandleResult::Ignored
        }
    }

    fn render(&self, _frame: &mut Frame, _area: Rect) {
        // Default render without context — no-op
    }

    fn render_with_context(&self, frame: &mut Frame, area: Rect, ctx: &mut dyn Any) {
        let app = match ctx.downcast_mut::<App>() {
            Some(a) => a,
            None => return,
        };
        let t = &app.theme.color;

        use ratatui::layout::Rect as TuiRect;
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};

        let w = area.width * self.width_pct / 100;
        let h = area.height * self.height_pct / 100;
        let overlay = TuiRect {
            x: (area.width - w) / 2,
            y: (area.height - h) / 2,
            width: w,
            height: h,
        };

        frame.render_widget(Clear, overlay);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(t.accent));
        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        let text: Vec<Line> = self
            .content_lines
            .iter()
            .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(t.text))))
            .collect();

        frame.render_widget(Paragraph::new(text), inner);
    }

    fn kind(&self) -> ComponentKind {
        ComponentKind::Overlay
    }
}

/// A generic picker item with a label and optional preview text.
#[derive(Debug, Clone)]
struct PickerItem {
    label: String,
    preview: Option<String>,
    value: String,
}

/// Generic picker/menu component with fuzzy filtering and optional preview.
struct Picker {
    title: String,
    items: Vec<PickerItem>,
    filtered: Vec<usize>,
    selected: usize,
    filter: String,
    show_preview: bool,
    width_pct: u16,
    height_pct: u16,
}

impl Picker {
    fn new(title: impl Into<String>, items: Vec<PickerItem>) -> Self {
        let filtered = (0..items.len()).collect();
        Self {
            title: title.into(),
            items,
            filtered,
            selected: 0,
            filter: String::new(),
            show_preview: true,
            width_pct: 70,
            height_pct: 70,
        }
    }

    fn with_preview(mut self, show: bool) -> Self {
        self.show_preview = show;
        self
    }

    fn with_size(mut self, width_pct: u16, height_pct: u16) -> Self {
        self.width_pct = width_pct;
        self.height_pct = height_pct;
        self
    }

    fn update_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered = (0..self.items.len()).collect();
        } else {
            // Use fuzzy search with scoring
            let labels: Vec<String> = self.items.iter().map(|i| i.label.clone()).collect();
            let scored = fuzzy_sort(&self.filter, &labels);
            self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        }
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    fn selected_item(&self) -> Option<&PickerItem> {
        self.filtered.get(self.selected).map(|&i| &self.items[i])
    }
}

impl Component for Picker {
    fn handle_event(&mut self, e: &KeyEvent) -> HandleResult {
        match e.code {
            KeyCode::Esc => HandleResult::Close,
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = self.filtered.len().saturating_sub(1);
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                } else {
                    self.selected = 0;
                }
                HandleResult::Consumed
            }
            KeyCode::Char(c) if e.modifiers.is_empty() || e.modifiers == KeyModifiers::SHIFT => {
                self.filter.push(c);
                self.update_filter();
                HandleResult::Consumed
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.update_filter();
                HandleResult::Consumed
            }
            KeyCode::Enter => HandleResult::Close,
            _ => HandleResult::Ignored,
        }
    }

    fn handle_event_with_context(&mut self, e: &KeyEvent, ctx: &mut dyn Any) -> HandleResult {
        if e.code == KeyCode::Enter {
            if let Some(app) = ctx.downcast_mut::<App>() {
                if let Some(item) = self.selected_item() {
                    app.config.model = item.value.clone();
                    app.status = format!("Model changed to: {}", item.value);
                }
            }
            HandleResult::Close
        } else {
            self.handle_event(e)
        }
    }

    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn render_with_context(&self, frame: &mut Frame, area: Rect, ctx: &mut dyn Any) {
        let app = match ctx.downcast_mut::<App>() {
            Some(a) => a,
            None => return,
        };
        let t = &app.theme.color;

        use ratatui::layout::{Constraint, Direction, Layout, Rect as TuiRect};
        use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

        let w = area.width * self.width_pct / 100;
        let h = area.height * self.height_pct / 100;
        let overlay = TuiRect {
            x: (area.width - w) / 2,
            y: (area.height - h) / 2,
            width: w,
            height: h,
        };

        frame.render_widget(Clear, overlay);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(t.accent));
        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        // Split: filter input, list, optional preview
        let constraints = if self.show_preview {
            vec![
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(8),
            ]
        } else {
            vec![Constraint::Length(3), Constraint::Min(5)]
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // Filter input
        let filter_display = format!("🔍 {}", self.filter);
        frame.render_widget(
            Paragraph::new(filter_display).style(Style::default().fg(t.text)),
            chunks[0],
        );

        // Item list
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .enumerate()
            .map(|(idx, &i)| {
                let item = &self.items[i];
                let is_selected = idx == self.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(t.text)
                        .bg(t.status_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(t.text)
                };
                let prefix = if is_selected { "▸ " } else { "  " };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", prefix, item.label),
                    style,
                )))
            })
            .collect();
        frame.render_widget(List::new(items), chunks[1]);

        // Preview pane
        if self.show_preview && chunks.len() > 2 {
            let preview_text = self
                .selected_item()
                .and_then(|i| i.preview.as_deref())
                .unwrap_or("(no preview)");
            frame.render_widget(
                Paragraph::new(preview_text)
                    .style(Style::default().fg(t.text_muted))
                    .wrap(Wrap { trim: true }),
                chunks[2],
            );
        }
    }

    fn kind(&self) -> ComponentKind {
        ComponentKind::Popup
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<AsyncCmd>();
    let (evt_tx, evt_rx) = std::sync::mpsc::channel::<AsyncEvent>();

    // Background thread for async LLM calls
    std::thread::spawn(move || {
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                AsyncCmd::SendPrompt(s) => {
                    let result = run_prompt_in_background(s.text, s.provider);
                    match result {
                        Ok(resp) => {
                            let _ = evt_tx.send(AsyncEvent::Response(resp.trim().to_string()));
                        }
                        Err(e) => {
                            let _ = evt_tx.send(AsyncEvent::Error(e));
                        }
                    }
                }
            }
        }
    });

    // Main loop
    loop {
        // Check for graceful shutdown signal
        if is_shutdown_requested() {
            break Ok(());
        }

        terminal.draw(|frame| {
            app.telemetry.record_frame();
            render(frame, app)
        })?;

        // Update rich status bar each frame
        app.status_bar.tick = app.status_bar.tick.wrapping_add(1);
        app.status_bar.elapsed_secs = app.started_at.elapsed().as_secs();
        app.status_bar.busy = app.is_streaming;
        app.status_bar.msg_count = app.messages.len();
        // Show streaming speed in status
        app.status_bar.message = if app.is_streaming {
            if let Some(start) = app.stream_start {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed > 0.5 {
                    let tps = app.stream_token_count as f64 / elapsed;
                    format!(
                        "Streaming… {:.1} tok/s ({} tokens)",
                        tps, app.stream_token_count
                    )
                } else {
                    "Streaming…".to_string()
                }
            } else {
                "Streaming…".to_string()
            }
        } else if app.last_tokens_per_sec > 0.0 {
            format!("{} ({:.1} tok/s)", app.status, app.last_tokens_per_sec)
        } else {
            app.status.clone()
        };

        // Check for async events (non-blocking)
        if let Ok(event) = evt_rx.try_recv() {
            match event {
                AsyncEvent::Delta(text) => {
                    app.markdown_stream.push(&text);
                    app.streaming_text.push_str(&text);
                    app.stream_token_count += text.split_whitespace().count().max(1);
                }
                AsyncEvent::Response(text) => {
                    app.telemetry.record_llm_call();
                    app.telemetry
                        .record_tokens(0, app.stream_token_count as u64);
                    app.streaming_text.clear();
                    app.add_msg(MsgRole::Assistant, text);
                    app.is_streaming = false;
                    app.status_bar.busy = false;
                    app.status_bar.mode_label = if app.multiline_mode {
                        "MULTILINE".into()
                    } else {
                        "NORMAL".into()
                    };
                    // Calculate tokens/sec
                    if let Some(start) = app.stream_start.take() {
                        let elapsed = start.elapsed().as_secs_f64();
                        if elapsed > 0.0 {
                            app.last_tokens_per_sec = app.stream_token_count as f64 / elapsed;
                        }
                    }
                    // Track estimated token usage
                    let total_chars: usize = app.messages.iter().map(|m| m.text.len()).sum();
                    app.status_bar.token_used = total_chars / 4;
                    if app.status_bar.token_limit == 0 {
                        app.status_bar.token_limit = 4000;
                    }
                    // Auto-show approval overlay if there are pending tool calls
                    if app
                        .tool_calls
                        .iter()
                        .any(|tc| tc.status == ToolStatus::Pending)
                        && !app.showing_approval
                    {
                        app.showing_approval = true;
                        app.approval_selection = 0;
                        app.compositor.remove_layer(ComponentKind::Approval);
                        app.compositor.push(Box::new(ApprovalLayer::new()));
                        app.status = format!(
                            "⚠ {} tool call(s) require approval — Y=Approve all, N=Deny all, Enter=Selective",
                            app.tool_calls.iter().filter(|tc| tc.status == ToolStatus::Pending).count()
                        );
                    } else {
                        app.status = "Response received".to_string();
                    }
                    app.reset_scroll();
                }
                AsyncEvent::Error(e) => {
                    app.telemetry.record_llm_error();
                    app.streaming_text.clear();
                    app.add_msg(MsgRole::Error, e.clone());
                    app.status = format!("Error: {e}");
                    app.status_bar.busy = false;
                    app.is_streaming = false;
                    app.status_bar.mode_label = if app.multiline_mode {
                        "MULTILINE".into()
                    } else {
                        "NORMAL".into()
                    };
                }
            }
        }

        // Read keyboard events
        if !event::poll(std::time::Duration::from_millis(50))? {
            continue;
        }
        let ev = event::read()?;

        // Handle mouse events
        if let Event::Mouse(mouse) = ev {
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                let col = mouse.column;
                let row = mouse.row;
                // Check if click is within sidebar area (if sidebar is visible)
                if let Some(sb_area) = app.sidebar.rendered_area {
                    if col >= sb_area.x
                        && col < sb_area.x + sb_area.width
                        && row >= sb_area.y
                        && row < sb_area.y + sb_area.height
                    {
                        app.sidebar.visible = !app.sidebar.visible;
                        app.status = if app.sidebar.visible {
                            "Sidebar: visible (click to hide)".to_string()
                        } else {
                            "Sidebar: hidden (click area to show)".to_string()
                        };
                        continue;
                    }
                }
                // Otherwise toggle sidebar if clicking near the left edge
                if col < 3 && !app.sidebar.visible {
                    app.sidebar.visible = true;
                    app.status = "Sidebar: visible".to_string();
                }
                app.status = format!("Click at ({}, {})", col, row);
            }
            continue;
        }

        if let Event::Key(key) = ev {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Delegate to compositor first (for overlay layers).
            // Use handle_event_with_context so overlay layers (Config, Help)
            // can respond to events via downcast access to App.
            // SAFETY: compositor does not store the context after the call.
            let app_ptr = &mut *app as *mut App;
            let ctx: &mut dyn Any = unsafe { &mut *app_ptr };
            match app.compositor.handle_event_with_context(&key, ctx) {
                HandleResult::Exit => return Ok(()),
                HandleResult::Consumed | HandleResult::Close => continue,
                HandleResult::Switch(target) => {
                    // Map target layer to UiScreen and manage layer stack
                    match target {
                        LayerId::Root => {
                            // Pop all non-root layers
                            while app.compositor.depth() > 1 {
                                app.compositor.pop();
                            }
                            app.screen = UiScreen::Chat;
                        }
                        LayerId::Config => {
                            app.screen = UiScreen::Config;
                        }
                        LayerId::Help => {
                            app.screen = UiScreen::Help;
                        }
                        LayerId::Dashboard => {
                            app.screen = UiScreen::Dashboard;
                        }
                        _ => {}
                    }
                    app.status = format!("Switched to {:?}", app.screen);
                    continue;
                }
                HandleResult::Ignored => {} // Fall through to existing logic
            }

            match app.screen {
                UiScreen::Chat => {
                    // If editor is visible and focused, route keys to editor first
                    if app.editor.visible && app.panel_focus == PanelFocus::Editor {
                        match key.code {
                            // Tab to switch panel focus
                            KeyCode::Tab => {
                                app.panel_focus = PanelFocus::Chat;
                                app.status = "Chat panel focused".to_string();
                            }
                            KeyCode::Esc if app.editor.mode == EditorMode::Insert => {
                                app.editor.mode = EditorMode::Normal;
                                app.status = "Editor: NORMAL".to_string();
                            }
                            KeyCode::Char('i')
                                if key.modifiers == KeyModifiers::NONE
                                    && app.editor.mode == EditorMode::Normal =>
                            {
                                app.editor.mode = EditorMode::Insert;
                                app.status = "Editor: INSERT".to_string();
                            }
                            // Insert mode key handling
                            _ if app.editor.mode == EditorMode::Insert => match key.code {
                                KeyCode::Char(c) => {
                                    app.editor.insert_char(c);
                                    app.editor.suggestion.visible = false;
                                }
                                KeyCode::Enter => {
                                    app.editor.insert_newline();
                                }
                                KeyCode::Backspace => {
                                    app.editor.backspace();
                                }
                                KeyCode::Delete => {
                                    app.editor.delete();
                                }
                                KeyCode::Left => app.editor.move_left(),
                                KeyCode::Right => app.editor.move_right(),
                                KeyCode::Up => app.editor.move_up(),
                                KeyCode::Down => app.editor.move_down(),
                                KeyCode::Home => app.editor.home(),
                                KeyCode::End => app.editor.end(),
                                _ => {}
                            },
                            // Normal mode key handling
                            _ => match key.code {
                                KeyCode::Char('h') | KeyCode::Left => app.editor.move_left(),
                                KeyCode::Char('l') | KeyCode::Right => app.editor.move_right(),
                                KeyCode::Char('k') | KeyCode::Up => app.editor.move_up(),
                                KeyCode::Char('j') | KeyCode::Down => app.editor.move_down(),
                                KeyCode::Char('0') => app.editor.home(),
                                KeyCode::Char('$') => app.editor.end(),
                                KeyCode::Char('x') => app.editor.delete(),
                                KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
                                    app.panel_focus = PanelFocus::Chat;
                                    app.status = "Chat panel focused".to_string();
                                }
                                _ => {}
                            },
                        }
                        continue;
                    }
                    match key.code {
                        // Command history navigation (↑/↓) — chat panel only
                        KeyCode::Up if !app.cmd_history.is_empty() => {
                            let new_idx = match app.history_index {
                                None => Some(app.cmd_history.len() - 1),
                                Some(0) => Some(0),
                                Some(i) => Some(i - 1),
                            };
                            app.history_index = new_idx;
                            if let Some(idx) = app.history_index {
                                app.input = app.cmd_history[idx].clone();
                                app.cursor = app.input.len();
                                app.status =
                                    format!("History [{}/{}]", idx + 1, app.cmd_history.len());
                            }
                        }
                        KeyCode::Down if !app.cmd_history.is_empty() => {
                            let new_idx = match app.history_index {
                                Some(i) if i + 1 < app.cmd_history.len() => Some(i + 1),
                                _ => None,
                            };
                            app.history_index = new_idx;
                            if let Some(idx) = app.history_index {
                                app.input = app.cmd_history[idx].clone();
                                app.cursor = app.input.len();
                                app.status =
                                    format!("History [{}/{}]", idx + 1, app.cmd_history.len());
                            } else {
                                app.input.clear();
                                app.cursor = 0;
                                app.status = "History: fresh input".to_string();
                            }
                        }
                        KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                            return Ok(())
                        }
                        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                            app.sidebar.visible = !app.sidebar.visible;
                            app.status = if app.sidebar.visible {
                                "Sidebar: visible — Ctrl+S to hide".to_string()
                            } else {
                                "Sidebar: hidden".to_string()
                            };
                        }
                        KeyCode::Char('m') if key.modifiers == KeyModifiers::CONTROL => {
                            app.ai_mode = app.ai_mode.next();
                            app.status = format!(
                                "AI mode: {} {} — Ctrl+M to cycle",
                                app.ai_mode.icon(),
                                app.ai_mode.label(),
                            );
                            // Auto-show tool panel for Agent / Autopilot modes
                            if matches!(app.ai_mode, AiMode::Agent | AiMode::Autopilot) {
                                app.tool_panel_visible = true;
                            }
                            // Auto-show right menu for Plan / Review modes
                            if matches!(app.ai_mode, AiMode::Plan | AiMode::Review) {
                                app.right_menu.visible = true;
                                if app.ai_mode == AiMode::Plan {
                                    app.right_menu.focus = RightMenuSection::Plan;
                                }
                            }
                        }
                        KeyCode::Char('i') if key.modifiers == KeyModifiers::CONTROL => {
                            app.right_menu.visible = !app.right_menu.visible;
                            app.status = if app.right_menu.visible {
                                "Right menu: visible — Ctrl+I to hide".to_string()
                            } else {
                                "Right menu: hidden".to_string()
                            };
                        }
                        KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                            app.screen = UiScreen::Dashboard;
                            app.status = "Dashboard — Ctrl+D to return to chat".to_string();
                            app.compositor
                                .replace_or_push(Box::new(DashboardLayer::new()));
                        }
                        KeyCode::Char('k') if key.modifiers == KeyModifiers::CONTROL => {
                            app.screen = UiScreen::Config;
                            app.config_field = 0;
                            app.config_edit = app.config_field_value(0);
                            app.status = "Config mode — edit values, Tab to switch, Esc to return"
                                .to_string();
                            app.compositor.replace_or_push(Box::new(ConfigLayer::new()));
                        }
                        KeyCode::F(1) => {
                            app.screen = UiScreen::Help;
                            app.status = "Help — press any key to return".to_string();
                            app.compositor.replace_or_push(Box::new(HelpLayer::new()));
                        }
                        KeyCode::Char('e') if key.modifiers == KeyModifiers::CONTROL => {
                            app.editor.visible = !app.editor.visible;
                            if app.editor.visible {
                                app.panel_focus = PanelFocus::Editor;
                                app.status = "Editor: visible — Ctrl+E to hide".to_string();
                            } else {
                                app.panel_focus = PanelFocus::Chat;
                                app.status = "Editor: hidden".to_string();
                            }
                        }
                        KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                            app.tool_panel_visible = !app.tool_panel_visible;
                            if app.tool_panel_visible {
                                app.status =
                                    format!("Tool panel: visible ({} tools)", app.tool_calls.len());
                            } else {
                                app.tool_calls.clear();
                                app.status = "Tool panel: hidden (cleared)".to_string();
                            }
                        }
                        KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                            if !app.tool_calls.is_empty() {
                                toggle_last_tool(app);
                                app.status = "Tool call toggled".to_string();
                            }
                        }
                        KeyCode::Char('m') if key.modifiers == KeyModifiers::CONTROL => {
                            app.multiline_mode = !app.multiline_mode;
                            app.status_bar.mode_label = if app.multiline_mode {
                                "MULTILINE".into()
                            } else {
                                "NORMAL".into()
                            };
                            app.status = if app.multiline_mode {
                                "Multi-line mode ON — Enter=newline, Alt+Enter=send".to_string()
                            } else {
                                "Multi-line mode OFF — Enter=send".to_string()
                            };
                            continue;
                        }
                        // Ctrl+P: Open model picker
                        KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                            let models = get_available_models(app.active_provider);
                            let items: Vec<PickerItem> = models
                                .iter()
                                .map(|m| PickerItem {
                                    label: m.to_string(),
                                    preview: Some(format!("Select {} as the active model", m)),
                                    value: m.to_string(),
                                })
                                .collect();
                            let picker = Picker::new("Model Picker", items);
                            app.compositor.push(Box::new(picker));
                            app.status =
                                "Model picker — ↑↓ to navigate, Enter to select, Esc to cancel"
                                    .to_string();
                            continue;
                        }
                        KeyCode::Enter => {
                            // Alt+Enter: always send (escape from multiline mode)
                            // Normal Enter in multiline_mode: insert newline
                            // Normal Enter in normal mode: send message
                            if key.modifiers != KeyModifiers::ALT && app.multiline_mode {
                                app.input.insert(app.cursor, '\n');
                                app.cursor += 1;
                                continue;
                            }
                            if app.is_streaming {
                                continue;
                            }
                            let text = app.input.trim().to_string();
                            if text.is_empty() {
                                continue;
                            }
                            // Check for slash commands first (RPC dispatch).
                            match app.slash_dispatcher.dispatch(&text) {
                                Some(action) => {
                                    app.input.clear();
                                    app.cursor = 0;
                                    match action {
                                        SlashAction::Help => {
                                            let mut help = String::from("Available commands:\n");
                                            for cmd in BUILTIN_COMMANDS {
                                                help.push_str(&format!(
                                                    "  {:<20} {}\n",
                                                    cmd.usage, cmd.description
                                                ));
                                            }
                                            app.add_msg(MsgRole::System, help);
                                        }
                                        SlashAction::Clear => {
                                            app.messages.clear();
                                            app.status = "History cleared".to_string();
                                        }
                                        SlashAction::Exit => {
                                            return Ok(());
                                        }
                                        SlashAction::SetModel { model } => {
                                            app.config.model = model.clone();
                                            app.add_msg(
                                                MsgRole::System,
                                                format!("Switched model to `{model}`"),
                                            );
                                            app.status = format!("Model set to `{model}`");
                                        }
                                        SlashAction::OpenConfig => {
                                            app.screen = UiScreen::Config;
                                            app.config_field = 0;
                                            app.config_edit = app.config_field_value(0);
                                            app.status = "Config mode — edit values, Tab to switch, Esc to return".to_string();
                                        }
                                        SlashAction::ListTools => {
                                            let tools = list_available_tools();
                                            app.add_msg(MsgRole::System, tools);
                                            app.status = "Tools listed".to_string();
                                        }
                                        SlashAction::SaveSession { filename } => {
                                            let saved = app.save_session(&filename);
                                            if saved {
                                                app.status = format!("Session saved to {filename}");
                                            } else {
                                                app.status =
                                                    format!("Failed to save session to {filename}");
                                            }
                                        }
                                        SlashAction::LoadSession { filename } => {
                                            let loaded = app.load_session(&filename);
                                            if loaded {
                                                app.add_msg(
                                                    MsgRole::System,
                                                    format!("Session loaded from {filename}"),
                                                );
                                                app.status = "Session loaded".to_string();
                                            } else {
                                                app.status = format!(
                                                    "Failed to load session from {filename}"
                                                );
                                            }
                                        }
                                        SlashAction::ShowPermissions => {
                                            let perms = list_permissions();
                                            app.add_msg(MsgRole::System, perms);
                                            app.status = "Permissions shown".to_string();
                                        }
                                        SlashAction::ShowStatus => {
                                            let status = app.show_status();
                                            app.add_msg(MsgRole::System, status);
                                            app.status = "Status shown".to_string();
                                        }
                                        SlashAction::ShowFiles => {
                                            app.add_msg(
                                                MsgRole::System,
                                                "Session files: (not yet implemented)".to_string(),
                                            );
                                        }
                                        SlashAction::Compact => {
                                            let count = app.messages.len();
                                            app.messages.clear();
                                            app.add_msg(
                                                MsgRole::System,
                                                format!("Compacted: removed {count} messages"),
                                            );
                                            app.status = "Session compacted".to_string();
                                        }
                                        SlashAction::Unknown { name } => {
                                            app.add_msg(MsgRole::System, format!("Unknown command: `/{name}`\nType `/help` for available commands."));
                                        }
                                    }
                                    app.reset_scroll();
                                    continue;
                                }
                                None => {
                                    // Save to command history (skip duplicates)
                                    let trimmed = text.trim().to_string();
                                    if !trimmed.is_empty()
                                        && app
                                            .cmd_history
                                            .last()
                                            .map_or(true, |last| last != &trimmed)
                                    {
                                        app.cmd_history.push(trimmed.clone());
                                    }
                                    app.history_index = None;
                                    app.input.clear();
                                    app.cursor = 0;
                                    app.add_msg(MsgRole::User, text.clone());
                                    if app.is_demo {
                                        // Simulate a response without making an API call.
                                        let response = demo_response(&text, app.active_provider);
                                        app.add_msg(MsgRole::Assistant, response.clone());
                                        app.status = "Demo response ready".to_string();
                                    } else {
                                        app.is_streaming = true;
                                        app.stream_start = Some(Instant::now());
                                        app.stream_token_count = 0;
                                        app.status_bar.mode_label = "STREAMING".into();
                                        app.status = "Sending...".to_string();
                                        let provider = app.active_provider;
                                        let _ = cmd_tx.send(AsyncCmd::SendPrompt(SendPrompt {
                                            text,
                                            provider,
                                        }));
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.insert(app.cursor, c);
                            app.cursor += c.len_utf8();
                            app.autocomplete.update(&app.input);
                        }
                        KeyCode::Tab => {
                            if app.right_menu.visible {
                                app.right_menu.next_section();
                                app.status =
                                    format!("Info: {} selected", app.right_menu.focus.label());
                            } else if app.sidebar.visible {
                                app.sidebar.next_section();
                                app.status =
                                    format!("Sidebar: {} selected", app.sidebar.focus.label());
                            } else if app.autocomplete.visible {
                                if let Some(selected) = app.autocomplete.selected_command() {
                                    let slash_cmd = format!("/{} ", selected.name);
                                    app.input = slash_cmd;
                                    app.cursor = app.input.len();
                                    app.autocomplete.visible = false;
                                    app.autocomplete.candidates.clear();
                                } else {
                                    app.autocomplete.next();
                                }
                            } else {
                                // Tab as 4-space indent
                                app.input.insert_str(app.cursor, "    ");
                                app.cursor += 4;
                            }
                        }
                        KeyCode::BackTab => {
                            if app.right_menu.visible {
                                app.right_menu.prev_section();
                                app.status =
                                    format!("Info: {} selected", app.right_menu.focus.label());
                            } else if app.sidebar.visible {
                                app.sidebar.prev_section();
                                app.status =
                                    format!("Sidebar: {} selected", app.sidebar.focus.label());
                            }
                        }
                        KeyCode::Backspace => {
                            if app.cursor > 0 {
                                let prev = app.input[..app.cursor].chars().next_back().unwrap();
                                let len = prev.len_utf8();
                                app.input.drain(app.cursor - len..app.cursor);
                                app.cursor -= len;
                            }
                            app.autocomplete.update(&app.input);
                        }
                        KeyCode::Delete => {
                            if app.cursor < app.input.len() {
                                let next = app.input[app.cursor..].chars().next().unwrap();
                                let len = next.len_utf8();
                                app.input.drain(app.cursor..app.cursor + len);
                            }
                        }
                        KeyCode::Left => {
                            if app.cursor > 0 {
                                let prev = app.input[..app.cursor].chars().next_back().unwrap();
                                app.cursor -= prev.len_utf8();
                            }
                        }
                        KeyCode::Right => {
                            if app.cursor < app.input.len() {
                                let next = app.input[app.cursor..].chars().next().unwrap();
                                app.cursor += next.len_utf8();
                            }
                        }
                        KeyCode::Home => {
                            app.cursor = 0;
                        }
                        KeyCode::End => {
                            app.cursor = app.input.len();
                        }
                        KeyCode::Up => {
                            app.scroll_up();
                        }
                        KeyCode::Down => {
                            app.scroll_down();
                        }
                        KeyCode::PageUp => {
                            for _ in 0..10 {
                                app.scroll_up();
                            }
                        }
                        KeyCode::PageDown => {
                            for _ in 0..10 {
                                app.scroll_down();
                            }
                        }
                        _ => {}
                    }
                }
                UiScreen::Config => match key.code {
                    KeyCode::Esc => {
                        app.screen = UiScreen::Chat;
                        app.status = "Ready".to_string();
                    }
                    KeyCode::Tab => {
                        let edit_val = app.config_edit.clone();
                        let field_idx = app.config_field;
                        app.apply_config_field(&edit_val, field_idx);
                        app.config_field = (app.config_field + 1) % 6;
                        app.config_edit = app.config_field_value(app.config_field);
                    }
                    KeyCode::BackTab => {
                        app.config_field = (app.config_field + 5) % 6;
                        app.config_edit = app.config_field_value(app.config_field);
                    }
                    KeyCode::Enter => {
                        let edit_val = app.config_edit.clone();
                        let field_idx = app.config_field;
                        app.apply_config_field(&edit_val, field_idx);
                        app.screen = UiScreen::Chat;
                        app.status = "Config applied".to_string();
                    }
                    KeyCode::Char(c) => {
                        app.config_edit.push(c);
                    }
                    KeyCode::Backspace => {
                        app.config_edit.pop();
                    }
                    _ => {}
                },
                UiScreen::Help => {
                    app.screen = UiScreen::Chat;
                    app.status = "Ready".to_string();
                }
                UiScreen::Dashboard => {
                    match key.code {
                        KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                            app.screen = UiScreen::Chat;
                            app.status = "Ready".to_string();
                        }
                        KeyCode::Enter => {
                            // Activate selected sidebar item
                            match app.dash_selection {
                                0 => {
                                    app.screen = UiScreen::Chat;
                                    app.status = "Switched to Chat".to_string();
                                }
                                1 => {} // already on dashboard
                                2 => {
                                    app.screen = UiScreen::Config;
                                    app.config_field = 0;
                                    app.config_edit = app.config_field_value(0);
                                    app.status =
                                        "Config mode — edit values, Tab to switch, Esc to return"
                                            .to_string();
                                }
                                3 => {
                                    app.screen = UiScreen::Help;
                                    app.status = "Help — press any key to return".to_string();
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.dash_tab = (app.dash_tab + 1) % 4;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.dash_tab = (app.dash_tab + 3) % 4;
                        }
                        KeyCode::Down | KeyCode::Tab => {
                            app.dash_selection = (app.dash_selection + 1) % 5;
                        }
                        KeyCode::Up | KeyCode::BackTab => {
                            app.dash_selection = (app.dash_selection + 4) % 5;
                        }
                        KeyCode::Char('1') => app.dash_tab = 0,
                        KeyCode::Char('2') => app.dash_tab = 1,
                        KeyCode::Char('3') => app.dash_tab = 2,
                        KeyCode::Char('4') => app.dash_tab = 3,
                        _ => {}
                    }
                }
                _ => {} // Other screens handled elsewhere
            }
        }
    }
}

// ---------------------------------------------------------------------------
// App helpers (must be free functions for borrow checker)
// ---------------------------------------------------------------------------

impl App {
    #[allow(clippy::only_used_in_recursion)]
    fn apply_config_field(&mut self, value: &str, field: usize) {
        match field {
            0 => {
                if value.to_lowercase().contains("openai") {
                    self.active_provider = ProviderKind::OpenAiCompat;
                } else {
                    self.active_provider = ProviderKind::Anthropic;
                }
            }
            1 => {
                self.config.model = value.to_string();
            }
            2 => {
                if let Ok(n) = value.parse::<u32>() {
                    self.config.max_tokens = n;
                }
            }
            3 => {
                self.config.temperature = if value.trim().is_empty() {
                    None
                } else {
                    value.parse::<f64>().ok()
                };
            }
            4 => {
                self.config.system_prompt = value.to_string();
            }
            5 => {
                let theme = Theme::from_name(value);
                self.theme_name = theme.name.to_string();
                self.theme = theme;
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tool call rendering (Hermes-inspired collapsible)
// ---------------------------------------------------------------------------

/// Render the full tool call panel as a sidebar in the chat area.
fn render_tool_panel(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Tools ")
        .border_style(Style::default().fg(t.label));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tool_calls.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  no tool calls yet",
                Style::default().fg(t.text_muted),
            ))),
            inner,
        );
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();
    for tc in &app.tool_calls {
        let elapsed = tc.elapsed_ms();
        let elapsed_str = fmt_elapsed_ms(elapsed);

        // Status symbols
        let (bullet, status_color) = match tc.status {
            ToolStatus::Pending => ("⏳", t.warn),
            ToolStatus::Running => ("⚡", t.info),
            ToolStatus::Done => ("✓", t.ok),
            ToolStatus::Error => ("✗", t.error),
        };

        // Header line
        let is_open = tc.is_open();
        let chevron = if is_open { "▼" } else { "▶" };
        let header_style = if is_open {
            Style::default().fg(t.text).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text)
        };

        let header = Line::from(vec![
            Span::styled(chevron, Style::default().fg(t.text_muted)),
            Span::raw(" "),
            Span::styled(bullet, Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(
                &tc.name,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    " {} ",
                    format_elapsed_short(tc.started_at.elapsed().as_secs())
                ),
                Style::default().fg(t.text_muted),
            ),
        ]);
        items.push(ListItem::new(header));

        // Expanded body
        if is_open {
            if !tc.context.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(
                        "  ctx:",
                        Style::default()
                            .fg(t.text_muted)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::styled(format!(" {}", tc.context), Style::default().fg(t.text)),
                ])));
            }
            if tc.status == ToolStatus::Running && !tc.preview.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ⟳", Style::default().fg(t.info)),
                    Span::styled(format!(" {}", tc.preview), Style::default().fg(t.info)),
                ])));
            }
            if !tc.inline_diff.is_empty() {
                items.push(ListItem::new(Line::from(vec![Span::styled(
                    "  diff:",
                    Style::default()
                        .fg(t.text_muted)
                        .add_modifier(Modifier::DIM),
                )])));
                for diff_line in tc.inline_diff.split('\n') {
                    let diff_style = if diff_line.starts_with('+') && !diff_line.starts_with("+++")
                    {
                        Style::default().fg(t.ok)
                    } else if diff_line.starts_with('-') && !diff_line.starts_with("---") {
                        Style::default().fg(t.error)
                    } else if diff_line.starts_with("@@") {
                        Style::default().fg(t.info)
                    } else {
                        Style::default().fg(t.text_muted)
                    };
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("    {}", diff_line),
                        diff_style,
                    ))));
                }
            }
            if !tc.summary.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ✓", Style::default().fg(t.ok)),
                    Span::styled(format!(" {}", tc.summary), Style::default().fg(t.text)),
                ])));
            }
            if !tc.error.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ✗", Style::default().fg(t.error)),
                    Span::styled(format!(" {}", tc.error), Style::default().fg(t.error)),
                ])));
            }
        }

        // Separator
        items.push(ListItem::new(Line::from(Span::styled(
            " ────────── ",
            Style::default().fg(t.border),
        ))));
    }

    frame.render_widget(List::new(items), inner);
}

fn fmt_elapsed_ms(ms: u128) -> String {
    let sec = (ms as f64) / 1000.0;
    if sec < 1.0 {
        format!("{}ms", ms)
    } else if sec < 10.0 {
        format!("{:.1}s", sec)
    } else if sec < 60.0 {
        format!("{:.0}s", sec)
    } else {
        let m = (sec as u64) / 60;
        let s = (sec as u64) % 60;
        if s == 0 {
            format!("{}m", m)
        } else {
            format!("{}m {}s", m, s)
        }
    }
}

fn format_elapsed_short(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}

// ---------------------------------------------------------------------------
// Chat sidebar (Hermes-inspired)
// ---------------------------------------------------------------------------

/// Sidebar section items.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SidebarItem {
    Status,
    Session,
    Tools,
    QuickActions,
    RightMenu,
}

impl SidebarItem {
    fn label(&self) -> &'static str {
        match self {
            Self::Status => "Connection",
            Self::Session => "Session",
            Self::Tools => "Tools",
            Self::QuickActions => "Actions",
            Self::RightMenu => "Right Menu",
        }
    }
    fn icon(&self) -> &'static str {
        match self {
            Self::Status => "🔌",
            Self::Session => "📋",
            Self::Tools => "🔧",
            Self::QuickActions => "⚡",
            Self::RightMenu => "📊",
        }
    }
}

/// Right-side menu sections (data / plan / terminal / tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RightMenuSection {
    Data,
    Plan,
    Terminal,
    Tokens,
}

impl RightMenuSection {
    fn label(&self) -> &'static str {
        match self {
            Self::Data => "Data",
            Self::Plan => "Plan",
            Self::Terminal => "Terminal",
            Self::Tokens => "Tokens",
        }
    }
    fn icon(&self) -> &'static str {
        match self {
            Self::Data => "📊",
            Self::Plan => "📋",
            Self::Terminal => "💻",
            Self::Tokens => "🔢",
        }
    }
    fn all() -> &'static [Self] {
        &[Self::Data, Self::Plan, Self::Terminal, Self::Tokens]
    }
}

#[derive(Debug, Clone)]
struct RightMenuState {
    /// Whether the right menu is visible.
    visible: bool,
    /// Currently selected section index.
    focus: RightMenuSection,
}

impl Default for RightMenuState {
    fn default() -> Self {
        Self {
            visible: true,
            focus: RightMenuSection::Data,
        }
    }
}

impl RightMenuState {
    fn next_section(&mut self) {
        let sections = RightMenuSection::all();
        let idx = sections.iter().position(|s| *s == self.focus).unwrap_or(0);
        self.focus = sections[(idx + 1) % sections.len()];
    }
    fn prev_section(&mut self) {
        let sections = RightMenuSection::all();
        let idx = sections.iter().position(|s| *s == self.focus).unwrap_or(0);
        self.focus = sections[(idx + sections.len() - 1) % sections.len()];
    }
}

#[derive(Debug, Clone)]
struct SidebarState {
    /// Whether the sidebar is visible.
    visible: bool,
    /// Currently selected section index.
    focus: SidebarItem,
    /// Sub-index within the focused section.
    sub_focus: usize,
    /// Last rendered sidebar area (for mouse click detection).
    rendered_area: Option<Rect>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            visible: true,
            focus: SidebarItem::Status,
            sub_focus: 0,
            rendered_area: None,
        }
    }
}

impl SidebarState {
    fn sections() -> &'static [SidebarItem] {
        &[
            SidebarItem::Status,
            SidebarItem::Session,
            SidebarItem::Tools,
            SidebarItem::QuickActions,
        ]
    }

    fn next_section(&mut self) {
        let sections = Self::sections();
        let idx = sections.iter().position(|s| *s == self.focus).unwrap_or(0);
        self.focus = sections[(idx + 1) % sections.len()];
        self.sub_focus = 0;
    }

    fn prev_section(&mut self) {
        let sections = Self::sections();
        let idx = sections.iter().position(|s| *s == self.focus).unwrap_or(0);
        self.focus = sections[(idx + sections.len() - 1) % sections.len()];
        self.sub_focus = 0;
    }
}

/// Render the chat sidebar (Hermes ChatSidebar-inspired).
fn render_sidebar(frame: &mut ratatui::Frame, area: Rect, app: &mut App) {
    // Record rendered area for mouse click detection
    app.sidebar.rendered_area = Some(area);
    let t = &app.theme.color;
    let sb = &app.sidebar;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Menu ")
        .border_style(Style::default().fg(t.label));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut y = inner.y;

    // ── Connection status ──────────────────────────────────────────
    let is_focused = sb.focus == SidebarItem::Status;
    render_sidebar_section_header(frame, inner, &mut y, "Connection", t, is_focused);

    let status = if app.is_demo {
        ("🔧 Demo", t.warn)
    } else if app.is_streaming {
        ("⏳ Streaming", t.info)
    } else {
        ("🟢 Connected", t.ok)
    };
    render_sidebar_item(
        frame,
        inner,
        &mut y,
        &format!("  {}", status.0),
        status.1,
        false,
    );
    render_sidebar_item(
        frame,
        inner,
        &mut y,
        &format!("  Model: {}", app.config.model),
        t.text_muted,
        false,
    );
    render_sidebar_item(
        frame,
        inner,
        &mut y,
        &format!("  Provider: {:?}", app.active_provider),
        t.text_muted,
        false,
    );

    // ── Session info ───────────────────────────────────────────────
    let is_focused = sb.focus == SidebarItem::Session;
    render_sidebar_section_header(frame, inner, &mut y, "Session", t, is_focused);

    let session_secs = app.started_at.elapsed().as_secs();
    render_sidebar_item(
        frame,
        inner,
        &mut y,
        &format!("  ⏱ {}", format_elapsed(session_secs)),
        t.text,
        false,
    );
    let showing = app.sidebar.visible;
    render_sidebar_item(
        frame,
        inner,
        &mut y,
        &format!("  💬 {} msgs", app.messages.len()),
        t.text,
        false,
    );
    render_sidebar_item(
        frame,
        inner,
        &mut y,
        &format!("  🧰 {} tools", app.tool_calls.len()),
        t.text,
        false,
    );
    let sidebar_status = if app.sidebar.visible { "ON" } else { "OFF" };
    render_sidebar_item(
        frame,
        inner,
        &mut y,
        &format!("  ◻ sidebar: {}", sidebar_status),
        t.text_muted,
        false,
    );

    // ── Tools list ─────────────────────────────────────────────────
    let is_focused = sb.focus == SidebarItem::Tools;
    render_sidebar_section_header(frame, inner, &mut y, "Tools", t, is_focused);

    for tc in &app.tool_calls {
        let (sym, color) = match tc.status {
            ToolStatus::Pending => ("⏳", t.warn),
            ToolStatus::Running => ("⚡", t.info),
            ToolStatus::Done => ("✅", t.ok),
            ToolStatus::Error => ("❌", t.error),
        };
        render_sidebar_item(
            frame,
            inner,
            &mut y,
            &format!("  {} {}", sym, tc.name),
            color,
            false,
        );
    }
    if app.tool_calls.is_empty() {
        render_sidebar_item(frame, inner, &mut y, "  (no tools)", t.text_muted, false);
    }

    // ── Quick actions ──────────────────────────────────────────────
    let is_focused = sb.focus == SidebarItem::QuickActions;
    render_sidebar_section_header(frame, inner, &mut y, "Actions", t, is_focused);

    let actions = [
        ("Ctrl+T", "Tool panel"),
        ("Ctrl+L", "Toggle tool"),
        ("Ctrl+D", "Dashboard"),
        ("Ctrl+K", "Settings"),
        ("Ctrl+S", "Sidebar"),
        ("F1", "Help"),
    ];
    for (key, desc) in &actions {
        let text = format!("  {}  {}", key, desc);
        render_sidebar_item(frame, inner, &mut y, &text, t.text_muted, false);
    }
}

fn render_sidebar_section_header(
    frame: &mut ratatui::Frame,
    inner: Rect,
    y: &mut u16,
    label: &str,
    t: &ThemeColors,
    focused: bool,
) {
    let area = frame.area();
    if *y >= area.bottom() {
        return;
    }
    let style = if focused {
        Style::default()
            .fg(t.primary)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().fg(t.primary).add_modifier(Modifier::BOLD)
    };
    let text = format!(" {} ", label);
    let buf = frame.buffer_mut();
    for (cx, ch) in text.chars().enumerate() {
        let cell_x = inner.x + cx as u16;
        if cell_x < area.width {
            let cell = buf.cell_mut((cell_x, *y)).unwrap();
            cell.set_char(ch);
            cell.set_style(style);
        }
    }
    *y += 1;
}

fn render_sidebar_item(
    frame: &mut ratatui::Frame,
    inner: Rect,
    y: &mut u16,
    text: &str,
    color: Color,
    _selected: bool,
) {
    let area = frame.area();
    if *y >= area.bottom() {
        return;
    }
    let buf = frame.buffer_mut();
    for (cx, ch) in text.chars().enumerate() {
        let cell_x = inner.x + cx as u16;
        if cell_x < area.width {
            let cell = buf.cell_mut((cell_x, *y)).unwrap();
            cell.set_char(ch);
            cell.set_style(Style::default().fg(color));
        }
    }
    *y += 1;
}

/// Toggle expanded state for the last active tool call.
fn toggle_last_tool(app: &mut App) {
    if let Some(last) = app.tool_calls.last_mut() {
        last.expanded = Some(!last.is_open());
    }
}

/// Render the right-side menu (data / plan / terminal / tokens).
fn render_right_menu(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let rm = &app.right_menu;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Info ")
        .border_style(Style::default().fg(t.label));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut y = inner.y;

    // ── Tab bar ──────────────────────────────────────────────────
    let tabs_str: String = RightMenuSection::all()
        .iter()
        .map(|s| {
            let marker = if *s == rm.focus { "●" } else { "○" };
            format!("{}{} ", marker, s.label())
        })
        .collect::<Vec<_>>()
        .join(" ");
    let tabs_style = Style::default().fg(t.text);
    if y < inner.bottom() {
        let buf = frame.buffer_mut();
        for (cx, ch) in tabs_str.chars().enumerate() {
            let cell_x = inner.x + cx as u16;
            if cell_x < area.width {
                if let Some(cell) = buf.cell_mut((cell_x, y)) {
                    cell.set_char(ch);
                    cell.set_style(tabs_style);
                }
            }
        }
        y += 1;
    }

    // Separator
    if y < inner.bottom() {
        let buf = frame.buffer_mut();
        for cx in 0..inner.width.saturating_sub(1) {
            let cell_x = inner.x + cx;
            if let Some(cell) = buf.cell_mut((cell_x, y)) {
                cell.set_char('─');
                cell.set_style(Style::default().fg(t.border));
            }
        }
        y += 1;
    }

    // ── Section content ──────────────────────────────────────────
    match rm.focus {
        RightMenuSection::Data => {
            let items = [
                format!("Model : {}", app.config.model),
                format!("Provider : {:?}", app.active_provider),
                format!("Messages : {}", app.messages.len()),
                format!("Tools : {}", app.tool_calls.len()),
                format!(
                    "Streaming : {}",
                    if app.is_streaming { "yes" } else { "no" }
                ),
                format!("Demo mode : {}", if app.is_demo { "yes" } else { "no" }),
            ];
            for item in &items {
                if y >= inner.bottom() {
                    break;
                }
                render_sidebar_item(
                    frame,
                    inner,
                    &mut y,
                    &format!("  {}", item),
                    t.text_muted,
                    false,
                );
            }
        }
        RightMenuSection::Plan => {
            let plan_path = "C:\\Drive\\Cargo\\OpenCode_Rs\\PLAN.md";
            if let Ok(content) = std::fs::read_to_string(plan_path) {
                let lines: Vec<&str> = content.lines().collect();
                let max_lines = inner.height.saturating_sub(3) as usize;
                let start = if lines.len() > max_lines {
                    lines.len() - max_lines
                } else {
                    0
                };
                for line in &lines[start..] {
                    if y >= inner.bottom() {
                        break;
                    }
                    let truncated = if line.len() > (inner.width.saturating_sub(3) as usize) {
                        format!("{}…", &line[..(inner.width.saturating_sub(4) as usize)])
                    } else {
                        line.to_string()
                    };
                    render_sidebar_item(
                        frame,
                        inner,
                        &mut y,
                        &format!("  {}", truncated),
                        t.text,
                        false,
                    );
                }
            } else {
                render_sidebar_item(
                    frame,
                    inner,
                    &mut y,
                    "  (Plan file not found)",
                    t.text_muted,
                    false,
                );
            }
        }
        RightMenuSection::Terminal => {
            render_sidebar_item(
                frame,
                inner,
                &mut y,
                "  Terminal (not yet available)",
                t.text_muted,
                false,
            );
            y += 1;
            render_sidebar_item(
                frame,
                inner,
                &mut y,
                "  Use the side panel terminal",
                t.text_muted,
                false,
            );
            render_sidebar_item(
                frame,
                inner,
                &mut y,
                "  for shell commands.",
                t.text_muted,
                false,
            );
        }
        RightMenuSection::Tokens => {
            // Token usage (from status_bar tracking)
            let token_used = app.status_bar.token_used;
            let token_limit = app.status_bar.token_limit;
            let total_chars: usize = app.messages.iter().map(|m| m.text.len()).sum();
            let est_tokens = total_chars / 4;

            let pct = if token_limit > 0 {
                (token_used as f64 / token_limit as f64 * 100.0) as usize
            } else {
                0
            };

            // Speed estimate (tokens/sec on last response)
            let speed_hint = if app.is_streaming {
                if let Some(start) = app.stream_start {
                    let elapsed = start.elapsed().as_secs_f64();
                    if elapsed > 0.5 {
                        format!("{:.1} tok/s", app.stream_token_count as f64 / elapsed)
                    } else {
                        "streaming…".to_string()
                    }
                } else {
                    "streaming…".to_string()
                }
            } else if app.last_tokens_per_sec > 0.0 {
                format!("{:.1} tok/s", app.last_tokens_per_sec)
            } else {
                "—".to_string()
            };

            let items = [
                format!("Total chars  : {}", total_chars),
                format!("Est. tokens  : ≈{}", est_tokens),
                format!("Session used : {}", token_used),
                format!("Context cap  : {}", token_limit),
                format!("Usage        : {}%", pct),
                format!("Input chars  : {}", app.input.len()),
                format!("Speed        : {}", speed_hint),
            ];
            for item in &items {
                if y >= inner.bottom() {
                    break;
                }
                render_sidebar_item(
                    frame,
                    inner,
                    &mut y,
                    &format!("  {}", item),
                    t.text_muted,
                    false,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(frame: &mut ratatui::Frame, app: &mut App) {
    // Let the compositor render all layers with the App context.
    // The base ChatLayer renders the full chat, and overlay layers
    // (Config, Help) render their overlays on top.
    // SAFETY: compositor does not store the context after the call.
    let app_ptr = &mut *app as *mut App;
    let ctx: &mut dyn Any = unsafe { &mut *app_ptr };
    app.compositor.render_with_context(frame, frame.area(), ctx);
}

fn render_chat(frame: &mut ratatui::Frame, app: &mut App) {
    let area = frame.area();
    let t = &app.theme.color;

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(3),    // messages area (may be split horizontally and/or vertically)
            Constraint::Length(1), // AI mode / model indicator
            Constraint::Length(3), // input
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // ── If editor is visible, split messages row vertically: editor (top) | chat (bottom) ──
    let msg_row = if app.editor.visible {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // editor
                Constraint::Percentage(50), // chat messages
            ])
            .split(vertical_chunks[1]);
        (split[0], Some(split[1]))
    } else {
        (vertical_chunks[1], None)
    };

    let editor_area = msg_row.0;
    let chat_area = msg_row.1.unwrap_or(vertical_chunks[1]);

    // ── Split chat area into sidebar + (messages [+ tool panel]) + right_menu ──
    let has_sidebar = app.sidebar.visible;
    let has_right_menu = app.right_menu.visible;
    let has_tool_panel = !app.tool_calls.is_empty() && app.tool_panel_visible;

    // First split: sidebar | center | right_menu
    let mut constraints = Vec::new();
    if has_sidebar {
        constraints.push(Constraint::Length(22)); // sidebar
    }
    constraints.push(Constraint::Min(30)); // center content
    if has_right_menu {
        constraints.push(Constraint::Length(24)); // right menu
    }
    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(chat_area);

    let mut idx = 0usize;
    let sidebar_area = if has_sidebar {
        idx += 1;
        Some(h[idx - 1])
    } else {
        None
    };
    let center_area = h[idx];
    let right_menu_area = if has_right_menu {
        idx += 1;
        Some(h[idx])
    } else {
        None
    };

    // Then split center area: messages | tool panel
    let (msg_area, tool_area) = if has_tool_panel {
        let h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65), // messages
                Constraint::Percentage(35), // tool panel
            ])
            .split(center_area);
        (h[0], Some(h[1]))
    } else {
        (center_area, None)
    };

    // ── Title bar ──────────────────────────────────────────────────
    let focus_badge = if app.editor.visible {
        match app.panel_focus {
            PanelFocus::Editor => " [EDITOR]",
            PanelFocus::Chat => " [CHAT]",
        }
    } else {
        ""
    };
    let title = format!(
        " OpenCode TUI [{}] — {:?}{} ",
        app.config.model, app.active_provider, focus_badge,
    );
    let title_style = Style::default()
        .fg(t.primary)
        .bg(t.status_bg)
        .add_modifier(Modifier::BOLD);
    let title_spans = Line::from(Span::styled(&title, title_style));
    frame.render_widget(Paragraph::new(title_spans), vertical_chunks[0]);

    // ── Editor area ─────────────────────────────────────────────────
    if app.editor.visible {
        let md_theme = MarkdownTheme {
            border: app.theme.color.border,
            primary: app.theme.color.primary,
            accent: app.theme.color.accent,
            text: app.theme.color.text,
            text_muted: app.theme.color.text_muted,
            status_bg: app.theme.color.status_bg,
            ok: app.theme.color.ok,
            error: app.theme.color.error,
        };
        app.editor.render(frame, editor_area, &md_theme);

        // Ensure cursor is visible
        let view_height = editor_area.height.saturating_sub(2) as usize;
        app.editor.ensure_cursor_visible(view_height);
    }

    // ── Messages area ──────────────────────────────────────────────
    let inner = msg_area;
    let width = inner.width as usize;

    let mut items: Vec<ListItem> = Vec::new();
    for msg in &app.messages {
        let label = msg.role_label();
        let color = msg.role_color();
        let style = Style::default().fg(color).add_modifier(Modifier::BOLD);
        let label_span = Span::styled(format!("[{}] ", label), style);

        let lines = render_markdown(&msg.text, t);
        let indent_text = " ".repeat(label.len() + 3);

        if lines.is_empty() {
            items.push(ListItem::new(Line::from(vec![
                label_span,
                Span::raw("(empty)"),
            ])));
        } else {
            // First line has label
            if let Some(first) = lines.first() {
                let mut first_spans = vec![label_span];
                for span in &first.spans {
                    first_spans.push(Span::styled(
                        wrap_line(&span.content, width.saturating_sub(4 + label.len() + 3)),
                        span.style,
                    ));
                }
                items.push(ListItem::new(Line::from(first_spans)));
            }
            // Subsequent lines are indented
            for line in &lines[1..] {
                if line.spans.is_empty()
                    || (line.spans.len() == 1 && line.spans[0].content.is_empty())
                {
                    items.push(ListItem::new(Line::from(Span::raw(""))));
                } else {
                    let mut indented_spans = vec![Span::raw(indent_text.clone())];
                    for span in &line.spans {
                        indented_spans.push(Span::styled(
                            wrap_line(&span.content, width.saturating_sub(4)),
                            span.style,
                        ));
                    }
                    items.push(ListItem::new(Line::from(indented_spans)));
                }
            }
        }
        items.push(ListItem::new(Line::from("")));
    }

    // Streaming indicator — show incremental text if available
    if app.is_streaming {
        // Throttle: skip rendering if less than 50ms since last render
        let now = Instant::now();
        let should_render = now.duration_since(app.last_stream_render).as_millis() >= 50;

        if app.streaming_text.is_empty() {
            let thinking_text = "⚡ AI is thinking…";
            let mut shimmered =
                shimmer_spans(thinking_text, app.status_bar.tick, t.accent, Color::White);
            // Prepend bracket spans
            shimmered.insert(0, Span::styled("[", Style::default().fg(t.accent)));
            shimmered.push(Span::styled("]", Style::default().fg(t.accent)));
            items.push(ListItem::new(Line::from(shimmered)));
        } else {
            // Use pulldown_cmark-based renderer for streaming text
            let source = if should_render {
                // Commit complete lines and render
                if let Some(committed) = app.markdown_stream.commit_complete_source() {
                    // Re-render with committed source (still add incomplete tail)
                    let full_source = app.markdown_stream.full_buffer();
                    app.last_stream_render = now;
                    app.markdown_renderer.render(full_source)
                } else {
                    // No new complete lines — re-render with full buffer (tail included)
                    render_markdown_simple(&app.streaming_text, t)
                }
            } else {
                // Use simple render for fast path (no pulldown_cmark call)
                render_markdown_simple(&app.streaming_text, t)
            };
            let lines = source;
            let indent_text = " ".repeat("[Assistant] ".len() + 3);
            // First line with label
            if let Some(first) = lines.first() {
                let label_span = Span::styled(
                    "[Assistant] ",
                    Style::default()
                        .fg(app.theme.color.accent)
                        .add_modifier(Modifier::BOLD),
                );
                let mut first_spans = vec![label_span];
                for span in &first.spans {
                    first_spans.push(Span::styled(
                        wrap_line(
                            &span.content,
                            width.saturating_sub(4 + "[Assistant] ".len() + 3),
                        ),
                        span.style,
                    ));
                }
                items.push(ListItem::new(Line::from(first_spans)));
            }
            for line in &lines[1..] {
                if line.spans.is_empty()
                    || (line.spans.len() == 1 && line.spans[0].content.is_empty())
                {
                    items.push(ListItem::new(Line::from(Span::raw(""))));
                } else {
                    let mut indented = vec![Span::raw(indent_text.clone())];
                    for span in &line.spans {
                        indented.push(Span::styled(
                            wrap_line(&span.content, width.saturating_sub(4)),
                            span.style,
                        ));
                    }
                    items.push(ListItem::new(Line::from(indented)));
                }
            }
            // Add blinking cursor indicator
            items.push(ListItem::new(Line::from(Span::styled(
                "▊",
                Style::default()
                    .fg(t.accent)
                    .add_modifier(Modifier::SLOW_BLINK),
            ))));
        }
    }

    // Calculate visible range
    let visible_count = inner.height.saturating_sub(1) as usize;
    let total = items.len();
    if total > visible_count {
        let scroll = app.scroll.min(total.saturating_sub(visible_count));
        let end = total.saturating_sub(scroll);
        let start = end.saturating_sub(visible_count);
        let visible = if start < end {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };
        let list = List::new(visible);
        frame.render_widget(list, inner);
    } else {
        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    // ── AI mode / model indicator bar ───────────────────────────────
    let model_area = vertical_chunks[2];
    let mode_name = if app.is_streaming {
        "streaming"
    } else {
        "ready"
    };
    let ai_mode_str = format!("{} {}", app.ai_mode.icon(), app.ai_mode.label());
    let model_label = format!(
        "  {}  │  {}  │  {}  │  {:?} ",
        ai_mode_str, app.config.model, mode_name, app.active_provider,
    );
    let model_style = Style::default().fg(t.text_muted).bg(t.model_bar_bg);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(&model_label, model_style))),
        model_area,
    );

    // ── Input area ─────────────────────────────────────────────────
    let input_area = vertical_chunks[3];

    // Autocomplete popup (shown above input when visible)
    let (input_block_area, ac_area) = if app.autocomplete.visible {
        let ac_height = (app.autocomplete.candidates.len().min(6) as u16) + 2; // borders
        let ac_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(ac_height)])
            .split(input_area);
        (ac_split[1], Some((ac_split[0], &app.autocomplete)))
    } else {
        (input_area, None)
    };

    let input_title = if app.multiline_mode {
        " Prompt [ML] (Enter: newline, Alt+Enter: send, Ctrl+M: toggle) "
    } else {
        " Prompt (/help for commands, Enter: send, Ctrl+K: config, Ctrl+Q: quit) "
    };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(input_title)
        .border_style(Style::default().fg(if app.multiline_mode { t.warn } else { t.border }));
    let input_inner = input_block.inner(input_block_area);
    frame.render_widget(input_block, input_block_area);

    let input_style = if app.input.is_empty() && !app.is_streaming {
        Style::default().fg(t.text_muted)
    } else {
        Style::default().fg(t.text)
    };
    let display_text = if app.is_streaming {
        "(awaiting response...)"
    } else if app.input.is_empty() {
        "Type your message or /command here..."
    } else {
        &app.input
    };
    let input_paragraph = if app.is_streaming && app.input.is_empty() {
        // Apply shimmer to awaiting response text
        let shimmered = shimmer_spans(
            "awaiting response…",
            app.status_bar.tick,
            t.text_muted,
            t.accent,
        );
        let mut spans = vec![Span::styled("(", Style::default().fg(t.text_muted))];
        spans.extend(shimmered);
        spans.push(Span::styled(")", Style::default().fg(t.text_muted)));
        Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false })
    } else {
        Paragraph::new(display_text)
            .style(input_style)
            .wrap(Wrap { trim: false })
    };
    frame.render_widget(input_paragraph, input_inner);

    // Render autocomplete popup
    if let Some((ac_area, ac)) = ac_area {
        let (title, items) = if ac.mention_mode {
            let title = " @Mentions ";
            let items: Vec<ListItem> = ac
                .mention_candidates
                .iter()
                .enumerate()
                .map(|(idx, mention)| {
                    let is_selected = idx == ac.mention_selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(t.text)
                            .bg(t.status_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(t.text)
                    };
                    let prefix = if is_selected { "▸ " } else { "  " };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{}{}", prefix, mention.prefix()), style),
                        Span::styled(
                            format!(" — {}", mention.description()),
                            Style::default().fg(t.text_muted),
                        ),
                    ]))
                })
                .collect();
            (title, items)
        } else {
            let title = " Commands ";
            let items: Vec<ListItem> = ac
                .candidates
                .iter()
                .enumerate()
                .map(|(idx, &cmd_idx)| {
                    let cmd = &BUILTIN_COMMANDS[cmd_idx];
                    let is_selected = idx == ac.selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(t.text)
                            .bg(t.status_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(t.text)
                    };
                    let prefix = if is_selected { "▸ " } else { "  " };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{}/{}", prefix, cmd.name), style),
                        Span::styled(
                            format!(" — {}", cmd.description),
                            Style::default().fg(t.text_muted),
                        ),
                    ]))
                })
                .collect();
            (title, items)
        };

        let ac_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(t.label));
        let ac_inner = ac_block.inner(ac_area);
        frame.render_widget(ac_block, ac_area);
        frame.render_widget(List::new(items), ac_inner);
    }

    // Cursor
    if !app.is_streaming {
        let cx = input_inner.x + app.cursor as u16;
        let cy = input_inner.y;
        if cx < input_inner.right() {
            frame.set_cursor_position(ratatui::prelude::Position::new(cx, cy));
        }
    }

    // ── Rich status bar (3-section: left / center / right) ──────────
    let sb = &app.status_bar;
    let left = sb.left_side(&app.config.model, &format!("{:?}", app.active_provider));
    let center = sb.center_side();
    let right = sb.right_side();

    let (status_fg, bg) = if sb.busy {
        (t.warn, t.status_busy_bg)
    } else if app.is_demo {
        (t.warn, t.status_demo_bg)
    } else {
        (t.status_fg, t.status_bg)
    };

    let status_style = Style::default()
        .fg(status_fg)
        .bg(bg)
        .add_modifier(Modifier::BOLD);

    // Build 3-section status line with proper spacing
    let bar_width = vertical_chunks[4].width as usize;
    let left_w = left.chars().count();
    let center_w = center.chars().count();
    let right_w = right.chars().count();

    let status_line = if sb.busy {
        let mut spans = vec![];
        spans.push(Span::styled(left.clone(), status_style));
        // Shimmer on center message
        let center_spans = shimmer_spans(&center, sb.tick, t.status_fg, bg);
        let center_rendered_w: usize = center_spans.iter().map(|s| s.width()).sum();
        let remaining = bar_width.saturating_sub(left_w + center_rendered_w + right_w);
        spans.extend(center_spans);
        if remaining > 0 {
            spans.push(Span::styled(" ".repeat(remaining), status_style));
        }
        if !right.is_empty() {
            spans.push(Span::styled(right.clone(), status_style));
        }
        Line::from(spans)
    } else {
        // Normal: left + padded center + right
        let padding = bar_width.saturating_sub(left_w + center_w + right_w);
        let pad_left = padding / 2;
        let pad_right = padding - pad_left;
        let text = format!(
            "{}{}{}{}{}",
            left,
            " ".repeat(pad_left),
            center,
            " ".repeat(pad_right),
            right
        );
        Line::from(Span::styled(text, status_style))
    };

    frame.render_widget(Paragraph::new(status_line), vertical_chunks[4]);

    // ── Tool panel sidebar (Hermes-inspired) ──────────────────────
    if let Some(tool_area) = tool_area {
        render_tool_panel(frame, tool_area, app);
    }

    // ── Chat sidebar (Hermes-inspired) ────────────────────────────
    if let Some(sidebar_area) = sidebar_area {
        render_sidebar(frame, sidebar_area, app);
    }

    // ── Right menu (data / plan / terminal / tokens) ─────────────
    if let Some(right_menu_area) = right_menu_area {
        render_right_menu(frame, right_menu_area, app);
    }
}

// ---------------------------------------------------------------------------
// Dashboard (shadcn-inspired sidebar + metrics + chart + table)
// ---------------------------------------------------------------------------

fn render_dashboard(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    let t = &app.theme.color;

    let sidebar_width = 22.min(area.width / 5);
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_width), Constraint::Min(40)])
        .split(area);

    // ── Sidebar ────────────────────────────────────────────────────
    render_dash_sidebar(frame, main_chunks[0], app);

    // ── Content ────────────────────────────────────────────────────
    let content_area = main_chunks[1];
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // breadcrumb
            Constraint::Length(2), // tabs
            Constraint::Length(6), // metric cards row
            Constraint::Min(5),    // main panel (varies by tab)
            Constraint::Length(1), // footer status
        ])
        .split(content_area);

    // Breadcrumb
    let breadcrumb = Line::from(vec![
        Span::styled(
            format!(" {} ", format_elapsed(app.started_at.elapsed().as_secs())),
            Style::default().fg(t.text_muted),
        ),
        Span::styled(" / ", Style::default().fg(t.border)),
        Span::styled(
            " Dashboard ",
            Style::default().fg(t.primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" / ", Style::default().fg(t.border)),
        Span::styled(
            ["Overview", "Performance", "Tasks", "Network"][app.dash_tab],
            Style::default().fg(t.accent),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(breadcrumb).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(t.border)),
        ),
        content_chunks[0],
    );

    // Tabs
    let tab_labels = ["[1]Overview", "[2]Perf", "[3]Tasks", "[4]Net"];
    let tab_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(tab_labels.iter().map(|_| Constraint::Length(16)))
        .split(content_chunks[1]);

    for (i, label) in tab_labels.iter().enumerate() {
        let is_active = i == app.dash_tab;
        let tab_style = if is_active {
            Style::default()
                .fg(t.text)
                .bg(t.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_muted)
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!(" {} ", label), tab_style))),
            tab_chunks[i],
        );
    }

    // Metric cards (always visible)
    render_dash_metrics(frame, content_chunks[2], app);

    // Main panel content based on tab
    let panel_area = content_chunks[3];
    match app.dash_tab {
        0 => render_overview_panel(frame, panel_area, app),
        1 => render_perf_panel(frame, panel_area, app),
        2 => render_tasks_panel(frame, panel_area, app),
        3 => render_network_panel(frame, panel_area, app),
        _ => {}
    }

    // Footer status
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(
                " ←/→ or h/l: tabs  ↑/↓: sidebar  Enter:go  Ctrl+D:back  {}msgs",
                app.messages.len()
            ),
            Style::default()
                .fg(t.text_muted)
                .add_modifier(Modifier::DIM),
        ))),
        content_chunks[4],
    );
}

// ── Sidebar ──────────────────────────────────────────────────────────

fn render_dash_sidebar(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let sb_block = Block::default()
        .borders(Borders::ALL)
        .title(" OpenCode ")
        .border_style(Style::default().fg(t.label));
    let sb_inner = sb_block.inner(area);
    frame.render_widget(sb_block, area);

    let mut lines: Vec<ListItem> = Vec::new();
    lines.push(ListItem::new(Line::from("")));
    lines.push(ListItem::new(Line::from(vec![
        Span::styled(" ⏱ ", Style::default().fg(t.text_muted)),
        Span::styled(
            format_elapsed(app.started_at.elapsed().as_secs()),
            Style::default().fg(t.text_muted),
        ),
    ])));
    lines.push(ListItem::new(Line::from(vec![
        Span::styled(" 💬 ", Style::default().fg(t.text_muted)),
        Span::styled(
            format!("{} msgs", app.messages.len()),
            Style::default().fg(t.text_muted),
        ),
    ])));
    lines.push(ListItem::new(Line::from("")));
    lines.push(ListItem::new(Line::from(Span::styled(
        " NAVIGATION ",
        Style::default().fg(t.label).add_modifier(Modifier::DIM),
    ))));
    lines.push(ListItem::new(Line::from("")));

    let nav = ["Chat", "Dashboard", "Config", "Help"];
    for (i, item) in nav.iter().enumerate() {
        let is_selected = i == app.dash_selection;
        let s = if is_selected {
            Style::default()
                .fg(t.text)
                .bg(t.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text)
        };
        lines.push(ListItem::new(Line::from(Span::styled(
            format!(" {} {}", if is_selected { "▸" } else { " " }, item),
            s,
        ))));
    }
    lines.push(ListItem::new(Line::from("")));
    lines.push(ListItem::new(Line::from(Span::styled(
        " Ctrl+D: back ",
        Style::default()
            .fg(t.text_muted)
            .add_modifier(Modifier::DIM),
    ))));

    frame.render_widget(List::new(lines), sb_inner);
}

// ── Metric cards ─────────────────────────────────────────────────────

fn render_dash_metrics(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let metric_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ])
        .split(area);

    let metrics = [
        ("Tokens Used", "847.2K", "+12%", t.primary, t.status_bg),
        ("Avg Latency", "1.2s", "-8%", t.ok, t.status_bg),
        ("Tools Called", "1,423", "+3.2%", t.info, t.status_bg),
        ("Sessions", "38", "+18%", t.accent, t.status_bg),
        ("Error Rate", "0.4%", "-2%", t.warn, t.status_bg),
    ];

    for (i, (title, value, change, color, _bg)) in metrics.iter().enumerate() {
        let card = metric_chunks[i];
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border));
        let inner = block.inner(card);
        frame.render_widget(block, card);

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(1),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                title.to_string(),
                Style::default().fg(t.label).add_modifier(Modifier::DIM),
            ))),
            inner_chunks[0],
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                value.to_string(),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ))),
            inner_chunks[1],
        );
        let change_color = if change.starts_with('+') {
            t.ok
        } else {
            t.error
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("  {} vs last period", change),
                Style::default().fg(change_color),
            ))),
            inner_chunks[2],
        );
    }
}

// ── Overview tab ─────────────────────────────────────────────────────

fn render_overview_panel(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
        .split(area);

    // Left: bar chart
    let left_area = panes[0];
    let chart_block = Block::default()
        .borders(Borders::ALL)
        .title(" Activity Overview ")
        .border_style(Style::default().fg(t.border));
    let chart_inner = chart_block.inner(left_area);
    frame.render_widget(chart_block, left_area);
    render_bar_chart(frame, chart_inner, app);

    // Right: mini progress bars and summary
    let right_area = panes[1];
    let summary_block = Block::default()
        .borders(Borders::ALL)
        .title(" System Status ")
        .border_style(Style::default().fg(t.border));
    let summary_inner = summary_block.inner(right_area);
    frame.render_widget(summary_block, right_area);

    let status_items = [
        ("API  ", 100, t.ok),
        ("Tools", 92, t.ok),
        ("Cache", 78, t.accent),
        ("Queue", 45, t.warn),
        ("Sync ", 23, t.error),
    ];

    let gw = summary_inner.width.saturating_sub(12) as usize;
    for (i, (label, pct, color)) in status_items.iter().enumerate() {
        let y = summary_inner.y + i as u16;
        if y >= summary_inner.bottom() {
            break;
        }
        let bar_len = (*pct as usize) * gw / 100;
        let bar = "█".repeat(bar_len);
        let empty = "░".repeat(gw.saturating_sub(bar_len));
        let line = format!(
            " {} {} [{}{}] {:>3}%",
            label,
            emoji_for_pct(*pct),
            bar,
            empty,
            pct
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(line, Style::default().fg(*color)))),
            Rect {
                x: summary_inner.x,
                y,
                width: summary_inner.width,
                height: 1,
            },
        );
    }
}

fn emoji_for_pct(pct: u16) -> &'static str {
    if pct >= 90 {
        "🟢"
    } else if pct >= 60 {
        "🟡"
    } else if pct >= 30 {
        "🟠"
    } else {
        "🔴"
    }
}

// ── Performance tab ──────────────────────────────────────────────────

fn render_perf_panel(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    // Left: Performance gauges
    let left = panes[0];
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title(" Resource Monitor ")
        .border_style(Style::default().fg(t.border));
    let left_inner = left_block.inner(left);
    frame.render_widget(left_block, left);

    let gauges = [
        ("CPU Usage   ", 67, t.info),
        ("Memory      ", 43, t.ok),
        ("Disk I/O    ", 82, t.warn),
        ("Network     ", 29, t.ok),
    ];
    let gw = left_inner.width.saturating_sub(14) as usize;
    for (i, (label, pct, color)) in gauges.iter().enumerate() {
        let y = left_inner.y + i as u16 * 2;
        if y >= left_inner.bottom() {
            break;
        }
        let bar_len = (*pct as usize) * gw / 100;
        let bar = "━".repeat(bar_len);
        let empty = "─".repeat(gw.saturating_sub(bar_len));
        let line = format!(" {} {}% |{}{}|", label, pct, bar, empty);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(line, Style::default().fg(*color)))),
            Rect {
                x: left_inner.x,
                y,
                width: left_inner.width,
                height: 1,
            },
        );
        // Sparkline underneath
        let spark = sparkline(*pct, left_inner.width.saturating_sub(2) as usize);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(spark, Style::default().fg(*color)))),
            Rect {
                x: left_inner.x,
                y: y + 1,
                width: left_inner.width,
                height: 1,
            },
        );
    }

    // Right: recent activity / latency
    let right = panes[1];
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title(" Response Times ")
        .border_style(Style::default().fg(t.border));
    let right_inner = right_block.inner(right);
    frame.render_widget(right_block, right);

    let latency_data = [
        ("Anthropic", "1.2s", "/min", t.primary),
        ("OpenAI   ", "2.8s", "+23%", t.warn),
        ("Local    ", "0.4s", "-5%", t.ok),
        ("Cache    ", "0.1s", "-45%", t.ok),
    ];
    for (i, (provider, p95, trend, color)) in latency_data.iter().enumerate() {
        let y = right_inner.y + i as u16 * 2;
        if y >= right_inner.bottom() {
            break;
        }
        let trend_color = if trend.starts_with('+') {
            t.error
        } else {
            t.ok
        };
        let line = Line::from(vec![
            Span::styled(format!(" {}", provider), Style::default().fg(*color)),
            Span::styled(
                format!("  P95: {}", p95),
                Style::default().fg(t.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" ({})", trend), Style::default().fg(trend_color)),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect {
                x: right_inner.x,
                y,
                width: right_inner.width,
                height: 1,
            },
        );

        let bar_len = match *p95 {
            "1.2s" => 20,
            "2.8s" => 30,
            "0.4s" => 8,
            "0.1s" => 2,
            _ => 10,
        };
        let bar = "▂▄▆██".repeat(bar_len / 5 + 1);
        let spark = bar
            .chars()
            .take(right_inner.width.saturating_sub(2) as usize)
            .collect::<String>();
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(spark, Style::default().fg(*color)))),
            Rect {
                x: right_inner.x,
                y: y + 1,
                width: right_inner.width,
                height: 1,
            },
        );
    }
}

/// Generate a fake sparkline string showing variation around a base value.
fn sparkline(base: u16, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let blocks = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let n = width.min(40);
    (0..n)
        .map(|i| {
            let variation = ((seed as u64).wrapping_add(i as u64 * 7919) % 8) as u16;
            let val = base
                .saturating_add(variation.saturating_sub(4) * 5)
                .min(100);
            let idx = (val * 7 / 100) as usize;
            blocks[idx.min(7)]
        })
        .collect()
}

// ── Tasks tab ────────────────────────────────────────────────────────

fn render_tasks_panel(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Task History ")
        .border_style(Style::default().fg(t.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let headers = ["   ID", "Type", "Model", "Status", "Duration", "Cost"];
    let rows = [
        (
            true,
            "#241",
            "Code Review",
            "claude-sonnet",
            "Done",
            "12.3s",
            "$0.09",
            t.ok,
        ),
        (
            false,
            "#240",
            "Chat",
            "claude-sonnet",
            "Done",
            "8.1s",
            "$0.04",
            t.ok,
        ),
        (
            true,
            "#239",
            "Code Gen",
            "claude-haiku",
            "Running",
            "23.4s",
            "$0.02",
            t.accent,
        ),
        (
            false,
            "#238",
            "Code Review",
            "gpt-4o",
            "Done",
            "15.7s",
            "$0.12",
            t.ok,
        ),
        (
            true,
            "#237",
            "Chat",
            "claude-sonnet",
            "Failed",
            "0.9s",
            "$0.00",
            t.error,
        ),
        (
            false,
            "#236",
            "Code Gen",
            "claude-haiku",
            "Done",
            "5.2s",
            "$0.01",
            t.ok,
        ),
        (
            true,
            "#235",
            "Chat",
            "claude-sonnet",
            "Done",
            "3.8s",
            "$0.02",
            t.ok,
        ),
        (
            false,
            "#234",
            "Code Review",
            "claude-opus",
            "Done",
            "42.1s",
            "$0.45",
            t.ok,
        ),
    ];

    let hdr_line = Line::from(
        headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                Span::styled(
                    format!("{}{}", if i > 0 { " │ " } else { " " }, h),
                    Style::default().fg(t.label).add_modifier(Modifier::BOLD),
                )
            })
            .collect::<Vec<_>>(),
    );
    frame.render_widget(Paragraph::new(hdr_line), inner);

    let row_start = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(std::iter::repeat(Constraint::Length(1)))
        .split(row_start);

    for (i, (is_odd, id, r#type, model, status, duration, cost, status_color)) in
        rows.iter().enumerate()
    {
        if i >= row_chunks.len() {
            break;
        }
        let status_icon = match *status {
            "Done" => "✅",
            "Running" | "In Progress" => "⟳",
            "Failed" => "❌",
            _ => "  ",
        };
        let mx = model.chars().take(14).collect::<String>();
        let cell_texts = [
            id.to_string(),
            r#type.chars().take(10).collect::<String>(),
            mx,
            format!("{} {}", status_icon, status),
            duration.to_string(),
            cost.to_string(),
        ];
        let bg = if *is_odd { t.status_bg } else { Color::Reset };
        let row_spans: Vec<Span> = cell_texts
            .iter()
            .enumerate()
            .map(|(ci, cell)| {
                let c = if ci == 3 { *status_color } else { t.text };
                Span::styled(
                    format!("{}{}", if ci > 0 { " │ " } else { " " }, cell),
                    Style::default().fg(c),
                )
            })
            .collect();
        frame.render_widget(
            Paragraph::new(Line::from(row_spans)).style(Style::default().bg(bg)),
            row_chunks[i],
        );
    }
}

// ── Network tab ─────────────────────────────────────────────────────

fn render_network_panel(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Activity Heatmap (Last 30 days) ")
        .border_style(Style::default().fg(t.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calendar-style heatmap: 5 weeks x 7 days
    let days_per_week = 7;
    let total_days = 28;
    let weeks = total_days / days_per_week;

    let heatmap_top = inner.y + 1;
    let day_width = 4usize;
    let start_x = inner.x + 1;

    // Day headers
    let day_labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    for (di, label) in day_labels.iter().enumerate() {
        let x = start_x + (di * day_width) as u16;
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{:>3}", label),
                Style::default().fg(t.label).add_modifier(Modifier::DIM),
            ))),
            Rect {
                x,
                y: heatmap_top,
                width: day_width as u16,
                height: 1,
            },
        );
    }

    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for week in 0..weeks {
        for day in 0..days_per_week {
            let idx = week * days_per_week + day;
            if idx >= total_days {
                break;
            }

            let intensity = ((seed.wrapping_add(idx as u64 * 2654435761)) % 10) as u16;
            let (ch, color) = match intensity {
                0..=1 => ("  ", t.border),
                2..=3 => ("▓▓", t.text_muted),
                4..=5 => ("▓▓", t.info),
                6..=7 => ("▓▓", t.primary),
                _ => ("▓▓", t.accent),
            };

            let x = start_x + (day * day_width) as u16;
            let y = heatmap_top + 1 + week as u16;
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(ch, Style::default().fg(color)))),
                Rect {
                    x,
                    y,
                    width: 2,
                    height: 1,
                },
            );
        }
    }

    // Legend
    let legend_y = heatmap_top + 1 + weeks as u16;
    if legend_y < inner.bottom() {
        let legend_line = Line::from(vec![
            Span::styled(" Less ", Style::default().fg(t.text_muted)),
            Span::styled("░░", Style::default().fg(t.border)),
            Span::styled(" ░░ ", Style::default().fg(t.text_muted)),
            Span::styled("▓▓", Style::default().fg(t.info)),
            Span::styled(" ▓▓ ", Style::default().fg(t.primary)),
            Span::styled("▓▓", Style::default().fg(t.accent)),
            Span::styled(" More ", Style::default().fg(t.text_muted)),
        ]);
        frame.render_widget(
            Paragraph::new(legend_line),
            Rect {
                x: start_x,
                y: legend_y,
                width: inner.width.saturating_sub(2),
                height: 1,
            },
        );
    }

    // Stats summary
    let stats_items = [
        "Total Requests:  2,847",
        "Avg. Tokens/req: 298",
        "Peak Throughput: 142 req/min",
        "Avg. Response:   1.8s",
        "Error Rate:      0.4%",
    ];
    for (i, item) in stats_items.iter().enumerate() {
        let y = legend_y + 2 + i as u16;
        if y >= inner.bottom() {
            break;
        }
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(*item, Style::default().fg(t.text)))),
            Rect {
                x: start_x,
                y,
                width: inner.width.saturating_sub(2),
                height: 1,
            },
        );
    }
}

/// ASCII bar chart rendered inside the chart inner area.
fn render_bar_chart(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let t = &app.theme.color;
    let data = [
        ("Sales", 85, t.accent),
        ("Revenue", 62, t.primary),
        ("Costs", 45, t.error),
        ("Profit", 38, t.ok),
        ("Growth", 70, t.info),
    ];
    let max_val = 100u16;
    let bar_max_width = area.width.saturating_sub(14) as usize;
    let bar_width = bar_max_width.min(50);

    for (i, (label, val, color)) in data.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.bottom() {
            break;
        }
        let bar_len = ((*val as usize) * bar_width / max_val as usize).max(1);
        let bar = "█".repeat(bar_len);
        let label_spaces = 8usize.saturating_sub(label.len());
        let line = format!(
            "  {}:{} ▏{}{}",
            label,
            " ".repeat(label_spaces),
            bar,
            if bar_len < bar_width {
                format!(" {}%", val)
            } else {
                String::new()
            }
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(line, Style::default().fg(*color)))),
            Rect {
                x: area.x + 1,
                y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
        );
    }
}

fn render_config(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    let t = &app.theme.color;
    let overlay = Rect {
        x: area.width / 6,
        y: area.height / 6,
        width: area.width * 2 / 3,
        height: area.height * 2 / 3,
    };

    frame.render_widget(Clear, overlay);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Configuration (Tab: next, Enter: apply, Esc: cancel) ")
        .border_style(Style::default().fg(t.label));
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Min(1),
        ])
        .split(inner);

    #[allow(clippy::type_complexity)]
    let fields: [(&str, &dyn Fn(&App) -> String); 5] = [
        ("Provider (anthropic / openai):", &|a| {
            format!("{:?}", a.active_provider)
        }),
        ("Model name:", &|a| a.config.model.clone()),
        ("Max tokens:", &|a| a.config.max_tokens.to_string()),
        ("Temperature (empty=default):", &|a| {
            a.config
                .temperature
                .map(|t| t.to_string())
                .unwrap_or_default()
        }),
        ("System prompt:", &|a| {
            let s = &a.config.system_prompt;
            if s.len() > 50 {
                format!("{}...", &s[..50])
            } else {
                s.clone()
            }
        }),
    ];

    for (i, (label, getter)) in fields.iter().enumerate() {
        let li = i * 2;
        let ci = i * 2 + 1;
        if li < chunks.len() {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    *label,
                    Style::default().fg(t.label),
                ))),
                chunks[li],
            );
        }
        if ci < chunks.len() {
            let is_current = i == app.config_field;
            let display = if is_current {
                format!("> {}", app.config_edit)
            } else {
                format!("  {}", getter(app))
            };
            let style = if is_current {
                Style::default()
                    .fg(t.text)
                    .bg(t.status_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            frame.render_widget(Paragraph::new(display).style(style), chunks[ci]);
        }
    }

    let hint_y = chunks.len().saturating_sub(1);
    if hint_y > 0 {
        frame.render_widget(
            Paragraph::new(" Tab: next | BackTab: prev | Enter: apply & return | Esc: cancel")
                .style(Style::default().fg(t.text_muted)),
            chunks[hint_y],
        );
    }
}

fn render_help(frame: &mut ratatui::Frame, app: &App) {
    let t = &app.theme.color;
    let area = frame.area();
    let overlay = Rect {
        x: area.width / 5,
        y: area.height / 6,
        width: area.width * 3 / 5,
        height: area.height * 2 / 3,
    };

    frame.render_widget(Clear, overlay);

    let lines = [
        "",
        "  OpenCode TUI - Keyboard Shortcuts",
        "",
        "  -- Chat mode -----------------------",
        "  Enter        Send prompt to LLM",
        "  Ctrl+Q       Quit TUI",
        "  Ctrl+K       Open configuration screen",
        "  F1           Toggle this help",
        "  Up / Down    Scroll message history",
        "  PageUp/Down  Scroll 10 lines",
        "",
        "  -- Config mode ---------------------",
        "  Tab          Next field",
        "  Shift+Tab    Previous field",
        "  Enter        Apply config and return",
        "  Esc          Cancel / return to chat",
        "",
        "  -- Environment variables -----------",
        "  ANTHROPIC_API_KEY     Anthropic API key",
        "  ANTHROPIC_AUTH_TOKEN  Bearer token (proxy)",
        "  ANTHROPIC_MODEL       Model override",
        "  OPENAI_API_KEY        OpenAI-compatible key",
        "  OPENAI_BASE_URL       OpenAI-compatible base URL",
        "  OPENAI_MODEL          OpenAI model override",
        "",
        "  Press any key to return.",
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(t.accent));
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let items: Vec<ListItem> = lines
        .iter()
        .map(|s| {
            let style = if s.starts_with("  --") {
                Style::default().fg(t.label).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            ListItem::new(Line::from(Span::styled(*s, style)))
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A simple word wrapper that preserves the string unchanged if it fits.
fn wrap_line(text: &str, max_width: usize) -> String {
    if text.len() <= max_width || max_width < 10 {
        text.to_string()
    } else {
        let mut out = String::with_capacity(max_width + 3);
        let mut rem = text;
        while !rem.is_empty() {
            if rem.len() <= max_width {
                out.push_str(rem);
                break;
            }
            if let Some(space) = rem[..max_width].rfind(' ') {
                out.push_str(&rem[..space]);
                out.push('\n');
                rem = &rem[space + 1..];
            } else {
                out.push_str(&rem[..max_width]);
                out.push('\n');
                rem = &rem[max_width..];
            }
        }
        out
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width < 10 || text.is_empty() {
        return vec![text.to_string()];
    }
    let w = max_width.saturating_sub(2);
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.len() <= w {
            lines.push(line.to_string());
        } else {
            let mut rem = line;
            while !rem.is_empty() {
                if rem.len() <= w {
                    lines.push(rem.to_string());
                    break;
                }
                if let Some(space) = rem[..w].rfind(' ') {
                    lines.push(rem[..space].to_string());
                    rem = &rem[space + 1..];
                } else {
                    lines.push(rem[..w].to_string());
                    rem = &rem[w..];
                }
            }
        }
    }
    lines
}

// ---------------------------------------------------------------------------
// Approval overlay
// ---------------------------------------------------------------------------

/// Renders the approval overlay for pending tool calls.
/// Covers the center of the screen with a bordered list of pending tools,
/// each with Y/N approval controls. The user navigates with Up/Down/Tab,
/// approves all with Y, denies all with N, approves individual with Enter,
/// and closes with Esc/q.
fn render_approval_overlay(frame: &mut ratatui::Frame, app: &App) {
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::style::{Modifier, Style};
    use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

    let area = frame.area();
    let t = &app.theme.color;

    let overlay = Rect {
        x: area.width / 5,
        y: area.height / 5,
        width: area.width * 3 / 5,
        height: area.height * 3 / 5,
    };

    frame.render_widget(Clear, overlay);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠ Tool Call Approval Required ")
        .border_style(Style::default().fg(t.warn));
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    // Split: instructions at top, list in middle
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(inner);

    // Instructions
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  Y: Approve all  |  N: Deny all  |  Enter: Approve selected  |  ↑↓: Navigate  |  Esc/q: Cancel",
                Style::default().fg(t.label)),
        ])),
        chunks[0],
    );

    // Pending tool call list
    let pending: Vec<&ToolCallEntry> = app
        .tool_calls
        .iter()
        .filter(|tc| tc.status == ToolStatus::Pending)
        .collect();

    if pending.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  No pending tool calls",
                Style::default().fg(t.text_muted),
            ))),
            chunks[1],
        );
    } else {
        let items: Vec<ListItem> = pending
            .iter()
            .enumerate()
            .map(|(i, tc)| {
                let is_selected = i == app.approval_selection;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(t.text)
                };
                let name = if tc.name.len() > 40 {
                    format!("{}...", &tc.name[..37])
                } else {
                    tc.name.clone()
                };
                let args_preview = if tc.context.len() > 60 {
                    format!("{}...", &tc.context[..57])
                } else {
                    tc.context.clone()
                };
                ListItem::new(vec![
                    Line::from(Span::styled(format!("{}{}", prefix, name), style)),
                    Line::from(Span::styled(
                        format!("    {}", args_preview),
                        Style::default().fg(t.text_muted),
                    )),
                ])
            })
            .collect();

        frame.render_widget(List::new(items), chunks[1]);
    }

    // Bottom hint
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Tools remain in Pending state until approved.",
            Style::default().fg(t.text_muted),
        ))),
        chunks[2],
    );
}

/// Render AI code template panel
fn render_ai_code_template(frame: &mut ratatui::Frame, app: &App) {
    if !app.ai_code_template.visible {
        return;
    }
    let area = centered_rect(80, 70, frame.area());
    let t = &app.theme.color;
    let block = Block::default()
        .title(" AI Code Templates ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.status_bg));
    frame.render_widget(Clear, area);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    // Search bar
    let search = Paragraph::new(Line::from(vec![
        Span::styled(" 🔍 ", Style::default().fg(t.text_muted)),
        Span::styled(
            &app.ai_code_template.search_query,
            Style::default().fg(t.text),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border)),
    );
    frame.render_widget(search, chunks[0]);

    // Template list
    let items: Vec<ListItem> = app
        .ai_code_template
        .templates
        .iter()
        .enumerate()
        .map(|(i, tmpl)| {
            let style = if i == app.ai_code_template.selected {
                Style::default().fg(t.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" 📝 {} ", tmpl.name), style),
                Span::styled(
                    format!("[{}]", tmpl.language),
                    Style::default().fg(t.text_muted),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Templates ")
            .border_style(Style::default().fg(t.border)),
    );
    frame.render_widget(list, chunks[1]);
}

/// Render project memo pad
fn render_project_memo(frame: &mut ratatui::Frame, app: &App) {
    if !app.project_memo.visible {
        return;
    }
    let area = centered_rect(70, 60, frame.area());
    let t = &app.theme.color;
    let block = Block::default()
        .title(" Project Memos ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.status_bg));
    frame.render_widget(Clear, area);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .project_memo
        .memos
        .iter()
        .enumerate()
        .map(|(i, memo)| {
            let style = if i == app.project_memo.selected {
                Style::default().fg(t.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" 📋 {} ", memo.title), style),
                Span::styled(
                    format!("({})", memo.created_at),
                    Style::default().fg(t.text_muted),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Memos ")
            .border_style(Style::default().fg(t.border)),
    );
    frame.render_widget(list, inner);
}

/// Render command snippet library
fn render_command_snippet(frame: &mut ratatui::Frame, app: &App) {
    if !app.command_snippet.visible {
        return;
    }
    let area = centered_rect(75, 65, frame.area());
    let t = &app.theme.color;
    let block = Block::default()
        .title(" Command Snippets ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.status_bg));
    frame.render_widget(Clear, area);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    // Search bar
    let search = Paragraph::new(Line::from(vec![
        Span::styled(" 🔍 ", Style::default().fg(t.text_muted)),
        Span::styled(
            &app.command_snippet.search_query,
            Style::default().fg(t.text),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border)),
    );
    frame.render_widget(search, chunks[0]);

    // Snippet list
    let items: Vec<ListItem> = app
        .command_snippet
        .snippets
        .iter()
        .enumerate()
        .map(|(i, snippet)| {
            let style = if i == app.command_snippet.selected {
                Style::default().fg(t.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" ⚡ {} ", snippet.name), style),
                Span::styled(
                    format!("$ {}", snippet.command),
                    Style::default().fg(t.text_muted),
                ),
                Span::styled(
                    format!(" ({} uses)", snippet.usage_count),
                    Style::default().fg(t.text_muted),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Snippets ")
            .border_style(Style::default().fg(t.border)),
    );
    frame.render_widget(list, chunks[1]);
}

/// Render unified diff viewer
fn render_unified_diff_viewer(frame: &mut ratatui::Frame, app: &App) {
    if !app.unified_diff_viewer.visible {
        return;
    }
    let area = centered_rect(85, 75, frame.area());
    let t = &app.theme.color;
    let block = Block::default()
        .title(format!(" Diff: {} ", app.unified_diff_viewer.file_path))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.status_bg));
    frame.render_widget(Clear, area);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = app
        .unified_diff_viewer
        .diff_content
        .iter()
        .map(|dline| {
            let (style, prefix) = match dline.line_type {
                DiffLineType::Header => {
                    (Style::default().fg(t.info).add_modifier(Modifier::BOLD), "")
                }
                DiffLineType::Context => (Style::default().fg(t.text), " "),
                DiffLineType::Added => (Style::default().fg(t.ok), "+"),
                DiffLineType::Removed => (Style::default().fg(t.error), "-"),
            };
            Line::from(Span::styled(format!("{}{}", prefix, dline.content), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).scroll((app.unified_diff_viewer.scroll as u16, 0));
    frame.render_widget(paragraph, inner);
}

/// Render task management board
fn render_task_board(frame: &mut ratatui::Frame, app: &App) {
    if !app.task_board.visible {
        return;
    }
    let area = centered_rect(90, 80, frame.area());
    let t = &app.theme.color;
    let block = Block::default()
        .title(" Task Board ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.status_bg));
    frame.render_widget(Clear, area);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let col_count = app.task_board.columns.len();
    if col_count == 0 {
        return;
    }
    let constraints: Vec<Constraint> = (0..col_count)
        .map(|_| Constraint::Ratio(1, col_count as u32))
        .collect();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (col_idx, column) in app.task_board.columns.iter().enumerate() {
        if col_idx >= chunks.len() {
            break;
        }
        let is_selected_col = col_idx == app.task_board.selected_col;
        let col_style = if is_selected_col {
            Style::default().fg(t.primary)
        } else {
            Style::default().fg(t.border)
        };

        let items: Vec<ListItem> = column
            .tasks
            .iter()
            .enumerate()
            .map(|(task_idx, task)| {
                let is_selected = is_selected_col && task_idx == app.task_board.selected_task;
                let priority_icon = match task.priority {
                    TaskPriority::Critical => "🔴",
                    TaskPriority::High => "🟠",
                    TaskPriority::Medium => "🟡",
                    TaskPriority::Low => "🟢",
                };
                let style = if is_selected {
                    Style::default().fg(t.primary).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(t.text)
                };
                ListItem::new(Line::from(vec![Span::styled(
                    format!(" {} {} ", priority_icon, task.title),
                    style,
                )]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ({}) ", column.title, column.tasks.len()))
                .border_style(col_style),
        );
        frame.render_widget(list, chunks[col_idx]);
    }
}

/// Helper: centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
