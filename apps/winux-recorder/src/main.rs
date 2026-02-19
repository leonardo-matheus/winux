//! Winux Recorder - Voice recording application for Winux OS
//!
//! Features:
//! - Record audio from microphone via PipeWire/ALSA
//! - Real-time waveform visualization
//! - Multiple output formats (WAV, MP3, OGG/Opus, FLAC)
//! - Playback with speed control
//! - Recording management (rename, delete, export)

mod audio;
mod recording;
mod ui;
mod window;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, gio};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::audio::AudioFormat;
use crate::recording::{Recording, RecordingState};

const APP_ID: &str = "org.winux.recorder";

/// Application configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// Default output format
    pub default_format: AudioFormat,
    /// Quality preset (high, medium, low)
    pub quality: QualityPreset,
    /// Sample rate
    pub sample_rate: u32,
    /// Mono or stereo
    pub channels: u16,
    /// Output directory
    pub output_dir: std::path::PathBuf,
    /// Selected input device name
    pub input_device: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let output_dir = dirs::audio_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Music")))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Recordings");

        Self {
            default_format: AudioFormat::Wav,
            quality: QualityPreset::High,
            sample_rate: 48000,
            channels: 1,
            output_dir,
            input_device: None,
        }
    }
}

/// Quality preset for encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum QualityPreset {
    Low,
    #[default]
    Medium,
    High,
}

impl QualityPreset {
    pub fn label(&self) -> &'static str {
        match self {
            QualityPreset::Low => "Low",
            QualityPreset::Medium => "Medium",
            QualityPreset::High => "High",
        }
    }

    pub fn bitrate(&self) -> u32 {
        match self {
            QualityPreset::Low => 64_000,
            QualityPreset::Medium => 128_000,
            QualityPreset::High => 320_000,
        }
    }
}

/// Global application state shared across components
pub struct AppState {
    /// Application configuration
    pub config: AppConfig,
    /// Current recording state
    pub recording_state: RecordingState,
    /// List of recordings
    pub recordings: Vec<Recording>,
    /// Currently playing recording index
    pub playing_index: Option<usize>,
    /// Playback speed (0.5 to 2.0)
    pub playback_speed: f64,
    /// Current waveform samples for visualization
    pub waveform_samples: Vec<f32>,
    /// Recording duration in seconds
    pub duration: f64,
    /// Markers/bookmarks during recording
    pub markers: Vec<f64>,
    /// Peak level for level meter
    pub peak_level: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            recording_state: RecordingState::Idle,
            recordings: Vec::new(),
            playing_index: None,
            playback_speed: 1.0,
            waveform_samples: Vec::new(),
            duration: 0.0,
            markers: Vec::new(),
            peak_level: 0.0,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from disk
    pub fn load_config(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("winux-recorder").join("config.json");
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.config = config;
                }
            }
        }
    }

    /// Save configuration to disk
    pub fn save_config(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("winux-recorder");
            let _ = std::fs::create_dir_all(&config_dir);
            let config_path = config_dir.join("config.json");
            if let Ok(content) = serde_json::to_string_pretty(&self.config) {
                let _ = std::fs::write(&config_path, content);
            }
        }
    }
}

fn main() -> gtk::glib::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("winux_recorder=debug".parse().unwrap()),
        )
        .init();

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_startup(|_| {
        adw::init().expect("Failed to initialize libadwaita");
    });

    app.connect_activate(|app| {
        let state = Arc::new(RwLock::new(AppState::new()));
        state.write().load_config();

        // Ensure output directory exists
        let output_dir = state.read().config.output_dir.clone();
        let _ = std::fs::create_dir_all(&output_dir);

        // Load existing recordings
        recording::storage::load_recordings(&state);

        let window = window::RecorderWindow::new(app, state);
        window.present();
    });

    app.run()
}
