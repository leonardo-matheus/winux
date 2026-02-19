//! Rsync backup backend - Remote storage via SSH

use super::*;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Rsync backend configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RsyncConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub remote_path: String,
    pub ssh_key: Option<String>,
    pub bandwidth_limit: Option<u64>, // KB/s
}

impl Default for RsyncConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            user: String::new(),
            remote_path: String::new(),
            ssh_key: None,
            bandwidth_limit: None,
        }
    }
}

/// Rsync backup backend - stores backups on remote server via SSH
pub struct RsyncBackend {
    config: RsyncConfig,
    local_cache: PathBuf,
}

impl RsyncBackend {
    /// Create a new rsync backend
    pub fn new(config: RsyncConfig, local_cache: impl Into<PathBuf>) -> Self {
        Self {
            config,
            local_cache: local_cache.into(),
        }
    }

    /// Get the remote path for a specific backup
    fn remote_backup_path(&self, backup_id: &str) -> String {
        format!("{}/{}", self.config.remote_path, backup_id)
    }

    /// Build SSH connection string
    fn ssh_connection(&self) -> String {
        format!("{}@{}", self.config.user, self.config.host)
    }

    /// Build rsync command with common options
    fn rsync_base_cmd(&self) -> Command {
        let mut cmd = Command::new("rsync");

        cmd.arg("-avz")
            .arg("--progress")
            .arg("--delete");

        // SSH options
        let mut ssh_cmd = format!("ssh -p {}", self.config.port);
        if let Some(ref key) = self.config.ssh_key {
            ssh_cmd.push_str(&format!(" -i {}", key));
        }
        cmd.arg("-e").arg(&ssh_cmd);

        // Bandwidth limit
        if let Some(limit) = self.config.bandwidth_limit {
            cmd.arg(format!("--bwlimit={}", limit));
        }

        cmd
    }

    /// Execute SSH command on remote server
    fn ssh_exec(&self, command: &str) -> Result<String> {
        let mut cmd = Command::new("ssh");

        cmd.arg("-p").arg(self.config.port.to_string());

        if let Some(ref key) = self.config.ssh_key {
            cmd.arg("-i").arg(key);
        }

        cmd.arg(&self.ssh_connection())
            .arg(command);

        let output = cmd.output().context("Failed to execute SSH command")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!(
                "SSH command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Generate a unique backup ID
    fn generate_id(&self) -> String {
        let now = chrono::Utc::now();
        format!("backup-{}", now.format("%Y%m%d-%H%M%S"))
    }

    /// Parse rsync progress output
    fn parse_progress(line: &str) -> Option<(u64, u64, u64)> {
        // Parse lines like: "1,234,567 100%  12.34MB/s    0:01:23"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let bytes = parts[0].replace(',', "").parse::<u64>().ok()?;
            let percent = parts[1].trim_end_matches('%').parse::<u64>().ok()?;
            let speed_str = parts[2];
            let speed = if speed_str.ends_with("MB/s") {
                speed_str.trim_end_matches("MB/s").parse::<f64>().ok()? as u64 * 1024 * 1024
            } else if speed_str.ends_with("kB/s") {
                speed_str.trim_end_matches("kB/s").parse::<f64>().ok()? as u64 * 1024
            } else {
                0
            };
            return Some((bytes, percent, speed));
        }
        None
    }
}

impl BackupBackend for RsyncBackend {
    fn name(&self) -> &str {
        "Rsync"
    }

    fn is_available(&self) -> bool {
        // Check if rsync is installed
        Command::new("rsync")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn test_connection(&self) -> Result<()> {
        // Test SSH connection
        self.ssh_exec("echo 'Connection OK'")?;

        // Test remote path exists or create it
        self.ssh_exec(&format!("mkdir -p {}", self.config.remote_path))?;

        Ok(())
    }

    fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let output = self.ssh_exec(&format!(
            "find {} -maxdepth 2 -name 'metadata.json' -exec cat {{}} \\;",
            self.config.remote_path
        ))?;

        let mut backups = Vec::new();

        for line in output.lines() {
            if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(line) {
                backups.push(metadata);
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
        let remote_path = self.remote_backup_path(&backup_id);
        let remote_data_path = format!("{}/data", remote_path);

        // Create remote directory
        self.ssh_exec(&format!("mkdir -p {}", remote_data_path))?;

        // Report scanning phase
        if let Some(ref cb) = progress {
            cb(BackupProgress {
                current_file: "Preparing transfer...".to_string(),
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

        // Sync each source
        for source in sources {
            let source_name = source.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());

            let dest = format!(
                "{}:{}/{}/",
                self.ssh_connection(),
                remote_data_path,
                source_name
            );

            let mut cmd = self.rsync_base_cmd();
            cmd.arg(format!("{}/", source.display()))
                .arg(&dest);

            // Report backing phase
            if let Some(ref cb) = progress {
                cb(BackupProgress {
                    current_file: format!("Syncing {}...", source.display()),
                    files_processed: 0,
                    files_total: 0,
                    bytes_processed: 0,
                    bytes_total: 0,
                    speed_bytes_per_sec: 0,
                    eta_seconds: None,
                    phase: BackupPhase::Backing,
                });
            }

            let output = cmd.output().context("Failed to execute rsync")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Rsync failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Count files from rsync output
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if !line.starts_with(' ') && !line.is_empty() {
                    total_files += 1;
                }
            }
        }

        // Get total size from remote
        let size_output = self.ssh_exec(&format!("du -sb {}", remote_data_path))?;
        if let Some(size_str) = size_output.split_whitespace().next() {
            total_bytes = size_str.parse().unwrap_or(0);
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

        // Save metadata to remote
        let metadata_json = serde_json::to_string(&metadata)?;
        self.ssh_exec(&format!(
            "echo '{}' > {}/metadata.json",
            metadata_json, remote_path
        ))?;

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
        let remote_data_path = format!("{}/data/", self.remote_backup_path(backup_id));
        let source = format!("{}:{}", self.ssh_connection(), remote_data_path);

        let mut cmd = self.rsync_base_cmd();

        // Add file filters if specified
        if let Some(filter_files) = files {
            for file in filter_files {
                cmd.arg("--include").arg(format!("{}", file.display()));
                cmd.arg("--include").arg(format!("{}/**", file.display()));
            }
            cmd.arg("--exclude").arg("*");
        }

        cmd.arg(&source)
            .arg(format!("{}/", destination.display()));

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

        let output = cmd.output().context("Failed to execute rsync")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Rsync restore failed: {}",
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
        let remote_path = self.remote_backup_path(backup_id);
        self.ssh_exec(&format!("rm -rf {}", remote_path))?;
        Ok(())
    }

    fn verify_backup(&self, backup_id: &str) -> Result<bool> {
        let remote_path = self.remote_backup_path(backup_id);

        // Check metadata exists
        let result = self.ssh_exec(&format!("test -f {}/metadata.json && echo OK", remote_path));
        if result.is_err() || !result?.contains("OK") {
            return Ok(false);
        }

        // Check data directory exists
        let result = self.ssh_exec(&format!("test -d {}/data && echo OK", remote_path));
        if result.is_err() || !result?.contains("OK") {
            return Ok(false);
        }

        Ok(true)
    }

    fn get_backup(&self, backup_id: &str) -> Result<Option<BackupMetadata>> {
        let remote_path = self.remote_backup_path(backup_id);
        let result = self.ssh_exec(&format!("cat {}/metadata.json 2>/dev/null", remote_path));

        match result {
            Ok(content) => {
                let metadata = serde_json::from_str(&content)?;
                Ok(Some(metadata))
            }
            Err(_) => Ok(None),
        }
    }

    fn list_files(&self, backup_id: &str, path: Option<&Path>) -> Result<Vec<FileEntry>> {
        let remote_data_path = format!("{}/data", self.remote_backup_path(backup_id));
        let search_path = match path {
            Some(p) => format!("{}/{}", remote_data_path, p.display()),
            None => remote_data_path,
        };

        let output = self.ssh_exec(&format!(
            "ls -la --time-style=+%s {} 2>/dev/null",
            search_path
        ))?;

        let mut entries = Vec::new();

        for line in output.lines().skip(1) {
            // Skip "total" line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let is_dir = parts[0].starts_with('d');
                let size = parts[4].parse().unwrap_or(0);
                let timestamp = parts[5].parse::<i64>().unwrap_or(0);
                let name = parts[6..].join(" ");

                if name != "." && name != ".." {
                    entries.push(FileEntry {
                        path: name,
                        is_dir,
                        size,
                        modified: chrono::DateTime::from_timestamp(timestamp, 0)
                            .unwrap_or_else(|| chrono::Utc::now()),
                        permissions: 0o644,
                    });
                }
            }
        }

        Ok(entries)
    }

    fn get_storage_usage(&self) -> Result<StorageUsage> {
        let output = self.ssh_exec(&format!("du -sb {} 2>/dev/null", self.config.remote_path))?;
        let used = output.split_whitespace().next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let df_output = self.ssh_exec(&format!(
            "df -B1 {} 2>/dev/null | tail -1",
            self.config.remote_path
        ))?;
        let df_parts: Vec<&str> = df_output.split_whitespace().collect();

        let total = df_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let available = df_parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

        let backups = self.list_backups()?.len() as u64;

        Ok(StorageUsage {
            used_bytes: used,
            available_bytes: available,
            total_bytes: total,
            backup_count: backups,
        })
    }
}
