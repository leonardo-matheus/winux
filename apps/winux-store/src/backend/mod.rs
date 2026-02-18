//! Backend module for package management
//!
//! Provides unified interface for different package managers (Flatpak, APT).

mod apt;
mod flatpak;

pub use apt::AptBackend;
pub use flatpak::FlatpakBackend;

use async_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPackage {
    /// Unique package identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Short description
    pub summary: String,
    /// Full description
    pub description: String,
    /// Package version
    pub version: String,
    /// Icon name or path
    pub icon: String,
    /// Categories (e.g., "AudioVideo", "Development")
    pub categories: Vec<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// License
    pub license: Option<String>,
    /// Package source (Flatpak, APT, etc.)
    pub source: PackageSource,
    /// Installation status
    pub status: InstallStatus,
    /// Download size in bytes
    pub download_size: u64,
    /// Installed size in bytes
    pub installed_size: u64,
    /// Screenshot URLs
    pub screenshots: Vec<String>,
    /// User rating (0.0 - 5.0)
    pub rating: Option<f32>,
    /// Number of ratings
    pub rating_count: u32,
}

impl AppPackage {
    pub fn new(id: &str, name: &str, source: PackageSource) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            summary: String::new(),
            description: String::new(),
            version: String::new(),
            icon: "application-x-executable".to_string(),
            categories: Vec::new(),
            homepage: None,
            license: None,
            source,
            status: InstallStatus::Available,
            download_size: 0,
            installed_size: 0,
            screenshots: Vec::new(),
            rating: None,
            rating_count: 0,
        }
    }
}

/// Package source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageSource {
    Flatpak,
    Apt,
    Snap,
    AppImage,
    Native,
}

impl std::fmt::Display for PackageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSource::Flatpak => write!(f, "Flatpak"),
            PackageSource::Apt => write!(f, "APT"),
            PackageSource::Snap => write!(f, "Snap"),
            PackageSource::AppImage => write!(f, "AppImage"),
            PackageSource::Native => write!(f, "Native"),
        }
    }
}

/// Installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallStatus {
    /// Not installed
    Available,
    /// Currently installed
    Installed,
    /// Update available
    UpdateAvailable,
    /// Currently installing
    Installing,
    /// Currently uninstalling
    Uninstalling,
    /// Currently updating
    Updating,
}

/// Backend operation result
pub type BackendResult<T> = Result<T, BackendError>;

/// Backend error types
#[derive(Debug, Clone)]
pub enum BackendError {
    /// Package not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Network error
    NetworkError(String),
    /// Installation failed
    InstallFailed(String),
    /// Uninstall failed
    UninstallFailed(String),
    /// Backend not available
    BackendUnavailable(String),
    /// Generic error
    Other(String),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::NotFound(msg) => write!(f, "Package not found: {}", msg),
            BackendError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            BackendError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            BackendError::InstallFailed(msg) => write!(f, "Installation failed: {}", msg),
            BackendError::UninstallFailed(msg) => write!(f, "Uninstall failed: {}", msg),
            BackendError::BackendUnavailable(msg) => write!(f, "Backend unavailable: {}", msg),
            BackendError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

/// Progress update during operations
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    /// Operation type
    pub operation: OperationType,
    /// Package ID
    pub package_id: String,
    /// Progress percentage (0-100)
    pub progress: u32,
    /// Status message
    pub message: String,
}

/// Type of operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Install,
    Uninstall,
    Update,
    Refresh,
    Search,
}

/// Trait for package management backends
pub trait PackageBackend: Send + Sync {
    /// Get backend name
    fn name(&self) -> &str;

    /// Check if backend is available
    fn is_available(&self) -> bool;

    /// Search for packages
    fn search(&self, query: &str) -> BackendResult<Vec<AppPackage>>;

    /// Get package details
    fn get_package(&self, id: &str) -> BackendResult<AppPackage>;

    /// List installed packages
    fn list_installed(&self) -> BackendResult<Vec<AppPackage>>;

    /// List available updates
    fn list_updates(&self) -> BackendResult<Vec<AppPackage>>;

    /// Install a package
    fn install(&self, id: &str, progress: Sender<ProgressUpdate>) -> BackendResult<()>;

    /// Uninstall a package
    fn uninstall(&self, id: &str, progress: Sender<ProgressUpdate>) -> BackendResult<()>;

    /// Update a package
    fn update(&self, id: &str, progress: Sender<ProgressUpdate>) -> BackendResult<()>;

    /// Refresh package cache
    fn refresh(&self) -> BackendResult<()>;
}

/// Unified package manager that aggregates multiple backends
pub struct PackageManager {
    backends: Vec<Box<dyn PackageBackend>>,
    cache: HashMap<String, AppPackage>,
}

impl PackageManager {
    pub fn new() -> Self {
        let mut backends: Vec<Box<dyn PackageBackend>> = Vec::new();

        // Add Flatpak backend
        let flatpak = FlatpakBackend::new();
        if flatpak.is_available() {
            info!("Flatpak backend available");
            backends.push(Box::new(flatpak));
        }

        // Add APT backend
        let apt = AptBackend::new();
        if apt.is_available() {
            info!("APT backend available");
            backends.push(Box::new(apt));
        }

        Self {
            backends,
            cache: HashMap::new(),
        }
    }

    /// Search across all backends
    pub fn search(&self, query: &str) -> Vec<AppPackage> {
        let mut results = Vec::new();

        for backend in &self.backends {
            match backend.search(query) {
                Ok(packages) => {
                    debug!("Found {} packages from {}", packages.len(), backend.name());
                    results.extend(packages);
                }
                Err(e) => {
                    error!("Search error in {}: {}", backend.name(), e);
                }
            }
        }

        // Sort by relevance (name match first)
        results.sort_by(|a, b| {
            let a_exact = a.name.to_lowercase() == query.to_lowercase();
            let b_exact = b.name.to_lowercase() == query.to_lowercase();
            b_exact.cmp(&a_exact)
        });

        results
    }

    /// Get all installed packages
    pub fn get_installed(&self) -> Vec<AppPackage> {
        let mut results = Vec::new();

        for backend in &self.backends {
            match backend.list_installed() {
                Ok(packages) => {
                    results.extend(packages);
                }
                Err(e) => {
                    error!("Error listing installed from {}: {}", backend.name(), e);
                }
            }
        }

        results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        results
    }

    /// Get available updates
    pub fn get_updates(&self) -> Vec<AppPackage> {
        let mut results = Vec::new();

        for backend in &self.backends {
            match backend.list_updates() {
                Ok(packages) => {
                    results.extend(packages);
                }
                Err(e) => {
                    error!("Error listing updates from {}: {}", backend.name(), e);
                }
            }
        }

        results
    }

    /// Install a package
    pub fn install(&self, package: &AppPackage) -> (Receiver<ProgressUpdate>, Receiver<BackendResult<()>>) {
        let (progress_tx, progress_rx) = async_channel::unbounded();
        let (result_tx, result_rx) = async_channel::bounded(1);

        // Find appropriate backend
        for backend in &self.backends {
            if backend.name() == package.source.to_string() {
                let id = package.id.clone();
                let _ = std::thread::spawn(move || {
                    // This would be async in real implementation
                    info!("Installing package: {}", id);
                });
                break;
            }
        }

        (progress_rx, result_rx)
    }

    /// Refresh all backend caches
    pub fn refresh(&self) {
        for backend in &self.backends {
            if let Err(e) = backend.refresh() {
                error!("Error refreshing {}: {}", backend.name(), e);
            }
        }
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new()
    }
}
