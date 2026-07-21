//! Compositor — Helix-inspired layer stack.
//!
//! Manages a stack of Component layers. Events are dispatched top-down:
//! the topmost (focused) layer receives the event first.
//! If it returns `HandleResult::Ignored`, the event bubbles down.
//!
//! Supports: push (focus), pop (close), switch (replace), and conditional render.
//!
//! # Context-aware dispatch
//!
//! `handle_event_with_context()` and `render_with_context()` pass an
//! `&dyn Any` to the topmost layer's `*_with_context()` method, allowing
//! layers to downcast to the application's concrete App struct.

use std::any::Any;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::prelude::Position;
use ratatui::Frame;

use super::component::{Component, ComponentKind, HandleResult, LayerId};

// The compositor manages a stack of component layers.
//
// # Event dispatch
//
// 1. The topmost layer receives the event first.
// 2. If it returns `HandleResult::Ignored`, the next layer down receives it.
// 3. If `HandleResult::Consumed`, propagation stops.
//
// # Rendering
//
// Layers are rendered bottom-up. The topmost layer renders last (on top).
// Semitransparent overlays can render the layer below first, then overlay.
pub struct Compositor {
    /// Layer stack: bottom (index 0) → top (last index).
    layers: Vec<Box<dyn Component>>,
    /// Whether the compositor is dirty (needs re-render).
    dirty: bool,
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compositor {
    /// Create an empty compositor.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            dirty: true,
        }
    }

    /// Push a new layer to the top (focus it).
    ///
    /// The pushed layer's `on_activate()` is called.
    pub fn push(&mut self, mut layer: Box<dyn Component>) {
        // Deactivate current top
        if let Some(top) = self.layers.last_mut() {
            top.on_deactivate();
        }
        layer.on_activate();
        self.layers.push(layer);
        self.mark_dirty();
    }

    /// Pop the topmost layer (close it).
    ///
    /// The new top's `on_activate()` is called.
    pub fn pop(&mut self) -> Option<Box<dyn Component>> {
        if self.layers.len() <= 1 {
            return None; // Don't pop the root layer
        }
        let popped = self.layers.pop();
        if let Some(top) = self.layers.last_mut() {
            top.on_activate();
        }
        self.mark_dirty();
        popped
    }

    /// Pop all layers down to and including the given layer.
    /// If the layer is not found, does nothing.
    pub fn pop_until(&mut self, target: LayerId) {
        while self.layers.len() > 1 {
            if self.layers.last().map(|l| l.kind()) == Some(ComponentKind::Chat) {
                break; // Never pop the root
            }
            let top_kind = self.layers.last().map(|l| l.kind());
            // Check if this is the target
            if top_kind.map(kind_to_layer) == Some(target) {
                self.layers.pop();
                if let Some(top) = self.layers.last_mut() {
                    top.on_activate();
                }
                self.mark_dirty();
                return;
            }
            self.layers.pop();
        }
        // Target not found; reactivate root
        if let Some(top) = self.layers.last_mut() {
            top.on_activate();
        }
        self.mark_dirty();
    }

    /// Replace the entire stack with a single root layer.
    pub fn switch_to(&mut self, mut layer: Box<dyn Component>) {
        self.layers.clear();
        layer.on_activate();
        self.layers.push(layer);
        self.mark_dirty();
    }

    /// The current / topmost layer.
    pub fn current(&self) -> Option<&dyn Component> {
        self.layers.last().map(|b| b.as_ref())
    }

    /// The current / topmost layer (mutable).
    pub fn current_mut(&mut self) -> Option<&mut dyn Component> {
        self.layers
            .last_mut()
            .map(|b| b.as_mut() as &mut dyn Component)
    }

    /// Number of layers in the stack.
    pub fn depth(&self) -> usize {
        self.layers.len()
    }

    /// Whether the compositor is dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.layers.iter().any(|l| l.should_update())
    }

    /// Mark as dirty (force re-render next frame).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear the dirty flag (called after render).
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Dispatch a key event to layers from top to bottom.
    ///
    /// Returns the first non-`Ignored` result.
    /// If all layers ignore, returns `HandleResult::Ignored`.
    pub fn handle_event(&mut self, event: &KeyEvent) -> HandleResult {
        for layer in self.layers.iter_mut().rev() {
            let result = layer.handle_event(event);
            match result {
                HandleResult::Ignored => continue,
                HandleResult::Consumed => {
                    self.mark_dirty();
                    return HandleResult::Consumed;
                }
                HandleResult::Close => {
                    self.pop();
                    return HandleResult::Consumed;
                }
                HandleResult::Switch(target) => {
                    // Pop down to the target
                    self.pop_until(target);
                    return HandleResult::Consumed;
                }
                HandleResult::Exit => return HandleResult::Exit,
            }
        }
        HandleResult::Ignored
    }

    /// Dispatch a key event with application context.
    /// Just like `handle_event`, but passes the context to layers that
    /// implement `handle_event_with_context`.
    pub fn handle_event_with_context(
        &mut self,
        event: &KeyEvent,
        ctx: &mut dyn Any,
    ) -> HandleResult {
        for layer in self.layers.iter_mut().rev() {
            let result = layer.handle_event_with_context(event, ctx);
            match result {
                HandleResult::Ignored => continue,
                HandleResult::Consumed => {
                    self.mark_dirty();
                    return HandleResult::Consumed;
                }
                HandleResult::Close => {
                    self.pop();
                    return HandleResult::Consumed;
                }
                HandleResult::Switch(target) => {
                    self.pop_until(target);
                    return HandleResult::Consumed;
                }
                HandleResult::Exit => return HandleResult::Exit,
            }
        }
        HandleResult::Ignored
    }

    /// Render all layers bottom-up.
    ///
    /// Each layer renders into the full area. Overlays are responsible for
    /// rendering the layer below (or the compositor handles it by rendering
    /// all layers in order — the top layer draws last, potentially covering).
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        for layer in &self.layers {
            layer.render(frame, area);
        }
    }

    /// Render all layers bottom-up with application context.
    /// Layers that implement `render_with_context` receive the context.
    pub fn render_with_context(&self, frame: &mut Frame, area: Rect, ctx: &mut dyn Any) {
        for layer in &self.layers {
            layer.render_with_context(frame, area, ctx);
        }
    }

    /// Cursor position from the topmost layer.
    pub fn cursor(&self) -> Option<Position> {
        self.layers.last().and_then(|l| l.cursor())
    }

    /// Collect all dirty layers for selective re-render.
    pub fn dirty_layers(&self) -> Vec<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter(|(_, l)| l.should_update())
            .map(|(i, _)| i)
            .collect()
    }

    /// Access a layer by its kind (bottom-up search for the first match).
    pub fn find_layer(&mut self, kind: ComponentKind) -> Option<&mut Box<dyn Component>> {
        self.layers.iter_mut().rev().find(|l| l.kind() == kind)
    }

    /// Check if a layer of the given kind exists in the stack.
    pub fn has_layer(&self, kind: ComponentKind) -> bool {
        self.layers.iter().any(|l| l.kind() == kind)
    }

    /// Remove a layer of the given kind from the stack.
    /// Useful for replacing an existing overlay.
    pub fn remove_layer(&mut self, kind: ComponentKind) {
        self.layers.retain(|l| l.kind() != kind);
    }

    /// Replace or push a layer. If a layer of the same kind exists, it is
    /// replaced at its current position. Otherwise, it is pushed on top.
    pub fn replace_or_push(&mut self, layer: Box<dyn Component>) {
        let kind = layer.kind();
        if let Some(pos) = self.layers.iter().position(|l| l.kind() == kind) {
            self.layers[pos] = layer;
            self.mark_dirty();
        } else {
            self.push(layer);
        }
    }
}

fn kind_to_layer(kind: ComponentKind) -> LayerId {
    match kind {
        ComponentKind::Chat => LayerId::Root,
        ComponentKind::Config => LayerId::Config,
        ComponentKind::Help => LayerId::Help,
        ComponentKind::Dashboard => LayerId::Dashboard,
        ComponentKind::Popup => LayerId::Popup,
        ComponentKind::Approval => LayerId::Approval,
        ComponentKind::Overlay => LayerId::Overlay,
    }
}
