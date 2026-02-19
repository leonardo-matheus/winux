//! Windows Registry (.reg) file handler
//!
//! Parses and displays Windows registry files in a readable format.
//! Also supports importing registry files when running under Wine.

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
};
use std::fs;
use std::path::Path;

/// Registry value types
#[derive(Debug, Clone)]
pub enum RegValueType {
    String,
    ExpandString,
    Binary,
    Dword,
    DwordBigEndian,
    MultiString,
    Qword,
    None,
    Unknown(String),
}

impl RegValueType {
    fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "REG_SZ" => RegValueType::String,
            "REG_EXPAND_SZ" => RegValueType::ExpandString,
            "REG_BINARY" => RegValueType::Binary,
            "REG_DWORD" | "REG_DWORD_LITTLE_ENDIAN" => RegValueType::Dword,
            "REG_DWORD_BIG_ENDIAN" => RegValueType::DwordBigEndian,
            "REG_MULTI_SZ" => RegValueType::MultiString,
            "REG_QWORD" | "REG_QWORD_LITTLE_ENDIAN" => RegValueType::Qword,
            "REG_NONE" => RegValueType::None,
            other => RegValueType::Unknown(other.to_string()),
        }
    }
}

/// A registry key with its values
#[derive(Debug, Clone)]
pub struct RegKey {
    pub path: String,
    pub values: Vec<RegValue>,
}

/// A registry value
#[derive(Debug, Clone)]
pub struct RegValue {
    pub name: String,
    pub value_type: RegValueType,
    pub data: String,
}

/// Parsed registry file
#[derive(Debug)]
pub struct RegFile {
    pub version: String,
    pub keys: Vec<RegKey>,
}

/// Parse a .reg file
pub fn parse_reg_file(path: &Path) -> FileHandlerResult<RegFile> {
    let content = fs::read_to_string(path)?;

    let mut lines = content.lines().peekable();

    // Parse header
    let version = lines
        .next()
        .ok_or_else(|| FileHandlerError::Parse("Empty reg file".to_string()))?
        .trim()
        .to_string();

    // Validate header
    if !version.starts_with("Windows Registry Editor") && !version.starts_with("REGEDIT") {
        return Err(FileHandlerError::Parse(
            "Invalid registry file header".to_string()
        ));
    }

    let mut keys = Vec::new();
    let mut current_key: Option<RegKey> = None;
    let mut continued_line = String::new();

    while let Some(line) = lines.next() {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Handle line continuation
        if !continued_line.is_empty() {
            continued_line.push_str(line.trim_end_matches('\\'));
            if !line.ends_with('\\') {
                // Process the complete line
                if let Some(ref mut key) = current_key {
                    if let Some(value) = parse_reg_value(&continued_line) {
                        key.values.push(value);
                    }
                }
                continued_line.clear();
            }
            continue;
        }

        if line.ends_with('\\') {
            continued_line = line.trim_end_matches('\\').to_string();
            continue;
        }

        // New key
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous key
            if let Some(key) = current_key.take() {
                keys.push(key);
            }

            // Start new key
            let key_path = line[1..line.len() - 1].to_string();
            current_key = Some(RegKey {
                path: key_path,
                values: Vec::new(),
            });
            continue;
        }

        // Skip comments
        if line.starts_with(';') {
            continue;
        }

        // Parse value
        if let Some(ref mut key) = current_key {
            if let Some(value) = parse_reg_value(line) {
                key.values.push(value);
            }
        }
    }

    // Don't forget the last key
    if let Some(key) = current_key {
        keys.push(key);
    }

    Ok(RegFile { version, keys })
}

/// Parse a registry value line
fn parse_reg_value(line: &str) -> Option<RegValue> {
    // Default value
    if line.starts_with('@') {
        let rest = line.strip_prefix('@')?;
        let rest = rest.trim_start_matches('=');
        return Some(parse_value_data("(Default)", rest));
    }

    // Named value: "name"=type:data or "name"="string"
    if line.starts_with('"') {
        let end_quote = line[1..].find('"')?;
        let name = &line[1..end_quote + 1];
        let rest = line[end_quote + 2..].trim_start_matches('=');
        return Some(parse_value_data(name, rest));
    }

    None
}

/// Parse the type and data portion of a value
fn parse_value_data(name: &str, data: &str) -> RegValue {
    let data = data.trim();

    // String value: "string"
    if data.starts_with('"') && data.ends_with('"') {
        return RegValue {
            name: name.to_string(),
            value_type: RegValueType::String,
            data: data[1..data.len() - 1].to_string(),
        };
    }

    // Typed value: type:data
    if let Some(colon_pos) = data.find(':') {
        let type_str = &data[..colon_pos];
        let value_data = &data[colon_pos + 1..];

        return RegValue {
            name: name.to_string(),
            value_type: RegValueType::from_str(type_str),
            data: value_data.to_string(),
        };
    }

    // DWORD value: dword:xxxxxxxx
    if data.starts_with("dword:") {
        return RegValue {
            name: name.to_string(),
            value_type: RegValueType::Dword,
            data: data.strip_prefix("dword:").unwrap_or(data).to_string(),
        };
    }

    // Hex/binary value: hex:xx,xx,xx,...
    if data.starts_with("hex") {
        let value_type = if data.starts_with("hex(2):") {
            RegValueType::ExpandString
        } else if data.starts_with("hex(7):") {
            RegValueType::MultiString
        } else if data.starts_with("hex(b):") {
            RegValueType::Qword
        } else {
            RegValueType::Binary
        };

        let hex_data = data
            .split(':')
            .nth(1)
            .unwrap_or("")
            .replace(',', " ");

        return RegValue {
            name: name.to_string(),
            value_type,
            data: hex_data,
        };
    }

    // Delete value marker: -
    if data == "-" {
        return RegValue {
            name: name.to_string(),
            value_type: RegValueType::None,
            data: "(Delete)".to_string(),
        };
    }

    // Unknown format
    RegValue {
        name: name.to_string(),
        value_type: RegValueType::Unknown("Unknown".to_string()),
        data: data.to_string(),
    }
}

/// Get information about a .reg file
pub fn get_reg_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Windows Registry File");

    match parse_reg_file(path) {
        Ok(reg) => {
            info.add_property("Version", &reg.version);
            info.add_property("Keys", &reg.keys.len().to_string());

            // Count values
            let total_values: usize = reg.keys.iter().map(|k| k.values.len()).sum();
            info.add_property("Total Values", &total_values.to_string());

            // Show first few keys
            let key_preview: Vec<&str> = reg.keys
                .iter()
                .take(5)
                .map(|k| k.path.as_str())
                .collect();
            info.add_property("Keys (preview)", &key_preview.join("\n"));
        }
        Err(e) => {
            info.add_property("Parse Error", &e.to_string());
        }
    }

    Ok(info)
}

/// Import a .reg file using Wine's regedit
pub fn import_reg(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try Wine regedit
    if command_exists("wine") {
        let output = run_command("wine", &["regedit", path_str])?;
        return Ok(format!("Imported registry file via Wine\n{}", output));
    }

    Err(FileHandlerError::NotSupported(
        "Wine is required to import Windows registry files".to_string()
    ))
}

/// Format registry file content for display
pub fn format_reg_content(path: &Path) -> FileHandlerResult<String> {
    let reg = parse_reg_file(path)?;
    let mut output = String::new();

    output.push_str(&format!("Registry File: {}\n", reg.version));
    output.push_str(&format!("{} keys\n\n", reg.keys.len()));

    for key in &reg.keys {
        output.push_str(&format!("[{}]\n", key.path));

        for value in &key.values {
            let type_str = match &value.value_type {
                RegValueType::String => "REG_SZ",
                RegValueType::ExpandString => "REG_EXPAND_SZ",
                RegValueType::Binary => "REG_BINARY",
                RegValueType::Dword => "REG_DWORD",
                RegValueType::DwordBigEndian => "REG_DWORD_BE",
                RegValueType::MultiString => "REG_MULTI_SZ",
                RegValueType::Qword => "REG_QWORD",
                RegValueType::None => "REG_NONE",
                RegValueType::Unknown(s) => s,
            };

            output.push_str(&format!("  {} ({}) = {}\n", value.name, type_str, value.data));
        }
        output.push('\n');
    }

    Ok(output)
}
