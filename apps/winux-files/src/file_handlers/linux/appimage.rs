//! AppImage file handler
//!
//! Provides functionality to:
//! - Get AppImage information
//! - Run AppImages
//! - Extract AppImage contents

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header,
};
use std::fs;
use std::path::Path;

/// AppImage Type 1 magic (ISO 9660)
const ISO_MAGIC: &[u8] = &[0x43, 0x44, 0x30, 0x30, 0x31]; // "CD001" at offset 0x8001

/// AppImage Type 2 magic (ELF with squashfs)
const ELF_MAGIC: &[u8] = &[0x7f, 0x45, 0x4c, 0x46]; // ELF header

/// AppImage type
#[derive(Debug, Clone)]
pub enum AppImageType {
    Type1,  // ISO 9660 based
    Type2,  // SquashFS based (modern)
    Unknown,
}

impl AppImageType {
    fn to_string(&self) -> &'static str {
        match self {
            AppImageType::Type1 => "Type 1 (ISO 9660)",
            AppImageType::Type2 => "Type 2 (SquashFS)",
            AppImageType::Unknown => "Unknown",
        }
    }
}

/// Detect AppImage type
pub fn detect_appimage_type(path: &Path) -> FileHandlerResult<AppImageType> {
    let header = read_file_header(path, 16)?;

    // Check for ELF header (Type 2)
    if header.starts_with(ELF_MAGIC) {
        // Read further to check for AppImage signature
        if let Ok(content) = fs::read(path) {
            // Look for "AI" magic in the ELF
            if content.windows(2).any(|w| w == b"AI") {
                return Ok(AppImageType::Type2);
            }
            // Also check for squashfs magic
            let squashfs_magic = b"hsqs";
            if content.windows(4).any(|w| w == squashfs_magic) {
                return Ok(AppImageType::Type2);
            }
        }
        return Ok(AppImageType::Type2); // Assume Type 2 if ELF
    }

    // Check for ISO (Type 1) - need to read at offset
    if let Ok(content) = fs::read(path) {
        if content.len() > 0x8005 && &content[0x8001..0x8006] == ISO_MAGIC {
            return Ok(AppImageType::Type1);
        }
    }

    Ok(AppImageType::Unknown)
}

/// Get information about an AppImage
pub fn get_appimage_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("AppImage");

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Detect type
    let appimage_type = detect_appimage_type(path)?;
    info.add_property("AppImage Type", appimage_type.to_string());

    // Check if executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let is_exec = metadata.permissions().mode() & 0o111 != 0;
            info.add_property("Executable", if is_exec { "Yes" } else { "No" });
        }
    }

    // Try to extract desktop file info
    if command_exists("unsquashfs") {
        // List contents to find .desktop file
        if let Ok(output) = run_command("unsquashfs", &["-l", path_str]) {
            // Find .desktop file
            let desktop_file = output.lines()
                .find(|l| l.ends_with(".desktop") && !l.contains("/"));

            if let Some(desktop) = desktop_file {
                info.add_property("Desktop Entry", desktop.trim());
            }

            // Count files
            let file_count = output.lines()
                .filter(|l| !l.is_empty() && !l.starts_with("Parallel"))
                .count();
            info.add_property("Files", &file_count.to_string());
        }
    }

    // Try --appimage-help to check if it's a valid AppImage
    if let Ok(output) = run_command(path_str, &["--appimage-help"]) {
        info.add_property("Valid AppImage", "Yes");

        // Extract version if available
        if let Ok(version) = run_command(path_str, &["--appimage-version"]) {
            let version = version.trim();
            if !version.is_empty() {
                info.add_property("AppImage Version", version);
            }
        }
    }

    // Try file command for additional info
    if command_exists("file") {
        if let Ok(output) = run_command("file", &[path_str]) {
            if let Some(desc) = output.split(':').nth(1) {
                info.add_property("File Type", desc.trim());
            }
        }
    }

    Ok(info)
}

/// Run an AppImage
pub fn run_appimage(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Ensure it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let mut perms = metadata.permissions();
        if perms.mode() & 0o111 == 0 {
            perms.set_mode(perms.mode() | 0o755);
            fs::set_permissions(path, perms)?;
        }
    }

    // Run the AppImage
    let child = std::process::Command::new(path_str)
        .spawn()
        .map_err(|e| FileHandlerError::CommandFailed(format!("Failed to run AppImage: {}", e)))?;

    Ok(format!("Started AppImage (PID: {})", child.id()))
}

/// Extract AppImage contents
pub fn extract_appimage(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    // Try built-in extraction (Type 2)
    // First make sure it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let mut perms = metadata.permissions();
            if perms.mode() & 0o111 == 0 {
                perms.set_mode(perms.mode() | 0o755);
                let _ = fs::set_permissions(path, perms);
            }
        }
    }

    // Try --appimage-extract
    if let Ok(output) = run_command(path_str, &["--appimage-extract"]) {
        // AppImage extracts to squashfs-root by default
        let squashfs_root = parent.join("squashfs-root");
        if squashfs_root.exists() {
            // Rename to our output dir
            fs::rename(&squashfs_root, &output_dir)?;
            return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
        }
        return Ok(format!("Extracted (check squashfs-root directory)\n{}", output));
    }

    // Try unsquashfs for Type 2
    if command_exists("unsquashfs") {
        std::fs::create_dir_all(&output_dir)?;

        // Find offset of squashfs
        if let Ok(content) = fs::read(path) {
            let squashfs_magic = b"hsqs";
            if let Some(offset) = content.windows(4).position(|w| w == squashfs_magic) {
                // Extract from offset using dd + unsquashfs
                let temp_squashfs = parent.join("_temp.squashfs");

                // Copy squashfs portion
                let squashfs_data = &content[offset..];
                fs::write(&temp_squashfs, squashfs_data)?;

                let result = run_command("unsquashfs", &[
                    "-d", output_dir.to_str().unwrap_or("."),
                    temp_squashfs.to_str().unwrap_or("")
                ]);

                let _ = fs::remove_file(&temp_squashfs);

                if let Ok(output) = result {
                    return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
                }
            }
        }
    }

    // Try 7z
    if command_exists("7z") {
        std::fs::create_dir_all(&output_dir)?;
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir.display()),
            path_str
        ])?;
        return Ok(format!("Extracted with 7z to {}\n{}", output_dir.display(), output));
    }

    Err(FileHandlerError::NotSupported(
        "AppImage extraction requires --appimage-extract support, unsquashfs, or 7z".to_string()
    ))
}

/// Make an AppImage executable
pub fn make_executable(path: &Path) -> FileHandlerResult<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let mut perms = metadata.permissions();
        perms.set_mode(perms.mode() | 0o755);
        fs::set_permissions(path, perms)?;
        return Ok("Made executable (chmod +x)".to_string());
    }

    #[cfg(not(unix))]
    {
        Err(FileHandlerError::NotSupported(
            "Making files executable is only supported on Unix systems".to_string()
        ))
    }
}

/// Integrate AppImage with desktop (create .desktop file)
pub fn integrate_appimage(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("appimage");

    // Get applications directory
    let applications_dir = dirs::data_dir()
        .map(|d| d.join("applications"))
        .unwrap_or_else(|| Path::new("~/.local/share/applications").to_path_buf());

    std::fs::create_dir_all(&applications_dir)?;

    // Create desktop file
    let desktop_content = format!(
        "[Desktop Entry]\n\
        Type=Application\n\
        Name={}\n\
        Exec=\"{}\"\n\
        Icon={}\n\
        Terminal=false\n\
        Categories=Utility;\n",
        stem, path_str, stem
    );

    let desktop_path = applications_dir.join(format!("{}.desktop", stem));
    fs::write(&desktop_path, desktop_content)?;

    Ok(format!("Created desktop entry: {}", desktop_path.display()))
}
