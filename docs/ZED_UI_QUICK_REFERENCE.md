# Zed UI Architecture - Quick Reference Guide

## Component Lifecycle Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    GPUI Component Lifecycle                      │
└─────────────────────────────────────────────────────────────────┘

CREATION PHASE (Frame N)
═══════════════════════════════════════════════════════════════════
  ┌─ Entity Created
  │  └─ implements Render trait
  │
  ├─ View::render() called
  │  └─ Returns impl IntoElement
  │     └─ Converted to AnyElement tree
  │
  └─ Component<C> wrapper created (for RenderOnce)


LAYOUT PHASE (Taffy computation)
═══════════════════════════════════════════════════════════════════
  ┌─ Element::request_layout() called recursively
  │  ├─ Converts style → Taffy node
  │  ├─ Calls window.request_layout(style, ...)
  │  └─ Returns LayoutId (stores computed bounds)
  │
  └─ Taffy::compute_layout() computes all bounds
     └─ Takes AvailableSpace, returns Bounds<Pixels>


PREPAINT PHASE (Hitbox registration)
═══════════════════════════════════════════════════════════════════
  ┌─ Element::prepaint() called recursively
  │  ├─ Retrieves LayoutId → Bounds<Pixels>
  │  ├─ Creates Hitbox for mouse/keyboard detection
  │  ├─ Stores element state (hover flags, etc)
  │  └─ Returns PrepaintState
  │
  └─ All hitboxes added to bounds tree for hit detection


PAINT PHASE (GPU rendering)
═══════════════════════════════════════════════════════════════════
  ┌─ Element::paint() called recursively (reverse depth order)
  │  ├─ Access PrepaintState from previous phase
  │  ├─ Issue GPU commands (draw rectangles, text, images)
  │  └─ Respect z-index and content masks
  │
  └─ Scene submitted to GPU for rendering


CLEANUP & FRAME END
═══════════════════════════════════════════════════════════════════
  ├─ Element tree dropped (but cached in ViewCacheKey)
  ├─ Element arena recycled for next frame
  └─ Next frame starts with cx.notify() → re-render


STATE CHANGES (During event handling)
═══════════════════════════════════════════════════════════════════
  Mouse Click Event
       ↓
  Div::on_mouse_down listener fires
       ↓
  User closure: cx.view(|this, cx| {
                  this.field = new_value;
                  cx.notify();  ← MARKS FOR RE-RENDER
                })
       ↓
  Next frame automatically calls render() again
       ↓
  Only changed elements are re-laid out
```

---

## Trait Composition Flowchart

```
           Button Component
                 │
                 ├─ struct Button { base: ButtonLike, label: ... }
                 │
          ┌──────┴──────────────────────────────┐
          │                                      │
    Implements traits:                  Composition chain:
          │                                      │
      RenderOnce ───┐                   Button::new()
          │         │                      ↓
      Toggleable    ├──→ renders         .toggle_state(true)
          │         │    Button's           ↓
      SelectableButton   content       .selected_style(...)
          │         │    with:            ↓
      Disableable   │                  .on_click(handler)
          │         │                     ↓
      IntoElement ──┘                  .into_element()
                                         ↓
                                    Component<Button>
                                         ↓
                                    Element::request_layout()
                                         ↓
                                    ... (paint phase) ...
                                         ↓
                                    Rendered Button
```

---

## Style Application Cascade

```
STYLING PRIORITY (Top = Higher Priority)
═══════════════════════════════════════════════════════════════════

    7. STATE OVERRIDES (hover, active, disabled)
       └─ Applied conditionally in .when() blocks
       
    6. USER BUILDER METHODS (.bg(), .p(), .rounded_lg())
       └─ .p(px(16.)).rounded_lg().bg(Color::Primary)
       
    5. COMPONENT STYLE (ButtonLike defaults)
       └─ Button sets size, padding, border by default
       
    4. THEME EXTENSION (.elevation_1(), .h_flex())
       └─ StyledExt trait adds theme-aware styling
       
    3. THEME TOKENS (color scheme selection)
       └─ cx.theme().colors() selects from color palette
       
    2. ELEMENT DEFAULT STYLE (Div/Text/Svg defaults)
       └─ display: flex, position: relative, etc
       
    1. STYLE REFINEMENT SYSTEM (Refineable trait)
       └─ Merges styles: default ← theme ← component ← user


EXAMPLE: Button with all layers
═══════════════════════════════════════════════════════════════════

// Layer 1-2: Element defaults + Theme
div()
    .flex()              // ElementId=div, default flex
    .items_center()      // justify-content: center

// Layer 3: Theme colors
    .bg(cx.theme().colors().element_background)
    .text_color(cx.theme().colors().text)

// Layer 4: Button component sizing
    .h(px(32.))
    .px(px(12.))

// Layer 5: Elevation styling (from StyledExt)
    .rounded_lg()
    .border_1()
    .shadow(ElevationIndex::Surface.shadow(cx))

// Layer 6: User customization
    .p(px(4.))
    .m(px(8.))

// Layer 7: State overrides (inside render)
    .when(is_hovered, |this| this.bg(cx.theme().colors().element_hover))
    .when(is_disabled, |this| this.opacity(0.5))
```

---

## Theme Color Semantic Map

```
┌────────────────────────────────────────────────────────────────┐
│                   THEME COLOR STRUCTURE                        │
└────────────────────────────────────────────────────────────────┘

SURFACES (z-indexed layers)
──────────────────────────────────────────────────────────────────
  background                     ← Z-3 (app background)
  ↓ 
  surface_background             ← Z-0 (panels, tabs, base)
  ↓
  elevated_surface_background    ← Z+1 (popovers, modals)
  
  Using: cx.theme().colors().surface_background


INTERACTIVE ELEMENTS (state variants)
──────────────────────────────────────────────────────────────────
  element_background      [default]  ← Used for buttons, inputs
  element_hover          [onHover]  ← Lightened/darkened version
  element_active         [onPress]  ← Further contrast
  element_selected       [selected] ← Toggle/radio selected
  element_disabled       [disabled] ← Grayed out
  
  Using: .bg(colors.element_background)
         .when(is_hovered, |t| t.bg(colors.element_hover))


GHOST ELEMENTS (surface-blending buttons)
──────────────────────────────────────────────────────────────────
  ghost_element_background  [default]  ← Same as surface
  ghost_element_hover       [onHover]  ← Subtle highlight
  ghost_element_active      [onPress]  ← Stronger highlight
  ghost_element_selected    [selected] ← Emphasized
  ghost_element_disabled    [disabled] ← Subtle reduction
  
  Using: .bg(colors.ghost_element_background)
         For buttons that blend into their container


TEXT COLORS (semantic importance)
──────────────────────────────────────────────────────────────────
  text            ← Default (high contrast, most readable)
  text_muted      ← Secondary (reduced emphasis)
  text_accent     ← Emphasized (highlights, links, focus)
  text_placeholder ← Input hints (very low contrast)
  text_disabled   ← Disabled state
  
  Using: .text_color(colors.text)
         .text_color(colors.text_muted)


ICON COLORS (fill colors for SVG icons)
──────────────────────────────────────────────────────────────────
  icon            ← Default icon fill
  icon_muted      ← Deemphasized icons
  icon_accent     ← Highlighted icons (selected toggles)
  icon_placeholder ← Icon in empty state
  icon_disabled   ← Disabled icon button
  
  Using: Icon::new(IconName::Save).color(colors.icon)


BORDERS (visual separation)
──────────────────────────────────────────────────────────────────
  border          ← Primary (high contrast divider)
  border_variant  ← Secondary (subtle visual divider)
  border_focused  ← Keyboard focus indicator
  border_selected ← Selected element indicator
  border_disabled ← Disabled element indicator
  border_transparent ← Placeholder (maintains space)
  
  Using: .border_1().border_color(colors.border)


STATUS COLORS (semantic meaning)
──────────────────────────────────────────────────────────────────
  success (green)   ← Positive action/state
  warning (yellow)  ← Caution/attention needed
  error (red)       ← Destructive/critical
  info (blue)       ← Informational
  
  Using: StatusColors { success, warning, error, info }


COMPONENT-SPECIFIC COLORS
──────────────────────────────────────────────────────────────────
  scrollbar_thumb_background     ← Scrollbar handle
  search_match_background        ← Search result highlight
  tab_active_background          ← Active tab
  pane_focused_border            ← Focused pane outline
  debugger_accent                ← Debugger UI elements
  
  Using: For specialized components needing custom colors
```

---

## Layout System Quick Reference

```
DISPLAY TYPES
═════════════════════════════════════════════════════════════════
  Flex Layout:
  ────────────────────────────────────────────────────────────
  div()
    .flex()                      ← display: flex
    .flex_row()                  ← flex-direction: row (default)
    .flex_col()                  ← flex-direction: column
    .flex_wrap()                 ← flex-wrap: wrap
    .justify_start()             ← justify-content: flex-start
    .justify_center()            ← justify-content: center
    .justify_between()           ← justify-content: space-between
    .justify_around()            ← justify-content: space-around
    .items_start()               ← align-items: flex-start
    .items_center()              ← align-items: center
    .items_stretch()             ← align-items: stretch
    .gap(px(8.))                 ← gap: 8px


  Block Layout:
  ────────────────────────────────────────────────────────────
  div()
    .block()                     ← display: block


  Hidden:
  ────────────────────────────────────────────────────────────
  div()
    .hidden()                    ← display: none


SIZING
═════════════════════════════════════════════════════════════════
  Fixed Sizes:
  ────────────────────────────────────────────────────────────
  .w(px(100.))                   ← width: 100px
  .h(px(50.))                    ← height: 50px
  
  
  Relative Sizes:
  ────────────────────────────────────────────────────────────
  .w_full()                      ← width: 100%
  .h_full()                      ← height: 100%
  .w_screen()                    ← width: 100vw
  .h_screen()                    ← height: 100vh
  .w(relative(0.5))              ← width: 50%
  
  
  Flexible Sizing:
  ────────────────────────────────────────────────────────────
  .flex_1()                      ← flex: 1 (grow and shrink)
  .flex_grow()                   ← flex-grow: 1
  .flex_shrink()                 ← flex-shrink: 1
  .flex_shrink_0()               ← flex-shrink: 0
  
  
  Min/Max:
  ────────────────────────────────────────────────────────────
  .min_w(px(200.))               ← min-width: 200px
  .max_w(px(600.))               ← max-width: 600px
  .min_h(px(100.))               ← min-height: 100px
  .max_h(px(800.))               ← max-height: 800px


SPACING (Box Model)
═════════════════════════════════════════════════════════════════
  Padding:
  ────────────────────────────────────────────────────────────
  .p(px(16.))                    ← padding: 16px (all sides)
  .px(px(12.))                   ← padding-left/right: 12px
  .py(px(8.))                    ← padding-top/bottom: 8px
  .pt(px(4.))                    ← padding-top: 4px
  .pb(px(4.))                    ← padding-bottom: 4px
  
  
  Margin:
  ────────────────────────────────────────────────────────────
  .m(px(16.))                    ← margin: 16px (all sides)
  .mx(px(12.))                   ← margin-left/right: 12px
  .my(px(8.))                    ← margin-top/bottom: 8px
  .mt(px(4.))                    ← margin-top: 4px
  .mb(px(4.))                    ← margin-bottom: 4px


POSITION
═════════════════════════════════════════════════════════════════
  Relative:
  ────────────────────────────────────────────────────────────
  div()
    .relative()                  ← position: relative
    .left(px(10.))               ← left: 10px
    .right(px(10.))              ← right: 10px
    .top(px(5.))                 ← top: 5px
    .bottom(px(5.))              ← bottom: 5px
  
  
  Absolute:
  ────────────────────────────────────────────────────────────
  div()
    .absolute()                  ← position: absolute
    .left(px(0.))                ← left: 0 (position from left edge)
  
  
  Fixed (rare, for overlays):
  ────────────────────────────────────────────────────────────
  div()
    .fixed()                     ← position: fixed


OVERFLOW
═════════════════════════════════════════════════════════════════
  .overflow_hidden()             ← overflow: hidden (clip content)
  .overflow_scroll()             ← overflow: scroll (always show scrollbar)
  .overflow_auto()               ← overflow: auto (show if needed)
  .overflow_x_hidden()           ← overflow-x: hidden
  .overflow_y_scroll()           ← overflow-y: scroll


BORDERS & CORNERS
═════════════════════════════════════════════════════════════════
  .border_1()                    ← border: 1px solid
  .border_2()                    ← border: 2px solid
  .border_color(colors.border)   ← border-color: ...
  .rounded_md()                  ← border-radius: 4px
  .rounded_lg()                  ← border-radius: 6px
  .rounded_xl()                  ← border-radius: 12px
  .rounded_full()                ← border-radius: 50%


TEXT STYLING
═════════════════════════════════════════════════════════════════
  .text_left()                   ← text-align: left
  .text_center()                 ← text-align: center
  .text_right()                  ← text-align: right
  .text_ellipsis()               ← text-overflow: ellipsis
  .truncate()                    ← white-space: nowrap + text-overflow: ellipsis
  .line_clamp(3)                 ← Show max 3 lines


BACKGROUND
═════════════════════════════════════════════════════════════════
  .bg(Hsla::default())           ← background-color: hsla(...)
  .bg(colors.element_background) ← Using theme token


VISIBILITY
═════════════════════════════════════════════════════════════════
  .opacity(0.5)                  ← opacity: 50%
  .invisible()                   ← visibility: hidden (takes space)
  .hidden()                      ← display: none (no space)
```

---

## Event Handling Patterns

```
MOUSE EVENTS
═════════════════════════════════════════════════════════════════

  Click (full cycle):
  ────────────────────────────────────────────────────────────
  div()
    .on_mouse_down(|event, window, cx| {
        // event: MouseDownEvent { button, position, modifiers, ... }
    })
    .on_mouse_up(|event, window, cx| {
        // event: MouseUpEvent { button, position, modifiers, ... }
    })
    .on_click(|event, window, cx| {
        // event: ClickEvent { button, position, count }
        // Fires on release if mouse was over element during press
    })
  
  
  Hover:
  ────────────────────────────────────────────────────────────
  div()
    .on_mouse_move(|event, window, cx| {
        // event: MouseMoveEvent { position, pressed_button }
        // Fires continuously as mouse moves
    })
    .on_mouse_down(|event, window, cx| {
        // Use event.button to detect which button
        if event.button == MouseButton::Left {
            // Left click
        }
    })
  
  
  Drag (multi-step):
  ────────────────────────────────────────────────────────────
  div()
    .on_mouse_down(|event, window, cx| {
        // Step 1: Start drag
        cx.start_drag(DragPayload::Files(vec![...]))
    })
    .on_mouse_move(|event, window, cx| {
        // Step 2: Drag over (receives DragMoveEvent)
        // Can show drop zone highlighting
    })
    .on_drop(|event, window, cx| {
        // Step 3: Drop received
        let files = event.drag::<Vec<PathBuf>>();
    })


KEYBOARD EVENTS
═════════════════════════════════════════════════════════════════

  Key Press:
  ────────────────────────────────────────────────────────────
  div()
    .on_key_down(|event, window, cx| {
        // event: KeyDownEvent { keystroke, is_held, ... }
        if event.keystroke.key == "Enter" {
            // Enter key pressed
        }
        if event.keystroke.modifiers.contains(Modifiers::SUPER) {
            // Command/Windows key held
        }
    })
    .on_key_up(|event, window, cx| {
        // event: KeyUpEvent { keystroke }
    })


FOCUS EVENTS
═════════════════════════════════════════════════════════════════

  Focus Management:
  ────────────────────────────────────────────────────────────
  let focus_handle = cx.focus_handle();
  
  div()
    .key_context("my_panel")          // Enable key dispatch in this context
    .on_focus_out(move |_, _, _| {
        // Lost focus
    })
    .when(cx.is_focused(&focus_handle), |this| {
        this.border_color(colors.border_focused)
    })


CUSTOM EVENT DISPATCH
═════════════════════════════════════════════════════════════════

  Define Action:
  ────────────────────────────────────────────────────────────
  #[derive(serde::Deserialize)]
  pub struct SaveFile;
  
  
  Dispatch Action:
  ────────────────────────────────────────────────────────────
  cx.dispatch_action(Box::new(SaveFile))
  
  
  Handle Action:
  ────────────────────────────────────────────────────────────
  pub fn register_actions(cx: &mut App) {
      cx.on_action(|action: &SaveFile, _| {
          // Handle save action
      });
  }
```

---

## Common Component Patterns

```
PATTERN 1: Simple Stateless Component
═════════════════════════════════════════════════════════════════

#[derive(IntoElement)]
pub struct Badge {
    text: SharedString,
    color: Color,
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .h(px(20.))
            .px(px(8.))
            .flex()
            .items_center()
            .rounded_full()
            .bg(self.color.base(cx))
            .child(Label::new(self.text).size(LabelSize::Small))
    }
}

// Usage
Badge {
    text: "New".into(),
    color: Color::Success,
}


PATTERN 2: Stateful View with Events
═════════════════════════════════════════════════════════════════

pub struct Counter {
    count: usize,
}

impl Render for Counter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.count;
        
        h_flex()
            .gap(px(8.))
            .child(
                Button::new("decrement", "-")
                    .on_click(move |_, _, cx| {
                        cx.view(|this, cx| {
                            this.count = this.count.saturating_sub(1);
                            cx.notify();
                        });
                    })
            )
            .child(Label::new(count.to_string()))
            .child(
                Button::new("increment", "+")
                    .on_click(move |_, _, cx| {
                        cx.view(|this, cx| {
                            this.count += 1;
                            cx.notify();
                        });
                    })
            )
    }
}


PATTERN 3: Async Data Loading
═════════════════════════════════════════════════════════════════

pub struct DataPanel {
    data: Option<Vec<Item>>,
    is_loading: bool,
    error: Option<String>,
}

impl Render for DataPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap(px(8.))
            .when(self.is_loading, |this| {
                this.child(Label::new("Loading..."))
            })
            .when_some(self.error.as_ref(), |this, error| {
                this.child(Label::new(error.clone()).color(Color::Error))
            })
            .when_some(self.data.as_ref(), |this, data| {
                this.children(data.iter().map(|item| {
                    Label::new(item.name.clone())
                }))
            })
    }
}

// In parent component:
let panel = cx.new_view(|_| DataPanel {
    data: None,
    is_loading: false,
    error: None,
});

// Load data in background
cx.spawn(|panel, mut cx| async move {
    match fetch_data().await {
        Ok(data) => {
            panel.update(&mut cx, |this, cx| {
                this.data = Some(data);
                this.is_loading = false;
                cx.notify();
            });
        }
        Err(e) => {
            panel.update(&mut cx, |this, cx| {
                this.error = Some(e.to_string());
                this.is_loading = false;
                cx.notify();
            });
        }
    }
});


PATTERN 4: Conditional Rendering with Traits
═════════════════════════════════════════════════════════════════

div()
    .h_flex()
    .gap(px(8.))
    // Conditional: render button only if not disabled
    .when(!disabled, |this| {
        this.child(Button::new("action", "Click me"))
    })
    // Conditional with some value
    .when_some(maybe_icon.as_ref(), |this, icon| {
        this.child(icon.clone())
    })
    // Conditional with result
    .when_some(result.as_ref().ok(), |this, value| {
        this.child(Label::new(format!("{:?}", value)))
    })


PATTERN 5: List Rendering with Keys
═════════════════════════════════════════════════════════════════

div()
    .flex_col()
    .gap(px(4.))
    .children(items.iter().enumerate().map(|(index, item)| {
        // Use index as key (not ideal for mutable lists)
        div()
            .key_down_event(index)  // Helps GPUI track element
            .h_flex()
            .gap(px(8.))
            .child(Label::new(&item.name))
            .child(Label::new(&item.value))
    }))


PATTERN 6: Hover State Styling
═════════════════════════════════════════════════════════════════

div()
    .h(px(40.))
    .px(px(12.))
    .rounded_lg()
    .bg(colors.element_background)
    .transition_bg()              // Smooth color transition
    .on_mouse_move(move |event, window, cx| {
        // Track hover state in parent or use event bounds
    })
    .when(is_hovered, |this| {
        this.bg(colors.element_hover)
    })
```

---

## Migration Checklist: React → Zed-Style

```
□ Define semantic color tokens (replace hex colors)
  ├─ Colors struct with named fields (not hex)
  ├─ Semantic names: elementBackground, textMuted, etc
  └─ Theme switching logic using global

□ Implement builder pattern styling utilities
  ├─ vStack(gap) function
  ├─ hStack(gap) function
  ├─ Elevation helpers (elevation1, elevation2)
  └─ flex/grid helper functions

□ Create trait-like composition patterns
  ├─ Higher-order components for clickable, disableable
  ├─ Custom hooks for behavior mixing
  └─ Consistent prop spreading

□ Centralize state management
  ├─ Entity-like state containers
  ├─ Update functions instead of scattered useState
  └─ Automatic change detection

□ Implement component registration system
  ├─ Component registry for demo/testing
  ├─ Storybook integration
  └─ Visual component testing

□ Design system documentation
  ├─ Color palette reference
  ├─ Component API docs
  ├─ Spacing/sizing scale
  └─ Typography guidelines

□ Performance optimization
  ├─ Memoization for expensive renders
  ├─ useCallback for stable handlers
  └─ React.memo for pure components

□ Migration path planning
  ├─ Start with new components in Zed style
  ├─ Refactor existing components incrementally
  ├─ Long-term: Tauri + Rust GPUI target
  └─ Document architecture decisions
```

---

## Resources & References

**Key Files in OpenCode_Rs:**
- AGENTS.md - Project context
- crates/gpui/src/element.rs - Element lifecycle (870 lines)
- crates/gpui/src/styled.rs - Styling trait (885 lines)
- crates/ui/src/components/button/button.rs - Button implementation (612 lines)
- crates/theme/src/styles/colors.rs - Color definitions (685 lines)

**External References:**
- https://gpui.rs - GPUI official documentation
- https://github.com/zed-industries/zed - Zed source code
- Taffy: https://github.com/DioxusLabs/taffy - Layout engine

**Learning Path:**
1. Read element.rs to understand lifecycle
2. Study Styled trait for styling patterns
3. Examine Button component for composition
4. Review theme system for color management
5. Start building components using these patterns
