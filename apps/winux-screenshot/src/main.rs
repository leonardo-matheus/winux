//! Winux Screenshot - Screenshot tool with integrated editor
//!
//! Features:
//! - Fullscreen, window, and region capture
//! - Wayland portal support (xdg-screenshot)
//! - Built-in editor with annotations
//! - Clipboard and file output

mod capture;
mod editor;
mod output;
mod ui;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, gio};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

use crate::capture::CaptureMode;
use crate::editor::EditorState;
use crate::ui::MainWindow;

const APP_ID: &str = "org.winux.screenshot";

/// Application state shared across components
#[derive(Default)]
pub struct AppState {
    /// Current captured image data
    pub captured_image: Option<image::DynamicImage>,
    /// Path to temporary captured file
    pub temp_path: Option<PathBuf>,
    /// Current capture mode
    pub capture_mode: CaptureMode,
    /// Timer delay in seconds (0 = no delay)
    pub timer_delay: u32,
    /// Editor state
    pub editor: EditorState,
    /// Whether we're in capture mode
    pub capturing: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            captured_image: None,
            temp_path: None,
            capture_mode: CaptureMode::Fullscreen,
            timer_delay: 0,
            editor: EditorState::default(),
            capturing: false,
        }
    }
}

fn main() -> gtk::glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_startup(|_| {
        adw::init().expect("Failed to initialize libadwaita");
    });

    // Handle command line arguments
    app.connect_command_line(|app, cmdline| {
        let args: Vec<String> = cmdline.arguments()
            .iter()
            .filter_map(|a| a.to_str().map(String::from))
            .collect();

        let state = Rc::new(RefCell::new(AppState::new()));

        // Parse arguments
        let mut mode = CaptureMode::Fullscreen;
        let mut delay = 0u32;
        let mut immediate_capture = false;

        let mut i = 1; // Skip program name
        while i < args.len() {
            match args[i].as_str() {
                "-f" | "--fullscreen" => {
                    mode = CaptureMode::Fullscreen;
                    immediate_capture = true;
                }
                "-w" | "--window" => {
                    mode = CaptureMode::Window;
                    immediate_capture = true;
                }
                "-r" | "--region" => {
                    mode = CaptureMode::Region;
                    immediate_capture = true;
                }
                "-d" | "--delay" => {
                    if i + 1 < args.len() {
                        delay = args[i + 1].parse().unwrap_or(0);
                        i += 1;
                    }
                }
                "-c" | "--clipboard" => {
                    state.borrow_mut().editor.copy_to_clipboard = true;
                }
                "-h" | "--help" => {
                    print_help();
                    return 0;
                }
                _ => {}
            }
            i += 1;
        }

        state.borrow_mut().capture_mode = mode;
        state.borrow_mut().timer_delay = delay;

        if immediate_capture {
            // Launch capture directly
            let app_clone = app.clone();
            let state_clone = state.clone();
            glib::idle_add_local_once(move || {
                capture::start_capture(&app_clone, &state_clone);
            });
        }

        // Build UI
        build_ui(app, state);

        0
    });

    app.connect_activate(|app| {
        let state = Rc::new(RefCell::new(AppState::new()));
        build_ui(app, state);
    });

    app.run()
}

fn print_help() {
    println!("Winux Screenshot - Screenshot tool for Winux OS");
    println!();
    println!("USAGE:");
    println!("    winux-screenshot [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -f, --fullscreen    Capture entire screen");
    println!("    -w, --window        Capture active window");
    println!("    -r, --region        Capture selected region");
    println!("    -d, --delay <SEC>   Delay capture by SEC seconds");
    println!("    -c, --clipboard     Copy result to clipboard");
    println!("    -h, --help          Print help information");
    println!();
    println!("KEYBOARD SHORTCUTS:");
    println!("    Print Screen        Capture fullscreen");
    println!("    Alt+Print           Capture active window");
    println!("    Shift+Print         Capture region");
}

fn build_ui(app: &Application, state: Rc<RefCell<AppState>>) {
    let window = MainWindow::new(app, state);
    window.present();
}
