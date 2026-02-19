//! Debian package (.deb) file handler
//!
//! Provides functionality to:
//! - Extract package information
//! - List package contents
//! - Install/extract packages

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header,
};
use std::path::Path;

/// Deb file magic (ar archive)
const DEB_MAGIC: &[u8] = b"!<arch>";

/// Package control information
#[derive(Debug, Default)]
pub struct DebInfo {
    pub package: Option<String>,
    pub version: Option<String>,
    pub architecture: Option<String>,
    pub maintainer: Option<String>,
    pub description: Option<String>,
    pub depends: Option<String>,
    pub installed_size: Option<String>,
    pub section: Option<String>,
    pub priority: Option<String>,
    pub homepage: Option<String>,
}

/// Verify file is a valid deb package
pub fn is_deb_file(path: &Path) -> FileHandlerResult<bool> {
    let header = read_file_header(path, 7)?;
    Ok(header == DEB_MAGIC)
}

/// Get information about a deb package
pub fn get_deb_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Debian Package");

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Verify it's a deb file
    if !is_deb_file(path).unwrap_or(false) {
        info.add_property("Warning", "File does not appear to be a valid deb package");
    }

    // Try dpkg-deb for detailed info
    if command_exists("dpkg-deb") {
        // Get package info
        if let Ok(output) = run_command("dpkg-deb", &["--info", path_str]) {
            for line in output.lines() {
                if let Some((key, value)) = parse_control_line(line) {
                    info.add_property(&key, &value);
                }
            }
        }

        // Get file count
        if let Ok(output) = run_command("dpkg-deb", &["--contents", path_str]) {
            let file_count = output.lines().count();
            info.add_property("Files", &file_count.to_string());
        }
    } else if command_exists("ar") {
        // Fallback: use ar to extract control
        let temp_dir = std::env::temp_dir().join(format!("deb_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir)?;

        if let Ok(_) = run_command("ar", &[
            "x", path_str,
            "--output", temp_dir.to_str().unwrap_or("")
        ]) {
            // Try to extract control.tar.gz or control.tar.xz
            let control_archives = ["control.tar.gz", "control.tar.xz", "control.tar.zst"];
            for archive in &control_archives {
                let archive_path = temp_dir.join(archive);
                if archive_path.exists() {
                    // Extract control file
                    if let Ok(output) = run_command("tar", &[
                        "-xOf", archive_path.to_str().unwrap_or(""),
                        "./control"
                    ]) {
                        for line in output.lines() {
                            if let Some((key, value)) = parse_control_line(line) {
                                info.add_property(&key, &value);
                            }
                        }
                    }
                    break;
                }
            }
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    } else {
        info.add_property("Note", "Install dpkg for detailed package information");
    }

    Ok(info)
}

/// Parse a control file line (Key: Value format)
fn parse_control_line(line: &str) -> Option<(String, String)> {
    if line.starts_with(' ') || line.starts_with('\t') {
        // Continuation line, skip for now
        return None;
    }

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

/// List contents of a deb package
pub fn list_deb_contents(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if command_exists("dpkg-deb") {
        let output = run_command("dpkg-deb", &["--contents", path_str])?;
        let files: Vec<String> = output.lines()
            .filter_map(|line| {
                // Format: drwxr-xr-x root/root 0 date time ./path
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts.last().map(|s| s.to_string())
            })
            .collect();
        return Ok(files);
    }

    Err(FileHandlerError::NotSupported(
        "Listing deb contents requires dpkg-deb".to_string()
    ))
}

/// Install a deb package
pub fn install_deb(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try apt first (handles dependencies)
    if command_exists("apt") {
        // Need sudo for installation
        let output = run_command("pkexec", &["apt", "install", "-y", path_str])?;
        return Ok(format!("Installed via apt:\n{}", output));
    }

    // Try dpkg directly
    if command_exists("dpkg") {
        let output = run_command("pkexec", &["dpkg", "-i", path_str])?;
        return Ok(format!("Installed via dpkg:\n{}", output));
    }

    // Try gdebi
    if command_exists("gdebi") {
        let output = run_command("pkexec", &["gdebi", "-n", path_str])?;
        return Ok(format!("Installed via gdebi:\n{}", output));
    }

    Err(FileHandlerError::NotSupported(
        "No package manager found (apt, dpkg, or gdebi required)".to_string()
    ))
}

/// Extract contents of a deb package
pub fn extract_deb(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    // Try dpkg-deb
    if command_exists("dpkg-deb") {
        let output = run_command("dpkg-deb", &["-x", path_str, output_dir_str])?;

        // Also extract control files
        let control_dir = output_dir.join("DEBIAN");
        std::fs::create_dir_all(&control_dir)?;
        let _ = run_command("dpkg-deb", &["-e", path_str, control_dir.to_str().unwrap_or("")]);

        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Manual extraction using ar and tar
    if command_exists("ar") && command_exists("tar") {
        // Extract ar archive
        let temp_dir = output_dir.join("_temp");
        std::fs::create_dir_all(&temp_dir)?;

        run_command("ar", &["x", path_str, "--output", temp_dir.to_str().unwrap_or("")])?;

        // Extract data archive
        let data_archives = ["data.tar.gz", "data.tar.xz", "data.tar.zst", "data.tar.bz2"];
        for archive in &data_archives {
            let archive_path = temp_dir.join(archive);
            if archive_path.exists() {
                run_command("tar", &[
                    "-xf", archive_path.to_str().unwrap_or(""),
                    "-C", output_dir_str
                ])?;
                break;
            }
        }

        // Extract control archive to DEBIAN
        let control_dir = output_dir.join("DEBIAN");
        std::fs::create_dir_all(&control_dir)?;
        let control_archives = ["control.tar.gz", "control.tar.xz", "control.tar.zst"];
        for archive in &control_archives {
            let archive_path = temp_dir.join(archive);
            if archive_path.exists() {
                run_command("tar", &[
                    "-xf", archive_path.to_str().unwrap_or(""),
                    "-C", control_dir.to_str().unwrap_or("")
                ])?;
                break;
            }
        }

        // Cleanup temp
        let _ = std::fs::remove_dir_all(&temp_dir);

        return Ok(format!("Extracted to {}", output_dir.display()));
    }

    Err(FileHandlerError::NotSupported(
        "Deb extraction requires dpkg-deb or ar+tar".to_string()
    ))
}

/// Get package dependencies
pub fn get_dependencies(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if command_exists("dpkg-deb") {
        let output = run_command("dpkg-deb", &["-f", path_str, "Depends"])?;
        let deps: Vec<String> = output
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        return Ok(deps);
    }

    Err(FileHandlerError::NotSupported(
        "Getting dependencies requires dpkg-deb".to_string()
    ))
}
