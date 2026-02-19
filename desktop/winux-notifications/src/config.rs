//! Notification daemon configuration
//!
//! Handles loading, saving, and managing notification settings.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Main notification daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Display settings
    pub display: DisplayConfig,

    /// Sound settings
    pub sound: SoundConfig,

    /// Do Not Disturb settings
    pub dnd: DndConfig,

    /// Per-application settings
    pub apps: AppsConfig,
}

/// Display settings for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Position on screen
    pub position: NotificationPosition,

    /// Default timeout in milliseconds (0 = never expire)
    pub default_timeout: u32,

    /// Maximum number of visible popups
    pub max_visible: u32,

    /// Popup width in pixels
    pub popup_width: u32,

    /// Gap between popups in pixels
    pub popup_gap: u32,

    /// Margin from screen edges in pixels
    pub screen_margin: u32,

    /// Enable animations
    pub animations_enabled: bool,

    /// Animation duration in milliseconds
    pub animation_duration: u32,

    /// Show notification icons
    pub show_icons: bool,

    /// Icon size in pixels
    pub icon_size: u32,

    /// Maximum body text lines
    pub max_body_lines: u32,

    /// Show action buttons
    pub show_actions: bool,

    /// Show progress bars
    pub show_progress: bool,

    /// Popup opacity (0.0 - 1.0)
    pub opacity: f64,

    /// Enable blur effect behind popups
    pub blur_enabled: bool,
}

/// Notification position on screen
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Default for NotificationPosition {
    fn default() -> Self {
        Self::TopRight
    }
}

/// Sound settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    /// Enable notification sounds
    pub enabled: bool,

    /// Sound theme name
    pub theme: String,

    /// Sound file for low urgency
    pub sound_low: Option<String>,

    /// Sound file for normal urgency
    pub sound_normal: Option<String>,

    /// Sound file for critical urgency
    pub sound_critical: Option<String>,

    /// Master volume (0.0 - 1.0)
    pub volume: f64,
}

/// Do Not Disturb settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DndConfig {
    /// DND currently enabled
    pub enabled: bool,

    /// Allow critical notifications during DND
    pub allow_critical: bool,

    /// Scheduled DND start time (HH:MM format)
    pub schedule_start: Option<String>,

    /// Scheduled DND end time (HH:MM format)
    pub schedule_end: Option<String>,

    /// Enable scheduled DND
    pub schedule_enabled: bool,
}

/// Per-application notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsConfig {
    /// Applications with notifications disabled
    pub disabled_apps: Vec<String>,

    /// Applications with custom sound disabled
    pub silent_apps: Vec<String>,

    /// Applications always allowed during DND
    pub priority_apps: Vec<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            sound: SoundConfig::default(),
            dnd: DndConfig::default(),
            apps: AppsConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            position: NotificationPosition::TopRight,
            default_timeout: 5000,
            max_visible: 5,
            popup_width: 380,
            popup_gap: 8,
            screen_margin: 12,
            animations_enabled: true,
            animation_duration: 200,
            show_icons: true,
            icon_size: 48,
            max_body_lines: 4,
            show_actions: true,
            show_progress: true,
            opacity: 0.95,
            blur_enabled: true,
        }
    }
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "freedesktop".to_string(),
            sound_low: None,
            sound_normal: Some("message".to_string()),
            sound_critical: Some("dialog-warning".to_string()),
            volume: 0.8,
        }
    }
}

impl Default for DndConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_critical: true,
            schedule_start: Some("22:00".to_string()),
            schedule_end: Some("07:00".to_string()),
            schedule_enabled: false,
        }
    }
}

impl Default for AppsConfig {
    fn default() -> Self {
        Self {
            disabled_apps: Vec::new(),
            silent_apps: Vec::new(),
            priority_apps: vec![
                "org.gnome.Evolution".to_string(),
                "thunderbird".to_string(),
            ],
        }
    }
}

impl NotificationConfig {
    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("org", "winux", "notifications")
            .context("Failed to determine config directory")?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).context("Failed to create config directory")?;

        Ok(config_dir.join("config.toml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            info!("Loading configuration from {:?}", config_path);
            let content = fs::read_to_string(&config_path)
                .context("Failed to read configuration file")?;

            let config: NotificationConfig = toml::from_str(&content)
                .context("Failed to parse configuration file")?;

            debug!("Configuration loaded successfully");
            Ok(config)
        } else {
            info!("No configuration file found, using defaults");
            let config = NotificationConfig::default();

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

    /// Check if DND is currently active (including schedule)
    pub fn is_dnd_active(&self) -> bool {
        if self.dnd.enabled {
            return true;
        }

        if !self.dnd.schedule_enabled {
            return false;
        }

        // Check scheduled DND
        if let (Some(start), Some(end)) = (&self.dnd.schedule_start, &self.dnd.schedule_end) {
            if let Ok(now) = chrono::Local::now().format("%H:%M").to_string().parse::<String>() {
                let now = now.as_str();
                if start <= end {
                    // Normal range (e.g., 09:00 - 17:00)
                    return now >= start && now < end;
                } else {
                    // Overnight range (e.g., 22:00 - 07:00)
                    return now >= start || now < end;
                }
            }
        }

        false
    }

    /// Check if an app should show notifications
    pub fn should_notify_app(&self, app_name: &str) -> bool {
        !self.apps.disabled_apps.iter().any(|a| a == app_name)
    }

    /// Check if an app should play sounds
    pub fn should_play_sound_for_app(&self, app_name: &str) -> bool {
        self.sound.enabled && !self.apps.silent_apps.iter().any(|a| a == app_name)
    }

    /// Check if an app is a priority app (bypass DND)
    pub fn is_priority_app(&self, app_name: &str) -> bool {
        self.apps.priority_apps.iter().any(|a| a == app_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NotificationConfig::default();
        assert_eq!(config.display.position, NotificationPosition::TopRight);
        assert_eq!(config.display.default_timeout, 5000);
        assert!(config.sound.enabled);
        assert!(!config.dnd.enabled);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = NotificationConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: NotificationConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.display.position, deserialized.display.position);
        assert_eq!(config.display.default_timeout, deserialized.display.default_timeout);
    }

    #[test]
    fn test_app_settings() {
        let mut config = NotificationConfig::default();
        config.apps.disabled_apps.push("test-app".to_string());
        config.apps.silent_apps.push("noisy-app".to_string());

        assert!(!config.should_notify_app("test-app"));
        assert!(config.should_notify_app("other-app"));
        assert!(!config.should_play_sound_for_app("noisy-app"));
        assert!(config.should_play_sound_for_app("other-app"));
    }
}
