//! Property List (.plist) file handler
//!
//! Supports both XML and binary plist formats.
//! Provides viewing and editing capabilities.

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Plist file format
#[derive(Debug, Clone)]
pub enum PlistFormat {
    Xml,
    Binary,
    Json,
    Unknown,
}

impl PlistFormat {
    fn to_string(&self) -> &'static str {
        match self {
            PlistFormat::Xml => "XML",
            PlistFormat::Binary => "Binary",
            PlistFormat::Json => "JSON",
            PlistFormat::Unknown => "Unknown",
        }
    }
}

/// Plist value types
#[derive(Debug, Clone)]
pub enum PlistValue {
    String(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Date(String),
    Data(Vec<u8>),
    Array(Vec<PlistValue>),
    Dictionary(HashMap<String, PlistValue>),
}

/// Detect plist format from file
pub fn detect_format(path: &Path) -> FileHandlerResult<PlistFormat> {
    let header = crate::file_handlers::common::read_file_header(path, 8)?;

    // Binary plist magic: "bplist"
    if header.starts_with(b"bplist") {
        return Ok(PlistFormat::Binary);
    }

    // XML plist starts with <?xml or <plist
    let content_start = String::from_utf8_lossy(&header);
    if content_start.contains("<?xml") || content_start.contains("<plist") || content_start.contains("<!DOC") {
        return Ok(PlistFormat::Xml);
    }

    // JSON plist starts with { or [
    if header[0] == b'{' || header[0] == b'[' {
        return Ok(PlistFormat::Json);
    }

    Ok(PlistFormat::Unknown)
}

/// Get information about a plist file
pub fn get_plist_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Property List");

    // Detect format
    let format = detect_format(path)?;
    info.add_property("Format", format.to_string());

    // Parse and show preview based on format
    match format {
        PlistFormat::Xml => {
            // Parse XML plist
            if let Ok(content) = fs::read_to_string(path) {
                // Count keys
                let key_count = content.matches("<key>").count();
                info.add_property("Keys", &key_count.to_string());

                // Extract some sample keys
                let keys: Vec<String> = extract_xml_keys(&content)
                    .into_iter()
                    .take(10)
                    .collect();
                if !keys.is_empty() {
                    info.add_property("Sample Keys", &keys.join(", "));
                }
            }
        }
        PlistFormat::Binary => {
            // Try to convert to XML for reading
            if command_exists("plutil") {
                if let Ok(output) = run_command("plutil", &["-p", path.to_str().unwrap_or("")]) {
                    // Count lines as approximation of entries
                    let entry_count = output.lines().count();
                    info.add_property("Entries (approx)", &entry_count.to_string());

                    // Show first few lines as preview
                    let preview: Vec<&str> = output.lines().take(10).collect();
                    info.add_property("Preview", &preview.join("\n"));
                }
            } else if command_exists("plistutil") {
                // plistutil from libplist
                if let Ok(output) = run_command("plistutil", &["-i", path.to_str().unwrap_or("")]) {
                    let key_count = output.matches("<key>").count();
                    info.add_property("Keys", &key_count.to_string());
                }
            } else {
                info.add_property("Note", "Install plutil or plistutil to view binary plists");
            }
        }
        PlistFormat::Json => {
            if let Ok(content) = fs::read_to_string(path) {
                let key_count = content.matches('"').count() / 2; // Rough estimate
                info.add_property("Keys (approx)", &key_count.to_string());
            }
        }
        PlistFormat::Unknown => {
            info.add_property("Warning", "Unknown plist format");
        }
    }

    Ok(info)
}

/// Extract keys from XML plist content
fn extract_xml_keys(content: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut search_start = 0;

    while let Some(key_start) = content[search_start..].find("<key>") {
        let absolute_start = search_start + key_start + 5; // 5 = len("<key>")
        if let Some(key_end) = content[absolute_start..].find("</key>") {
            let key = &content[absolute_start..absolute_start + key_end];
            keys.push(key.to_string());
            search_start = absolute_start + key_end + 6; // 6 = len("</key>")
        } else {
            break;
        }
    }

    keys
}

/// View plist content in human-readable format
pub fn view_plist_content(path: &Path) -> FileHandlerResult<String> {
    let format = detect_format(path)?;

    match format {
        PlistFormat::Xml => {
            // XML is already readable
            fs::read_to_string(path).map_err(|e| FileHandlerError::Io(e))
        }
        PlistFormat::Binary => {
            // Convert to readable format
            if command_exists("plutil") {
                run_command("plutil", &["-p", path.to_str().unwrap_or("")])
            } else if command_exists("plistutil") {
                run_command("plistutil", &["-i", path.to_str().unwrap_or("")])
            } else {
                Err(FileHandlerError::NotSupported(
                    "Binary plist viewing requires plutil or plistutil".to_string()
                ))
            }
        }
        PlistFormat::Json => {
            fs::read_to_string(path).map_err(|e| FileHandlerError::Io(e))
        }
        PlistFormat::Unknown => {
            Err(FileHandlerError::NotSupported("Unknown plist format".to_string()))
        }
    }
}

/// Convert plist between formats
pub fn convert_plist(path: &Path, target_format: PlistFormat) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let format_arg = match target_format {
        PlistFormat::Xml => "xml1",
        PlistFormat::Binary => "binary1",
        PlistFormat::Json => "json",
        PlistFormat::Unknown => return Err(FileHandlerError::NotSupported(
            "Cannot convert to unknown format".to_string()
        )),
    };

    // Use plutil on macOS
    if command_exists("plutil") {
        run_command("plutil", &["-convert", format_arg, path_str])
    } else if command_exists("plistutil") {
        // plistutil only converts between binary and XML
        match target_format {
            PlistFormat::Xml => run_command("plistutil", &[
                "-i", path_str,
                "-o", path_str,
                "-f", "xml"
            ]),
            PlistFormat::Binary => run_command("plistutil", &[
                "-i", path_str,
                "-o", path_str,
                "-f", "bin"
            ]),
            _ => Err(FileHandlerError::NotSupported(
                "plistutil only supports XML and binary formats".to_string()
            )),
        }
    } else {
        Err(FileHandlerError::NotSupported(
            "Plist conversion requires plutil or plistutil".to_string()
        ))
    }
}

/// Parse a simple value from XML plist
pub fn parse_xml_value(content: &str, key: &str) -> Option<String> {
    let key_pattern = format!("<key>{}</key>", key);
    let key_pos = content.find(&key_pattern)?;
    let after_key = &content[key_pos + key_pattern.len()..];

    // Skip whitespace
    let trimmed = after_key.trim_start();

    // Check for different value types
    if trimmed.starts_with("<string>") {
        let start = "<string>".len();
        let end = trimmed.find("</string>")?;
        return Some(trimmed[start..end].to_string());
    }

    if trimmed.starts_with("<integer>") {
        let start = "<integer>".len();
        let end = trimmed.find("</integer>")?;
        return Some(trimmed[start..end].to_string());
    }

    if trimmed.starts_with("<real>") {
        let start = "<real>".len();
        let end = trimmed.find("</real>")?;
        return Some(trimmed[start..end].to_string());
    }

    if trimmed.starts_with("<true/>") {
        return Some("true".to_string());
    }

    if trimmed.starts_with("<false/>") {
        return Some("false".to_string());
    }

    None
}

/// Format plist as indented text (simple pretty print)
pub fn format_plist(content: &str) -> String {
    let mut result = String::new();
    let mut indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Decrease indent for closing tags
        if trimmed.starts_with("</dict>") || trimmed.starts_with("</array>") {
            indent = indent.saturating_sub(1);
        }

        // Add indentation
        for _ in 0..indent {
            result.push_str("  ");
        }
        result.push_str(trimmed);
        result.push('\n');

        // Increase indent for opening tags
        if (trimmed.starts_with("<dict>") || trimmed.starts_with("<array>")) &&
           !trimmed.contains("</") {
            indent += 1;
        }
    }

    result
}
