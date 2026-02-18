//! Rendering module for the Winux compositor
//!
//! Provides rendering abstractions and implementations for drawing
//! the compositor output.

use crate::state::WinuxState;
use anyhow::Result;
use smithay::{
    backend::renderer::{
        damage::OutputDamageTracker,
        element::{
            surface::WaylandSurfaceRenderElement,
            AsRenderElements,
        },
        gles::GlesRenderer,
        Bind, Frame, Renderer as SmithayRenderer, Unbind,
    },
    desktop::space::SpaceRenderElements,
    output::Output,
    utils::{Physical, Rectangle, Scale, Transform},
};

/// Renderer trait for the compositor
pub trait Renderer {
    /// Initialize the renderer
    fn init(&mut self) -> Result<()>;

    /// Render a frame
    fn render_frame(&mut self, state: &mut WinuxState, output: &Output) -> Result<bool>;

    /// Get the renderer name
    fn name(&self) -> &str;
}

/// GLES-based renderer implementation
pub struct GlesCompositorRenderer {
    /// The underlying GLES renderer
    renderer: GlesRenderer,
    /// Damage tracker for efficient rendering
    damage_tracker: OutputDamageTracker,
    /// Whether damage tracking is enabled
    damage_tracking_enabled: bool,
    /// Clear color (RGBA)
    clear_color: [f32; 4],
}

impl GlesCompositorRenderer {
    /// Create a new GLES renderer
    pub fn new(renderer: GlesRenderer) -> Self {
        Self {
            renderer,
            damage_tracker: OutputDamageTracker::new(
                (1920, 1080).into(), // Default size, updated on first render
                1.0,
                Transform::Normal,
            ),
            damage_tracking_enabled: true,
            clear_color: [0.1, 0.1, 0.1, 1.0], // Dark gray
        }
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }

    /// Enable or disable damage tracking
    pub fn set_damage_tracking(&mut self, enabled: bool) {
        self.damage_tracking_enabled = enabled;
    }

    /// Get a reference to the underlying renderer
    pub fn renderer(&self) -> &GlesRenderer {
        &self.renderer
    }

    /// Get a mutable reference to the underlying renderer
    pub fn renderer_mut(&mut self) -> &mut GlesRenderer {
        &mut self.renderer
    }

    /// Parse a hex color string to RGBA floats
    pub fn parse_color(hex: &str) -> Option<[f32; 4]> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 8 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;

        Some([r, g, b, a])
    }
}

impl Renderer for GlesCompositorRenderer {
    fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing GLES renderer");
        Ok(())
    }

    fn render_frame(&mut self, state: &mut WinuxState, output: &Output) -> Result<bool> {
        let output_geometry = state.space.output_geometry(output);
        let scale = Scale::from(output.current_scale().fractional_scale());
        let transform = output.current_transform();

        // Update damage tracker if output size changed
        if let Some(geometry) = output_geometry {
            let size = transform.transform_size(geometry.size);
            self.damage_tracker = OutputDamageTracker::new(size, scale.x, transform);
        }

        // Collect render elements from the space
        let render_elements = state
            .space
            .render_elements_for_output(&mut self.renderer, output, scale.x)
            .map_err(|e| anyhow::anyhow!("Failed to get render elements: {:?}", e))?;

        // Render with damage tracking
        let damage = if self.damage_tracking_enabled {
            self.damage_tracker
                .render_output(
                    &mut self.renderer,
                    0, // Age - should be tracked properly
                    &render_elements,
                    self.clear_color,
                )
                .map_err(|e| anyhow::anyhow!("Render failed: {:?}", e))?
        } else {
            // Full redraw without damage tracking
            self.damage_tracker
                .render_output(&mut self.renderer, 0, &render_elements, self.clear_color)
                .map_err(|e| anyhow::anyhow!("Render failed: {:?}", e))?
        };

        Ok(damage.is_some())
    }

    fn name(&self) -> &str {
        "GLES"
    }
}

/// Software renderer for fallback
pub struct SoftwareRenderer {
    /// Frame buffer
    buffer: Vec<u32>,
    /// Buffer dimensions
    dimensions: (u32, u32),
    /// Clear color
    clear_color: u32,
}

impl SoftwareRenderer {
    /// Create a new software renderer
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            buffer: vec![0; (width * height) as usize],
            dimensions: (width, height),
            clear_color: 0xFF1E1E1E, // Dark gray in ARGB
        }
    }

    /// Resize the buffer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.dimensions = (width, height);
        self.buffer.resize((width * height) as usize, 0);
    }

    /// Set clear color from RGBA hex
    pub fn set_clear_color(&mut self, color: u32) {
        self.clear_color = color;
    }

    /// Get the frame buffer
    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    /// Get buffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.fill(self.clear_color);
    }

    /// Draw a rectangle
    pub fn draw_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: u32) {
        let (buf_width, buf_height) = self.dimensions;

        for dy in 0..height {
            for dx in 0..width {
                let px = x + dx as i32;
                let py = y + dy as i32;

                if px >= 0 && px < buf_width as i32 && py >= 0 && py < buf_height as i32 {
                    let idx = (py as u32 * buf_width + px as u32) as usize;
                    self.buffer[idx] = color;
                }
            }
        }
    }

    /// Draw a border rectangle
    pub fn draw_border(&mut self, x: i32, y: i32, width: u32, height: u32, border: u32, color: u32) {
        // Top border
        self.draw_rect(x, y, width, border, color);
        // Bottom border
        self.draw_rect(x, y + height as i32 - border as i32, width, border, color);
        // Left border
        self.draw_rect(x, y, border, height, color);
        // Right border
        self.draw_rect(x + width as i32 - border as i32, y, border, height, color);
    }
}

impl Renderer for SoftwareRenderer {
    fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing software renderer");
        self.clear();
        Ok(())
    }

    fn render_frame(&mut self, state: &mut WinuxState, _output: &Output) -> Result<bool> {
        self.clear();

        // Render windows from the space
        for window in state.space.elements() {
            if let Some(geometry) = state.space.element_geometry(window) {
                // Draw window border
                let border_color = if state.focused_surface().is_some() {
                    0xFF0078D4 // Active - Fluent blue
                } else {
                    0xFF808080 // Inactive - gray
                };

                self.draw_border(
                    geometry.loc.x,
                    geometry.loc.y,
                    geometry.size.w as u32,
                    geometry.size.h as u32,
                    2,
                    border_color,
                );
            }
        }

        Ok(true)
    }

    fn name(&self) -> &str {
        "Software"
    }
}

/// Render element types for the compositor
pub type WinuxRenderElements<R> = SpaceRenderElements<R, WaylandSurfaceRenderElement<R>>;

/// Helper function to convert damage rectangles
pub fn convert_damage(
    damage: &[Rectangle<i32, Physical>],
    scale: Scale<f64>,
) -> Vec<Rectangle<i32, Physical>> {
    damage
        .iter()
        .map(|rect| {
            Rectangle::from_loc_and_size(
                (
                    (rect.loc.x as f64 * scale.x) as i32,
                    (rect.loc.y as f64 * scale.y) as i32,
                ),
                (
                    (rect.size.w as f64 * scale.x).ceil() as i32,
                    (rect.size.h as f64 * scale.y).ceil() as i32,
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        let color = GlesCompositorRenderer::parse_color("#FF0000FF");
        assert_eq!(color, Some([1.0, 0.0, 0.0, 1.0]));

        let color = GlesCompositorRenderer::parse_color("#00FF00FF");
        assert_eq!(color, Some([0.0, 1.0, 0.0, 1.0]));

        let color = GlesCompositorRenderer::parse_color("#0000FFFF");
        assert_eq!(color, Some([0.0, 0.0, 1.0, 1.0]));

        let color = GlesCompositorRenderer::parse_color("#FFFFFF80");
        assert!(color.is_some());
        let c = color.unwrap();
        assert!((c[3] - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_software_renderer() {
        let mut renderer = SoftwareRenderer::new(100, 100);
        renderer.clear();

        // Check buffer is filled with clear color
        assert!(renderer.buffer().iter().all(|&p| p == renderer.clear_color));

        // Draw a rect
        renderer.draw_rect(10, 10, 20, 20, 0xFFFFFFFF);

        // Check that some pixels changed
        assert!(renderer.buffer().iter().any(|&p| p == 0xFFFFFFFF));
    }
}
