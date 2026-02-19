//! Flatpak package file handler
//!
//! Provides functionality to:
//! - Get information about Flatpak packages
//! - Install Flatpak applications

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
};
use std::fs;
use std::path::Path;

/// Get information about a Flatpak file
pub fn get_flatpak_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let file_type = match extension.as_str() {
        "flatpak" => "Flatpak Bundle",
        "flatpakref" => "Flatpak Reference",
        _ => "Flatpak File",
    };

    let mut info = FileInfo::new(path)?.with_type(file_type);

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    match extension.as_str() {
        "flatpakref" => {
            // Parse flatpakref file (INI-like format)
            if let Ok(content) = fs::read_to_string(path) {
                for line in content.lines() {
                    if let Some(eq_pos) = line.find('=') {
                        let key = line[..eq_pos].trim();
                        let value = line[eq_pos + 1..].trim();

                        match key {
                            "Name" => info.add_property("Name", value),
                            "Branch" => info.add_property("Branch", value),
                            "Url" => info.add_property("Repository URL", value),
                            "RuntimeRepo" => info.add_property("Runtime Repository", value),
                            "Title" => info.add_property("Title", value),
                            "Comment" => info.add_property("Description", value),
                            "Icon" => info.add_property("Icon URL", value),
                            "GPGKey" => info.add_property("GPG Key", "(present)"),
                            _ => {}
                        }
                    }
                }
            }
        }
        "flatpak" => {
            // Bundle file - try to get info using flatpak
            if command_exists("flatpak") {
                // Get bundle info
                if let Ok(output) = run_command("flatpak", &["info", "--file", path_str]) {
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

                // Try to get metadata
                if let Ok(output) = run_command("flatpak", &["info", "--show-metadata", "--file", path_str]) {
                    // Parse first few lines for context
                    let preview: Vec<&str> = output.lines().take(10).collect();
                    if !preview.is_empty() {
                        info.add_property("Metadata Preview", &preview.join("\n"));
                    }
                }
            } else {
                info.add_property("Note", "Install flatpak for detailed bundle information");

                // Try to get basic info using file
                if command_exists("file") {
                    if let Ok(output) = run_command("file", &[path_str]) {
                        if let Some(desc) = output.split(':').nth(1) {
                            info.add_property("File Type", desc.trim());
                        }
                    }
                }
            }
        }
        _ => {}
    }

    // Check if flatpak is available
    info.add_property("Flatpak Available", if command_exists("flatpak") { "Yes" } else { "No" });

    Ok(info)
}

/// Install a Flatpak application
pub fn install_flatpak(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if !command_exists("flatpak") {
        return Err(FileHandlerError::NotSupported(
            "Flatpak is not installed".to_string()
        ));
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "flatpakref" => {
            // Install from reference file
            let output = run_command("flatpak", &["install", "-y", path_str])?;
            Ok(format!("Installing from flatpakref:\n{}", output))
        }
        "flatpak" => {
            // Install from bundle
            let output = run_command("flatpak", &["install", "-y", "--bundle", path_str])?;
            Ok(format!("Installing from bundle:\n{}", output))
        }
        _ => Err(FileHandlerError::NotSupported(
            "Unknown flatpak file type".to_string()
        )),
    }
}

/// Add a Flatpak remote repository
pub fn add_remote(name: &str, url: &str) -> FileHandlerResult<String> {
    if !command_exists("flatpak") {
        return Err(FileHandlerError::NotSupported(
            "Flatpak is not installed".to_string()
        ));
    }

    let output = run_command("flatpak", &["remote-add", "--if-not-exists", name, url])?;
    Ok(format!("Added remote {}:\n{}", name, output))
}

/// List installed Flatpak applications
pub fn list_installed() -> FileHandlerResult<Vec<String>> {
    if !command_exists("flatpak") {
        return Err(FileHandlerError::NotSupported(
            "Flatpak is not installed".to_string()
        ));
    }

    let output = run_command("flatpak", &["list", "--columns=application"])?;
    Ok(output.lines().skip(1).map(|s| s.to_string()).collect())
}

/// Get information about an installed Flatpak app
pub fn get_installed_info(app_id: &str) -> FileHandlerResult<String> {
    if !command_exists("flatpak") {
        return Err(FileHandlerError::NotSupported(
            "Flatpak is not installed".to_string()
        ));
    }

    run_command("flatpak", &["info", app_id])
}

/// Run a Flatpak application
pub fn run_flatpak(app_id: &str) -> FileHandlerResult<String> {
    if !command_exists("flatpak") {
        return Err(FileHandlerError::NotSupported(
            "Flatpak is not installed".to_string()
        ));
    }

    let child = std::process::Command::new("flatpak")
        .args(["run", app_id])
        .spawn()
        .map_err(|e| FileHandlerError::CommandFailed(format!("Failed to run: {}", e)))?;

    Ok(format!("Started {} (PID: {})", app_id, child.id()))
}

/// Parse a flatpakref file
pub fn parse_flatpakref(path: &Path) -> FileHandlerResult<FlatpakRef> {
    let content = fs::read_to_string(path)?;
    let mut flatpakref = FlatpakRef::default();

    for line in content.lines() {
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim().to_string();

            match key {
                "Name" => flatpakref.name = Some(value),
                "Branch" => flatpakref.branch = Some(value),
                "Url" => flatpakref.url = Some(value),
                "RuntimeRepo" => flatpakref.runtime_repo = Some(value),
                "Title" => flatpakref.title = Some(value),
                "Comment" => flatpakref.comment = Some(value),
                "Icon" => flatpakref.icon = Some(value),
                "GPGKey" => flatpakref.gpg_key = Some(value),
                "IsRuntime" => flatpakref.is_runtime = value == "true",
                _ => {}
            }
        }
    }

    Ok(flatpakref)
}

/// Parsed flatpakref file
#[derive(Debug, Default)]
pub struct FlatpakRef {
    pub name: Option<String>,
    pub branch: Option<String>,
    pub url: Option<String>,
    pub runtime_repo: Option<String>,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub icon: Option<String>,
    pub gpg_key: Option<String>,
    pub is_runtime: bool,
}
