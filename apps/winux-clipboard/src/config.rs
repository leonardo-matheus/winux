//! Configuration for Winux Clipboard Manager

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum number of items to keep in history
    #[serde(default = "default_max_history")]
    pub max_history: usize,

    /// Whether to store images in history
    #[serde(default = "default_true")]
    pub store_images: bool,

    /// Whether to store files in history
    #[serde(default = "default_true")]
    pub store_files: bool,

    /// Whether to store HTML content
    #[serde(default = "default_true")]
    pub store_html: bool,

    /// Maximum size of text content to store (in bytes)
    #[serde(default = "default_max_text_size")]
    pub max_text_size: usize,

    /// Maximum size of image to store (in bytes)
    #[serde(default = "default_max_image_size")]
    pub max_image_size: usize,

    /// Clear history on logout
    #[serde(default)]
    pub clear_on_logout: bool,

    /// Encrypt stored history
    #[serde(default = "default_true")]
    pub encrypt_history: bool,

    /// Detect and skip password fields
    #[serde(default = "default_true")]
    pub skip_passwords: bool,

    /// List of applications to ignore clipboard from
    #[serde(default)]
    pub ignored_apps: Vec<String>,

    /// Custom keyboard shortcut (default: Super+V)
    #[serde(default = "default_shortcut")]
    pub shortcut: String,

    /// Show notifications on copy
    #[serde(default)]
    pub show_notifications: bool,

    /// Synchronize with other devices (future feature)
    #[serde(default)]
    pub sync_enabled: bool,

    /// Theme preference (system, light, dark)
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_max_history() -> usize {
    100
}

fn default_true() -> bool {
    true
}

fn default_max_text_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_max_image_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_shortcut() -> String {
    "Super+V".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_history: default_max_history(),
            store_images: true,
            store_files: true,
            store_html: true,
            max_text_size: default_max_text_size(),
            max_image_size: default_max_image_size(),
            clear_on_logout: false,
            encrypt_history: true,
            skip_passwords: true,
            ignored_apps: vec![
                "keepassxc".to_string(),
                "1password".to_string(),
                "bitwarden".to_string(),
                "lastpass".to_string(),
            ],
            shortcut: default_shortcut(),
            show_notifications: false,
            sync_enabled: false,
            theme: default_theme(),
        }
    }
}

impl Config {
    /// Get the configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("winux-clipboard");

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }

        Ok(config_dir)
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("winux-clipboard");

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)
                .context("Failed to create data directory")?;
        }

        Ok(data_dir)
    }

    /// Get the history file path
    pub fn history_path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("history.json"))
    }

    /// Get the images directory path
    pub fn images_dir() -> Result<PathBuf> {
        let images_dir = Self::data_dir()?.join("images");

        if !images_dir.exists() {
            std::fs::create_dir_all(&images_dir)
                .context("Failed to create images directory")?;
        }

        Ok(images_dir)
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .context("Failed to read config file")?;
            toml::from_str(&content).context("Failed to parse config file")
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(&path, content)
            .context("Failed to write config file")?;
        Ok(())
    }
}

/// Security-related utilities
pub mod security {
    use super::*;

    /// Patterns that indicate a password field
    const PASSWORD_PATTERNS: &[&str] = &[
        "password",
        "passwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "private_key",
        "privatekey",
        "credential",
        "auth",
    ];

    /// Check if content looks like it might be from a password field
    pub fn looks_like_password(content: &str) -> bool {
        // Very short content that's alphanumeric might be a password
        if content.len() >= 8 && content.len() <= 128 {
            let has_upper = content.chars().any(|c| c.is_ascii_uppercase());
            let has_lower = content.chars().any(|c| c.is_ascii_lowercase());
            let has_digit = content.chars().any(|c| c.is_ascii_digit());
            let has_special = content.chars().any(|c| !c.is_alphanumeric());

            // If it has characteristics of a strong password and no spaces
            if !content.contains(' ')
                && has_upper
                && has_lower
                && has_digit
                && has_special
            {
                return true;
            }
        }

        false
    }

    /// Check if an application name suggests it's a password manager
    pub fn is_password_manager(app_name: &str) -> bool {
        let app_lower = app_name.to_lowercase();
        PASSWORD_PATTERNS.iter().any(|p| app_lower.contains(p))
            || app_lower.contains("keepass")
            || app_lower.contains("bitwarden")
            || app_lower.contains("1password")
            || app_lower.contains("lastpass")
            || app_lower.contains("dashlane")
    }
}
