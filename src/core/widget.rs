use crate::core::Rect;
use crate::graphics::Canvas;

/// State passed to widgets during drawing.
#[derive(Debug, Clone, Copy, Default)]
pub struct WidgetState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

/// Keyboard key codes for text input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Enter,
}

/// Events that widgets can handle.
#[derive(Debug, Clone)]
pub enum WidgetEvent {
    MouseDown { x: i32, y: i32 },
    MouseUp { x: i32, y: i32 },
    MouseMove { x: i32, y: i32 },
    Click,
    CharInput { c: char },
    KeyDown { key: KeyCode },
    FocusGained,
    FocusLost,
}

/// The core trait for UI widgets.
pub trait Widget {
    /// Draw the widget to the canvas.
    /// `bounds` is the computed layout rect, `state` contains hover/press/focus info.
    fn draw(&self, canvas: &mut Canvas, bounds: &Rect, state: WidgetState);

    /// Returns the preferred size of this widget (width, height).
    /// Used by layout algorithms.
    fn preferred_size(&self) -> (u32, u32) {
        (0, 0)
    }

    /// Handle an event. Return true if the event was consumed.
    fn on_event(&mut self, _event: &WidgetEvent) -> bool {
        false
    }
}
