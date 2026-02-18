//! Configuration - Application settings management
//!
//! Handles loading, saving, and managing user preferences for Winux Player.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Window settings
    pub window: WindowConfig,
    /// Playback settings
    pub playback: PlaybackConfig,
    /// Subtitle settings
    pub subtitles: SubtitleConfig,
    /// Audio settings
    pub audio: AudioConfig,
    /// Video settings
    pub video: VideoConfig,
    /// Keyboard shortcuts
    pub shortcuts: ShortcutsConfig,
    /// Recent files
    pub recent_files: Vec<String>,
    /// Maximum recent files to remember
    pub max_recent_files: usize,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Default window width
    pub width: i32,
    /// Default window height
    pub height: i32,
    /// Remember window size
    pub remember_size: bool,
    /// Start maximized
    pub start_maximized: bool,
    /// Start fullscreen
    pub start_fullscreen: bool,
    /// Always on top
    pub always_on_top: bool,
    /// Hide controls after timeout (seconds)
    pub hide_controls_timeout: u32,
}

/// Playback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackConfig {
    /// Default volume (0.0 - 1.0)
    pub default_volume: f64,
    /// Remember volume
    pub remember_volume: bool,
    /// Resume playback position
    pub resume_playback: bool,
    /// Default playback speed
    pub default_speed: f64,
    /// Auto-play on open
    pub auto_play: bool,
    /// Loop single file
    pub loop_single: bool,
    /// Hardware acceleration
    pub hardware_acceleration: bool,
}

/// Subtitle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleConfig {
    /// Auto-load subtitles with same name as video
    pub auto_load: bool,
    /// Preferred subtitle language (ISO 639-1 code)
    pub preferred_language: Option<String>,
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: u32,
    /// Font color (hex)
    pub font_color: String,
    /// Background color (hex)
    pub background_color: String,
    /// Outline/border
    pub outline: bool,
    /// Outline color
    pub outline_color: String,
    /// Position from bottom (0.0 - 1.0)
    pub position: f64,
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Preferred audio language
    pub preferred_language: Option<String>,
    /// Audio delay (seconds)
    pub delay: f64,
    /// Audio channels (stereo, 5.1, etc.)
    pub channels: String,
    /// Normalize audio
    pub normalize: bool,
}

/// Video configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Aspect ratio override (None = auto)
    pub aspect_ratio: Option<String>,
    /// Deinterlace mode
    pub deinterlace: DeinterlaceMode,
    /// Video filters
    pub filters: VideoFilters,
    /// Screenshot format
    pub screenshot_format: String,
    /// Screenshot directory
    pub screenshot_dir: Option<String>,
}

/// Deinterlace mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeinterlaceMode {
    Off,
    Auto,
    On,
}

/// Video filters configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFilters {
    /// Brightness (-100 to 100)
    pub brightness: i32,
    /// Contrast (-100 to 100)
    pub contrast: i32,
    /// Saturation (-100 to 100)
    pub saturation: i32,
    /// Hue (-180 to 180)
    pub hue: i32,
    /// Gamma (0.1 to 10.0)
    pub gamma: f64,
}

/// Keyboard shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    /// Play/Pause
    pub play_pause: String,
    /// Stop
    pub stop: String,
    /// Seek forward (short)
    pub seek_forward_short: String,
    /// Seek forward (medium)
    pub seek_forward_medium: String,
    /// Seek forward (long)
    pub seek_forward_long: String,
    /// Seek backward (short)
    pub seek_backward_short: String,
    /// Seek backward (medium)
    pub seek_backward_medium: String,
    /// Seek backward (long)
    pub seek_backward_long: String,
    /// Volume up
    pub volume_up: String,
    /// Volume down
    pub volume_down: String,
    /// Mute
    pub mute: String,
    /// Fullscreen
    pub fullscreen: String,
    /// Picture-in-Picture
    pub pip: String,
    /// Toggle playlist
    pub toggle_playlist: String,
    /// Open file
    pub open_file: String,
    /// Open URL
    pub open_url: String,
    /// Screenshot
    pub screenshot: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            playback: PlaybackConfig::default(),
            subtitles: SubtitleConfig::default(),
            audio: AudioConfig::default(),
            video: VideoConfig::default(),
            shortcuts: ShortcutsConfig::default(),
            recent_files: Vec::new(),
            max_recent_files: 20,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            remember_size: true,
            start_maximized: false,
            start_fullscreen: false,
            always_on_top: false,
            hide_controls_timeout: 3,
        }
    }
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            default_volume: 1.0,
            remember_volume: true,
            resume_playback: true,
            default_speed: 1.0,
            auto_play: true,
            loop_single: false,
            hardware_acceleration: true,
        }
    }
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            auto_load: true,
            preferred_language: None,
            font_family: "Sans".to_string(),
            font_size: 24,
            font_color: "#FFFFFF".to_string(),
            background_color: "#00000080".to_string(),
            outline: true,
            outline_color: "#000000".to_string(),
            position: 0.1,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            preferred_language: None,
            delay: 0.0,
            channels: "stereo".to_string(),
            normalize: false,
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: None,
            deinterlace: DeinterlaceMode::Auto,
            filters: VideoFilters::default(),
            screenshot_format: "png".to_string(),
            screenshot_dir: None,
        }
    }
}

impl Default for VideoFilters {
    fn default() -> Self {
        Self {
            brightness: 0,
            contrast: 0,
            saturation: 0,
            hue: 0,
            gamma: 1.0,
        }
    }
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            play_pause: "space".to_string(),
            stop: "s".to_string(),
            seek_forward_short: "Right".to_string(),
            seek_forward_medium: "<Shift>Right".to_string(),
            seek_forward_long: "<Control>Right".to_string(),
            seek_backward_short: "Left".to_string(),
            seek_backward_medium: "<Shift>Left".to_string(),
            seek_backward_long: "<Control>Left".to_string(),
            volume_up: "Up".to_string(),
            volume_down: "Down".to_string(),
            mute: "m".to_string(),
            fullscreen: "F11".to_string(),
            pip: "p".to_string(),
            toggle_playlist: "<Control>l".to_string(),
            open_file: "<Control>o".to_string(),
            open_url: "<Control>u".to_string(),
            screenshot: "<Control>s".to_string(),
        }
    }
}

impl Config {
    /// Get config file path
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-player")
            .join("config.toml")
    }

    /// Load configuration from file
    pub fn load() -> Option<Self> {
        let path = Self::config_path();

        if !path.exists() {
            info!("Config file not found, using defaults");
            return None;
        }

        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    info!("Configuration loaded from {:?}", path);
                    Some(config)
                }
                Err(e) => {
                    error!("Failed to parse config: {}", e);
                    None
                }
            },
            Err(e) => {
                error!("Failed to read config: {}", e);
                None
            }
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;

        info!("Configuration saved to {:?}", path);
        Ok(())
    }

    /// Add file to recent files list
    pub fn add_recent_file(&mut self, uri: &str) {
        // Remove if already exists
        self.recent_files.retain(|f| f != uri);

        // Add to front
        self.recent_files.insert(0, uri.to_string());

        // Trim to max size
        self.recent_files.truncate(self.max_recent_files);
    }

    /// Get playback position for a file (for resume feature)
    pub fn get_playback_position(&self, uri: &str) -> Option<f64> {
        // This would typically read from a separate positions file
        // For simplicity, we'll return None here
        None
    }

    /// Save playback position for a file
    pub fn save_playback_position(&mut self, uri: &str, position: f64) {
        // This would typically save to a separate positions file
        info!("Saving position {} for {}", position, uri);
    }
}

/// Playback position storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybackPositions {
    /// Map of URI to position (seconds)
    pub positions: std::collections::HashMap<String, f64>,
}

impl PlaybackPositions {
    /// Get positions file path
    fn positions_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-player")
            .join("positions.json")
    }

    /// Load positions from file
    pub fn load() -> Self {
        let path = Self::positions_path();

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save positions to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::positions_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    /// Get position for URI
    pub fn get(&self, uri: &str) -> Option<f64> {
        self.positions.get(uri).copied()
    }

    /// Set position for URI
    pub fn set(&mut self, uri: &str, position: f64) {
        self.positions.insert(uri.to_string(), position);
    }

    /// Remove position for URI (e.g., when playback completes)
    pub fn remove(&mut self, uri: &str) {
        self.positions.remove(uri);
    }

    /// Clear old positions (older than specified days)
    pub fn cleanup(&mut self, max_entries: usize) {
        if self.positions.len() > max_entries {
            // Keep only the most recent entries
            // Since we don't track timestamps, just truncate
            let mut entries: Vec<_> = self.positions.drain().collect();
            entries.truncate(max_entries);
            self.positions.extend(entries);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.window.width, 1280);
        assert_eq!(config.window.height, 720);
        assert_eq!(config.playback.default_volume, 1.0);
    }

    #[test]
    fn test_recent_files() {
        let mut config = Config::default();
        config.max_recent_files = 3;

        config.add_recent_file("file1.mp4");
        config.add_recent_file("file2.mp4");
        config.add_recent_file("file3.mp4");
        config.add_recent_file("file4.mp4");

        assert_eq!(config.recent_files.len(), 3);
        assert_eq!(config.recent_files[0], "file4.mp4");
    }

    #[test]
    fn test_serialize_config() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[window]"));
        assert!(toml_str.contains("[playback]"));
    }
}
