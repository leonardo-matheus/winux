//! Editor module - annotation and editing tools for screenshots

mod canvas;
mod tools;
mod blur;
mod crop;

pub use canvas::EditorCanvas;
pub use tools::{Tool, ToolType, DrawingOperation};
pub use blur::BlurEffect;
pub use crop::CropTool;

use gtk4::gdk::RGBA;

/// Current state of the editor
#[derive(Debug, Clone)]
pub struct EditorState {
    /// Currently selected tool
    pub current_tool: ToolType,
    /// Current drawing color
    pub color: RGBA,
    /// Current stroke width
    pub stroke_width: f64,
    /// Font size for text
    pub font_size: f64,
    /// List of operations (for undo/redo)
    pub operations: Vec<DrawingOperation>,
    /// Undo stack position
    pub undo_position: usize,
    /// Whether to copy to clipboard after save
    pub copy_to_clipboard: bool,
    /// Blur strength (1-10)
    pub blur_strength: u32,
    /// Whether currently drawing
    pub is_drawing: bool,
    /// Current crop selection
    pub crop_region: Option<CropRegion>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_tool: ToolType::Arrow,
            color: RGBA::new(1.0, 0.2, 0.2, 1.0), // Red
            stroke_width: 3.0,
            font_size: 18.0,
            operations: Vec::new(),
            undo_position: 0,
            copy_to_clipboard: false,
            blur_strength: 5,
            is_drawing: false,
            crop_region: None,
        }
    }
}

impl EditorState {
    /// Add a new drawing operation
    pub fn add_operation(&mut self, op: DrawingOperation) {
        // Remove any undone operations
        self.operations.truncate(self.undo_position);
        self.operations.push(op);
        self.undo_position = self.operations.len();
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> bool {
        if self.undo_position > 0 {
            self.undo_position -= 1;
            true
        } else {
            false
        }
    }

    /// Redo a previously undone operation
    pub fn redo(&mut self) -> bool {
        if self.undo_position < self.operations.len() {
            self.undo_position += 1;
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo_position > 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.undo_position < self.operations.len()
    }

    /// Get all visible operations (up to undo position)
    pub fn visible_operations(&self) -> &[DrawingOperation] {
        &self.operations[..self.undo_position]
    }

    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
        self.undo_position = 0;
        self.crop_region = None;
    }
}

/// Crop region definition
#[derive(Debug, Clone, Copy)]
pub struct CropRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl CropRegion {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_points(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x1 - x2).abs();
        let height = (y1 - y2).abs();
        Self { x, y, width, height }
    }

    pub fn is_valid(&self) -> bool {
        self.width > 10.0 && self.height > 10.0
    }
}

/// Predefined colors for the color palette
pub const COLOR_PALETTE: &[RGBA] = &[
    RGBA { red: 1.0, green: 0.2, blue: 0.2, alpha: 1.0 },   // Red
    RGBA { red: 1.0, green: 0.5, blue: 0.0, alpha: 1.0 },   // Orange
    RGBA { red: 1.0, green: 0.9, blue: 0.0, alpha: 1.0 },   // Yellow
    RGBA { red: 0.2, green: 0.8, blue: 0.2, alpha: 1.0 },   // Green
    RGBA { red: 0.2, green: 0.6, blue: 1.0, alpha: 1.0 },   // Blue
    RGBA { red: 0.6, green: 0.2, blue: 0.8, alpha: 1.0 },   // Purple
    RGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 },   // White
    RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 },   // Black
];
