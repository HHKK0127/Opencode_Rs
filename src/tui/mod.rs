//! TUI framework module — Compositor + Component trait + Layers + Markdown streaming.
//!
//! Implements Helix-inspired Compositor (layer stack) and Component trait,
//! plus pulldown_cmark-based streaming markdown rendering.
//! Integrates with the existing App struct in in/opencode_tui.rs.

pub mod component;
pub mod compositor;
pub mod editor;
pub mod layers;
pub mod markdown_stream;

pub use component::{Component, ComponentKind, EventResult, HandleResult, LayerId};
pub use compositor::Compositor;
pub use layers::RootComponent;
pub use markdown_stream::{MarkdownRenderer, MarkdownStream, MarkdownTheme};
