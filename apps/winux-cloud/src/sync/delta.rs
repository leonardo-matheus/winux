//! Delta sync module - Efficient incremental synchronization

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::database::Database;
use super::SyncState;

/// Delta entry representing a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEntry {
    /// File path
    pub path: String,
    /// Action to take
    pub action: DeltaAction,
    /// File hash
    pub hash: Option<String>,
    /// File size
    pub size: u64,
    /// Modification time
    pub modified: DateTime<Utc>,
    /// Is directory
    pub is_dir: bool,
}

/// Actions for delta sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaAction {
    /// File was created
    Create,
    /// File was modified
    Modify,
    /// File was deleted
    Delete,
    /// File was moved/renamed
    Move,
    /// No change (for verification)
    NoChange,
}

/// Delta sync engine for efficient synchronization
pub struct DeltaSync {
    database: Arc<Database>,
}

impl DeltaSync {
    /// Create a new delta sync engine
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Calculate local changes since last sync
    pub fn calculate_local_delta(&self, folder: &Path) -> Result<Vec<DeltaEntry>> {
        let mut changes = Vec::new();

        // Walk the directory
        for entry in walkdir::WalkDir::new(folder)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(folder)?.to_string_lossy().to_string();

            let metadata = entry.metadata()?;
            let is_dir = metadata.is_dir();

            // Get stored state
            let stored = self.database.get_sync_state_by_path(&relative_path)?;

            if is_dir {
                // Directory handling
                if stored.is_none() {
                    changes.push(DeltaEntry {
                        path: relative_path,
                        action: DeltaAction::Create,
                        hash: None,
                        size: 0,
                        modified: Utc::now(),
                        is_dir: true,
                    });
                }
            } else {
                // File handling
                let file_hash = self.calculate_file_hash(path)?;
                let modified: DateTime<Utc> = metadata.modified()?.into();
                let size = metadata.len();

                match stored {
                    Some(state) => {
                        // Check if file changed
                        if state.local_hash.as_ref() != Some(&file_hash) {
                            changes.push(DeltaEntry {
                                path: relative_path,
                                action: DeltaAction::Modify,
                                hash: Some(file_hash),
                                size,
                                modified,
                                is_dir: false,
                            });
                        }
                    }
                    None => {
                        // New file
                        changes.push(DeltaEntry {
                            path: relative_path,
                            action: DeltaAction::Create,
                            hash: Some(file_hash),
                            size,
                            modified,
                            is_dir: false,
                        });
                    }
                }
            }
        }

        // Check for deleted files
        let stored_files = self.database.get_all_sync_states()?;
        for state in stored_files {
            let full_path = folder.join(&state.local_path);
            if !full_path.exists() {
                changes.push(DeltaEntry {
                    path: state.local_path,
                    action: DeltaAction::Delete,
                    hash: state.local_hash,
                    size: 0,
                    modified: Utc::now(),
                    is_dir: false,
                });
            }
        }

        Ok(changes)
    }

    /// Calculate file hash (SHA256)
    pub fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};

        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();

        Ok(hex::encode(result))
    }

    /// Calculate quick hash for large files (first + last chunks)
    pub fn calculate_quick_hash(&self, path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        use std::io::{Read, Seek, SeekFrom};

        let mut file = std::fs::File::open(path)?;
        let metadata = file.metadata()?;
        let size = metadata.len();

        const CHUNK_SIZE: u64 = 1024 * 1024; // 1MB chunks

        let mut hasher = Sha256::new();

        if size <= CHUNK_SIZE * 2 {
            // Small file - hash everything
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;
            hasher.update(&content);
        } else {
            // Large file - hash first and last chunks plus size
            let mut buffer = vec![0u8; CHUNK_SIZE as usize];

            // First chunk
            file.read_exact(&mut buffer)?;
            hasher.update(&buffer);

            // Last chunk
            file.seek(SeekFrom::End(-(CHUNK_SIZE as i64)))?;
            file.read_exact(&mut buffer)?;
            hasher.update(&buffer);

            // Include size in hash
            hasher.update(size.to_le_bytes());
        }

        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Merge local and remote deltas
    pub fn merge_deltas(
        &self,
        local: &[DeltaEntry],
        remote: &[DeltaEntry],
    ) -> Vec<MergedDelta> {
        let mut merged = Vec::new();
        let mut local_map: HashMap<String, &DeltaEntry> = HashMap::new();
        let mut remote_map: HashMap<String, &DeltaEntry> = HashMap::new();

        for entry in local {
            local_map.insert(entry.path.clone(), entry);
        }

        for entry in remote {
            remote_map.insert(entry.path.clone(), entry);
        }

        // Process all paths
        let all_paths: std::collections::HashSet<String> = local_map.keys()
            .chain(remote_map.keys())
            .cloned()
            .collect();

        for path in all_paths {
            let local_entry = local_map.get(&path);
            let remote_entry = remote_map.get(&path);

            let result = match (local_entry, remote_entry) {
                (Some(l), None) => {
                    // Only local change
                    MergedDelta {
                        path: path.clone(),
                        action: MergedAction::Upload,
                        local: Some((*l).clone()),
                        remote: None,
                        has_conflict: false,
                    }
                }
                (None, Some(r)) => {
                    // Only remote change
                    MergedDelta {
                        path: path.clone(),
                        action: MergedAction::Download,
                        local: None,
                        remote: Some((*r).clone()),
                        has_conflict: false,
                    }
                }
                (Some(l), Some(r)) => {
                    // Both changed - check for conflict
                    let has_conflict = l.action != DeltaAction::Delete
                        && r.action != DeltaAction::Delete
                        && l.hash != r.hash;

                    let action = if has_conflict {
                        MergedAction::Conflict
                    } else if l.action == DeltaAction::Delete {
                        MergedAction::DeleteRemote
                    } else if r.action == DeltaAction::Delete {
                        MergedAction::DeleteLocal
                    } else if l.modified > r.modified {
                        MergedAction::Upload
                    } else {
                        MergedAction::Download
                    };

                    MergedDelta {
                        path: path.clone(),
                        action,
                        local: Some((*l).clone()),
                        remote: Some((*r).clone()),
                        has_conflict,
                    }
                }
                (None, None) => continue,
            };

            merged.push(result);
        }

        merged
    }

    /// Apply delta to database
    pub fn apply_delta(&self, delta: &DeltaEntry, state: &SyncState) -> Result<()> {
        match delta.action {
            DeltaAction::Create | DeltaAction::Modify => {
                self.database.update_sync_state(state)?;
            }
            DeltaAction::Delete => {
                self.database.delete_sync_state(&delta.path)?;
            }
            DeltaAction::Move => {
                // Handle in higher level
            }
            DeltaAction::NoChange => {
                // Nothing to do
            }
        }

        Ok(())
    }
}

/// Merged delta result
#[derive(Debug, Clone)]
pub struct MergedDelta {
    /// File path
    pub path: String,
    /// Action to take
    pub action: MergedAction,
    /// Local delta entry
    pub local: Option<DeltaEntry>,
    /// Remote delta entry
    pub remote: Option<DeltaEntry>,
    /// Whether there's a conflict
    pub has_conflict: bool,
}

/// Merged action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergedAction {
    /// Upload local to remote
    Upload,
    /// Download remote to local
    Download,
    /// Delete from remote
    DeleteRemote,
    /// Delete from local
    DeleteLocal,
    /// Conflict needs resolution
    Conflict,
    /// No action needed
    None,
}
