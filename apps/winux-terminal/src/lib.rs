//! Winux Terminal - GPU-accelerated Terminal Emulator for Winux OS
//!
//! This library provides the core functionality for the Winux Terminal application.

pub mod config;
pub mod tabs;
pub mod terminal;
pub mod themes;

pub use config::Config;
pub use tabs::TabManager;
pub use terminal::TerminalWidget;
pub use themes::{Theme, ThemeManager};
