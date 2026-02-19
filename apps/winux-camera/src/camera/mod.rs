//! Camera module - handles camera device access and streaming

mod device;
mod pipewire;
mod v4l2;

pub use device::{CameraDevice, CameraInfo, CameraCapability};
pub use pipewire::PipeWireCamera;
pub use v4l2::V4L2Camera;

use std::path::PathBuf;

/// Camera resolution presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Hd720p,    // 1280x720
    Hd1080p,   // 1920x1080
    Uhd4k,     // 3840x2160
    Custom(u32, u32),
}

impl Resolution {
    pub fn width(&self) -> u32 {
        match self {
            Resolution::Hd720p => 1280,
            Resolution::Hd1080p => 1920,
            Resolution::Uhd4k => 3840,
            Resolution::Custom(w, _) => *w,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Resolution::Hd720p => 720,
            Resolution::Hd1080p => 1080,
            Resolution::Uhd4k => 2160,
            Resolution::Custom(_, h) => *h,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Resolution::Hd720p => "720p HD",
            Resolution::Hd1080p => "1080p Full HD",
            Resolution::Uhd4k => "4K UHD",
            Resolution::Custom(_, _) => "Custom",
        }
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution::Hd1080p
    }
}

/// Aspect ratio options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectRatio {
    Ratio4x3,
    Ratio16x9,
    Ratio1x1,
}

impl AspectRatio {
    pub fn label(&self) -> &'static str {
        match self {
            AspectRatio::Ratio4x3 => "4:3",
            AspectRatio::Ratio16x9 => "16:9",
            AspectRatio::Ratio1x1 => "1:1",
        }
    }

    pub fn aspect_value(&self) -> f64 {
        match self {
            AspectRatio::Ratio4x3 => 4.0 / 3.0,
            AspectRatio::Ratio16x9 => 16.0 / 9.0,
            AspectRatio::Ratio1x1 => 1.0,
        }
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        AspectRatio::Ratio16x9
    }
}

/// Frame rate options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRate {
    Fps24,
    Fps30,
    Fps60,
}

impl FrameRate {
    pub fn value(&self) -> u32 {
        match self {
            FrameRate::Fps24 => 24,
            FrameRate::Fps30 => 30,
            FrameRate::Fps60 => 60,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FrameRate::Fps24 => "24 FPS",
            FrameRate::Fps30 => "30 FPS",
            FrameRate::Fps60 => "60 FPS",
        }
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        FrameRate::Fps30
    }
}

/// Camera state management
#[derive(Debug, Clone)]
pub struct CameraState {
    pub available_cameras: Vec<CameraInfo>,
    pub selected_camera: Option<usize>,
    pub resolution: Resolution,
    pub aspect_ratio: AspectRatio,
    pub frame_rate: FrameRate,
    pub is_streaming: bool,
    pub flash_enabled: bool,
    pub exposure: f32,       // -2.0 to 2.0
    pub white_balance: WhiteBalance,
    pub focus_mode: FocusMode,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            available_cameras: Vec::new(),
            selected_camera: None,
            resolution: Resolution::default(),
            aspect_ratio: AspectRatio::default(),
            frame_rate: FrameRate::default(),
            is_streaming: false,
            flash_enabled: false,
            exposure: 0.0,
            white_balance: WhiteBalance::Auto,
            focus_mode: FocusMode::Auto,
        }
    }
}

impl CameraState {
    /// Detect available cameras on the system
    pub fn detect_cameras(&mut self) {
        // Try PipeWire first, then V4L2 as fallback
        let mut cameras = PipeWireCamera::enumerate_devices();

        if cameras.is_empty() {
            cameras = V4L2Camera::enumerate_devices();
        }

        self.available_cameras = cameras;

        if !self.available_cameras.is_empty() && self.selected_camera.is_none() {
            self.selected_camera = Some(0);
        }

        log::info!("Detected {} camera(s)", self.available_cameras.len());
    }

    /// Switch to next available camera
    pub fn switch_camera(&mut self) {
        if self.available_cameras.len() > 1 {
            if let Some(current) = self.selected_camera {
                self.selected_camera = Some((current + 1) % self.available_cameras.len());
            }
        }
    }

    /// Get current camera info
    pub fn current_camera(&self) -> Option<&CameraInfo> {
        self.selected_camera.and_then(|i| self.available_cameras.get(i))
    }

    /// Get output directory for photos/videos
    pub fn output_directory(&self) -> PathBuf {
        let pictures_dir = dirs::picture_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Pictures"));
        pictures_dir.join("Winux Camera")
    }
}

/// White balance modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhiteBalance {
    #[default]
    Auto,
    Daylight,
    Cloudy,
    Tungsten,
    Fluorescent,
}

impl WhiteBalance {
    pub fn label(&self) -> &'static str {
        match self {
            WhiteBalance::Auto => "Auto",
            WhiteBalance::Daylight => "Daylight",
            WhiteBalance::Cloudy => "Cloudy",
            WhiteBalance::Tungsten => "Tungsten",
            WhiteBalance::Fluorescent => "Fluorescent",
        }
    }

    pub fn all() -> &'static [WhiteBalance] {
        &[
            WhiteBalance::Auto,
            WhiteBalance::Daylight,
            WhiteBalance::Cloudy,
            WhiteBalance::Tungsten,
            WhiteBalance::Fluorescent,
        ]
    }
}

/// Focus modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusMode {
    #[default]
    Auto,
    Continuous,
    Manual,
}

impl FocusMode {
    pub fn label(&self) -> &'static str {
        match self {
            FocusMode::Auto => "Auto Focus",
            FocusMode::Continuous => "Continuous",
            FocusMode::Manual => "Manual",
        }
    }
}
