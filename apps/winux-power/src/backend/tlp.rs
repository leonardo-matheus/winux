// TLP (Advanced Power Management) integration
// Provides additional power management features on Linux laptops

use std::collections::HashMap;
use std::process::Command;

/// TLP client for advanced power management
pub struct TlpClient {
    is_installed: bool,
    is_active: bool,
}

impl TlpClient {
    pub fn new() -> Self {
        let is_installed = Self::check_installed();
        let is_active = if is_installed {
            Self::check_active()
        } else {
            false
        };

        Self {
            is_installed,
            is_active,
        }
    }

    /// Check if TLP is installed
    fn check_installed() -> bool {
        Command::new("which")
            .arg("tlp")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if TLP service is active
    fn check_active() -> bool {
        Command::new("systemctl")
            .args(["is-active", "tlp.service"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false)
    }

    /// Check if TLP is installed on the system
    pub fn is_installed(&self) -> bool {
        self.is_installed
    }

    /// Check if TLP service is running
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get TLP status
    pub fn get_status(&self) -> TlpStatus {
        if !self.is_installed {
            return TlpStatus::default();
        }

        // Parse output of `tlp-stat -s`
        let output = Command::new("tlp-stat")
            .arg("-s")
            .output()
            .ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_status(&stdout)
        } else {
            TlpStatus::default()
        }
    }

    fn parse_status(output: &str) -> TlpStatus {
        let mut status = TlpStatus::default();

        for line in output.lines() {
            if line.contains("TLP power save") {
                status.power_save_enabled = line.contains("enabled");
            } else if line.contains("Mode") {
                if line.contains("AC") {
                    status.mode = TlpMode::AC;
                } else if line.contains("battery") {
                    status.mode = TlpMode::Battery;
                }
            }
        }

        status
    }

    /// Get current TLP configuration
    pub fn get_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();

        if !self.is_installed {
            return config;
        }

        // Read from /etc/tlp.conf or /etc/tlp.d/
        if let Ok(content) = std::fs::read_to_string("/etc/tlp.conf") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = line.split_once('=') {
                    config.insert(key.to_string(), value.trim_matches('"').to_string());
                }
            }
        }

        config
    }

    /// Get CPU scaling configuration
    pub fn get_cpu_config(&self) -> CpuConfig {
        let config = self.get_config();

        CpuConfig {
            scaling_governor_on_ac: config
                .get("CPU_SCALING_GOVERNOR_ON_AC")
                .cloned()
                .unwrap_or_else(|| "schedutil".to_string()),
            scaling_governor_on_bat: config
                .get("CPU_SCALING_GOVERNOR_ON_BAT")
                .cloned()
                .unwrap_or_else(|| "powersave".to_string()),
            energy_perf_policy_on_ac: config
                .get("CPU_ENERGY_PERF_POLICY_ON_AC")
                .cloned()
                .unwrap_or_else(|| "balance_performance".to_string()),
            energy_perf_policy_on_bat: config
                .get("CPU_ENERGY_PERF_POLICY_ON_BAT")
                .cloned()
                .unwrap_or_else(|| "balance_power".to_string()),
            boost_on_ac: config
                .get("CPU_BOOST_ON_AC")
                .map(|s| s == "1")
                .unwrap_or(true),
            boost_on_bat: config
                .get("CPU_BOOST_ON_BAT")
                .map(|s| s == "1")
                .unwrap_or(false),
            min_perf_on_ac: config
                .get("CPU_MIN_PERF_ON_AC")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            max_perf_on_ac: config
                .get("CPU_MAX_PERF_ON_AC")
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            min_perf_on_bat: config
                .get("CPU_MIN_PERF_ON_BAT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            max_perf_on_bat: config
                .get("CPU_MAX_PERF_ON_BAT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(80),
        }
    }

    /// Get disk configuration
    pub fn get_disk_config(&self) -> DiskConfig {
        let config = self.get_config();

        DiskConfig {
            devices: config
                .get("DISK_DEVICES")
                .cloned()
                .unwrap_or_else(|| "nvme0n1 sda".to_string()),
            apm_level_on_ac: config
                .get("DISK_APM_LEVEL_ON_AC")
                .cloned()
                .unwrap_or_else(|| "254".to_string()),
            apm_level_on_bat: config
                .get("DISK_APM_LEVEL_ON_BAT")
                .cloned()
                .unwrap_or_else(|| "128".to_string()),
            spindown_timeout_on_ac: config
                .get("DISK_SPINDOWN_TIMEOUT_ON_AC")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            spindown_timeout_on_bat: config
                .get("DISK_SPINDOWN_TIMEOUT_ON_BAT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(12),
            iosched: config
                .get("DISK_IOSCHED")
                .cloned()
                .unwrap_or_else(|| "mq-deadline".to_string()),
        }
    }

    /// Get USB autosuspend configuration
    pub fn get_usb_config(&self) -> UsbConfig {
        let config = self.get_config();

        UsbConfig {
            autosuspend: config
                .get("USB_AUTOSUSPEND")
                .map(|s| s == "1")
                .unwrap_or(true),
            autosuspend_disable_on_shutdown: config
                .get("USB_AUTOSUSPEND_DISABLE_ON_SHUTDOWN")
                .map(|s| s == "1")
                .unwrap_or(false),
            denylist: config
                .get("USB_DENYLIST")
                .cloned()
                .unwrap_or_default(),
            allowlist: config
                .get("USB_ALLOWLIST")
                .cloned()
                .unwrap_or_default(),
            exclude_audio: config
                .get("USB_EXCLUDE_AUDIO")
                .map(|s| s == "1")
                .unwrap_or(true),
            exclude_btusb: config
                .get("USB_EXCLUDE_BTUSB")
                .map(|s| s == "1")
                .unwrap_or(false),
            exclude_phone: config
                .get("USB_EXCLUDE_PHONE")
                .map(|s| s == "1")
                .unwrap_or(false),
            exclude_printer: config
                .get("USB_EXCLUDE_PRINTER")
                .map(|s| s == "1")
                .unwrap_or(true),
            exclude_wwan: config
                .get("USB_EXCLUDE_WWAN")
                .map(|s| s == "1")
                .unwrap_or(true),
        }
    }

    /// Get WiFi power management configuration
    pub fn get_wifi_config(&self) -> WifiConfig {
        let config = self.get_config();

        WifiConfig {
            pwr_on_ac: config
                .get("WIFI_PWR_ON_AC")
                .cloned()
                .unwrap_or_else(|| "off".to_string()),
            pwr_on_bat: config
                .get("WIFI_PWR_ON_BAT")
                .cloned()
                .unwrap_or_else(|| "on".to_string()),
        }
    }

    /// Set TLP to AC mode
    pub fn set_ac_mode(&self) {
        if self.is_installed {
            let _ = Command::new("tlp").arg("ac").output();
        }
    }

    /// Set TLP to battery mode
    pub fn set_battery_mode(&self) {
        if self.is_installed {
            let _ = Command::new("tlp").arg("bat").output();
        }
    }

    /// Start TLP
    pub fn start(&self) {
        if self.is_installed {
            let _ = Command::new("tlp").arg("start").output();
        }
    }
}

impl Default for TlpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// TLP operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TlpMode {
    #[default]
    Unknown,
    AC,
    Battery,
}

/// TLP status information
#[derive(Debug, Clone, Default)]
pub struct TlpStatus {
    pub power_save_enabled: bool,
    pub mode: TlpMode,
}

/// CPU scaling configuration
#[derive(Debug, Clone)]
pub struct CpuConfig {
    pub scaling_governor_on_ac: String,
    pub scaling_governor_on_bat: String,
    pub energy_perf_policy_on_ac: String,
    pub energy_perf_policy_on_bat: String,
    pub boost_on_ac: bool,
    pub boost_on_bat: bool,
    pub min_perf_on_ac: u32,
    pub max_perf_on_ac: u32,
    pub min_perf_on_bat: u32,
    pub max_perf_on_bat: u32,
}

/// Disk power management configuration
#[derive(Debug, Clone)]
pub struct DiskConfig {
    pub devices: String,
    pub apm_level_on_ac: String,
    pub apm_level_on_bat: String,
    pub spindown_timeout_on_ac: u32,
    pub spindown_timeout_on_bat: u32,
    pub iosched: String,
}

/// USB autosuspend configuration
#[derive(Debug, Clone)]
pub struct UsbConfig {
    pub autosuspend: bool,
    pub autosuspend_disable_on_shutdown: bool,
    pub denylist: String,
    pub allowlist: String,
    pub exclude_audio: bool,
    pub exclude_btusb: bool,
    pub exclude_phone: bool,
    pub exclude_printer: bool,
    pub exclude_wwan: bool,
}

/// WiFi power management configuration
#[derive(Debug, Clone)]
pub struct WifiConfig {
    pub pwr_on_ac: String,
    pub pwr_on_bat: String,
}
