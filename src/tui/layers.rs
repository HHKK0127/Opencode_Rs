//! Layer implementations — each screen as a Component.
//!
//! Each layer wraps the rendering and event logic for its screen.
//! The concrete wrapper types (e.g. `ChatWrapper`, `ConfigWrapper`) are
//! defined in the binary crate (`opencode_tui.rs`) because they need to
//! reference the `App` struct which lives there.
//!
//! This module only provides the `RootComponent` placeholder for the
//! compositor root layer.

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::tui::component::{Component, ComponentKind, HandleResult};

// ---------------------------------------------------------------------------
// RootComponent — bridge to the existing App
// ---------------------------------------------------------------------------

/// Marker layer that identifies the root compositor layer.
/// The existing App struct handles all events/render natively;
/// this just satisfies the Compositor stack.
pub struct RootComponent;

impl Default for RootComponent {
    fn default() -> Self {
        Self
    }
}

impl RootComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for RootComponent {
    fn handle_event(&mut self, _event: &KeyEvent) -> HandleResult {
        HandleResult::Ignored // Handled by App's main loop
    }

    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn kind(&self) -> ComponentKind {
        ComponentKind::Chat
    }
}
