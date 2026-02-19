//! Archive file handlers
//!
//! This module handles various archive formats including:
//! - ZIP, RAR, 7Z
//! - TAR variants (tar.gz, tar.xz, tar.bz2)
//! - ISO disk images
//! - IMG disk images

pub mod iso;
pub mod img;

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    format_size, read_file_header,
};
use crate::file_handlers::FileType;
use std::path::Path;

/// Archive magic bytes
const ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
const RAR_MAGIC: [u8; 7] = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
const RAR5_MAGIC: [u8; 8] = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00];
const SEVEN_Z_MAGIC: [u8; 6] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];
const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];
const BZIP2_MAGIC: [u8; 3] = [0x42, 0x5A, 0x68]; // "BZh"
const XZ_MAGIC: [u8; 6] = [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];

/// Detect archive type from header
pub fn detect_archive_type(path: &Path) -> FileHandlerResult<FileType> {
    let header = read_file_header(path, 16)?;

    if header.starts_with(&ZIP_MAGIC) {
        return Ok(FileType::Zip);
    }
    if header.starts_with(&RAR_MAGIC) || header.starts_with(&RAR5_MAGIC) {
        return Ok(FileType::Rar);
    }
    if header.starts_with(&SEVEN_Z_MAGIC) {
        return Ok(FileType::SevenZip);
    }
    if header.starts_with(&GZIP_MAGIC) {
        // Could be tar.gz
        return Ok(FileType::TarGz);
    }
    if header.starts_with(&BZIP2_MAGIC) {
        return Ok(FileType::TarBz2);
    }
    if header.starts_with(&XZ_MAGIC) {
        return Ok(FileType::TarXz);
    }

    // Check for tar (no magic, but check file name)
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if name.ends_with(".tar") {
        return Ok(FileType::Tar);
    }

    Ok(FileType::Unknown)
}

/// Get information about an archive
pub fn get_archive_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let file_type = detect_archive_type(path)
        .unwrap_or(FileType::from_path(path));

    let type_str = match file_type {
        FileType::Zip => "ZIP Archive",
        FileType::Rar => "RAR Archive",
        FileType::SevenZip => "7-Zip Archive",
        FileType::TarGz => "Gzipped TAR Archive",
        FileType::TarXz => "XZ-compressed TAR Archive",
        FileType::TarBz2 => "Bzip2-compressed TAR Archive",
        FileType::Tar => "TAR Archive",
        _ => "Archive",
    };

    let mut info = FileInfo::new(path)?.with_type(type_str);

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try different tools based on archive type
    let (file_count, compressed_size, uncompressed_size) = match file_type {
        FileType::Zip => get_zip_stats(path_str),
        FileType::Rar => get_rar_stats(path_str),
        FileType::SevenZip | FileType::TarGz | FileType::TarXz | FileType::TarBz2 | FileType::Tar => {
            get_7z_stats(path_str)
        }
        _ => (None, None, None),
    };

    if let Some(count) = file_count {
        info.add_property("Files", &count.to_string());
    }
    if let Some(size) = compressed_size {
        info.add_property("Compressed Size", &format_size(size));
    }
    if let Some(size) = uncompressed_size {
        info.add_property("Uncompressed Size", &format_size(size));
        if let Some(compressed) = compressed_size {
            let ratio = (compressed as f64 / size as f64) * 100.0;
            info.add_property("Compression Ratio", &format!("{:.1}%", ratio));
        }
    }

    // List some files
    if let Ok(files) = list_contents(path) {
        let sample: Vec<&str> = files.iter().take(10).map(|s| s.as_str()).collect();
        if !sample.is_empty() {
            info.add_property("Contents (sample)", &sample.join("\n"));
        }
    }

    Ok(info)
}

/// Get ZIP archive statistics
fn get_zip_stats(path: &str) -> (Option<usize>, Option<u64>, Option<u64>) {
    if command_exists("unzip") {
        if let Ok(output) = run_command("unzip", &["-l", path]) {
            let lines: Vec<&str> = output.lines().collect();
            if let Some(summary) = lines.last() {
                // Parse: "  12345678   100 files"
                let parts: Vec<&str> = summary.split_whitespace().collect();
                if parts.len() >= 2 {
                    let uncompressed = parts[0].parse::<u64>().ok();
                    let files = parts[1].parse::<usize>().ok();
                    return (files, None, uncompressed);
                }
            }
        }
    }
    get_7z_stats(path)
}

/// Get RAR archive statistics
fn get_rar_stats(path: &str) -> (Option<usize>, Option<u64>, Option<u64>) {
    if command_exists("unrar") {
        if let Ok(output) = run_command("unrar", &["l", path]) {
            let file_count = output.lines()
                .filter(|l| l.starts_with(' ') && l.len() > 10)
                .count();
            return (Some(file_count), None, None);
        }
    }
    get_7z_stats(path)
}

/// Get archive statistics using 7z
fn get_7z_stats(path: &str) -> (Option<usize>, Option<u64>, Option<u64>) {
    if command_exists("7z") {
        if let Ok(output) = run_command("7z", &["l", path]) {
            let mut file_count = 0;
            let mut uncompressed = 0u64;
            let mut compressed = 0u64;

            for line in output.lines() {
                // Parse summary lines
                if line.contains("files") && line.contains(",") {
                    let parts: Vec<&str> = line.split(',').collect();
                    for part in parts {
                        let trimmed = part.trim();
                        if trimmed.ends_with("files") {
                            if let Some(num_str) = trimmed.strip_suffix("files") {
                                file_count = num_str.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                }

                // Parse size info
                if line.starts_with("Size =") {
                    if let Some(size_str) = line.strip_prefix("Size =") {
                        uncompressed = size_str.trim().parse().unwrap_or(0);
                    }
                }
                if line.starts_with("Packed Size =") {
                    if let Some(size_str) = line.strip_prefix("Packed Size =") {
                        compressed = size_str.trim().parse().unwrap_or(0);
                    }
                }
            }

            let file_count = if file_count > 0 { Some(file_count) } else { None };
            let compressed = if compressed > 0 { Some(compressed) } else { None };
            let uncompressed = if uncompressed > 0 { Some(uncompressed) } else { None };

            return (file_count, compressed, uncompressed);
        }
    }
    (None, None, None)
}

/// List contents of an archive
pub fn list_contents(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let file_type = FileType::from_path(path);

    // Try format-specific tools first
    match file_type {
        FileType::Zip => {
            if command_exists("unzip") {
                let output = run_command("unzip", &["-Z1", path_str])?;
                return Ok(output.lines().map(|s| s.to_string()).collect());
            }
        }
        FileType::Rar => {
            if command_exists("unrar") {
                let output = run_command("unrar", &["lb", path_str])?;
                return Ok(output.lines().map(|s| s.to_string()).collect());
            }
        }
        FileType::Tar | FileType::TarGz | FileType::TarXz | FileType::TarBz2 => {
            if command_exists("tar") {
                let output = run_command("tar", &["-tf", path_str])?;
                return Ok(output.lines().map(|s| s.to_string()).collect());
            }
        }
        _ => {}
    }

    // Fallback to 7z (works with most formats)
    if command_exists("7z") {
        let output = run_command("7z", &["l", "-slt", path_str])?;
        let files: Vec<String> = output.lines()
            .filter(|l| l.starts_with("Path = "))
            .map(|l| l.strip_prefix("Path = ").unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .collect();
        return Ok(files);
    }

    Err(FileHandlerError::NotSupported(
        "No archive tool available (install p7zip, unzip, or unrar)".to_string()
    ))
}

/// Extract an archive
pub fn extract_archive(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");

    // Handle .tar.* double extensions
    let stem = if stem.ends_with(".tar") {
        &stem[..stem.len() - 4]
    } else {
        stem
    };

    let output_dir = parent.join(format!("{}_extracted", stem));
    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    let file_type = FileType::from_path(path);

    // Try format-specific tools
    match file_type {
        FileType::Zip => {
            if command_exists("unzip") {
                let output = run_command("unzip", &["-o", path_str, "-d", output_dir_str])?;
                return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
            }
        }
        FileType::Rar => {
            if command_exists("unrar") {
                let output = run_command("unrar", &["x", "-o+", path_str, output_dir_str])?;
                return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
            }
        }
        FileType::Tar => {
            if command_exists("tar") {
                let output = run_command("tar", &["-xf", path_str, "-C", output_dir_str])?;
                return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
            }
        }
        FileType::TarGz => {
            if command_exists("tar") {
                let output = run_command("tar", &["-xzf", path_str, "-C", output_dir_str])?;
                return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
            }
        }
        FileType::TarXz => {
            if command_exists("tar") {
                let output = run_command("tar", &["-xJf", path_str, "-C", output_dir_str])?;
                return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
            }
        }
        FileType::TarBz2 => {
            if command_exists("tar") {
                let output = run_command("tar", &["-xjf", path_str, "-C", output_dir_str])?;
                return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
            }
        }
        _ => {}
    }

    // Fallback to 7z
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;
        return Ok(format!("Extracted with 7z to {}\n{}", output_dir.display(), output));
    }

    Err(FileHandlerError::NotSupported(
        "No extraction tool available for this format".to_string()
    ))
}

/// Create an archive from files
pub fn create_archive(output_path: &Path, files: &[&Path], format: FileType) -> FileHandlerResult<String> {
    let output_str = output_path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid output path".to_string()))?;

    let file_args: Vec<&str> = files.iter()
        .filter_map(|p| p.to_str())
        .collect();

    match format {
        FileType::Zip => {
            if command_exists("zip") {
                let mut args = vec!["-r", output_str];
                args.extend(file_args);
                let output = run_command("zip", &args)?;
                return Ok(format!("Created ZIP archive: {}\n{}", output_str, output));
            }
        }
        FileType::TarGz => {
            if command_exists("tar") {
                let mut args = vec!["-czf", output_str];
                args.extend(file_args);
                let output = run_command("tar", &args)?;
                return Ok(format!("Created tar.gz archive: {}\n{}", output_str, output));
            }
        }
        FileType::SevenZip => {
            if command_exists("7z") {
                let mut args = vec!["a", output_str];
                args.extend(file_args);
                let output = run_command("7z", &args)?;
                return Ok(format!("Created 7z archive: {}\n{}", output_str, output));
            }
        }
        _ => {}
    }

    // Fallback to 7z
    if command_exists("7z") {
        let mut args = vec!["a", output_str];
        args.extend(file_args);
        let output = run_command("7z", &args)?;
        return Ok(format!("Created archive: {}\n{}", output_str, output));
    }

    Err(FileHandlerError::NotSupported(
        "No archive creation tool available".to_string()
    ))
}

/// Test archive integrity
pub fn test_archive(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let file_type = FileType::from_path(path);

    match file_type {
        FileType::Zip => {
            if command_exists("unzip") {
                let output = run_command("unzip", &["-t", path_str])?;
                return Ok(output);
            }
        }
        FileType::Rar => {
            if command_exists("unrar") {
                let output = run_command("unrar", &["t", path_str])?;
                return Ok(output);
            }
        }
        _ => {}
    }

    // Fallback to 7z
    if command_exists("7z") {
        let output = run_command("7z", &["t", path_str])?;
        return Ok(output);
    }

    Err(FileHandlerError::NotSupported(
        "No archive testing tool available".to_string()
    ))
}
