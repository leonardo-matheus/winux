//! Recording session management

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

use crate::AppState;
use crate::audio::AudioSamples;

/// Current state of recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecordingState {
    /// Not recording
    #[default]
    Idle,
    /// Recording in progress
    Recording,
    /// Recording paused
    Paused,
    /// Playing back a recording
    Playing,
    /// Playback paused
    PlaybackPaused,
}

impl RecordingState {
    pub fn is_recording(&self) -> bool {
        matches!(self, RecordingState::Recording | RecordingState::Paused)
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, RecordingState::Playing | RecordingState::PlaybackPaused)
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, RecordingState::Idle)
    }
}

/// Marker/bookmark during recording
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Marker {
    /// Timestamp in seconds from start
    pub timestamp: f64,
    /// Optional label
    pub label: Option<String>,
}

/// Active recording session
pub struct RecordingSession {
    /// Application state reference
    state: Arc<RwLock<AppState>>,
    /// Session start time
    start_time: Option<Instant>,
    /// Total paused duration
    paused_duration: Duration,
    /// Pause start time (when paused)
    pause_start: Option<Instant>,
    /// Current samples being recorded
    samples: Vec<f32>,
    /// Markers added during recording
    markers: Vec<Marker>,
    /// Peak level for visualization
    peak_level: f32,
    /// Sample rate
    sample_rate: u32,
    /// Number of channels
    channels: u16,
}

impl RecordingSession {
    /// Create a new recording session
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        let config = state.read().config.clone();
        Self {
            state,
            start_time: None,
            paused_duration: Duration::ZERO,
            pause_start: None,
            samples: Vec::new(),
            markers: Vec::new(),
            peak_level: 0.0,
            sample_rate: config.sample_rate,
            channels: config.channels,
        }
    }

    /// Start the recording session
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.paused_duration = Duration::ZERO;
        self.pause_start = None;
        self.samples.clear();
        self.markers.clear();
        self.peak_level = 0.0;

        self.state.write().recording_state = RecordingState::Recording;
    }

    /// Pause the recording
    pub fn pause(&mut self) {
        if self.state.read().recording_state == RecordingState::Recording {
            self.pause_start = Some(Instant::now());
            self.state.write().recording_state = RecordingState::Paused;
        }
    }

    /// Resume the recording
    pub fn resume(&mut self) {
        if self.state.read().recording_state == RecordingState::Paused {
            if let Some(pause_start) = self.pause_start.take() {
                self.paused_duration += pause_start.elapsed();
            }
            self.state.write().recording_state = RecordingState::Recording;
        }
    }

    /// Stop and finalize the recording
    pub fn stop(&mut self) -> AudioSamples {
        self.state.write().recording_state = RecordingState::Idle;

        AudioSamples {
            data: std::mem::take(&mut self.samples),
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }

    /// Cancel the recording without saving
    pub fn cancel(&mut self) {
        self.state.write().recording_state = RecordingState::Idle;
        self.samples.clear();
        self.markers.clear();
    }

    /// Add a marker at current position
    pub fn add_marker(&mut self, label: Option<String>) {
        let timestamp = self.duration().as_secs_f64();
        self.markers.push(Marker { timestamp, label });
        self.state.write().markers.push(timestamp);
    }

    /// Get current duration
    pub fn duration(&self) -> Duration {
        match self.start_time {
            Some(start) => {
                let total = start.elapsed();
                let pause_time = match self.pause_start {
                    Some(pause_start) => self.paused_duration + pause_start.elapsed(),
                    None => self.paused_duration,
                };
                total.saturating_sub(pause_time)
            }
            None => Duration::ZERO,
        }
    }

    /// Process incoming audio samples
    pub fn process_samples(&mut self, samples: &[f32]) {
        if self.state.read().recording_state != RecordingState::Recording {
            return;
        }

        // Calculate peak level
        let peak: f32 = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        self.peak_level = self.peak_level * 0.9 + peak * 0.1; // Smooth decay

        // Append samples
        self.samples.extend_from_slice(samples);

        // Update state
        let mut state = self.state.write();
        state.peak_level = self.peak_level;
        state.duration = self.duration().as_secs_f64();

        // Update waveform (downsample for display)
        let display_samples = if self.samples.len() > 400 {
            self.samples
                .chunks(self.samples.len() / 200)
                .map(|chunk| chunk.iter().map(|s| s.abs()).fold(0.0f32, f32::max))
                .collect()
        } else {
            self.samples.iter().map(|s| s.abs()).collect()
        };
        state.waveform_samples = display_samples;
    }

    /// Get current peak level
    pub fn peak_level(&self) -> f32 {
        self.peak_level
    }

    /// Get markers
    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    /// Check if recording is active
    pub fn is_active(&self) -> bool {
        self.state.read().recording_state.is_recording()
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.state.read().recording_state == RecordingState::Paused
    }
}

/// Format duration as HH:MM:SS
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

/// Format duration with milliseconds as HH:MM:SS.mmm
pub fn format_duration_precise(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
    } else {
        format!("{:02}:{:02}.{:03}", mins, secs, millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00");
        assert_eq!(format_duration(Duration::from_secs(65)), "01:05");
        assert_eq!(format_duration(Duration::from_secs(3665)), "01:01:05");
    }

    #[test]
    fn test_recording_state() {
        assert!(RecordingState::Recording.is_recording());
        assert!(RecordingState::Paused.is_recording());
        assert!(!RecordingState::Idle.is_recording());
        assert!(!RecordingState::Playing.is_recording());
    }
}
