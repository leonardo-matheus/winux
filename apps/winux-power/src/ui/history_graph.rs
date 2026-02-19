// History graph widget for Winux Power
// Displays battery level over time

use cairo;
use gtk4::prelude::*;
use gtk4::Box;
use std::cell::RefCell;
use std::rc::Rc;

/// Battery history data point
#[derive(Debug, Clone)]
pub struct HistoryPoint {
    pub time: f64,       // 0.0 to 1.0 (normalized time)
    pub percentage: f64, // 0.0 to 100.0
    pub charging: bool,
}

/// Graph showing battery history over time
pub struct HistoryGraph {
    container: Box,
    drawing_area: gtk4::DrawingArea,
    data: Rc<RefCell<Vec<HistoryPoint>>>,
}

impl HistoryGraph {
    pub fn new() -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);

        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_size_request(-1, 200);
        drawing_area.set_hexpand(true);

        container.append(&drawing_area);

        // Generate sample data
        let data = Rc::new(RefCell::new(Self::generate_sample_data()));

        // Set up draw function
        let data_clone = data.clone();
        drawing_area.set_draw_func(move |_, cr, width, height| {
            let points = data_clone.borrow();
            draw_history_graph(cr, width, height, &points);
        });

        Self {
            container,
            drawing_area,
            data,
        }
    }

    /// Get the widget for embedding
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set new data points
    pub fn set_data(&self, points: Vec<HistoryPoint>) {
        *self.data.borrow_mut() = points;
        self.drawing_area.queue_draw();
    }

    /// Generate sample data for demonstration
    fn generate_sample_data() -> Vec<HistoryPoint> {
        let mut points = Vec::new();
        let mut percentage = 40.0;
        let mut charging = false;

        for i in 0..100 {
            let time = i as f64 / 100.0;

            // Simulate charging/discharging cycles
            if i == 20 {
                charging = true;
            } else if i == 50 {
                charging = false;
            } else if i == 75 {
                charging = true;
            }

            if charging {
                percentage = (percentage + 1.5).min(100.0);
            } else {
                percentage = (percentage - 0.8).max(0.0);
            }

            points.push(HistoryPoint {
                time,
                percentage,
                charging,
            });
        }

        points
    }

    /// Refresh the display
    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
    }
}

impl Default for HistoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_history_graph(cr: &cairo::Context, width: i32, height: i32, data: &[HistoryPoint]) {
    let w = width as f64;
    let h = height as f64;

    let margin_left = 40.0;
    let margin_right = 20.0;
    let margin_top = 20.0;
    let margin_bottom = 30.0;

    let graph_w = w - margin_left - margin_right;
    let graph_h = h - margin_top - margin_bottom;

    // Clear background
    cr.set_source_rgb(0.12, 0.12, 0.12);
    let _ = cr.paint();

    // Draw grid
    cr.set_source_rgba(0.3, 0.3, 0.3, 0.5);
    cr.set_line_width(1.0);

    // Horizontal grid lines (percentage levels)
    for i in 0..=4 {
        let y = margin_top + (i as f64 / 4.0) * graph_h;
        cr.move_to(margin_left, y);
        cr.line_to(w - margin_right, y);
        let _ = cr.stroke();

        // Labels
        let label = format!("{}%", 100 - i * 25);
        cr.set_source_rgba(0.6, 0.6, 0.6, 1.0);
        cr.move_to(5.0, y + 4.0);
        let _ = cr.show_text(&label);
    }

    // Vertical grid lines (time)
    cr.set_source_rgba(0.3, 0.3, 0.3, 0.5);
    for i in 0..=6 {
        let x = margin_left + (i as f64 / 6.0) * graph_w;
        cr.move_to(x, margin_top);
        cr.line_to(x, h - margin_bottom);
        let _ = cr.stroke();

        // Time labels
        let hours_ago = 24 - i * 4;
        let label = if hours_ago == 0 {
            "Agora".to_string()
        } else {
            format!("-{}h", hours_ago)
        };
        cr.set_source_rgba(0.6, 0.6, 0.6, 1.0);
        cr.move_to(x - 15.0, h - 10.0);
        let _ = cr.show_text(&label);
    }

    if data.is_empty() {
        return;
    }

    // Draw fill area
    cr.move_to(margin_left, margin_top + graph_h);

    for point in data {
        let x = margin_left + point.time * graph_w;
        let y = margin_top + (1.0 - point.percentage / 100.0) * graph_h;
        cr.line_to(x, y);
    }

    let last_x = margin_left + data.last().map(|p| p.time).unwrap_or(1.0) * graph_w;
    cr.line_to(last_x, margin_top + graph_h);
    cr.close_path();

    // Gradient fill
    let gradient = cairo::LinearGradient::new(0.0, margin_top, 0.0, margin_top + graph_h);
    gradient.add_color_stop_rgba(0.0, 0.34, 0.89, 0.54, 0.4);  // Green at top
    gradient.add_color_stop_rgba(1.0, 0.34, 0.89, 0.54, 0.05); // Fade at bottom
    cr.set_source(&gradient).unwrap();
    let _ = cr.fill_preserve();

    // Draw the line on top
    cr.new_path();
    let mut first = true;

    for (i, point) in data.iter().enumerate() {
        let x = margin_left + point.time * graph_w;
        let y = margin_top + (1.0 - point.percentage / 100.0) * graph_h;

        if first {
            cr.move_to(x, y);
            first = false;
        } else {
            cr.line_to(x, y);
        }

        // Color segments based on charging state
        if i > 0 && data[i - 1].charging != point.charging {
            // Stroke current segment
            if data[i - 1].charging {
                cr.set_source_rgb(0.34, 0.89, 0.54); // Green for charging
            } else {
                cr.set_source_rgb(0.96, 0.38, 0.32); // Red for discharging
            }
            cr.set_line_width(2.5);
            let _ = cr.stroke();

            // Start new path
            cr.move_to(x, y);
        }
    }

    // Stroke the last segment
    if let Some(last) = data.last() {
        if last.charging {
            cr.set_source_rgb(0.34, 0.89, 0.54);
        } else {
            cr.set_source_rgb(0.96, 0.38, 0.32);
        }
    }
    cr.set_line_width(2.5);
    let _ = cr.stroke();

    // Draw current value point
    if let Some(last) = data.last() {
        let x = margin_left + last.time * graph_w;
        let y = margin_top + (1.0 - last.percentage / 100.0) * graph_h;

        // Outer glow
        cr.set_source_rgba(0.34, 0.89, 0.54, 0.3);
        cr.arc(x, y, 8.0, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.fill();

        // Inner point
        cr.set_source_rgb(0.34, 0.89, 0.54);
        cr.arc(x, y, 4.0, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.fill();
    }

    // Draw border around graph area
    cr.set_source_rgba(0.4, 0.4, 0.4, 0.5);
    cr.set_line_width(1.0);
    cr.rectangle(margin_left, margin_top, graph_w, graph_h);
    let _ = cr.stroke();
}
