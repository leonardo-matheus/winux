//! Configuration for Winux Terminal

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Terminal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: u32,
    /// Theme name
    pub theme: String,
    /// Enable cursor blinking
    pub cursor_blink: bool,
    /// Cursor shape: "block", "ibeam", or "underline"
    pub cursor_shape: String,
    /// Number of scrollback lines
    pub scrollback_lines: u32,
    /// Scroll on output
    pub scroll_on_output: bool,
    /// Scroll on keystroke
    pub scroll_on_keystroke: bool,
    /// Enable audible bell
    pub audible_bell: bool,
    /// Enable visual bell
    pub visual_bell: bool,
    /// Enable bold text
    pub allow_bold: bool,
    /// Cell width spacing
    pub cell_width_scale: f64,
    /// Cell height spacing
    pub cell_height_scale: f64,
    /// Shell to use
    pub shell: Option<String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
    /// Environment variables
    pub environment: Vec<(String, String)>,
    /// Keyboard shortcuts
    pub shortcuts: ShortcutsConfig,
    /// Window configuration
    pub window: WindowConfig,
}

/// Keyboard shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    /// New tab shortcut
    pub new_tab: String,
    /// Close tab shortcut
    pub close_tab: String,
    /// Copy shortcut
    pub copy: String,
    /// Paste shortcut
    pub paste: String,
    /// Search shortcut
    pub search: String,
    /// Zoom in shortcut
    pub zoom_in: String,
    /// Zoom out shortcut
    pub zoom_out: String,
    /// Reset zoom shortcut
    pub zoom_reset: String,
    /// Next tab shortcut
    pub next_tab: String,
    /// Previous tab shortcut
    pub prev_tab: String,
    /// Split horizontal shortcut
    pub split_horizontal: String,
    /// Split vertical shortcut
    pub split_vertical: String,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Default width
    pub width: i32,
    /// Default height
    pub height: i32,
    /// Start maximized
    pub maximized: bool,
    /// Window opacity (0.0 - 1.0)
    pub opacity: f64,
    /// Enable CSD (Client Side Decorations)
    pub csd: bool,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            new_tab: "Ctrl+Shift+T".to_string(),
            close_tab: "Ctrl+Shift+W".to_string(),
            copy: "Ctrl+Shift+C".to_string(),
            paste: "Ctrl+Shift+V".to_string(),
            search: "Ctrl+Shift+F".to_string(),
            zoom_in: "Ctrl+Plus".to_string(),
            zoom_out: "Ctrl+Minus".to_string(),
            zoom_reset: "Ctrl+0".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            prev_tab: "Ctrl+Shift+Tab".to_string(),
            split_horizontal: "Ctrl+Shift+H".to_string(),
            split_vertical: "Ctrl+Shift+V".to_string(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 900,
            height: 600,
            maximized: false,
            opacity: 1.0,
            csd: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 12,
            theme: "winux-dark".to_string(),
            cursor_blink: true,
            cursor_shape: "block".to_string(),
            scrollback_lines: 10000,
            scroll_on_output: false,
            scroll_on_keystroke: true,
            audible_bell: false,
            visual_bell: false,
            allow_bold: true,
            cell_width_scale: 1.0,
            cell_height_scale: 1.0,
            shell: None,
            working_directory: None,
            environment: Vec::new(),
            shortcuts: ShortcutsConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

impl Config {
    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-terminal")
            .join("config.toml")
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        debug!("Loading config from {:?}", path);

        if !path.exists() {
            info!("Config file not found, using defaults");
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        debug!("Saving config to {:?}", path);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        info!("Config saved");
        Ok(())
    }

    /// Get the shell to use
    pub fn get_shell(&self) -> String {
        self.shell.clone().unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        })
    }

    /// Get working directory
    pub fn get_working_directory(&self) -> PathBuf {
        self.working_directory
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.font_size, 12);
        assert_eq!(config.scrollback_lines, 10000);
        assert!(!config.audible_bell);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.font_size, config.font_size);
    }

    #[test]
    fn test_get_shell() {
        let config = Config::default();
        let shell = config.get_shell();
        assert!(!shell.is_empty());
    }
}
