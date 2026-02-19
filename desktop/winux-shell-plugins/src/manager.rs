//! Plugin manager
//!
//! Manages the lifecycle of all loaded plugins.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, Sender};
use parking_lot::RwLock;
use tokio::sync::broadcast;
use uuid::Uuid;

#[cfg(feature = "hot-reload")]
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::loader::{LoadedPlugin, PluginLoader};
use crate::plugin::{
    Plugin, PluginContext, PluginContextEvent, PluginError, PluginManifest,
    PluginMetadata, PluginResult, PluginState,
};
use crate::sandbox::permissions::PermissionSet;

/// Events emitted by the plugin manager
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// Plugin was loaded
    PluginLoaded { id: String },
    /// Plugin was unloaded
    PluginUnloaded { id: String },
    /// Plugin state changed
    PluginStateChanged { id: String, state: PluginState },
    /// Plugin error occurred
    PluginError { id: String, error: String },
    /// Plugin file changed (hot reload)
    PluginFileChanged { id: String },
    /// Plugin requested UI refresh
    PluginRefreshRequested { id: String },
    /// Plugin sent a notification
    PluginNotification {
        id: String,
        title: String,
        body: String,
        icon: Option<String>,
    },
}

/// Handle to a loaded plugin
pub struct PluginHandle {
    /// Unique instance ID
    pub instance_id: Uuid,
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin state
    pub state: PluginState,
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Plugin manifest
    pub manifest: Option<PluginManifest>,
    /// Plugin directory
    pub path: PathBuf,
    /// The loaded plugin
    plugin: LoadedPlugin,
    /// Plugin context
    context: PluginContext,
    /// Event receiver for this plugin
    event_receiver: Receiver<PluginContextEvent>,
}

impl PluginHandle {
    /// Get a reference to the plugin
    pub fn plugin(&self) -> &dyn Plugin {
        self.plugin.plugin.as_ref()
    }

    /// Get a mutable reference to the plugin
    pub fn plugin_mut(&mut self) -> &mut dyn Plugin {
        self.plugin.plugin.as_mut()
    }

    /// Get the plugin context
    pub fn context(&self) -> &PluginContext {
        &self.context
    }

    /// Poll for plugin events
    pub fn poll_events(&self) -> Vec<PluginContextEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_receiver.try_recv() {
            events.push(event);
        }
        events
    }
}

/// Configuration for the plugin manager
#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    /// Plugin directories to search
    pub plugin_dirs: Vec<PathBuf>,
    /// User data directory for plugin state
    pub data_dir: PathBuf,
    /// Enable hot reload
    pub hot_reload: bool,
    /// Default permissions for plugins
    pub default_permissions: PermissionSet,
    /// Maximum number of loaded plugins
    pub max_plugins: usize,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux")
            .join("plugins");

        Self {
            plugin_dirs: PluginLoader::default_plugin_dirs(),
            data_dir,
            hot_reload: true,
            default_permissions: PermissionSet::default(),
            max_plugins: 100,
        }
    }
}

/// Plugin manager
pub struct PluginManager {
    /// Configuration
    config: PluginManagerConfig,
    /// Plugin loader
    loader: PluginLoader,
    /// Loaded plugins
    plugins: Arc<RwLock<HashMap<String, PluginHandle>>>,
    /// Plugin states
    states: Arc<RwLock<HashMap<String, PluginState>>>,
    /// Disabled plugins (by user)
    disabled: Arc<RwLock<Vec<String>>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<PluginEvent>,
    /// File watcher for hot reload
    #[cfg(feature = "hot-reload")]
    watcher: Option<RecommendedWatcher>,
    /// Watched paths
    #[cfg(feature = "hot-reload")]
    watched_paths: Arc<RwLock<HashMap<PathBuf, String>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: PluginManagerConfig) -> Self {
        let mut loader = PluginLoader::new();
        for dir in &config.plugin_dirs {
            loader.add_search_path(dir);
        }

        let (event_tx, _) = broadcast::channel(256);

        Self {
            config,
            loader,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            disabled: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            #[cfg(feature = "hot-reload")]
            watcher: None,
            #[cfg(feature = "hot-reload")]
            watched_paths: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the plugin manager
    pub fn init(&mut self) -> PluginResult<()> {
        // Create data directory
        std::fs::create_dir_all(&self.config.data_dir)?;

        // Load disabled plugins list
        self.load_disabled_list()?;

        // Setup hot reload if enabled
        #[cfg(feature = "hot-reload")]
        if self.config.hot_reload {
            self.setup_hot_reload()?;
        }

        Ok(())
    }

    /// Subscribe to plugin events
    pub fn subscribe(&self) -> broadcast::Receiver<PluginEvent> {
        self.event_tx.subscribe()
    }

    /// Discover all available plugins
    pub fn discover_plugins(&self) -> Vec<PluginManifest> {
        self.loader.discover_plugins()
    }

    /// Load a plugin by ID
    pub fn load_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        // Check if already loaded
        if self.plugins.read().contains_key(plugin_id) {
            return Err(PluginError::AlreadyLoaded(plugin_id.to_string()));
        }

        // Check if disabled
        if self.disabled.read().contains(&plugin_id.to_string()) {
            return Err(PluginError::Disabled(plugin_id.to_string()));
        }

        // Find plugin directory
        let plugin_dir = self.find_plugin_dir(plugin_id)?;

        self.load_plugin_from_dir(&plugin_dir)
    }

    /// Load a plugin from a directory
    pub fn load_plugin_from_dir<P: AsRef<Path>>(&self, plugin_dir: P) -> PluginResult<()> {
        let plugin_dir = plugin_dir.as_ref();

        // Update state
        let temp_id = plugin_dir.to_string_lossy().to_string();
        self.states.write().insert(temp_id.clone(), PluginState::Loading);

        // Load the plugin
        let loaded = match self.loader.load_plugin(plugin_dir) {
            Ok(p) => p,
            Err(e) => {
                self.states.write().insert(temp_id, PluginState::Failed);
                return Err(e);
            }
        };

        let plugin_id = loaded.metadata.id.clone();

        // Check dependencies
        self.check_dependencies(&loaded)?;

        // Create plugin context
        let plugin_data_dir = self.config.data_dir.join(&plugin_id);
        let (event_sender, event_receiver) = async_channel::bounded(64);

        let permissions = loaded
            .manifest
            .as_ref()
            .map(|m| m.plugin.permissions.clone())
            .unwrap_or_else(|| self.config.default_permissions.clone());

        let context = PluginContext::new(
            plugin_id.clone(),
            plugin_data_dir,
            permissions,
            Some(event_sender),
        );

        // Create plugin handle
        let mut handle = PluginHandle {
            instance_id: Uuid::new_v4(),
            plugin_id: plugin_id.clone(),
            state: PluginState::Loading,
            metadata: loaded.metadata.clone(),
            manifest: loaded.manifest.clone(),
            path: loaded.path.clone(),
            plugin: loaded,
            context,
            event_receiver,
        };

        // Initialize the plugin
        if let Err(e) = handle.plugin_mut().init(&handle.context) {
            self.states.write().insert(plugin_id.clone(), PluginState::Failed);
            let _ = self.event_tx.send(PluginEvent::PluginError {
                id: plugin_id,
                error: e.to_string(),
            });
            return Err(e);
        }

        handle.state = PluginState::Active;

        // Store the plugin
        self.plugins.write().insert(plugin_id.clone(), handle);
        self.states.write().insert(plugin_id.clone(), PluginState::Active);

        // Setup hot reload watching
        #[cfg(feature = "hot-reload")]
        if self.config.hot_reload {
            self.watch_plugin(&plugin_id, plugin_dir);
        }

        // Emit event
        let _ = self.event_tx.send(PluginEvent::PluginLoaded { id: plugin_id });

        Ok(())
    }

    /// Unload a plugin
    pub fn unload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write();

        let mut handle = plugins
            .remove(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        // Update state
        self.states.write().insert(plugin_id.to_string(), PluginState::Unloading);

        // Shutdown the plugin
        if let Err(e) = handle.plugin_mut().shutdown() {
            log::warn!("Plugin {} shutdown error: {}", plugin_id, e);
        }

        // Update state
        self.states.write().insert(plugin_id.to_string(), PluginState::Unloaded);

        // Stop watching
        #[cfg(feature = "hot-reload")]
        self.unwatch_plugin(plugin_id);

        // Emit event
        let _ = self.event_tx.send(PluginEvent::PluginUnloaded {
            id: plugin_id.to_string(),
        });

        Ok(())
    }

    /// Reload a plugin (hot reload)
    pub fn reload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let path = {
            let plugins = self.plugins.read();
            plugins
                .get(plugin_id)
                .map(|h| h.path.clone())
                .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?
        };

        // Unload first
        self.unload_plugin(plugin_id)?;

        // Small delay to ensure library is released
        std::thread::sleep(Duration::from_millis(100));

        // Reload
        self.load_plugin_from_dir(&path)
    }

    /// Suspend a plugin
    pub fn suspend_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write();
        let handle = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        handle.plugin_mut().suspend()?;
        handle.state = PluginState::Suspended;
        self.states.write().insert(plugin_id.to_string(), PluginState::Suspended);

        let _ = self.event_tx.send(PluginEvent::PluginStateChanged {
            id: plugin_id.to_string(),
            state: PluginState::Suspended,
        });

        Ok(())
    }

    /// Resume a suspended plugin
    pub fn resume_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write();
        let handle = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        handle.plugin_mut().resume()?;
        handle.state = PluginState::Active;
        self.states.write().insert(plugin_id.to_string(), PluginState::Active);

        let _ = self.event_tx.send(PluginEvent::PluginStateChanged {
            id: plugin_id.to_string(),
            state: PluginState::Active,
        });

        Ok(())
    }

    /// Disable a plugin (won't load on startup)
    pub fn disable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        // Unload if loaded
        if self.plugins.read().contains_key(plugin_id) {
            self.unload_plugin(plugin_id)?;
        }

        // Add to disabled list
        let mut disabled = self.disabled.write();
        if !disabled.contains(&plugin_id.to_string()) {
            disabled.push(plugin_id.to_string());
        }

        self.states.write().insert(plugin_id.to_string(), PluginState::Disabled);
        self.save_disabled_list()?;

        Ok(())
    }

    /// Enable a disabled plugin
    pub fn enable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        // Remove from disabled list
        self.disabled.write().retain(|id| id != plugin_id);
        self.save_disabled_list()?;

        // Load the plugin
        self.load_plugin(plugin_id)
    }

    /// Get plugin state
    pub fn plugin_state(&self, plugin_id: &str) -> Option<PluginState> {
        self.states.read().get(plugin_id).copied()
    }

    /// Get all loaded plugin IDs
    pub fn loaded_plugins(&self) -> Vec<String> {
        self.plugins.read().keys().cloned().collect()
    }

    /// Get plugin metadata
    pub fn plugin_metadata(&self, plugin_id: &str) -> Option<PluginMetadata> {
        self.plugins.read().get(plugin_id).map(|h| h.metadata.clone())
    }

    /// Get all plugin metadata
    pub fn all_plugin_metadata(&self) -> Vec<PluginMetadata> {
        self.plugins.read().values().map(|h| h.metadata.clone()).collect()
    }

    /// Execute with plugin reference
    pub fn with_plugin<F, R>(&self, plugin_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&dyn Plugin) -> R,
    {
        self.plugins.read().get(plugin_id).map(|h| f(h.plugin()))
    }

    /// Execute with mutable plugin reference
    pub fn with_plugin_mut<F, R>(&self, plugin_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Plugin) -> R,
    {
        self.plugins.write().get_mut(plugin_id).map(|h| f(h.plugin_mut()))
    }

    /// Update all plugins that want updates
    pub fn update_plugins(&self) {
        let mut plugins = self.plugins.write();
        for (id, handle) in plugins.iter_mut() {
            if handle.state == PluginState::Active && handle.plugin().wants_updates() {
                if let Err(e) = handle.plugin_mut().update() {
                    log::warn!("Plugin {} update error: {}", id, e);
                }

                // Process plugin events
                for event in handle.poll_events() {
                    match event {
                        PluginContextEvent::RefreshUi => {
                            let _ = self.event_tx.send(PluginEvent::PluginRefreshRequested {
                                id: id.clone(),
                            });
                        }
                        PluginContextEvent::ShowNotification { title, body, icon } => {
                            let _ = self.event_tx.send(PluginEvent::PluginNotification {
                                id: id.clone(),
                                title,
                                body,
                                icon,
                            });
                        }
                        PluginContextEvent::Log { level, message } => {
                            log::log!(level, "[{}] {}", id, message);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Load all auto-start plugins
    pub fn load_autostart_plugins(&self) -> Vec<PluginError> {
        let mut errors = Vec::new();
        let manifests = self.discover_plugins();

        for manifest in manifests {
            if manifest.runtime.auto_start {
                if let Err(e) = self.load_plugin(&manifest.plugin.id) {
                    errors.push(e);
                }
            }
        }

        errors
    }

    /// Shutdown all plugins
    pub fn shutdown_all(&self) {
        let plugin_ids: Vec<String> = self.plugins.read().keys().cloned().collect();
        for id in plugin_ids {
            if let Err(e) = self.unload_plugin(&id) {
                log::warn!("Error unloading plugin {}: {}", id, e);
            }
        }
    }

    /// Find plugin directory by ID
    fn find_plugin_dir(&self, plugin_id: &str) -> PluginResult<PathBuf> {
        for search_dir in self.loader.search_paths() {
            let plugin_dir = search_dir.join(plugin_id);
            if plugin_dir.exists() && plugin_dir.join("plugin.toml").exists() {
                return Ok(plugin_dir);
            }

            // Also check by directory name (without domain prefix)
            let short_name = plugin_id.rsplit('.').next().unwrap_or(plugin_id);
            let plugin_dir = search_dir.join(short_name);
            if plugin_dir.exists() && plugin_dir.join("plugin.toml").exists() {
                return Ok(plugin_dir);
            }
        }

        Err(PluginError::NotFound(plugin_id.to_string()))
    }

    /// Check plugin dependencies
    fn check_dependencies(&self, plugin: &LoadedPlugin) -> PluginResult<()> {
        if let Some(manifest) = &plugin.manifest {
            for (dep_id, version_req) in &manifest.plugin.dependencies {
                let dep_loaded = self.plugins.read().contains_key(dep_id);
                if !dep_loaded {
                    return Err(PluginError::DependencyNotSatisfied(format!(
                        "{} requires {} {}",
                        plugin.metadata.id, dep_id, version_req
                    )));
                }
            }
        }
        Ok(())
    }

    /// Load disabled plugins list
    fn load_disabled_list(&self) -> PluginResult<()> {
        let path = self.config.data_dir.join("disabled.json");
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let list: Vec<String> = serde_json::from_str(&content)
                .map_err(|e| PluginError::ConfigError(e.to_string()))?;
            *self.disabled.write() = list;
        }
        Ok(())
    }

    /// Save disabled plugins list
    fn save_disabled_list(&self) -> PluginResult<()> {
        let path = self.config.data_dir.join("disabled.json");
        let content = serde_json::to_string_pretty(&*self.disabled.read())
            .map_err(|e| PluginError::ConfigError(e.to_string()))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Setup hot reload file watcher
    #[cfg(feature = "hot-reload")]
    fn setup_hot_reload(&mut self) -> PluginResult<()> {
        let event_tx = self.event_tx.clone();
        let watched_paths = self.watched_paths.clone();

        let watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() {
                        for path in &event.paths {
                            if let Some(plugin_id) = watched_paths.read().get(path) {
                                let _ = event_tx.send(PluginEvent::PluginFileChanged {
                                    id: plugin_id.clone(),
                                });
                            }
                        }
                    }
                }
            },
        )
        .map_err(|e| PluginError::RuntimeError(format!("Failed to create file watcher: {}", e)))?;

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Watch a plugin directory for changes
    #[cfg(feature = "hot-reload")]
    fn watch_plugin(&self, plugin_id: &str, path: &Path) {
        if let Some(watcher) = &self.watcher {
            // Note: notify watcher needs to be mutable, but we can't easily do that here
            // In a real implementation, you'd use interior mutability or a different pattern
            log::debug!("Would watch {:?} for plugin {}", path, plugin_id);
            self.watched_paths
                .write()
                .insert(path.to_path_buf(), plugin_id.to_string());
        }
    }

    /// Stop watching a plugin directory
    #[cfg(feature = "hot-reload")]
    fn unwatch_plugin(&self, plugin_id: &str) {
        self.watched_paths.write().retain(|_, id| id != plugin_id);
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let config = PluginManagerConfig::default();
        let _manager = PluginManager::new(config);
    }

    #[test]
    fn test_default_config() {
        let config = PluginManagerConfig::default();
        assert!(config.hot_reload);
        assert_eq!(config.max_plugins, 100);
    }
}
