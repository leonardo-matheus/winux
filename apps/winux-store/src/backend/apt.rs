//! APT backend for package management
//!
//! Provides integration with APT (Advanced Package Tool) for Debian-based systems.

use async_channel::Sender;
use std::process::Command;
use tracing::{debug, error, info, warn};

use super::{
    AppPackage, BackendError, BackendResult, InstallStatus, PackageBackend, PackageSource,
    ProgressUpdate,
};

/// APT backend implementation
pub struct AptBackend {
    /// Whether apt is available on the system
    available: bool,
}

impl AptBackend {
    pub fn new() -> Self {
        let available = Self::check_availability();
        Self { available }
    }

    /// Check if apt is installed and available
    fn check_availability() -> bool {
        Command::new("apt-cache")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Parse apt-cache show output into AppPackage
    fn parse_package_info(&self, output: &str) -> Option<AppPackage> {
        let mut package = AppPackage::new("", "", PackageSource::Apt);
        let mut in_description = false;
        let mut description_lines = Vec::new();

        for line in output.lines() {
            if in_description {
                if line.starts_with(' ') {
                    description_lines.push(line.trim());
                } else {
                    in_description = false;
                }
            }

            if let Some((key, value)) = line.split_once(": ") {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Package" => package.id = value.to_string(),
                    "Version" => package.version = value.to_string(),
                    "Section" => {
                        package.categories = vec![value.to_string()];
                    }
                    "Installed-Size" => {
                        if let Ok(size) = value.parse::<u64>() {
                            package.installed_size = size * 1024; // Convert KB to bytes
                        }
                    }
                    "Size" => {
                        if let Ok(size) = value.parse() {
                            package.download_size = size;
                        }
                    }
                    "Homepage" => package.homepage = Some(value.to_string()),
                    "Description" | "Description-en" => {
                        package.summary = value.to_string();
                        in_description = true;
                    }
                    _ => {}
                }
            }
        }

        if !description_lines.is_empty() {
            package.description = description_lines.join("\n");
        }

        // Use package name as display name if not set
        if package.name.is_empty() {
            package.name = package.id.clone();
        }

        if package.id.is_empty() {
            None
        } else {
            Some(package)
        }
    }

    /// Search for packages using apt-cache
    fn apt_search(&self, query: &str) -> BackendResult<Vec<AppPackage>> {
        let output = Command::new("apt-cache")
            .args(["search", "--names-only", query])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute apt-cache: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<AppPackage> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() >= 2 {
                    let mut pkg = AppPackage::new(parts[0], parts[0], PackageSource::Apt);
                    pkg.summary = parts[1].to_string();
                    Some(pkg)
                } else {
                    None
                }
            })
            .take(50) // Limit results
            .collect();

        Ok(packages)
    }

    /// Get list of installed packages
    fn get_installed_packages(&self) -> BackendResult<Vec<AppPackage>> {
        let output = Command::new("dpkg-query")
            .args(["-W", "-f=${Package}\t${Version}\t${Status}\n"])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute dpkg-query: {}", e)))?;

        if !output.status.success() {
            return Err(BackendError::Other("Failed to query installed packages".into()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<AppPackage> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 && parts[2].contains("installed") {
                    let mut pkg = AppPackage::new(parts[0], parts[0], PackageSource::Apt);
                    pkg.version = parts[1].to_string();
                    pkg.status = InstallStatus::Installed;
                    Some(pkg)
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    /// Get packages with available updates
    fn get_upgradable(&self) -> BackendResult<Vec<AppPackage>> {
        let output = Command::new("apt")
            .args(["list", "--upgradable"])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute apt: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<AppPackage> = stdout
            .lines()
            .skip(1) // Skip "Listing..." header
            .filter_map(|line| {
                // Format: package/source version arch [upgradable from: old_version]
                let parts: Vec<&str> = line.split('/').collect();
                if !parts.is_empty() {
                    let name = parts[0];
                    let mut pkg = AppPackage::new(name, name, PackageSource::Apt);
                    pkg.status = InstallStatus::UpdateAvailable;
                    Some(pkg)
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }
}

impl Default for AptBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageBackend for AptBackend {
    fn name(&self) -> &str {
        "APT"
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn search(&self, query: &str) -> BackendResult<Vec<AppPackage>> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        debug!("Searching APT for: {}", query);
        self.apt_search(query)
    }

    fn get_package(&self, id: &str) -> BackendResult<AppPackage> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        let output = Command::new("apt-cache")
            .args(["show", id])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute apt-cache: {}", e)))?;

        if !output.status.success() {
            return Err(BackendError::NotFound(id.to_string()));
        }

        let info = String::from_utf8_lossy(&output.stdout);
        self.parse_package_info(&info)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    fn list_installed(&self) -> BackendResult<Vec<AppPackage>> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        self.get_installed_packages()
    }

    fn list_updates(&self) -> BackendResult<Vec<AppPackage>> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        self.get_upgradable()
    }

    fn install(&self, id: &str, progress: Sender<ProgressUpdate>) -> BackendResult<()> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        info!("Installing APT package: {}", id);

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Install,
            package_id: id.to_string(),
            progress: 0,
            message: "Starting installation...".to_string(),
        });

        // Note: In production, this would use pkexec or similar for privilege escalation
        let output = Command::new("pkexec")
            .args(["apt-get", "install", "-y", id])
            .output()
            .map_err(|e| BackendError::InstallFailed(format!("Failed to execute apt: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::InstallFailed(stderr.to_string()));
        }

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Install,
            package_id: id.to_string(),
            progress: 100,
            message: "Installation complete".to_string(),
        });

        info!("Successfully installed: {}", id);
        Ok(())
    }

    fn uninstall(&self, id: &str, progress: Sender<ProgressUpdate>) -> BackendResult<()> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        info!("Uninstalling APT package: {}", id);

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Uninstall,
            package_id: id.to_string(),
            progress: 0,
            message: "Starting uninstallation...".to_string(),
        });

        let output = Command::new("pkexec")
            .args(["apt-get", "remove", "-y", id])
            .output()
            .map_err(|e| BackendError::UninstallFailed(format!("Failed to execute apt: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::UninstallFailed(stderr.to_string()));
        }

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Uninstall,
            package_id: id.to_string(),
            progress: 100,
            message: "Uninstallation complete".to_string(),
        });

        info!("Successfully uninstalled: {}", id);
        Ok(())
    }

    fn update(&self, id: &str, progress: Sender<ProgressUpdate>) -> BackendResult<()> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        info!("Updating APT package: {}", id);

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Update,
            package_id: id.to_string(),
            progress: 0,
            message: "Starting update...".to_string(),
        });

        let output = Command::new("pkexec")
            .args(["apt-get", "install", "--only-upgrade", "-y", id])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute apt: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::Other(format!("Update failed: {}", stderr)));
        }

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Update,
            package_id: id.to_string(),
            progress: 100,
            message: "Update complete".to_string(),
        });

        info!("Successfully updated: {}", id);
        Ok(())
    }

    fn refresh(&self) -> BackendResult<()> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("APT is not available".into()));
        }

        info!("Refreshing APT package cache");

        let output = Command::new("pkexec")
            .args(["apt-get", "update"])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute apt: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("APT refresh warning: {}", stderr);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_backend_creation() {
        let backend = AptBackend::new();
        assert_eq!(backend.name(), "APT");
    }
}
