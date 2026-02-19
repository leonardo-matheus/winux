//! Common types and utilities for file handlers

use std::collections::HashMap;
use std::fmt;
use std::fs::{self, Metadata};
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

/// Result type for file handler operations
pub type FileHandlerResult<T> = Result<T, FileHandlerError>;

/// Error type for file handler operations
#[derive(Debug)]
pub enum FileHandlerError {
    Io(io::Error),
    Parse(String),
    NotSupported(String),
    UnsupportedAction(String),
    CommandFailed(String),
    NotFound(String),
    PermissionDenied(String),
}

impl fmt::Display for FileHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileHandlerError::Io(e) => write!(f, "I/O error: {}", e),
            FileHandlerError::Parse(msg) => write!(f, "Parse error: {}", msg),
            FileHandlerError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            FileHandlerError::UnsupportedAction(msg) => write!(f, "Unsupported action: {}", msg),
            FileHandlerError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            FileHandlerError::NotFound(msg) => write!(f, "Not found: {}", msg),
            FileHandlerError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for FileHandlerError {}

impl From<io::Error> for FileHandlerError {
    fn from(err: io::Error) -> Self {
        FileHandlerError::Io(err)
    }
}

/// Actions that can be performed on files
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileAction {
    // Common actions
    ViewInfo,
    ViewContent,
    Edit,
    Open,

    // Execution
    Run,
    RunWithWine,

    // Installation
    Install,
    Extract,

    // Navigation
    Browse,
    FollowLink,

    // Mounting
    Mount,
    Unmount,

    // Registry specific
    Import,

    // Permission
    MakeExecutable,
}

impl fmt::Display for FileAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileAction::ViewInfo => write!(f, "View Information"),
            FileAction::ViewContent => write!(f, "View Content"),
            FileAction::Edit => write!(f, "Edit"),
            FileAction::Open => write!(f, "Open"),
            FileAction::Run => write!(f, "Run"),
            FileAction::RunWithWine => write!(f, "Run with Wine"),
            FileAction::Install => write!(f, "Install"),
            FileAction::Extract => write!(f, "Extract"),
            FileAction::Browse => write!(f, "Browse Contents"),
            FileAction::FollowLink => write!(f, "Follow Link"),
            FileAction::Mount => write!(f, "Mount"),
            FileAction::Unmount => write!(f, "Unmount"),
            FileAction::Import => write!(f, "Import"),
            FileAction::MakeExecutable => write!(f, "Make Executable"),
        }
    }
}

/// Information about a file
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub file_type: String,
    pub mime_type: String,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub is_executable: bool,
    pub properties: HashMap<String, String>,
}

impl FileInfo {
    pub fn new(path: &Path) -> FileHandlerResult<Self> {
        let metadata = fs::metadata(path)?;

        Ok(FileInfo {
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            file_type: "Unknown".to_string(),
            mime_type: detect_mime_type(path),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
            is_executable: is_executable(&metadata),
            properties: HashMap::new(),
        })
    }

    pub fn with_type(mut self, file_type: &str) -> Self {
        self.file_type = file_type.to_string();
        self
    }

    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    pub fn add_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }
}

impl fmt::Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Path: {}", self.path)?;
        writeln!(f, "Size: {}", format_size(self.size))?;
        writeln!(f, "Type: {}", self.file_type)?;
        writeln!(f, "MIME Type: {}", self.mime_type)?;

        if let Some(created) = self.created {
            if let Ok(duration) = created.duration_since(SystemTime::UNIX_EPOCH) {
                writeln!(f, "Created: {} seconds since epoch", duration.as_secs())?;
            }
        }

        if let Some(modified) = self.modified {
            if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                writeln!(f, "Modified: {} seconds since epoch", duration.as_secs())?;
            }
        }

        if !self.properties.is_empty() {
            writeln!(f, "\nProperties:")?;
            for (key, value) in &self.properties {
                writeln!(f, "  {}: {}", key, value)?;
            }
        }

        Ok(())
    }
}

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Detect MIME type from file extension
pub fn detect_mime_type(path: &Path) -> String {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        // Windows
        "exe" => "application/x-msdownload",
        "msi" => "application/x-msi",
        "dll" => "application/x-msdownload",
        "lnk" => "application/x-ms-shortcut",
        "reg" => "text/x-registry",
        "bat" | "cmd" => "application/x-bat",
        "ps1" => "application/x-powershell",

        // macOS
        "dmg" => "application/x-apple-diskimage",
        "app" => "application/x-apple-application",
        "pkg" => "application/x-apple-pkg",
        "plist" => "application/x-plist",
        "icns" => "image/x-icns",
        "dylib" => "application/x-mach-dylib",

        // Linux
        "deb" => "application/vnd.debian.binary-package",
        "rpm" => "application/x-rpm",
        "appimage" => "application/x-appimage",
        "flatpak" | "flatpakref" => "application/vnd.flatpak",
        "snap" => "application/vnd.snap",
        "so" => "application/x-sharedlib",

        // Archives
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "xz" => "application/x-xz",
        "bz2" => "application/x-bzip2",
        "iso" => "application/x-iso9660-image",
        "img" => "application/x-raw-disk-image",

        _ => "application/octet-stream",
    }
    .to_string()
}

/// Check if file is executable (Unix)
#[cfg(unix)]
fn is_executable(metadata: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

/// Check if file is executable (Windows)
#[cfg(windows)]
fn is_executable(_metadata: &Metadata) -> bool {
    // On Windows, executability is determined by extension
    false
}

/// Get generic file information
pub fn get_generic_info(path: &Path) -> FileHandlerResult<FileInfo> {
    FileInfo::new(path)
}

/// View file content as text
pub fn view_content(path: &Path) -> FileHandlerResult<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

/// Open file in default text editor
pub fn open_in_editor(path: &Path) -> FileHandlerResult<String> {
    // Try common editors
    let editors = ["xdg-open", "gedit", "kate", "nano", "vim"];

    for editor in editors {
        if let Ok(status) = Command::new(editor)
            .arg(path)
            .spawn()
        {
            return Ok(format!("Opened with {}", editor));
        }
    }

    Err(FileHandlerError::NotFound("No suitable editor found".to_string()))
}

/// Run a system command and return output
pub fn run_command(program: &str, args: &[&str]) -> FileHandlerResult<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| FileHandlerError::CommandFailed(format!("Failed to run {}: {}", program, e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(FileHandlerError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}

/// Check if a command is available in the system
pub fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Read binary file header
pub fn read_file_header(path: &Path, size: usize) -> FileHandlerResult<Vec<u8>> {
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut buffer = vec![0u8; size];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

/// Parse a key-value line (key=value or key: value)
pub fn parse_key_value(line: &str) -> Option<(String, String)> {
    if let Some(pos) = line.find('=') {
        let key = line[..pos].trim().to_string();
        let value = line[pos + 1..].trim().to_string();
        return Some((key, value));
    }
    if let Some(pos) = line.find(':') {
        let key = line[..pos].trim().to_string();
        let value = line[pos + 1..].trim().to_string();
        return Some((key, value));
    }
    None
}
