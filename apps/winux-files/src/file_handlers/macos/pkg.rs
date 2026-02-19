//! macOS Package (.pkg) installer handler
//!
//! Provides functionality to:
//! - Get information about PKG installers
//! - Extract PKG contents
//! - View package contents

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
};
use std::path::Path;

/// PKG file format types
#[derive(Debug, Clone)]
pub enum PkgFormat {
    FlatPackage,    // Modern .pkg (xar archive)
    BundlePackage,  // Legacy .pkg (directory)
    Unknown,
}

/// Get information about a PKG file
pub fn get_pkg_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("macOS Installer Package");

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Determine package format
    let format = if path.is_dir() {
        PkgFormat::BundlePackage
    } else {
        // Check for xar header
        if let Ok(header) = crate::file_handlers::common::read_file_header(path, 4) {
            if &header == b"xar!" {
                PkgFormat::FlatPackage
            } else {
                PkgFormat::Unknown
            }
        } else {
            PkgFormat::Unknown
        }
    };

    match &format {
        PkgFormat::FlatPackage => info.add_property("Format", "Flat Package (xar)"),
        PkgFormat::BundlePackage => info.add_property("Format", "Bundle Package (legacy)"),
        PkgFormat::Unknown => info.add_property("Format", "Unknown"),
    }

    // Try xar for flat packages
    if matches!(format, PkgFormat::FlatPackage) && command_exists("xar") {
        if let Ok(output) = run_command("xar", &["-t", "-f", path_str]) {
            let files: Vec<&str> = output.lines().take(20).collect();
            if !files.is_empty() {
                info.add_property("Contents", &files.join("\n"));
            }

            // Count total files
            let total = output.lines().count();
            info.add_property("Total Items", &total.to_string());
        }
    }

    // Try 7z as fallback
    if command_exists("7z") {
        if let Ok(output) = run_command("7z", &["l", path_str]) {
            // Parse 7z output for file count
            let file_lines: Vec<&str> = output.lines()
                .filter(|l| l.len() > 20 && !l.starts_with('-') && !l.contains("Type ="))
                .collect();

            if file_lines.len() > 2 {
                // Get approximate file count
                info.add_property("Files (approx)", &(file_lines.len() - 2).to_string());
            }
        }
    }

    // Try to detect if it contains a bundle
    if let Ok(contents) = list_pkg_contents(path) {
        let has_app = contents.iter().any(|f| f.ends_with(".app") || f.contains(".app/"));
        if has_app {
            info.add_property("Contains App Bundle", "Yes");
        }
    }

    Ok(info)
}

/// List contents of a PKG file
pub fn list_pkg_contents(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try xar for flat packages
    if command_exists("xar") {
        if let Ok(output) = run_command("xar", &["-t", "-f", path_str]) {
            return Ok(output.lines().map(|s| s.to_string()).collect());
        }
    }

    // Try 7z
    if command_exists("7z") {
        if let Ok(output) = run_command("7z", &["l", path_str]) {
            let files: Vec<String> = output.lines()
                .filter(|l| l.len() > 50)
                .filter_map(|l| {
                    // 7z output format: Date Time Attr Size Compressed Name
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    parts.last().map(|s| s.to_string())
                })
                .collect();
            return Ok(files);
        }
    }

    Err(FileHandlerError::NotSupported(
        "No tool available to list PKG contents (install xar or 7z)".to_string()
    ))
}

/// Extract PKG contents
pub fn extract_pkg(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    // Try xar first (for flat packages)
    if command_exists("xar") {
        if let Ok(output) = run_command("xar", &["-x", "-f", path_str, "-C", output_dir_str]) {
            // Now extract any Payload files (which are usually cpio.gz)
            extract_payloads(&output_dir)?;
            return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
        }
    }

    // Try 7z as fallback
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;

        // Try to extract nested archives
        extract_payloads(&output_dir)?;

        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try pkgutil on macOS
    if command_exists("pkgutil") {
        let output = run_command("pkgutil", &[
            "--expand", path_str, output_dir_str
        ])?;
        return Ok(format!("Expanded to {}\n{}", output_dir.display(), output));
    }

    Err(FileHandlerError::NotSupported(
        "PKG extraction requires xar, 7z, or pkgutil".to_string()
    ))
}

/// Extract Payload files from an expanded PKG
fn extract_payloads(dir: &Path) -> FileHandlerResult<()> {
    use std::fs;

    if !dir.is_dir() {
        return Ok(());
    }

    // Look for Payload files
    for entry in fs::read_dir(dir)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == "Payload" || name.ends_with(".pkg") {
            // Check if it's a cpio or gzip archive
            if path.is_file() {
                let payload_dir = dir.join(format!("{}_contents", name));
                std::fs::create_dir_all(&payload_dir)?;

                // Try to extract with cpio
                if command_exists("cpio") && command_exists("gzip") {
                    // gunzip + cpio
                    let gunzip_path = dir.join(format!("{}.cpio", name));
                    if let Ok(_) = run_command("gzip", &[
                        "-d", "-k", "-f", path.to_str().unwrap_or("")
                    ]) {
                        let _ = run_command("cpio", &[
                            "-i", "-d",
                            "-F", gunzip_path.to_str().unwrap_or(""),
                            "-D", payload_dir.to_str().unwrap_or(""),
                        ]);
                    }
                }

                // Try 7z as fallback
                if command_exists("7z") {
                    let _ = run_command("7z", &[
                        "x", "-y",
                        &format!("-o{}", payload_dir.display()),
                        path.to_str().unwrap_or("")
                    ]);
                }
            }
        }

        // Recurse into subdirectories
        if path.is_dir() {
            let _ = extract_payloads(&path);
        }
    }

    Ok(())
}

/// Get package identifier from Distribution file
pub fn get_package_identifier(path: &Path) -> FileHandlerResult<Option<String>> {
    // First extract the Distribution file if present
    let parent = path.parent().unwrap_or(Path::new("/tmp"));
    let temp_dir = parent.join(".pkg_temp");

    // Try to read Distribution from the package
    if command_exists("xar") {
        std::fs::create_dir_all(&temp_dir)?;
        if let Ok(_) = run_command("xar", &[
            "-x", "-f", path.to_str().unwrap_or(""),
            "-C", temp_dir.to_str().unwrap_or(""),
            "Distribution"
        ]) {
            let dist_file = temp_dir.join("Distribution");
            if dist_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&dist_file) {
                    // Look for pkg-ref id
                    if let Some(start) = content.find("identifier=\"") {
                        let after_start = &content[start + 12..];
                        if let Some(end) = after_start.find('"') {
                            let _ = std::fs::remove_dir_all(&temp_dir);
                            return Ok(Some(after_start[..end].to_string()));
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&temp_dir);
        }
    }

    Ok(None)
}
