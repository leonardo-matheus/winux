//! Configuration management for Winux Launcher

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,

    /// Search settings
    pub search: SearchConfig,

    /// UI settings
    pub ui: UiConfig,

    /// Plugin settings
    pub plugins: PluginConfig,

    /// Keyboard shortcuts
    pub shortcuts: ShortcutConfig,

    /// Web search engines
    pub web_engines: HashMap<String, WebEngine>,
}

impl Default for Config {
    fn default() -> Self {
        let mut web_engines = HashMap::new();
        web_engines.insert(
            "g".to_string(),
            WebEngine {
                name: "Google".to_string(),
                url: "https://www.google.com/search?q={query}".to_string(),
                icon: "web-browser-symbolic".to_string(),
            },
        );
        web_engines.insert(
            "ddg".to_string(),
            WebEngine {
                name: "DuckDuckGo".to_string(),
                url: "https://duckduckgo.com/?q={query}".to_string(),
                icon: "web-browser-symbolic".to_string(),
            },
        );
        web_engines.insert(
            "yt".to_string(),
            WebEngine {
                name: "YouTube".to_string(),
                url: "https://www.youtube.com/results?search_query={query}".to_string(),
                icon: "video-symbolic".to_string(),
            },
        );
        web_engines.insert(
            "gh".to_string(),
            WebEngine {
                name: "GitHub".to_string(),
                url: "https://github.com/search?q={query}".to_string(),
                icon: "system-software-install-symbolic".to_string(),
            },
        );
        web_engines.insert(
            "wiki".to_string(),
            WebEngine {
                name: "Wikipedia".to_string(),
                url: "https://en.wikipedia.org/wiki/Special:Search?search={query}".to_string(),
                icon: "accessories-dictionary-symbolic".to_string(),
            },
        );

        Self {
            general: GeneralConfig::default(),
            search: SearchConfig::default(),
            ui: UiConfig::default(),
            plugins: PluginConfig::default(),
            shortcuts: ShortcutConfig::default(),
            web_engines,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config and save it
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-launcher")
            .join("config.toml")
    }

    /// Get history file path
    pub fn history_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-launcher")
            .join("history.json")
    }

    /// Get plugins directory
    pub fn plugins_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-launcher")
            .join("plugins")
    }
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Maximum number of results to show
    pub max_results: usize,

    /// Enable history
    pub enable_history: bool,

    /// Maximum history entries
    pub max_history: usize,

    /// Auto-hide after action
    pub auto_hide: bool,

    /// Show in all monitors
    pub all_monitors: bool,

    /// Enable fuzzy matching
    pub fuzzy_matching: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_results: 8,
            enable_history: true,
            max_history: 100,
            auto_hide: true,
            all_monitors: false,
            fuzzy_matching: true,
        }
    }
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Enable application search
    pub apps_enabled: bool,

    /// Enable file search
    pub files_enabled: bool,

    /// Enable calculator
    pub calculator_enabled: bool,

    /// Enable unit conversions
    pub conversions_enabled: bool,

    /// Enable web search
    pub web_enabled: bool,

    /// Enable system commands
    pub commands_enabled: bool,

    /// Enable plugins
    pub plugins_enabled: bool,

    /// File search paths
    pub file_search_paths: Vec<PathBuf>,

    /// Excluded paths for file search
    pub excluded_paths: Vec<PathBuf>,

    /// Application search paths
    pub app_search_paths: Vec<PathBuf>,

    /// Minimum query length
    pub min_query_length: usize,

    /// Search delay in milliseconds
    pub search_delay_ms: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            apps_enabled: true,
            files_enabled: true,
            calculator_enabled: true,
            conversions_enabled: true,
            web_enabled: true,
            commands_enabled: true,
            plugins_enabled: true,
            file_search_paths: vec![
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            ],
            excluded_paths: vec![
                PathBuf::from("/proc"),
                PathBuf::from("/sys"),
                PathBuf::from("/dev"),
                PathBuf::from("/run"),
            ],
            app_search_paths: vec![
                PathBuf::from("/usr/share/applications"),
                PathBuf::from("/usr/local/share/applications"),
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("applications"),
            ],
            min_query_length: 1,
            search_delay_ms: 100,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Window width
    pub width: i32,

    /// Window height (max)
    pub max_height: i32,

    /// Enable blur background
    pub blur_enabled: bool,

    /// Blur radius
    pub blur_radius: i32,

    /// Window opacity
    pub opacity: f64,

    /// Show icons
    pub show_icons: bool,

    /// Icon size
    pub icon_size: i32,

    /// Show categories
    pub show_categories: bool,

    /// Show preview panel
    pub show_preview: bool,

    /// Animation duration in milliseconds
    pub animation_duration_ms: u64,

    /// Theme (auto, light, dark)
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            width: 700,
            max_height: 500,
            blur_enabled: true,
            blur_radius: 20,
            opacity: 0.9,
            show_icons: true,
            icon_size: 40,
            show_categories: true,
            show_preview: true,
            animation_duration_ms: 200,
            theme: "auto".to_string(),
        }
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Enabled plugins
    pub enabled: Vec<String>,

    /// Disabled plugins
    pub disabled: Vec<String>,

    /// Plugin-specific settings
    pub settings: HashMap<String, toml::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: vec![],
            disabled: vec![],
            settings: HashMap::new(),
        }
    }
}

/// Keyboard shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Activate launcher
    pub activate: Vec<String>,

    /// Move selection up
    pub move_up: String,

    /// Move selection down
    pub move_down: String,

    /// Execute selected
    pub execute: String,

    /// Execute secondary action
    pub secondary: String,

    /// Show preview
    pub preview: String,

    /// Close launcher
    pub close: String,

    /// Copy to clipboard
    pub copy: String,

    /// Open file location
    pub open_location: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            activate: vec!["<Super>space".to_string(), "<Control>space".to_string()],
            move_up: "Up".to_string(),
            move_down: "Down".to_string(),
            execute: "Return".to_string(),
            secondary: "<Shift>Return".to_string(),
            preview: "Tab".to_string(),
            close: "Escape".to_string(),
            copy: "<Control>c".to_string(),
            open_location: "<Control>o".to_string(),
        }
    }
}

/// Web search engine definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebEngine {
    /// Display name
    pub name: String,

    /// URL template with {query} placeholder
    pub url: String,

    /// Icon name
    pub icon: String,
}
