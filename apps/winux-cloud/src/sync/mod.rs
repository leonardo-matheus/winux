//! Synchronization engine module
//!
//! Handles bidirectional synchronization between local and cloud storage:
//! - Delta sync for efficient transfers
//! - Conflict resolution
//! - File system watching
//! - Background sync daemon

mod engine;
mod conflict;
mod delta;
mod watcher;

pub use engine::{SyncEngine, SyncStatus, SyncDirection, SyncConfig};
pub use conflict::{ConflictResolver, ConflictResolution, ConflictStrategy};
pub use delta::{DeltaSync, DeltaEntry, DeltaAction};
pub use watcher::{FileWatcher, FileEvent, FileEventKind};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Sync state for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// Local file path
    pub local_path: String,
    /// Remote file ID
    pub remote_id: String,
    /// Provider name
    pub provider: String,
    /// Last local modification time
    pub local_modified: DateTime<Utc>,
    /// Last remote modification time
    pub remote_modified: DateTime<Utc>,
    /// Local content hash
    pub local_hash: Option<String>,
    /// Remote content hash
    pub remote_hash: Option<String>,
    /// Current sync status
    pub status: FileSyncStatus,
    /// Last sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Version number
    pub version: u64,
}

/// Sync status for individual files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSyncStatus {
    /// File is synced
    Synced,
    /// File is pending upload
    PendingUpload,
    /// File is pending download
    PendingDownload,
    /// File is currently syncing
    Syncing,
    /// File has conflict
    Conflict,
    /// Sync error
    Error,
    /// File is ignored
    Ignored,
}

/// Sync event for activity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    /// Event ID
    pub id: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: SyncEventType,
    /// File path
    pub path: String,
    /// File name
    pub name: String,
    /// Provider name
    pub provider: String,
    /// Bytes transferred (if applicable)
    pub bytes: Option<u64>,
    /// Error message (if applicable)
    pub error: Option<String>,
}

/// Types of sync events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncEventType {
    Upload,
    Download,
    Delete,
    Move,
    Rename,
    ConflictResolved,
    Error,
}

/// Sync statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    /// Files uploaded today
    pub files_uploaded_today: u64,
    /// Files downloaded today
    pub files_downloaded_today: u64,
    /// Bytes uploaded today
    pub bytes_uploaded_today: u64,
    /// Bytes downloaded today
    pub bytes_downloaded_today: u64,
    /// Total files synced
    pub total_files_synced: u64,
    /// Total bytes transferred
    pub total_bytes_transferred: u64,
    /// Conflicts resolved
    pub conflicts_resolved: u64,
    /// Errors encountered
    pub errors: u64,
}
