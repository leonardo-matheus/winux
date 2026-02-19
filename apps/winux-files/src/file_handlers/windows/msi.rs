//! MSI (Windows Installer) file handler
//!
//! Provides functionality to:
//! - Extract information from MSI packages
//! - Install MSI packages using msitools
//! - Extract contents of MSI files

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
};
use std::path::Path;

/// MSI file magic bytes (OLE Compound Document)
const OLE_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Check if file is a valid MSI/OLE compound document
pub fn is_msi_file(path: &Path) -> FileHandlerResult<bool> {
    use crate::file_handlers::common::read_file_header;
    let header = read_file_header(path, 8)?;
    Ok(header == OLE_MAGIC)
}

/// Get information about an MSI file
pub fn get_msi_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Windows Installer Package");

    // Verify it's an MSI file
    if !is_msi_file(path).unwrap_or(false) {
        info.add_property("Warning", "File does not appear to be a valid MSI package");
    }

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try to get MSI info using msiinfo from msitools
    if command_exists("msiinfo") {
        // Get summary information
        if let Ok(output) = run_command("msiinfo", &["suminfo", path_str]) {
            for line in output.lines() {
                if let Some((key, value)) = parse_msi_property(line) {
                    info.add_property(&key, &value);
                }
            }
        }

        // Get tables
        if let Ok(output) = run_command("msiinfo", &["tables", path_str]) {
            let tables: Vec<&str> = output.lines().take(20).collect();
            if !tables.is_empty() {
                info.add_property("Tables", &tables.join(", "));
            }
        }
    } else {
        // Fallback: try using 7z to list contents
        if let Ok(output) = run_command("7z", &["l", path_str]) {
            let file_count = output.lines()
                .filter(|l| l.contains("."))
                .count();
            info.add_property("Files (approx)", &file_count.to_string());
        }

        info.add_property("Note", "Install msitools for detailed MSI information");
    }

    Ok(info)
}

/// Parse MSI property line from msiinfo output
fn parse_msi_property(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() == 2 {
        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();
        if !key.is_empty() && !value.is_empty() {
            return Some((key, value));
        }
    }
    None
}

/// Install an MSI package
pub fn install_msi(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // First, try native Linux installation with msiexec (Wine)
    if command_exists("msiexec") {
        let output = run_command("msiexec", &["/i", path_str])?;
        return Ok(format!("Installing via Wine msiexec...\n{}", output));
    }

    // Try with wine msiexec
    if command_exists("wine") {
        let output = run_command("wine", &["msiexec", "/i", path_str])?;
        return Ok(format!("Installing via Wine...\n{}", output));
    }

    Err(FileHandlerError::NotSupported(
        "No MSI installation tool found. Install Wine to run MSI installers.".to_string()
    ))
}

/// Extract contents of an MSI file
pub fn extract_msi(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Create output directory
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    // Try msiextract from msitools
    if command_exists("msiextract") {
        let output = run_command("msiextract", &[
            "-C", output_dir_str,
            path_str
        ])?;
        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try 7z as fallback
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;
        return Ok(format!("Extracted with 7z to {}\n{}", output_dir.display(), output));
    }

    // Try cabextract as another fallback
    if command_exists("cabextract") {
        let output = run_command("cabextract", &[
            "-d", output_dir_str,
            path_str
        ])?;
        return Ok(format!("Extracted CAB contents to {}\n{}", output_dir.display(), output));
    }

    Err(FileHandlerError::NotSupported(
        "No MSI extraction tool found. Install msitools, 7z, or cabextract.".to_string()
    ))
}

/// List files in an MSI package
pub fn list_msi_contents(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try msiinfo
    if command_exists("msiinfo") {
        if let Ok(output) = run_command("msiinfo", &["export", path_str, "File"]) {
            let files: Vec<String> = output
                .lines()
                .skip(1) // Skip header
                .filter_map(|line| {
                    line.split('\t').nth(2).map(|s| s.to_string())
                })
                .collect();
            return Ok(files);
        }
    }

    // Try 7z
    if command_exists("7z") {
        if let Ok(output) = run_command("7z", &["l", path_str]) {
            let files: Vec<String> = output
                .lines()
                .filter(|line| line.len() > 50 && !line.starts_with('-'))
                .filter_map(|line| {
                    // 7z output format varies, try to extract filename
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.last().map(|s| s.to_string())
                })
                .collect();
            return Ok(files);
        }
    }

    Err(FileHandlerError::NotSupported(
        "No tool available to list MSI contents".to_string()
    ))
}
