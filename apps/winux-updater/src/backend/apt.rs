//! APT backend for system package updates

use std::process::Command;
use anyhow::{anyhow, Result};
use regex::Regex;
use tracing::{info, warn, error};

use super::{PackageUpdate, UpdateSource, UpdatePriority};

/// APT backend for Debian/Ubuntu package management
pub struct AptBackend {
    cache_dir: String,
}

impl AptBackend {
    pub fn new() -> Self {
        Self {
            cache_dir: "/var/cache/apt/archives".to_string(),
        }
    }

    /// Refresh the package lists (apt update)
    pub async fn refresh(&self) -> Result<()> {
        info!("Refreshing APT package lists");

        let output = Command::new("pkexec")
            .args(["apt-get", "update"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to refresh APT: {}", stderr);
            return Err(anyhow!("Failed to refresh package lists: {}", stderr));
        }

        info!("APT package lists refreshed successfully");
        Ok(())
    }

    /// Check for available updates
    pub async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        info!("Checking APT updates");

        let output = Command::new("apt-get")
            .args(["--simulate", "upgrade"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("APT simulate upgrade returned error: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates = self.parse_upgrade_output(&stdout);

        info!("Found {} APT updates", updates.len());
        Ok(updates)
    }

    /// Parse the output of apt-get upgrade --simulate
    fn parse_upgrade_output(&self, output: &str) -> Vec<PackageUpdate> {
        let mut updates = Vec::new();
        let upgrade_regex = Regex::new(r"Inst (\S+) \[([^\]]+)\] \(([^\s]+)").unwrap();

        for line in output.lines() {
            if let Some(caps) = upgrade_regex.captures(line) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let current = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let new = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                let priority = if name.contains("security") || line.contains("security") {
                    UpdatePriority::Security
                } else if name.starts_with("linux-") || name == "systemd" {
                    UpdatePriority::Important
                } else {
                    UpdatePriority::Normal
                };

                let requires_restart = name.starts_with("linux-image")
                    || name.starts_with("linux-headers")
                    || name == "systemd"
                    || name == "dbus"
                    || name == "libc6";

                updates.push(PackageUpdate {
                    id: format!("apt:{}", name),
                    name: name.to_string(),
                    current_version: current.to_string(),
                    new_version: new.to_string(),
                    source: UpdateSource::Apt,
                    download_size: 0, // Would need additional query
                    installed_size: 0,
                    description: self.get_package_description(name).unwrap_or_default(),
                    changelog: None,
                    priority,
                    requires_restart,
                });
            }
        }

        updates
    }

    /// Get package description
    fn get_package_description(&self, package: &str) -> Option<String> {
        let output = Command::new("apt-cache")
            .args(["show", package])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Description:") || line.starts_with("Description-en:") {
                return Some(line.split(':').nth(1)?.trim().to_string());
            }
        }
        None
    }

    /// Get changelog for a package
    pub async fn get_changelog(&self, package: &str) -> Result<String> {
        let output = Command::new("apt-get")
            .args(["changelog", package])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("Failed to get changelog"))
        }
    }

    /// Install a single package
    pub async fn install_package(&self, package: &str) -> Result<()> {
        info!("Installing package: {}", package);

        let output = Command::new("pkexec")
            .args(["apt-get", "install", "-y", package])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to install {}: {}", package, stderr);
            return Err(anyhow!("Failed to install package: {}", stderr));
        }

        info!("Package {} installed successfully", package);
        Ok(())
    }

    /// Upgrade all packages
    pub async fn upgrade_all(&self) -> Result<()> {
        info!("Upgrading all APT packages");

        let output = Command::new("pkexec")
            .args(["apt-get", "upgrade", "-y"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to upgrade: {}", stderr);
            return Err(anyhow!("Failed to upgrade packages: {}", stderr));
        }

        info!("All packages upgraded successfully");
        Ok(())
    }

    /// Full upgrade (dist-upgrade)
    pub async fn full_upgrade(&self) -> Result<()> {
        info!("Performing full upgrade");

        let output = Command::new("pkexec")
            .args(["apt-get", "dist-upgrade", "-y"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to dist-upgrade: {}", stderr);
            return Err(anyhow!("Failed to perform full upgrade: {}", stderr));
        }

        info!("Full upgrade completed successfully");
        Ok(())
    }

    /// Download packages without installing
    pub async fn download_only(&self, packages: &[String]) -> Result<()> {
        info!("Downloading packages: {:?}", packages);

        let mut args = vec!["apt-get", "install", "-y", "--download-only"];
        for pkg in packages {
            args.push(pkg);
        }

        let output = Command::new("pkexec")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to download packages: {}", stderr));
        }

        Ok(())
    }

    /// Clean package cache
    pub async fn clean_cache(&self) -> Result<u64> {
        info!("Cleaning APT cache");

        // Get cache size before cleaning
        let size_before = self.get_cache_size()?;

        let output = Command::new("pkexec")
            .args(["apt-get", "clean"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to clean cache: {}", stderr));
        }

        info!("APT cache cleaned, freed {} bytes", size_before);
        Ok(size_before)
    }

    /// Get size of package cache
    fn get_cache_size(&self) -> Result<u64> {
        let output = Command::new("du")
            .args(["-sb", &self.cache_dir])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let size: u64 = stdout
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(size)
    }

    /// Remove orphaned packages
    pub async fn autoremove(&self) -> Result<Vec<String>> {
        info!("Removing orphaned packages");

        let output = Command::new("pkexec")
            .args(["apt-get", "autoremove", "-y"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to autoremove: {}", stderr));
        }

        // Parse removed packages from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let removed: Vec<String> = stdout
            .lines()
            .filter(|l| l.contains("Removing"))
            .filter_map(|l| l.split_whitespace().nth(1))
            .map(|s| s.to_string())
            .collect();

        info!("Removed {} orphaned packages", removed.len());
        Ok(removed)
    }

    /// Fix broken packages
    pub async fn fix_broken(&self) -> Result<()> {
        info!("Fixing broken packages");

        let output = Command::new("pkexec")
            .args(["dpkg", "--configure", "-a"])
            .output()?;

        if !output.status.success() {
            warn!("dpkg configure returned non-zero");
        }

        let output = Command::new("pkexec")
            .args(["apt-get", "install", "-f", "-y"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to fix broken packages: {}", stderr));
        }

        info!("Broken packages fixed");
        Ok(())
    }

    /// Check if a package is installed
    pub fn is_installed(&self, package: &str) -> bool {
        Command::new("dpkg")
            .args(["-s", package])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get installed version of a package
    pub fn installed_version(&self, package: &str) -> Option<String> {
        let output = Command::new("dpkg-query")
            .args(["-W", "-f=${Version}", package])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// List security updates
    pub async fn security_updates(&self) -> Result<Vec<PackageUpdate>> {
        let updates = self.check_updates().await?;
        Ok(updates
            .into_iter()
            .filter(|u| u.priority == UpdatePriority::Security)
            .collect())
    }
}

impl Default for AptBackend {
    fn default() -> Self {
        Self::new()
    }
}
