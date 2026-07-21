//! Component trait — Helix-inspired unified interface for all UI components.
//!
//! Each screen / overlay implements Component, providing:
//! - `handle_event()` — return HandleResult to propagate or consume
//! - `render()` — draw to Frame
//! - `should_update()` — skip rendering when unchanged
//! - `required_size()` — inform layout constraints
//! - `cursor()` — expose cursor position for the input line
//!
//! Components that need access to application context (e.g. App) implement
//! `render_with_context()` and `handle_event_with_context()`, which receive
//! a `&dyn std::any::Any` that can be downcast to the concrete type.

use std::any::Any;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::prelude::Position;
use ratatui::Frame;

/// Unique identifier for each layer in the compositor stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerId {
    /// Root / base layer (chat screen)
    Root,
    /// Configuration overlay
    Config,
    /// Help overlay
    Help,
    /// Dashboard screen
    Dashboard,
    /// Modal dialog / popup (dynamic)
    Popup,
    /// Autocomplete overlay
    Autocomplete,
    /// Approval overlay (tool execution)
    Approval,
    /// Generic overlay
    Overlay,
}

/// What happens after a component processes (or ignores) an event.
#[derive(Debug, Clone, PartialEq)]
pub enum HandleResult {
    /// Event was consumed; stop propagation.
    Consumed,
    /// Event was ignored; let it bubble to the next layer.
    Ignored,
    /// Event should close this component/layer.
    Close,
    /// Switch to a different layer / screen.
    Switch(LayerId),
    /// Exit the application entirely.
    Exit,
}

/// Alias for back-compat — the return type from `Component::handle_event`.
pub type EventResult = HandleResult;

/// Describes the kind of component for debug/status display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    Chat,
    Config,
    Help,
    Dashboard,
    Popup,
    Approval,
    Overlay,
}

/// The core trait every UI component must implement.
///
/// Inspired by Helix editor's `Component` trait but simplified for TUI chat.
///
/// Components that need access to the application context (App) should
/// override `handle_event_with_context` and/or `render_with_context`.
/// The default implementations fall back to the context-free versions.
pub trait Component {
    /// Process a key event. Return `HandleResult` to control event propagation.
    fn handle_event(&mut self, event: &KeyEvent) -> HandleResult;

    /// Process a key event with application context.
    /// Default: calls `handle_event()` (ignores context).
    fn handle_event_with_context(&mut self, event: &KeyEvent, _ctx: &mut dyn Any) -> HandleResult {
        self.handle_event(event)
    }

    /// Render the component into the given area.
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Render with application context.
    /// Default: calls `render()` (ignores context).
    fn render_with_context(&self, frame: &mut Frame, area: Rect, _ctx: &mut dyn Any) {
        self.render(frame, area);
    }

    /// Whether the component needs re-rendering.
    /// Return `true` to draw, `false` to skip (saves CPU).
    fn should_update(&self) -> bool {
        true
    }

    /// Report the minimum / preferred size.
    /// Return `None` to accept any size.
    fn required_size(&self, _constraint: Rect) -> Option<Rect> {
        None
    }

    /// Cursor position (for text input components).
    fn cursor(&self) -> Option<Position> {
        None
    }

    /// Unique layer identifier.
    fn kind(&self) -> ComponentKind;

    /// Called when this component becomes the active (top) layer.
    fn on_activate(&mut self) {}

    /// Called when this component loses focus (a new layer is pushed above).
    fn on_deactivate(&mut self) {}
}
