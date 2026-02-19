//! Capture module - handles photo and video capture

mod photo;
mod video;

pub use photo::{PhotoCapture, PhotoSettings, TimerMode, BurstSettings};
pub use video::{VideoCapture, VideoSettings, VideoFormat};

use std::path::PathBuf;
use chrono::Local;

/// Capture mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureMode {
    #[default]
    Photo,
    Video,
}

impl CaptureMode {
    pub fn label(&self) -> &'static str {
        match self {
            CaptureMode::Photo => "Photo",
            CaptureMode::Video => "Video",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CaptureMode::Photo => "camera-photo-symbolic",
            CaptureMode::Video => "camera-video-symbolic",
        }
    }
}

/// Unified capture settings
#[derive(Debug, Clone)]
pub struct CaptureSettings {
    pub photo: PhotoSettings,
    pub video: VideoSettings,
    pub output_directory: PathBuf,
    pub file_prefix: String,
}

impl Default for CaptureSettings {
    fn default() -> Self {
        let output_dir = dirs::picture_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Pictures"))
            .join("Winux Camera");

        Self {
            photo: PhotoSettings::default(),
            video: VideoSettings::default(),
            output_directory: output_dir,
            file_prefix: "IMG".to_string(),
        }
    }
}

impl CaptureSettings {
    /// Generate a unique filename for photo
    pub fn generate_photo_filename(&self) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.jpg", self.file_prefix, timestamp);
        self.output_directory.join(filename)
    }

    /// Generate a unique filename for video
    pub fn generate_video_filename(&self) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let extension = match self.video.format {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Webm => "webm",
            VideoFormat::Mkv => "mkv",
        };
        let filename = format!("VID_{}_{}.{}", self.file_prefix, timestamp, extension);
        self.output_directory.join(filename)
    }

    /// Ensure output directory exists
    pub fn ensure_output_directory(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.output_directory)
    }
}

/// Capture result
#[derive(Debug, Clone)]
pub struct CaptureResult {
    pub file_path: PathBuf,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: CaptureMetadata,
}

impl CaptureResult {
    pub fn success(path: PathBuf, metadata: CaptureMetadata) -> Self {
        Self {
            file_path: path,
            success: true,
            error_message: None,
            metadata,
        }
    }

    pub fn failure(error: &str) -> Self {
        Self {
            file_path: PathBuf::new(),
            success: false,
            error_message: Some(error.to_string()),
            metadata: CaptureMetadata::default(),
        }
    }
}

/// Metadata for captured media
#[derive(Debug, Clone, Default)]
pub struct CaptureMetadata {
    pub width: u32,
    pub height: u32,
    pub timestamp: u64,
    pub camera_name: String,
    pub exposure: Option<f32>,
    pub duration_ms: Option<u64>, // For video
}
