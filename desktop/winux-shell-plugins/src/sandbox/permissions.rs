//! Permission system for plugins
//!
//! Defines permissions that plugins can request and the system can grant.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Individual permissions that can be granted to plugins
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Network permissions
    /// Access to the network (all)
    Network,
    /// Access to specific hosts
    NetworkHost(String),
    /// Access to localhost only
    NetworkLocalhost,

    // Filesystem permissions
    /// Full filesystem access
    Filesystem,
    /// Read access to user home directory
    FilesystemHome,
    /// Read access to specific paths
    FilesystemRead(PathBuf),
    /// Write access to specific paths
    FilesystemWrite(PathBuf),
    /// Access to downloads folder
    FilesystemDownloads,
    /// Access to documents folder
    FilesystemDocuments,
    /// Access to pictures folder
    FilesystemPictures,

    // System permissions
    /// Read system information (CPU, memory, etc.)
    SystemInfo,
    /// Access to system notifications
    Notifications,
    /// Send notifications
    NotificationsSend,
    /// Access clipboard
    Clipboard,
    /// Write to clipboard
    ClipboardWrite,
    /// Access to audio system
    Audio,
    /// Control audio playback
    AudioControl,
    /// Access to display/screen information
    Display,
    /// Take screenshots
    Screenshot,
    /// Access to power management
    Power,
    /// Prevent system sleep
    PowerInhibit,
    /// Access to Bluetooth
    Bluetooth,
    /// Access to location services
    Location,
    /// Access to camera
    Camera,
    /// Access to microphone
    Microphone,

    // Desktop integration
    /// Register keyboard shortcuts
    KeyboardShortcuts,
    /// Add panel widgets
    PanelWidgets,
    /// Add launcher providers
    LauncherProviders,
    /// Add settings pages
    SettingsPages,
    /// Add context menu items
    ContextMenus,
    /// Run background tasks
    BackgroundTasks,
    /// Auto-start on login
    Autostart,

    // D-Bus permissions
    /// Access to session bus
    DBusSession,
    /// Access to system bus
    DBusSystem,
    /// Access to specific D-Bus names
    DBusName(String),

    // Process permissions
    /// Spawn child processes
    SpawnProcess,
    /// Execute specific commands
    Execute(String),

    // Storage permissions
    /// Access to plugin's own data directory
    OwnData,
    /// Access to shared plugin data
    SharedData,
    /// Access to GSettings/dconf
    GSettings,

    // Custom permission
    Custom(String),
}

impl Permission {
    /// Check if this permission implies another
    pub fn implies(&self, other: &Permission) -> bool {
        match (self, other) {
            // Network implies specific hosts
            (Permission::Network, Permission::NetworkHost(_)) => true,
            (Permission::Network, Permission::NetworkLocalhost) => true,

            // Filesystem implies specific paths
            (Permission::Filesystem, Permission::FilesystemHome) => true,
            (Permission::Filesystem, Permission::FilesystemRead(_)) => true,
            (Permission::Filesystem, Permission::FilesystemWrite(_)) => true,
            (Permission::Filesystem, Permission::FilesystemDownloads) => true,
            (Permission::Filesystem, Permission::FilesystemDocuments) => true,
            (Permission::Filesystem, Permission::FilesystemPictures) => true,

            // FilesystemWrite implies FilesystemRead for same path
            (Permission::FilesystemWrite(p1), Permission::FilesystemRead(p2)) if p1 == p2 => true,

            // Notifications send implies notifications read
            (Permission::NotificationsSend, Permission::Notifications) => true,

            // Clipboard write implies clipboard read
            (Permission::ClipboardWrite, Permission::Clipboard) => true,

            // Audio control implies audio access
            (Permission::AudioControl, Permission::Audio) => true,

            // D-Bus session implies specific names on session bus
            (Permission::DBusSession, Permission::DBusName(name))
                if !name.starts_with("org.freedesktop.systemd") =>
            {
                true
            }

            // Same permission
            _ if self == other => true,

            _ => false,
        }
    }

    /// Get the display name for this permission
    pub fn display_name(&self) -> &str {
        match self {
            Permission::Network => "Network Access",
            Permission::NetworkHost(_) => "Network Host Access",
            Permission::NetworkLocalhost => "Localhost Access",
            Permission::Filesystem => "Full Filesystem Access",
            Permission::FilesystemHome => "Home Directory Access",
            Permission::FilesystemRead(_) => "File Read Access",
            Permission::FilesystemWrite(_) => "File Write Access",
            Permission::FilesystemDownloads => "Downloads Folder Access",
            Permission::FilesystemDocuments => "Documents Folder Access",
            Permission::FilesystemPictures => "Pictures Folder Access",
            Permission::SystemInfo => "System Information",
            Permission::Notifications => "Read Notifications",
            Permission::NotificationsSend => "Send Notifications",
            Permission::Clipboard => "Read Clipboard",
            Permission::ClipboardWrite => "Write to Clipboard",
            Permission::Audio => "Audio Access",
            Permission::AudioControl => "Audio Control",
            Permission::Display => "Display Information",
            Permission::Screenshot => "Take Screenshots",
            Permission::Power => "Power Management",
            Permission::PowerInhibit => "Prevent Sleep",
            Permission::Bluetooth => "Bluetooth Access",
            Permission::Location => "Location Services",
            Permission::Camera => "Camera Access",
            Permission::Microphone => "Microphone Access",
            Permission::KeyboardShortcuts => "Keyboard Shortcuts",
            Permission::PanelWidgets => "Panel Widgets",
            Permission::LauncherProviders => "Launcher Providers",
            Permission::SettingsPages => "Settings Pages",
            Permission::ContextMenus => "Context Menus",
            Permission::BackgroundTasks => "Background Tasks",
            Permission::Autostart => "Auto-start",
            Permission::DBusSession => "Session Bus Access",
            Permission::DBusSystem => "System Bus Access",
            Permission::DBusName(_) => "D-Bus Name Access",
            Permission::SpawnProcess => "Spawn Processes",
            Permission::Execute(_) => "Execute Command",
            Permission::OwnData => "Own Data Access",
            Permission::SharedData => "Shared Data Access",
            Permission::GSettings => "Settings Access",
            Permission::Custom(_) => "Custom Permission",
        }
    }

    /// Get the description for this permission
    pub fn description(&self) -> String {
        match self {
            Permission::Network => "Access the internet and local network".to_string(),
            Permission::NetworkHost(host) => format!("Connect to {}", host),
            Permission::NetworkLocalhost => "Connect to services on this computer".to_string(),
            Permission::Filesystem => "Read and write all files".to_string(),
            Permission::FilesystemHome => "Read files in your home directory".to_string(),
            Permission::FilesystemRead(path) => format!("Read files in {:?}", path),
            Permission::FilesystemWrite(path) => format!("Read and write files in {:?}", path),
            Permission::FilesystemDownloads => "Access your Downloads folder".to_string(),
            Permission::FilesystemDocuments => "Access your Documents folder".to_string(),
            Permission::FilesystemPictures => "Access your Pictures folder".to_string(),
            Permission::SystemInfo => "Read CPU, memory, and system information".to_string(),
            Permission::Notifications => "See system notifications".to_string(),
            Permission::NotificationsSend => "Show notifications".to_string(),
            Permission::Clipboard => "Read clipboard contents".to_string(),
            Permission::ClipboardWrite => "Copy to clipboard".to_string(),
            Permission::Audio => "Access audio devices".to_string(),
            Permission::AudioControl => "Control audio playback".to_string(),
            Permission::Display => "Get information about displays".to_string(),
            Permission::Screenshot => "Capture the screen".to_string(),
            Permission::Power => "Access power/battery information".to_string(),
            Permission::PowerInhibit => "Prevent the computer from sleeping".to_string(),
            Permission::Bluetooth => "Access Bluetooth devices".to_string(),
            Permission::Location => "Access your location".to_string(),
            Permission::Camera => "Use the camera".to_string(),
            Permission::Microphone => "Use the microphone".to_string(),
            Permission::KeyboardShortcuts => "Register global keyboard shortcuts".to_string(),
            Permission::PanelWidgets => "Add widgets to the panel".to_string(),
            Permission::LauncherProviders => "Add search results to the launcher".to_string(),
            Permission::SettingsPages => "Add pages to system settings".to_string(),
            Permission::ContextMenus => "Add items to context menus".to_string(),
            Permission::BackgroundTasks => "Run tasks in the background".to_string(),
            Permission::Autostart => "Start automatically on login".to_string(),
            Permission::DBusSession => "Communicate with other applications".to_string(),
            Permission::DBusSystem => "Communicate with system services".to_string(),
            Permission::DBusName(name) => format!("Access D-Bus service: {}", name),
            Permission::SpawnProcess => "Run other programs".to_string(),
            Permission::Execute(cmd) => format!("Run command: {}", cmd),
            Permission::OwnData => "Store plugin data".to_string(),
            Permission::SharedData => "Access shared plugin data".to_string(),
            Permission::GSettings => "Read and write system settings".to_string(),
            Permission::Custom(name) => format!("Custom permission: {}", name),
        }
    }

    /// Check if this is a potentially dangerous permission
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Permission::Filesystem
                | Permission::DBusSystem
                | Permission::SpawnProcess
                | Permission::Execute(_)
                | Permission::Camera
                | Permission::Microphone
                | Permission::Location
                | Permission::Screenshot
        )
    }
}

/// A set of permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create an empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    /// Create a permission set with common safe permissions
    pub fn safe_defaults() -> Self {
        let mut set = Self::new();
        set.add(Permission::OwnData);
        set.add(Permission::NotificationsSend);
        set
    }

    /// Add a permission
    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission
    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if a permission is granted (directly or implied)
    pub fn has(&self, permission: &Permission) -> bool {
        if self.permissions.contains(permission) {
            return true;
        }

        // Check if any granted permission implies this one
        self.permissions.iter().any(|p| p.implies(permission))
    }

    /// Check if all permissions in another set are granted
    pub fn has_all(&self, other: &PermissionSet) -> bool {
        other.permissions.iter().all(|p| self.has(p))
    }

    /// Get all permissions
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }

    /// Get missing permissions (from required set)
    pub fn missing(&self, required: &PermissionSet) -> Vec<Permission> {
        required
            .permissions
            .iter()
            .filter(|p| !self.has(p))
            .cloned()
            .collect()
    }

    /// Check if any dangerous permissions are granted
    pub fn has_dangerous(&self) -> bool {
        self.permissions.iter().any(|p| p.is_dangerous())
    }

    /// Get all dangerous permissions that are granted
    pub fn dangerous_permissions(&self) -> Vec<&Permission> {
        self.permissions.iter().filter(|p| p.is_dangerous()).collect()
    }

    /// Merge another permission set into this one
    pub fn merge(&mut self, other: &PermissionSet) {
        for permission in &other.permissions {
            self.add(permission.clone());
        }
    }

    /// Create union of two permission sets
    pub fn union(&self, other: &PermissionSet) -> PermissionSet {
        let mut result = self.clone();
        result.merge(other);
        result
    }

    /// Create intersection of two permission sets
    pub fn intersection(&self, other: &PermissionSet) -> PermissionSet {
        let mut result = PermissionSet::new();
        for permission in &self.permissions {
            if other.has(permission) {
                result.add(permission.clone());
            }
        }
        result
    }

    /// Check if set is empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Get number of permissions
    pub fn len(&self) -> usize {
        self.permissions.len()
    }
}

impl FromIterator<Permission> for PermissionSet {
    fn from_iter<I: IntoIterator<Item = Permission>>(iter: I) -> Self {
        Self {
            permissions: iter.into_iter().collect(),
        }
    }
}

/// A permission request from a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin name
    pub plugin_name: String,
    /// Requested permissions
    pub permissions: PermissionSet,
    /// Reason for requesting (optional)
    pub reason: Option<String>,
}

impl PermissionRequest {
    /// Create a new permission request
    pub fn new(plugin_id: &str, plugin_name: &str, permissions: PermissionSet) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            plugin_name: plugin_name.to_string(),
            permissions,
            reason: None,
        }
    }

    /// Add a reason
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

/// Result of a permission request
#[derive(Debug, Clone)]
pub enum PermissionResponse {
    /// All permissions granted
    Granted(PermissionSet),
    /// Some permissions denied
    PartiallyGranted {
        granted: PermissionSet,
        denied: Vec<Permission>,
    },
    /// All permissions denied
    Denied,
    /// User needs to be asked
    RequiresUserConfirmation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_implies() {
        assert!(Permission::Network.implies(&Permission::NetworkHost("example.com".to_string())));
        assert!(Permission::Network.implies(&Permission::NetworkLocalhost));
        assert!(!Permission::NetworkLocalhost.implies(&Permission::Network));
    }

    #[test]
    fn test_permission_set_has() {
        let mut set = PermissionSet::new();
        set.add(Permission::Network);

        assert!(set.has(&Permission::Network));
        assert!(set.has(&Permission::NetworkLocalhost));
        assert!(!set.has(&Permission::Filesystem));
    }

    #[test]
    fn test_permission_set_missing() {
        let mut granted = PermissionSet::new();
        granted.add(Permission::Network);

        let mut required = PermissionSet::new();
        required.add(Permission::Network);
        required.add(Permission::Clipboard);

        let missing = granted.missing(&required);
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&Permission::Clipboard));
    }

    #[test]
    fn test_dangerous_permissions() {
        let mut set = PermissionSet::new();
        set.add(Permission::Network);
        assert!(!set.has_dangerous());

        set.add(Permission::Filesystem);
        assert!(set.has_dangerous());
    }
}
