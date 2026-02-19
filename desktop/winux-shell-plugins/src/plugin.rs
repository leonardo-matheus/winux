//! Core plugin traits and types
//!
//! This module defines the fundamental plugin interface that all plugins must implement.

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::api::panel::PanelWidget;
use crate::api::notifications::NotificationHandler;
use crate::api::launcher::LauncherProvider;
use crate::api::settings::SettingsProvider;
use crate::api::commands::CommandProvider;
use crate::sandbox::permissions::PermissionSet;

/// Plugin error types
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Plugin dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Plugin disabled: {0}")]
    Disabled(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("API error: {0}")]
    ApiError(String),
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is registered but not loaded
    Unloaded,
    /// Plugin is being loaded
    Loading,
    /// Plugin is loaded and active
    Active,
    /// Plugin is paused/suspended
    Suspended,
    /// Plugin is being unloaded
    Unloading,
    /// Plugin failed to load
    Failed,
    /// Plugin is disabled by user
    Disabled,
}

impl Default for PluginState {
    fn default() -> Self {
        Self::Unloaded
    }
}

/// Plugin capabilities that can be provided
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    /// Can add widgets to the panel
    PanelWidget,
    /// Can handle notifications
    NotificationHandler,
    /// Can provide launcher search results
    LauncherProvider,
    /// Can add settings pages
    SettingsProvider,
    /// Can register commands
    CommandProvider,
    /// Can register keyboard shortcuts
    KeyboardShortcuts,
    /// Can add context menu items
    ContextMenu,
    /// Can provide file previews
    FilePreview,
    /// Can run background tasks
    BackgroundTask,
    /// Can access network
    Network,
    /// Can access filesystem
    Filesystem,
    /// Can access system information
    SystemInfo,
    /// Can send notifications
    SendNotifications,
}

/// Plugin metadata describing the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier (reverse domain notation)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Plugin version
    #[serde(with = "version_serde")]
    pub version: Version,
    /// Plugin description
    pub description: String,
    /// Plugin author(s)
    pub authors: Vec<String>,
    /// Project homepage
    pub homepage: Option<String>,
    /// License identifier
    pub license: Option<String>,
    /// Minimum API version required
    #[serde(with = "version_serde")]
    pub min_api_version: Version,
    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Required permissions
    pub permissions: PermissionSet,
    /// Plugin dependencies (id -> version requirement)
    pub dependencies: HashMap<String, String>,
    /// Plugin icon path (relative to plugin directory)
    pub icon: Option<String>,
    /// Plugin category for store
    pub category: Option<String>,
    /// Keywords for search
    pub keywords: Vec<String>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: Version::new(0, 1, 0),
            description: String::new(),
            authors: Vec::new(),
            homepage: None,
            license: None,
            min_api_version: Version::new(1, 0, 0),
            capabilities: Vec::new(),
            permissions: PermissionSet::default(),
            dependencies: HashMap::new(),
            icon: None,
            category: None,
            keywords: Vec::new(),
        }
    }
}

/// Plugin manifest file (plugin.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginMetadata,
    /// Build configuration
    #[serde(default)]
    pub build: PluginBuildConfig,
    /// Runtime configuration
    #[serde(default)]
    pub runtime: PluginRuntimeConfig,
}

/// Plugin build configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginBuildConfig {
    /// Library filename (without extension)
    pub lib_name: Option<String>,
    /// Additional resources to include
    #[serde(default)]
    pub resources: Vec<String>,
    /// Build features to enable
    #[serde(default)]
    pub features: Vec<String>,
}

/// Plugin runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRuntimeConfig {
    /// Auto-start on shell launch
    #[serde(default = "default_true")]
    pub auto_start: bool,
    /// Allow hot reload
    #[serde(default = "default_true")]
    pub hot_reload: bool,
    /// Run in sandbox
    #[serde(default = "default_true")]
    pub sandbox: bool,
    /// Maximum memory usage (MB)
    #[serde(default = "default_memory")]
    pub max_memory_mb: u32,
    /// Plugin priority (higher = loaded first)
    #[serde(default)]
    pub priority: i32,
}

impl Default for PluginRuntimeConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            hot_reload: true,
            sandbox: true,
            max_memory_mb: 128,
            priority: 0,
        }
    }
}

fn default_true() -> bool { true }
fn default_memory() -> u32 { 128 }

/// Context provided to plugins for accessing shell functionality
pub struct PluginContext {
    /// Plugin's unique ID
    pub plugin_id: String,
    /// Plugin's data directory
    pub data_dir: PathBuf,
    /// Plugin's config directory
    pub config_dir: PathBuf,
    /// Plugin's cache directory
    pub cache_dir: PathBuf,
    /// Plugin's resource directory
    pub resource_dir: PathBuf,
    /// Granted permissions
    pub permissions: PermissionSet,
    /// Shared state storage
    state: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    /// Event sender for plugin events
    event_sender: Option<async_channel::Sender<PluginContextEvent>>,
}

/// Events that plugins can emit through the context
#[derive(Debug, Clone)]
pub enum PluginContextEvent {
    /// Request to show a notification
    ShowNotification {
        title: String,
        body: String,
        icon: Option<String>,
    },
    /// Request to refresh plugin UI
    RefreshUi,
    /// Request to save plugin state
    SaveState,
    /// Log message
    Log {
        level: log::Level,
        message: String,
    },
    /// Custom event
    Custom {
        name: String,
        data: String,
    },
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(
        plugin_id: String,
        base_dir: PathBuf,
        permissions: PermissionSet,
        event_sender: Option<async_channel::Sender<PluginContextEvent>>,
    ) -> Self {
        let data_dir = base_dir.join("data");
        let config_dir = base_dir.join("config");
        let cache_dir = base_dir.join("cache");
        let resource_dir = base_dir.join("resources");

        // Create directories if they don't exist
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = std::fs::create_dir_all(&config_dir);
        let _ = std::fs::create_dir_all(&cache_dir);

        Self {
            plugin_id,
            data_dir,
            config_dir,
            cache_dir,
            resource_dir,
            permissions,
            state: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    /// Store a value in the context state
    pub fn set_state<T: Any + Send + Sync + Clone>(&self, key: &str, value: T) {
        self.state.write().insert(key.to_string(), Box::new(value));
    }

    /// Retrieve a value from the context state
    pub fn get_state<T: Any + Clone>(&self, key: &str) -> Option<T> {
        self.state
            .read()
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    /// Check if a permission is granted
    pub fn has_permission(&self, permission: &crate::sandbox::permissions::Permission) -> bool {
        self.permissions.has(permission)
    }

    /// Send an event from the plugin
    pub fn emit_event(&self, event: PluginContextEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.try_send(event);
        }
    }

    /// Request to show a notification
    pub fn show_notification(&self, title: &str, body: &str, icon: Option<&str>) {
        self.emit_event(PluginContextEvent::ShowNotification {
            title: title.to_string(),
            body: body.to_string(),
            icon: icon.map(String::from),
        });
    }

    /// Log a message
    pub fn log(&self, level: log::Level, message: &str) {
        self.emit_event(PluginContextEvent::Log {
            level,
            message: message.to_string(),
        });
    }

    /// Request UI refresh
    pub fn request_refresh(&self) {
        self.emit_event(PluginContextEvent::RefreshUi);
    }

    /// Get plugin's config file path
    pub fn config_file(&self, filename: &str) -> PathBuf {
        self.config_dir.join(filename)
    }

    /// Get plugin's data file path
    pub fn data_file(&self, filename: &str) -> PathBuf {
        self.data_dir.join(filename)
    }

    /// Get plugin's resource file path
    pub fn resource_file(&self, filename: &str) -> PathBuf {
        self.resource_dir.join(filename)
    }
}

/// The main plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin
    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()>;

    /// Shutdown the plugin
    fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Called when plugin is suspended
    fn suspend(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Called when plugin is resumed
    fn resume(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Get panel widget if this plugin provides one
    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        None
    }

    /// Get notification handler if this plugin provides one
    fn notification_handler(&self) -> Option<Box<dyn NotificationHandler>> {
        None
    }

    /// Get launcher provider if this plugin provides one
    fn launcher_provider(&self) -> Option<Box<dyn LauncherProvider>> {
        None
    }

    /// Get settings provider if this plugin provides one
    fn settings_provider(&self) -> Option<Box<dyn SettingsProvider>> {
        None
    }

    /// Get command provider if this plugin provides one
    fn command_provider(&self) -> Option<Box<dyn CommandProvider>> {
        None
    }

    /// Handle configuration changes
    fn on_config_changed(&mut self, _key: &str, _value: &str) -> PluginResult<()> {
        Ok(())
    }

    /// Periodic update (called every second if plugin requests it)
    fn update(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Check if plugin wants periodic updates
    fn wants_updates(&self) -> bool {
        false
    }

    /// Get update interval in milliseconds (default: 1000ms)
    fn update_interval(&self) -> u32 {
        1000
    }
}

/// Serde support for semver Version
mod version_serde {
    use semver::Version;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        version.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata_default() {
        let meta = PluginMetadata::default();
        assert!(meta.id.is_empty());
        assert_eq!(meta.version, Version::new(0, 1, 0));
    }

    #[test]
    fn test_plugin_state_default() {
        let state = PluginState::default();
        assert_eq!(state, PluginState::Unloaded);
    }
}
