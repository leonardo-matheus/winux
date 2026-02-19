//! Capture module - handles different screenshot capture modes

mod fullscreen;
mod window;
mod region;
mod wayland;

pub use fullscreen::capture_fullscreen;
pub use window::capture_window;
pub use region::capture_region;
pub use wayland::WaylandCapture;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

use crate::AppState;
use crate::ui::preview::PreviewWindow;

/// Capture mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureMode {
    #[default]
    Fullscreen,
    Window,
    Region,
}

impl CaptureMode {
    pub fn label(&self) -> &'static str {
        match self {
            CaptureMode::Fullscreen => "Fullscreen",
            CaptureMode::Window => "Window",
            CaptureMode::Region => "Region",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CaptureMode::Fullscreen => "video-display-symbolic",
            CaptureMode::Window => "window-symbolic",
            CaptureMode::Region => "edit-select-all-symbolic",
        }
    }
}

/// Captured screenshot data
#[derive(Debug, Clone)]
pub struct CaptureResult {
    pub image: image::DynamicImage,
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// Start the capture process based on current mode
pub fn start_capture(app: &Application, state: &Rc<RefCell<AppState>>) {
    let mode = state.borrow().capture_mode;
    let delay = state.borrow().timer_delay;

    state.borrow_mut().capturing = true;

    let app_clone = app.clone();
    let state_clone = state.clone();

    // Apply delay if set
    if delay > 0 {
        glib::timeout_add_seconds_local_once(delay, move || {
            execute_capture(&app_clone, &state_clone, mode);
        });
    } else {
        execute_capture(app, state, mode);
    }
}

fn execute_capture(app: &Application, state: &Rc<RefCell<AppState>>, mode: CaptureMode) {
    let result = match mode {
        CaptureMode::Fullscreen => capture_fullscreen(),
        CaptureMode::Window => capture_window(),
        CaptureMode::Region => {
            // For region capture, we need to show the overlay first
            let state_clone = state.clone();
            let app_clone = app.clone();

            crate::ui::overlay::show_region_overlay(app, state, move |result| {
                if let Some(capture) = result {
                    handle_capture_result(&app_clone, &state_clone, capture);
                } else {
                    state_clone.borrow_mut().capturing = false;
                }
            });
            return;
        }
    };

    match result {
        Ok(capture) => {
            handle_capture_result(app, state, capture);
        }
        Err(e) => {
            eprintln!("Capture failed: {}", e);
            state.borrow_mut().capturing = false;
            show_error_dialog(app, &format!("Capture failed: {}", e));
        }
    }
}

fn handle_capture_result(app: &Application, state: &Rc<RefCell<AppState>>, capture: CaptureResult) {
    {
        let mut state = state.borrow_mut();
        state.captured_image = Some(capture.image.clone());
        state.temp_path = Some(capture.path.clone());
        state.capturing = false;
    }

    // Show preview window with editor
    PreviewWindow::show(app, state, capture);
}

fn show_error_dialog(app: &Application, message: &str) {
    if let Some(window) = app.active_window() {
        let dialog = gtk::AlertDialog::builder()
            .message("Screenshot Error")
            .detail(message)
            .modal(true)
            .build();

        dialog.show(Some(&window));
    }
}

/// Generate a unique filename for the screenshot
pub fn generate_filename() -> String {
    let now = chrono::Local::now();
    format!("Screenshot_{}.png", now.format("%Y-%m-%d_%H-%M-%S"))
}

/// Get the default screenshots directory
pub fn get_screenshots_dir() -> PathBuf {
    let pictures_dir = dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()));

    let screenshots_dir = pictures_dir.join("Screenshots");

    if !screenshots_dir.exists() {
        std::fs::create_dir_all(&screenshots_dir).ok();
    }

    screenshots_dir
}
