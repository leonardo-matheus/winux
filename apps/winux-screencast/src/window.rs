//! Window utilities and monitor detection
//!
//! Provides utilities for detecting available monitors and windows
//! for screen recording source selection.

use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Information about a display/monitor
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Unique identifier
    pub id: u32,
    /// Display name (e.g., "DP-1", "HDMI-A-1")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// X position in virtual screen
    pub x: i32,
    /// Y position in virtual screen
    pub y: i32,
    /// Refresh rate in mHz
    pub refresh_rate: u32,
    /// Scale factor
    pub scale: f64,
    /// Whether this is the primary monitor
    pub is_primary: bool,
}

impl MonitorInfo {
    /// Get a formatted resolution string
    pub fn resolution_string(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// Get refresh rate in Hz
    pub fn refresh_hz(&self) -> f64 {
        self.refresh_rate as f64 / 1000.0
    }
}

/// Information about a window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window identifier
    pub id: String,
    /// Window title
    pub title: String,
    /// Application name
    pub app_name: String,
    /// Application icon name
    pub icon_name: Option<String>,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
}

/// Get list of available monitors using GDK
pub fn get_monitors() -> Vec<MonitorInfo> {
    use gtk4 as gtk;
    use gtk::prelude::*;

    let mut monitors = Vec::new();
    let display = gtk::gdk::Display::default().expect("No display available");
    let monitor_list = display.monitors();

    for i in 0..monitor_list.n_items() {
        if let Some(obj) = monitor_list.item(i) {
            if let Ok(monitor) = obj.downcast::<gtk::gdk::Monitor>() {
                let geometry = monitor.geometry();
                let connector = monitor.connector().map(|s| s.to_string()).unwrap_or_default();
                let description = monitor.description().map(|s| s.to_string()).unwrap_or_default();

                monitors.push(MonitorInfo {
                    id: i,
                    name: connector.clone(),
                    description: if description.is_empty() {
                        format!("Monitor {}", i + 1)
                    } else {
                        description
                    },
                    width: geometry.width() as u32,
                    height: geometry.height() as u32,
                    x: geometry.x(),
                    y: geometry.y(),
                    refresh_rate: monitor.refresh_rate() as u32,
                    scale: monitor.scale_factor() as f64,
                    is_primary: i == 0, // First monitor is typically primary
                });
            }
        }
    }

    monitors
}

/// Get the primary monitor
pub fn get_primary_monitor() -> Option<MonitorInfo> {
    get_monitors().into_iter().find(|m| m.is_primary)
}

/// Get total virtual screen dimensions (spanning all monitors)
pub fn get_virtual_screen_size() -> (u32, u32) {
    let monitors = get_monitors();
    if monitors.is_empty() {
        return (1920, 1080); // Default fallback
    }

    let mut max_x = 0i32;
    let mut max_y = 0i32;

    for monitor in &monitors {
        let right = monitor.x + monitor.width as i32;
        let bottom = monitor.y + monitor.height as i32;
        max_x = max_x.max(right);
        max_y = max_y.max(bottom);
    }

    (max_x as u32, max_y as u32)
}

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether this is an input (microphone) device
    pub is_input: bool,
    /// Whether this is the default device
    pub is_default: bool,
}

/// Get available audio input devices (microphones)
pub fn get_audio_input_devices() -> Vec<AudioDevice> {
    // This would typically use PulseAudio/PipeWire to enumerate devices
    // For now, return a default device
    vec![
        AudioDevice {
            id: "default".to_string(),
            name: "Default Microphone".to_string(),
            is_input: true,
            is_default: true,
        },
    ]
}

/// Get available audio output devices (for system audio capture)
pub fn get_audio_output_devices() -> Vec<AudioDevice> {
    vec![
        AudioDevice {
            id: "default".to_string(),
            name: "System Audio".to_string(),
            is_input: false,
            is_default: true,
        },
    ]
}
