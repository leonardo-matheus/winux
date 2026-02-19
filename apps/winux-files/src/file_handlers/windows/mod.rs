//! Windows file format handlers
//!
//! This module handles Windows-specific file formats including:
//! - PE executables (.exe, .dll)
//! - MSI installers
//! - LNK shortcuts
//! - Registry files
//! - Batch and PowerShell scripts

pub mod pe;
pub mod msi;
pub mod lnk;
pub mod reg;
pub mod scripts;
