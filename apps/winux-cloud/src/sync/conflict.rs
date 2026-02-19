//! Conflict resolution module

use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::providers::CloudFile;
use super::SyncState;

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// Keep both files (create conflict copy)
    KeepBoth,
    /// Local file wins (upload local)
    LocalWins,
    /// Remote file wins (download remote)
    RemoteWins,
    /// Ask user every time
    AskUser,
}

/// Conflict resolution result
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// Keep local version
    KeepLocal,
    /// Keep remote version
    KeepRemote,
    /// Keep both with suffixes
    KeepBoth {
        local_suffix: String,
        remote_suffix: String,
    },
    /// Ask user to decide
    AskUser,
}

/// Conflict information
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Local file path
    pub local_path: String,
    /// Remote file ID
    pub remote_id: String,
    /// Provider name
    pub provider: String,
    /// Local modification time
    pub local_modified: chrono::DateTime<Utc>,
    /// Remote modification time
    pub remote_modified: chrono::DateTime<Utc>,
    /// Local file size
    pub local_size: u64,
    /// Remote file size
    pub remote_size: u64,
    /// Local content hash
    pub local_hash: Option<String>,
    /// Remote content hash
    pub remote_hash: Option<String>,
}

/// Conflict resolver
pub struct ConflictResolver {
    strategy: ConflictStrategy,
}

impl ConflictResolver {
    /// Create a new conflict resolver with the given strategy
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }

    /// Resolve a conflict between local and remote versions
    pub fn resolve(&self, local: &SyncState, remote: &CloudFile) -> Result<ConflictResolution> {
        // Check if files are actually different
        if let (Some(local_hash), Some(remote_hash)) = (&local.local_hash, &remote.hash) {
            if local_hash == remote_hash {
                // Files are identical - no conflict
                return Ok(ConflictResolution::KeepLocal);
            }
        }

        match self.strategy {
            ConflictStrategy::KeepBoth => {
                let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                Ok(ConflictResolution::KeepBoth {
                    local_suffix: format!("_conflito_local_{}", timestamp),
                    remote_suffix: format!("_conflito_remoto_{}", timestamp),
                })
            }
            ConflictStrategy::LocalWins => {
                Ok(ConflictResolution::KeepLocal)
            }
            ConflictStrategy::RemoteWins => {
                Ok(ConflictResolution::KeepRemote)
            }
            ConflictStrategy::AskUser => {
                Ok(ConflictResolution::AskUser)
            }
        }
    }

    /// Resolve conflict automatically based on timestamps
    pub fn resolve_by_timestamp(&self, local: &SyncState, remote: &CloudFile) -> ConflictResolution {
        if local.local_modified > remote.modified_at {
            ConflictResolution::KeepLocal
        } else {
            ConflictResolution::KeepRemote
        }
    }

    /// Resolve conflict automatically based on file size
    /// Keeps the larger file (assumption: more content = more recent work)
    pub fn resolve_by_size(&self, local_size: u64, remote_size: u64) -> ConflictResolution {
        if local_size >= remote_size {
            ConflictResolution::KeepLocal
        } else {
            ConflictResolution::KeepRemote
        }
    }

    /// Get the current strategy
    pub fn strategy(&self) -> ConflictStrategy {
        self.strategy
    }

    /// Set a new strategy
    pub fn set_strategy(&mut self, strategy: ConflictStrategy) {
        self.strategy = strategy;
    }

    /// Create a conflict copy filename
    pub fn create_conflict_filename(original: &str, suffix: &str) -> String {
        let path = std::path::Path::new(original);
        let stem = path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let extension = path.extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();

        format!("{}{}{}", stem, suffix, extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_filename() {
        assert_eq!(
            ConflictResolver::create_conflict_filename("document.pdf", "_conflict"),
            "document_conflict.pdf"
        );

        assert_eq!(
            ConflictResolver::create_conflict_filename("file", "_v2"),
            "file_v2"
        );

        assert_eq!(
            ConflictResolver::create_conflict_filename("archive.tar.gz", "_backup"),
            "archive.tar_backup.gz"
        );
    }
}
