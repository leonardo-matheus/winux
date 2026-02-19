//! Backend module for update management

mod apt;
mod flatpak;
mod snap;
mod fwupd;

pub use apt::AptBackend;
pub use flatpak::FlatpakBackend;
pub use snap::SnapBackend;
pub use fwupd::FwupdBackend;

use std::collections::HashMap;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Source of the update
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateSource {
    Apt,
    Flatpak,
    Snap,
    Fwupd,
}

impl std::fmt::Display for UpdateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateSource::Apt => write!(f, "APT"),
            UpdateSource::Flatpak => write!(f, "Flatpak"),
            UpdateSource::Snap => write!(f, "Snap"),
            UpdateSource::Fwupd => write!(f, "Firmware"),
        }
    }
}

/// Priority level for updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdatePriority {
    Security,
    Important,
    Normal,
    Optional,
}

/// Represents a single package update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpdate {
    pub id: String,
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub source: UpdateSource,
    pub download_size: u64,
    pub installed_size: u64,
    pub description: String,
    pub changelog: Option<String>,
    pub priority: UpdatePriority,
    pub requires_restart: bool,
}

impl PackageUpdate {
    pub fn size_display(&self) -> String {
        format_size(self.download_size)
    }
}

/// Represents a completed update in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHistoryEntry {
    pub id: String,
    pub package_name: String,
    pub old_version: String,
    pub new_version: String,
    pub source: UpdateSource,
    pub timestamp: DateTime<Local>,
    pub success: bool,
    pub is_security: bool,
}

/// Update progress information
#[derive(Debug, Clone)]
pub struct UpdateProgress {
    pub current_package: String,
    pub current_index: usize,
    pub total_packages: usize,
    pub package_progress: f64,
    pub overall_progress: f64,
    pub status: UpdateStatus,
    pub log_output: String,
}

/// Status of an update operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    Checking,
    Downloading,
    Installing,
    Configuring,
    Finished,
    Failed,
    Cancelled,
}

/// Settings for automatic updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub auto_check: bool,
    pub check_frequency_hours: u32,
    pub auto_download: bool,
    pub auto_install_security: bool,
    pub auto_install_all: bool,
    pub notify_available: bool,
    pub notify_security: bool,
    pub notify_complete: bool,
    pub notify_restart: bool,
    pub maintenance_window_enabled: bool,
    pub maintenance_start_hour: u8,
    pub maintenance_duration_hours: u8,
    pub weekdays_only: bool,
    pub sources_enabled: HashMap<UpdateSource, bool>,
    pub pause_on_metered: bool,
    pub parallel_downloads: u8,
    pub bandwidth_limit_kbps: Option<u32>,
    pub keep_cache: bool,
    pub auto_remove_orphans: bool,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        let mut sources_enabled = HashMap::new();
        sources_enabled.insert(UpdateSource::Apt, true);
        sources_enabled.insert(UpdateSource::Flatpak, true);
        sources_enabled.insert(UpdateSource::Snap, true);
        sources_enabled.insert(UpdateSource::Fwupd, true);

        Self {
            auto_check: true,
            check_frequency_hours: 24,
            auto_download: true,
            auto_install_security: false,
            auto_install_all: false,
            notify_available: true,
            notify_security: true,
            notify_complete: true,
            notify_restart: true,
            maintenance_window_enabled: false,
            maintenance_start_hour: 2,
            maintenance_duration_hours: 2,
            weekdays_only: false,
            sources_enabled,
            pause_on_metered: true,
            parallel_downloads: 2,
            bandwidth_limit_kbps: None,
            keep_cache: false,
            auto_remove_orphans: true,
        }
    }
}

/// Main update manager
pub struct UpdateManager {
    apt: AptBackend,
    flatpak: FlatpakBackend,
    snap: SnapBackend,
    fwupd: FwupdBackend,
    settings: UpdateSettings,
    available_updates: Vec<PackageUpdate>,
    history: Vec<UpdateHistoryEntry>,
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            apt: AptBackend::new(),
            flatpak: FlatpakBackend::new(),
            snap: SnapBackend::new(),
            fwupd: FwupdBackend::new(),
            settings: UpdateSettings::default(),
            available_updates: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Check all enabled sources for updates
    pub async fn check_all_updates(&mut self) -> anyhow::Result<Vec<PackageUpdate>> {
        let mut all_updates = Vec::new();

        if *self.settings.sources_enabled.get(&UpdateSource::Apt).unwrap_or(&true) {
            let apt_updates = self.apt.check_updates().await?;
            all_updates.extend(apt_updates);
        }

        if *self.settings.sources_enabled.get(&UpdateSource::Flatpak).unwrap_or(&true) {
            let flatpak_updates = self.flatpak.check_updates().await?;
            all_updates.extend(flatpak_updates);
        }

        if *self.settings.sources_enabled.get(&UpdateSource::Snap).unwrap_or(&true) {
            let snap_updates = self.snap.check_updates().await?;
            all_updates.extend(snap_updates);
        }

        if *self.settings.sources_enabled.get(&UpdateSource::Fwupd).unwrap_or(&true) {
            let fwupd_updates = self.fwupd.check_updates().await?;
            all_updates.extend(fwupd_updates);
        }

        self.available_updates = all_updates.clone();
        Ok(all_updates)
    }

    /// Install selected updates
    pub async fn install_updates(
        &mut self,
        update_ids: Vec<String>,
        progress_callback: impl Fn(UpdateProgress) + Send + 'static,
    ) -> anyhow::Result<()> {
        let updates: Vec<_> = self.available_updates
            .iter()
            .filter(|u| update_ids.contains(&u.id))
            .cloned()
            .collect();

        let total = updates.len();

        for (idx, update) in updates.iter().enumerate() {
            let progress = UpdateProgress {
                current_package: update.name.clone(),
                current_index: idx + 1,
                total_packages: total,
                package_progress: 0.0,
                overall_progress: idx as f64 / total as f64,
                status: UpdateStatus::Installing,
                log_output: format!("Installing {}...\n", update.name),
            };
            progress_callback(progress);

            match update.source {
                UpdateSource::Apt => self.apt.install_package(&update.name).await?,
                UpdateSource::Flatpak => self.flatpak.install_update(&update.id).await?,
                UpdateSource::Snap => self.snap.refresh_package(&update.name).await?,
                UpdateSource::Fwupd => self.fwupd.install_update(&update.id).await?,
            }

            // Add to history
            self.history.push(UpdateHistoryEntry {
                id: update.id.clone(),
                package_name: update.name.clone(),
                old_version: update.current_version.clone(),
                new_version: update.new_version.clone(),
                source: update.source,
                timestamp: Local::now(),
                success: true,
                is_security: update.priority == UpdatePriority::Security,
            });
        }

        // Remove installed updates from available list
        self.available_updates.retain(|u| !update_ids.contains(&u.id));

        Ok(())
    }

    /// Get available updates
    pub fn available_updates(&self) -> &[PackageUpdate] {
        &self.available_updates
    }

    /// Get update history
    pub fn history(&self) -> &[UpdateHistoryEntry] {
        &self.history
    }

    /// Get current settings
    pub fn settings(&self) -> &UpdateSettings {
        &self.settings
    }

    /// Update settings
    pub fn set_settings(&mut self, settings: UpdateSettings) {
        self.settings = settings;
    }

    /// Get total download size for selected updates
    pub fn total_download_size(&self, update_ids: &[String]) -> u64 {
        self.available_updates
            .iter()
            .filter(|u| update_ids.contains(&u.id))
            .map(|u| u.download_size)
            .sum()
    }

    /// Count updates by source
    pub fn count_by_source(&self) -> HashMap<UpdateSource, usize> {
        let mut counts = HashMap::new();
        for update in &self.available_updates {
            *counts.entry(update.source).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format byte size to human readable
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
