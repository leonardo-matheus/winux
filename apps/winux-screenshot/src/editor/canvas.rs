//! Canvas widget for editing screenshots

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use gtk::gdk::RGBA;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

use super::{EditorState, ToolType, DrawingOperation, Point, CropRegion};
use super::tools::Point as ToolPoint;

/// Editor canvas that handles drawing operations
pub struct EditorCanvas {
    pub drawing_area: gtk::DrawingArea,
    image_surface: Rc<RefCell<Option<cairo::ImageSurface>>>,
    editor_state: Rc<RefCell<EditorState>>,
    current_path: Rc<RefCell<Vec<Point>>>,
    start_point: Rc<RefCell<Option<Point>>>,
}

/// Point in canvas coordinates
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl EditorCanvas {
    pub fn new(editor_state: Rc<RefCell<EditorState>>) -> Self {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        let image_surface: Rc<RefCell<Option<cairo::ImageSurface>>> = Rc::new(RefCell::new(None));
        let current_path: Rc<RefCell<Vec<Point>>> = Rc::new(RefCell::new(Vec::new()));
        let start_point: Rc<RefCell<Option<Point>>> = Rc::new(RefCell::new(None));

        // Set up drawing
        {
            let image_surface = image_surface.clone();
            let editor_state = editor_state.clone();
            let current_path = current_path.clone();
            let start_point = start_point.clone();

            drawing_area.set_draw_func(move |_, cr, width, height| {
                Self::draw(
                    cr,
                    width,
                    height,
                    &image_surface,
                    &editor_state,
                    &current_path,
                    &start_point,
                );
            });
        }

        // Mouse event handling
        let gesture_drag = gtk::GestureDrag::new();

        {
            let editor_state = editor_state.clone();
            let current_path = current_path.clone();
            let start_point = start_point.clone();
            let drawing_area_clone = drawing_area.clone();

            gesture_drag.connect_drag_begin(move |_, x, y| {
                let point = Point::new(x, y);
                *start_point.borrow_mut() = Some(point);
                current_path.borrow_mut().clear();
                current_path.borrow_mut().push(point);
                editor_state.borrow_mut().is_drawing = true;
            });
        }

        {
            let editor_state = editor_state.clone();
            let current_path = current_path.clone();
            let start_point = start_point.clone();
            let drawing_area_clone = drawing_area.clone();

            gesture_drag.connect_drag_update(move |_, offset_x, offset_y| {
                if let Some(start) = *start_point.borrow() {
                    let point = Point::new(start.x + offset_x, start.y + offset_y);

                    let tool = editor_state.borrow().current_tool;
                    match tool {
                        ToolType::Pen | ToolType::Highlighter => {
                            current_path.borrow_mut().push(point);
                        }
                        _ => {
                            // For shapes, just update the end point
                            let mut path = current_path.borrow_mut();
                            if path.len() > 1 {
                                path.pop();
                            }
                            path.push(point);
                        }
                    }
                    drawing_area_clone.queue_draw();
                }
            });
        }

        {
            let editor_state = editor_state.clone();
            let current_path = current_path.clone();
            let start_point = start_point.clone();
            let drawing_area_clone = drawing_area.clone();

            gesture_drag.connect_drag_end(move |_, offset_x, offset_y| {
                if let Some(start) = start_point.borrow().take() {
                    let end = Point::new(start.x + offset_x, start.y + offset_y);
                    let path: Vec<Point> = current_path.borrow().clone();

                    let mut state = editor_state.borrow_mut();
                    let operation = Self::create_operation(&state, start, end, &path);

                    if let Some(op) = operation {
                        state.add_operation(op);
                    }

                    state.is_drawing = false;
                    current_path.borrow_mut().clear();
                }
                drawing_area_clone.queue_draw();
            });
        }

        drawing_area.add_controller(gesture_drag);

        Self {
            drawing_area,
            image_surface,
            editor_state,
            current_path,
            start_point,
        }
    }

    fn create_operation(
        state: &EditorState,
        start: Point,
        end: Point,
        path: &[Point],
    ) -> Option<DrawingOperation> {
        let tool = state.current_tool;
        let color = state.color;
        let stroke_width = state.stroke_width;

        match tool {
            ToolType::Arrow => Some(DrawingOperation::Arrow {
                start: ToolPoint::new(start.x, start.y),
                end: ToolPoint::new(end.x, end.y),
                color,
                stroke_width,
            }),
            ToolType::Line => Some(DrawingOperation::Line {
                start: ToolPoint::new(start.x, start.y),
                end: ToolPoint::new(end.x, end.y),
                color,
                stroke_width,
            }),
            ToolType::Rectangle => Some(DrawingOperation::Rectangle {
                x: start.x.min(end.x),
                y: start.y.min(end.y),
                width: (end.x - start.x).abs(),
                height: (end.y - start.y).abs(),
                color,
                stroke_width,
                filled: false,
            }),
            ToolType::FilledRectangle => Some(DrawingOperation::Rectangle {
                x: start.x.min(end.x),
                y: start.y.min(end.y),
                width: (end.x - start.x).abs(),
                height: (end.y - start.y).abs(),
                color,
                stroke_width,
                filled: true,
            }),
            ToolType::Circle => {
                let cx = (start.x + end.x) / 2.0;
                let cy = (start.y + end.y) / 2.0;
                let radius = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt() / 2.0;
                Some(DrawingOperation::Circle {
                    cx,
                    cy,
                    radius,
                    color,
                    stroke_width,
                    filled: false,
                })
            }
            ToolType::Pen => Some(DrawingOperation::Pen {
                points: path.iter().map(|p| ToolPoint::new(p.x, p.y)).collect(),
                color,
                stroke_width,
            }),
            ToolType::Highlighter => Some(DrawingOperation::Highlighter {
                points: path.iter().map(|p| ToolPoint::new(p.x, p.y)).collect(),
                color,
                stroke_width: stroke_width * 3.0,
            }),
            ToolType::Blur => Some(DrawingOperation::Blur {
                x: start.x.min(end.x),
                y: start.y.min(end.y),
                width: (end.x - start.x).abs(),
                height: (end.y - start.y).abs(),
                strength: state.blur_strength,
            }),
            ToolType::Pixelate => Some(DrawingOperation::Pixelate {
                x: start.x.min(end.x),
                y: start.y.min(end.y),
                width: (end.x - start.x).abs(),
                height: (end.y - start.y).abs(),
                block_size: state.blur_strength as f64 * 2.0,
            }),
            ToolType::Text => {
                // Text is handled separately via dialog
                None
            }
            ToolType::Crop => {
                // Crop updates the crop region
                None
            }
        }
    }

    /// Load an image into the canvas
    pub fn load_image(&self, path: &PathBuf) -> Result<(), anyhow::Error> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let width = rgba.width() as i32;
        let height = rgba.height() as i32;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)?;

        {
            let mut data = surface.data()?;
            for (i, pixel) in rgba.pixels().enumerate() {
                let offset = i * 4;
                // Cairo uses BGRA format
                data[offset] = pixel[2];     // B
                data[offset + 1] = pixel[1]; // G
                data[offset + 2] = pixel[0]; // R
                data[offset + 3] = pixel[3]; // A
            }
        }

        surface.mark_dirty();

        *self.image_surface.borrow_mut() = Some(surface);
        self.drawing_area.set_content_width(width);
        self.drawing_area.set_content_height(height);
        self.drawing_area.queue_draw();

        Ok(())
    }

    fn draw(
        cr: &cairo::Context,
        width: i32,
        height: i32,
        image_surface: &Rc<RefCell<Option<cairo::ImageSurface>>>,
        editor_state: &Rc<RefCell<EditorState>>,
        current_path: &Rc<RefCell<Vec<Point>>>,
        start_point: &Rc<RefCell<Option<Point>>>,
    ) {
        // Clear background
        cr.set_source_rgb(0.1, 0.1, 0.1);
        cr.paint().ok();

        // Draw the image
        if let Some(ref surface) = *image_surface.borrow() {
            cr.set_source_surface(surface, 0.0, 0.0).ok();
            cr.paint().ok();
        }

        // Draw all completed operations
        let state = editor_state.borrow();
        for op in state.visible_operations() {
            Self::draw_operation(cr, op);
        }

        // Draw current operation in progress
        if state.is_drawing {
            if let Some(start) = *start_point.borrow() {
                let path = current_path.borrow();
                if !path.is_empty() {
                    let end = path.last().unwrap();
                    Self::draw_preview(cr, &state, start, *end, &path);
                }
            }
        }

        // Draw crop overlay if in crop mode
        if let Some(crop) = state.crop_region {
            Self::draw_crop_overlay(cr, width as f64, height as f64, &crop);
        }
    }

    fn draw_operation(cr: &cairo::Context, op: &DrawingOperation) {
        match op {
            DrawingOperation::Arrow { start, end, color, stroke_width } => {
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
                cr.set_line_width(*stroke_width);

                // Draw line
                cr.move_to(start.x, start.y);
                cr.line_to(end.x, end.y);
                cr.stroke().ok();

                // Draw arrow head
                let angle = (end.y - start.y).atan2(end.x - start.x);
                let arrow_size = stroke_width * 4.0;

                cr.move_to(end.x, end.y);
                cr.line_to(
                    end.x - arrow_size * (angle - 0.5).cos(),
                    end.y - arrow_size * (angle - 0.5).sin(),
                );
                cr.move_to(end.x, end.y);
                cr.line_to(
                    end.x - arrow_size * (angle + 0.5).cos(),
                    end.y - arrow_size * (angle + 0.5).sin(),
                );
                cr.stroke().ok();
            }
            DrawingOperation::Line { start, end, color, stroke_width } => {
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
                cr.set_line_width(*stroke_width);
                cr.move_to(start.x, start.y);
                cr.line_to(end.x, end.y);
                cr.stroke().ok();
            }
            DrawingOperation::Rectangle { x, y, width, height, color, stroke_width, filled } => {
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
                cr.set_line_width(*stroke_width);
                cr.rectangle(*x, *y, *width, *height);
                if *filled {
                    cr.fill().ok();
                } else {
                    cr.stroke().ok();
                }
            }
            DrawingOperation::Circle { cx, cy, radius, color, stroke_width, filled } => {
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
                cr.set_line_width(*stroke_width);
                cr.arc(*cx, *cy, *radius, 0.0, 2.0 * std::f64::consts::PI);
                if *filled {
                    cr.fill().ok();
                } else {
                    cr.stroke().ok();
                }
            }
            DrawingOperation::Pen { points, color, stroke_width } => {
                if points.is_empty() {
                    return;
                }
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
                cr.set_line_width(*stroke_width);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);

                cr.move_to(points[0].x, points[0].y);
                for point in &points[1..] {
                    cr.line_to(point.x, point.y);
                }
                cr.stroke().ok();
            }
            DrawingOperation::Highlighter { points, color, stroke_width } => {
                if points.is_empty() {
                    return;
                }
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), 0.4);
                cr.set_line_width(*stroke_width);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);

                cr.move_to(points[0].x, points[0].y);
                for point in &points[1..] {
                    cr.line_to(point.x, point.y);
                }
                cr.stroke().ok();
            }
            DrawingOperation::Text { x, y, text, color, font_size } => {
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
                cr.set_font_size(*font_size);
                cr.move_to(*x, *y);
                cr.show_text(text).ok();
            }
            DrawingOperation::Blur { x, y, width, height, strength: _ } => {
                // Draw blur preview as translucent rectangle
                cr.set_source_rgba(0.5, 0.5, 0.5, 0.3);
                cr.rectangle(*x, *y, *width, *height);
                cr.fill().ok();
            }
            DrawingOperation::Pixelate { x, y, width, height, block_size } => {
                // Draw pixelate preview as grid
                cr.set_source_rgba(0.3, 0.3, 0.3, 0.3);
                cr.rectangle(*x, *y, *width, *height);
                cr.fill().ok();

                cr.set_source_rgba(0.5, 0.5, 0.5, 0.5);
                cr.set_line_width(1.0);
                let mut gx = *x;
                while gx < x + width {
                    cr.move_to(gx, *y);
                    cr.line_to(gx, y + height);
                    gx += *block_size;
                }
                let mut gy = *y;
                while gy < y + height {
                    cr.move_to(*x, gy);
                    cr.line_to(x + width, gy);
                    gy += *block_size;
                }
                cr.stroke().ok();
            }
        }
    }

    fn draw_preview(
        cr: &cairo::Context,
        state: &EditorState,
        start: Point,
        end: Point,
        path: &[Point],
    ) {
        let color = &state.color;
        let stroke_width = state.stroke_width;

        cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), color.alpha().into());
        cr.set_line_width(stroke_width);

        match state.current_tool {
            ToolType::Arrow => {
                cr.move_to(start.x, start.y);
                cr.line_to(end.x, end.y);
                cr.stroke().ok();

                let angle = (end.y - start.y).atan2(end.x - start.x);
                let arrow_size = stroke_width * 4.0;

                cr.move_to(end.x, end.y);
                cr.line_to(
                    end.x - arrow_size * (angle - 0.5).cos(),
                    end.y - arrow_size * (angle - 0.5).sin(),
                );
                cr.move_to(end.x, end.y);
                cr.line_to(
                    end.x - arrow_size * (angle + 0.5).cos(),
                    end.y - arrow_size * (angle + 0.5).sin(),
                );
                cr.stroke().ok();
            }
            ToolType::Line => {
                cr.move_to(start.x, start.y);
                cr.line_to(end.x, end.y);
                cr.stroke().ok();
            }
            ToolType::Rectangle | ToolType::FilledRectangle => {
                let x = start.x.min(end.x);
                let y = start.y.min(end.y);
                let w = (end.x - start.x).abs();
                let h = (end.y - start.y).abs();
                cr.rectangle(x, y, w, h);
                if state.current_tool == ToolType::FilledRectangle {
                    cr.fill().ok();
                } else {
                    cr.stroke().ok();
                }
            }
            ToolType::Circle => {
                let cx = (start.x + end.x) / 2.0;
                let cy = (start.y + end.y) / 2.0;
                let radius = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt() / 2.0;
                cr.arc(cx, cy, radius, 0.0, 2.0 * std::f64::consts::PI);
                cr.stroke().ok();
            }
            ToolType::Pen => {
                if path.is_empty() {
                    return;
                }
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                cr.move_to(path[0].x, path[0].y);
                for point in &path[1..] {
                    cr.line_to(point.x, point.y);
                }
                cr.stroke().ok();
            }
            ToolType::Highlighter => {
                if path.is_empty() {
                    return;
                }
                cr.set_source_rgba(color.red().into(), color.green().into(), color.blue().into(), 0.4);
                cr.set_line_width(stroke_width * 3.0);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                cr.move_to(path[0].x, path[0].y);
                for point in &path[1..] {
                    cr.line_to(point.x, point.y);
                }
                cr.stroke().ok();
            }
            ToolType::Blur | ToolType::Pixelate => {
                let x = start.x.min(end.x);
                let y = start.y.min(end.y);
                let w = (end.x - start.x).abs();
                let h = (end.y - start.y).abs();
                cr.set_source_rgba(0.5, 0.5, 0.5, 0.3);
                cr.rectangle(x, y, w, h);
                cr.fill().ok();

                // Dashed border
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.8);
                cr.set_line_width(1.0);
                cr.set_dash(&[5.0, 5.0], 0.0);
                cr.rectangle(x, y, w, h);
                cr.stroke().ok();
                cr.set_dash(&[], 0.0);
            }
            _ => {}
        }
    }

    fn draw_crop_overlay(cr: &cairo::Context, width: f64, height: f64, crop: &CropRegion) {
        // Darken areas outside crop region
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.6);

        // Top
        cr.rectangle(0.0, 0.0, width, crop.y);
        cr.fill().ok();

        // Bottom
        cr.rectangle(0.0, crop.y + crop.height, width, height - crop.y - crop.height);
        cr.fill().ok();

        // Left
        cr.rectangle(0.0, crop.y, crop.x, crop.height);
        cr.fill().ok();

        // Right
        cr.rectangle(crop.x + crop.width, crop.y, width - crop.x - crop.width, crop.height);
        cr.fill().ok();

        // Crop border
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_line_width(2.0);
        cr.rectangle(crop.x, crop.y, crop.width, crop.height);
        cr.stroke().ok();

        // Corner handles
        let handle_size = 10.0;
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);

        // Top-left
        cr.rectangle(crop.x - handle_size / 2.0, crop.y - handle_size / 2.0, handle_size, handle_size);
        cr.fill().ok();

        // Top-right
        cr.rectangle(crop.x + crop.width - handle_size / 2.0, crop.y - handle_size / 2.0, handle_size, handle_size);
        cr.fill().ok();

        // Bottom-left
        cr.rectangle(crop.x - handle_size / 2.0, crop.y + crop.height - handle_size / 2.0, handle_size, handle_size);
        cr.fill().ok();

        // Bottom-right
        cr.rectangle(crop.x + crop.width - handle_size / 2.0, crop.y + crop.height - handle_size / 2.0, handle_size, handle_size);
        cr.fill().ok();

        // Dimensions label
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(12.0);
        let label = format!("{}x{}", crop.width as i32, crop.height as i32);
        cr.move_to(crop.x + 5.0, crop.y + crop.height - 5.0);
        cr.show_text(&label).ok();
    }

    /// Export the canvas to an image file
    pub fn export(&self, path: &PathBuf) -> Result<(), anyhow::Error> {
        let surface = self.image_surface.borrow();
        if let Some(ref surf) = *surface {
            let width = surf.width();
            let height = surf.height();

            // Create a new surface for export
            let export_surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)?;
            let cr = cairo::Context::new(&export_surface)?;

            // Draw base image
            cr.set_source_surface(surf, 0.0, 0.0)?;
            cr.paint()?;

            // Draw all operations
            let state = self.editor_state.borrow();
            for op in state.visible_operations() {
                Self::draw_operation(&cr, op);
            }

            // Apply crop if set
            let final_surface = if let Some(crop) = state.crop_region {
                let cropped = cairo::ImageSurface::create(
                    cairo::Format::ARgb32,
                    crop.width as i32,
                    crop.height as i32,
                )?;
                let crop_cr = cairo::Context::new(&cropped)?;
                crop_cr.set_source_surface(&export_surface, -crop.x, -crop.y)?;
                crop_cr.paint()?;
                cropped
            } else {
                export_surface
            };

            // Save to file
            let mut file = std::fs::File::create(path)?;
            final_surface.write_to_png(&mut file)?;
        }

        Ok(())
    }

    /// Get the widget
    pub fn widget(&self) -> &gtk::DrawingArea {
        &self.drawing_area
    }

    /// Refresh the canvas
    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
    }
}
