//! ISO 9660 disk image file handler
//!
//! Provides functionality to:
//! - Get information about ISO images
//! - Mount ISO images
//! - Extract ISO contents

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header, format_size,
};
use std::path::Path;

/// ISO 9660 identifier at offset 0x8001
const ISO_IDENTIFIER: &[u8] = b"CD001";

/// ISO volume descriptor types
const VOLUME_DESCRIPTOR_PRIMARY: u8 = 1;
const VOLUME_DESCRIPTOR_TERMINATOR: u8 = 255;

/// Verify file is a valid ISO image
pub fn is_iso_file(path: &Path) -> FileHandlerResult<bool> {
    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    let mut file = File::open(path)?;

    // ISO 9660 identifier is at offset 0x8001 (32769)
    file.seek(SeekFrom::Start(0x8001))?;

    let mut identifier = [0u8; 5];
    file.read_exact(&mut identifier)?;

    Ok(&identifier == ISO_IDENTIFIER)
}

/// Read ISO volume descriptor
fn read_volume_descriptor(path: &Path) -> FileHandlerResult<IsoVolumeInfo> {
    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    let mut file = File::open(path)?;
    let mut info = IsoVolumeInfo::default();

    // Volume descriptors start at sector 16 (0x8000)
    let mut offset = 0x8000u64;

    loop {
        file.seek(SeekFrom::Start(offset))?;

        let mut descriptor = [0u8; 2048];
        if file.read_exact(&mut descriptor).is_err() {
            break;
        }

        let descriptor_type = descriptor[0];

        // Check for CD001 magic
        if &descriptor[1..6] != ISO_IDENTIFIER {
            break;
        }

        match descriptor_type {
            VOLUME_DESCRIPTOR_PRIMARY => {
                // System Identifier (offset 8, 32 bytes)
                info.system_id = read_string(&descriptor[8..40]);

                // Volume Identifier (offset 40, 32 bytes)
                info.volume_id = read_string(&descriptor[40..72]);

                // Volume Space Size (offset 80, 8 bytes - both endian)
                let space_size = u32::from_le_bytes([
                    descriptor[80], descriptor[81], descriptor[82], descriptor[83]
                ]);
                info.volume_size = space_size as u64 * 2048; // Sectors to bytes

                // Volume Set Identifier (offset 190, 128 bytes)
                info.volume_set_id = read_string(&descriptor[190..318]);

                // Publisher Identifier (offset 318, 128 bytes)
                info.publisher_id = read_string(&descriptor[318..446]);

                // Application Identifier (offset 574, 128 bytes)
                info.application_id = read_string(&descriptor[574..702]);

                // Creation Date (offset 813, 17 bytes)
                info.creation_date = read_date(&descriptor[813..830]);
            }
            VOLUME_DESCRIPTOR_TERMINATOR => {
                break;
            }
            _ => {}
        }

        offset += 2048;
    }

    Ok(info)
}

/// Read a string from ISO descriptor (space-padded ASCII)
fn read_string(data: &[u8]) -> Option<String> {
    let s: String = data.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Read an ISO date string
fn read_date(data: &[u8]) -> Option<String> {
    if data.len() < 17 || data[0] == 0 || data[0] == b' ' {
        return None;
    }

    // Format: YYYYMMDDHHMMSScc (where cc is hundredths of second)
    let year = std::str::from_utf8(&data[0..4]).ok()?;
    let month = std::str::from_utf8(&data[4..6]).ok()?;
    let day = std::str::from_utf8(&data[6..8]).ok()?;
    let hour = std::str::from_utf8(&data[8..10]).ok()?;
    let minute = std::str::from_utf8(&data[10..12]).ok()?;
    let second = std::str::from_utf8(&data[12..14]).ok()?;

    Some(format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second))
}

/// ISO volume information
#[derive(Debug, Default)]
pub struct IsoVolumeInfo {
    pub system_id: Option<String>,
    pub volume_id: Option<String>,
    pub volume_set_id: Option<String>,
    pub publisher_id: Option<String>,
    pub application_id: Option<String>,
    pub creation_date: Option<String>,
    pub volume_size: u64,
}

/// Get information about an ISO image
pub fn get_iso_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("ISO 9660 Disk Image");

    // Verify it's an ISO
    if !is_iso_file(path).unwrap_or(false) {
        info.add_property("Warning", "File may not be a valid ISO 9660 image");
    }

    // Read volume descriptor
    if let Ok(vol_info) = read_volume_descriptor(path) {
        if let Some(id) = vol_info.volume_id {
            info.add_property("Volume Label", &id);
        }
        if let Some(system) = vol_info.system_id {
            info.add_property("System", &system);
        }
        if let Some(publisher) = vol_info.publisher_id {
            info.add_property("Publisher", &publisher);
        }
        if let Some(app) = vol_info.application_id {
            info.add_property("Application", &app);
        }
        if let Some(date) = vol_info.creation_date {
            info.add_property("Created", &date);
        }
        if vol_info.volume_size > 0 {
            info.add_property("Volume Size", &format_size(vol_info.volume_size));
        }
    }

    let path_str = path.to_str().unwrap_or("");

    // Try isoinfo for more details
    if command_exists("isoinfo") {
        if let Ok(output) = run_command("isoinfo", &["-d", "-i", path_str]) {
            for line in output.lines() {
                if line.contains("Joliet") && line.contains("found") {
                    info.add_property("Joliet Extension", "Yes");
                }
                if line.contains("Rock Ridge") {
                    info.add_property("Rock Ridge Extension", "Yes");
                }
            }
        }

        // Count files
        if let Ok(output) = run_command("isoinfo", &["-f", "-i", path_str]) {
            let file_count = output.lines().count();
            info.add_property("Files", &file_count.to_string());
        }
    }

    // Check for bootable
    if let Ok(output) = run_command("file", &[path_str]) {
        if output.contains("bootable") {
            info.add_property("Bootable", "Yes");
        }
    }

    Ok(info)
}

/// Mount an ISO image
pub fn mount_iso(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Create mount point
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("iso");
    let mount_point = format!("/tmp/iso_mount_{}_{}", stem, std::process::id());

    std::fs::create_dir_all(&mount_point)?;

    // Try udisksctl (preferred, no root needed for most cases)
    if command_exists("udisksctl") {
        if let Ok(output) = run_command("udisksctl", &["loop-setup", "-f", path_str]) {
            // Parse loop device from output
            if let Some(line) = output.lines().find(|l| l.contains("/dev/loop")) {
                if let Some(dev) = line.split_whitespace()
                    .find(|s| s.starts_with("/dev/loop")) {
                    // Mount the loop device
                    if let Ok(mount_output) = run_command("udisksctl", &["mount", "-b", dev]) {
                        return Ok(format!("Mounted ISO:\n{}\n{}", output, mount_output));
                    }
                }
            }
            return Ok(format!("Loop device created:\n{}", output));
        }
    }

    // Try mount command (requires root)
    if command_exists("mount") {
        let output = run_command("pkexec", &[
            "mount", "-o", "loop,ro",
            path_str, &mount_point
        ])?;
        return Ok(format!("Mounted at {}\n{}", mount_point, output));
    }

    // Try fuseiso (FUSE-based, no root needed)
    if command_exists("fuseiso") {
        let output = run_command("fuseiso", &[path_str, &mount_point])?;
        return Ok(format!("Mounted with FUSE at {}\n{}", mount_point, output));
    }

    Err(FileHandlerError::NotSupported(
        "ISO mounting requires udisksctl, mount, or fuseiso".to_string()
    ))
}

/// Extract ISO contents
pub fn extract_iso(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    // Try 7z (most reliable cross-platform)
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;
        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try bsdtar
    if command_exists("bsdtar") {
        let output = run_command("bsdtar", &[
            "-xf", path_str,
            "-C", output_dir_str
        ])?;
        return Ok(format!("Extracted with bsdtar to {}\n{}", output_dir.display(), output));
    }

    // Try xorriso
    if command_exists("xorriso") {
        let output = run_command("xorriso", &[
            "-osirrox", "on",
            "-indev", path_str,
            "-extract", "/", output_dir_str
        ])?;
        return Ok(format!("Extracted with xorriso to {}\n{}", output_dir.display(), output));
    }

    // Try isoinfo with manual extraction
    if command_exists("isoinfo") {
        // Get file list
        let files = run_command("isoinfo", &["-f", "-i", path_str])?;

        for file in files.lines() {
            let file = file.trim();
            if file.is_empty() || file.ends_with('/') {
                continue;
            }

            let output_path = output_dir.join(file.trim_start_matches('/'));
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Extract file
            if let Ok(content) = run_command("isoinfo", &[
                "-i", path_str,
                "-x", file
            ]) {
                std::fs::write(&output_path, content.as_bytes())?;
            }
        }

        return Ok(format!("Extracted to {}", output_dir.display()));
    }

    Err(FileHandlerError::NotSupported(
        "ISO extraction requires 7z, bsdtar, xorriso, or isoinfo".to_string()
    ))
}

/// List contents of an ISO image
pub fn list_contents(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try isoinfo
    if command_exists("isoinfo") {
        let output = run_command("isoinfo", &["-f", "-i", path_str])?;
        return Ok(output.lines().map(|s| s.to_string()).collect());
    }

    // Try 7z
    if command_exists("7z") {
        let output = run_command("7z", &["l", "-slt", path_str])?;
        let files: Vec<String> = output.lines()
            .filter(|l| l.starts_with("Path = "))
            .map(|l| l.strip_prefix("Path = ").unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .collect();
        return Ok(files);
    }

    Err(FileHandlerError::NotSupported(
        "Listing ISO contents requires isoinfo or 7z".to_string()
    ))
}
