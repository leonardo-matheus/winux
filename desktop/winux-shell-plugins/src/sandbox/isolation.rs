//! Process isolation for plugins
//!
//! Provides sandboxing capabilities to isolate plugins from the system.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::permissions::PermissionSet;

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable sandboxing
    pub enabled: bool,
    /// Allowed read paths
    pub read_paths: Vec<PathBuf>,
    /// Allowed write paths
    pub write_paths: Vec<PathBuf>,
    /// Allowed network hosts
    pub network_hosts: Vec<String>,
    /// Allow localhost network access
    pub allow_localhost: bool,
    /// Environment variables to pass through
    pub env_passthrough: Vec<String>,
    /// Environment variables to set
    pub env_set: HashMap<String, String>,
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
    /// Maximum CPU time in seconds
    pub max_cpu_time: Option<u32>,
    /// Maximum number of open file descriptors
    pub max_fds: Option<u32>,
    /// Maximum number of processes/threads
    pub max_processes: Option<u32>,
    /// Allowed D-Bus names
    pub dbus_names: Vec<String>,
    /// Allow session D-Bus
    pub allow_session_bus: bool,
    /// Allow system D-Bus
    pub allow_system_bus: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            network_hosts: Vec::new(),
            allow_localhost: false,
            env_passthrough: vec![
                "HOME".to_string(),
                "USER".to_string(),
                "LANG".to_string(),
                "LC_ALL".to_string(),
                "XDG_RUNTIME_DIR".to_string(),
                "XDG_DATA_DIRS".to_string(),
                "XDG_CONFIG_DIRS".to_string(),
                "DISPLAY".to_string(),
                "WAYLAND_DISPLAY".to_string(),
                "DBUS_SESSION_BUS_ADDRESS".to_string(),
            ],
            env_set: HashMap::new(),
            max_memory: Some(256 * 1024 * 1024), // 256 MB
            max_cpu_time: None,
            max_fds: Some(256),
            max_processes: Some(10),
            dbus_names: Vec::new(),
            allow_session_bus: true,
            allow_system_bus: false,
        }
    }
}

impl SandboxConfig {
    /// Create a sandbox config from a permission set
    pub fn from_permissions(permissions: &PermissionSet, plugin_data_dir: &PathBuf) -> Self {
        use super::permissions::Permission;

        let mut config = Self::default();

        // Always allow plugin's own data directory
        config.read_paths.push(plugin_data_dir.clone());
        config.write_paths.push(plugin_data_dir.clone());

        for permission in permissions.permissions() {
            match permission {
                Permission::Network => {
                    config.network_hosts.push("*".to_string());
                    config.allow_localhost = true;
                }
                Permission::NetworkHost(host) => {
                    config.network_hosts.push(host.clone());
                }
                Permission::NetworkLocalhost => {
                    config.allow_localhost = true;
                }
                Permission::Filesystem => {
                    config.read_paths.push(PathBuf::from("/"));
                    config.write_paths.push(PathBuf::from("/"));
                }
                Permission::FilesystemHome => {
                    if let Some(home) = dirs::home_dir() {
                        config.read_paths.push(home);
                    }
                }
                Permission::FilesystemRead(path) => {
                    config.read_paths.push(path.clone());
                }
                Permission::FilesystemWrite(path) => {
                    config.read_paths.push(path.clone());
                    config.write_paths.push(path.clone());
                }
                Permission::FilesystemDownloads => {
                    if let Some(path) = dirs::download_dir() {
                        config.read_paths.push(path.clone());
                        config.write_paths.push(path);
                    }
                }
                Permission::FilesystemDocuments => {
                    if let Some(path) = dirs::document_dir() {
                        config.read_paths.push(path.clone());
                        config.write_paths.push(path);
                    }
                }
                Permission::FilesystemPictures => {
                    if let Some(path) = dirs::picture_dir() {
                        config.read_paths.push(path.clone());
                        config.write_paths.push(path);
                    }
                }
                Permission::DBusSession => {
                    config.allow_session_bus = true;
                }
                Permission::DBusSystem => {
                    config.allow_system_bus = true;
                }
                Permission::DBusName(name) => {
                    config.dbus_names.push(name.clone());
                }
                _ => {}
            }
        }

        config
    }

    /// Create a minimal sandbox config
    pub fn minimal() -> Self {
        Self {
            enabled: true,
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            network_hosts: Vec::new(),
            allow_localhost: false,
            env_passthrough: vec!["LANG".to_string(), "LC_ALL".to_string()],
            env_set: HashMap::new(),
            max_memory: Some(64 * 1024 * 1024), // 64 MB
            max_cpu_time: Some(60),
            max_fds: Some(32),
            max_processes: Some(1),
            dbus_names: Vec::new(),
            allow_session_bus: false,
            allow_system_bus: false,
        }
    }

    /// Create a permissive config (for trusted plugins)
    pub fn permissive() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Add a read path
    pub fn allow_read(&mut self, path: PathBuf) -> &mut Self {
        self.read_paths.push(path);
        self
    }

    /// Add a write path
    pub fn allow_write(&mut self, path: PathBuf) -> &mut Self {
        self.write_paths.push(path.clone());
        self.read_paths.push(path);
        self
    }

    /// Allow a network host
    pub fn allow_host(&mut self, host: &str) -> &mut Self {
        self.network_hosts.push(host.to_string());
        self
    }

    /// Set memory limit
    pub fn memory_limit(&mut self, bytes: u64) -> &mut Self {
        self.max_memory = Some(bytes);
        self
    }

    /// Set process limit
    pub fn process_limit(&mut self, count: u32) -> &mut Self {
        self.max_processes = Some(count);
        self
    }
}

/// A sandboxed process handle
#[derive(Debug)]
pub struct SandboxedProcess {
    /// Process ID
    pub pid: Option<u32>,
    /// Sandbox configuration used
    pub config: SandboxConfig,
    /// Plugin ID
    pub plugin_id: String,
}

impl SandboxedProcess {
    /// Create a new sandboxed process handle
    pub fn new(plugin_id: &str, config: SandboxConfig) -> Self {
        Self {
            pid: None,
            config,
            plugin_id: plugin_id.to_string(),
        }
    }

    /// Check if the process is running
    pub fn is_running(&self) -> bool {
        self.pid.is_some()
    }

    /// Get the process ID
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Setup sandbox using available mechanisms
    ///
    /// This attempts to use available sandboxing on the platform:
    /// - Linux: seccomp, namespaces, landlock
    /// - All platforms: resource limits
    #[cfg(target_os = "linux")]
    pub fn setup_sandbox(&self) -> Result<(), SandboxError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Setup resource limits
        self.setup_resource_limits()?;

        // Setup seccomp filter
        #[cfg(feature = "sandbox")]
        self.setup_seccomp()?;

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn setup_sandbox(&self) -> Result<(), SandboxError> {
        if !self.config.enabled {
            return Ok(());
        }

        // On non-Linux platforms, we only apply resource limits
        self.setup_resource_limits()?;

        Ok(())
    }

    /// Setup resource limits
    fn setup_resource_limits(&self) -> Result<(), SandboxError> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            // Note: In a real implementation, we would use setrlimit here
            // For now, we just document what would be done
            log::debug!(
                "Would set resource limits: memory={:?}, fds={:?}, processes={:?}",
                self.config.max_memory,
                self.config.max_fds,
                self.config.max_processes
            );
        }

        Ok(())
    }

    /// Setup seccomp filter (Linux only)
    #[cfg(all(target_os = "linux", feature = "sandbox"))]
    fn setup_seccomp(&self) -> Result<(), SandboxError> {
        use seccompiler::{BpfMap, SeccompAction, SeccompFilter, SeccompRule};

        // Build seccomp filter based on permissions
        // This is a simplified example - a real implementation would be more comprehensive

        log::debug!("Would setup seccomp filter for plugin {}", self.plugin_id);

        Ok(())
    }

    /// Terminate the sandboxed process
    pub fn terminate(&mut self) -> Result<(), SandboxError> {
        if let Some(pid) = self.pid {
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
                    .map_err(|e| SandboxError::ProcessError(e.to_string()))?;
            }

            #[cfg(not(unix))]
            {
                log::warn!("Process termination not implemented for this platform");
            }

            self.pid = None;
        }
        Ok(())
    }

    /// Force kill the sandboxed process
    pub fn kill(&mut self) -> Result<(), SandboxError> {
        if let Some(pid) = self.pid {
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
                    .map_err(|e| SandboxError::ProcessError(e.to_string()))?;
            }

            #[cfg(not(unix))]
            {
                log::warn!("Process kill not implemented for this platform");
            }

            self.pid = None;
        }
        Ok(())
    }
}

/// Sandbox-related errors
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Failed to create sandbox: {0}")]
    CreationFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource limit error: {0}")]
    ResourceLimit(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Seccomp error: {0}")]
    SeccompError(String),

    #[error("Namespace error: {0}")]
    NamespaceError(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
}

/// Sandbox violation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxViolation {
    /// Plugin that caused the violation
    pub plugin_id: String,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Details about the violation
    pub details: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of sandbox violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Attempted unauthorized file access
    FileAccess { path: String, write: bool },
    /// Attempted unauthorized network access
    NetworkAccess { host: String, port: u16 },
    /// Attempted unauthorized D-Bus access
    DBusAccess { name: String },
    /// Attempted unauthorized syscall
    Syscall { name: String },
    /// Resource limit exceeded
    ResourceLimit { resource: String },
    /// Other violation
    Other(String),
}

impl SandboxViolation {
    /// Create a new violation
    pub fn new(plugin_id: &str, violation_type: ViolationType, details: &str) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            violation_type,
            details: details.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(config.enabled);
        assert!(config.allow_session_bus);
        assert!(!config.allow_system_bus);
    }

    #[test]
    fn test_sandbox_config_minimal() {
        let config = SandboxConfig::minimal();
        assert!(config.enabled);
        assert!(!config.allow_session_bus);
        assert!(config.max_memory.is_some());
    }

    #[test]
    fn test_sandbox_config_from_permissions() {
        use super::super::permissions::Permission;

        let mut permissions = PermissionSet::new();
        permissions.add(Permission::NetworkLocalhost);
        permissions.add(Permission::FilesystemDownloads);

        let config = SandboxConfig::from_permissions(&permissions, &PathBuf::from("/tmp/plugin"));
        assert!(config.allow_localhost);
    }
}
