//! Snap backend for Snap package updates

use std::process::Command;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use tracing::{info, warn, error};

use super::{PackageUpdate, UpdateSource, UpdatePriority};

/// Response from snap refresh --list
#[derive(Debug, Deserialize)]
struct SnapRefreshItem {
    name: String,
    version: String,
    #[serde(rename = "tracking-channel")]
    channel: Option<String>,
}

/// Snap backend for Snap package management
pub struct SnapBackend;

impl SnapBackend {
    pub fn new() -> Self {
        Self
    }

    /// Check for available Snap updates
    pub async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        info!("Checking Snap updates");

        let output = Command::new("snap")
            .args(["refresh", "--list"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // "All snaps up to date" returns non-zero, which is fine
            if stderr.contains("All snaps up to date") {
                return Ok(Vec::new());
            }
            warn!("Snap refresh --list returned error: {}", stderr);
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates = self.parse_refresh_output(&stdout);

        info!("Found {} Snap updates", updates.len());
        Ok(updates)
    }

    /// Parse snap refresh --list output
    fn parse_refresh_output(&self, output: &str) -> Vec<PackageUpdate> {
        let mut updates = Vec::new();

        // Skip header line
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0];
                let new_version = parts[1];

                // Get current installed version
                let current_version = self.get_installed_version(name).unwrap_or_default();

                // Get snap info for size and description
                let (size, description) = self.get_snap_info(name).unwrap_or((0, String::new()));

                updates.push(PackageUpdate {
                    id: format!("snap:{}", name),
                    name: name.to_string(),
                    current_version,
                    new_version: new_version.to_string(),
                    source: UpdateSource::Snap,
                    download_size: size,
                    installed_size: size,
                    description,
                    changelog: None,
                    priority: UpdatePriority::Normal,
                    requires_restart: false,
                });
            }
        }

        updates
    }

    /// Get installed version of a snap
    fn get_installed_version(&self, name: &str) -> Option<String> {
        let output = Command::new("snap")
            .args(["info", name])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("installed:") {
                // Format: "installed:   1.2.3    (123) 45MB"
                return line.split_whitespace().nth(1).map(|s| s.to_string());
            }
        }
        None
    }

    /// Get snap info (size, description)
    fn get_snap_info(&self, name: &str) -> Option<(u64, String)> {
        let output = Command::new("snap")
            .args(["info", name])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut size = 0u64;
        let mut description = String::new();

        for line in stdout.lines() {
            if line.starts_with("installed:") {
                // Parse size from installed line
                if let Some(size_str) = line.split_whitespace().last() {
                    size = self.parse_size(size_str);
                }
            } else if line.starts_with("summary:") {
                description = line.strip_prefix("summary:").unwrap_or("").trim().to_string();
            }
        }

        Some((size, description))
    }

    /// Parse size string to bytes
    fn parse_size(&self, size_str: &str) -> u64 {
        let size_str = size_str.to_uppercase();
        let (num, unit) = if size_str.ends_with("GB") {
            (size_str.trim_end_matches("GB"), 1024u64 * 1024 * 1024)
        } else if size_str.ends_with("MB") {
            (size_str.trim_end_matches("MB"), 1024u64 * 1024)
        } else if size_str.ends_with("KB") {
            (size_str.trim_end_matches("KB"), 1024u64)
        } else {
            (size_str.as_str(), 1u64)
        };

        num.parse::<f64>().unwrap_or(0.0) as u64 * unit
    }

    /// Refresh (update) a single snap
    pub async fn refresh_package(&self, name: &str) -> Result<()> {
        info!("Refreshing snap: {}", name);

        // Remove "snap:" prefix if present
        let name = name.strip_prefix("snap:").unwrap_or(name);

        let output = Command::new("snap")
            .args(["refresh", name])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to refresh snap {}: {}", name, stderr);
            return Err(anyhow!("Failed to refresh snap: {}", stderr));
        }

        info!("Snap {} refreshed successfully", name);
        Ok(())
    }

    /// Refresh all snaps
    pub async fn refresh_all(&self) -> Result<()> {
        info!("Refreshing all snaps");

        let output = Command::new("snap")
            .args(["refresh"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to refresh snaps: {}", stderr);
            return Err(anyhow!("Failed to refresh snaps: {}", stderr));
        }

        info!("All snaps refreshed successfully");
        Ok(())
    }

    /// List installed snaps
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new("snap")
            .args(["list"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| line.split_whitespace().next())
            .map(|s| s.to_string())
            .collect())
    }

    /// Revert snap to previous version
    pub async fn revert(&self, name: &str) -> Result<()> {
        info!("Reverting snap: {}", name);

        let output = Command::new("snap")
            .args(["revert", name])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to revert snap: {}", stderr));
        }

        Ok(())
    }

    /// Get snap changes/transactions
    pub fn get_changes(&self) -> Result<Vec<String>> {
        let output = Command::new("snap")
            .args(["changes"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Check if snapd is running
    pub fn is_snapd_running(&self) -> bool {
        Command::new("systemctl")
            .args(["is-active", "snapd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get snap store connection status
    pub fn is_connected(&self) -> bool {
        Command::new("snap")
            .args(["find", "--narrow", "hello-world"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for SnapBackend {
    fn default() -> Self {
        Self::new()
    }
}
