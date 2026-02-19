//! Plugin system for extending launcher functionality

use crate::config::Config;
use crate::search::{SearchCategory, SearchResult, SearchResultKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Plugin description
    pub description: String,

    /// Plugin version
    pub version: String,

    /// Plugin author
    pub author: String,

    /// Plugin website
    pub website: Option<String>,

    /// Trigger keywords (e.g., "!" for snippets)
    pub keywords: Vec<String>,

    /// Plugin icon
    pub icon: String,

    /// Plugin type
    pub plugin_type: PluginType,
}

/// Plugin types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// Script-based plugin (shell scripts)
    Script,
    /// JSON data plugin (static data)
    Json,
    /// Native plugin (shared library)
    Native,
    /// Web service plugin
    WebService,
}

/// Plugin definition
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Plugin directory
    pub path: PathBuf,

    /// Whether plugin is enabled
    pub enabled: bool,

    /// Plugin data (for JSON plugins)
    pub data: Option<PluginData>,
}

/// Plugin data for JSON-based plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginData {
    /// Static items
    pub items: Vec<PluginItem>,
}

/// Plugin item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginItem {
    /// Item title
    pub title: String,

    /// Item subtitle
    pub subtitle: Option<String>,

    /// Item icon
    pub icon: Option<String>,

    /// Action to perform
    pub action: PluginAction,

    /// Keywords for matching
    pub keywords: Vec<String>,
}

/// Plugin action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PluginAction {
    /// Open URL
    Url { url: String },

    /// Run command
    Command { command: String },

    /// Copy to clipboard
    Copy { text: String },

    /// Open file/folder
    Open { path: String },

    /// Send notification
    Notify { title: String, body: String },
}

/// Plugin manager
pub struct PluginManager {
    config: Arc<Config>,
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new(config: Arc<Config>) -> Self {
        let mut manager = Self {
            config,
            plugins: HashMap::new(),
        };

        manager.refresh();
        manager
    }

    /// Refresh/reload plugins
    pub fn refresh(&mut self) {
        self.plugins.clear();

        let plugins_dir = Config::plugins_dir();
        if !plugins_dir.exists() {
            // Create plugins directory
            if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
                warn!("Failed to create plugins directory: {}", e);
            }
            return;
        }

        // Load built-in plugins
        self.load_builtin_plugins();

        // Scan plugins directory
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if let Err(e) = self.load_plugin(&path) {
                        warn!("Failed to load plugin from {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} plugins", self.plugins.len());
    }

    /// Load built-in plugins
    fn load_builtin_plugins(&mut self) {
        // Emoji plugin
        self.plugins.insert(
            "builtin:emoji".to_string(),
            Plugin {
                metadata: PluginMetadata {
                    id: "builtin:emoji".to_string(),
                    name: "Emoji".to_string(),
                    description: "Search and insert emojis".to_string(),
                    version: "1.0.0".to_string(),
                    author: "Winux Team".to_string(),
                    website: None,
                    keywords: vec!["emoji".to_string(), ":".to_string()],
                    icon: "face-smile-symbolic".to_string(),
                    plugin_type: PluginType::Json,
                },
                path: PathBuf::new(),
                enabled: true,
                data: Some(self.create_emoji_data()),
            },
        );

        // Snippets plugin
        self.plugins.insert(
            "builtin:snippets".to_string(),
            Plugin {
                metadata: PluginMetadata {
                    id: "builtin:snippets".to_string(),
                    name: "Snippets".to_string(),
                    description: "Text snippets and templates".to_string(),
                    version: "1.0.0".to_string(),
                    author: "Winux Team".to_string(),
                    website: None,
                    keywords: vec!["snip".to_string(), "!".to_string()],
                    icon: "edit-paste-symbolic".to_string(),
                    plugin_type: PluginType::Json,
                },
                path: PathBuf::new(),
                enabled: true,
                data: Some(PluginData { items: vec![] }),
            },
        );
    }

    /// Create emoji data
    fn create_emoji_data(&self) -> PluginData {
        // Common emojis - in a real implementation, this would be a comprehensive list
        let emojis = vec![
            ("smile", "Smiling Face", "face-smile-symbolic"),
            ("heart", "Red Heart", "emblem-favorite-symbolic"),
            ("thumbsup", "Thumbs Up", "emblem-ok-symbolic"),
            ("fire", "Fire", "weather-severe-alert-symbolic"),
            ("star", "Star", "starred-symbolic"),
            ("check", "Check Mark", "emblem-ok-symbolic"),
            ("rocket", "Rocket", "send-to-symbolic"),
            ("coffee", "Coffee", "accessories-dictionary-symbolic"),
        ];

        let items = emojis
            .into_iter()
            .map(|(keyword, title, icon)| PluginItem {
                title: title.to_string(),
                subtitle: Some(format!(":{}: ", keyword)),
                icon: Some(icon.to_string()),
                action: PluginAction::Copy {
                    text: format!(":{}: ", keyword),
                },
                keywords: vec![keyword.to_string()],
            })
            .collect();

        PluginData { items }
    }

    /// Load plugin from directory
    fn load_plugin(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        let metadata_path = path.join("plugin.json");
        if !metadata_path.exists() {
            return Err(anyhow::anyhow!("plugin.json not found"));
        }

        let content = std::fs::read_to_string(&metadata_path)?;
        let metadata: PluginMetadata = serde_json::from_str(&content)?;

        // Check if plugin is enabled/disabled
        let enabled = !self.config.plugins.disabled.contains(&metadata.id)
            && (self.config.plugins.enabled.is_empty()
                || self.config.plugins.enabled.contains(&metadata.id));

        // Load plugin data for JSON plugins
        let data = if matches!(metadata.plugin_type, PluginType::Json) {
            let data_path = path.join("data.json");
            if data_path.exists() {
                let data_content = std::fs::read_to_string(&data_path)?;
                Some(serde_json::from_str(&data_content)?)
            } else {
                None
            }
        } else {
            None
        };

        let plugin = Plugin {
            metadata,
            path: path.clone(),
            enabled,
            data,
        };

        debug!("Loaded plugin: {}", plugin.metadata.name);
        self.plugins.insert(plugin.metadata.id.clone(), plugin);

        Ok(())
    }

    /// Search plugins
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for plugin in self.plugins.values() {
            if !plugin.enabled {
                continue;
            }

            // Check if query matches plugin keywords
            let matches_keyword = plugin
                .metadata
                .keywords
                .iter()
                .any(|k| query_lower.starts_with(&k.to_lowercase()));

            if !matches_keyword {
                continue;
            }

            // Search plugin data
            if let Some(ref data) = plugin.data {
                for item in &data.items {
                    // Check if item matches
                    let item_matches = item
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&query_lower))
                        || item.title.to_lowercase().contains(&query_lower);

                    if item_matches {
                        let action_str = match &item.action {
                            PluginAction::Url { url } => url.clone(),
                            PluginAction::Command { command } => command.clone(),
                            PluginAction::Copy { text } => format!("Copy: {}", text),
                            PluginAction::Open { path } => format!("Open: {}", path),
                            PluginAction::Notify { title, .. } => format!("Notify: {}", title),
                        };

                        results.push(SearchResult {
                            id: format!("plugin:{}:{}", plugin.metadata.id, item.title),
                            title: item.title.clone(),
                            subtitle: item.subtitle.clone().unwrap_or_default(),
                            icon: item
                                .icon
                                .clone()
                                .unwrap_or_else(|| plugin.metadata.icon.clone()),
                            category: SearchCategory::Plugins,
                            kind: SearchResultKind::Plugin {
                                plugin_id: plugin.metadata.id.clone(),
                                action: action_str,
                            },
                            score: 70,
                            from_history: false,
                        });
                    }
                }
            }
        }

        results.truncate(5);
        results
    }

    /// Get all plugins
    pub fn plugins(&self) -> &HashMap<String, Plugin> {
        &self.plugins
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, id: &str) {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.enabled = true;
        }
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, id: &str) {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.enabled = false;
        }
    }
}
