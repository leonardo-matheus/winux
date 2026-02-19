//! Camera viewfinder/preview widget

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Orientation, DrawingArea, Frame, Label, Overlay};
use gdk4 as gdk;

use std::cell::RefCell;
use std::rc::Rc;

use crate::window::AppState;
use crate::processing::FilterType;

/// Build the camera viewfinder widget
pub fn build_viewfinder(state: Rc<RefCell<AppState>>) -> GtkBox {
    let viewfinder_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .vexpand(true)
        .hexpand(true)
        .build();

    // Create overlay for viewfinder and overlays
    let overlay = Overlay::new();

    // Main drawing area for camera preview
    let drawing_area = DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .content_width(640)
        .content_height(480)
        .build();

    // Store state for drawing
    let state_clone = state.clone();

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let state = state_clone.borrow();

        // Draw background
        cr.set_source_rgb(0.1, 0.1, 0.12);
        let _ = cr.paint();

        // Calculate viewport based on aspect ratio
        let aspect = state.camera.aspect_ratio.aspect_value();
        let view_width: f64;
        let view_height: f64;
        let view_x: f64;
        let view_y: f64;

        if (width as f64 / height as f64) > aspect {
            // Width limited
            view_height = height as f64;
            view_width = view_height * aspect;
            view_x = (width as f64 - view_width) / 2.0;
            view_y = 0.0;
        } else {
            // Height limited
            view_width = width as f64;
            view_height = view_width / aspect;
            view_x = 0.0;
            view_y = (height as f64 - view_height) / 2.0;
        }

        // Draw viewfinder background (simulated camera view)
        cr.set_source_rgb(0.15, 0.15, 0.18);
        cr.rectangle(view_x, view_y, view_width, view_height);
        let _ = cr.fill();

        // Draw camera preview placeholder with animated pattern
        draw_camera_preview(cr, view_x, view_y, view_width, view_height, state.mirror_mode, state.current_filter);

        // Draw viewfinder grid (rule of thirds)
        draw_grid(cr, view_x, view_y, view_width, view_height);

        // Draw recording indicator if recording
        if state.is_recording {
            draw_recording_indicator(cr, view_x + 20.0, view_y + 20.0);
        }

        // Draw filter indicator
        if state.current_filter != FilterType::None {
            draw_filter_indicator(cr, view_x + view_width - 100.0, view_y + 20.0, state.current_filter);
        }
    });

    // Focus indicator overlay
    let focus_indicator = Label::builder()
        .label("")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    focus_indicator.add_css_class("focus-indicator");

    // Timer countdown overlay
    let timer_label = Label::builder()
        .label("")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .visible(false)
        .build();
    timer_label.add_css_class("timer-countdown");

    // Add frame around the drawing area
    let frame = Frame::builder()
        .child(&drawing_area)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(6)
        .build();
    frame.add_css_class("viewfinder-frame");

    overlay.set_child(Some(&frame));
    overlay.add_overlay(&focus_indicator);
    overlay.add_overlay(&timer_label);

    // Status bar below viewfinder
    let status_bar = build_status_bar(state.clone());

    viewfinder_box.append(&overlay);
    viewfinder_box.append(&status_bar);

    // Set up periodic redraw for animation
    let drawing_area_clone = drawing_area.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
        drawing_area_clone.queue_draw();
        glib::ControlFlow::Continue
    });

    viewfinder_box
}

/// Draw simulated camera preview
fn draw_camera_preview(cr: &gtk::cairo::Context, x: f64, y: f64, width: f64, height: f64, mirror: bool, filter: FilterType) {
    // Apply mirror transformation if enabled
    if mirror {
        cr.translate(x + width, y);
        cr.scale(-1.0, 1.0);
        cr.translate(-x, -y);
    }

    // Draw a dynamic gradient to simulate camera feed
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64 / 1000.0)
        .unwrap_or(0.0);

    // Base gradient (simulated scene)
    let gradient = gtk::cairo::LinearGradient::new(x, y, x + width, y + height);

    // Apply filter color tint
    let (r1, g1, b1, r2, g2, b2) = match filter {
        FilterType::None => (0.2, 0.3, 0.4, 0.4, 0.3, 0.2),
        FilterType::Grayscale => (0.3, 0.3, 0.3, 0.3, 0.3, 0.3),
        FilterType::Sepia => (0.4, 0.3, 0.2, 0.35, 0.25, 0.15),
        FilterType::Vintage => (0.35, 0.3, 0.25, 0.3, 0.25, 0.2),
        FilterType::Cool => (0.2, 0.3, 0.45, 0.35, 0.35, 0.5),
        FilterType::Warm => (0.45, 0.35, 0.25, 0.5, 0.3, 0.2),
        FilterType::HighContrast => (0.1, 0.2, 0.35, 0.5, 0.4, 0.3),
        FilterType::LowContrast => (0.25, 0.28, 0.32, 0.35, 0.33, 0.3),
        FilterType::Negative => (0.8, 0.7, 0.6, 0.6, 0.7, 0.8),
        FilterType::Posterize => (0.3, 0.3, 0.4, 0.4, 0.3, 0.3),
    };

    // Animate colors slightly
    let phase = (time * 0.5).sin() * 0.05;
    gradient.add_color_stop_rgb(0.0, r1 + phase, g1 + phase, b1);
    gradient.add_color_stop_rgb(1.0, r2 - phase, g2 - phase, b2);

    cr.set_source(&gradient).unwrap();
    cr.rectangle(x, y, width, height);
    let _ = cr.fill();

    // Draw some animated shapes to simulate scene
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.1);

    // Moving circle (simulated object)
    let cx = x + width / 2.0 + (time * 0.3).sin() * 50.0;
    let cy = y + height / 2.0 + (time * 0.4).cos() * 30.0;
    cr.arc(cx, cy, 60.0, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();

    // Face detection box placeholder
    cr.set_source_rgba(0.2, 0.8, 0.4, 0.5);
    cr.set_line_width(2.0);
    let face_x = x + width / 2.0 - 50.0;
    let face_y = y + height / 3.0 - 60.0;
    cr.rectangle(face_x, face_y, 100.0, 120.0);
    let _ = cr.stroke();

    // Corner markers for face box
    let corner_len = 15.0;
    let corners = [
        (face_x, face_y),
        (face_x + 100.0, face_y),
        (face_x, face_y + 120.0),
        (face_x + 100.0, face_y + 120.0),
    ];

    for (i, (corner_x, corner_y)) in corners.iter().enumerate() {
        let h_dir = if i % 2 == 0 { 1.0 } else { -1.0 };
        let v_dir = if i < 2 { 1.0 } else { -1.0 };

        cr.move_to(*corner_x, *corner_y);
        cr.line_to(*corner_x + corner_len * h_dir, *corner_y);
        cr.move_to(*corner_x, *corner_y);
        cr.line_to(*corner_x, *corner_y + corner_len * v_dir);
        let _ = cr.stroke();
    }

    // Reset transformation
    if mirror {
        cr.identity_matrix();
    }
}

/// Draw rule of thirds grid
fn draw_grid(cr: &gtk::cairo::Context, x: f64, y: f64, width: f64, height: f64) {
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.15);
    cr.set_line_width(1.0);

    // Vertical lines
    let third_w = width / 3.0;
    for i in 1..=2 {
        cr.move_to(x + third_w * i as f64, y);
        cr.line_to(x + third_w * i as f64, y + height);
        let _ = cr.stroke();
    }

    // Horizontal lines
    let third_h = height / 3.0;
    for i in 1..=2 {
        cr.move_to(x, y + third_h * i as f64);
        cr.line_to(x + width, y + third_h * i as f64);
        let _ = cr.stroke();
    }
}

/// Draw recording indicator
fn draw_recording_indicator(cr: &gtk::cairo::Context, x: f64, y: f64) {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64 / 500.0)
        .unwrap_or(0.0);

    // Pulsing red dot
    let alpha = 0.5 + 0.5 * (time * 2.0).sin();
    cr.set_source_rgba(1.0, 0.2, 0.2, alpha);
    cr.arc(x + 10.0, y + 10.0, 8.0, 0.0, 2.0 * std::f64::consts::PI);
    let _ = cr.fill();

    // REC text
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.select_font_face("Sans", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Bold);
    cr.set_font_size(14.0);
    cr.move_to(x + 25.0, y + 15.0);
    let _ = cr.show_text("REC");
}

/// Draw filter indicator
fn draw_filter_indicator(cr: &gtk::cairo::Context, x: f64, y: f64, filter: FilterType) {
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
    cr.rectangle(x - 5.0, y - 5.0, 90.0, 24.0);
    let _ = cr.fill();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.8);
    cr.select_font_face("Sans", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal);
    cr.set_font_size(11.0);
    cr.move_to(x, y + 10.0);
    let _ = cr.show_text(filter.label());
}

/// Build status bar below viewfinder
fn build_status_bar(state: Rc<RefCell<AppState>>) -> GtkBox {
    let status_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(6)
        .build();

    // Resolution indicator
    let resolution_label = Label::builder()
        .label("1080p")
        .css_classes(["dim-label", "caption"])
        .build();

    // FPS indicator
    let fps_label = Label::builder()
        .label("30 FPS")
        .css_classes(["dim-label", "caption"])
        .build();

    // Storage indicator
    let storage_label = Label::builder()
        .label("")
        .css_classes(["dim-label", "caption"])
        .hexpand(true)
        .halign(gtk::Align::End)
        .build();

    // Update labels based on state
    let state_clone = state.clone();
    let res_label = resolution_label.clone();
    let fps = fps_label.clone();

    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let state = state_clone.borrow();
        res_label.set_label(state.camera.resolution.label());
        fps.set_label(state.camera.frame_rate.label());
        glib::ControlFlow::Continue
    });

    status_box.append(&resolution_label);
    status_box.append(&fps_label);
    status_box.append(&storage_label);

    status_box
}
