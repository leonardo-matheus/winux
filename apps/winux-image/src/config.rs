//! Configuration management for Winux Image

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Zoom mode for image display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoomMode {
    /// Fit image within window (default)
    Fit,
    /// Fill window (may crop)
    Fill,
    /// Original size (100%)
    Original,
    /// Custom zoom level
    Custom(u32), // percentage
}

impl Default for ZoomMode {
    fn default() -> Self {
        Self::Fit
    }
}

/// Slideshow transition effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionEffect {
    None,
    Fade,
    Slide,
    Zoom,
}

impl Default for TransitionEffect {
    fn default() -> Self {
        Self::Fade
    }
}

/// Background color mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundMode {
    /// Follow system theme
    Auto,
    /// Dark background
    Dark,
    /// Light background
    Light,
    /// Checkerboard for transparency
    Checkerboard,
    /// Custom color
    Custom,
}

impl Default for BackgroundMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// Default zoom mode
    pub zoom_mode: ZoomMode,
    /// Remember zoom level per image
    pub remember_zoom: bool,
    /// Show thumbnail strip
    pub show_thumbnails: bool,
    /// Thumbnail size in pixels
    pub thumbnail_size: u32,
    /// Show metadata panel
    pub show_metadata: bool,
    /// Background mode
    pub background_mode: BackgroundMode,
    /// Custom background color (hex)
    pub background_color: String,
    /// Slideshow interval in seconds
    pub slideshow_interval: u32,
    /// Slideshow transition effect
    pub slideshow_transition: TransitionEffect,
    /// Loop slideshow
    pub slideshow_loop: bool,
    /// Shuffle slideshow
    pub slideshow_shuffle: bool,
    /// Smooth zoom animation
    pub smooth_zoom: bool,
    /// Zoom step percentage
    pub zoom_step: u32,
    /// Maximum zoom percentage
    pub max_zoom: u32,
    /// Minimum zoom percentage
    pub min_zoom: u32,
    /// Window width
    pub window_width: i32,
    /// Window height
    pub window_height: i32,
    /// Window maximized
    pub window_maximized: bool,
    /// Recent files list (max 20)
    pub recent_files: Vec<PathBuf>,
    /// Auto-rotate based on EXIF
    pub auto_rotate: bool,
    /// Use GPU acceleration
    pub gpu_acceleration: bool,
    /// Preload adjacent images
    pub preload_adjacent: bool,
    /// Number of images to preload
    pub preload_count: u32,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            zoom_mode: ZoomMode::Fit,
            remember_zoom: false,
            show_thumbnails: true,
            thumbnail_size: 80,
            show_metadata: false,
            background_mode: BackgroundMode::Auto,
            background_color: "#1e1e1e".to_string(),
            slideshow_interval: 5,
            slideshow_transition: TransitionEffect::Fade,
            slideshow_loop: true,
            slideshow_shuffle: false,
            smooth_zoom: true,
            zoom_step: 10,
            max_zoom: 1000,
            min_zoom: 5,
            window_width: 1200,
            window_height: 800,
            window_maximized: false,
            recent_files: Vec::new(),
            auto_rotate: true,
            gpu_acceleration: true,
            preload_adjacent: true,
            preload_count: 2,
        }
    }
}

impl ImageConfig {
    /// Load configuration from disk
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// Save configuration to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-image")
            .join("config.toml")
    }

    /// Add a file to recent files
    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        // Add to front
        self.recent_files.insert(0, path);
        // Keep only last 20
        self.recent_files.truncate(20);
    }

    /// Clear recent files
    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }

    /// Get zoom level as float (1.0 = 100%)
    pub fn zoom_level(&self) -> f64 {
        match self.zoom_mode {
            ZoomMode::Fit => 1.0,  // Will be calculated
            ZoomMode::Fill => 1.0, // Will be calculated
            ZoomMode::Original => 1.0,
            ZoomMode::Custom(pct) => pct as f64 / 100.0,
        }
    }
}
