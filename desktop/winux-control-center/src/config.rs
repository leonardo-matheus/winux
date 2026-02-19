//! Configuration management for Winux Control Center
//!
//! Handles loading and saving user preferences for quick settings.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Window appearance settings
    pub appearance: AppearanceConfig,
    /// Quick toggle states
    pub toggles: ToggleStates,
    /// Audio settings
    pub audio: AudioConfig,
    /// Display settings
    pub display: DisplayConfig,
    /// Power settings
    pub power: PowerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig::default(),
            toggles: ToggleStates::default(),
            audio: AudioConfig::default(),
            display: DisplayConfig::default(),
            power: PowerConfig::default(),
        }
    }
}

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Enable blur background effect
    pub blur_enabled: bool,
    /// Blur intensity (0.0 - 1.0)
    pub blur_intensity: f64,
    /// Animation duration in milliseconds
    pub animation_duration: u32,
    /// Control center width
    pub width: i32,
    /// Control center height (auto if 0)
    pub height: i32,
    /// Corner radius for main window
    pub corner_radius: i32,
    /// Position from screen edge
    pub margin_top: i32,
    pub margin_right: i32,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            blur_enabled: true,
            blur_intensity: 0.85,
            animation_duration: 250,
            width: 380,
            height: 0,
            corner_radius: 24,
            margin_top: 8,
            margin_right: 8,
        }
    }
}

/// Quick toggle states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleStates {
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub airplane_mode: bool,
    pub do_not_disturb: bool,
    pub night_light: bool,
    pub screen_recording: bool,
    pub screen_mirroring: bool,
}

impl Default for ToggleStates {
    fn default() -> Self {
        Self {
            wifi_enabled: true,
            bluetooth_enabled: true,
            airplane_mode: false,
            do_not_disturb: false,
            night_light: false,
            screen_recording: false,
            screen_mirroring: false,
        }
    }
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Volume level (0-100)
    pub volume: u32,
    /// Muted state
    pub muted: bool,
    /// Current output device
    pub output_device: String,
    /// Current input device
    pub input_device: String,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            volume: 50,
            muted: false,
            output_device: String::from("default"),
            input_device: String::from("default"),
        }
    }
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Brightness level (0-100)
    pub brightness: u32,
    /// Night light color temperature (1000-10000K)
    pub night_light_temperature: u32,
    /// Night light schedule enabled
    pub night_light_schedule: bool,
    /// Night light start time (HH:MM)
    pub night_light_start: String,
    /// Night light end time (HH:MM)
    pub night_light_end: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            brightness: 80,
            night_light_temperature: 4500,
            night_light_schedule: false,
            night_light_start: String::from("22:00"),
            night_light_end: String::from("06:00"),
        }
    }
}

/// Power configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    /// Power mode: "balanced", "power_saver", "performance"
    pub power_mode: String,
    /// Battery saver threshold (0-100)
    pub battery_saver_threshold: u32,
    /// Auto battery saver
    pub auto_battery_saver: bool,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            power_mode: String::from("balanced"),
            battery_saver_threshold: 20,
            auto_battery_saver: true,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(config) => {
                            info!("Configuration loaded from {:?}", config_path);
                            return config;
                        }
                        Err(e) => {
                            warn!("Failed to parse config file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read config file: {}", e);
                }
            }
        }

        info!("Using default configuration");
        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;

        info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Get the configuration file path
    fn config_path() -> PathBuf {
        let config_dir = directories::ProjectDirs::from("org", "winux", "control-center")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".config/winux/control-center")
            });

        config_dir.join("config.toml")
    }
}

/// Keyboard shortcuts configuration
#[derive(Debug, Clone)]
pub struct KeyboardShortcuts {
    /// Toggle control center (default: Super+A)
    pub toggle: String,
    /// Close control center (default: Escape)
    pub close: String,
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self {
            toggle: String::from("<Super>a"),
            close: String::from("Escape"),
        }
    }
}
