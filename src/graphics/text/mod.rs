use std::path::Path;
use std::sync::OnceLock;

use fontdue::{Font, FontSettings};

use crate::core::Rect;
use crate::graphics::Canvas;

/// Global font instance.
static FONT: OnceLock<Font> = OnceLock::new();
static FONT_SIZE: OnceLock<f32> = OnceLock::new();

/// Initialize the font system with a TTF file.
/// Must be called before any text rendering.
pub fn init_font(path: &Path, size: f32) -> Result<(), FontError> {
    let font_data = std::fs::read(path).map_err(|e| FontError::Io(e))?;
    let font = Font::from_bytes(font_data, FontSettings::default())
        .map_err(|e| FontError::Parse(e.to_string()))?;

    FONT.set(font).map_err(|_| FontError::AlreadyInitialized)?;
    FONT_SIZE.set(size).map_err(|_| FontError::AlreadyInitialized)?;

    Ok(())
}

/// Get the loaded font, panics if not initialized.
fn get_font() -> &'static Font {
    FONT.get().expect("Font not initialized. Call init_font() first.")
}

/// Get the font size.
fn get_font_size() -> f32 {
    *FONT_SIZE.get().expect("Font not initialized. Call init_font() first.")
}

/// Get the line height for the current font.
pub fn line_height() -> u32 {
    let font = get_font();
    let size = get_font_size();
    let metrics = font.horizontal_line_metrics(size).unwrap_or(fontdue::LineMetrics {
        ascent: size,
        descent: 0.0,
        line_gap: 0.0,
        new_line_size: size,
    });
    metrics.new_line_size.ceil() as u32
}

/// Text style for rendering.
#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub color: u32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self { color: 0xFFFFFF } // White
    }
}

impl TextStyle {
    pub fn with_color(color: u32) -> Self {
        Self { color }
    }
}

/// Measure the width of a string in pixels.
pub fn measure_text(text: &str) -> (u32, u32) {
    let font = get_font();
    let size = get_font_size();

    if text.is_empty() {
        return (0, line_height());
    }

    let mut width = 0.0;
    for c in text.chars() {
        let metrics = font.metrics(c, size);
        width += metrics.advance_width;
    }

    (width.ceil() as u32, line_height())
}

/// Get the x offset of the caret at the given character index.
pub fn caret_x(text: &str, cursor_index: usize) -> u32 {
    let font = get_font();
    let size = get_font_size();

    let mut x = 0.0;
    for (i, c) in text.chars().enumerate() {
        if i >= cursor_index {
            break;
        }
        let metrics = font.metrics(c, size);
        x += metrics.advance_width;
    }

    x.ceil() as u32
}

/// Draw text to the canvas at the given position.
/// Clips rendering to the optional clip_rect.
pub fn draw_text(
    canvas: &mut Canvas,
    x: i32,
    y: i32,
    clip_rect: Option<&Rect>,
    text: &str,
    style: TextStyle,
) {
    let font = get_font();
    let size = get_font_size();

    let mut cursor_x = x as f32;

    // Get baseline offset
    let metrics = font.horizontal_line_metrics(size).unwrap_or(fontdue::LineMetrics {
        ascent: size,
        descent: 0.0,
        line_gap: 0.0,
        new_line_size: size,
    });
    let baseline_y = y as f32 + metrics.ascent;

    for c in text.chars() {
        let (glyph_metrics, bitmap) = font.rasterize(c, size);

        // Calculate glyph position
        let glyph_x = cursor_x + glyph_metrics.xmin as f32;
        let glyph_y = baseline_y - glyph_metrics.height as f32 - glyph_metrics.ymin as f32;

        // Draw the glyph bitmap
        for row in 0..glyph_metrics.height {
            for col in 0..glyph_metrics.width {
                let alpha = bitmap[row * glyph_metrics.width + col];
                if alpha > 0 {
                    let px = glyph_x as i32 + col as i32;
                    let py = glyph_y as i32 + row as i32;

                    // Clip to rect if provided
                    if let Some(clip) = clip_rect {
                        if px < clip.x || px >= clip.right() || py < clip.y || py >= clip.bottom() {
                            continue;
                        }
                    }

                    // Clip to canvas and draw with alpha blending
                    if px >= 0 && py >= 0 && (px as u32) < canvas.width() && (py as u32) < canvas.height() {
                        if alpha == 255 {
                            canvas.set_pixel(px as u32, py as u32, style.color);
                        } else {
                            // Simple alpha blend with black background
                            let r = ((style.color >> 16) & 0xFF) as u32 * alpha as u32 / 255;
                            let g = ((style.color >> 8) & 0xFF) as u32 * alpha as u32 / 255;
                            let b = (style.color & 0xFF) as u32 * alpha as u32 / 255;
                            let blended = (r << 16) | (g << 8) | b;
                            canvas.set_pixel(px as u32, py as u32, blended);
                        }
                    }
                }
            }
        }

        cursor_x += glyph_metrics.advance_width;
    }
}

/// Draw a vertical caret (cursor) at the given position.
pub fn draw_caret(
    canvas: &mut Canvas,
    x: i32,
    y: i32,
    height: u32,
    clip_rect: Option<&Rect>,
    color: u32,
) {
    for row in 0..height {
        let px = x;
        let py = y + row as i32;

        // Clip to rect if provided
        if let Some(clip) = clip_rect {
            if px < clip.x || px >= clip.right() || py < clip.y || py >= clip.bottom() {
                continue;
            }
        }

        // Clip to canvas
        if px >= 0 && py >= 0 && (px as u32) < canvas.width() && (py as u32) < canvas.height() {
            canvas.set_pixel(px as u32, py as u32, color);
        }
    }
}

/// Errors that can occur when loading fonts.
#[derive(Debug)]
pub enum FontError {
    Io(std::io::Error),
    Parse(String),
    AlreadyInitialized,
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::Io(e) => write!(f, "IO error: {}", e),
            FontError::Parse(e) => write!(f, "Font parse error: {}", e),
            FontError::AlreadyInitialized => write!(f, "Font already initialized"),
        }
    }
}

impl std::error::Error for FontError {}
