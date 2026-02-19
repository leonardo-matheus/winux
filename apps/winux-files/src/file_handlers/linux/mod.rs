//! Linux file format handlers
//!
//! This module handles Linux-specific file formats including:
//! - Debian packages (.deb)
//! - RPM packages (.rpm)
//! - AppImage executables
//! - Flatpak packages
//! - Snap packages
//! - Shared libraries (.so)

pub mod deb;
pub mod rpm;
pub mod appimage;
pub mod flatpak;
pub mod snap;
pub mod so;
