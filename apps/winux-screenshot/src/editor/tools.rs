//! Drawing tools and operations

use gtk4::gdk::RGBA;
use serde::{Deserialize, Serialize};

/// Tool types available in the editor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolType {
    #[default]
    Arrow,
    Line,
    Rectangle,
    FilledRectangle,
    Circle,
    Pen,
    Highlighter,
    Text,
    Blur,
    Pixelate,
    Crop,
}

impl ToolType {
    pub fn icon(&self) -> &'static str {
        match self {
            ToolType::Arrow => "go-next-symbolic",
            ToolType::Line => "draw-line-symbolic",
            ToolType::Rectangle => "draw-rectangle-symbolic",
            ToolType::FilledRectangle => "view-fullscreen-symbolic",
            ToolType::Circle => "draw-ellipse-symbolic",
            ToolType::Pen => "draw-freehand-symbolic",
            ToolType::Highlighter => "format-text-highlight-symbolic",
            ToolType::Text => "format-text-symbolic",
            ToolType::Blur => "blur-symbolic",
            ToolType::Pixelate => "view-grid-symbolic",
            ToolType::Crop => "edit-cut-symbolic",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ToolType::Arrow => "Arrow",
            ToolType::Line => "Line",
            ToolType::Rectangle => "Rectangle",
            ToolType::FilledRectangle => "Filled Rectangle",
            ToolType::Circle => "Circle",
            ToolType::Pen => "Pen",
            ToolType::Highlighter => "Highlighter",
            ToolType::Text => "Text",
            ToolType::Blur => "Blur",
            ToolType::Pixelate => "Pixelate",
            ToolType::Crop => "Crop",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            ToolType::Arrow => "Draw an arrow",
            ToolType::Line => "Draw a straight line",
            ToolType::Rectangle => "Draw a rectangle outline",
            ToolType::FilledRectangle => "Draw a filled rectangle",
            ToolType::Circle => "Draw a circle/ellipse",
            ToolType::Pen => "Freehand drawing",
            ToolType::Highlighter => "Highlight text or areas",
            ToolType::Text => "Add text annotation",
            ToolType::Blur => "Blur to hide sensitive info",
            ToolType::Pixelate => "Pixelate to hide sensitive info",
            ToolType::Crop => "Crop the screenshot",
        }
    }

    /// All available tools
    pub fn all() -> &'static [ToolType] {
        &[
            ToolType::Arrow,
            ToolType::Line,
            ToolType::Rectangle,
            ToolType::FilledRectangle,
            ToolType::Circle,
            ToolType::Pen,
            ToolType::Highlighter,
            ToolType::Text,
            ToolType::Blur,
            ToolType::Pixelate,
            ToolType::Crop,
        ]
    }
}

/// Generic tool trait
pub trait Tool {
    fn tool_type(&self) -> ToolType;
    fn icon(&self) -> &'static str;
    fn label(&self) -> &'static str;
}

/// A point in 2D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn midpoint(&self, other: &Point) -> Point {
        Point {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }
}

/// A drawing operation that can be serialized and replayed
#[derive(Debug, Clone)]
pub enum DrawingOperation {
    Arrow {
        start: Point,
        end: Point,
        color: RGBA,
        stroke_width: f64,
    },
    Line {
        start: Point,
        end: Point,
        color: RGBA,
        stroke_width: f64,
    },
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: RGBA,
        stroke_width: f64,
        filled: bool,
    },
    Circle {
        cx: f64,
        cy: f64,
        radius: f64,
        color: RGBA,
        stroke_width: f64,
        filled: bool,
    },
    Pen {
        points: Vec<Point>,
        color: RGBA,
        stroke_width: f64,
    },
    Highlighter {
        points: Vec<Point>,
        color: RGBA,
        stroke_width: f64,
    },
    Text {
        x: f64,
        y: f64,
        text: String,
        color: RGBA,
        font_size: f64,
    },
    Blur {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        strength: u32,
    },
    Pixelate {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        block_size: f64,
    },
}

impl DrawingOperation {
    /// Get a bounding box for this operation
    pub fn bounding_box(&self) -> (f64, f64, f64, f64) {
        match self {
            DrawingOperation::Arrow { start, end, stroke_width, .. } |
            DrawingOperation::Line { start, end, stroke_width, .. } => {
                let min_x = start.x.min(end.x) - stroke_width;
                let min_y = start.y.min(end.y) - stroke_width;
                let max_x = start.x.max(end.x) + stroke_width;
                let max_y = start.y.max(end.y) + stroke_width;
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
            DrawingOperation::Rectangle { x, y, width, height, stroke_width, .. } => {
                (x - stroke_width, y - stroke_width, width + stroke_width * 2.0, height + stroke_width * 2.0)
            }
            DrawingOperation::Circle { cx, cy, radius, stroke_width, .. } => {
                let half = radius + stroke_width;
                (cx - half, cy - half, half * 2.0, half * 2.0)
            }
            DrawingOperation::Pen { points, stroke_width, .. } |
            DrawingOperation::Highlighter { points, stroke_width, .. } => {
                if points.is_empty() {
                    return (0.0, 0.0, 0.0, 0.0);
                }
                let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min) - stroke_width;
                let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min) - stroke_width;
                let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + stroke_width;
                let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max) + stroke_width;
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
            DrawingOperation::Text { x, y, font_size, text, .. } => {
                // Approximate text bounds
                let width = text.len() as f64 * font_size * 0.6;
                let height = *font_size * 1.2;
                (*x, y - height, width, height)
            }
            DrawingOperation::Blur { x, y, width, height, .. } |
            DrawingOperation::Pixelate { x, y, width, height, .. } => {
                (*x, *y, *width, *height)
            }
        }
    }
}
