//! Main window for Winux Camera

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Box as GtkBox, Orientation, Stack, StackTransitionType};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::camera::CameraState;
use crate::capture::{CaptureMode, CaptureSettings};
use crate::processing::FilterType;
use crate::ui::{build_viewfinder, build_controls, build_gallery, build_settings_panel};

/// Application state shared across UI components
pub struct AppState {
    pub camera: CameraState,
    pub capture_mode: CaptureMode,
    pub capture_settings: CaptureSettings,
    pub current_filter: FilterType,
    pub is_recording: bool,
    pub timer_seconds: u32,
    pub mirror_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            camera: CameraState::default(),
            capture_mode: CaptureMode::Photo,
            capture_settings: CaptureSettings::default(),
            current_filter: FilterType::None,
            is_recording: false,
            timer_seconds: 0,
            mirror_mode: true, // Mirror by default for selfies
        }
    }
}

pub fn build_window(app: &Application) {
    // Apply dark theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

    // Shared application state
    let state = Rc::new(RefCell::new(AppState::default()));

    // Main content stack
    let main_stack = Stack::builder()
        .transition_type(StackTransitionType::Crossfade)
        .transition_duration(200)
        .build();

    // Camera view
    let camera_view = build_camera_view(state.clone());
    main_stack.add_named(&camera_view, Some("camera"));

    // Gallery view
    let gallery_view = build_gallery(state.clone());
    main_stack.add_named(&gallery_view, Some("gallery"));

    // Settings view
    let settings_view = build_settings_panel(state.clone());
    main_stack.add_named(&settings_view, Some("settings"));

    main_stack.set_visible_child_name("camera");

    // Header bar
    let header_bar = build_header_bar(main_stack.clone(), state.clone());

    // Main layout
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.append(&header_bar);
    main_box.append(&main_stack);

    // Window
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Winux Camera")
        .default_width(800)
        .default_height(600)
        .content(&main_box)
        .build();

    // Add CSS styling
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string(include_str!("../style.css"));

    if let Some(display) = gdk4::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    window.present();
}

fn build_camera_view(state: Rc<RefCell<AppState>>) -> GtkBox {
    let camera_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .vexpand(true)
        .hexpand(true)
        .build();

    // Viewfinder (camera preview)
    let viewfinder = build_viewfinder(state.clone());
    camera_box.append(&viewfinder);

    // Controls (capture button, mode switch, etc.)
    let controls = build_controls(state.clone());
    camera_box.append(&controls);

    camera_box
}

fn build_header_bar(stack: Stack, state: Rc<RefCell<AppState>>) -> adw::HeaderBar {
    let header = adw::HeaderBar::new();

    // Gallery button (left side)
    let gallery_button = gtk::Button::builder()
        .icon_name("folder-pictures-symbolic")
        .tooltip_text("Gallery")
        .build();

    let stack_clone = stack.clone();
    gallery_button.connect_clicked(move |_| {
        let current = stack_clone.visible_child_name();
        if current.as_deref() == Some("gallery") {
            stack_clone.set_visible_child_name("camera");
        } else {
            stack_clone.set_visible_child_name("gallery");
        }
    });

    // Camera switch button (for multiple cameras)
    let switch_camera_button = gtk::Button::builder()
        .icon_name("camera-switch-symbolic")
        .tooltip_text("Switch Camera")
        .build();

    let state_clone = state.clone();
    switch_camera_button.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.camera.switch_camera();
    });

    // Settings button (right side)
    let settings_button = gtk::Button::builder()
        .icon_name("emblem-system-symbolic")
        .tooltip_text("Settings")
        .build();

    let stack_clone = stack.clone();
    settings_button.connect_clicked(move |_| {
        let current = stack_clone.visible_child_name();
        if current.as_deref() == Some("settings") {
            stack_clone.set_visible_child_name("camera");
        } else {
            stack_clone.set_visible_child_name("settings");
        }
    });

    // Title
    let title = gtk::Label::builder()
        .label("Camera")
        .css_classes(["title"])
        .build();

    header.set_title_widget(Some(&title));
    header.pack_start(&gallery_button);
    header.pack_start(&switch_camera_button);
    header.pack_end(&settings_button);

    header
}
