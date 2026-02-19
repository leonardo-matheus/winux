// Winux Mobile Studio - iOS Device Manager
// Copyright (c) 2026 Winux OS Project
//
// iOS device management via libimobiledevice:
// - List connected iOS devices
// - Install/uninstall apps
// - View system logs
// - Get device information
// - File transfer

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use super::{Device, DeviceType, DeviceStatus};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IOSDeviceInfo {
    pub udid: String,
    pub name: String,
    pub model: String,
    pub product_type: String,
    pub ios_version: String,
    pub build_version: String,
    pub serial_number: String,
    pub wifi_address: Option<String>,
    pub bluetooth_address: Option<String>,
    pub battery_level: Option<u32>,
    pub is_jailbroken: bool,
}

pub struct IOSDeviceManager {
    idevice_id_path: Option<PathBuf>,
    ideviceinfo_path: Option<PathBuf>,
    ideviceinstaller_path: Option<PathBuf>,
    idevicesyslog_path: Option<PathBuf>,
}

impl IOSDeviceManager {
    pub fn new() -> Self {
        Self {
            idevice_id_path: Self::find_binary("idevice_id"),
            ideviceinfo_path: Self::find_binary("ideviceinfo"),
            ideviceinstaller_path: Self::find_binary("ideviceinstaller"),
            idevicesyslog_path: Self::find_binary("idevicesyslog"),
        }
    }

    fn find_binary(name: &str) -> Option<PathBuf> {
        let common_paths = vec![
            PathBuf::from(format!("/usr/bin/{}", name)),
            PathBuf::from(format!("/usr/local/bin/{}", name)),
            PathBuf::from(format!("/opt/bin/{}", name)),
        ];

        for path in common_paths {
            if path.exists() {
                return Some(path);
            }
        }

        // Try using which
        if let Ok(output) = std::process::Command::new("which")
            .arg(name)
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }

        None
    }

    /// Check if libimobiledevice tools are available
    pub fn is_available(&self) -> bool {
        self.idevice_id_path.is_some()
    }

    /// Get status of available tools
    pub fn get_tools_status(&self) -> IOSToolsStatus {
        IOSToolsStatus {
            idevice_id: self.idevice_id_path.is_some(),
            ideviceinfo: self.ideviceinfo_path.is_some(),
            ideviceinstaller: self.ideviceinstaller_path.is_some(),
            idevicesyslog: self.idevicesyslog_path.is_some(),
        }
    }

    /// List all connected iOS devices
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        let idevice_id = self.idevice_id_path.as_ref()
            .context("idevice_id not found. Please install libimobiledevice.")?;

        let output = Command::new(idevice_id)
            .arg("-l")
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for line in stdout.lines() {
            let udid = line.trim();
            if udid.is_empty() {
                continue;
            }

            // Get device name
            let name = self.get_device_name(udid).await
                .unwrap_or_else(|_| "iOS Device".to_string());

            devices.push(Device {
                id: udid.to_string(),
                name,
                device_type: DeviceType::IOSPhysical,
                status: DeviceStatus::Connected,
                os_version: None,
                model: None,
            });
        }

        Ok(devices)
    }

    async fn get_device_name(&self, udid: &str) -> Result<String> {
        let info = self.get_device_info(udid).await?;
        Ok(info.name)
    }

    /// Get detailed information about a specific device
    pub async fn get_device_info(&self, udid: &str) -> Result<IOSDeviceInfo> {
        let ideviceinfo = self.ideviceinfo_path.as_ref()
            .context("ideviceinfo not found")?;

        let output = Command::new(ideviceinfo)
            .arg("-u")
            .arg(udid)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut info = std::collections::HashMap::new();

        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                info.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(IOSDeviceInfo {
            udid: udid.to_string(),
            name: info.get("DeviceName").cloned().unwrap_or_default(),
            model: info.get("ModelNumber").cloned().unwrap_or_default(),
            product_type: info.get("ProductType").cloned().unwrap_or_default(),
            ios_version: info.get("ProductVersion").cloned().unwrap_or_default(),
            build_version: info.get("BuildVersion").cloned().unwrap_or_default(),
            serial_number: info.get("SerialNumber").cloned().unwrap_or_default(),
            wifi_address: info.get("WiFiAddress").cloned(),
            bluetooth_address: info.get("BluetoothAddress").cloned(),
            battery_level: info.get("BatteryCurrentCapacity")
                .and_then(|s| s.parse().ok()),
            is_jailbroken: self.check_jailbreak(udid).await.unwrap_or(false),
        })
    }

    async fn check_jailbreak(&self, udid: &str) -> Result<bool> {
        // Check for Cydia by trying to list it
        if let Some(installer) = &self.ideviceinstaller_path {
            let output = Command::new(installer)
                .arg("-u")
                .arg(udid)
                .arg("-l")
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(stdout.contains("com.saurik.Cydia") || stdout.contains("org.coolstar.sileo"));
        }

        Ok(false)
    }

    /// Install an IPA on the device
    pub async fn install_ipa(&self, udid: &str, ipa_path: &Path) -> Result<()> {
        let installer = self.ideviceinstaller_path.as_ref()
            .context("ideviceinstaller not found")?;

        let output = Command::new(installer)
            .arg("-u")
            .arg(udid)
            .arg("-i")
            .arg(ipa_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install IPA: {}", error));
        }

        Ok(())
    }

    /// Uninstall an app from the device
    pub async fn uninstall_app(&self, udid: &str, bundle_id: &str) -> Result<()> {
        let installer = self.ideviceinstaller_path.as_ref()
            .context("ideviceinstaller not found")?;

        let output = Command::new(installer)
            .arg("-u")
            .arg(udid)
            .arg("-U")
            .arg(bundle_id)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall app: {}", error));
        }

        Ok(())
    }

    /// List installed apps
    pub async fn list_apps(&self, udid: &str, app_type: IOSAppType) -> Result<Vec<IOSAppInfo>> {
        let installer = self.ideviceinstaller_path.as_ref()
            .context("ideviceinstaller not found")?;

        let mut cmd = Command::new(installer);
        cmd.arg("-u")
           .arg(udid)
           .arg("-l");

        match app_type {
            IOSAppType::All => {}
            IOSAppType::User => { cmd.arg("-o").arg("list_user"); }
            IOSAppType::System => { cmd.arg("-o").arg("list_system"); }
        }

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let apps: Vec<IOSAppInfo> = stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(" - ").collect();
                if parts.len() >= 2 {
                    Some(IOSAppInfo {
                        bundle_id: parts[0].trim().to_string(),
                        name: parts.get(1).unwrap_or(&"").trim().to_string(),
                        version: parts.get(2).map(|s| s.trim().to_string()),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(apps)
    }

    /// Get system logs (syslog)
    pub async fn syslog(&self, udid: &str, filter: Option<&str>) -> Result<tokio::process::Child> {
        let syslog = self.idevicesyslog_path.as_ref()
            .context("idevicesyslog not found")?;

        let mut cmd = Command::new(syslog);
        cmd.arg("-u")
           .arg(udid);

        if let Some(f) = filter {
            cmd.arg("-m").arg(f);
        }

        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        Ok(cmd.spawn()?)
    }

    /// Take a screenshot
    pub async fn screenshot(&self, udid: &str, output_path: &Path) -> Result<()> {
        let idevicescreenshot = Self::find_binary("idevicescreenshot")
            .context("idevicescreenshot not found")?;

        let output = Command::new(idevicescreenshot)
            .arg("-u")
            .arg(udid)
            .arg(output_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to take screenshot: {}", error));
        }

        Ok(())
    }

    /// Reboot the device
    pub async fn reboot(&self, udid: &str) -> Result<()> {
        let idevicediagnostics = Self::find_binary("idevicediagnostics")
            .context("idevicediagnostics not found")?;

        let output = Command::new(idevicediagnostics)
            .arg("-u")
            .arg(udid)
            .arg("restart")
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to reboot device: {}", error));
        }

        Ok(())
    }

    /// Pair with the device
    pub async fn pair(&self, udid: &str) -> Result<()> {
        let idevicepair = Self::find_binary("idevicepair")
            .context("idevicepair not found")?;

        let output = Command::new(idevicepair)
            .arg("-u")
            .arg(udid)
            .arg("pair")
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to pair device: {}", error));
        }

        Ok(())
    }

    /// Validate pairing
    pub async fn validate_pair(&self, udid: &str) -> Result<bool> {
        let idevicepair = Self::find_binary("idevicepair")
            .context("idevicepair not found")?;

        let output = Command::new(idevicepair)
            .arg("-u")
            .arg(udid)
            .arg("validate")
            .output()
            .await?;

        Ok(output.status.success())
    }

    /// Get device crash logs
    pub async fn get_crash_logs(&self, udid: &str, output_dir: &Path) -> Result<()> {
        let idevicecrashreport = Self::find_binary("idevicecrashreport")
            .context("idevicecrashreport not found")?;

        std::fs::create_dir_all(output_dir)?;

        let output = Command::new(idevicecrashreport)
            .arg("-u")
            .arg(udid)
            .arg("-e") // Extract
            .arg(output_dir)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to get crash logs: {}", error));
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct IOSToolsStatus {
    pub idevice_id: bool,
    pub ideviceinfo: bool,
    pub ideviceinstaller: bool,
    pub idevicesyslog: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IOSAppInfo {
    pub bundle_id: String,
    pub name: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IOSAppType {
    All,
    User,
    System,
}

impl Default for IOSDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
