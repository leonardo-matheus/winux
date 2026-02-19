//! Archive operations module

pub mod extract;
pub mod create;
pub mod add;
pub mod test;

use std::path::PathBuf;

/// Options for extraction operations
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    /// Destination directory
    pub destination: PathBuf,
    /// Overwrite existing files
    pub overwrite: bool,
    /// Preserve file permissions
    pub preserve_permissions: bool,
    /// Password for encrypted archives
    pub password: Option<String>,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            destination: std::env::current_dir().unwrap_or_default(),
            overwrite: true,
            preserve_permissions: true,
            password: None,
        }
    }
}

/// Options for creating archives
#[derive(Debug, Clone)]
pub struct CreateOptions {
    /// Output file path
    pub output_path: PathBuf,
    /// Archive format
    pub format: crate::archive::ArchiveFormat,
    /// Compression level (1-9)
    pub compression_level: u8,
    /// Password for encryption
    pub password: Option<String>,
    /// Split into volumes of this size (in bytes)
    pub split_size: Option<u64>,
    /// Archive comment
    pub comment: Option<String>,
    /// Store paths relative to this directory
    pub base_path: Option<PathBuf>,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            output_path: PathBuf::new(),
            format: crate::archive::ArchiveFormat::Zip,
            compression_level: 6,
            password: None,
            split_size: None,
            comment: None,
            base_path: None,
        }
    }
}

/// Progress callback for operations
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + Sync>;

/// Progress information
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Current file being processed
    pub current_file: String,
    /// Current file number
    pub current_index: usize,
    /// Total number of files
    pub total_files: usize,
    /// Bytes processed
    pub bytes_processed: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Operation status
    pub status: OperationStatus,
}

/// Operation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatus {
    /// Operation in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation failed with error
    Failed(String),
    /// Operation was cancelled
    Cancelled,
}

impl ProgressInfo {
    /// Calculate progress percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            if self.total_files == 0 {
                return 0.0;
            }
            return (self.current_index as f64 / self.total_files as f64) * 100.0;
        }
        (self.bytes_processed as f64 / self.total_bytes as f64) * 100.0
    }
}
