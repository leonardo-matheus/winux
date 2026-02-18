//! Flatpak backend for package management
//!
//! Provides integration with Flatpak for sandboxed application management.

use async_channel::Sender;
use std::process::Command;
use tracing::{debug, error, info, warn};

use super::{
    AppPackage, BackendError, BackendResult, InstallStatus, PackageBackend, PackageSource,
    ProgressUpdate,
};

/// Flatpak backend implementation
pub struct FlatpakBackend {
    /// Whether flatpak is available on the system
    available: bool,
    /// Configured remotes (e.g., "flathub")
    remotes: Vec<String>,
}

impl FlatpakBackend {
    pub fn new() -> Self {
        let available = Self::check_availability();
        let remotes = if available {
            Self::get_remotes()
        } else {
            Vec::new()
        };

        Self { available, remotes }
    }

    /// Check if flatpak is installed and available
    fn check_availability() -> bool {
        Command::new("flatpak")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get configured remotes
    fn get_remotes() -> Vec<String> {
        let output = Command::new("flatpak")
            .args(["remote-list", "--columns=name"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(String::from)
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Parse flatpak info output into AppPackage
    fn parse_package_info(&self, id: &str, info_output: &str) -> Option<AppPackage> {
        let mut package = AppPackage::new(id, id, PackageSource::Flatpak);

        for line in info_output.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Name" => package.name = value.to_string(),
                    "Summary" => package.summary = value.to_string(),
                    "Description" => package.description = value.to_string(),
                    "Version" => package.version = value.to_string(),
                    "License" => package.license = Some(value.to_string()),
                    "Homepage" => package.homepage = Some(value.to_string()),
                    "Installed" => {
                        if let Ok(size) = value.replace(" bytes", "").trim().parse() {
                            package.installed_size = size;
                        }
                    }
                    _ => {}
                }
            }
        }

        Some(package)
    }

    /// Search Flathub for packages
    fn search_remote(&self, remote: &str, query: &str) -> BackendResult<Vec<AppPackage>> {
        let output = Command::new("flatpak")
            .args(["search", "--columns=application,name,description,version", query])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute flatpak: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<AppPackage> = stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 4 {
                    let mut pkg = AppPackage::new(parts[0], parts[1], PackageSource::Flatpak);
                    pkg.summary = parts[2].to_string();
                    pkg.version = parts[3].to_string();
                    Some(pkg)
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    /// Get installed flatpak applications
    fn get_installed_apps(&self) -> BackendResult<Vec<AppPackage>> {
        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=application,name,version,origin"])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute flatpak: {}", e)))?;

        if !output.status.success() {
            return Err(BackendError::Other("Failed to list installed apps".into()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<AppPackage> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    let mut pkg = AppPackage::new(parts[0], parts[1], PackageSource::Flatpak);
                    pkg.version = parts[2].to_string();
                    pkg.status = InstallStatus::Installed;
                    Some(pkg)
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    /// Check for updates
    fn check_updates(&self) -> BackendResult<Vec<AppPackage>> {
        let output = Command::new("flatpak")
            .args(["remote-ls", "--updates", "--columns=application,name,version"])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute flatpak: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<AppPackage> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    let mut pkg = AppPackage::new(parts[0], parts[1], PackageSource::Flatpak);
                    pkg.version = parts[2].to_string();
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

impl Default for FlatpakBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageBackend for FlatpakBackend {
    fn name(&self) -> &str {
        "Flatpak"
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn search(&self, query: &str) -> BackendResult<Vec<AppPackage>> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        debug!("Searching Flatpak for: {}", query);

        let mut all_packages = Vec::new();
        for remote in &self.remotes {
            match self.search_remote(remote, query) {
                Ok(packages) => all_packages.extend(packages),
                Err(e) => warn!("Error searching {}: {}", remote, e),
            }
        }

        Ok(all_packages)
    }

    fn get_package(&self, id: &str) -> BackendResult<AppPackage> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        let output = Command::new("flatpak")
            .args(["info", id])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute flatpak: {}", e)))?;

        if !output.status.success() {
            // Try searching in remotes
            let remote_output = Command::new("flatpak")
                .args(["remote-info", "flathub", id])
                .output()
                .map_err(|e| BackendError::Other(format!("Failed to execute flatpak: {}", e)))?;

            if !remote_output.status.success() {
                return Err(BackendError::NotFound(id.to_string()));
            }

            let info = String::from_utf8_lossy(&remote_output.stdout);
            return self.parse_package_info(id, &info)
                .ok_or_else(|| BackendError::NotFound(id.to_string()));
        }

        let info = String::from_utf8_lossy(&output.stdout);
        let mut package = self.parse_package_info(id, &info)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))?;
        package.status = InstallStatus::Installed;

        Ok(package)
    }

    fn list_installed(&self) -> BackendResult<Vec<AppPackage>> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        self.get_installed_apps()
    }

    fn list_updates(&self) -> BackendResult<Vec<AppPackage>> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        self.check_updates()
    }

    fn install(&self, id: &str, progress: Sender<ProgressUpdate>) -> BackendResult<()> {
        if !self.available {
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        info!("Installing Flatpak package: {}", id);

        // Send initial progress
        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Install,
            package_id: id.to_string(),
            progress: 0,
            message: "Starting installation...".to_string(),
        });

        let output = Command::new("flatpak")
            .args(["install", "-y", "flathub", id])
            .output()
            .map_err(|e| BackendError::InstallFailed(format!("Failed to execute flatpak: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::InstallFailed(stderr.to_string()));
        }

        // Send completion progress
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
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        info!("Uninstalling Flatpak package: {}", id);

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Uninstall,
            package_id: id.to_string(),
            progress: 0,
            message: "Starting uninstallation...".to_string(),
        });

        let output = Command::new("flatpak")
            .args(["uninstall", "-y", id])
            .output()
            .map_err(|e| BackendError::UninstallFailed(format!("Failed to execute flatpak: {}", e)))?;

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
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        info!("Updating Flatpak package: {}", id);

        let _ = progress.send_blocking(ProgressUpdate {
            operation: super::OperationType::Update,
            package_id: id.to_string(),
            progress: 0,
            message: "Starting update...".to_string(),
        });

        let output = Command::new("flatpak")
            .args(["update", "-y", id])
            .output()
            .map_err(|e| BackendError::Other(format!("Failed to execute flatpak: {}", e)))?;

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
            return Err(BackendError::BackendUnavailable("Flatpak is not installed".into()));
        }

        info!("Refreshing Flatpak remotes");

        for remote in &self.remotes {
            let output = Command::new("flatpak")
                .args(["update", "--appstream", remote])
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    debug!("Refreshed remote: {}", remote);
                }
                Ok(out) => {
                    warn!(
                        "Failed to refresh {}: {}",
                        remote,
                        String::from_utf8_lossy(&out.stderr)
                    );
                }
                Err(e) => {
                    warn!("Error refreshing {}: {}", remote, e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatpak_backend_creation() {
        let backend = FlatpakBackend::new();
        assert_eq!(backend.name(), "Flatpak");
    }
}
