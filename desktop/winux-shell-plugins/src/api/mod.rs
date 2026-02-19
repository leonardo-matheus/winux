//! Plugin API modules
//!
//! This module provides APIs for plugins to extend various parts of Winux Shell.

pub mod panel;
pub mod notifications;
pub mod launcher;
pub mod settings;
pub mod commands;

// Re-export commonly used types
pub use panel::{PanelWidget, PanelPosition, WidgetSize, WidgetAction};
pub use notifications::{NotificationHandler, NotificationFilter};
pub use launcher::{LauncherProvider, SearchResult, SearchCategory};
pub use settings::{SettingsProvider, SettingsPage, SettingValue};
pub use commands::{CommandProvider, Command, CommandContext};
