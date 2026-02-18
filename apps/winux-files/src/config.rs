//! Configuration for Winux Files

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::app::SortBy;
use crate::file_view::ViewMode;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default view mode
    pub view_mode: ViewModeConfig,
    /// Show hidden files by default
    pub show_hidden: bool,
    /// Default sort order
    pub sort_by: SortByConfig,
    /// Sort descending
    pub sort_descending: bool,
    /// Single click to activate
    pub single_click: bool,
    /// Show file extensions
    pub show_extensions: bool,
    /// Thumbnail size in grid view
    pub thumbnail_size: ThumbnailSize,
    /// Confirm before delete
    pub confirm_delete: bool,
    /// Use trash instead of permanent delete
    pub use_trash: bool,
    /// Custom bookmarks
    pub bookmarks: Vec<BookmarkConfig>,
    /// Window state
    pub window: WindowConfig,
}

/// View mode configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewModeConfig {
    #[default]
    Grid,
    List,
}

impl From<ViewModeConfig> for ViewMode {
    fn from(config: ViewModeConfig) -> Self {
        match config {
            ViewModeConfig::Grid => ViewMode::Grid,
            ViewModeConfig::List => ViewMode::List,
        }
    }
}

/// Sort by configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortByConfig {
    #[default]
    Name,
    Size,
    Modified,
    Type,
}

impl From<SortByConfig> for SortBy {
    fn from(config: SortByConfig) -> Self {
        match config {
            SortByConfig::Name => SortBy::Name,
            SortByConfig::Size => SortBy::Size,
            SortByConfig::Modified => SortBy::Modified,
            SortByConfig::Type => SortBy::Type,
        }
    }
}

/// Thumbnail size options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThumbnailSize {
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}

impl ThumbnailSize {
    pub fn pixels(&self) -> i32 {
        match self {
            ThumbnailSize::Small => 32,
            ThumbnailSize::Medium => 48,
            ThumbnailSize::Large => 64,
            ThumbnailSize::ExtraLarge => 96,
        }
    }
}

/// Bookmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkConfig {
    pub name: String,
    pub path: PathBuf,
    pub icon: Option<String>,
}

/// Window state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
    pub sidebar_width: i32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            maximized: false,
            sidebar_width: 200,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            view_mode: ViewModeConfig::Grid,
            show_hidden: false,
            sort_by: SortByConfig::Name,
            sort_descending: false,
            single_click: false,
            show_extensions: true,
            thumbnail_size: ThumbnailSize::Medium,
            confirm_delete: true,
            use_trash: true,
            bookmarks: Vec::new(),
            window: WindowConfig::default(),
        }
    }
}

impl Config {
    /// Get the configuration file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-files")
            .join("config.json")
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
        let config: Config = serde_json::from_str(&content)?;
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

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        info!("Config saved");
        Ok(())
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, name: String, path: PathBuf) {
        self.bookmarks.push(BookmarkConfig {
            name,
            path,
            icon: None,
        });
    }

    /// Remove a bookmark by index
    pub fn remove_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.show_hidden);
        assert!(config.use_trash);
        assert!(config.confirm_delete);
    }

    #[test]
    fn test_thumbnail_size() {
        assert_eq!(ThumbnailSize::Small.pixels(), 32);
        assert_eq!(ThumbnailSize::Medium.pixels(), 48);
        assert_eq!(ThumbnailSize::Large.pixels(), 64);
        assert_eq!(ThumbnailSize::ExtraLarge.pixels(), 96);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.show_hidden, config.show_hidden);
    }
}
