//! PipeWire screen capture integration
//!
//! Handles the actual video stream capture from PipeWire
//! after the portal has granted access.

use anyhow::{Result, anyhow};
use std::sync::Arc;
use parking_lot::Mutex;

/// Frame data from PipeWire
#[derive(Debug, Clone)]
pub struct CaptureFrame {
    /// Raw pixel data (BGRA or similar)
    pub data: Vec<u8>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Bytes per row (stride)
    pub stride: u32,
    /// Timestamp in nanoseconds
    pub timestamp: u64,
    /// Pixel format
    pub format: PixelFormat,
}

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    BGRA,
    RGBA,
    BGRx,
    RGBx,
    NV12,
    I420,
}

impl PixelFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            PixelFormat::BGRA | PixelFormat::RGBA |
            PixelFormat::BGRx | PixelFormat::RGBx => 4,
            PixelFormat::NV12 | PixelFormat::I420 => 1, // Planar formats
        }
    }

    pub fn gstreamer_format(&self) -> &'static str {
        match self {
            PixelFormat::BGRA => "BGRA",
            PixelFormat::RGBA => "RGBA",
            PixelFormat::BGRx => "BGRx",
            PixelFormat::RGBx => "RGBx",
            PixelFormat::NV12 => "NV12",
            PixelFormat::I420 => "I420",
        }
    }
}

/// Callback type for frame delivery
pub type FrameCallback = Box<dyn Fn(CaptureFrame) + Send + Sync>;

/// PipeWire capture state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureState {
    /// Not started
    Idle,
    /// Connecting to PipeWire
    Connecting,
    /// Actively capturing
    Capturing,
    /// Paused
    Paused,
    /// Stopped
    Stopped,
    /// Error occurred
    Error,
}

/// PipeWire capture configuration
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Target framerate
    pub framerate: u32,
    /// Preferred pixel format
    pub format: Option<PixelFormat>,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            framerate: 30,
            format: None,
        }
    }
}

/// PipeWire screen capture handler
///
/// This struct manages the connection to PipeWire and handles
/// the video stream from the screencast portal.
pub struct PipeWireCapture {
    /// PipeWire node ID to capture from
    node_id: u32,
    /// Current state
    state: Arc<Mutex<CaptureState>>,
    /// Capture configuration
    config: CaptureConfig,
    /// Frame statistics
    stats: Arc<Mutex<CaptureStats>>,
}

/// Capture statistics
#[derive(Debug, Default, Clone)]
pub struct CaptureStats {
    /// Total frames captured
    pub frames_captured: u64,
    /// Dropped frames
    pub frames_dropped: u64,
    /// Average framerate
    pub avg_framerate: f64,
    /// Last frame timestamp
    pub last_timestamp: u64,
    /// Total bytes captured
    pub bytes_captured: u64,
}

impl PipeWireCapture {
    /// Create a new PipeWire capture instance
    pub fn new(node_id: u32, config: CaptureConfig) -> Self {
        Self {
            node_id,
            state: Arc::new(Mutex::new(CaptureState::Idle)),
            config,
            stats: Arc::new(Mutex::new(CaptureStats::default())),
        }
    }

    /// Get the PipeWire node ID
    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    /// Get the current capture state
    pub fn state(&self) -> CaptureState {
        *self.state.lock()
    }

    /// Get capture statistics
    pub fn stats(&self) -> CaptureStats {
        self.stats.lock().clone()
    }

    /// Start capturing frames
    ///
    /// This method initializes the PipeWire connection and starts
    /// receiving frames. Frames are delivered via the provided callback.
    pub fn start(&self, callback: FrameCallback) -> Result<()> {
        let mut state = self.state.lock();
        if *state != CaptureState::Idle && *state != CaptureState::Stopped {
            return Err(anyhow!("Capture already in progress"));
        }
        *state = CaptureState::Connecting;
        drop(state);

        // In a real implementation, this would:
        // 1. Initialize PipeWire main loop
        // 2. Connect to the PipeWire daemon
        // 3. Create a stream connected to node_id
        // 4. Set up buffer handlers
        // 5. Start the stream

        // For now, we'll use a placeholder that simulates the capture
        // The actual implementation would use the pipewire crate
        self.simulate_capture(callback)?;

        Ok(())
    }

    /// Pause capturing
    pub fn pause(&self) -> Result<()> {
        let mut state = self.state.lock();
        if *state != CaptureState::Capturing {
            return Err(anyhow!("Not currently capturing"));
        }
        *state = CaptureState::Paused;
        Ok(())
    }

    /// Resume capturing
    pub fn resume(&self) -> Result<()> {
        let mut state = self.state.lock();
        if *state != CaptureState::Paused {
            return Err(anyhow!("Not paused"));
        }
        *state = CaptureState::Capturing;
        Ok(())
    }

    /// Stop capturing
    pub fn stop(&self) -> Result<()> {
        let mut state = self.state.lock();
        *state = CaptureState::Stopped;
        Ok(())
    }

    /// Simulate capture for development/testing
    fn simulate_capture(&self, _callback: FrameCallback) -> Result<()> {
        *self.state.lock() = CaptureState::Capturing;

        // In a real implementation, frames would be delivered via the callback
        // For now, this is a placeholder

        Ok(())
    }

    /// Build a GStreamer pipeline for PipeWire capture
    pub fn gstreamer_source_element(&self) -> String {
        format!(
            "pipewiresrc path={} do-timestamp=true ! \
             videoconvert ! \
             video/x-raw,framerate={}/1",
            self.node_id,
            self.config.framerate
        )
    }
}

/// Build a GStreamer pipeline string for PipeWire capture
pub fn build_gstreamer_pipeline(
    node_id: u32,
    framerate: u32,
    width: Option<u32>,
    height: Option<u32>,
) -> String {
    let mut pipeline = format!(
        "pipewiresrc path={} do-timestamp=true",
        node_id
    );

    // Add video conversion
    pipeline.push_str(" ! videoconvert");

    // Add scaling if dimensions specified
    if let (Some(w), Some(h)) = (width, height) {
        pipeline.push_str(&format!(
            " ! videoscale ! video/x-raw,width={},height={}",
            w, h
        ));
    }

    // Add framerate
    pipeline.push_str(&format!(
        " ! video/x-raw,framerate={}/1",
        framerate
    ));

    pipeline
}

/// Check if PipeWire is available
pub fn is_available() -> bool {
    // Check for PipeWire socket
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        let socket_path = std::path::Path::new(&runtime_dir).join("pipewire-0");
        return socket_path.exists();
    }
    false
}

/// Get PipeWire version string
pub fn get_version() -> Option<String> {
    // Try to get version from pw-cli
    let output = std::process::Command::new("pw-cli")
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}
