//! Configuration module for Winux Compositor
//!
//! Handles loading, parsing, and managing compositor configuration
//! from TOML files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure for the compositor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CompositorConfig {
    /// General compositor settings
    pub general: GeneralConfig,
    /// Display/output configuration
    pub outputs: OutputsConfig,
    /// Input device configuration
    pub input: InputConfig,
    /// Appearance settings
    pub appearance: AppearanceConfig,
    /// Performance tuning
    pub performance: PerformanceConfig,
}

impl Default for CompositorConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            outputs: OutputsConfig::default(),
            input: InputConfig::default(),
            appearance: AppearanceConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// General compositor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Enable VSync
    pub vsync: bool,
    /// Default backend: "auto", "drm", "winit"
    pub backend: String,
    /// Log level: "trace", "debug", "info", "warn", "error"
    pub log_level: String,
    /// Enable XWayland support
    pub xwayland: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            backend: "auto".to_string(),
            log_level: "info".to_string(),
            xwayland: true,
        }
    }
}

/// Output/display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputsConfig {
    /// Default scale factor
    pub scale: f64,
    /// Default refresh rate in Hz (0 = auto)
    pub refresh_rate: u32,
    /// Output-specific configurations
    pub outputs: Vec<OutputConfig>,
}

impl Default for OutputsConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            refresh_rate: 0,
            outputs: Vec::new(),
        }
    }
}

/// Individual output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output name/identifier
    pub name: String,
    /// Enable this output
    pub enabled: bool,
    /// Resolution (width x height)
    pub resolution: Option<(u32, u32)>,
    /// Position on the virtual screen
    pub position: Option<(i32, i32)>,
    /// Scale factor for this output
    pub scale: Option<f64>,
    /// Refresh rate in Hz
    pub refresh_rate: Option<u32>,
    /// Rotation in degrees (0, 90, 180, 270)
    pub rotation: Option<u32>,
}

/// Input device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    /// Keyboard settings
    pub keyboard: KeyboardConfig,
    /// Mouse/pointer settings
    pub pointer: PointerConfig,
    /// Touchpad settings
    pub touchpad: TouchpadConfig,
    /// Touch screen settings
    pub touch: TouchConfig,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            keyboard: KeyboardConfig::default(),
            pointer: PointerConfig::default(),
            touchpad: TouchpadConfig::default(),
            touch: TouchConfig::default(),
        }
    }
}

/// Keyboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardConfig {
    /// XKB layout
    pub layout: String,
    /// XKB variant
    pub variant: String,
    /// XKB options
    pub options: String,
    /// Repeat delay in ms
    pub repeat_delay: u32,
    /// Repeat rate in chars/sec
    pub repeat_rate: u32,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            layout: "us".to_string(),
            variant: String::new(),
            options: String::new(),
            repeat_delay: 400,
            repeat_rate: 25,
        }
    }
}

/// Pointer/mouse configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PointerConfig {
    /// Acceleration profile: "flat", "adaptive"
    pub accel_profile: String,
    /// Acceleration speed (-1.0 to 1.0)
    pub accel_speed: f64,
    /// Natural scrolling
    pub natural_scroll: bool,
    /// Left-handed mode
    pub left_handed: bool,
}

impl Default for PointerConfig {
    fn default() -> Self {
        Self {
            accel_profile: "adaptive".to_string(),
            accel_speed: 0.0,
            natural_scroll: false,
            left_handed: false,
        }
    }
}

/// Touchpad configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TouchpadConfig {
    /// Enable tap-to-click
    pub tap_to_click: bool,
    /// Enable two-finger tap for right-click
    pub two_finger_right_click: bool,
    /// Natural scrolling
    pub natural_scroll: bool,
    /// Disable while typing
    pub disable_while_typing: bool,
    /// Scroll method: "two_finger", "edge", "none"
    pub scroll_method: String,
    /// Acceleration speed
    pub accel_speed: f64,
}

impl Default for TouchpadConfig {
    fn default() -> Self {
        Self {
            tap_to_click: true,
            two_finger_right_click: true,
            natural_scroll: true,
            disable_while_typing: true,
            scroll_method: "two_finger".to_string(),
            accel_speed: 0.0,
        }
    }
}

/// Touch screen configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TouchConfig {
    /// Enable touch input
    pub enabled: bool,
    /// Map to specific output
    pub output: Option<String>,
}

impl Default for TouchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            output: None,
        }
    }
}

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    /// Gap between windows in pixels
    pub window_gap: u32,
    /// Border width in pixels
    pub border_width: u32,
    /// Active window border color (RGBA hex)
    pub border_color_active: String,
    /// Inactive window border color (RGBA hex)
    pub border_color_inactive: String,
    /// Background color (RGBA hex)
    pub background_color: String,
    /// Enable window shadows
    pub shadows: bool,
    /// Enable window animations
    pub animations: bool,
    /// Animation duration in ms
    pub animation_duration: u32,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            window_gap: 8,
            border_width: 2,
            border_color_active: "#0078D4FF".to_string(),
            border_color_inactive: "#808080FF".to_string(),
            background_color: "#1E1E1EFF".to_string(),
            shadows: true,
            animations: true,
            animation_duration: 200,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    /// Maximum render rate (0 = unlimited)
    pub max_render_rate: u32,
    /// Enable direct scanout when possible
    pub direct_scanout: bool,
    /// Use damage tracking for rendering
    pub damage_tracking: bool,
    /// Number of render threads (0 = auto)
    pub render_threads: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_render_rate: 0,
            direct_scanout: true,
            damage_tracking: true,
            render_threads: 0,
        }
    }
}

impl CompositorConfig {
    /// Load configuration from the default location
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            tracing::info!("No config file found, using defaults");
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        tracing::info!("Loaded configuration from {:?}", path);
        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        self.save_to_file(&config_path)
    }

    /// Save configuration to a specific file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        tracing::info!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Get the default configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;

        Ok(config_dir.join("winux").join("compositor.toml"))
    }

    /// Reload configuration from disk
    pub fn reload(&mut self) -> Result<()> {
        let new_config = Self::load()?;
        *self = new_config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CompositorConfig::default();
        assert!(config.general.vsync);
        assert_eq!(config.general.backend, "auto");
        assert_eq!(config.outputs.scale, 1.0);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = CompositorConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: CompositorConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.general.vsync, parsed.general.vsync);
    }
}
