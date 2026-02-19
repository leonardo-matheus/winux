//! Archive creation operations

use super::{CreateOptions, OperationStatus, ProgressCallback, ProgressInfo};
use crate::archive::{Archive, ArchiveFormat};
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

/// Create a new archive from files
pub fn create_archive(files: &[impl AsRef<Path>], options: &CreateOptions) -> Result<Archive> {
    let mut archive = Archive::create(&options.output_path, options.format)?;

    for file_path in files {
        let file_path = file_path.as_ref();
        let archive_path = calculate_archive_path(file_path, options.base_path.as_deref());

        if file_path.is_dir() {
            add_directory(&mut archive, file_path, &archive_path)?;
        } else {
            archive.add_file(file_path, &archive_path)?;
        }
    }

    Ok(archive)
}

/// Create archive with progress callback
pub fn create_archive_with_progress(
    files: &[impl AsRef<Path>],
    options: &CreateOptions,
    callback: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<Archive> {
    // First, collect all files to add
    let mut all_files: Vec<(std::path::PathBuf, String)> = Vec::new();
    let mut total_bytes = 0u64;

    for file_path in files {
        let file_path = file_path.as_ref();
        let base_archive_path = calculate_archive_path(file_path, options.base_path.as_deref());

        if file_path.is_dir() {
            for entry in WalkDir::new(file_path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let relative = entry_path.strip_prefix(file_path)?;
                    let archive_path = if base_archive_path.is_empty() {
                        relative.to_string_lossy().to_string()
                    } else {
                        format!("{}/{}", base_archive_path, relative.to_string_lossy())
                    };

                    if let Ok(metadata) = entry.metadata() {
                        total_bytes += metadata.len();
                    }

                    all_files.push((entry_path.to_path_buf(), archive_path));
                }
            }
        } else {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                total_bytes += metadata.len();
            }
            all_files.push((file_path.to_path_buf(), base_archive_path));
        }
    }

    let total_files = all_files.len();
    let mut archive = Archive::create(&options.output_path, options.format)?;
    let mut bytes_processed = 0u64;

    for (index, (file_path, archive_path)) in all_files.iter().enumerate() {
        // Check for cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            callback(ProgressInfo {
                current_file: file_path.to_string_lossy().to_string(),
                current_index: index,
                total_files,
                bytes_processed,
                total_bytes,
                status: OperationStatus::Cancelled,
            });
            return Err(anyhow::anyhow!("Operation cancelled"));
        }

        // Report progress
        callback(ProgressInfo {
            current_file: file_path.to_string_lossy().to_string(),
            current_index: index,
            total_files,
            bytes_processed,
            total_bytes,
            status: OperationStatus::InProgress,
        });

        // Add file
        match archive.add_file(file_path, archive_path) {
            Ok(_) => {
                if let Ok(metadata) = std::fs::metadata(file_path) {
                    bytes_processed += metadata.len();
                }
            }
            Err(e) => {
                callback(ProgressInfo {
                    current_file: file_path.to_string_lossy().to_string(),
                    current_index: index,
                    total_files,
                    bytes_processed,
                    total_bytes,
                    status: OperationStatus::Failed(e.to_string()),
                });
                return Err(e);
            }
        }
    }

    // Report completion
    callback(ProgressInfo {
        current_file: String::new(),
        current_index: total_files,
        total_files,
        bytes_processed: total_bytes,
        total_bytes,
        status: OperationStatus::Completed,
    });

    Ok(archive)
}

/// Add a directory recursively to the archive
fn add_directory(archive: &mut Archive, dir_path: &Path, base_archive_path: &str) -> Result<()> {
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();

        if entry_path.is_file() {
            let relative = entry_path.strip_prefix(dir_path)?;
            let archive_path = if base_archive_path.is_empty() {
                relative.to_string_lossy().to_string()
            } else {
                format!("{}/{}", base_archive_path, relative.to_string_lossy())
            };

            archive.add_file(entry_path, &archive_path)?;
        }
    }

    Ok(())
}

/// Calculate the archive path for a file
fn calculate_archive_path(file_path: &Path, base_path: Option<&Path>) -> String {
    if let Some(base) = base_path {
        if let Ok(relative) = file_path.strip_prefix(base) {
            return relative.to_string_lossy().to_string();
        }
    }

    // Use file name only
    file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string_lossy().to_string())
}

/// Suggest an output filename based on input files
pub fn suggest_output_name(files: &[impl AsRef<Path>], format: ArchiveFormat) -> String {
    if files.len() == 1 {
        let file_path = files[0].as_ref();
        let name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "archive".to_string());

        // Remove any existing extension if it's a single file
        let name = if file_path.is_file() {
            name.rsplit_once('.')
                .map(|(n, _)| n.to_string())
                .unwrap_or(name)
        } else {
            name
        };

        format!("{}.{}", name, format.extension())
    } else {
        format!("archive.{}", format.extension())
    }
}

/// Create archive from a directory
pub fn create_from_directory(
    dir_path: &Path,
    output_path: &Path,
    format: ArchiveFormat,
    compression_level: u8,
) -> Result<Archive> {
    let options = CreateOptions {
        output_path: output_path.to_path_buf(),
        format,
        compression_level,
        base_path: Some(dir_path.to_path_buf()),
        ..Default::default()
    };

    create_archive(&[dir_path], &options)
}

/// Estimate compressed size (rough estimate)
pub fn estimate_compressed_size(files: &[impl AsRef<Path>], format: ArchiveFormat) -> u64 {
    let mut total_size = 0u64;

    for file_path in files {
        let file_path = file_path.as_ref();

        if file_path.is_dir() {
            for entry in WalkDir::new(file_path).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        } else {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                total_size += metadata.len();
            }
        }
    }

    // Apply rough compression ratio based on format
    let ratio = match format {
        ArchiveFormat::Zip => 0.6,
        ArchiveFormat::TarGz => 0.55,
        ArchiveFormat::TarBz2 => 0.5,
        ArchiveFormat::TarXz => 0.45,
        ArchiveFormat::SevenZip => 0.4,
        ArchiveFormat::Zstd => 0.5,
        ArchiveFormat::Tar => 1.0,
        ArchiveFormat::Rar => 0.5,
        ArchiveFormat::Iso => 1.0,
    };

    (total_size as f64 * ratio) as u64
}
