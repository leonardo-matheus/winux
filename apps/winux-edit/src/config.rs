//! Configuration management for Winux Edit
//!
//! Handles loading, saving, and accessing editor preferences.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Editor settings
    #[serde(default)]
    pub editor: EditorSettings,

    /// Appearance settings
    #[serde(default)]
    pub appearance: AppearanceSettings,

    /// File settings
    #[serde(default)]
    pub files: FileSettings,

    /// Keyboard shortcuts
    #[serde(default)]
    pub keybindings: KeybindingSettings,
}

/// Editor behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    /// Number of spaces per tab
    #[serde(default = "default_tab_width")]
    pub tab_width: u32,

    /// Use spaces instead of tabs
    #[serde(default = "default_true")]
    pub insert_spaces: bool,

    /// Enable auto-indent
    #[serde(default = "default_true")]
    pub auto_indent: bool,

    /// Enable smart backspace
    #[serde(default = "default_true")]
    pub smart_backspace: bool,

    /// Enable word wrap
    #[serde(default)]
    pub word_wrap: bool,

    /// Show line numbers
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,

    /// Highlight current line
    #[serde(default = "default_true")]
    pub highlight_current_line: bool,

    /// Highlight matching brackets
    #[serde(default = "default_true")]
    pub highlight_matching_brackets: bool,

    /// Show whitespace characters
    #[serde(default)]
    pub show_whitespace: bool,

    /// Enable minimap
    #[serde(default)]
    pub show_minimap: bool,

    /// Right margin position (0 to disable)
    #[serde(default = "default_margin")]
    pub right_margin: u32,

    /// Font family
    #[serde(default = "default_font")]
    pub font_family: String,

    /// Font size in points
    #[serde(default = "default_font_size")]
    pub font_size: u32,
}

/// Appearance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// Color scheme ID
    #[serde(default = "default_scheme")]
    pub color_scheme: String,

    /// Use dark mode
    #[serde(default = "default_true")]
    pub dark_mode: bool,

    /// UI scale factor
    #[serde(default = "default_scale")]
    pub ui_scale: f64,
}

/// File handling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSettings {
    /// Auto-save interval in seconds (0 to disable)
    #[serde(default)]
    pub auto_save_interval: u32,

    /// Create backup before saving
    #[serde(default)]
    pub create_backup: bool,

    /// Default line ending
    #[serde(default = "default_line_ending")]
    pub default_line_ending: String,

    /// Default encoding
    #[serde(default = "default_encoding")]
    pub default_encoding: String,

    /// Trim trailing whitespace on save
    #[serde(default = "default_true")]
    pub trim_trailing_whitespace: bool,

    /// Ensure final newline
    #[serde(default = "default_true")]
    pub ensure_final_newline: bool,

    /// Recently opened files
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,

    /// Maximum recent files to remember
    #[serde(default = "default_recent_count")]
    pub max_recent_files: usize,
}

/// Keybinding settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeybindingSettings {
    /// Custom keybindings (action -> key combo)
    #[serde(default)]
    pub custom: std::collections::HashMap<String, String>,
}

impl EditorConfig {
    /// Load configuration from file
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(config) => {
                            info!("Loaded configuration from {:?}", config_path);
                            return config;
                        }
                        Err(e) => {
                            warn!("Failed to parse config: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read config file: {}", e);
                }
            }
        }

        debug!("Using default configuration");
        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), std::io::Error> {
        let config_path = Self::config_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        std::fs::write(&config_path, contents)?;
        info!("Saved configuration to {:?}", config_path);

        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .map(|p| p.join("winux-edit").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.config/winux-edit/config.toml"))
    }

    /// Add a file to recent files list
    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists
        self.files.recent_files.retain(|p| p != &path);

        // Add at the beginning
        self.files.recent_files.insert(0, path);

        // Trim to max
        self.files.recent_files.truncate(self.files.max_recent_files);
    }

    /// Clear recent files
    pub fn clear_recent_files(&mut self) {
        self.files.recent_files.clear();
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            editor: EditorSettings::default(),
            appearance: AppearanceSettings::default(),
            files: FileSettings::default(),
            keybindings: KeybindingSettings::default(),
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            tab_width: default_tab_width(),
            insert_spaces: true,
            auto_indent: true,
            smart_backspace: true,
            word_wrap: false,
            show_line_numbers: true,
            highlight_current_line: true,
            highlight_matching_brackets: true,
            show_whitespace: false,
            show_minimap: false,
            right_margin: default_margin(),
            font_family: default_font(),
            font_size: default_font_size(),
        }
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            color_scheme: default_scheme(),
            dark_mode: true,
            ui_scale: default_scale(),
        }
    }
}

impl Default for FileSettings {
    fn default() -> Self {
        Self {
            auto_save_interval: 0,
            create_backup: false,
            default_line_ending: default_line_ending(),
            default_encoding: default_encoding(),
            trim_trailing_whitespace: true,
            ensure_final_newline: true,
            recent_files: Vec::new(),
            max_recent_files: default_recent_count(),
        }
    }
}

// Default value functions for serde
fn default_tab_width() -> u32 {
    4
}

fn default_true() -> bool {
    true
}

fn default_margin() -> u32 {
    80
}

fn default_font() -> String {
    "Monospace".to_string()
}

fn default_font_size() -> u32 {
    12
}

fn default_scheme() -> String {
    "Adwaita-dark".to_string()
}

fn default_scale() -> f64 {
    1.0
}

fn default_line_ending() -> String {
    "unix".to_string()
}

fn default_encoding() -> String {
    "UTF-8".to_string()
}

fn default_recent_count() -> usize {
    20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EditorConfig::default();
        assert_eq!(config.editor.tab_width, 4);
        assert!(config.editor.insert_spaces);
        assert!(config.appearance.dark_mode);
    }

    #[test]
    fn test_config_serialization() {
        let config = EditorConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: EditorConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.editor.tab_width, config.editor.tab_width);
    }
}
