use image::RgbImage;
use std::time::Instant;

use crate::core::{KeyCode, Rect, Widget, WidgetEvent, WidgetState};
use crate::graphics::{caret_x, draw_caret, draw_text, line_height, Canvas, TextStyle};

/// A text input widget for editable single-line text.
///
/// ## Limitations (v0)
/// - ASCII input only (characters 32-126)
/// - No text selection, copy/paste, or IME
/// - No internal scrolling (text is clipped if too long)
/// - No undo/redo
pub struct TextInput {
    /// The current text content.
    text: String,
    /// Cursor position (0..=text.len()).
    cursor: usize,
    /// Background images for different states.
    normal: RgbImage,
    hover: RgbImage,
    focused: RgbImage,
    /// Optional invalid state background.
    invalid: Option<RgbImage>,
    /// Widget dimensions.
    width: u32,
    height: u32,
    /// Text padding from edges.
    padding: u32,
    /// Text color.
    text_color: u32,
    /// Caret color.
    caret_color: u32,
    /// Whether the input is currently marked as invalid.
    is_invalid: bool,
    /// Caret blink timing.
    caret_visible: bool,
    last_blink: Instant,
    /// Action to emit on change.
    on_change_action: Option<String>,
    /// Action to emit on submit (Enter).
    on_submit_action: Option<String>,
}

impl TextInput {
    /// Create a new text input with the given state images.
    pub fn new(
        normal: RgbImage,
        hover: RgbImage,
        focused: RgbImage,
        invalid: Option<RgbImage>,
    ) -> Self {
        let width = normal.width();
        let height = normal.height();
        Self {
            text: String::new(),
            cursor: 0,
            normal,
            hover,
            focused,
            invalid,
            width,
            height,
            padding: 4,
            text_color: 0x000000, // Black text
            caret_color: 0x000000,
            is_invalid: false,
            caret_visible: true,
            last_blink: Instant::now(),
            on_change_action: None,
            on_submit_action: None,
        }
    }

    /// Set the text padding.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Set the text color.
    pub fn with_text_color(mut self, color: u32) -> Self {
        self.text_color = color;
        self
    }

    /// Set the caret color.
    pub fn with_caret_color(mut self, color: u32) -> Self {
        self.caret_color = color;
        self
    }

    /// Set the on_change action.
    pub fn with_on_change(mut self, action: String) -> Self {
        self.on_change_action = Some(action);
        self
    }

    /// Set the on_submit action.
    pub fn with_on_submit(mut self, action: String) -> Self {
        self.on_submit_action = Some(action);
        self
    }

    /// Get the current text value.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text value.
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor = self.cursor.min(self.text.len());
    }

    /// Mark the input as invalid (e.g., for validation feedback).
    pub fn set_invalid(&mut self, invalid: bool) {
        self.is_invalid = invalid;
    }

    /// Check if the input is marked as invalid.
    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    /// Get the on_change action.
    pub fn on_change_action(&self) -> Option<&str> {
        self.on_change_action.as_deref()
    }

    /// Get the on_submit action.
    pub fn on_submit_action(&self) -> Option<&str> {
        self.on_submit_action.as_deref()
    }

    /// Insert a character at the cursor position.
    /// Returns true if the text was modified.
    fn insert_char(&mut self, c: char) -> bool {
        // Only accept printable ASCII
        if c as u32 >= 32 && c as u32 <= 126 {
            self.text.insert(self.cursor, c);
            self.cursor += 1;
            self.reset_blink();
            return true;
        }
        false
    }

    /// Delete the character before the cursor (backspace).
    /// Returns true if the text was modified.
    fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
            self.reset_blink();
            return true;
        }
        false
    }

    /// Delete the character at the cursor position.
    /// Returns true if the text was modified.
    fn delete(&mut self) -> bool {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
            self.reset_blink();
            return true;
        }
        false
    }

    /// Move cursor left.
    fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.reset_blink();
        }
    }

    /// Move cursor right.
    fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
            self.reset_blink();
        }
    }

    /// Move cursor to the beginning.
    fn move_home(&mut self) {
        self.cursor = 0;
        self.reset_blink();
    }

    /// Move cursor to the end.
    fn move_end(&mut self) {
        self.cursor = self.text.len();
        self.reset_blink();
    }

    /// Reset the blink timer and make the caret visible.
    fn reset_blink(&mut self) {
        self.caret_visible = true;
        self.last_blink = Instant::now();
    }

    /// Update blink state (call this periodically).
    #[allow(dead_code)]
    fn update_blink(&mut self) {
        let elapsed = self.last_blink.elapsed();
        if elapsed.as_millis() >= 530 {
            self.caret_visible = !self.caret_visible;
            self.last_blink = Instant::now();
        }
    }

    /// Set cursor position based on click x position relative to text start.
    #[allow(dead_code)]
    fn set_cursor_from_x(&mut self, click_x: i32, text_start_x: i32) {
        let relative_x = (click_x - text_start_x).max(0) as u32;

        // Find the character position closest to the click
        let mut best_pos = 0;
        let mut best_dist = relative_x;

        for i in 0..=self.text.len() {
            let char_x = caret_x(&self.text, i);
            let dist = if char_x > relative_x {
                char_x - relative_x
            } else {
                relative_x - char_x
            };
            if dist < best_dist {
                best_dist = dist;
                best_pos = i;
            }
        }

        self.cursor = best_pos;
        self.reset_blink();
    }

    fn draw_image(&self, canvas: &mut Canvas, bounds: &Rect, image: &RgbImage) {
        for (ix, iy, pixel) in image.enumerate_pixels() {
            let x = bounds.x + ix as i32;
            let y = bounds.y + iy as i32;

            if x >= bounds.x && x < bounds.right() && y >= bounds.y && y < bounds.bottom() {
                if x >= 0 && y >= 0 {
                    let [r, g, b] = pixel.0;
                    canvas.set_pixel_rgb(x as u32, y as u32, r, g, b);
                }
            }
        }
    }
}

impl Widget for TextInput {
    fn draw(&self, canvas: &mut Canvas, bounds: &Rect, state: WidgetState) {
        // Select background image based on state
        let image = if self.is_invalid && self.invalid.is_some() {
            self.invalid.as_ref().unwrap()
        } else if state.focused {
            &self.focused
        } else if state.hovered {
            &self.hover
        } else {
            &self.normal
        };

        // Draw background
        self.draw_image(canvas, bounds, image);

        // Calculate content rect (with padding)
        let content_rect = Rect::new(
            bounds.x + self.padding as i32,
            bounds.y + self.padding as i32,
            bounds.width.saturating_sub(self.padding * 2),
            bounds.height.saturating_sub(self.padding * 2),
        );

        // Center text vertically
        let text_y = content_rect.y + (content_rect.height as i32 - line_height() as i32) / 2;

        // Draw text clipped to content rect
        draw_text(
            canvas,
            content_rect.x,
            text_y,
            Some(&content_rect),
            &self.text,
            TextStyle::with_color(self.text_color),
        );

        // Draw caret if focused and visible
        if state.focused && self.caret_visible {
            let caret_offset = caret_x(&self.text, self.cursor);
            let caret_x_pos = content_rect.x + caret_offset as i32;
            draw_caret(
                canvas,
                caret_x_pos,
                text_y,
                line_height(),
                Some(&content_rect),
                self.caret_color,
            );
        }
    }

    fn preferred_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn on_event(&mut self, event: &WidgetEvent) -> bool {
        match event {
            WidgetEvent::CharInput { c } => {
                let modified = self.insert_char(*c);
                if modified {
                    if let Some(action) = &self.on_change_action {
                        println!("TextInput change: {} -> {}", action, self.text);
                    }
                }
                modified
            }
            WidgetEvent::KeyDown { key } => {
                let modified = match key {
                    KeyCode::Backspace => self.backspace(),
                    KeyCode::Delete => self.delete(),
                    KeyCode::Left => {
                        self.move_left();
                        false
                    }
                    KeyCode::Right => {
                        self.move_right();
                        false
                    }
                    KeyCode::Home => {
                        self.move_home();
                        false
                    }
                    KeyCode::End => {
                        self.move_end();
                        false
                    }
                    KeyCode::Enter => {
                        if let Some(action) = &self.on_submit_action {
                            println!("TextInput submit: {} -> {}", action, self.text);
                        }
                        false
                    }
                };
                if modified {
                    if let Some(action) = &self.on_change_action {
                        println!("TextInput change: {} -> {}", action, self.text);
                    }
                }
                true // Consume all key events when focused
            }
            WidgetEvent::FocusGained => {
                self.reset_blink();
                true
            }
            WidgetEvent::FocusLost => {
                self.caret_visible = false;
                true
            }
            WidgetEvent::Click => {
                // Request focus handled externally
                true
            }
            WidgetEvent::MouseDown { .. } => {
                // Set cursor position based on click
                // We don't have bounds here, so this is handled in Click
                // For now, just consume the event
                true
            }
            _ => false,
        }
    }
}
