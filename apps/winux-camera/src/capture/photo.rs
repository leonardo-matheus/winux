//! Photo capture functionality

use std::path::PathBuf;
use image::{ImageBuffer, Rgba, ImageFormat};

use crate::camera::device::FrameData;
use crate::processing::FilterType;
use super::{CaptureResult, CaptureMetadata, CaptureSettings};

/// Photo capture handler
pub struct PhotoCapture {
    settings: PhotoSettings,
}

impl PhotoCapture {
    pub fn new(settings: PhotoSettings) -> Self {
        Self { settings }
    }

    /// Capture a single photo from frame data
    pub fn capture(
        &self,
        frame: &FrameData,
        output_path: &PathBuf,
        filter: FilterType,
    ) -> CaptureResult {
        log::info!("Capturing photo to {:?}", output_path);

        // Convert frame to RGBA
        let rgba_data = frame.to_rgba();

        // Apply filter if needed
        let processed_data = if filter != FilterType::None {
            crate::processing::apply_filter(&rgba_data, frame.width, frame.height, filter)
        } else {
            rgba_data
        };

        // Create image buffer
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            frame.width,
            frame.height,
            processed_data,
        ).expect("Failed to create image buffer");

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return CaptureResult::failure(&format!("Failed to create directory: {}", e));
            }
        }

        // Apply HDR processing if enabled
        let final_img = if self.settings.hdr_enabled {
            self.apply_hdr_tone_mapping(&img)
        } else {
            img
        };

        // Save the image
        match final_img.save_with_format(output_path, ImageFormat::Jpeg) {
            Ok(_) => {
                log::info!("Photo saved successfully");
                CaptureResult::success(
                    output_path.clone(),
                    CaptureMetadata {
                        width: frame.width,
                        height: frame.height,
                        timestamp: frame.timestamp,
                        camera_name: String::new(),
                        exposure: None,
                        duration_ms: None,
                    },
                )
            }
            Err(e) => {
                log::error!("Failed to save photo: {}", e);
                CaptureResult::failure(&format!("Failed to save photo: {}", e))
            }
        }
    }

    /// Capture burst photos
    pub fn capture_burst(
        &self,
        frames: &[FrameData],
        settings: &CaptureSettings,
        filter: FilterType,
    ) -> Vec<CaptureResult> {
        log::info!("Capturing burst of {} photos", frames.len());

        let mut results = Vec::new();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

        for (i, frame) in frames.iter().enumerate() {
            let filename = format!(
                "{}_{}_burst{:03}.jpg",
                settings.file_prefix, timestamp, i + 1
            );
            let output_path = settings.output_directory.join(filename);

            let result = self.capture(frame, &output_path, filter);
            results.push(result);
        }

        results
    }

    /// Simple HDR tone mapping (simulated)
    fn apply_hdr_tone_mapping(
        &self,
        img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        // Simple tone mapping simulation
        // In a real implementation, this would use multiple exposures
        let mut result = img.clone();

        for pixel in result.pixels_mut() {
            // Apply simple local tone mapping
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;

            // Reinhard tone mapping
            let mapped_r = r / (1.0 + r);
            let mapped_g = g / (1.0 + g);
            let mapped_b = b / (1.0 + b);

            // Increase local contrast
            let factor = 1.2;
            let final_r = (mapped_r * factor).clamp(0.0, 1.0);
            let final_g = (mapped_g * factor).clamp(0.0, 1.0);
            let final_b = (mapped_b * factor).clamp(0.0, 1.0);

            pixel[0] = (final_r * 255.0) as u8;
            pixel[1] = (final_g * 255.0) as u8;
            pixel[2] = (final_b * 255.0) as u8;
        }

        result
    }
}

/// Photo capture settings
#[derive(Debug, Clone)]
pub struct PhotoSettings {
    pub timer_mode: TimerMode,
    pub burst_settings: BurstSettings,
    pub hdr_enabled: bool,
    pub flash_mode: FlashMode,
    pub quality: PhotoQuality,
}

impl Default for PhotoSettings {
    fn default() -> Self {
        Self {
            timer_mode: TimerMode::Off,
            burst_settings: BurstSettings::default(),
            hdr_enabled: false,
            flash_mode: FlashMode::Auto,
            quality: PhotoQuality::High,
        }
    }
}

/// Timer mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerMode {
    #[default]
    Off,
    Seconds3,
    Seconds10,
}

impl TimerMode {
    pub fn seconds(&self) -> u32 {
        match self {
            TimerMode::Off => 0,
            TimerMode::Seconds3 => 3,
            TimerMode::Seconds10 => 10,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimerMode::Off => "Off",
            TimerMode::Seconds3 => "3s",
            TimerMode::Seconds10 => "10s",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TimerMode::Off => "alarm-symbolic",
            TimerMode::Seconds3 => "timer-symbolic",
            TimerMode::Seconds10 => "timer-symbolic",
        }
    }

    pub fn all() -> &'static [TimerMode] {
        &[TimerMode::Off, TimerMode::Seconds3, TimerMode::Seconds10]
    }
}

/// Burst capture settings
#[derive(Debug, Clone)]
pub struct BurstSettings {
    pub enabled: bool,
    pub count: u32,
    pub interval_ms: u32,
}

impl Default for BurstSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            count: 5,
            interval_ms: 200,
        }
    }
}

/// Flash mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlashMode {
    Off,
    On,
    #[default]
    Auto,
}

impl FlashMode {
    pub fn label(&self) -> &'static str {
        match self {
            FlashMode::Off => "Off",
            FlashMode::On => "On",
            FlashMode::Auto => "Auto",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FlashMode::Off => "flash-off-symbolic",
            FlashMode::On => "flash-on-symbolic",
            FlashMode::Auto => "flash-auto-symbolic",
        }
    }
}

/// Photo quality options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhotoQuality {
    Low,    // 50%
    Medium, // 75%
    #[default]
    High,   // 90%
    Maximum, // 100%
}

impl PhotoQuality {
    pub fn jpeg_quality(&self) -> u8 {
        match self {
            PhotoQuality::Low => 50,
            PhotoQuality::Medium => 75,
            PhotoQuality::High => 90,
            PhotoQuality::Maximum => 100,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PhotoQuality::Low => "Low",
            PhotoQuality::Medium => "Medium",
            PhotoQuality::High => "High",
            PhotoQuality::Maximum => "Maximum",
        }
    }
}
