//! Recording module - manages recording sessions
//!
//! This module handles the lifecycle of screen recordings,
//! including session management, output handling, and state tracking.

pub mod session;
pub mod output;

pub use session::RecordingSession;
pub use output::OutputManager;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::AppState;
use crate::capture::{SourceType, CursorMode};
use crate::{VideoCodec, OutputFormat, Resolution, Framerate};

/// Recording state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecordingState {
    /// No recording in progress
    #[default]
    Idle,
    /// Countdown before recording
    Countdown,
    /// Starting up (requesting permissions, etc.)
    Starting,
    /// Actively recording
    Recording,
    /// Recording is paused
    Paused,
    /// Stopping and finalizing
    Stopping,
    /// Recording completed successfully
    Completed,
    /// Recording failed
    Failed,
}

impl RecordingState {
    pub fn label(&self) -> &'static str {
        match self {
            RecordingState::Idle => "Ready",
            RecordingState::Countdown => "Starting...",
            RecordingState::Starting => "Initializing...",
            RecordingState::Recording => "Recording",
            RecordingState::Paused => "Paused",
            RecordingState::Stopping => "Finishing...",
            RecordingState::Completed => "Completed",
            RecordingState::Failed => "Failed",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, RecordingState::Recording | RecordingState::Paused | RecordingState::Starting)
    }

    pub fn can_start(&self) -> bool {
        matches!(self, RecordingState::Idle | RecordingState::Completed | RecordingState::Failed)
    }

    pub fn can_pause(&self) -> bool {
        matches!(self, RecordingState::Recording)
    }

    pub fn can_stop(&self) -> bool {
        matches!(self, RecordingState::Recording | RecordingState::Paused)
    }
}

/// Recording configuration
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    /// Video codec
    pub codec: VideoCodec,
    /// Output format
    pub format: OutputFormat,
    /// Resolution
    pub resolution: Resolution,
    /// Framerate
    pub framerate: Framerate,
    /// Video bitrate in kbps
    pub video_bitrate: u32,
    /// Audio bitrate in kbps
    pub audio_bitrate: u32,
    /// Include system audio
    pub include_system_audio: bool,
    /// Include microphone
    pub include_microphone: bool,
    /// Cursor mode
    pub cursor_mode: CursorMode,
    /// Show mouse clicks
    pub show_mouse_clicks: bool,
    /// Show key presses
    pub show_key_presses: bool,
    /// Use hardware acceleration
    pub hardware_acceleration: bool,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            format: OutputFormat::MP4,
            resolution: Resolution::Original,
            framerate: Framerate::Fps30,
            video_bitrate: 8000,
            audio_bitrate: 192,
            include_system_audio: true,
            include_microphone: false,
            cursor_mode: CursorMode::Embedded,
            show_mouse_clicks: false,
            show_key_presses: false,
            hardware_acceleration: true,
        }
    }
}

/// Start a new recording
pub fn start_recording(app: &Application, state: &Rc<RefCell<AppState>>) {
    let delay = state.borrow().timer_delay;

    if delay > 0 {
        // Start countdown
        state.borrow_mut().recording_state = RecordingState::Countdown;

        let app_clone = app.clone();
        let state_clone = state.clone();

        // Show countdown notification
        show_countdown(&app_clone, delay);

        glib::timeout_add_seconds_local_once(delay, move || {
            execute_recording(&app_clone, &state_clone);
        });
    } else {
        execute_recording(app, state);
    }
}

fn show_countdown(app: &Application, seconds: u32) {
    // Could show a notification or overlay with countdown
    if let Some(window) = app.active_window() {
        let toast = libadwaita::Toast::new(&format!("Recording starts in {} seconds...", seconds));
        toast.set_timeout(seconds);

        if let Ok(adw_window) = window.clone().downcast::<libadwaita::ApplicationWindow>() {
            if let Some(content) = adw_window.content() {
                if let Ok(toast_overlay) = content.downcast::<libadwaita::ToastOverlay>() {
                    toast_overlay.add_toast(toast);
                }
            }
        }
    }
}

fn execute_recording(app: &Application, state: &Rc<RefCell<AppState>>) {
    state.borrow_mut().recording_state = RecordingState::Starting;

    let app_clone = app.clone();
    let state_clone = state.clone();

    // Spawn async task to start recording
    glib::spawn_future_local(async move {
        match RecordingSession::start(&state_clone).await {
            Ok(session) => {
                state_clone.borrow_mut().recording_state = RecordingState::Recording;
                // Store session somewhere accessible
                // The session will be managed by the UI
            }
            Err(e) => {
                eprintln!("Failed to start recording: {}", e);
                state_clone.borrow_mut().recording_state = RecordingState::Failed;
                show_error_dialog(&app_clone, &format!("Failed to start recording: {}", e));
            }
        }
    });
}

/// Pause the current recording
pub fn pause_recording(state: &Rc<RefCell<AppState>>) {
    if state.borrow().recording_state == RecordingState::Recording {
        state.borrow_mut().recording_state = RecordingState::Paused;
        // Actual pause logic would be handled by the session
    }
}

/// Resume a paused recording
pub fn resume_recording(state: &Rc<RefCell<AppState>>) {
    if state.borrow().recording_state == RecordingState::Paused {
        state.borrow_mut().recording_state = RecordingState::Recording;
        // Actual resume logic would be handled by the session
    }
}

/// Stop the current recording
pub fn stop_recording(state: &Rc<RefCell<AppState>>) {
    let current_state = state.borrow().recording_state;
    if current_state == RecordingState::Recording || current_state == RecordingState::Paused {
        state.borrow_mut().recording_state = RecordingState::Stopping;
        // Actual stop logic would be handled by the session
    }
}

/// Cancel the current recording without saving
pub fn cancel_recording(state: &Rc<RefCell<AppState>>) {
    state.borrow_mut().recording_state = RecordingState::Idle;
    state.borrow_mut().recording_path = None;
    // Cleanup temporary files
}

fn show_error_dialog(app: &Application, message: &str) {
    if let Some(window) = app.active_window() {
        let dialog = gtk::AlertDialog::builder()
            .message("Recording Error")
            .detail(message)
            .modal(true)
            .build();

        dialog.show(Some(&window));
    }
}

/// Format duration for display (HH:MM:SS)
pub fn format_duration(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// Format file size for display
pub fn format_file_size(bytes: u64) -> String {
    use bytesize::ByteSize;
    ByteSize(bytes).to_string()
}
