//! Dynamic plugin loader
//!
//! Handles loading plugins from shared libraries at runtime.

use std::collections::HashMap;
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use libloading::{Library, Symbol};
use parking_lot::RwLock;
use semver::Version;
use walkdir::WalkDir;

use crate::plugin::{Plugin, PluginError, PluginManifest, PluginMetadata, PluginResult};
use crate::API_VERSION;

/// Type signature for plugin creation function
type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn Plugin;
/// Type signature for plugin destruction function
type PluginDestroyFn = unsafe extern "C" fn(*mut dyn Plugin);
/// Type signature for API version function
type PluginApiVersionFn = unsafe extern "C" fn() -> *const std::os::raw::c_char;

/// A dynamically loaded plugin
pub struct DynamicPlugin {
    /// The loaded library
    _library: Library,
    /// The plugin instance
    plugin: *mut dyn Plugin,
    /// Destroy function
    destroy_fn: PluginDestroyFn,
}

// Safety: Plugin trait requires Send + Sync
unsafe impl Send for DynamicPlugin {}
unsafe impl Sync for DynamicPlugin {}

impl DynamicPlugin {
    /// Get a reference to the plugin
    pub fn as_ref(&self) -> &dyn Plugin {
        unsafe { &*self.plugin }
    }

    /// Get a mutable reference to the plugin
    pub fn as_mut(&mut self) -> &mut dyn Plugin {
        unsafe { &mut *self.plugin }
    }
}

impl Drop for DynamicPlugin {
    fn drop(&mut self) {
        unsafe {
            (self.destroy_fn)(self.plugin);
        }
    }
}

/// Information about a loaded plugin
pub struct LoadedPlugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Plugin manifest
    pub manifest: Option<PluginManifest>,
    /// Path to the plugin
    pub path: PathBuf,
    /// The dynamic plugin instance
    pub plugin: DynamicPlugin,
}

/// Plugin loader for dynamic libraries
pub struct PluginLoader {
    /// Plugin search paths
    search_paths: Vec<PathBuf>,
    /// Cached plugin manifests
    manifests: Arc<RwLock<HashMap<String, PluginManifest>>>,
    /// Platform-specific library extension
    lib_extension: &'static str,
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        let lib_extension = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        Self {
            search_paths: Vec::new(),
            manifests: Arc::new(RwLock::new(HashMap::new())),
            lib_extension,
        }
    }

    /// Add a search path for plugins
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref().to_path_buf();
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Get default plugin directories
    pub fn default_plugin_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // User plugins directory
        if let Some(data_dir) = dirs::data_dir() {
            dirs.push(data_dir.join("winux").join("plugins"));
        }

        // System plugins directory
        dirs.push(PathBuf::from("/usr/share/winux/plugins"));
        dirs.push(PathBuf::from("/usr/local/share/winux/plugins"));

        // Development directory
        if let Ok(cwd) = std::env::current_dir() {
            dirs.push(cwd.join("plugins"));
        }

        dirs
    }

    /// Discover available plugins in search paths
    pub fn discover_plugins(&self) -> Vec<PluginManifest> {
        let mut plugins = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            for entry in WalkDir::new(search_path)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.file_name() == Some(std::ffi::OsStr::new("plugin.toml")) {
                    if let Ok(manifest) = self.load_manifest(path) {
                        plugins.push(manifest);
                    }
                }
            }
        }

        plugins
    }

    /// Load a plugin manifest from a file
    pub fn load_manifest<P: AsRef<Path>>(&self, path: P) -> PluginResult<PluginManifest> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let manifest: PluginManifest = toml::from_str(&content)
            .map_err(|e| PluginError::ConfigError(e.to_string()))?;

        // Cache the manifest
        self.manifests
            .write()
            .insert(manifest.plugin.id.clone(), manifest.clone());

        Ok(manifest)
    }

    /// Load a plugin from a directory
    pub fn load_plugin<P: AsRef<Path>>(&self, plugin_dir: P) -> PluginResult<LoadedPlugin> {
        let plugin_dir = plugin_dir.as_ref();
        let manifest_path = plugin_dir.join("plugin.toml");

        // Load manifest if exists
        let manifest = if manifest_path.exists() {
            Some(self.load_manifest(&manifest_path)?)
        } else {
            None
        };

        // Determine library name
        let lib_name = manifest
            .as_ref()
            .and_then(|m| m.build.lib_name.clone())
            .unwrap_or_else(|| {
                plugin_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().replace('-', "_"))
                    .unwrap_or_else(|| "plugin".to_string())
            });

        // Find the library file
        let lib_path = self.find_library(plugin_dir, &lib_name)?;

        // Load the dynamic library
        let plugin = self.load_library(&lib_path)?;

        let metadata = plugin.plugin.as_ref().metadata();

        Ok(LoadedPlugin {
            metadata,
            manifest,
            path: plugin_dir.to_path_buf(),
            plugin,
        })
    }

    /// Find the plugin library in a directory
    fn find_library(&self, plugin_dir: &Path, lib_name: &str) -> PluginResult<PathBuf> {
        // Check common locations
        let candidates = [
            plugin_dir.join(format!("lib{}.{}", lib_name, self.lib_extension)),
            plugin_dir.join(format!("{}.{}", lib_name, self.lib_extension)),
            plugin_dir
                .join("target")
                .join("release")
                .join(format!("lib{}.{}", lib_name, self.lib_extension)),
            plugin_dir
                .join("target")
                .join("debug")
                .join(format!("lib{}.{}", lib_name, self.lib_extension)),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Ok(candidate.clone());
            }
        }

        Err(PluginError::NotFound(format!(
            "Plugin library not found in {:?}",
            plugin_dir
        )))
    }

    /// Load a plugin from a shared library file
    pub fn load_library<P: AsRef<Path>>(&self, lib_path: P) -> PluginResult<DynamicPlugin> {
        let lib_path = lib_path.as_ref();

        // Load the library
        let library = unsafe {
            Library::new(lib_path).map_err(|e| {
                PluginError::InitializationFailed(format!(
                    "Failed to load library {:?}: {}",
                    lib_path, e
                ))
            })?
        };

        // Check API version
        let api_version_fn: Symbol<PluginApiVersionFn> = unsafe {
            library
                .get(b"_winux_plugin_api_version\0")
                .map_err(|e| {
                    PluginError::InitializationFailed(format!(
                        "Missing API version function: {}",
                        e
                    ))
                })?
        };

        let api_version_ptr = unsafe { api_version_fn() };
        let api_version = unsafe { CStr::from_ptr(api_version_ptr) }
            .to_str()
            .map_err(|_| PluginError::InitializationFailed("Invalid API version string".into()))?;

        self.check_api_compatibility(api_version)?;

        // Get create and destroy functions
        let create_fn: Symbol<PluginCreateFn> = unsafe {
            library.get(b"_winux_plugin_create\0").map_err(|e| {
                PluginError::InitializationFailed(format!("Missing create function: {}", e))
            })?
        };

        let destroy_fn: PluginDestroyFn = unsafe {
            *library.get(b"_winux_plugin_destroy\0").map_err(|e| {
                PluginError::InitializationFailed(format!("Missing destroy function: {}", e))
            })?
        };

        // Create the plugin instance
        let plugin = unsafe { create_fn() };
        if plugin.is_null() {
            return Err(PluginError::InitializationFailed(
                "Plugin creation returned null".into(),
            ));
        }

        Ok(DynamicPlugin {
            _library: library,
            plugin,
            destroy_fn,
        })
    }

    /// Check if a plugin's API version is compatible
    fn check_api_compatibility(&self, plugin_api_version: &str) -> PluginResult<()> {
        let host_version = Version::parse(API_VERSION)
            .map_err(|e| PluginError::ConfigError(format!("Invalid host API version: {}", e)))?;

        let plugin_version = Version::parse(plugin_api_version).map_err(|e| {
            PluginError::ConfigError(format!("Invalid plugin API version: {}", e))
        })?;

        // Major version must match, plugin minor version must be <= host minor version
        if plugin_version.major != host_version.major {
            return Err(PluginError::VersionMismatch {
                expected: API_VERSION.to_string(),
                actual: plugin_api_version.to_string(),
            });
        }

        if plugin_version.minor > host_version.minor {
            return Err(PluginError::VersionMismatch {
                expected: API_VERSION.to_string(),
                actual: plugin_api_version.to_string(),
            });
        }

        Ok(())
    }

    /// Reload a plugin (for hot reload)
    pub fn reload_plugin<P: AsRef<Path>>(&self, plugin_dir: P) -> PluginResult<LoadedPlugin> {
        // Simply load again - the old instance should be dropped by the caller
        self.load_plugin(plugin_dir)
    }

    /// Get a cached manifest
    pub fn get_manifest(&self, plugin_id: &str) -> Option<PluginManifest> {
        self.manifests.read().get(plugin_id).cloned()
    }

    /// List all search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = PluginLoader::new();
        assert!(loader.search_paths.is_empty());
    }

    #[test]
    fn test_add_search_path() {
        let mut loader = PluginLoader::new();
        loader.add_search_path("/test/path");
        assert_eq!(loader.search_paths.len(), 1);

        // Adding same path again should not duplicate
        loader.add_search_path("/test/path");
        assert_eq!(loader.search_paths.len(), 1);
    }

    #[test]
    fn test_api_compatibility() {
        let loader = PluginLoader::new();

        // Same version should be compatible
        assert!(loader.check_api_compatibility(API_VERSION).is_ok());

        // Different major version should fail
        assert!(loader.check_api_compatibility("2.0.0").is_err());
    }
}
