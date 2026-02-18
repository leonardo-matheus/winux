//! Winux Image - Native image viewer for Winux OS
//!
//! A modern, GPU-accelerated image viewer with support for multiple formats,
//! smooth zoom/pan, metadata display, and slideshow capabilities.

pub mod config;
pub mod metadata;
pub mod thumbnail;
pub mod viewer;

pub use config::ImageConfig;
pub use metadata::MetadataPanel;
pub use thumbnail::ThumbnailStrip;
pub use viewer::ImageViewer;

/// Supported image formats
pub const SUPPORTED_FORMATS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp", "svg", "ico",
    "heic", "heif", "avif", "raw", "cr2", "nef", "arw", "dng",
];

/// MIME types for supported formats
pub const SUPPORTED_MIME_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/bmp",
    "image/tiff",
    "image/webp",
    "image/svg+xml",
    "image/x-icon",
    "image/heic",
    "image/heif",
    "image/avif",
    "image/x-raw",
    "image/x-canon-cr2",
    "image/x-nikon-nef",
    "image/x-sony-arw",
    "image/x-adobe-dng",
];

/// Check if a file extension is supported
pub fn is_supported_extension(ext: &str) -> bool {
    SUPPORTED_FORMATS.contains(&ext.to_lowercase().as_str())
}

/// Check if a MIME type is supported
pub fn is_supported_mime(mime: &str) -> bool {
    SUPPORTED_MIME_TYPES.contains(&mime)
}
