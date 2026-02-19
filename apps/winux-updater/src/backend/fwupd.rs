//! Fwupd backend for firmware updates

use std::process::Command;
use anyhow::{anyhow, Result};
use tracing::{info, warn, error};

use super::{PackageUpdate, UpdateSource, UpdatePriority};

/// Fwupd backend for firmware updates
pub struct FwupdBackend;

impl FwupdBackend {
    pub fn new() -> Self {
        Self
    }

    /// Check if fwupd is available
    pub fn is_available(&self) -> bool {
        Command::new("fwupdmgr")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Refresh firmware metadata
    pub async fn refresh(&self) -> Result<()> {
        info!("Refreshing fwupd metadata");

        let output = Command::new("fwupdmgr")
            .args(["refresh", "--force"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("fwupdmgr refresh warning: {}", stderr);
        }

        Ok(())
    }

    /// Check for available firmware updates
    pub async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        info!("Checking firmware updates");

        // First refresh metadata
        let _ = self.refresh().await;

        let output = Command::new("fwupdmgr")
            .args(["get-updates", "--json"])
            .output()?;

        // fwupdmgr returns non-zero if no updates, which is fine
        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.trim().is_empty() || stdout.contains("No upgrades") {
            return Ok(Vec::new());
        }

        let updates = self.parse_updates_output(&stdout);

        info!("Found {} firmware updates", updates.len());
        Ok(updates)
    }

    /// Parse fwupdmgr JSON output
    fn parse_updates_output(&self, output: &str) -> Vec<PackageUpdate> {
        let mut updates = Vec::new();

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
            if let Some(devices) = json.get("Devices").and_then(|d| d.as_array()) {
                for device in devices {
                    if let Some(releases) = device.get("Releases").and_then(|r| r.as_array()) {
                        for release in releases {
                            let device_name = device
                                .get("Name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("Unknown Device");

                            let device_id = device
                                .get("DeviceId")
                                .and_then(|n| n.as_str())
                                .unwrap_or("");

                            let current_version = device
                                .get("Version")
                                .and_then(|n| n.as_str())
                                .unwrap_or("");

                            let new_version = release
                                .get("Version")
                                .and_then(|n| n.as_str())
                                .unwrap_or("");

                            let description = release
                                .get("Summary")
                                .and_then(|n| n.as_str())
                                .unwrap_or("");

                            let size = release
                                .get("Size")
                                .and_then(|n| n.as_u64())
                                .unwrap_or(0);

                            let urgency = release
                                .get("Urgency")
                                .and_then(|n| n.as_str())
                                .unwrap_or("medium");

                            let priority = match urgency {
                                "critical" | "high" => UpdatePriority::Security,
                                "medium" => UpdatePriority::Important,
                                _ => UpdatePriority::Normal,
                            };

                            updates.push(PackageUpdate {
                                id: format!("fwupd:{}", device_id),
                                name: device_name.to_string(),
                                current_version: current_version.to_string(),
                                new_version: new_version.to_string(),
                                source: UpdateSource::Fwupd,
                                download_size: size,
                                installed_size: size,
                                description: description.to_string(),
                                changelog: release.get("Description")
                                    .and_then(|d| d.as_str())
                                    .map(|s| s.to_string()),
                                priority,
                                requires_restart: true,
                            });
                        }
                    }
                }
            }
        } else {
            // Fallback: parse text output
            updates = self.parse_text_output(output);
        }

        updates
    }

    /// Parse text output (fallback)
    fn parse_text_output(&self, output: &str) -> Vec<PackageUpdate> {
        let mut updates = Vec::new();
        let mut current_device = String::new();
        let mut current_version = String::new();

        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("Device:") {
                current_device = line.strip_prefix("Device:").unwrap_or("").trim().to_string();
            } else if line.starts_with("Current version:") {
                current_version = line.strip_prefix("Current version:").unwrap_or("").trim().to_string();
            } else if line.starts_with("Update:") || line.starts_with("Version:") {
                let new_version = line.split(':').nth(1).unwrap_or("").trim().to_string();

                if !current_device.is_empty() && !new_version.is_empty() {
                    updates.push(PackageUpdate {
                        id: format!("fwupd:{}", current_device.replace(' ', "_")),
                        name: current_device.clone(),
                        current_version: current_version.clone(),
                        new_version,
                        source: UpdateSource::Fwupd,
                        download_size: 0,
                        installed_size: 0,
                        description: "Firmware update".to_string(),
                        changelog: None,
                        priority: UpdatePriority::Important,
                        requires_restart: true,
                    });
                }
            }
        }

        updates
    }

    /// Install a firmware update
    pub async fn install_update(&self, device_id: &str) -> Result<()> {
        info!("Installing firmware update: {}", device_id);

        // Remove "fwupd:" prefix if present
        let device_id = device_id.strip_prefix("fwupd:").unwrap_or(device_id);

        let output = Command::new("pkexec")
            .args(["fwupdmgr", "update", device_id, "--no-reboot-check", "-y"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to install firmware {}: {}", device_id, stderr);
            return Err(anyhow!("Failed to install firmware: {}", stderr));
        }

        info!("Firmware {} installed successfully", device_id);
        Ok(())
    }

    /// Install all available firmware updates
    pub async fn install_all(&self) -> Result<()> {
        info!("Installing all firmware updates");

        let output = Command::new("pkexec")
            .args(["fwupdmgr", "update", "--no-reboot-check", "-y"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // "No updatable devices" is not an error
            if !stderr.contains("No updatable devices") {
                error!("Failed to install firmware updates: {}", stderr);
                return Err(anyhow!("Failed to install firmware updates: {}", stderr));
            }
        }

        info!("All firmware updates installed successfully");
        Ok(())
    }

    /// List devices with firmware
    pub fn list_devices(&self) -> Result<Vec<String>> {
        let output = Command::new("fwupdmgr")
            .args(["get-devices"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<String> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(devices)
    }

    /// Get device history
    pub fn get_history(&self) -> Result<Vec<String>> {
        let output = Command::new("fwupdmgr")
            .args(["get-history"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Downgrade firmware (if supported)
    pub async fn downgrade(&self, device_id: &str) -> Result<()> {
        info!("Downgrading firmware: {}", device_id);

        let output = Command::new("pkexec")
            .args(["fwupdmgr", "downgrade", device_id])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to downgrade firmware: {}", stderr));
        }

        Ok(())
    }

    /// Get security status
    pub fn security_status(&self) -> Result<String> {
        let output = Command::new("fwupdmgr")
            .args(["security", "--json"])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Verify firmware signatures
    pub async fn verify(&self, device_id: &str) -> Result<bool> {
        let output = Command::new("fwupdmgr")
            .args(["verify", device_id])
            .output()?;

        Ok(output.status.success())
    }
}

impl Default for FwupdBackend {
    fn default() -> Self {
        Self::new()
    }
}
