//! Video recording functionality

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::camera::{Resolution, FrameRate};
use crate::camera::device::FrameData;
use super::{CaptureResult, CaptureMetadata};

/// Video capture handler
pub struct VideoCapture {
    settings: VideoSettings,
    is_recording: Arc<AtomicBool>,
    start_time: Option<Instant>,
    frame_count: u64,
    output_path: Option<PathBuf>,
}

impl VideoCapture {
    pub fn new(settings: VideoSettings) -> Self {
        Self {
            settings,
            is_recording: Arc::new(AtomicBool::new(false)),
            start_time: None,
            frame_count: 0,
            output_path: None,
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    /// Start video recording
    pub fn start_recording(&mut self, output_path: PathBuf) -> Result<(), VideoError> {
        if self.is_recording() {
            return Err(VideoError::AlreadyRecording);
        }

        log::info!("Starting video recording to {:?}", output_path);

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| VideoError::IoError(e.to_string()))?;
        }

        // In a full implementation, this would:
        // 1. Initialize video encoder (via GStreamer, FFmpeg, or pipewire-media-session)
        // 2. Set up audio capture if enabled
        // 3. Create output file with proper container format

        self.output_path = Some(output_path);
        self.start_time = Some(Instant::now());
        self.frame_count = 0;
        self.is_recording.store(true, Ordering::Relaxed);

        Ok(())
    }

    /// Add a frame to the recording
    pub fn add_frame(&mut self, frame: &FrameData) -> Result<(), VideoError> {
        if !self.is_recording() {
            return Err(VideoError::NotRecording);
        }

        // In a full implementation:
        // 1. Convert frame to appropriate format for encoder
        // 2. Encode frame using video codec
        // 3. Write to output file

        self.frame_count += 1;

        // Log progress periodically
        if self.frame_count % 30 == 0 {
            if let Some(start) = self.start_time {
                let elapsed = start.elapsed();
                log::debug!(
                    "Recording: {} frames, {:.1}s elapsed",
                    self.frame_count,
                    elapsed.as_secs_f32()
                );
            }
        }

        Ok(())
    }

    /// Stop recording and finalize the video file
    pub fn stop_recording(&mut self) -> CaptureResult {
        if !self.is_recording() {
            return CaptureResult::failure("Not recording");
        }

        log::info!("Stopping video recording");

        let duration = self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        // In a full implementation:
        // 1. Flush encoder buffers
        // 2. Write remaining data
        // 3. Close output file
        // 4. Add metadata (if supported by format)

        self.is_recording.store(false, Ordering::Relaxed);

        let path = self.output_path.take().unwrap_or_default();

        CaptureResult::success(
            path,
            CaptureMetadata {
                width: self.settings.resolution.width(),
                height: self.settings.resolution.height(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                camera_name: String::new(),
                exposure: None,
                duration_ms: Some(duration.as_millis() as u64),
            },
        )
    }

    /// Get recording duration
    pub fn duration(&self) -> Duration {
        self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Update settings (only when not recording)
    pub fn update_settings(&mut self, settings: VideoSettings) -> Result<(), VideoError> {
        if self.is_recording() {
            return Err(VideoError::CannotChangeWhileRecording);
        }
        self.settings = settings;
        Ok(())
    }
}

/// Video recording settings
#[derive(Debug, Clone)]
pub struct VideoSettings {
    pub resolution: Resolution,
    pub frame_rate: FrameRate,
    pub format: VideoFormat,
    pub bitrate: VideoBitrate,
    pub audio_enabled: bool,
    pub audio_device: Option<String>,
    pub max_duration: Option<Duration>,
    pub max_file_size: Option<u64>,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            resolution: Resolution::Hd1080p,
            frame_rate: FrameRate::Fps30,
            format: VideoFormat::Mp4,
            bitrate: VideoBitrate::Medium,
            audio_enabled: true,
            audio_device: None,
            max_duration: None,
            max_file_size: None,
        }
    }
}

/// Video container format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoFormat {
    #[default]
    Mp4,
    Webm,
    Mkv,
}

impl VideoFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Webm => "webm",
            VideoFormat::Mkv => "mkv",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "MP4 (H.264)",
            VideoFormat::Webm => "WebM (VP9)",
            VideoFormat::Mkv => "MKV (H.264)",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "video/mp4",
            VideoFormat::Webm => "video/webm",
            VideoFormat::Mkv => "video/x-matroska",
        }
    }

    pub fn all() -> &'static [VideoFormat] {
        &[VideoFormat::Mp4, VideoFormat::Webm, VideoFormat::Mkv]
    }
}

/// Video bitrate presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoBitrate {
    Low,      // 2 Mbps
    #[default]
    Medium,   // 5 Mbps
    High,     // 10 Mbps
    Maximum,  // 20 Mbps
}

impl VideoBitrate {
    pub fn kbps(&self) -> u32 {
        match self {
            VideoBitrate::Low => 2000,
            VideoBitrate::Medium => 5000,
            VideoBitrate::High => 10000,
            VideoBitrate::Maximum => 20000,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            VideoBitrate::Low => "Low (2 Mbps)",
            VideoBitrate::Medium => "Medium (5 Mbps)",
            VideoBitrate::High => "High (10 Mbps)",
            VideoBitrate::Maximum => "Maximum (20 Mbps)",
        }
    }
}

/// Video capture errors
#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error("Already recording")]
    AlreadyRecording,

    #[error("Not recording")]
    NotRecording,

    #[error("Cannot change settings while recording")]
    CannotChangeWhileRecording,

    #[error("Encoder error: {0}")]
    EncoderError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Format not supported: {0}")]
    FormatNotSupported(String),
}
