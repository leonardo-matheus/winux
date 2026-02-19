//! Add files to existing archives

use super::{OperationStatus, ProgressCallback, ProgressInfo};
use crate::archive::Archive;
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

/// Add files to an existing archive
pub fn add_files(archive: &mut Archive, files: &[impl AsRef<Path>], base_path: &str) -> Result<()> {
    for file_path in files {
        let file_path = file_path.as_ref();

        if file_path.is_dir() {
            add_directory(archive, file_path, base_path)?;
        } else {
            let archive_path = calculate_archive_path(file_path, base_path);
            archive.add_file(file_path, &archive_path)?;
        }
    }

    Ok(())
}

/// Add files with progress callback
pub fn add_files_with_progress(
    archive: &mut Archive,
    files: &[impl AsRef<Path>],
    base_path: &str,
    callback: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<()> {
    // First, collect all files to add
    let mut all_files: Vec<(std::path::PathBuf, String)> = Vec::new();
    let mut total_bytes = 0u64;

    for file_path in files {
        let file_path = file_path.as_ref();

        if file_path.is_dir() {
            for entry in WalkDir::new(file_path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let archive_path = calculate_archive_path_for_dir(file_path, entry_path, base_path);

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

            let archive_path = calculate_archive_path(file_path, base_path);
            all_files.push((file_path.to_path_buf(), archive_path));
        }
    }

    let total_files = all_files.len();
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

    Ok(())
}

/// Add a directory recursively
fn add_directory(archive: &mut Archive, dir_path: &Path, base_path: &str) -> Result<()> {
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();

        if entry_path.is_file() {
            let archive_path = calculate_archive_path_for_dir(dir_path, entry_path, base_path);
            archive.add_file(entry_path, &archive_path)?;
        }
    }

    Ok(())
}

/// Calculate archive path for a file
fn calculate_archive_path(file_path: &Path, base_path: &str) -> String {
    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());

    if base_path.is_empty() {
        file_name
    } else {
        format!("{}/{}", base_path.trim_end_matches('/'), file_name)
    }
}

/// Calculate archive path for a file within a directory
fn calculate_archive_path_for_dir(dir_path: &Path, file_path: &Path, base_path: &str) -> String {
    let relative = file_path
        .strip_prefix(dir_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string())
        });

    let dir_name = dir_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "folder".to_string());

    if base_path.is_empty() {
        format!("{}/{}", dir_name, relative)
    } else {
        format!("{}/{}/{}", base_path.trim_end_matches('/'), dir_name, relative)
    }
}

/// Remove files from an archive
pub fn remove_files(archive: &mut Archive, paths: &[&str]) -> Result<()> {
    for path in paths {
        archive.remove_entry(path)?;
    }
    Ok(())
}

/// Update a file in the archive (remove and re-add)
pub fn update_file(archive: &mut Archive, file_path: &Path, archive_path: &str) -> Result<()> {
    // Try to remove existing entry (ignore error if not exists)
    let _ = archive.remove_entry(archive_path);

    // Add the new file
    archive.add_file(file_path, archive_path)
}

/// Check if a path already exists in the archive
pub fn path_exists(archive: &Archive, path: &str) -> Result<bool> {
    let entries = archive.list_entries("")?;
    Ok(entries.iter().any(|e| e.path == path))
}

/// Get suggested paths for adding files
pub fn suggest_add_paths(
    archive: &Archive,
    files: &[impl AsRef<Path>],
) -> Result<Vec<(std::path::PathBuf, String, bool)>> {
    let existing_entries = archive.list_entries("")?;
    let existing_paths: std::collections::HashSet<_> = existing_entries.iter().map(|e| &e.path).collect();

    let mut suggestions = Vec::new();

    for file_path in files {
        let file_path = file_path.as_ref();
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        let exists = existing_paths.contains(&file_name);

        suggestions.push((file_path.to_path_buf(), file_name, exists));
    }

    Ok(suggestions)
}
