// Analog Clock Widget - Cairo-based analog clock display

use gtk4::prelude::*;
use gtk4::DrawingArea;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

#[derive(Clone)]
pub struct AnalogClock {
    drawing_area: DrawingArea,
    size: i32,
}

impl AnalogClock {
    pub fn new(size: i32) -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(size, size);
        drawing_area.add_css_class("analog-clock");

        let size_cell = Rc::new(RefCell::new(size as f64));

        drawing_area.set_draw_func({
            let size_cell = size_cell.clone();
            move |_, cr, width, height| {
                let size = *size_cell.borrow();
                let center_x = width as f64 / 2.0;
                let center_y = height as f64 / 2.0;
                let radius = size.min(width as f64).min(height as f64) / 2.0 - 10.0;

                // Get current time
                let now = chrono::Local::now();
                let hours = now.format("%I").to_string().parse::<f64>().unwrap_or(0.0);
                let minutes = now.format("%M").to_string().parse::<f64>().unwrap_or(0.0);
                let seconds = now.format("%S").to_string().parse::<f64>().unwrap_or(0.0);

                // Draw clock face
                cr.set_source_rgba(0.2, 0.2, 0.2, 0.1);
                cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
                let _ = cr.fill();

                // Draw clock border
                cr.set_source_rgba(0.5, 0.5, 0.5, 0.3);
                cr.set_line_width(2.0);
                cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
                let _ = cr.stroke();

                // Draw hour markers
                cr.set_source_rgba(0.6, 0.6, 0.6, 0.8);
                for i in 0..12 {
                    let angle = (i as f64) * PI / 6.0 - PI / 2.0;
                    let inner_radius = radius - 15.0;
                    let outer_radius = radius - 5.0;

                    let x1 = center_x + inner_radius * angle.cos();
                    let y1 = center_y + inner_radius * angle.sin();
                    let x2 = center_x + outer_radius * angle.cos();
                    let y2 = center_y + outer_radius * angle.sin();

                    cr.set_line_width(if i % 3 == 0 { 3.0 } else { 1.5 });
                    cr.move_to(x1, y1);
                    cr.line_to(x2, y2);
                    let _ = cr.stroke();
                }

                // Calculate hand angles
                let second_angle = (seconds / 60.0) * 2.0 * PI - PI / 2.0;
                let minute_angle = ((minutes + seconds / 60.0) / 60.0) * 2.0 * PI - PI / 2.0;
                let hour_angle = ((hours + minutes / 60.0) / 12.0) * 2.0 * PI - PI / 2.0;

                // Draw hour hand
                cr.set_source_rgba(0.8, 0.8, 0.8, 1.0);
                cr.set_line_width(4.0);
                cr.set_line_cap(gtk4::cairo::LineCap::Round);
                cr.move_to(center_x, center_y);
                cr.line_to(
                    center_x + (radius * 0.5) * hour_angle.cos(),
                    center_y + (radius * 0.5) * hour_angle.sin(),
                );
                let _ = cr.stroke();

                // Draw minute hand
                cr.set_source_rgba(0.8, 0.8, 0.8, 1.0);
                cr.set_line_width(3.0);
                cr.move_to(center_x, center_y);
                cr.line_to(
                    center_x + (radius * 0.7) * minute_angle.cos(),
                    center_y + (radius * 0.7) * minute_angle.sin(),
                );
                let _ = cr.stroke();

                // Draw second hand
                cr.set_source_rgba(0.9, 0.3, 0.3, 1.0);
                cr.set_line_width(1.5);
                cr.move_to(center_x, center_y);
                cr.line_to(
                    center_x + (radius * 0.8) * second_angle.cos(),
                    center_y + (radius * 0.8) * second_angle.sin(),
                );
                let _ = cr.stroke();

                // Draw center dot
                cr.set_source_rgba(0.9, 0.3, 0.3, 1.0);
                cr.arc(center_x, center_y, 5.0, 0.0, 2.0 * PI);
                let _ = cr.fill();

                // Inner center
                cr.set_source_rgba(0.2, 0.2, 0.2, 1.0);
                cr.arc(center_x, center_y, 3.0, 0.0, 2.0 * PI);
                let _ = cr.fill();
            }
        });

        Self { drawing_area, size }
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn update(&self) {
        self.drawing_area.queue_draw();
    }

    pub fn set_size(&self, size: i32) {
        self.drawing_area.set_size_request(size, size);
    }
}

/// Mini analog clock for world clock cards
#[derive(Clone)]
pub struct MiniAnalogClock {
    drawing_area: DrawingArea,
    timezone: Rc<RefCell<String>>,
}

impl MiniAnalogClock {
    pub fn new(size: i32, timezone: &str) -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(size, size);

        let timezone = Rc::new(RefCell::new(timezone.to_string()));
        let tz_clone = timezone.clone();

        drawing_area.set_draw_func(move |_, cr, width, height| {
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            let radius = (width.min(height) as f64 / 2.0) - 2.0;

            // Get time in specified timezone
            let tz_str = tz_clone.borrow();
            let tz: chrono_tz::Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);
            let now = chrono::Utc::now().with_timezone(&tz);

            let hours = now.format("%I").to_string().parse::<f64>().unwrap_or(0.0);
            let minutes = now.format("%M").to_string().parse::<f64>().unwrap_or(0.0);

            // Draw simple clock face
            cr.set_source_rgba(0.5, 0.5, 0.5, 0.2);
            cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
            let _ = cr.fill();

            // Calculate hand angles
            let minute_angle = (minutes / 60.0) * 2.0 * PI - PI / 2.0;
            let hour_angle = ((hours + minutes / 60.0) / 12.0) * 2.0 * PI - PI / 2.0;

            // Draw hour hand
            cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
            cr.set_line_width(2.0);
            cr.set_line_cap(gtk4::cairo::LineCap::Round);
            cr.move_to(center_x, center_y);
            cr.line_to(
                center_x + (radius * 0.5) * hour_angle.cos(),
                center_y + (radius * 0.5) * hour_angle.sin(),
            );
            let _ = cr.stroke();

            // Draw minute hand
            cr.set_line_width(1.5);
            cr.move_to(center_x, center_y);
            cr.line_to(
                center_x + (radius * 0.7) * minute_angle.cos(),
                center_y + (radius * 0.7) * minute_angle.sin(),
            );
            let _ = cr.stroke();

            // Draw center dot
            cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
            cr.arc(center_x, center_y, 2.0, 0.0, 2.0 * PI);
            let _ = cr.fill();
        });

        Self {
            drawing_area,
            timezone,
        }
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn update(&self) {
        self.drawing_area.queue_draw();
    }

    pub fn set_timezone(&self, timezone: &str) {
        *self.timezone.borrow_mut() = timezone.to_string();
        self.update();
    }
}
