//! RPM package file handler
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

/// RPM file magic
const RPM_MAGIC: [u8; 4] = [0xed, 0xab, 0xee, 0xdb];

/// Verify file is a valid RPM package
pub fn is_rpm_file(path: &Path) -> FileHandlerResult<bool> {
    let header = read_file_header(path, 4)?;
    Ok(header == RPM_MAGIC)
}

/// Get information about an RPM package
pub fn get_rpm_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("RPM Package");

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Verify it's an RPM file
    if !is_rpm_file(path).unwrap_or(false) {
        info.add_property("Warning", "File does not appear to be a valid RPM package");
    }

    // Try rpm command for detailed info
    if command_exists("rpm") {
        // Basic info
        let queries = [
            ("%{NAME}", "Name"),
            ("%{VERSION}", "Version"),
            ("%{RELEASE}", "Release"),
            ("%{ARCH}", "Architecture"),
            ("%{SUMMARY}", "Summary"),
            ("%{LICENSE}", "License"),
            ("%{URL}", "URL"),
            ("%{PACKAGER}", "Packager"),
            ("%{SIZE}", "Installed Size"),
            ("%{BUILDTIME:date}", "Build Date"),
        ];

        for (query, label) in &queries {
            if let Ok(output) = run_command("rpm", &["-qp", &format!("--queryformat={}", query), path_str]) {
                let value = output.trim();
                if !value.is_empty() && value != "(none)" {
                    info.add_property(label, value);
                }
            }
        }

        // Get file count
        if let Ok(output) = run_command("rpm", &["-qpl", path_str]) {
            let file_count = output.lines().count();
            info.add_property("Files", &file_count.to_string());
        }

        // Get dependencies
        if let Ok(output) = run_command("rpm", &["-qpR", path_str]) {
            let deps: Vec<&str> = output.lines().take(10).collect();
            if !deps.is_empty() {
                info.add_property("Dependencies (sample)", &deps.join("\n"));
            }
        }
    } else if command_exists("rpminfo") {
        // Try rpminfo from rpm-tools
        if let Ok(output) = run_command("rpminfo", &[path_str]) {
            for line in output.lines() {
                if let Some(colon) = line.find(':') {
                    let key = line[..colon].trim();
                    let value = line[colon + 1..].trim();
                    if !value.is_empty() {
                        info.add_property(key, value);
                    }
                }
            }
        }
    } else {
        info.add_property("Note", "Install rpm for detailed package information");

        // Try basic info with cpio
        if command_exists("rpm2cpio") && command_exists("cpio") {
            info.add_property("Format", "RPM (can extract with rpm2cpio)");
        }
    }

    Ok(info)
}

/// List contents of an RPM package
pub fn list_rpm_contents(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try rpm
    if command_exists("rpm") {
        let output = run_command("rpm", &["-qpl", path_str])?;
        return Ok(output.lines().map(|s| s.to_string()).collect());
    }

    // Try rpm2cpio + cpio
    if command_exists("rpm2cpio") && command_exists("cpio") {
        // This is a bit tricky without shell pipes
        // We'd need to create a temp file
        let temp_cpio = std::env::temp_dir().join(format!("rpm_{}.cpio", std::process::id()));

        // Extract cpio
        if let Ok(cpio_data) = run_command("rpm2cpio", &[path_str]) {
            std::fs::write(&temp_cpio, cpio_data.as_bytes())?;

            if let Ok(output) = run_command("cpio", &["-t", "-F", temp_cpio.to_str().unwrap_or("")]) {
                let _ = std::fs::remove_file(&temp_cpio);
                return Ok(output.lines().map(|s| s.to_string()).collect());
            }
            let _ = std::fs::remove_file(&temp_cpio);
        }
    }

    Err(FileHandlerError::NotSupported(
        "Listing RPM contents requires rpm or rpm2cpio+cpio".to_string()
    ))
}

/// Install an RPM package
pub fn install_rpm(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try dnf first (Fedora/RHEL)
    if command_exists("dnf") {
        let output = run_command("pkexec", &["dnf", "install", "-y", path_str])?;
        return Ok(format!("Installed via dnf:\n{}", output));
    }

    // Try yum (older RHEL/CentOS)
    if command_exists("yum") {
        let output = run_command("pkexec", &["yum", "localinstall", "-y", path_str])?;
        return Ok(format!("Installed via yum:\n{}", output));
    }

    // Try zypper (openSUSE)
    if command_exists("zypper") {
        let output = run_command("pkexec", &["zypper", "install", "-y", path_str])?;
        return Ok(format!("Installed via zypper:\n{}", output));
    }

    // Try rpm directly (no dependency resolution)
    if command_exists("rpm") {
        let output = run_command("pkexec", &["rpm", "-ivh", path_str])?;
        return Ok(format!("Installed via rpm:\n{}", output));
    }

    Err(FileHandlerError::NotSupported(
        "No RPM package manager found (dnf, yum, zypper, or rpm required)".to_string()
    ))
}

/// Extract contents of an RPM package
pub fn extract_rpm(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    // Try rpm2cpio + cpio
    if command_exists("rpm2cpio") && command_exists("cpio") {
        // Extract cpio data
        let cpio_output = run_command("rpm2cpio", &[path_str])?;
        let temp_cpio = output_dir.join("_temp.cpio");
        std::fs::write(&temp_cpio, cpio_output.as_bytes())?;

        // Extract files
        let output = run_command("cpio", &[
            "-idmv",
            "-D", output_dir_str,
            "-F", temp_cpio.to_str().unwrap_or("")
        ])?;

        let _ = std::fs::remove_file(&temp_cpio);
        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try 7z
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;
        return Ok(format!("Extracted with 7z to {}\n{}", output_dir.display(), output));
    }

    // Try bsdtar
    if command_exists("bsdtar") {
        let output = run_command("bsdtar", &[
            "-xf", path_str,
            "-C", output_dir_str
        ])?;
        return Ok(format!("Extracted with bsdtar to {}\n{}", output_dir.display(), output));
    }

    Err(FileHandlerError::NotSupported(
        "RPM extraction requires rpm2cpio+cpio, 7z, or bsdtar".to_string()
    ))
}

/// Get package dependencies
pub fn get_dependencies(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if command_exists("rpm") {
        let output = run_command("rpm", &["-qpR", path_str])?;
        let deps: Vec<String> = output
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        return Ok(deps);
    }

    Err(FileHandlerError::NotSupported(
        "Getting dependencies requires rpm".to_string()
    ))
}

/// Get package scripts (pre/post install, etc.)
pub fn get_scripts(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if command_exists("rpm") {
        let output = run_command("rpm", &["-qp", "--scripts", path_str])?;
        return Ok(output);
    }

    Err(FileHandlerError::NotSupported(
        "Getting scripts requires rpm".to_string()
    ))
}
