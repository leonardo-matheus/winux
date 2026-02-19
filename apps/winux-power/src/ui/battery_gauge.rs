// Circular battery gauge widget for Winux Power

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label};
use std::cell::Cell;
use std::f64::consts::PI;

/// A circular gauge showing battery percentage
pub struct BatteryGauge {
    container: Box,
    drawing_area: gtk4::DrawingArea,
    percentage_label: Label,
    percentage: Cell<u32>,
}

impl BatteryGauge {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.set_halign(gtk4::Align::Center);

        // Drawing area for the circular gauge
        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_size_request(180, 180);

        // Percentage label in center
        let percentage_label = Label::new(Some("78%"));
        percentage_label.add_css_class("title-1");

        // Create an overlay to center the label on the drawing area
        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&drawing_area));

        let label_box = Box::new(Orientation::Vertical, 4);
        label_box.set_halign(gtk4::Align::Center);
        label_box.set_valign(gtk4::Align::Center);
        label_box.append(&percentage_label);

        let battery_icon = gtk4::Image::from_icon_name("battery-good-charging-symbolic");
        battery_icon.set_pixel_size(24);
        label_box.append(&battery_icon);

        overlay.add_overlay(&label_box);

        container.append(&overlay);

        let percentage = Cell::new(78);

        // Draw function for the circular gauge
        let percentage_clone = percentage.clone();
        drawing_area.set_draw_func(move |_, cr, width, height| {
            let pct = percentage_clone.get();
            draw_gauge(cr, width, height, pct);
        });

        Self {
            container,
            drawing_area,
            percentage_label,
            percentage,
        }
    }

    /// Get the widget for embedding
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the battery percentage
    pub fn set_percentage(&self, pct: u32) {
        let pct = pct.min(100);
        self.percentage.set(pct);
        self.percentage_label.set_text(&format!("{}%", pct));
        self.drawing_area.queue_draw();
    }

    /// Get the current percentage
    pub fn get_percentage(&self) -> u32 {
        self.percentage.get()
    }
}

impl Default for BatteryGauge {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_gauge(cr: &cairo::Context, width: i32, height: i32, percentage: u32) {
    let w = width as f64;
    let h = height as f64;
    let center_x = w / 2.0;
    let center_y = h / 2.0;
    let radius = (w.min(h) / 2.0) - 10.0;
    let line_width = 12.0;

    // Clear background (transparent)
    cr.set_operator(cairo::Operator::Clear);
    let _ = cr.paint();
    cr.set_operator(cairo::Operator::Over);

    // Draw background arc (track)
    cr.set_line_width(line_width);
    cr.set_line_cap(cairo::LineCap::Round);

    // Dark gray track
    cr.set_source_rgba(0.3, 0.3, 0.3, 0.5);
    cr.arc(
        center_x,
        center_y,
        radius,
        -PI / 2.0,           // Start at top (12 o'clock)
        3.0 * PI / 2.0,      // Go full circle
    );
    let _ = cr.stroke();

    // Draw progress arc
    let progress = (percentage as f64 / 100.0) * 2.0 * PI;
    let end_angle = -PI / 2.0 + progress;

    // Color based on percentage
    let (r, g, b) = get_gauge_color(percentage);
    cr.set_source_rgb(r, g, b);

    cr.arc(
        center_x,
        center_y,
        radius,
        -PI / 2.0,  // Start at top
        end_angle,
    );
    let _ = cr.stroke();

    // Draw tick marks
    cr.set_line_width(2.0);
    cr.set_source_rgba(0.5, 0.5, 0.5, 0.5);

    for i in 0..12 {
        let angle = -PI / 2.0 + (i as f64 / 12.0) * 2.0 * PI;
        let inner_radius = radius - line_width / 2.0 - 5.0;
        let outer_radius = radius - line_width / 2.0 - 10.0;

        let x1 = center_x + inner_radius * angle.cos();
        let y1 = center_y + inner_radius * angle.sin();
        let x2 = center_x + outer_radius * angle.cos();
        let y2 = center_y + outer_radius * angle.sin();

        cr.move_to(x1, y1);
        cr.line_to(x2, y2);
        let _ = cr.stroke();
    }
}

/// Get color based on battery percentage
fn get_gauge_color(percentage: u32) -> (f64, f64, f64) {
    if percentage <= 10 {
        // Red - critical
        (0.90, 0.11, 0.14)
    } else if percentage <= 20 {
        // Orange - low
        (0.99, 0.47, 0.15)
    } else if percentage <= 50 {
        // Yellow - medium
        (0.96, 0.76, 0.07)
    } else {
        // Green - good
        (0.34, 0.89, 0.54)
    }
}

use cairo;
