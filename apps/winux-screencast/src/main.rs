//! Winux Screencast - Screen recording tool for Winux OS
//!
//! Features:
//! - Screen, window, and region recording
//! - XDG Desktop Portal support (Wayland native)
//! - PipeWire integration for screen capture
//! - Multiple codecs (H.264, H.265, VP9, AV1)
//! - Audio capture (system + microphone)
//! - Mouse/keyboard visualization
//! - Global hotkeys
//! - GIF export for short recordings

mod capture;
mod recording;
mod ui;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, gio};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::capture::SourceType;
use crate::recording::{RecordingConfig, RecordingState};
use crate::ui::MainWindow;

const APP_ID: &str = "org.winux.screencast";

/// Video codec options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoCodec {
    #[default]
    H264,
    H265,
    VP9,
    AV1,
}

impl VideoCodec {
    pub fn label(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "H.264 (AVC)",
            VideoCodec::H265 => "H.265 (HEVC)",
            VideoCodec::VP9 => "VP9",
            VideoCodec::AV1 => "AV1",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            VideoCodec::H264 | VideoCodec::H265 => "mp4",
            VideoCodec::VP9 | VideoCodec::AV1 => "webm",
        }
    }

    pub fn gstreamer_encoder(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "x264enc",
            VideoCodec::H265 => "x265enc",
            VideoCodec::VP9 => "vp9enc",
            VideoCodec::AV1 => "av1enc",
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    MP4,
    WebM,
    MKV,
    GIF,
}

impl OutputFormat {
    pub fn label(&self) -> &'static str {
        match self {
            OutputFormat::MP4 => "MP4",
            OutputFormat::WebM => "WebM",
            OutputFormat::MKV => "MKV",
            OutputFormat::GIF => "GIF",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::MP4 => "mp4",
            OutputFormat::WebM => "webm",
            OutputFormat::MKV => "mkv",
            OutputFormat::GIF => "gif",
        }
    }

    pub fn gstreamer_muxer(&self) -> &'static str {
        match self {
            OutputFormat::MP4 => "mp4mux",
            OutputFormat::WebM => "webmmux",
            OutputFormat::MKV => "matroskamux",
            OutputFormat::GIF => "gifenc",
        }
    }
}

/// Resolution preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Resolution {
    #[default]
    Original,
    R1080p,
    R720p,
    R480p,
}

impl Resolution {
    pub fn label(&self) -> &'static str {
        match self {
            Resolution::Original => "Original",
            Resolution::R1080p => "1080p (1920x1080)",
            Resolution::R720p => "720p (1280x720)",
            Resolution::R480p => "480p (854x480)",
        }
    }

    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            Resolution::Original => None,
            Resolution::R1080p => Some((1920, 1080)),
            Resolution::R720p => Some((1280, 720)),
            Resolution::R480p => Some((854, 480)),
        }
    }
}

/// Framerate options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Framerate {
    Fps24,
    #[default]
    Fps30,
    Fps60,
}

impl Framerate {
    pub fn label(&self) -> &'static str {
        match self {
            Framerate::Fps24 => "24 FPS",
            Framerate::Fps30 => "30 FPS",
            Framerate::Fps60 => "60 FPS",
        }
    }

    pub fn value(&self) -> u32 {
        match self {
            Framerate::Fps24 => 24,
            Framerate::Fps30 => 30,
            Framerate::Fps60 => 60,
        }
    }
}

/// Application state shared across components
pub struct AppState {
    /// Current recording source type
    pub source_type: SourceType,
    /// Selected monitor index (for multi-monitor)
    pub selected_monitor: Option<u32>,
    /// Recording configuration
    pub config: RecordingConfig,
    /// Current recording state
    pub recording_state: RecordingState,
    /// Path to current/last recording
    pub recording_path: Option<PathBuf>,
    /// Recording duration in seconds
    pub recording_duration: f64,
    /// Timer delay before recording starts
    pub timer_delay: u32,
    /// Whether to show mouse clicks
    pub show_mouse_clicks: bool,
    /// Whether to show key presses
    pub show_key_presses: bool,
    /// Include system audio
    pub include_system_audio: bool,
    /// Include microphone
    pub include_microphone: bool,
    /// Selected microphone device
    pub microphone_device: Option<String>,
    /// Output directory
    pub output_directory: PathBuf,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let videos_dir = dirs::video_dir()
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()));
        let output_dir = videos_dir.join("Screencasts");

        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir).ok();
        }

        Self {
            source_type: SourceType::Screen,
            selected_monitor: None,
            config: RecordingConfig::default(),
            recording_state: RecordingState::Idle,
            recording_path: None,
            recording_duration: 0.0,
            timer_delay: 0,
            show_mouse_clicks: false,
            show_key_presses: false,
            include_system_audio: true,
            include_microphone: false,
            microphone_device: None,
            output_directory: output_dir,
        }
    }

    /// Generate a unique filename for the recording
    pub fn generate_filename(&self) -> String {
        let now = chrono::Local::now();
        let extension = self.config.format.extension();
        format!("Screencast_{}.{}", now.format("%Y-%m-%d_%H-%M-%S"), extension)
    }

    /// Get the full output path for a new recording
    pub fn get_output_path(&self) -> PathBuf {
        self.output_directory.join(self.generate_filename())
    }
}

fn main() -> gtk::glib::ExitCode {
    // Initialize GStreamer
    if let Err(e) = gstreamer::init() {
        eprintln!("Failed to initialize GStreamer: {}", e);
        return gtk::glib::ExitCode::FAILURE;
    }

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
        let mut immediate_record = false;

        let mut i = 1; // Skip program name
        while i < args.len() {
            match args[i].as_str() {
                "-s" | "--screen" => {
                    state.borrow_mut().source_type = SourceType::Screen;
                    immediate_record = true;
                }
                "-w" | "--window" => {
                    state.borrow_mut().source_type = SourceType::Window;
                    immediate_record = true;
                }
                "-r" | "--region" => {
                    state.borrow_mut().source_type = SourceType::Region;
                    immediate_record = true;
                }
                "-d" | "--delay" => {
                    if i + 1 < args.len() {
                        state.borrow_mut().timer_delay = args[i + 1].parse().unwrap_or(0);
                        i += 1;
                    }
                }
                "-a" | "--audio" => {
                    state.borrow_mut().include_system_audio = true;
                }
                "-m" | "--microphone" => {
                    state.borrow_mut().include_microphone = true;
                }
                "-o" | "--output" => {
                    if i + 1 < args.len() {
                        state.borrow_mut().output_directory = PathBuf::from(&args[i + 1]);
                        i += 1;
                    }
                }
                "-h" | "--help" => {
                    print_help();
                    return 0;
                }
                _ => {}
            }
            i += 1;
        }

        if immediate_record {
            // Start recording immediately
            let app_clone = app.clone();
            let state_clone = state.clone();
            glib::idle_add_local_once(move || {
                recording::start_recording(&app_clone, &state_clone);
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
    println!("Winux Screencast - Screen recording tool for Winux OS");
    println!();
    println!("USAGE:");
    println!("    winux-screencast [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -s, --screen          Record entire screen");
    println!("    -w, --window          Record specific window");
    println!("    -r, --region          Record selected region");
    println!("    -d, --delay <SEC>     Delay recording by SEC seconds");
    println!("    -a, --audio           Include system audio");
    println!("    -m, --microphone      Include microphone audio");
    println!("    -o, --output <DIR>    Set output directory");
    println!("    -h, --help            Print help information");
    println!();
    println!("KEYBOARD SHORTCUTS:");
    println!("    Ctrl+Alt+R            Start/stop recording");
    println!("    Ctrl+Alt+P            Pause/resume recording");
    println!("    Escape                Cancel recording");
}

fn build_ui(app: &Application, state: Rc<RefCell<AppState>>) {
    let window = MainWindow::new(app, state);
    window.present();
}
