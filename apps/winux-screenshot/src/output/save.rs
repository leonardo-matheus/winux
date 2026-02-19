//! Save functionality for screenshots

use anyhow::{Result, anyhow};
use image::{DynamicImage, ImageFormat};
use std::path::{Path, PathBuf};

use crate::capture::{get_screenshots_dir, generate_filename};

/// Supported save formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaveFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
    Bmp,
    Gif,
}

impl SaveFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            SaveFormat::Png => "png",
            SaveFormat::Jpeg => "jpg",
            SaveFormat::Webp => "webp",
            SaveFormat::Bmp => "bmp",
            SaveFormat::Gif => "gif",
        }
    }

    /// Get the ImageFormat for this save format
    pub fn image_format(&self) -> ImageFormat {
        match self {
            SaveFormat::Png => ImageFormat::Png,
            SaveFormat::Jpeg => ImageFormat::Jpeg,
            SaveFormat::Webp => ImageFormat::WebP,
            SaveFormat::Bmp => ImageFormat::Bmp,
            SaveFormat::Gif => ImageFormat::Gif,
        }
    }

    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            SaveFormat::Png => "image/png",
            SaveFormat::Jpeg => "image/jpeg",
            SaveFormat::Webp => "image/webp",
            SaveFormat::Bmp => "image/bmp",
            SaveFormat::Gif => "image/gif",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            SaveFormat::Png => "PNG (Lossless)",
            SaveFormat::Jpeg => "JPEG (Lossy)",
            SaveFormat::Webp => "WebP (Modern)",
            SaveFormat::Bmp => "BMP (Uncompressed)",
            SaveFormat::Gif => "GIF (Limited colors)",
        }
    }

    /// All available formats
    pub fn all() -> &'static [SaveFormat] {
        &[
            SaveFormat::Png,
            SaveFormat::Jpeg,
            SaveFormat::Webp,
            SaveFormat::Bmp,
            SaveFormat::Gif,
        ]
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<SaveFormat> {
        match ext.to_lowercase().as_str() {
            "png" => Some(SaveFormat::Png),
            "jpg" | "jpeg" => Some(SaveFormat::Jpeg),
            "webp" => Some(SaveFormat::Webp),
            "bmp" => Some(SaveFormat::Bmp),
            "gif" => Some(SaveFormat::Gif),
            _ => None,
        }
    }
}

/// Save screenshot to a file
pub fn save_screenshot(
    image: &DynamicImage,
    path: &Path,
    format: SaveFormat,
) -> Result<PathBuf> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Save with appropriate format
    image.save_with_format(path, format.image_format())?;

    Ok(path.to_path_buf())
}

/// Save screenshot to default location with auto-generated name
pub fn save_screenshot_auto(image: &DynamicImage, format: SaveFormat) -> Result<PathBuf> {
    let screenshots_dir = get_screenshots_dir();
    let base_filename = generate_filename();

    // Replace .png extension with appropriate one
    let filename = base_filename.replace(".png", &format!(".{}", format.extension()));
    let path = screenshots_dir.join(&filename);

    save_screenshot(image, &path, format)
}

/// Save options
#[derive(Debug, Clone)]
pub struct SaveOptions {
    /// Output format
    pub format: SaveFormat,
    /// JPEG quality (0-100)
    pub jpeg_quality: u8,
    /// Whether to include annotations
    pub include_annotations: bool,
    /// Whether to apply crop
    pub apply_crop: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            format: SaveFormat::Png,
            jpeg_quality: 90,
            include_annotations: true,
            apply_crop: true,
        }
    }
}

/// Validate a save path
pub fn validate_save_path(path: &Path) -> Result<()> {
    // Check if path has a valid extension
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow!("Path must have a file extension"))?;

    if SaveFormat::from_extension(extension).is_none() {
        return Err(anyhow!("Unsupported file format: {}", extension));
    }

    // Check if parent directory exists or can be created
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Cannot create directory: {}", e))?;
        }
    }

    // Check if we can write to the directory
    if let Some(parent) = path.parent() {
        let test_path = parent.join(".winux-screenshot-write-test");
        std::fs::write(&test_path, b"test")
            .map_err(|_| anyhow!("Cannot write to directory"))?;
        std::fs::remove_file(&test_path).ok();
    }

    Ok(())
}

/// Get suggested filename for a screenshot
pub fn suggest_filename(format: SaveFormat) -> String {
    let now = chrono::Local::now();
    format!(
        "Screenshot_{}.{}",
        now.format("%Y-%m-%d_%H-%M-%S"),
        format.extension()
    )
}
