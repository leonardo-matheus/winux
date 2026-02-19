//! Local filesystem backup backend

use super::*;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Local backup backend - stores backups on local filesystem
pub struct LocalBackend {
    /// Base path for storing backups
    base_path: PathBuf,
}

impl LocalBackend {
    /// Create a new local backend
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Get the path for a specific backup
    fn backup_path(&self, backup_id: &str) -> PathBuf {
        self.base_path.join(backup_id)
    }

    /// Get metadata file path for a backup
    fn metadata_path(&self, backup_id: &str) -> PathBuf {
        self.backup_path(backup_id).join("metadata.json")
    }

    /// Generate a unique backup ID
    fn generate_id(&self) -> String {
        let now = chrono::Utc::now();
        format!("backup-{}", now.format("%Y%m%d-%H%M%S"))
    }

    /// Calculate directory size
    fn calculate_size(path: &Path) -> Result<u64> {
        let mut size = 0u64;
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
        Ok(size)
    }

    /// Count files in directory
    fn count_files(path: &Path) -> Result<u64> {
        let mut count = 0u64;
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                count += 1;
            }
        }
        Ok(count)
    }
}

impl BackupBackend for LocalBackend {
    fn name(&self) -> &str {
        "Local"
    }

    fn is_available(&self) -> bool {
        self.base_path.exists() && self.base_path.is_dir()
    }

    fn test_connection(&self) -> Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)
                .context("Failed to create backup directory")?;
        }

        // Test write access
        let test_file = self.base_path.join(".test_write");
        fs::write(&test_file, "test").context("Failed to write test file")?;
        fs::remove_file(&test_file).context("Failed to remove test file")?;

        Ok(())
    }

    fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let mut backups = Vec::new();

        if !self.base_path.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let metadata_path = path.join("metadata.json");
                if metadata_path.exists() {
                    let content = fs::read_to_string(&metadata_path)?;
                    if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&content) {
                        backups.push(metadata);
                    }
                }
            }
        }

        // Sort by timestamp, newest first
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    fn create_backup(
        &self,
        sources: &[&Path],
        name: &str,
        backup_type: BackupType,
        compression: CompressionType,
        encrypt: bool,
        progress: Option<ProgressCallback>,
    ) -> Result<BackupMetadata> {
        let backup_id = self.generate_id();
        let backup_path = self.backup_path(&backup_id);
        let data_path = backup_path.join("data");

        fs::create_dir_all(&data_path)?;

        // Report scanning phase
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Scanning files...".to_string(),
                files_processed: 0,
                files_total: 0,
                bytes_processed: 0,
                bytes_total: 0,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
                phase: BackupPhase::Scanning,
            });
        }

        // Count total files and size
        let mut total_files = 0u64;
        let mut total_bytes = 0u64;

        for source in sources {
            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    total_files += 1;
                    total_bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }

        // Copy files
        let mut files_processed = 0u64;
        let mut bytes_processed = 0u64;
        let start_time = std::time::Instant::now();

        for source in sources {
            let source_name = source.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());

            for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                let relative_path = entry_path.strip_prefix(source).unwrap_or(entry_path);
                let dest_path = data_path.join(&source_name).join(relative_path);

                if entry.file_type().is_dir() {
                    fs::create_dir_all(&dest_path)?;
                } else if entry.file_type().is_file() {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::copy(entry_path, &dest_path)?;
                    files_processed += 1;
                    bytes_processed += entry.metadata().map(|m| m.len()).unwrap_or(0);

                    // Report progress
                    if let Some(ref cb) = progress {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let speed = if elapsed > 0.0 {
                            (bytes_processed as f64 / elapsed) as u64
                        } else {
                            0
                        };
                        let eta = if speed > 0 && bytes_processed < total_bytes {
                            Some((total_bytes - bytes_processed) / speed)
                        } else {
                            None
                        };

                        cb(BackupProgress {
                            current_file: entry_path.display().to_string(),
                            files_processed,
                            files_total: total_files,
                            bytes_processed,
                            bytes_total: total_bytes,
                            speed_bytes_per_sec: speed,
                            eta_seconds: eta,
                            phase: BackupPhase::Backing,
                        });
                    }
                }
            }
        }

        // Create metadata
        let metadata = BackupMetadata {
            id: backup_id.clone(),
            name: name.to_string(),
            timestamp: chrono::Utc::now(),
            backup_type,
            size_bytes: bytes_processed,
            file_count: files_processed,
            compression,
            encrypted: encrypt,
            verified: false,
            tags: vec![],
        };

        // Save metadata
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(self.metadata_path(&backup_id), metadata_json)?;

        // Report complete
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Complete".to_string(),
                files_processed,
                files_total: total_files,
                bytes_processed,
                bytes_total: total_bytes,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
                phase: BackupPhase::Complete,
            });
        }

        Ok(metadata)
    }

    fn restore_backup(
        &self,
        backup_id: &str,
        destination: &Path,
        files: Option<&[&Path]>,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        let backup_path = self.backup_path(backup_id);
        let data_path = backup_path.join("data");

        if !data_path.exists() {
            return Err(anyhow!("Backup data not found: {}", backup_id));
        }

        // Count files to restore
        let total_files = Self::count_files(&data_path)?;
        let total_bytes = Self::calculate_size(&data_path)?;

        let mut files_processed = 0u64;
        let mut bytes_processed = 0u64;
        let start_time = std::time::Instant::now();

        for entry in WalkDir::new(&data_path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(&data_path).unwrap_or(entry_path);

            // Check if this file should be restored
            if let Some(filter_files) = files {
                let should_restore = filter_files.iter().any(|f| {
                    relative_path.starts_with(f) || f.starts_with(relative_path)
                });
                if !should_restore {
                    continue;
                }
            }

            let dest_path = destination.join(relative_path);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest_path)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::copy(entry_path, &dest_path)?;
                files_processed += 1;
                bytes_processed += entry.metadata().map(|m| m.len()).unwrap_or(0);

                // Report progress
                if let Some(ref cb) = progress {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed = if elapsed > 0.0 {
                        (bytes_processed as f64 / elapsed) as u64
                    } else {
                        0
                    };

                    cb(BackupProgress {
                        current_file: entry_path.display().to_string(),
                        files_processed,
                        files_total: total_files,
                        bytes_processed,
                        bytes_total: total_bytes,
                        speed_bytes_per_sec: speed,
                        eta_seconds: None,
                        phase: BackupPhase::Backing,
                    });
                }
            }
        }

        Ok(())
    }

    fn delete_backup(&self, backup_id: &str) -> Result<()> {
        let backup_path = self.backup_path(backup_id);
        if backup_path.exists() {
            fs::remove_dir_all(backup_path)?;
        }
        Ok(())
    }

    fn verify_backup(&self, backup_id: &str) -> Result<bool> {
        let backup_path = self.backup_path(backup_id);
        let data_path = backup_path.join("data");
        let metadata_path = self.metadata_path(backup_id);

        // Check that required paths exist
        if !backup_path.exists() || !data_path.exists() || !metadata_path.exists() {
            return Ok(false);
        }

        // Verify metadata can be read
        let content = fs::read_to_string(&metadata_path)?;
        let metadata: BackupMetadata = serde_json::from_str(&content)?;

        // Verify file count matches
        let actual_count = Self::count_files(&data_path)?;
        if actual_count != metadata.file_count {
            return Ok(false);
        }

        Ok(true)
    }

    fn get_backup(&self, backup_id: &str) -> Result<Option<BackupMetadata>> {
        let metadata_path = self.metadata_path(backup_id);
        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&metadata_path)?;
        let metadata = serde_json::from_str(&content)?;
        Ok(Some(metadata))
    }

    fn list_files(&self, backup_id: &str, path: Option<&Path>) -> Result<Vec<FileEntry>> {
        let backup_path = self.backup_path(backup_id);
        let data_path = backup_path.join("data");

        let search_path = match path {
            Some(p) => data_path.join(p),
            None => data_path,
        };

        if !search_path.exists() {
            return Ok(vec![]);
        }

        let mut entries = Vec::new();

        for entry in fs::read_dir(&search_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&data_path).unwrap_or(&path);

            entries.push(FileEntry {
                path: relative_path.display().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: chrono::DateTime::from(metadata.modified()?),
                permissions: 0o644, // Default permissions
            });
        }

        // Sort directories first, then by name
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.path.cmp(&b.path),
            }
        });

        Ok(entries)
    }

    fn get_storage_usage(&self) -> Result<StorageUsage> {
        let used = Self::calculate_size(&self.base_path)?;
        let backups = self.list_backups()?;

        // Get filesystem stats (simplified)
        let total = 500 * 1024 * 1024 * 1024; // 500 GB default
        let available = total - used;

        Ok(StorageUsage {
            used_bytes: used,
            available_bytes: available,
            total_bytes: total,
            backup_count: backups.len() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_local_backend_creation() {
        let dir = tempdir().unwrap();
        let backend = LocalBackend::new(dir.path());
        assert_eq!(backend.name(), "Local");
    }

    #[test]
    fn test_connection() {
        let dir = tempdir().unwrap();
        let backend = LocalBackend::new(dir.path());
        assert!(backend.test_connection().is_ok());
    }
}
