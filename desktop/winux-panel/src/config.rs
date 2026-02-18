//! Panel configuration module
//!
//! Handles loading, saving, and managing panel configuration settings.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Main panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// General panel settings
    pub general: GeneralConfig,

    /// Taskbar settings
    pub taskbar: TaskbarConfig,

    /// Start menu settings
    pub start_menu: StartMenuConfig,

    /// System tray settings
    pub system_tray: SystemTrayConfig,

    /// Clock widget settings
    pub clock: ClockConfig,
}

/// General panel settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Panel height in pixels
    pub height: u32,

    /// Panel position (top or bottom)
    pub position: PanelPosition,

    /// Panel opacity (0.0 - 1.0)
    pub opacity: f64,

    /// Enable blur effect behind panel
    pub blur_enabled: bool,

    /// Auto-hide panel when not in use
    pub auto_hide: bool,

    /// Auto-hide delay in milliseconds
    pub auto_hide_delay: u32,
}

/// Panel position options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PanelPosition {
    Top,
    Bottom,
}

/// Taskbar-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskbarConfig {
    /// Show window labels or just icons
    pub show_labels: bool,

    /// Maximum label width in characters
    pub max_label_width: u32,

    /// Show window previews on hover
    pub show_previews: bool,

    /// Preview delay in milliseconds
    pub preview_delay: u32,

    /// Group windows by application
    pub group_windows: bool,

    /// Pinned applications (desktop file IDs)
    pub pinned_apps: Vec<String>,

    /// Show running indicator
    pub show_running_indicator: bool,
}

/// Start menu settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMenuConfig {
    /// Number of columns in pinned apps grid
    pub pinned_columns: u32,

    /// Number of rows in pinned apps grid
    pub pinned_rows: u32,

    /// Pinned applications in start menu
    pub pinned_apps: Vec<String>,

    /// Show recent files section
    pub show_recent_files: bool,

    /// Number of recent files to show
    pub recent_files_count: u32,

    /// Show power options
    pub show_power_options: bool,

    /// Show user profile section
    pub show_user_profile: bool,

    /// Start menu width
    pub width: u32,

    /// Start menu height
    pub height: u32,
}

/// System tray settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTrayConfig {
    /// Enable system tray
    pub enabled: bool,

    /// Icon size in pixels
    pub icon_size: u32,

    /// Icon spacing in pixels
    pub icon_spacing: u32,

    /// Hidden tray icons (app IDs)
    pub hidden_icons: Vec<String>,

    /// Show battery indicator
    pub show_battery: bool,

    /// Show network indicator
    pub show_network: bool,

    /// Show volume indicator
    pub show_volume: bool,

    /// Show notifications indicator
    pub show_notifications: bool,
}

/// Clock widget settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    /// Time format (12h or 24h)
    pub format_24h: bool,

    /// Show seconds
    pub show_seconds: bool,

    /// Show date below time
    pub show_date: bool,

    /// Date format string
    pub date_format: String,

    /// Show calendar popup on click
    pub show_calendar: bool,

    /// Show week numbers in calendar
    pub show_week_numbers: bool,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            taskbar: TaskbarConfig::default(),
            start_menu: StartMenuConfig::default(),
            system_tray: SystemTrayConfig::default(),
            clock: ClockConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            height: 48,
            position: PanelPosition::Bottom,
            opacity: 0.85,
            blur_enabled: true,
            auto_hide: false,
            auto_hide_delay: 1000,
        }
    }
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        Self {
            show_labels: false,
            max_label_width: 20,
            show_previews: true,
            preview_delay: 500,
            group_windows: true,
            pinned_apps: vec![
                "org.winux.Files".to_string(),
                "org.winux.Terminal".to_string(),
                "firefox".to_string(),
                "org.winux.Settings".to_string(),
            ],
            show_running_indicator: true,
        }
    }
}

impl Default for StartMenuConfig {
    fn default() -> Self {
        Self {
            pinned_columns: 6,
            pinned_rows: 4,
            pinned_apps: vec![
                "org.winux.Files".to_string(),
                "org.winux.Terminal".to_string(),
                "firefox".to_string(),
                "org.winux.Settings".to_string(),
                "org.winux.Store".to_string(),
                "org.winux.Edit".to_string(),
                "org.gnome.Calculator".to_string(),
                "org.gnome.Calendar".to_string(),
            ],
            show_recent_files: true,
            recent_files_count: 10,
            show_power_options: true,
            show_user_profile: true,
            width: 600,
            height: 700,
        }
    }
}

impl Default for SystemTrayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            icon_size: 20,
            icon_spacing: 4,
            hidden_icons: vec![],
            show_battery: true,
            show_network: true,
            show_volume: true,
            show_notifications: true,
        }
    }
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format_24h: true,
            show_seconds: false,
            show_date: true,
            date_format: "%a, %b %d".to_string(),
            show_calendar: true,
            show_week_numbers: false,
        }
    }
}

impl PanelConfig {
    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("org", "winux", "panel")
            .context("Failed to determine config directory")?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).context("Failed to create config directory")?;

        Ok(config_dir.join("panel.toml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            info!("Loading configuration from {:?}", config_path);
            let content = fs::read_to_string(&config_path)
                .context("Failed to read configuration file")?;

            let config: PanelConfig = toml::from_str(&content)
                .context("Failed to parse configuration file")?;

            debug!("Configuration loaded successfully");
            Ok(config)
        } else {
            info!("No configuration file found, using defaults");
            let config = PanelConfig::default();

            // Save default configuration
            if let Err(e) = config.save() {
                warn!("Failed to save default configuration: {}", e);
            }

            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        fs::write(&config_path, content)
            .context("Failed to write configuration file")?;

        info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Reload configuration from file
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
        let config = PanelConfig::default();
        assert_eq!(config.general.height, 48);
        assert_eq!(config.general.position, PanelPosition::Bottom);
        assert!(!config.taskbar.pinned_apps.is_empty());
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = PanelConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: PanelConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.general.height, deserialized.general.height);
        assert_eq!(config.general.position, deserialized.general.position);
    }
}
