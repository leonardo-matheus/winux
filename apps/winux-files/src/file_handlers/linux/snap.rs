//! Snap package file handler
//!
//! Provides functionality to:
//! - Get information about Snap packages
//! - Install Snap applications

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header,
};
use std::path::Path;

/// Snap file magic (squashfs)
const SQUASHFS_MAGIC: &[u8] = b"hsqs";

/// Verify file is a valid Snap package
pub fn is_snap_file(path: &Path) -> FileHandlerResult<bool> {
    let header = read_file_header(path, 4)?;
    Ok(header == SQUASHFS_MAGIC)
}

/// Get information about a Snap package
pub fn get_snap_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Snap Package");

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Verify it's a snap file
    if !is_snap_file(path).unwrap_or(false) {
        info.add_property("Warning", "File may not be a valid Snap package");
    }

    // Try snap command for info
    if command_exists("snap") {
        // Try to get info from the file
        if let Ok(output) = run_command("snap", &["info", "--verbose", path_str]) {
            for line in output.lines() {
                if let Some(colon) = line.find(':') {
                    let key = line[..colon].trim();
                    let value = line[colon + 1..].trim();
                    if !value.is_empty() && !key.is_empty() {
                        info.add_property(key, value);
                    }
                }
            }
        }
    }

    // Try unsquashfs to extract metadata
    if command_exists("unsquashfs") {
        // List contents
        if let Ok(output) = run_command("unsquashfs", &["-l", path_str]) {
            // Look for snap.yaml
            let has_snap_yaml = output.lines().any(|l| l.contains("snap.yaml"));
            info.add_property("Has snap.yaml", if has_snap_yaml { "Yes" } else { "No" });

            // Count files
            let file_count = output.lines()
                .filter(|l| !l.is_empty() && !l.starts_with("Parallel"))
                .count();
            info.add_property("Files", &file_count.to_string());
        }

        // Try to extract and read snap.yaml
        let temp_dir = std::env::temp_dir().join(format!("snap_info_{}", std::process::id()));
        if std::fs::create_dir_all(&temp_dir).is_ok() {
            if run_command("unsquashfs", &[
                "-d", temp_dir.to_str().unwrap_or(""),
                "-e", "meta/snap.yaml",
                path_str
            ]).is_ok() {
                let snap_yaml = temp_dir.join("meta").join("snap.yaml");
                if let Ok(content) = std::fs::read_to_string(&snap_yaml) {
                    // Parse YAML-like content
                    for line in content.lines() {
                        if !line.starts_with(' ') && !line.starts_with('\t') {
                            if let Some(colon) = line.find(':') {
                                let key = line[..colon].trim();
                                let value = line[colon + 1..].trim();
                                if !value.is_empty() && !key.is_empty() {
                                    match key {
                                        "name" => info.add_property("Name", value),
                                        "version" => info.add_property("Version", value),
                                        "summary" => info.add_property("Summary", value),
                                        "description" => info.add_property("Description", value),
                                        "grade" => info.add_property("Grade", value),
                                        "confinement" => info.add_property("Confinement", value),
                                        "base" => info.add_property("Base", value),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&temp_dir);
        }
    } else {
        // Try with file command
        if command_exists("file") {
            if let Ok(output) = run_command("file", &[path_str]) {
                if let Some(desc) = output.split(':').nth(1) {
                    info.add_property("File Type", desc.trim());
                }
            }
        }

        info.add_property("Note", "Install unsquashfs for detailed package information");
    }

    // Check if snapd is available
    info.add_property("Snap Daemon", if command_exists("snap") { "Available" } else { "Not installed" });

    Ok(info)
}

/// Install a Snap package
pub fn install_snap(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if !command_exists("snap") {
        return Err(FileHandlerError::NotSupported(
            "Snap is not installed (snapd required)".to_string()
        ));
    }

    // Install with dangerous flag for local snaps
    let output = run_command("pkexec", &["snap", "install", "--dangerous", path_str])?;
    Ok(format!("Installing snap:\n{}", output))
}

/// Install a Snap from the store
pub fn install_from_store(snap_name: &str) -> FileHandlerResult<String> {
    if !command_exists("snap") {
        return Err(FileHandlerError::NotSupported(
            "Snap is not installed".to_string()
        ));
    }

    let output = run_command("pkexec", &["snap", "install", snap_name])?;
    Ok(format!("Installing {}:\n{}", snap_name, output))
}

/// List installed snaps
pub fn list_installed() -> FileHandlerResult<Vec<String>> {
    if !command_exists("snap") {
        return Err(FileHandlerError::NotSupported(
            "Snap is not installed".to_string()
        ));
    }

    let output = run_command("snap", &["list"])?;
    Ok(output.lines()
        .skip(1) // Skip header
        .filter_map(|l| l.split_whitespace().next())
        .map(|s| s.to_string())
        .collect())
}

/// Get information about an installed snap
pub fn get_installed_info(snap_name: &str) -> FileHandlerResult<String> {
    if !command_exists("snap") {
        return Err(FileHandlerError::NotSupported(
            "Snap is not installed".to_string()
        ));
    }

    run_command("snap", &["info", snap_name])
}

/// Run a snap application
pub fn run_snap(snap_name: &str) -> FileHandlerResult<String> {
    if !command_exists("snap") {
        return Err(FileHandlerError::NotSupported(
            "Snap is not installed".to_string()
        ));
    }

    let child = std::process::Command::new("snap")
        .args(["run", snap_name])
        .spawn()
        .map_err(|e| FileHandlerError::CommandFailed(format!("Failed to run: {}", e)))?;

    Ok(format!("Started {} (PID: {})", snap_name, child.id()))
}

/// Extract snap contents
pub fn extract_snap(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;

    // Use unsquashfs
    if command_exists("unsquashfs") {
        let output = run_command("unsquashfs", &[
            "-d", output_dir.to_str().unwrap_or("."),
            path_str
        ])?;
        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try 7z
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir.display()),
            path_str
        ])?;
        return Ok(format!("Extracted with 7z to {}\n{}", output_dir.display(), output));
    }

    Err(FileHandlerError::NotSupported(
        "Snap extraction requires unsquashfs or 7z".to_string()
    ))
}
