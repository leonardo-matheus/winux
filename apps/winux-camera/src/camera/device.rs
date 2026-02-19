//! Camera device abstraction

use std::path::PathBuf;
use super::{Resolution, FrameRate, WhiteBalance, FocusMode};

/// Information about a camera device
#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub id: String,
    pub name: String,
    pub device_path: PathBuf,
    pub capabilities: Vec<CameraCapability>,
    pub is_front_facing: bool,
}

impl CameraInfo {
    pub fn new(id: &str, name: &str, device_path: PathBuf) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            device_path,
            capabilities: Vec::new(),
            is_front_facing: name.to_lowercase().contains("front"),
        }
    }

    /// Check if the camera supports a specific resolution
    pub fn supports_resolution(&self, resolution: &Resolution) -> bool {
        self.capabilities.iter().any(|cap| {
            matches!(cap, CameraCapability::Resolution(r) if r == resolution)
        })
    }

    /// Check if the camera supports HDR
    pub fn supports_hdr(&self) -> bool {
        self.capabilities.iter().any(|cap| matches!(cap, CameraCapability::Hdr))
    }

    /// Get maximum supported resolution
    pub fn max_resolution(&self) -> Option<Resolution> {
        self.capabilities.iter()
            .filter_map(|cap| {
                if let CameraCapability::Resolution(r) = cap {
                    Some(*r)
                } else {
                    None
                }
            })
            .max_by_key(|r| r.width() * r.height())
    }
}

/// Camera capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum CameraCapability {
    Resolution(Resolution),
    FrameRate(FrameRate),
    Flash,
    AutoFocus,
    ManualFocus,
    Hdr,
    WhiteBalance(WhiteBalance),
    ExposureControl,
    ZoomDigital,
    ZoomOptical,
    FaceDetection,
    Stabilization,
}

/// Trait for camera device implementations
pub trait CameraDevice: Send + Sync {
    /// Get device information
    fn info(&self) -> &CameraInfo;

    /// Start streaming frames
    fn start_stream(&mut self) -> Result<(), CameraError>;

    /// Stop streaming
    fn stop_stream(&mut self) -> Result<(), CameraError>;

    /// Capture a single frame (for photo)
    fn capture_frame(&mut self) -> Result<FrameData, CameraError>;

    /// Set resolution
    fn set_resolution(&mut self, resolution: Resolution) -> Result<(), CameraError>;

    /// Set frame rate
    fn set_frame_rate(&mut self, fps: FrameRate) -> Result<(), CameraError>;

    /// Set exposure compensation (-2.0 to +2.0)
    fn set_exposure(&mut self, exposure: f32) -> Result<(), CameraError>;

    /// Set white balance mode
    fn set_white_balance(&mut self, wb: WhiteBalance) -> Result<(), CameraError>;

    /// Set focus mode
    fn set_focus_mode(&mut self, mode: FocusMode) -> Result<(), CameraError>;

    /// Enable/disable flash
    fn set_flash(&mut self, enabled: bool) -> Result<(), CameraError>;

    /// Check if device is streaming
    fn is_streaming(&self) -> bool;
}

/// Raw frame data from camera
#[derive(Debug, Clone)]
pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

impl FrameData {
    pub fn new(width: u32, height: u32, format: PixelFormat, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            format,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Convert frame to RGBA format
    pub fn to_rgba(&self) -> Vec<u8> {
        match self.format {
            PixelFormat::Rgba => self.data.clone(),
            PixelFormat::Rgb => {
                let mut rgba = Vec::with_capacity(self.data.len() * 4 / 3);
                for chunk in self.data.chunks(3) {
                    rgba.extend_from_slice(chunk);
                    rgba.push(255);
                }
                rgba
            }
            PixelFormat::Bgra => {
                let mut rgba = self.data.clone();
                for chunk in rgba.chunks_mut(4) {
                    chunk.swap(0, 2);
                }
                rgba
            }
            PixelFormat::Yuyv => {
                // YUYV to RGBA conversion
                yuyv_to_rgba(&self.data, self.width, self.height)
            }
            PixelFormat::Mjpeg => {
                // Decode MJPEG
                decode_mjpeg(&self.data)
                    .unwrap_or_else(|_| vec![0u8; (self.width * self.height * 4) as usize])
            }
            PixelFormat::Nv12 => {
                nv12_to_rgba(&self.data, self.width, self.height)
            }
        }
    }
}

/// Pixel format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba,
    Rgb,
    Bgra,
    Yuyv,
    Mjpeg,
    Nv12,
}

/// Camera errors
#[derive(Debug, thiserror::Error)]
pub enum CameraError {
    #[error("Camera device not found: {0}")]
    DeviceNotFound(String),

    #[error("Failed to open camera: {0}")]
    OpenFailed(String),

    #[error("Failed to start streaming: {0}")]
    StreamError(String),

    #[error("Failed to capture frame: {0}")]
    CaptureError(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Device busy: {0}")]
    DeviceBusy(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Helper functions for pixel format conversion

fn yuyv_to_rgba(yuyv: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let num_pixels = (width * height) as usize;

    for i in 0..(num_pixels / 2) {
        let y0 = yuyv[i * 4] as f32;
        let u = yuyv[i * 4 + 1] as f32 - 128.0;
        let y1 = yuyv[i * 4 + 2] as f32;
        let v = yuyv[i * 4 + 3] as f32 - 128.0;

        // YUV to RGB conversion
        let r0 = (y0 + 1.402 * v).clamp(0.0, 255.0) as u8;
        let g0 = (y0 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
        let b0 = (y0 + 1.772 * u).clamp(0.0, 255.0) as u8;

        let r1 = (y1 + 1.402 * v).clamp(0.0, 255.0) as u8;
        let g1 = (y1 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
        let b1 = (y1 + 1.772 * u).clamp(0.0, 255.0) as u8;

        rgba[i * 8] = r0;
        rgba[i * 8 + 1] = g0;
        rgba[i * 8 + 2] = b0;
        rgba[i * 8 + 3] = 255;

        rgba[i * 8 + 4] = r1;
        rgba[i * 8 + 5] = g1;
        rgba[i * 8 + 6] = b1;
        rgba[i * 8 + 7] = 255;
    }

    rgba
}

fn nv12_to_rgba(nv12: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let mut rgba = vec![0u8; w * h * 4];

    let y_plane = &nv12[0..w * h];
    let uv_plane = &nv12[w * h..];

    for y in 0..h {
        for x in 0..w {
            let y_val = y_plane[y * w + x] as f32;
            let uv_idx = (y / 2) * w + (x / 2) * 2;
            let u = uv_plane.get(uv_idx).copied().unwrap_or(128) as f32 - 128.0;
            let v = uv_plane.get(uv_idx + 1).copied().unwrap_or(128) as f32 - 128.0;

            let r = (y_val + 1.402 * v).clamp(0.0, 255.0) as u8;
            let g = (y_val - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
            let b = (y_val + 1.772 * u).clamp(0.0, 255.0) as u8;

            let idx = (y * w + x) * 4;
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = 255;
        }
    }

    rgba
}

fn decode_mjpeg(data: &[u8]) -> Result<Vec<u8>, CameraError> {
    // Use the image crate to decode MJPEG
    let img = image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)
        .map_err(|e| CameraError::CaptureError(format!("MJPEG decode failed: {}", e)))?;

    Ok(img.to_rgba8().into_raw())
}
