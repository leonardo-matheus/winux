//! Cloud storage backup backend - Google Drive, Dropbox, OneDrive

use super::*;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Supported cloud providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CloudProvider {
    GoogleDrive,
    Dropbox,
    OneDrive,
}

impl CloudProvider {
    /// Get the rclone remote type for this provider
    fn rclone_type(&self) -> &'static str {
        match self {
            CloudProvider::GoogleDrive => "drive",
            CloudProvider::Dropbox => "dropbox",
            CloudProvider::OneDrive => "onedrive",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            CloudProvider::GoogleDrive => "Google Drive",
            CloudProvider::Dropbox => "Dropbox",
            CloudProvider::OneDrive => "OneDrive",
        }
    }
}

/// Cloud backend configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CloudConfig {
    pub provider: CloudProvider,
    pub remote_name: String,
    pub remote_path: String,
    pub bandwidth_limit: Option<u64>, // KB/s
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            provider: CloudProvider::GoogleDrive,
            remote_name: String::new(),
            remote_path: "/Backups/Winux".to_string(),
            bandwidth_limit: None,
        }
    }
}

/// Cloud backup backend using rclone
pub struct CloudBackend {
    config: CloudConfig,
    local_cache: PathBuf,
}

impl CloudBackend {
    /// Create a new cloud backend
    pub fn new(config: CloudConfig, local_cache: impl Into<PathBuf>) -> Self {
        Self {
            config,
            local_cache: local_cache.into(),
        }
    }

    /// Get the full remote path
    fn remote_path(&self, backup_id: &str) -> String {
        format!("{}:{}/{}", self.config.remote_name, self.config.remote_path, backup_id)
    }

    /// Get the base remote path
    fn base_remote_path(&self) -> String {
        format!("{}:{}", self.config.remote_name, self.config.remote_path)
    }

    /// Build base rclone command
    fn rclone_cmd(&self) -> Command {
        let mut cmd = Command::new("rclone");

        if let Some(limit) = self.config.bandwidth_limit {
            cmd.arg("--bwlimit").arg(format!("{}k", limit));
        }

        cmd.arg("--progress");

        cmd
    }

    /// Execute rclone command
    fn exec(&self, args: &[&str]) -> Result<String> {
        let mut cmd = self.rclone_cmd();
        cmd.args(args);

        let output = cmd.output().context("Failed to execute rclone")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!(
                "Rclone command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Generate a unique backup ID
    fn generate_id(&self) -> String {
        let now = chrono::Utc::now();
        format!("backup-{}", now.format("%Y%m%d-%H%M%S"))
    }

    /// Configure rclone remote (interactive)
    pub fn configure_remote(&self) -> Result<()> {
        let mut cmd = Command::new("rclone");
        cmd.arg("config")
            .arg("create")
            .arg(&self.config.remote_name)
            .arg(self.config.provider.rclone_type());

        let status = cmd.status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to configure rclone remote"))
        }
    }

    /// Check if remote is configured
    pub fn is_remote_configured(&self) -> bool {
        let result = Command::new("rclone")
            .arg("listremotes")
            .output();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().any(|line| {
                    line.trim().trim_end_matches(':') == self.config.remote_name
                })
            }
            Err(_) => false,
        }
    }
}

impl BackupBackend for CloudBackend {
    fn name(&self) -> &str {
        self.config.provider.display_name()
    }

    fn is_available(&self) -> bool {
        // Check if rclone is installed
        let rclone_available = Command::new("rclone")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        rclone_available && self.is_remote_configured()
    }

    fn test_connection(&self) -> Result<()> {
        if !self.is_remote_configured() {
            return Err(anyhow!(
                "Remote '{}' is not configured. Run 'rclone config' to set it up.",
                self.config.remote_name
            ));
        }

        // Test by listing the root
        self.exec(&["lsd", &self.base_remote_path()])?;

        // Create backup directory if it doesn't exist
        self.exec(&["mkdir", &self.base_remote_path()])?;

        Ok(())
    }

    fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let mut backups = Vec::new();

        // List directories in backup path
        let output = self.exec(&["lsjson", &self.base_remote_path()])?;

        let entries: Vec<serde_json::Value> = serde_json::from_str(&output)?;

        for entry in entries {
            if entry.get("IsDir").and_then(|v| v.as_bool()).unwrap_or(false) {
                let name = entry.get("Name").and_then(|v| v.as_str()).unwrap_or("");

                // Try to read metadata
                let metadata_path = format!("{}/{}/metadata.json", self.base_remote_path(), name);
                if let Ok(metadata_content) = self.exec(&["cat", &metadata_path]) {
                    if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&metadata_content) {
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
        let remote_backup_path = self.remote_path(&backup_id);
        let remote_data_path = format!("{}/data", remote_backup_path);

        // Create remote directory
        self.exec(&["mkdir", &remote_data_path])?;

        // Report scanning phase
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Preparing upload...".to_string(),
                files_processed: 0,
                files_total: 0,
                bytes_processed: 0,
                bytes_total: 0,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
                phase: BackupPhase::Scanning,
            });
        }

        let mut total_files = 0u64;
        let mut total_bytes = 0u64;

        // Upload each source
        for source in sources {
            let source_name = source.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());

            let dest = format!("{}/{}", remote_data_path, source_name);

            // Report backing phase
            if let Some(ref cb) = progress {
                cb(BackupProgress {
                    current_file: format!("Uploading {}...", source.display()),
                    files_processed: 0,
                    files_total: 0,
                    bytes_processed: 0,
                    bytes_total: 0,
                    speed_bytes_per_sec: 0,
                    eta_seconds: None,
                    phase: BackupPhase::Backing,
                });
            }

            // Sync to cloud
            self.exec(&["sync", &source.display().to_string(), &dest])?;

            // Count files
            let count_output = self.exec(&["size", "--json", &dest])?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&count_output) {
                total_files += json.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                total_bytes += json.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
            }
        }

        // Create metadata
        let metadata = BackupMetadata {
            id: backup_id.clone(),
            name: name.to_string(),
            timestamp: chrono::Utc::now(),
            backup_type,
            size_bytes: total_bytes,
            file_count: total_files,
            compression,
            encrypted: encrypt,
            verified: false,
            tags: vec![],
        };

        // Save metadata to cloud
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        let local_metadata_path = self.local_cache.join(format!("{}_metadata.json", backup_id));
        std::fs::write(&local_metadata_path, &metadata_json)?;

        let remote_metadata_path = format!("{}/metadata.json", remote_backup_path);
        self.exec(&["copyto", &local_metadata_path.display().to_string(), &remote_metadata_path])?;

        std::fs::remove_file(&local_metadata_path)?;

        // Report complete
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Complete".to_string(),
                files_processed: total_files,
                files_total: total_files,
                bytes_processed: total_bytes,
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
        let remote_data_path = format!("{}/data", self.remote_path(backup_id));

        // Report restore phase
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Downloading...".to_string(),
                files_processed: 0,
                files_total: 0,
                bytes_processed: 0,
                bytes_total: 0,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
                phase: BackupPhase::Backing,
            });
        }

        let mut args = vec!["sync", &remote_data_path, &destination.display().to_string()];

        // Add file filters if specified
        let include_args: Vec<String>;
        if let Some(filter_files) = files {
            include_args = filter_files
                .iter()
                .flat_map(|f| vec!["--include".to_string(), format!("{}**", f.display())])
                .collect();

            for arg in &include_args {
                args.push(arg);
            }
            args.push("--exclude");
            args.push("*");
        }

        self.exec(&args)?;

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
        let remote_path = self.remote_path(backup_id);
        self.exec(&["purge", &remote_path])?;
        Ok(())
    }

    fn verify_backup(&self, backup_id: &str) -> Result<bool> {
        let remote_path = self.remote_path(backup_id);

        // Check metadata exists
        let metadata_path = format!("{}/metadata.json", remote_path);
        let result = self.exec(&["lsf", &metadata_path]);

        if result.is_err() {
            return Ok(false);
        }

        // Check data directory exists
        let data_path = format!("{}/data", remote_path);
        let result = self.exec(&["lsf", &data_path]);

        Ok(result.is_ok())
    }

    fn get_backup(&self, backup_id: &str) -> Result<Option<BackupMetadata>> {
        let remote_path = self.remote_path(backup_id);
        let metadata_path = format!("{}/metadata.json", remote_path);

        match self.exec(&["cat", &metadata_path]) {
            Ok(content) => {
                let metadata = serde_json::from_str(&content)?;
                Ok(Some(metadata))
            }
            Err(_) => Ok(None),
        }
    }

    fn list_files(&self, backup_id: &str, path: Option<&Path>) -> Result<Vec<FileEntry>> {
        let base_path = format!("{}/data", self.remote_path(backup_id));
        let search_path = match path {
            Some(p) => format!("{}/{}", base_path, p.display()),
            None => base_path,
        };

        let output = self.exec(&["lsjson", &search_path])?;
        let entries: Vec<serde_json::Value> = serde_json::from_str(&output)?;

        let mut files = Vec::new();

        for entry in entries {
            let name = entry.get("Name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let is_dir = entry.get("IsDir").and_then(|v| v.as_bool()).unwrap_or(false);
            let size = entry.get("Size").and_then(|v| v.as_u64()).unwrap_or(0);

            let mtime_str = entry.get("ModTime").and_then(|v| v.as_str()).unwrap_or("");
            let modified = chrono::DateTime::parse_from_rfc3339(mtime_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            files.push(FileEntry {
                path: name,
                is_dir,
                size,
                modified,
                permissions: 0o644,
            });
        }

        // Sort directories first, then by name
        files.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.path.cmp(&b.path),
            }
        });

        Ok(files)
    }

    fn get_storage_usage(&self) -> Result<StorageUsage> {
        let output = self.exec(&["about", "--json", &format!("{}:", self.config.remote_name)])?;

        let json: serde_json::Value = serde_json::from_str(&output)?;

        let total = json.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
        let used = json.get("used").and_then(|v| v.as_u64()).unwrap_or(0);
        let available = json.get("free").and_then(|v| v.as_u64()).unwrap_or(total - used);

        let backups = self.list_backups()?.len() as u64;

        Ok(StorageUsage {
            used_bytes: used,
            available_bytes: available,
            total_bytes: total,
            backup_count: backups,
        })
    }
}
