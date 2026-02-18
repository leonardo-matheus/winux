//! Winux Files - Modern File Manager for Winux OS
//!
//! This library provides the core functionality for the Winux Files application.
//!
//! ## Features
//!
//! - Grid and list view modes
//! - Sidebar with favorites and devices
//! - Full file operations (copy, move, delete, rename)
//! - Thumbnail previews
//! - Search functionality
//! - Native archive support (zip, tar, rar, 7z, etc.)

pub mod app;
pub mod archive;
pub mod archive_dialog;
pub mod config;
pub mod file_ops;
pub mod file_view;
pub mod sidebar;

pub use app::{AppInput, AppModel, AppOutput, ClipboardContents, SortBy};
pub use archive::{
    ArchiveEntry, ArchiveFormat, ArchiveInfo, ArchiveManager, ArchiveOperation,
    ArchiveProgress, CompressionLevel, CompressOptions, ExtractOptions,
};
pub use archive_dialog::{
    CompressDialog, CompressDialogInput, CompressDialogOutput,
    ExtractDialog, ExtractDialogInput, ExtractDialogOutput,
    PreviewDialog, PreviewDialogInput, PreviewDialogOutput,
};
pub use config::Config;
pub use file_ops::{FileOperation, FileOperationInput, FileOperationMsg, OperationProgress};
pub use file_view::{FileEntry, FileView, FileViewInput, FileViewOutput, ViewMode};
pub use sidebar::{Sidebar, SidebarInput, SidebarLocation, SidebarOutput};
