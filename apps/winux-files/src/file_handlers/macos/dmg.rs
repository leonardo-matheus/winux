//! DMG (Apple Disk Image) file handler
//!
//! Provides functionality to:
//! - Get information about DMG files
//! - Mount DMG files
//! - Extract DMG contents

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists, read_file_header,
};
use std::path::Path;

/// DMG file signatures
const KOLY_MAGIC: &[u8] = b"koly"; // DMG trailer
const UDIF_MAGIC: &[u8] = &[0x78, 0x01]; // zlib compressed
const BZ2_MAGIC: &[u8] = b"BZ"; // bzip2 compressed

/// DMG compression types
#[derive(Debug, Clone)]
pub enum DmgCompression {
    None,
    Zlib,
    Bzip2,
    Lzfse,
    Lzma,
    Unknown,
}

impl DmgCompression {
    fn to_string(&self) -> &'static str {
        match self {
            DmgCompression::None => "Uncompressed",
            DmgCompression::Zlib => "Zlib",
            DmgCompression::Bzip2 => "Bzip2",
            DmgCompression::Lzfse => "LZFSE",
            DmgCompression::Lzma => "LZMA",
            DmgCompression::Unknown => "Unknown",
        }
    }
}

/// Check if file appears to be a DMG
pub fn is_dmg_file(path: &Path) -> bool {
    // Check file extension
    if let Some(ext) = path.extension() {
        if ext.to_string_lossy().to_lowercase() == "dmg" {
            return true;
        }
    }

    // Check for DMG magic bytes
    if let Ok(header) = read_file_header(path, 4) {
        // Check for compressed DMG headers
        if header.starts_with(UDIF_MAGIC) || header.starts_with(BZ2_MAGIC) {
            return true;
        }
    }

    false
}

/// Get information about a DMG file
pub fn get_dmg_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Apple Disk Image");

    // Try to detect compression type from header
    if let Ok(header) = read_file_header(path, 16) {
        let compression = if header.starts_with(UDIF_MAGIC) {
            DmgCompression::Zlib
        } else if header.starts_with(BZ2_MAGIC) {
            DmgCompression::Bzip2
        } else {
            DmgCompression::Unknown
        };
        info.add_property("Compression", compression.to_string());
    }

    // Try dmg2img for info
    if command_exists("dmg2img") {
        if let Ok(output) = run_command("dmg2img", &["-l", path.to_str().unwrap_or("")]) {
            // Parse partition info
            for line in output.lines() {
                if line.contains("partition") || line.contains("Partition") {
                    info.add_property("Partitions", line.trim());
                }
            }
        }
    }

    // Try 7z for listing contents
    if command_exists("7z") {
        if let Ok(output) = run_command("7z", &["l", path.to_str().unwrap_or("")]) {
            // Count files
            let file_count = output.lines()
                .filter(|l| l.contains("....A") || l.contains("D...."))
                .count();
            if file_count > 0 {
                info.add_property("Files (approx)", &file_count.to_string());
            }
        }
    }

    // Try file command for additional info
    if command_exists("file") {
        if let Ok(output) = run_command("file", &[path.to_str().unwrap_or("")]) {
            if let Some(desc) = output.split(':').nth(1) {
                info.add_property("File Type", desc.trim());
            }
        }
    }

    Ok(info)
}

/// Mount a DMG file
pub fn mount_dmg(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // On actual macOS, use hdiutil
    if command_exists("hdiutil") {
        let output = run_command("hdiutil", &["attach", path_str])?;
        return Ok(format!("Mounted DMG:\n{}", output));
    }

    // On Linux, try several approaches

    // 1. Try dmg2img + mount
    if command_exists("dmg2img") {
        let parent = path.parent().unwrap_or(Path::new("/tmp"));
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("dmg");
        let img_path = parent.join(format!("{}.img", stem));

        // Convert DMG to IMG
        run_command("dmg2img", &[
            path_str,
            img_path.to_str().unwrap_or("")
        ])?;

        // Create mount point
        let mount_point = format!("/tmp/dmg_mount_{}", std::process::id());
        std::fs::create_dir_all(&mount_point)?;

        // Try to mount
        if let Ok(output) = run_command("mount", &[
            "-o", "loop,ro",
            img_path.to_str().unwrap_or(""),
            &mount_point
        ]) {
            return Ok(format!("Mounted at {}\n{}", mount_point, output));
        }

        // Alternative: use udisksctl
        if command_exists("udisksctl") {
            let output = run_command("udisksctl", &[
                "loop-setup", "-f", img_path.to_str().unwrap_or("")
            ])?;
            return Ok(format!("Setup loop device:\n{}", output));
        }
    }

    // 2. Try 7z for extraction (not mounting, but better than nothing)
    if command_exists("7z") {
        return Err(FileHandlerError::NotSupported(
            "Direct mounting not available. Use extract function instead.".to_string()
        ));
    }

    Err(FileHandlerError::NotSupported(
        "DMG mounting requires hdiutil (macOS) or dmg2img (Linux)".to_string()
    ))
}

/// Extract contents of a DMG file
pub fn extract_dmg(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    // Try 7z first (most reliable cross-platform)
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;
        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try p7zip
    if command_exists("7za") {
        let output = run_command("7za", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;
        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try dmg2img then extract
    if command_exists("dmg2img") {
        let img_path = parent.join(format!("{}.img", stem));

        run_command("dmg2img", &[path_str, img_path.to_str().unwrap_or("")])?;

        // Now extract from the img
        if command_exists("7z") {
            let output = run_command("7z", &[
                "x", "-y",
                &format!("-o{}", output_dir_str),
                img_path.to_str().unwrap_or("")
            ])?;

            // Clean up intermediate img file
            let _ = std::fs::remove_file(&img_path);

            return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
        }
    }

    Err(FileHandlerError::NotSupported(
        "DMG extraction requires 7z or dmg2img".to_string()
    ))
}

/// Unmount a mounted DMG
pub fn unmount_dmg(mount_point: &str) -> FileHandlerResult<String> {
    // On macOS
    if command_exists("hdiutil") {
        let output = run_command("hdiutil", &["detach", mount_point])?;
        return Ok(format!("Unmounted:\n{}", output));
    }

    // On Linux
    if command_exists("umount") {
        let output = run_command("umount", &[mount_point])?;
        return Ok(format!("Unmounted:\n{}", output));
    }

    Err(FileHandlerError::NotSupported(
        "No unmount tool available".to_string()
    ))
}
