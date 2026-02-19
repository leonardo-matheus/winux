//! Backup backends module
//!
//! Provides different storage backends for backups:
//! - Local: Direct filesystem storage
//! - Rsync: Remote storage via SSH
//! - Restic: Incremental, deduplicated backups
//! - Cloud: Google Drive, Dropbox, OneDrive

mod local;
mod rsync;
mod restic;
mod cloud;

pub use local::LocalBackend;
pub use rsync::RsyncBackend;
pub use restic::ResticBackend;
pub use cloud::{CloudBackend, CloudProvider};

use anyhow::Result;
use std::path::Path;

/// Backup metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub backup_type: BackupType,
    pub size_bytes: u64,
    pub file_count: u64,
    pub compression: CompressionType,
    pub encrypted: bool,
    pub verified: bool,
    pub tags: Vec<String>,
}

/// Types of backups
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BackupType {
    System,
    Home,
    Custom,
    Config,
}

/// Compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompressionType {
    None,
    Lz4,
    Zstd,
    Lzma,
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(BackupProgress) + Send + Sync>;

/// Backup progress information
#[derive(Debug, Clone)]
pub struct BackupProgress {
    pub current_file: String,
    pub files_processed: u64,
    pub files_total: u64,
    pub bytes_processed: u64,
    pub bytes_total: u64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: Option<u64>,
    pub phase: BackupPhase,
}

/// Backup phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupPhase {
    Scanning,
    Backing,
    Compressing,
    Encrypting,
    Verifying,
    Cleaning,
    Complete,
    Failed,
}

/// Common trait for all backup backends
pub trait BackupBackend: Send + Sync {
    /// Get backend name
    fn name(&self) -> &str;

    /// Check if backend is available/configured
    fn is_available(&self) -> bool;

    /// Test connection to backend
    fn test_connection(&self) -> Result<()>;

    /// List available backups
    fn list_backups(&self) -> Result<Vec<BackupMetadata>>;

    /// Create a new backup
    fn create_backup(
        &self,
        sources: &[&Path],
        name: &str,
        backup_type: BackupType,
        compression: CompressionType,
        encrypt: bool,
        progress: Option<ProgressCallback>,
    ) -> Result<BackupMetadata>;

    /// Restore a backup
    fn restore_backup(
        &self,
        backup_id: &str,
        destination: &Path,
        files: Option<&[&Path]>,
        progress: Option<ProgressCallback>,
    ) -> Result<()>;

    /// Delete a backup
    fn delete_backup(&self, backup_id: &str) -> Result<()>;

    /// Verify backup integrity
    fn verify_backup(&self, backup_id: &str) -> Result<bool>;

    /// Get backup details
    fn get_backup(&self, backup_id: &str) -> Result<Option<BackupMetadata>>;

    /// List files in a backup
    fn list_files(&self, backup_id: &str, path: Option<&Path>) -> Result<Vec<FileEntry>>;

    /// Get storage usage
    fn get_storage_usage(&self) -> Result<StorageUsage>;
}

/// File entry in a backup
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub permissions: u32,
}

/// Storage usage information
#[derive(Debug, Clone)]
pub struct StorageUsage {
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub backup_count: u64,
}

/// Backup configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupConfig {
    pub sources: Vec<String>,
    pub exclusions: Vec<String>,
    pub compression: CompressionType,
    pub encrypt: bool,
    pub verify_after: bool,
    pub incremental: bool,
    pub deduplication: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            sources: vec![],
            exclusions: vec![
                "*.tmp".to_string(),
                "**/cache/**".to_string(),
                "**/.cache/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/__pycache__/**".to_string(),
                "*.log".to_string(),
            ],
            compression: CompressionType::Zstd,
            encrypt: false,
            verify_after: true,
            incremental: true,
            deduplication: true,
        }
    }
}
