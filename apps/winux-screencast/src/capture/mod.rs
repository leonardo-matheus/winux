//! Capture module - handles screen capture sources
//!
//! This module provides the infrastructure for capturing screen content
//! using XDG Desktop Portal and PipeWire on Wayland.

pub mod portal;
pub mod pipewire;
pub mod encoder;

pub use portal::ScreencastPortal;
pub use pipewire::PipeWireCapture;
pub use encoder::VideoEncoder;

use anyhow::{Result, anyhow};
use std::path::PathBuf;

/// Source type for screen recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceType {
    /// Record entire screen
    #[default]
    Screen,
    /// Record specific window
    Window,
    /// Record custom region
    Region,
    /// Record all monitors (virtual screen)
    VirtualScreen,
}

impl SourceType {
    pub fn label(&self) -> &'static str {
        match self {
            SourceType::Screen => "Screen",
            SourceType::Window => "Window",
            SourceType::Region => "Region",
            SourceType::VirtualScreen => "All Monitors",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SourceType::Screen => "video-display-symbolic",
            SourceType::Window => "window-symbolic",
            SourceType::Region => "edit-select-all-symbolic",
            SourceType::VirtualScreen => "view-dual-symbolic",
        }
    }

    pub fn portal_source_type(&self) -> u32 {
        match self {
            SourceType::Screen | SourceType::VirtualScreen => 1, // MONITOR
            SourceType::Window => 2, // WINDOW
            SourceType::Region => 1, // MONITOR with region selection
        }
    }
}

/// Capture source information returned by portal
#[derive(Debug, Clone)]
pub struct CaptureSource {
    /// PipeWire node ID for the capture stream
    pub node_id: u32,
    /// Source type
    pub source_type: SourceType,
    /// Width of the capture
    pub width: u32,
    /// Height of the capture
    pub height: u32,
    /// Position X (for region capture)
    pub x: i32,
    /// Position Y (for region capture)
    pub y: i32,
    /// Associated portal session path
    pub session_path: String,
}

/// Region selection for partial screen capture
#[derive(Debug, Clone, Copy, Default)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl CaptureRegion {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Cursor mode for recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorMode {
    /// Hide cursor
    Hidden,
    /// Show cursor as part of the stream
    #[default]
    Embedded,
    /// Show cursor metadata separately
    Metadata,
}

impl CursorMode {
    pub fn portal_value(&self) -> u32 {
        match self {
            CursorMode::Hidden => 1,
            CursorMode::Embedded => 2,
            CursorMode::Metadata => 4,
        }
    }
}

/// Initialize capture subsystem
pub fn init() -> Result<()> {
    // Check if portal is available
    if !portal::is_portal_available() {
        return Err(anyhow!("XDG Desktop Portal is not available. Please ensure xdg-desktop-portal and a portal backend are installed."));
    }

    Ok(())
}

/// Check if screen capture is available
pub fn is_capture_available() -> bool {
    portal::is_portal_available()
}

/// Start a capture session
pub async fn start_capture_session(
    source_type: SourceType,
    cursor_mode: CursorMode,
) -> Result<CaptureSource> {
    let portal = ScreencastPortal::new().await?;

    // Create session
    let session_path = portal.create_session().await?;

    // Select sources
    portal.select_sources(
        &session_path,
        source_type.portal_source_type(),
        cursor_mode.portal_value(),
        true, // Allow multiple sources
    ).await?;

    // Start the stream
    let streams = portal.start(&session_path).await?;

    if streams.is_empty() {
        return Err(anyhow!("No capture sources selected"));
    }

    let stream = &streams[0];

    Ok(CaptureSource {
        node_id: stream.node_id,
        source_type,
        width: stream.width,
        height: stream.height,
        x: stream.x,
        y: stream.y,
        session_path,
    })
}

/// Stop a capture session
pub async fn stop_capture_session(session_path: &str) -> Result<()> {
    let portal = ScreencastPortal::new().await?;
    portal.close_session(session_path).await
}
