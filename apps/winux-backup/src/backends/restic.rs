//! Restic backup backend - Incremental, deduplicated backups

use super::*;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Restic backend configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResticConfig {
    pub repository: String,
    pub password: String,
    pub password_file: Option<String>,
    pub cache_dir: Option<String>,
}

impl Default for ResticConfig {
    fn default() -> Self {
        Self {
            repository: String::new(),
            password: String::new(),
            password_file: None,
            cache_dir: None,
        }
    }
}

/// Restic backup backend - incremental backups with deduplication
pub struct ResticBackend {
    config: ResticConfig,
}

/// Restic snapshot information
#[derive(Debug, serde::Deserialize)]
struct ResticSnapshot {
    id: String,
    time: String,
    hostname: String,
    tags: Option<Vec<String>>,
    paths: Vec<String>,
}

/// Restic stats
#[derive(Debug, serde::Deserialize)]
struct ResticStats {
    total_size: u64,
    total_file_count: u64,
}

impl ResticBackend {
    /// Create a new restic backend
    pub fn new(config: ResticConfig) -> Self {
        Self { config }
    }

    /// Build base restic command with authentication
    fn restic_cmd(&self) -> Command {
        let mut cmd = Command::new("restic");

        cmd.arg("-r").arg(&self.config.repository);

        if let Some(ref password_file) = self.config.password_file {
            cmd.arg("--password-file").arg(password_file);
        } else {
            cmd.env("RESTIC_PASSWORD", &self.config.password);
        }

        if let Some(ref cache_dir) = self.config.cache_dir {
            cmd.arg("--cache-dir").arg(cache_dir);
        }

        cmd
    }

    /// Execute restic command and return output
    fn exec(&self, args: &[&str]) -> Result<String> {
        let mut cmd = self.restic_cmd();
        cmd.args(args);

        let output = cmd.output().context("Failed to execute restic")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!(
                "Restic command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Execute restic command with JSON output
    fn exec_json<T: serde::de::DeserializeOwned>(&self, args: &[&str]) -> Result<T> {
        let mut full_args = vec!["--json"];
        full_args.extend(args);
        let output = self.exec(&full_args)?;
        serde_json::from_str(&output).context("Failed to parse restic JSON output")
    }

    /// Initialize repository if it doesn't exist
    pub fn init_repository(&self) -> Result<()> {
        let mut cmd = self.restic_cmd();
        cmd.arg("init");

        let output = cmd.output()?;

        if output.status.success() || String::from_utf8_lossy(&output.stderr).contains("already initialized") {
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to initialize repository: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Convert restic snapshot to backup metadata
    fn snapshot_to_metadata(&self, snapshot: &ResticSnapshot) -> BackupMetadata {
        let timestamp = chrono::DateTime::parse_from_rfc3339(&snapshot.time)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let name = snapshot.tags
            .as_ref()
            .and_then(|t| t.first())
            .cloned()
            .unwrap_or_else(|| format!("Snapshot {}", &snapshot.id[..8]));

        let backup_type = if snapshot.paths.iter().any(|p| p == "/") {
            BackupType::System
        } else if snapshot.paths.iter().any(|p| p.contains("/home")) {
            BackupType::Home
        } else if snapshot.paths.iter().any(|p| p.contains(".config")) {
            BackupType::Config
        } else {
            BackupType::Custom
        };

        BackupMetadata {
            id: snapshot.id.clone(),
            name,
            timestamp,
            backup_type,
            size_bytes: 0, // Will be filled by stats
            file_count: 0,
            compression: CompressionType::Zstd, // Restic uses zstd by default
            encrypted: true, // Restic always encrypts
            verified: true,
            tags: snapshot.tags.clone().unwrap_or_default(),
        }
    }
}

impl BackupBackend for ResticBackend {
    fn name(&self) -> &str {
        "Restic"
    }

    fn is_available(&self) -> bool {
        Command::new("restic")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn test_connection(&self) -> Result<()> {
        // Try to get snapshots, which tests both connection and authentication
        let result = self.exec(&["snapshots", "--latest", "1"]);

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // If repository doesn't exist, try to initialize it
                if e.to_string().contains("repository does not exist") {
                    self.init_repository()
                } else {
                    Err(e)
                }
            }
        }
    }

    fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let snapshots: Vec<ResticSnapshot> = self.exec_json(&["snapshots"])?;

        let mut backups: Vec<BackupMetadata> = snapshots
            .iter()
            .map(|s| self.snapshot_to_metadata(s))
            .collect();

        // Get stats for each snapshot
        for backup in &mut backups {
            if let Ok(stats) = self.exec_json::<ResticStats>(&["stats", &backup.id]) {
                backup.size_bytes = stats.total_size;
                backup.file_count = stats.total_file_count;
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
        _compression: CompressionType, // Restic handles compression internally
        _encrypt: bool, // Restic always encrypts
        progress: Option<ProgressCallback>,
    ) -> Result<BackupMetadata> {
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

        let mut cmd = self.restic_cmd();
        cmd.arg("backup")
            .arg("--tag").arg(name)
            .arg("--json");

        for source in sources {
            cmd.arg(source);
        }

        // Report backing phase
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Creating backup...".to_string(),
                files_processed: 0,
                files_total: 0,
                bytes_processed: 0,
                bytes_total: 0,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
                phase: BackupPhase::Backing,
            });
        }

        let output = cmd.output().context("Failed to execute restic backup")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Restic backup failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Parse output to get snapshot ID
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut snapshot_id = String::new();
        let mut files_new = 0u64;
        let mut files_changed = 0u64;
        let mut data_added = 0u64;

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(id) = json.get("snapshot_id").and_then(|v| v.as_str()) {
                    snapshot_id = id.to_string();
                }
                if let Some(stats) = json.get("files_new").and_then(|v| v.as_u64()) {
                    files_new = stats;
                }
                if let Some(stats) = json.get("files_changed").and_then(|v| v.as_u64()) {
                    files_changed = stats;
                }
                if let Some(stats) = json.get("data_added").and_then(|v| v.as_u64()) {
                    data_added = stats;
                }
            }
        }

        let metadata = BackupMetadata {
            id: snapshot_id,
            name: name.to_string(),
            timestamp: chrono::Utc::now(),
            backup_type,
            size_bytes: data_added,
            file_count: files_new + files_changed,
            compression: CompressionType::Zstd,
            encrypted: true,
            verified: true,
            tags: vec![name.to_string()],
        };

        // Report complete
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Complete".to_string(),
                files_processed: metadata.file_count,
                files_total: metadata.file_count,
                bytes_processed: metadata.size_bytes,
                bytes_total: metadata.size_bytes,
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
        // Report restore phase
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Restoring...".to_string(),
                files_processed: 0,
                files_total: 0,
                bytes_processed: 0,
                bytes_total: 0,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
                phase: BackupPhase::Backing,
            });
        }

        let mut cmd = self.restic_cmd();
        cmd.arg("restore")
            .arg(backup_id)
            .arg("--target").arg(destination);

        // Add file includes if specified
        if let Some(filter_files) = files {
            for file in filter_files {
                cmd.arg("--include").arg(file.display().to_string());
            }
        }

        let output = cmd.output().context("Failed to execute restic restore")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Restic restore failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Report complete
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Complete".to_string(),
                files_processed: 0,
                files_total: 0,
                bytes_processed: 0,
                bytes_total: 0,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
                phase: BackupPhase::Complete,
            });
        }

        Ok(())
    }

    fn delete_backup(&self, backup_id: &str) -> Result<()> {
        self.exec(&["forget", backup_id])?;
        self.exec(&["prune"])?;
        Ok(())
    }

    fn verify_backup(&self, backup_id: &str) -> Result<bool> {
        let result = self.exec(&["check", "--read-data-subset=1%"]);
        Ok(result.is_ok())
    }

    fn get_backup(&self, backup_id: &str) -> Result<Option<BackupMetadata>> {
        let snapshots: Vec<ResticSnapshot> = self.exec_json(&["snapshots", backup_id])?;

        if let Some(snapshot) = snapshots.first() {
            let mut metadata = self.snapshot_to_metadata(snapshot);

            // Get stats
            if let Ok(stats) = self.exec_json::<ResticStats>(&["stats", backup_id]) {
                metadata.size_bytes = stats.total_size;
                metadata.file_count = stats.total_file_count;
            }

            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    fn list_files(&self, backup_id: &str, path: Option<&Path>) -> Result<Vec<FileEntry>> {
        let mut args = vec!["ls", "--json", backup_id];

        let path_str;
        if let Some(p) = path {
            path_str = p.display().to_string();
            args.push(&path_str);
        }

        let output = self.exec(&args)?;

        let mut entries = Vec::new();

        for line in output.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(struct_type) = json.get("struct_type").and_then(|v| v.as_str()) {
                    if struct_type == "node" {
                        let path = json.get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let node_type = json.get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("file");

                        let size = json.get("size")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        let mtime = json.get("mtime")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|| chrono::Utc::now());

                        let mode = json.get("mode")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0o644) as u32;

                        entries.push(FileEntry {
                            path,
                            is_dir: node_type == "dir",
                            size,
                            modified: mtime,
                            permissions: mode,
                        });
                    }
                }
            }
        }

        Ok(entries)
    }

    fn get_storage_usage(&self) -> Result<StorageUsage> {
        #[derive(Debug, serde::Deserialize)]
        struct RepoStats {
            total_size: u64,
            total_file_count: u64,
        }

        let stats: RepoStats = self.exec_json(&["stats", "--mode", "raw-data"])?;
        let snapshots: Vec<ResticSnapshot> = self.exec_json(&["snapshots"])?;

        Ok(StorageUsage {
            used_bytes: stats.total_size,
            available_bytes: 0, // Depends on backend
            total_bytes: 0,
            backup_count: snapshots.len() as u64,
        })
    }
}

impl ResticBackend {
    /// Apply retention policy and prune old snapshots
    pub fn apply_retention(
        &self,
        keep_daily: u32,
        keep_weekly: u32,
        keep_monthly: u32,
        keep_yearly: u32,
    ) -> Result<()> {
        let mut cmd = self.restic_cmd();
        cmd.arg("forget")
            .arg("--prune")
            .arg("--keep-daily").arg(keep_daily.to_string())
            .arg("--keep-weekly").arg(keep_weekly.to_string())
            .arg("--keep-monthly").arg(keep_monthly.to_string())
            .arg("--keep-yearly").arg(keep_yearly.to_string());

        let output = cmd.output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to apply retention: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Check repository integrity
    pub fn check_repository(&self) -> Result<bool> {
        let result = self.exec(&["check"]);
        Ok(result.is_ok())
    }

    /// Unlock repository (remove stale locks)
    pub fn unlock(&self) -> Result<()> {
        self.exec(&["unlock"])?;
        Ok(())
    }
}
