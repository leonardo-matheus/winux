//! Crop tool for trimming screenshots

use image::{DynamicImage, GenericImageView};

use super::CropRegion;

/// Crop tool for selecting and applying crop regions
pub struct CropTool;

impl CropTool {
    /// Apply crop to an image
    pub fn apply(image: &DynamicImage, region: &CropRegion) -> DynamicImage {
        let img_width = image.width();
        let img_height = image.height();

        // Clamp and validate region
        let x = (region.x as u32).min(img_width);
        let y = (region.y as u32).min(img_height);
        let width = (region.width as u32).min(img_width.saturating_sub(x));
        let height = (region.height as u32).min(img_height.saturating_sub(y));

        if width == 0 || height == 0 {
            return image.clone();
        }

        image.crop_imm(x, y, width, height)
    }

    /// Calculate crop region from aspect ratio
    pub fn aspect_ratio_region(
        image_width: f64,
        image_height: f64,
        aspect_width: f64,
        aspect_height: f64,
    ) -> CropRegion {
        let target_ratio = aspect_width / aspect_height;
        let current_ratio = image_width / image_height;

        let (new_width, new_height) = if current_ratio > target_ratio {
            // Image is wider than target, crop width
            let new_width = image_height * target_ratio;
            (new_width, image_height)
        } else {
            // Image is taller than target, crop height
            let new_height = image_width / target_ratio;
            (image_width, new_height)
        };

        let x = (image_width - new_width) / 2.0;
        let y = (image_height - new_height) / 2.0;

        CropRegion::new(x, y, new_width, new_height)
    }

    /// Common aspect ratios
    pub fn aspect_ratios() -> &'static [(&'static str, f64, f64)] {
        &[
            ("Free", 0.0, 0.0),
            ("1:1 (Square)", 1.0, 1.0),
            ("4:3", 4.0, 3.0),
            ("16:9", 16.0, 9.0),
            ("16:10", 16.0, 10.0),
            ("21:9", 21.0, 9.0),
            ("3:2", 3.0, 2.0),
            ("2:3 (Portrait)", 2.0, 3.0),
            ("9:16 (Phone)", 9.0, 16.0),
        ]
    }
}

/// Handle positions for crop region resizing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropHandle {
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Move,
}

impl CropHandle {
    /// Get the cursor name for this handle
    pub fn cursor_name(&self) -> &'static str {
        match self {
            CropHandle::None => "default",
            CropHandle::TopLeft => "nw-resize",
            CropHandle::TopRight => "ne-resize",
            CropHandle::BottomLeft => "sw-resize",
            CropHandle::BottomRight => "se-resize",
            CropHandle::Top => "n-resize",
            CropHandle::Bottom => "s-resize",
            CropHandle::Left => "w-resize",
            CropHandle::Right => "e-resize",
            CropHandle::Move => "move",
        }
    }

    /// Detect which handle is at the given position
    pub fn at_position(x: f64, y: f64, region: &CropRegion, handle_size: f64) -> CropHandle {
        let half = handle_size / 2.0;

        // Check corners first (they take priority)
        if Self::is_near(x, y, region.x, region.y, half) {
            return CropHandle::TopLeft;
        }
        if Self::is_near(x, y, region.x + region.width, region.y, half) {
            return CropHandle::TopRight;
        }
        if Self::is_near(x, y, region.x, region.y + region.height, half) {
            return CropHandle::BottomLeft;
        }
        if Self::is_near(x, y, region.x + region.width, region.y + region.height, half) {
            return CropHandle::BottomRight;
        }

        // Check edges
        if Self::is_near_line_horizontal(x, y, region.x, region.x + region.width, region.y, half) {
            return CropHandle::Top;
        }
        if Self::is_near_line_horizontal(x, y, region.x, region.x + region.width, region.y + region.height, half) {
            return CropHandle::Bottom;
        }
        if Self::is_near_line_vertical(x, y, region.y, region.y + region.height, region.x, half) {
            return CropHandle::Left;
        }
        if Self::is_near_line_vertical(x, y, region.y, region.y + region.height, region.x + region.width, half) {
            return CropHandle::Right;
        }

        // Check if inside region
        if x >= region.x && x <= region.x + region.width &&
           y >= region.y && y <= region.y + region.height {
            return CropHandle::Move;
        }

        CropHandle::None
    }

    fn is_near(x: f64, y: f64, target_x: f64, target_y: f64, threshold: f64) -> bool {
        (x - target_x).abs() <= threshold && (y - target_y).abs() <= threshold
    }

    fn is_near_line_horizontal(x: f64, y: f64, x1: f64, x2: f64, line_y: f64, threshold: f64) -> bool {
        (y - line_y).abs() <= threshold && x >= x1 && x <= x2
    }

    fn is_near_line_vertical(x: f64, y: f64, y1: f64, y2: f64, line_x: f64, threshold: f64) -> bool {
        (x - line_x).abs() <= threshold && y >= y1 && y <= y2
    }

    /// Update crop region based on handle drag
    pub fn update_region(
        &self,
        region: &mut CropRegion,
        delta_x: f64,
        delta_y: f64,
        image_width: f64,
        image_height: f64,
    ) {
        match self {
            CropHandle::TopLeft => {
                let new_x = (region.x + delta_x).max(0.0).min(region.x + region.width - 10.0);
                let new_y = (region.y + delta_y).max(0.0).min(region.y + region.height - 10.0);
                region.width += region.x - new_x;
                region.height += region.y - new_y;
                region.x = new_x;
                region.y = new_y;
            }
            CropHandle::TopRight => {
                let new_width = (region.width + delta_x).max(10.0).min(image_width - region.x);
                let new_y = (region.y + delta_y).max(0.0).min(region.y + region.height - 10.0);
                region.width = new_width;
                region.height += region.y - new_y;
                region.y = new_y;
            }
            CropHandle::BottomLeft => {
                let new_x = (region.x + delta_x).max(0.0).min(region.x + region.width - 10.0);
                let new_height = (region.height + delta_y).max(10.0).min(image_height - region.y);
                region.width += region.x - new_x;
                region.x = new_x;
                region.height = new_height;
            }
            CropHandle::BottomRight => {
                region.width = (region.width + delta_x).max(10.0).min(image_width - region.x);
                region.height = (region.height + delta_y).max(10.0).min(image_height - region.y);
            }
            CropHandle::Top => {
                let new_y = (region.y + delta_y).max(0.0).min(region.y + region.height - 10.0);
                region.height += region.y - new_y;
                region.y = new_y;
            }
            CropHandle::Bottom => {
                region.height = (region.height + delta_y).max(10.0).min(image_height - region.y);
            }
            CropHandle::Left => {
                let new_x = (region.x + delta_x).max(0.0).min(region.x + region.width - 10.0);
                region.width += region.x - new_x;
                region.x = new_x;
            }
            CropHandle::Right => {
                region.width = (region.width + delta_x).max(10.0).min(image_width - region.x);
            }
            CropHandle::Move => {
                region.x = (region.x + delta_x).max(0.0).min(image_width - region.width);
                region.y = (region.y + delta_y).max(0.0).min(image_height - region.height);
            }
            CropHandle::None => {}
        }
    }
}
