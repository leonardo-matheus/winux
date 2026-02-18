//! Winux Files - Modern File Manager for Winux OS
//!
//! This library provides the core functionality for the Winux Files application.

pub mod app;
pub mod config;
pub mod file_ops;
pub mod file_view;
pub mod sidebar;

pub use app::{AppInput, AppModel, AppOutput, ClipboardContents, SortBy};
pub use config::Config;
pub use file_ops::{FileOperation, FileOperationInput, FileOperationMsg, OperationProgress};
pub use file_view::{FileEntry, FileView, FileViewInput, FileViewOutput, ViewMode};
pub use sidebar::{Sidebar, SidebarInput, SidebarLocation, SidebarOutput};
