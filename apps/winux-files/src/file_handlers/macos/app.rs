//! macOS Application Bundle (.app) handler
//!
//! Provides functionality to:
//! - View information about .app bundles
//! - Browse bundle contents
//! - Extract Info.plist data

use crate::file_handlers::common::{FileHandlerError, FileHandlerResult, FileInfo};
use std::fs;
use std::path::Path;

/// Parsed Info.plist data (basic subset)
#[derive(Debug, Default)]
pub struct AppInfo {
    pub bundle_name: Option<String>,
    pub bundle_identifier: Option<String>,
    pub bundle_version: Option<String>,
    pub short_version: Option<String>,
    pub minimum_os_version: Option<String>,
    pub executable: Option<String>,
    pub icon_file: Option<String>,
    pub copyright: Option<String>,
    pub category: Option<String>,
}

/// Get information about a .app bundle
pub fn get_app_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("macOS Application Bundle");

    // Check if it's actually a directory (bundle)
    if !path.is_dir() {
        info.add_property("Warning", "Path is not a directory/bundle");
        return Ok(info);
    }

    // Look for Info.plist
    let info_plist = path.join("Contents").join("Info.plist");
    if info_plist.exists() {
        info.add_property("Info.plist", "Present");

        // Try to parse Info.plist
        if let Ok(app_info) = parse_info_plist(&info_plist) {
            if let Some(name) = &app_info.bundle_name {
                info.add_property("Bundle Name", name);
            }
            if let Some(id) = &app_info.bundle_identifier {
                info.add_property("Bundle ID", id);
            }
            if let Some(version) = &app_info.bundle_version {
                info.add_property("Build Version", version);
            }
            if let Some(short) = &app_info.short_version {
                info.add_property("Version", short);
            }
            if let Some(min_os) = &app_info.minimum_os_version {
                info.add_property("Minimum macOS", min_os);
            }
            if let Some(exec) = &app_info.executable {
                info.add_property("Executable", exec);
            }
            if let Some(copyright) = &app_info.copyright {
                info.add_property("Copyright", copyright);
            }
            if let Some(category) = &app_info.category {
                info.add_property("Category", category);
            }
        }
    } else {
        info.add_property("Info.plist", "Not found");
    }

    // Check for MacOS binary
    let macos_dir = path.join("Contents").join("MacOS");
    if macos_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&macos_dir) {
            let binaries: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if !binaries.is_empty() {
                info.add_property("Binaries", &binaries.join(", "));
            }
        }
    }

    // Check for Resources
    let resources_dir = path.join("Contents").join("Resources");
    if resources_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&resources_dir) {
            let count = entries.count();
            info.add_property("Resources", &format!("{} items", count));
        }

        // Check for localizations
        let localizations: Vec<String> = fs::read_dir(&resources_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .ends_with(".lproj")
                    })
                    .map(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .trim_end_matches(".lproj")
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !localizations.is_empty() {
            info.add_property("Localizations", &localizations.join(", "));
        }
    }

    // Check for Frameworks
    let frameworks_dir = path.join("Contents").join("Frameworks");
    if frameworks_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&frameworks_dir) {
            let frameworks: Vec<String> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .take(10)
                .collect();
            if !frameworks.is_empty() {
                info.add_property("Frameworks", &frameworks.join(", "));
            }
        }
    }

    // Check code signature
    let code_signature = path.join("Contents").join("_CodeSignature");
    info.add_property("Code Signed", if code_signature.is_dir() { "Yes" } else { "No" });

    Ok(info)
}

/// Parse Info.plist file (simple XML parsing)
fn parse_info_plist(path: &Path) -> FileHandlerResult<AppInfo> {
    let content = fs::read_to_string(path)?;

    let mut info = AppInfo::default();

    // Simple XML parsing for common keys
    info.bundle_name = extract_plist_string(&content, "CFBundleName");
    info.bundle_identifier = extract_plist_string(&content, "CFBundleIdentifier");
    info.bundle_version = extract_plist_string(&content, "CFBundleVersion");
    info.short_version = extract_plist_string(&content, "CFBundleShortVersionString");
    info.minimum_os_version = extract_plist_string(&content, "LSMinimumSystemVersion");
    info.executable = extract_plist_string(&content, "CFBundleExecutable");
    info.icon_file = extract_plist_string(&content, "CFBundleIconFile");
    info.copyright = extract_plist_string(&content, "NSHumanReadableCopyright");
    info.category = extract_plist_string(&content, "LSApplicationCategoryType");

    Ok(info)
}

/// Extract a string value from plist XML
fn extract_plist_string(content: &str, key: &str) -> Option<String> {
    // Look for <key>KEY</key> followed by <string>VALUE</string>
    let key_pattern = format!("<key>{}</key>", key);
    let key_pos = content.find(&key_pattern)?;

    // Find the string value after the key
    let after_key = &content[key_pos + key_pattern.len()..];
    let string_start = after_key.find("<string>")?;
    let value_start = string_start + "<string>".len();
    let after_string_start = &after_key[value_start..];
    let string_end = after_string_start.find("</string>")?;

    Some(after_string_start[..string_end].to_string())
}

/// Browse the contents of an app bundle
pub fn browse_app_bundle(path: &Path) -> FileHandlerResult<String> {
    if !path.is_dir() {
        return Err(FileHandlerError::NotSupported(
            "Not a valid app bundle directory".to_string()
        ));
    }

    let mut output = String::new();

    output.push_str(&format!("App Bundle: {}\n\n", path.display()));

    // List Contents directory
    let contents_dir = path.join("Contents");
    if contents_dir.is_dir() {
        output.push_str("Contents/\n");

        if let Ok(entries) = fs::read_dir(&contents_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                output.push_str(&format!("  {}{}\n", name, if is_dir { "/" } else { "" }));

                // List immediate children of subdirectories
                if is_dir {
                    if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                        for sub_entry in sub_entries.filter_map(|e| e.ok()).take(5) {
                            let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                            let sub_is_dir = sub_entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                            output.push_str(&format!("    {}{}\n", sub_name, if sub_is_dir { "/" } else { "" }));
                        }
                    }
                }
            }
        }
    } else {
        // Maybe it's a flat bundle
        output.push_str("(Non-standard bundle structure)\n");
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()).take(20) {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                output.push_str(&format!("  {}{}\n", name, if is_dir { "/" } else { "" }));
            }
        }
    }

    Ok(output)
}

/// Get the main executable path from an app bundle
pub fn get_executable_path(path: &Path) -> FileHandlerResult<String> {
    let info_plist = path.join("Contents").join("Info.plist");

    if info_plist.exists() {
        if let Ok(app_info) = parse_info_plist(&info_plist) {
            if let Some(exec) = app_info.executable {
                let exec_path = path.join("Contents").join("MacOS").join(&exec);
                if exec_path.exists() {
                    return Ok(exec_path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Fallback: look for any executable in MacOS directory
    let macos_dir = path.join("Contents").join("MacOS");
    if macos_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&macos_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    return Ok(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }

    Err(FileHandlerError::NotFound(
        "Could not find executable in app bundle".to_string()
    ))
}
