//! Winux Shell Plugin System
//!
//! A comprehensive plugin system for extending Winux Shell functionality.
//!
//! # Features
//!
//! - **Panel Widgets**: Add custom indicators, buttons, and menus to the panel
//! - **Notification Handlers**: Process and respond to system notifications
//! - **Launcher Providers**: Add custom search results to the launcher
//! - **Settings Pages**: Extend the settings application
//! - **Command Palette**: Register custom commands
//! - **Keyboard Shortcuts**: Define global keyboard shortcuts
//! - **Context Menu Items**: Add items to file/desktop context menus
//! - **File Previews**: Provide custom file preview implementations
//!
//! # Example
//!
//! ```rust,ignore
//! use winux_shell_plugins::prelude::*;
//!
//! #[derive(Default)]
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn metadata(&self) -> PluginMetadata {
//!         PluginMetadata {
//!             id: "com.example.my-plugin".into(),
//!             name: "My Plugin".into(),
//!             version: Version::new(1, 0, 0),
//!             ..Default::default()
//!         }
//!     }
//!
//!     fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
//!         log::info!("My plugin initialized!");
//!         Ok(())
//!     }
//! }
//!
//! declare_plugin!(MyPlugin, MyPlugin::default);
//! ```

pub mod plugin;
pub mod loader;
pub mod manager;
pub mod api;
pub mod sandbox;

// Re-exports
pub use plugin::{
    Plugin, PluginMetadata, PluginContext, PluginResult, PluginError,
    PluginState, PluginCapability, PluginManifest,
};
pub use loader::{PluginLoader, DynamicPlugin, LoadedPlugin};
pub use manager::{PluginManager, PluginEvent, PluginHandle};

/// Plugin API version for compatibility checking
pub const API_VERSION: &str = "1.0.0";

/// Prelude module for common imports
pub mod prelude {
    pub use crate::plugin::{
        Plugin, PluginMetadata, PluginContext, PluginResult, PluginError,
        PluginState, PluginCapability, PluginManifest,
    };
    pub use crate::api::panel::{PanelWidget, PanelPosition, WidgetSize};
    pub use crate::api::notifications::{NotificationHandler, NotificationFilter};
    pub use crate::api::launcher::{LauncherProvider, SearchResult, SearchCategory};
    pub use crate::api::settings::{SettingsProvider, SettingsPage, SettingValue};
    pub use crate::api::commands::{CommandProvider, Command, CommandContext};
    pub use crate::sandbox::permissions::{Permission, PermissionSet};
    pub use crate::declare_plugin;
    pub use semver::Version;
}

/// Macro to declare a plugin entry point
///
/// This macro creates the necessary FFI functions for dynamic loading.
///
/// # Example
///
/// ```rust,ignore
/// use winux_shell_plugins::prelude::*;
///
/// struct MyPlugin;
/// impl Plugin for MyPlugin { /* ... */ }
///
/// declare_plugin!(MyPlugin, MyPlugin::default);
/// ```
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:expr) => {
        #[no_mangle]
        pub extern "C" fn _winux_plugin_api_version() -> *const std::os::raw::c_char {
            concat!($crate::API_VERSION, "\0").as_ptr() as *const std::os::raw::c_char
        }

        #[no_mangle]
        pub extern "C" fn _winux_plugin_create() -> *mut dyn $crate::Plugin {
            let constructor: fn() -> $plugin_type = $constructor;
            let plugin = constructor();
            let boxed: Box<dyn $crate::Plugin> = Box::new(plugin);
            Box::into_raw(boxed)
        }

        #[no_mangle]
        pub extern "C" fn _winux_plugin_destroy(plugin: *mut dyn $crate::Plugin) {
            if !plugin.is_null() {
                unsafe {
                    let _ = Box::from_raw(plugin);
                }
            }
        }
    };
}
