//! File transfer service
//!
//! Handles bidirectional file transfers between PC and phone:
//! - Send files to phone
//! - Receive files from phone
//! - Progress tracking
//! - Resume support

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// File transfer service
pub struct FileTransferService {
    running: Arc<RwLock<bool>>,
    transfers: Arc<RwLock<HashMap<String, FileTransfer>>>,
    download_dir: PathBuf,
    auto_accept: bool,
}

impl FileTransferService {
    pub fn new() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Winux Connect");

        Self {
            running: Arc::new(RwLock::new(false)),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            download_dir,
            auto_accept: false,
        }
    }

    /// Start the file transfer service
    pub fn start(&self) -> Result<(), String> {
        *self.running.write().unwrap() = true;

        // Create download directory if it doesn't exist
        std::fs::create_dir_all(&self.download_dir)
            .map_err(|e| format!("Failed to create download directory: {}", e))?;

        // In production, would start TCP listener for incoming transfers

        tracing::info!("File transfer service started");
        Ok(())
    }

    /// Stop the file transfer service
    pub fn stop(&self) {
        *self.running.write().unwrap() = false;
        tracing::info!("File transfer service stopped");
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        *self.running.read().unwrap()
    }

    /// Get download directory
    pub fn get_download_dir(&self) -> PathBuf {
        self.download_dir.clone()
    }

    /// Set download directory
    pub fn set_download_dir(&mut self, path: PathBuf) {
        self.download_dir = path;
    }

    /// Set auto-accept mode
    pub fn set_auto_accept(&mut self, auto_accept: bool) {
        self.auto_accept = auto_accept;
    }

    /// Send a file to a device
    pub fn send_file(&self, device_id: &str, file_path: &str) -> Result<String, String> {
        let path = PathBuf::from(file_path);

        if !path.exists() {
            return Err("File does not exist".to_string());
        }

        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let transfer_id = uuid::Uuid::new_v4().to_string();

        let transfer = FileTransfer {
            id: transfer_id.clone(),
            filename: filename.to_string(),
            file_path: file_path.to_string(),
            size: metadata.len(),
            transferred: 0,
            status: TransferStatus::Pending,
            direction: TransferDirection::Upload,
            device_id: device_id.to_string(),
            error: None,
        };

        self.transfers.write().unwrap().insert(transfer_id.clone(), transfer);

        tracing::info!("Queued file for upload: {} to {}", filename, device_id);

        // In production, would initiate TCP connection and start transfer
        Ok(transfer_id)
    }

    /// Send multiple files
    pub fn send_files(&self, device_id: &str, file_paths: &[&str]) -> Vec<Result<String, String>> {
        file_paths.iter()
            .map(|path| self.send_file(device_id, path))
            .collect()
    }

    /// Receive a file (called when transfer is accepted)
    pub fn receive_file(&self, transfer_id: &str) -> Result<(), String> {
        if let Some(transfer) = self.transfers.write().unwrap().get_mut(transfer_id) {
            transfer.status = TransferStatus::InProgress;
            tracing::info!("Receiving file: {}", transfer.filename);
            Ok(())
        } else {
            Err("Transfer not found".to_string())
        }
    }

    /// Cancel a transfer
    pub fn cancel_transfer(&self, transfer_id: &str) -> Result<(), String> {
        if let Some(transfer) = self.transfers.write().unwrap().get_mut(transfer_id) {
            transfer.status = TransferStatus::Cancelled;
            tracing::info!("Cancelled transfer: {}", transfer_id);
            Ok(())
        } else {
            Err("Transfer not found".to_string())
        }
    }

    /// Get transfer status
    pub fn get_transfer(&self, transfer_id: &str) -> Option<FileTransfer> {
        self.transfers.read().unwrap().get(transfer_id).cloned()
    }

    /// Get all transfers
    pub fn get_transfers(&self) -> Vec<FileTransfer> {
        self.transfers.read().unwrap().values().cloned().collect()
    }

    /// Get active transfers
    pub fn get_active_transfers(&self) -> Vec<FileTransfer> {
        self.transfers.read().unwrap()
            .values()
            .filter(|t| matches!(t.status, TransferStatus::InProgress | TransferStatus::Pending))
            .cloned()
            .collect()
    }

    /// Update transfer progress (called from transfer handler)
    pub fn update_progress(&self, transfer_id: &str, transferred: u64) {
        if let Some(transfer) = self.transfers.write().unwrap().get_mut(transfer_id) {
            transfer.transferred = transferred;
        }
    }

    /// Mark transfer as complete
    pub fn complete_transfer(&self, transfer_id: &str) {
        if let Some(transfer) = self.transfers.write().unwrap().get_mut(transfer_id) {
            transfer.status = TransferStatus::Completed;
            transfer.transferred = transfer.size;
            tracing::info!("Transfer completed: {}", transfer.filename);
        }
    }

    /// Mark transfer as failed
    pub fn fail_transfer(&self, transfer_id: &str, error: &str) {
        if let Some(transfer) = self.transfers.write().unwrap().get_mut(transfer_id) {
            transfer.status = TransferStatus::Failed;
            transfer.error = Some(error.to_string());
            tracing::error!("Transfer failed: {} - {}", transfer.filename, error);
        }
    }

    /// Clear completed/failed transfers
    pub fn clear_finished(&self) {
        self.transfers.write().unwrap().retain(|_, t| {
            !matches!(t.status, TransferStatus::Completed | TransferStatus::Failed | TransferStatus::Cancelled)
        });
    }

    /// Handle incoming file transfer request
    pub fn handle_incoming_request(&self, device_id: &str, filename: &str, size: u64) -> Result<String, String> {
        let transfer_id = uuid::Uuid::new_v4().to_string();

        let dest_path = self.download_dir.join(filename);

        let transfer = FileTransfer {
            id: transfer_id.clone(),
            filename: filename.to_string(),
            file_path: dest_path.to_string_lossy().to_string(),
            size,
            transferred: 0,
            status: if self.auto_accept { TransferStatus::InProgress } else { TransferStatus::Pending },
            direction: TransferDirection::Download,
            device_id: device_id.to_string(),
            error: None,
        };

        self.transfers.write().unwrap().insert(transfer_id.clone(), transfer);

        tracing::info!("Incoming file transfer: {} from {} ({} bytes)", filename, device_id, size);

        Ok(transfer_id)
    }
}

impl Default for FileTransferService {
    fn default() -> Self {
        Self::new()
    }
}

/// File transfer data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileTransfer {
    pub id: String,
    pub filename: String,
    pub file_path: String,
    pub size: u64,
    pub transferred: u64,
    pub status: TransferStatus,
    pub direction: TransferDirection,
    pub device_id: String,
    pub error: Option<String>,
}

impl FileTransfer {
    /// Get transfer progress as percentage
    pub fn progress(&self) -> f64 {
        if self.size == 0 {
            0.0
        } else {
            (self.transferred as f64 / self.size as f64) * 100.0
        }
    }

    /// Get human-readable size
    pub fn size_human(&self) -> String {
        format_size(self.size)
    }

    /// Get human-readable transferred
    pub fn transferred_human(&self) -> String {
        format_size(self.transferred)
    }
}

/// Transfer status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Transfer direction
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferDirection {
    Upload,
    Download,
}

/// Format file size in human-readable format
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_transfer_service_creation() {
        let service = FileTransferService::new();
        assert!(!service.is_running());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_transfer_progress() {
        let transfer = FileTransfer {
            id: "test".to_string(),
            filename: "test.txt".to_string(),
            file_path: "/tmp/test.txt".to_string(),
            size: 1000,
            transferred: 500,
            status: TransferStatus::InProgress,
            direction: TransferDirection::Upload,
            device_id: "device1".to_string(),
            error: None,
        };

        assert_eq!(transfer.progress(), 50.0);
    }

    #[test]
    fn test_handle_incoming_request() {
        let service = FileTransferService::new();
        let result = service.handle_incoming_request("device1", "test.txt", 1024);
        assert!(result.is_ok());

        let transfers = service.get_transfers();
        assert_eq!(transfers.len(), 1);
    }
}
