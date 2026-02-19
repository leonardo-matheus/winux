//! UI module - user interface components

mod viewfinder;
mod controls;
mod gallery;

pub use viewfinder::build_viewfinder;
pub use controls::build_controls;
pub use gallery::build_gallery;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Orientation, Label, ScrolledWindow, Button};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::window::AppState;
use crate::camera::{Resolution, FrameRate, AspectRatio, WhiteBalance};
use crate::capture::{TimerMode, VideoFormat};
use crate::processing::FilterType;

/// Build the settings panel
pub fn build_settings_panel(state: Rc<RefCell<AppState>>) -> GtkBox {
    let settings_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(24)
        .build();

    // Camera settings group
    let camera_group = build_camera_settings_group(state.clone());
    content.append(&camera_group);

    // Photo settings group
    let photo_group = build_photo_settings_group(state.clone());
    content.append(&photo_group);

    // Video settings group
    let video_group = build_video_settings_group(state.clone());
    content.append(&video_group);

    // Filter settings group
    let filter_group = build_filter_settings_group(state.clone());
    content.append(&filter_group);

    scrolled.set_child(Some(&content));
    settings_box.append(&scrolled);

    settings_box
}

fn build_camera_settings_group(state: Rc<RefCell<AppState>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Camera")
        .description("Camera device and capture settings")
        .build();

    // Resolution selection
    let resolution_row = adw::ComboRow::builder()
        .title("Resolution")
        .subtitle("Photo and video resolution")
        .build();

    let resolutions = gtk::StringList::new(&[
        Resolution::Hd720p.label(),
        Resolution::Hd1080p.label(),
        Resolution::Uhd4k.label(),
    ]);
    resolution_row.set_model(Some(&resolutions));
    resolution_row.set_selected(1); // Default to 1080p

    let state_clone = state.clone();
    resolution_row.connect_selected_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.camera.resolution = match row.selected() {
            0 => Resolution::Hd720p,
            1 => Resolution::Hd1080p,
            2 => Resolution::Uhd4k,
            _ => Resolution::Hd1080p,
        };
    });

    group.add(&resolution_row);

    // Aspect ratio selection
    let aspect_row = adw::ComboRow::builder()
        .title("Aspect Ratio")
        .subtitle("Viewfinder aspect ratio")
        .build();

    let aspects = gtk::StringList::new(&[
        AspectRatio::Ratio4x3.label(),
        AspectRatio::Ratio16x9.label(),
        AspectRatio::Ratio1x1.label(),
    ]);
    aspect_row.set_model(Some(&aspects));
    aspect_row.set_selected(1); // Default to 16:9

    let state_clone = state.clone();
    aspect_row.connect_selected_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.camera.aspect_ratio = match row.selected() {
            0 => AspectRatio::Ratio4x3,
            1 => AspectRatio::Ratio16x9,
            2 => AspectRatio::Ratio1x1,
            _ => AspectRatio::Ratio16x9,
        };
    });

    group.add(&aspect_row);

    // Mirror mode toggle
    let mirror_row = adw::SwitchRow::builder()
        .title("Mirror Preview")
        .subtitle("Flip the camera preview horizontally")
        .active(true)
        .build();

    let state_clone = state.clone();
    mirror_row.connect_active_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.mirror_mode = row.is_active();
    });

    group.add(&mirror_row);

    // White balance
    let wb_row = adw::ComboRow::builder()
        .title("White Balance")
        .build();

    let wb_options: Vec<&str> = WhiteBalance::all().iter().map(|wb| wb.label()).collect();
    let wb_list = gtk::StringList::new(&wb_options);
    wb_row.set_model(Some(&wb_list));

    let state_clone = state.clone();
    wb_row.connect_selected_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.camera.white_balance = WhiteBalance::all()[row.selected() as usize];
    });

    group.add(&wb_row);

    // Exposure control
    let exposure_row = adw::ActionRow::builder()
        .title("Exposure")
        .subtitle("Adjust exposure compensation")
        .build();

    let exposure_scale = gtk::Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&gtk::Adjustment::new(0.0, -2.0, 2.0, 0.1, 0.5, 0.0))
        .width_request(200)
        .draw_value(true)
        .value_pos(gtk::PositionType::Left)
        .build();

    let state_clone = state.clone();
    exposure_scale.connect_value_changed(move |scale| {
        let mut state = state_clone.borrow_mut();
        state.camera.exposure = scale.value() as f32;
    });

    exposure_row.add_suffix(&exposure_scale);
    group.add(&exposure_row);

    group
}

fn build_photo_settings_group(state: Rc<RefCell<AppState>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Photo")
        .description("Photo capture settings")
        .build();

    // Timer selection
    let timer_row = adw::ComboRow::builder()
        .title("Timer")
        .subtitle("Countdown before capture")
        .build();

    let timer_options: Vec<&str> = TimerMode::all().iter().map(|t| t.label()).collect();
    let timer_list = gtk::StringList::new(&timer_options);
    timer_row.set_model(Some(&timer_list));

    let state_clone = state.clone();
    timer_row.connect_selected_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.capture_settings.photo.timer_mode = TimerMode::all()[row.selected() as usize];
    });

    group.add(&timer_row);

    // Burst mode toggle
    let burst_row = adw::SwitchRow::builder()
        .title("Burst Mode")
        .subtitle("Capture multiple photos in quick succession")
        .build();

    let state_clone = state.clone();
    burst_row.connect_active_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.capture_settings.photo.burst_settings.enabled = row.is_active();
    });

    group.add(&burst_row);

    // HDR toggle
    let hdr_row = adw::SwitchRow::builder()
        .title("HDR")
        .subtitle("High Dynamic Range (if supported)")
        .build();

    let state_clone = state.clone();
    hdr_row.connect_active_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.capture_settings.photo.hdr_enabled = row.is_active();
    });

    group.add(&hdr_row);

    // Flash toggle
    let flash_row = adw::SwitchRow::builder()
        .title("Flash/LED")
        .subtitle("Enable flash or LED light")
        .build();

    let state_clone = state.clone();
    flash_row.connect_active_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.camera.flash_enabled = row.is_active();
    });

    group.add(&flash_row);

    group
}

fn build_video_settings_group(state: Rc<RefCell<AppState>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Video")
        .description("Video recording settings")
        .build();

    // Frame rate selection
    let fps_row = adw::ComboRow::builder()
        .title("Frame Rate")
        .subtitle("Video recording FPS")
        .build();

    let fps_options = gtk::StringList::new(&[
        FrameRate::Fps24.label(),
        FrameRate::Fps30.label(),
        FrameRate::Fps60.label(),
    ]);
    fps_row.set_model(Some(&fps_options));
    fps_row.set_selected(1); // Default to 30 FPS

    let state_clone = state.clone();
    fps_row.connect_selected_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.camera.frame_rate = match row.selected() {
            0 => FrameRate::Fps24,
            1 => FrameRate::Fps30,
            2 => FrameRate::Fps60,
            _ => FrameRate::Fps30,
        };
    });

    group.add(&fps_row);

    // Video format selection
    let format_row = adw::ComboRow::builder()
        .title("Format")
        .subtitle("Video container format")
        .build();

    let format_options: Vec<&str> = VideoFormat::all().iter().map(|f| f.label()).collect();
    let format_list = gtk::StringList::new(&format_options);
    format_row.set_model(Some(&format_list));

    let state_clone = state.clone();
    format_row.connect_selected_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.capture_settings.video.format = VideoFormat::all()[row.selected() as usize];
    });

    group.add(&format_row);

    // Audio toggle
    let audio_row = adw::SwitchRow::builder()
        .title("Record Audio")
        .subtitle("Include microphone audio in video")
        .active(true)
        .build();

    let state_clone = state.clone();
    audio_row.connect_active_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.capture_settings.video.audio_enabled = row.is_active();
    });

    group.add(&audio_row);

    group
}

fn build_filter_settings_group(state: Rc<RefCell<AppState>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Filters")
        .description("Real-time image filters")
        .build();

    // Filter selection
    let filter_row = adw::ComboRow::builder()
        .title("Filter")
        .subtitle("Apply filter to preview and capture")
        .build();

    let filter_options: Vec<&str> = FilterType::all().iter().map(|f| f.label()).collect();
    let filter_list = gtk::StringList::new(&filter_options);
    filter_row.set_model(Some(&filter_list));

    let state_clone = state.clone();
    filter_row.connect_selected_notify(move |row| {
        let mut state = state_clone.borrow_mut();
        state.current_filter = FilterType::all()[row.selected() as usize];
    });

    group.add(&filter_row);

    group
}
