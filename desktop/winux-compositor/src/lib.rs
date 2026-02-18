//! Winux Compositor - Wayland compositor for Winux OS
//!
//! This crate implements a modern Wayland compositor using Smithay.
//! It provides the core desktop experience for Winux OS with support
//! for multiple outputs, window management, and input handling.

pub mod config;
pub mod input;
pub mod rendering;
pub mod state;

pub use config::CompositorConfig;
pub use input::InputHandler;
pub use rendering::Renderer;
pub use state::WinuxState;

/// Version of the compositor
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the compositor
pub const NAME: &str = "winux-compositor";
