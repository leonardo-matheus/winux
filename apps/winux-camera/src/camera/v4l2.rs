//! V4L2 (Video4Linux2) camera fallback implementation
//!
//! V4L2 is the traditional Linux API for video capture devices.
//! Used as a fallback when PipeWire is not available.

use std::path::PathBuf;
use super::device::{CameraInfo, CameraDevice, CameraError, FrameData, PixelFormat, CameraCapability};
use super::{Resolution, FrameRate, WhiteBalance, FocusMode};

/// V4L2 camera implementation
pub struct V4L2Camera {
    info: CameraInfo,
    is_streaming: bool,
    resolution: Resolution,
    frame_rate: FrameRate,
    // In a full implementation:
    // device: Option<v4l::Device>,
    // buffers: Vec<v4l::Buffer>,
}

impl V4L2Camera {
    /// Create a new V4L2 camera instance
    pub fn new(info: CameraInfo) -> Self {
        Self {
            info,
            is_streaming: false,
            resolution: Resolution::default(),
            frame_rate: FrameRate::default(),
        }
    }

    /// Enumerate available V4L2 camera devices
    pub fn enumerate_devices() -> Vec<CameraInfo> {
        log::debug!("Enumerating V4L2 cameras...");

        let mut cameras = Vec::new();

        // Scan /dev for video devices
        for i in 0..10 {
            let device_path = PathBuf::from(format!("/dev/video{}", i));

            if !device_path.exists() {
                continue;
            }

            // Try to open and query the device
            if let Some(info) = Self::query_device(&device_path, i) {
                cameras.push(info);
            }
        }

        log::info!("Found {} V4L2 camera(s)", cameras.len());
        cameras
    }

    /// Query a V4L2 device for its capabilities
    fn query_device(path: &PathBuf, index: usize) -> Option<CameraInfo> {
        // In a full implementation, this would use ioctl to query device caps
        // For now, we do basic file-based detection

        // Check if it's a video capture device by reading sysfs
        let sysfs_path = PathBuf::from(format!("/sys/class/video4linux/video{}", index));

        if !sysfs_path.exists() {
            return None;
        }

        // Read device name
        let name_path = sysfs_path.join("name");
        let device_name = std::fs::read_to_string(&name_path)
            .unwrap_or_else(|_| format!("Camera {}", index));

        // Read device capabilities from sysfs
        let dev_debug_path = sysfs_path.join("dev_debug");
        let is_capture_device = std::fs::read_to_string(&dev_debug_path)
            .map(|s| s.contains("VIDEO_CAPTURE"))
            .unwrap_or(true); // Assume it's a capture device if we can't read

        if !is_capture_device {
            return None;
        }

        let mut info = CameraInfo::new(
            &format!("video{}", index),
            device_name.trim(),
            path.clone(),
        );

        // Query supported formats and capabilities
        info.capabilities = Self::query_capabilities(path);

        Some(info)
    }

    /// Query device capabilities
    fn query_capabilities(path: &PathBuf) -> Vec<CameraCapability> {
        let mut caps = Vec::new();

        // In a full implementation, use VIDIOC_ENUM_FMT and VIDIOC_ENUM_FRAMESIZES
        // For now, assume common capabilities

        // Check if we can access the device
        if std::fs::metadata(path).is_ok() {
            // Common resolutions supported by most webcams
            caps.push(CameraCapability::Resolution(Resolution::Hd720p));
            caps.push(CameraCapability::Resolution(Resolution::Hd1080p));

            // Common frame rates
            caps.push(CameraCapability::FrameRate(FrameRate::Fps30));

            // Auto focus is common
            caps.push(CameraCapability::AutoFocus);

            // White balance control
            caps.push(CameraCapability::WhiteBalance(WhiteBalance::Auto));

            // Exposure control
            caps.push(CameraCapability::ExposureControl);
        }

        caps
    }

    /// Open the V4L2 device
    fn open_device(&mut self) -> Result<(), CameraError> {
        // In a full implementation:
        // self.device = Some(v4l::Device::new(&self.info.device_path)?);

        // Check permissions
        if !self.info.device_path.exists() {
            return Err(CameraError::DeviceNotFound(
                self.info.device_path.display().to_string()
            ));
        }

        // Check read permission
        std::fs::metadata(&self.info.device_path)
            .map_err(|e| CameraError::PermissionDenied(e.to_string()))?;

        Ok(())
    }

    /// Configure the device format
    fn configure_format(&mut self) -> Result<(), CameraError> {
        // In a full implementation, this would use VIDIOC_S_FMT
        log::debug!(
            "Configuring V4L2 format: {}x{} @ {} FPS",
            self.resolution.width(),
            self.resolution.height(),
            self.frame_rate.value()
        );
        Ok(())
    }

    /// Request memory-mapped buffers
    #[allow(dead_code)]
    fn request_buffers(&mut self, count: u32) -> Result<(), CameraError> {
        // In a full implementation, this would use VIDIOC_REQBUFS
        log::debug!("Requesting {} V4L2 buffers", count);
        Ok(())
    }

    /// Generate a simulated frame for testing
    fn generate_test_frame(&self) -> FrameData {
        let width = self.resolution.width();
        let height = self.resolution.height();

        // Create a test pattern
        let mut data = vec![0u8; (width * height * 4) as usize];
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Create color bars pattern (classic video test pattern)
        let bar_width = width / 8;
        let colors = [
            (255, 255, 255), // White
            (255, 255, 0),   // Yellow
            (0, 255, 255),   // Cyan
            (0, 255, 0),     // Green
            (255, 0, 255),   // Magenta
            (255, 0, 0),     // Red
            (0, 0, 255),     // Blue
            (0, 0, 0),       // Black
        ];

        for y in 0..height {
            for x in 0..width {
                let bar_index = ((x / bar_width) as usize).min(7);
                let (r, g, b) = colors[bar_index];

                let idx = ((y * width + x) * 4) as usize;
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = b;
                data[idx + 3] = 255;
            }
        }

        FrameData {
            width,
            height,
            format: PixelFormat::Rgba,
            data,
            timestamp,
        }
    }
}

impl CameraDevice for V4L2Camera {
    fn info(&self) -> &CameraInfo {
        &self.info
    }

    fn start_stream(&mut self) -> Result<(), CameraError> {
        log::info!("Starting V4L2 stream for camera: {}", self.info.name);

        // Open device
        self.open_device()?;

        // Configure format
        self.configure_format()?;

        // In a full implementation:
        // 1. Request buffers (VIDIOC_REQBUFS)
        // 2. Map buffers (mmap)
        // 3. Queue buffers (VIDIOC_QBUF)
        // 4. Start streaming (VIDIOC_STREAMON)

        self.is_streaming = true;
        Ok(())
    }

    fn stop_stream(&mut self) -> Result<(), CameraError> {
        log::info!("Stopping V4L2 stream");

        // In a full implementation:
        // 1. Stop streaming (VIDIOC_STREAMOFF)
        // 2. Unmap buffers
        // 3. Close device

        self.is_streaming = false;
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<FrameData, CameraError> {
        if !self.is_streaming {
            return Err(CameraError::StreamError("Camera not streaming".into()));
        }

        // In a full implementation:
        // 1. Dequeue buffer (VIDIOC_DQBUF)
        // 2. Copy frame data
        // 3. Requeue buffer (VIDIOC_QBUF)

        // For now, generate a test frame
        Ok(self.generate_test_frame())
    }

    fn set_resolution(&mut self, resolution: Resolution) -> Result<(), CameraError> {
        log::debug!("Setting V4L2 resolution to {}x{}", resolution.width(), resolution.height());

        // Would need to stop stream, reconfigure, and restart
        let was_streaming = self.is_streaming;
        if was_streaming {
            self.stop_stream()?;
        }

        self.resolution = resolution;

        if was_streaming {
            self.start_stream()?;
        }

        Ok(())
    }

    fn set_frame_rate(&mut self, fps: FrameRate) -> Result<(), CameraError> {
        log::debug!("Setting V4L2 frame rate to {} FPS", fps.value());

        // In a full implementation, use VIDIOC_S_PARM
        self.frame_rate = fps;
        Ok(())
    }

    fn set_exposure(&mut self, exposure: f32) -> Result<(), CameraError> {
        log::debug!("Setting V4L2 exposure to {}", exposure);

        // In a full implementation, use V4L2_CID_EXPOSURE_ABSOLUTE
        // Clamp to valid range
        let _clamped = exposure.clamp(-2.0, 2.0);
        Ok(())
    }

    fn set_white_balance(&mut self, wb: WhiteBalance) -> Result<(), CameraError> {
        log::debug!("Setting V4L2 white balance to {:?}", wb);

        // In a full implementation, use V4L2_CID_AUTO_WHITE_BALANCE
        // and V4L2_CID_WHITE_BALANCE_TEMPERATURE
        Ok(())
    }

    fn set_focus_mode(&mut self, mode: FocusMode) -> Result<(), CameraError> {
        log::debug!("Setting V4L2 focus mode to {:?}", mode);

        // In a full implementation, use V4L2_CID_FOCUS_AUTO
        Ok(())
    }

    fn set_flash(&mut self, enabled: bool) -> Result<(), CameraError> {
        log::debug!("Setting V4L2 flash/LED to {}", enabled);

        // In a full implementation, use V4L2_CID_FLASH_LED_MODE
        // Most webcams don't have flash support
        Ok(())
    }

    fn is_streaming(&self) -> bool {
        self.is_streaming
    }
}
