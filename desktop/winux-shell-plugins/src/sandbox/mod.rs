//! Sandbox module
//!
//! Provides permission management and process isolation for plugins.

pub mod permissions;
pub mod isolation;

pub use permissions::{Permission, PermissionSet, PermissionRequest};
pub use isolation::{SandboxConfig, SandboxedProcess};
