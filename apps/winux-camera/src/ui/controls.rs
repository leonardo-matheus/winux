//! Camera control buttons and mode switching

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Orientation, Button, Label, ToggleButton};
use libadwaita as adw;
use adw::prelude::*;
use glib::clone;

use std::cell::RefCell;
use std::rc::Rc;

use crate::window::AppState;
use crate::capture::{CaptureMode, TimerMode};
use crate::processing::FilterType;

/// Build the camera controls panel
pub fn build_controls(state: Rc<RefCell<AppState>>) -> GtkBox {
    let controls_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_start(24)
        .margin_end(24)
        .margin_top(12)
        .margin_bottom(24)
        .build();

    // Quick settings row
    let quick_settings = build_quick_settings(state.clone());
    controls_box.append(&quick_settings);

    // Mode switcher
    let mode_box = build_mode_switcher(state.clone());
    controls_box.append(&mode_box);

    // Main capture button area
    let capture_box = build_capture_area(state.clone());
    controls_box.append(&capture_box);

    // Filter strip
    let filter_strip = build_filter_strip(state.clone());
    controls_box.append(&filter_strip);

    controls_box
}

/// Build quick settings row (timer, flash, etc.)
fn build_quick_settings(state: Rc<RefCell<AppState>>) -> GtkBox {
    let settings_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Center)
        .build();

    // Timer button
    let timer_button = Button::builder()
        .icon_name("alarm-symbolic")
        .tooltip_text("Timer")
        .css_classes(["circular", "flat"])
        .build();

    let timer_label = Label::builder()
        .label("")
        .css_classes(["caption"])
        .build();

    let state_clone = state.clone();
    let timer_label_clone = timer_label.clone();
    timer_button.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        // Cycle through timer modes
        state.capture_settings.photo.timer_mode = match state.capture_settings.photo.timer_mode {
            TimerMode::Off => TimerMode::Seconds3,
            TimerMode::Seconds3 => TimerMode::Seconds10,
            TimerMode::Seconds10 => TimerMode::Off,
        };
        let label = match state.capture_settings.photo.timer_mode {
            TimerMode::Off => "",
            TimerMode::Seconds3 => "3s",
            TimerMode::Seconds10 => "10s",
        };
        timer_label_clone.set_label(label);
    });

    let timer_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();
    timer_box.append(&timer_button);
    timer_box.append(&timer_label);

    // Flash button
    let flash_button = Button::builder()
        .icon_name("flash-off-symbolic")
        .tooltip_text("Flash/LED")
        .css_classes(["circular", "flat"])
        .build();

    let state_clone = state.clone();
    let flash_btn = flash_button.clone();
    flash_button.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.camera.flash_enabled = !state.camera.flash_enabled;
        flash_btn.set_icon_name(if state.camera.flash_enabled {
            "flash-on-symbolic"
        } else {
            "flash-off-symbolic"
        });
    });

    // HDR button
    let hdr_button = Button::builder()
        .label("HDR")
        .tooltip_text("High Dynamic Range")
        .css_classes(["flat"])
        .build();

    let state_clone = state.clone();
    let hdr_btn = hdr_button.clone();
    hdr_button.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.capture_settings.photo.hdr_enabled = !state.capture_settings.photo.hdr_enabled;
        if state.capture_settings.photo.hdr_enabled {
            hdr_btn.add_css_class("suggested-action");
        } else {
            hdr_btn.remove_css_class("suggested-action");
        }
    });

    // Burst button
    let burst_button = Button::builder()
        .icon_name("view-grid-symbolic")
        .tooltip_text("Burst Mode")
        .css_classes(["circular", "flat"])
        .build();

    let state_clone = state.clone();
    let burst_btn = burst_button.clone();
    burst_button.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.capture_settings.photo.burst_settings.enabled = !state.capture_settings.photo.burst_settings.enabled;
        if state.capture_settings.photo.burst_settings.enabled {
            burst_btn.add_css_class("suggested-action");
        } else {
            burst_btn.remove_css_class("suggested-action");
        }
    });

    settings_box.append(&timer_box);
    settings_box.append(&flash_button);
    settings_box.append(&hdr_button);
    settings_box.append(&burst_button);

    settings_box
}

/// Build mode switcher (Photo/Video)
fn build_mode_switcher(state: Rc<RefCell<AppState>>) -> GtkBox {
    let mode_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .halign(gtk::Align::Center)
        .css_classes(["linked"])
        .build();

    let photo_button = ToggleButton::builder()
        .label("Photo")
        .active(true)
        .css_classes(["toggle"])
        .build();

    let video_button = ToggleButton::builder()
        .label("Video")
        .group(&photo_button)
        .css_classes(["toggle"])
        .build();

    let state_clone = state.clone();
    photo_button.connect_toggled(move |btn| {
        if btn.is_active() {
            let mut state = state_clone.borrow_mut();
            state.capture_mode = CaptureMode::Photo;
        }
    });

    let state_clone = state.clone();
    video_button.connect_toggled(move |btn| {
        if btn.is_active() {
            let mut state = state_clone.borrow_mut();
            state.capture_mode = CaptureMode::Video;
        }
    });

    mode_box.append(&photo_button);
    mode_box.append(&video_button);

    mode_box
}

/// Build the main capture button area
fn build_capture_area(state: Rc<RefCell<AppState>>) -> GtkBox {
    let capture_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(24)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    // Left side - Gallery thumbnail
    let gallery_button = Button::builder()
        .icon_name("folder-pictures-symbolic")
        .css_classes(["circular"])
        .width_request(50)
        .height_request(50)
        .tooltip_text("Recent Photos")
        .build();

    gallery_button.connect_clicked(|_| {
        // Open gallery or most recent photo
        log::info!("Opening gallery...");
    });

    // Center - Main capture button
    let capture_button = Button::builder()
        .css_classes(["circular", "capture-button"])
        .width_request(80)
        .height_request(80)
        .tooltip_text("Capture")
        .build();

    // Inner circle for capture button
    let inner = gtk::DrawingArea::builder()
        .content_width(64)
        .content_height(64)
        .build();

    let state_for_draw = state.clone();
    inner.set_draw_func(move |_area, cr, width, height| {
        let state = state_for_draw.borrow();
        let is_recording = state.is_recording;
        let mode = state.capture_mode;

        let cx = width as f64 / 2.0;
        let cy = height as f64 / 2.0;
        let radius = (width.min(height) as f64 / 2.0) - 4.0;

        if mode == CaptureMode::Video && is_recording {
            // Recording state - show red square
            cr.set_source_rgb(0.9, 0.2, 0.2);
            let size = radius * 0.7;
            cr.rectangle(cx - size / 2.0, cy - size / 2.0, size, size);
            let _ = cr.fill();
        } else {
            // Normal state - show white circle
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.arc(cx, cy, radius, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();

            // Video mode - add red center
            if mode == CaptureMode::Video {
                cr.set_source_rgb(0.9, 0.2, 0.2);
                cr.arc(cx, cy, radius * 0.4, 0.0, 2.0 * std::f64::consts::PI);
                let _ = cr.fill();
            }
        }
    });

    capture_button.set_child(Some(&inner));

    let state_clone = state.clone();
    let inner_clone = inner.clone();
    capture_button.connect_clicked(move |_btn| {
        let mut state = state_clone.borrow_mut();

        match state.capture_mode {
            CaptureMode::Photo => {
                // Handle timer
                let timer = state.capture_settings.photo.timer_mode;
                if timer != TimerMode::Off {
                    log::info!("Starting {}s timer...", timer.seconds());
                    // TODO: Implement countdown
                }
                log::info!("Capturing photo...");
                // TODO: Trigger actual capture
            }
            CaptureMode::Video => {
                state.is_recording = !state.is_recording;
                if state.is_recording {
                    log::info!("Starting video recording...");
                } else {
                    log::info!("Stopping video recording...");
                }
            }
        }

        inner_clone.queue_draw();
    });

    // Right side - Switch camera
    let switch_button = Button::builder()
        .icon_name("camera-switch-symbolic")
        .css_classes(["circular"])
        .width_request(50)
        .height_request(50)
        .tooltip_text("Switch Camera")
        .build();

    let state_clone = state.clone();
    switch_button.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.camera.switch_camera();
        log::info!("Switched camera");
    });

    capture_box.append(&gallery_button);
    capture_box.append(&capture_button);
    capture_box.append(&switch_button);

    capture_box
}

/// Build filter strip for quick filter selection
fn build_filter_strip(state: Rc<RefCell<AppState>>) -> gtk::ScrolledWindow {
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .height_request(80)
        .build();

    let filter_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Create filter buttons
    let mut buttons: Vec<ToggleButton> = Vec::new();
    let first_button: Option<ToggleButton> = None;

    for filter in FilterType::all() {
        let button = if let Some(ref group) = first_button {
            ToggleButton::builder()
                .group(group)
                .css_classes(["flat", "filter-button"])
                .build()
        } else {
            ToggleButton::builder()
                .css_classes(["flat", "filter-button"])
                .active(*filter == FilterType::None)
                .build()
        };

        // Filter preview box
        let preview_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        // Preview thumbnail
        let preview = gtk::DrawingArea::builder()
            .content_width(50)
            .content_height(50)
            .build();

        let filter_type = *filter;
        preview.set_draw_func(move |_area, cr, width, height| {
            // Draw filter preview gradient
            let gradient = gtk::cairo::LinearGradient::new(0.0, 0.0, width as f64, height as f64);

            let (r1, g1, b1, r2, g2, b2) = match filter_type {
                FilterType::None => (0.4, 0.5, 0.6, 0.6, 0.5, 0.4),
                FilterType::Grayscale => (0.4, 0.4, 0.4, 0.5, 0.5, 0.5),
                FilterType::Sepia => (0.5, 0.4, 0.3, 0.6, 0.45, 0.35),
                FilterType::Vintage => (0.45, 0.4, 0.35, 0.55, 0.45, 0.4),
                FilterType::Cool => (0.3, 0.4, 0.55, 0.4, 0.5, 0.65),
                FilterType::Warm => (0.55, 0.4, 0.3, 0.65, 0.5, 0.4),
                FilterType::HighContrast => (0.2, 0.3, 0.45, 0.6, 0.5, 0.35),
                FilterType::LowContrast => (0.35, 0.4, 0.45, 0.5, 0.48, 0.45),
                FilterType::Negative => (0.7, 0.6, 0.5, 0.5, 0.6, 0.7),
                FilterType::Posterize => (0.4, 0.4, 0.5, 0.5, 0.4, 0.4),
            };

            gradient.add_color_stop_rgb(0.0, r1, g1, b1);
            gradient.add_color_stop_rgb(1.0, r2, g2, b2);

            cr.set_source(&gradient).unwrap();
            cr.rectangle(0.0, 0.0, width as f64, height as f64);
            let _ = cr.fill();
        });

        // Filter name
        let label = Label::builder()
            .label(filter.label())
            .css_classes(["caption"])
            .build();

        preview_box.append(&preview);
        preview_box.append(&label);
        button.set_child(Some(&preview_box));

        let state_clone = state.clone();
        let filter_type = *filter;
        button.connect_toggled(move |btn| {
            if btn.is_active() {
                let mut state = state_clone.borrow_mut();
                state.current_filter = filter_type;
                log::info!("Selected filter: {:?}", filter_type);
            }
        });

        buttons.push(button.clone());
        filter_box.append(&button);
    }

    // Group all buttons together
    if let Some(first) = buttons.first() {
        for button in buttons.iter().skip(1) {
            button.set_group(Some(first));
        }
    }

    scrolled.set_child(Some(&filter_box));
    scrolled
}
