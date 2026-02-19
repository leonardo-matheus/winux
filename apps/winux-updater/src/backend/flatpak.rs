//! Flatpak backend for application updates

use std::process::Command;
use anyhow::{anyhow, Result};
use tracing::{info, warn, error};

use super::{PackageUpdate, UpdateSource, UpdatePriority};

/// Flatpak backend for sandboxed application management
pub struct FlatpakBackend {
    user_install: bool,
}

impl FlatpakBackend {
    pub fn new() -> Self {
        Self {
            user_install: true,
        }
    }

    /// Check for available Flatpak updates
    pub async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        info!("Checking Flatpak updates");

        let mut args = vec!["remote-ls", "--updates", "--columns=application,version,installed-size,description"];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Flatpak remote-ls returned error: {}", stderr);
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates = self.parse_updates_output(&stdout);

        info!("Found {} Flatpak updates", updates.len());
        Ok(updates)
    }

    /// Parse Flatpak updates output
    fn parse_updates_output(&self, output: &str) -> Vec<PackageUpdate> {
        let mut updates = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let app_id = parts[0].trim();
                let new_version = parts.get(1).map(|s| s.trim()).unwrap_or("unknown");
                let description = parts.get(3).map(|s| s.trim()).unwrap_or("");

                // Get current installed version
                let current_version = self.get_installed_version(app_id).unwrap_or_default();

                // Parse size
                let size_str = parts.get(2).map(|s| s.trim()).unwrap_or("0");
                let download_size = self.parse_size(size_str);

                updates.push(PackageUpdate {
                    id: format!("flatpak:{}", app_id),
                    name: app_id.to_string(),
                    current_version,
                    new_version: new_version.to_string(),
                    source: UpdateSource::Flatpak,
                    download_size,
                    installed_size: download_size,
                    description: description.to_string(),
                    changelog: None,
                    priority: UpdatePriority::Normal,
                    requires_restart: false,
                });
            }
        }

        updates
    }

    /// Get installed version of a Flatpak app
    fn get_installed_version(&self, app_id: &str) -> Option<String> {
        let mut args = vec!["info", app_id];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim().starts_with("Version:") {
                return Some(line.split(':').nth(1)?.trim().to_string());
            }
        }
        None
    }

    /// Parse size string (e.g., "45.2 MB") to bytes
    fn parse_size(&self, size_str: &str) -> u64 {
        let parts: Vec<&str> = size_str.split_whitespace().collect();
        if parts.len() < 2 {
            return 0;
        }

        let value: f64 = parts[0].parse().unwrap_or(0.0);
        let unit = parts[1].to_uppercase();

        let multiplier = match unit.as_str() {
            "KB" | "KIB" => 1024u64,
            "MB" | "MIB" => 1024 * 1024,
            "GB" | "GIB" => 1024 * 1024 * 1024,
            _ => 1,
        };

        (value * multiplier as f64) as u64
    }

    /// Install/update a Flatpak app
    pub async fn install_update(&self, app_id: &str) -> Result<()> {
        info!("Updating Flatpak: {}", app_id);

        // Remove "flatpak:" prefix if present
        let app_id = app_id.strip_prefix("flatpak:").unwrap_or(app_id);

        let mut args = vec!["update", "-y", app_id];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to update Flatpak {}: {}", app_id, stderr);
            return Err(anyhow!("Failed to update Flatpak: {}", stderr));
        }

        info!("Flatpak {} updated successfully", app_id);
        Ok(())
    }

    /// Update all Flatpak apps
    pub async fn update_all(&self) -> Result<()> {
        info!("Updating all Flatpak apps");

        let mut args = vec!["update", "-y"];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to update Flatpaks: {}", stderr);
            return Err(anyhow!("Failed to update Flatpaks: {}", stderr));
        }

        info!("All Flatpak apps updated successfully");
        Ok(())
    }

    /// List installed Flatpak apps
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let mut args = vec!["list", "--app", "--columns=application"];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
    }

    /// List configured remotes
    pub fn list_remotes(&self) -> Result<Vec<(String, String)>> {
        let mut args = vec!["remotes", "--columns=name,url"];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let remotes: Vec<(String, String)> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                } else {
                    None
                }
            })
            .collect();

        Ok(remotes)
    }

    /// Add a remote
    pub async fn add_remote(&self, name: &str, url: &str) -> Result<()> {
        let mut args = vec!["remote-add", "--if-not-exists", name, url];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to add remote: {}", stderr));
        }

        Ok(())
    }

    /// Remove unused runtimes
    pub async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up unused Flatpak runtimes");

        let mut args = vec!["uninstall", "--unused", "-y"];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Flatpak cleanup warning: {}", stderr);
        }

        Ok(())
    }

    /// Repair Flatpak installation
    pub async fn repair(&self) -> Result<()> {
        info!("Repairing Flatpak installation");

        let mut args = vec!["repair"];
        if self.user_install {
            args.push("--user");
        }

        let output = Command::new("flatpak")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to repair Flatpak: {}", stderr));
        }

        Ok(())
    }
}

impl Default for FlatpakBackend {
    fn default() -> Self {
        Self::new()
    }
}
