//! Extraction operations

use super::{ExtractOptions, OperationStatus, ProgressCallback, ProgressInfo};
use crate::archive::{Archive, ArchiveEntry};
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Extract all files from an archive
pub fn extract_all(archive: &Archive, options: &ExtractOptions) -> Result<()> {
    archive.extract_all(&options.destination, options.password.as_deref())
}

/// Extract all files with progress callback
pub fn extract_all_with_progress(
    archive: &Archive,
    options: &ExtractOptions,
    callback: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<()> {
    let entries = archive.list_entries("")?;
    let total_files = entries.len();
    let total_bytes: u64 = entries.iter().map(|e| e.uncompressed_size).sum();

    let mut bytes_processed = 0u64;

    for (index, entry) in entries.iter().enumerate() {
        // Check for cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            callback(ProgressInfo {
                current_file: entry.path.clone(),
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
            current_file: entry.path.clone(),
            current_index: index,
            total_files,
            bytes_processed,
            total_bytes,
            status: OperationStatus::InProgress,
        });

        // Extract entry
        match archive.extract_entry(entry, &options.destination, options.password.as_deref()) {
            Ok(_) => {
                bytes_processed += entry.uncompressed_size;
            }
            Err(e) => {
                callback(ProgressInfo {
                    current_file: entry.path.clone(),
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

/// Extract selected entries
pub fn extract_selected(
    archive: &Archive,
    entries: &[ArchiveEntry],
    options: &ExtractOptions,
) -> Result<()> {
    for entry in entries {
        archive.extract_entry(entry, &options.destination, options.password.as_deref())?;
    }
    Ok(())
}

/// Extract selected entries with progress callback
pub fn extract_selected_with_progress(
    archive: &Archive,
    entries: &[ArchiveEntry],
    options: &ExtractOptions,
    callback: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<()> {
    let total_files = entries.len();
    let total_bytes: u64 = entries.iter().map(|e| e.uncompressed_size).sum();

    let mut bytes_processed = 0u64;

    for (index, entry) in entries.iter().enumerate() {
        // Check for cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            callback(ProgressInfo {
                current_file: entry.path.clone(),
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
            current_file: entry.path.clone(),
            current_index: index,
            total_files,
            bytes_processed,
            total_bytes,
            status: OperationStatus::InProgress,
        });

        // Extract entry
        match archive.extract_entry(entry, &options.destination, options.password.as_deref()) {
            Ok(_) => {
                bytes_processed += entry.uncompressed_size;
            }
            Err(e) => {
                callback(ProgressInfo {
                    current_file: entry.path.clone(),
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

/// Extract to a specific path, creating directory if needed
pub fn extract_to(archive: &Archive, dest: &Path, password: Option<&str>) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    archive.extract_all(dest, password)
}

/// Extract and flatten directory structure
pub fn extract_flat(archive: &Archive, dest: &Path, password: Option<&str>) -> Result<()> {
    std::fs::create_dir_all(dest)?;

    let entries = archive.list_entries("")?;

    for entry in &entries {
        if entry.is_directory {
            continue;
        }

        // Use only the file name, not the full path
        let file_name = entry.path.rsplit('/').next().unwrap_or(&entry.path);
        let output_path = dest.join(file_name);

        // Handle duplicates by adding a number
        let final_path = if output_path.exists() {
            let stem = output_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = output_path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            let mut counter = 1;
            loop {
                let new_name = if ext.is_empty() {
                    format!("{} ({})", stem, counter)
                } else {
                    format!("{} ({}).{}", stem, counter, ext)
                };

                let new_path = dest.join(new_name);
                if !new_path.exists() {
                    break new_path;
                }
                counter += 1;
            }
        } else {
            output_path
        };

        // Extract the entry
        // We need to extract to a temp location first, then move
        let temp_dir = tempfile::tempdir()?;
        archive.extract_entry(entry, temp_dir.path(), password)?;

        let temp_file = temp_dir.path().join(&entry.path);
        if temp_file.exists() {
            std::fs::rename(&temp_file, &final_path)?;
        }
    }

    Ok(())
}
