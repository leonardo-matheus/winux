//! macOS file format handlers
//!
//! This module handles macOS-specific file formats including:
//! - DMG disk images
//! - Application bundles (.app)
//! - PKG installers
//! - Property lists (.plist)
//! - Icon files (.icns)
//! - Dynamic libraries (.dylib)

pub mod dmg;
pub mod app;
pub mod pkg;
pub mod plist;
pub mod icns;
pub mod dylib;
