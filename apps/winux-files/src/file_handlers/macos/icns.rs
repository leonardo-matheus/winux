//! Apple Icon Image (.icns) file handler
//!
//! Provides functionality to:
//! - Parse ICNS files and extract icon information
//! - Extract individual icon sizes
//! - Convert to other formats

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header,
};
use std::path::Path;

/// ICNS file magic
const ICNS_MAGIC: &[u8] = b"icns";

/// Known ICNS icon types and their sizes
const ICON_TYPES: &[(&str, &str)] = &[
    ("ICON", "32x32 1-bit"),
    ("ICN#", "32x32 1-bit with mask"),
    ("icm#", "16x12 1-bit with mask"),
    ("icm4", "16x12 4-bit"),
    ("icm8", "16x12 8-bit"),
    ("ics#", "16x16 1-bit with mask"),
    ("ics4", "16x16 4-bit"),
    ("ics8", "16x16 8-bit"),
    ("is32", "16x16 24-bit"),
    ("s8mk", "16x16 8-bit mask"),
    ("icl4", "32x32 4-bit"),
    ("icl8", "32x32 8-bit"),
    ("il32", "32x32 24-bit"),
    ("l8mk", "32x32 8-bit mask"),
    ("ich#", "48x48 1-bit with mask"),
    ("ich4", "48x48 4-bit"),
    ("ich8", "48x48 8-bit"),
    ("ih32", "48x48 24-bit"),
    ("h8mk", "48x48 8-bit mask"),
    ("it32", "128x128 24-bit"),
    ("t8mk", "128x128 8-bit mask"),
    ("icp4", "16x16 JPEG2000/PNG"),
    ("icp5", "32x32 JPEG2000/PNG"),
    ("icp6", "64x64 JPEG2000/PNG"),
    ("ic07", "128x128 JPEG2000/PNG"),
    ("ic08", "256x256 JPEG2000/PNG"),
    ("ic09", "512x512 JPEG2000/PNG"),
    ("ic10", "1024x1024 JPEG2000/PNG (Retina)"),
    ("ic11", "32x32 Retina (16@2x)"),
    ("ic12", "64x64 Retina (32@2x)"),
    ("ic13", "256x256 Retina (128@2x)"),
    ("ic14", "512x512 Retina (256@2x)"),
];

/// Parsed ICNS icon entry
#[derive(Debug, Clone)]
pub struct IconEntry {
    pub icon_type: String,
    pub description: String,
    pub size: u32,
    pub data_offset: u32,
}

/// Parsed ICNS file
#[derive(Debug)]
pub struct IcnsFile {
    pub file_size: u32,
    pub entries: Vec<IconEntry>,
}

/// Parse ICNS file header and entries
pub fn parse_icns(path: &Path) -> FileHandlerResult<IcnsFile> {
    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    let mut file = File::open(path)?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header)?;

    // Check magic
    if &header[0..4] != ICNS_MAGIC {
        return Err(FileHandlerError::Parse("Not a valid ICNS file".to_string()));
    }

    let file_size = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);

    let mut entries = Vec::new();
    let mut offset: u32 = 8;

    while offset < file_size {
        let mut entry_header = [0u8; 8];
        if file.read_exact(&mut entry_header).is_err() {
            break;
        }

        let icon_type = String::from_utf8_lossy(&entry_header[0..4]).to_string();
        let entry_size = u32::from_be_bytes([
            entry_header[4], entry_header[5],
            entry_header[6], entry_header[7]
        ]);

        // Find description
        let description = ICON_TYPES
            .iter()
            .find(|(t, _)| *t == icon_type)
            .map(|(_, d)| d.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        entries.push(IconEntry {
            icon_type,
            description,
            size: entry_size,
            data_offset: offset,
        });

        // Move to next entry
        offset += entry_size;
        if entry_size > 8 {
            file.seek(SeekFrom::Current((entry_size - 8) as i64))?;
        }
    }

    Ok(IcnsFile {
        file_size,
        entries,
    })
}

/// Get information about an ICNS file
pub fn get_icns_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Apple Icon Image");

    match parse_icns(path) {
        Ok(icns) => {
            info.add_property("File Size", &format!("{} bytes", icns.file_size));
            info.add_property("Icon Count", &icns.entries.len().to_string());

            // List available sizes
            let sizes: Vec<String> = icns.entries
                .iter()
                .map(|e| format!("{}: {}", e.icon_type, e.description))
                .collect();
            info.add_property("Available Icons", &sizes.join("\n"));

            // Identify largest icon
            let largest = icns.entries
                .iter()
                .max_by_key(|e| e.size);
            if let Some(largest) = largest {
                info.add_property("Largest Icon", &format!("{} ({} bytes)",
                    largest.description, largest.size));
            }
        }
        Err(e) => {
            info.add_property("Parse Error", &e.to_string());
        }
    }

    Ok(info)
}

/// Extract icons from ICNS file to PNG
pub fn extract_icons(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("icons");
    let output_dir = parent.join(format!("{}_icons", stem));

    std::fs::create_dir_all(&output_dir)?;

    // Try icns2png (from libicns)
    if command_exists("icns2png") {
        let output = run_command("icns2png", &[
            "-x",
            "-o", output_dir.to_str().unwrap_or("."),
            path_str
        ])?;
        return Ok(format!("Extracted icons to {}\n{}", output_dir.display(), output));
    }

    // Try iconutil (macOS only)
    if command_exists("iconutil") {
        // iconutil requires conversion to iconset
        let iconset_dir = parent.join(format!("{}.iconset", stem));
        let output = run_command("iconutil", &[
            "-c", "iconset",
            "-o", iconset_dir.to_str().unwrap_or("."),
            path_str
        ])?;
        return Ok(format!("Extracted icons to {}\n{}", iconset_dir.display(), output));
    }

    // Try sips (macOS)
    if command_exists("sips") {
        let png_path = output_dir.join("icon.png");
        let output = run_command("sips", &[
            "-s", "format", "png",
            path_str,
            "--out", png_path.to_str().unwrap_or("")
        ])?;
        return Ok(format!("Converted to PNG: {}\n{}", png_path.display(), output));
    }

    // Manual extraction for embedded PNGs
    let icns = parse_icns(path)?;
    let mut extracted = 0;

    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    let mut file = File::open(path)?;

    for entry in &icns.entries {
        // Modern icon types (ic07+) contain PNG or JPEG2000 data
        if entry.icon_type.starts_with("ic") {
            file.seek(SeekFrom::Start((entry.data_offset + 8) as u64))?;
            let mut data = vec![0u8; (entry.size - 8) as usize];
            file.read_exact(&mut data)?;

            // Check if it's a PNG (magic: 0x89 PNG)
            if data.len() > 8 && data[0] == 0x89 && &data[1..4] == b"PNG" {
                let output_path = output_dir.join(format!("{}_{}.png", stem, entry.icon_type));
                std::fs::write(&output_path, &data)?;
                extracted += 1;
            }
        }
    }

    if extracted > 0 {
        Ok(format!("Extracted {} PNG icons to {}", extracted, output_dir.display()))
    } else {
        Err(FileHandlerError::NotSupported(
            "No PNG icons found. Install libicns (icns2png) for full extraction.".to_string()
        ))
    }
}

/// Convert ICNS to PNG (largest size)
pub fn icns_to_png(path: &Path, output: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;
    let output_str = output.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid output path".to_string()))?;

    // Try sips (macOS)
    if command_exists("sips") {
        return run_command("sips", &[
            "-s", "format", "png",
            path_str,
            "--out", output_str
        ]);
    }

    // Try icns2png
    if command_exists("icns2png") {
        return run_command("icns2png", &["-x", "-o", output_str, path_str]);
    }

    // Try convert (ImageMagick)
    if command_exists("convert") {
        return run_command("convert", &[path_str, output_str]);
    }

    Err(FileHandlerError::NotSupported(
        "ICNS conversion requires sips (macOS), icns2png, or ImageMagick".to_string()
    ))
}

/// Get the best (largest) icon from ICNS
pub fn get_best_icon_type(icns: &IcnsFile) -> Option<&IconEntry> {
    // Prefer modern high-resolution icons
    let preferred_types = ["ic10", "ic09", "ic14", "ic08", "ic13", "ic07"];

    for pref in &preferred_types {
        if let Some(entry) = icns.entries.iter().find(|e| e.icon_type == *pref) {
            return Some(entry);
        }
    }

    // Fall back to largest by size
    icns.entries.iter().max_by_key(|e| e.size)
}
