// Winux Mobile Studio - Android Device Manager
// Copyright (c) 2026 Winux OS Project
//
// Android device management via ADB:
// - List connected devices
// - Install/uninstall apps
// - View logs (logcat)
// - Take screenshots
// - Screen recording
// - File transfer

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use super::{Device, DeviceType, DeviceStatus};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AndroidDeviceInfo {
    pub serial: String,
    pub model: String,
    pub manufacturer: String,
    pub android_version: String,
    pub sdk_version: u32,
    pub build_id: String,
    pub abi: String,
    pub screen_size: Option<String>,
    pub battery_level: Option<u32>,
}

pub struct AndroidDeviceManager {
    adb_path: PathBuf,
}

impl AndroidDeviceManager {
    pub fn new() -> Result<Self> {
        let adb_path = Self::find_adb()?;
        Ok(Self { adb_path })
    }

    fn find_adb() -> Result<PathBuf> {
        // Check ANDROID_SDK_ROOT
        if let Ok(sdk) = std::env::var("ANDROID_SDK_ROOT") {
            let adb = PathBuf::from(&sdk).join("platform-tools").join("adb");
            if adb.exists() {
                return Ok(adb);
            }
        }

        // Check ANDROID_HOME
        if let Ok(sdk) = std::env::var("ANDROID_HOME") {
            let adb = PathBuf::from(&sdk).join("platform-tools").join("adb");
            if adb.exists() {
                return Ok(adb);
            }
        }

        // Check common locations
        let home = dirs::home_dir().context("Could not find home directory")?;
        let common_paths = vec![
            home.join("Android/Sdk/platform-tools/adb"),
            PathBuf::from("/usr/bin/adb"),
            PathBuf::from("/usr/local/bin/adb"),
        ];

        for path in common_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!("ADB not found. Please install Android SDK."))
    }

    /// Start the ADB server
    pub async fn start_server(&self) -> Result<()> {
        Command::new(&self.adb_path)
            .arg("start-server")
            .output()
            .await?;
        Ok(())
    }

    /// Stop the ADB server
    pub async fn stop_server(&self) -> Result<()> {
        Command::new(&self.adb_path)
            .arg("kill-server")
            .output()
            .await?;
        Ok(())
    }

    /// List all connected Android devices
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        let output = Command::new(&self.adb_path)
            .arg("devices")
            .arg("-l")
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for line in stdout.lines().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let serial = parts[0].to_string();
                let status_str = parts[1];

                let status = match status_str {
                    "device" => DeviceStatus::Connected,
                    "offline" => DeviceStatus::Offline,
                    "unauthorized" => DeviceStatus::Unauthorized,
                    "bootloader" => DeviceStatus::Booting,
                    _ => DeviceStatus::Disconnected,
                };

                // Determine if emulator
                let is_emulator = serial.starts_with("emulator-");
                let device_type = if is_emulator {
                    DeviceType::AndroidEmulator
                } else {
                    DeviceType::AndroidPhysical
                };

                // Extract model from the line
                let model = parts.iter()
                    .find(|p| p.starts_with("model:"))
                    .map(|p| p.trim_start_matches("model:").to_string());

                devices.push(Device {
                    id: serial,
                    name: model.clone().unwrap_or_else(|| "Android Device".to_string()),
                    device_type,
                    status,
                    os_version: None,
                    model,
                });
            }
        }

        Ok(devices)
    }

    /// Get detailed information about a specific device
    pub async fn get_device_info(&self, serial: &str) -> Result<AndroidDeviceInfo> {
        let props = self.get_device_props(serial).await?;

        Ok(AndroidDeviceInfo {
            serial: serial.to_string(),
            model: props.get("ro.product.model").cloned().unwrap_or_default(),
            manufacturer: props.get("ro.product.manufacturer").cloned().unwrap_or_default(),
            android_version: props.get("ro.build.version.release").cloned().unwrap_or_default(),
            sdk_version: props.get("ro.build.version.sdk")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            build_id: props.get("ro.build.id").cloned().unwrap_or_default(),
            abi: props.get("ro.product.cpu.abi").cloned().unwrap_or_default(),
            screen_size: None,
            battery_level: self.get_battery_level(serial).await.ok(),
        })
    }

    async fn get_device_props(&self, serial: &str) -> Result<std::collections::HashMap<String, String>> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("shell")
            .arg("getprop")
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut props = std::collections::HashMap::new();

        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once("]: [") {
                let key = key.trim_start_matches('[');
                let value = value.trim_end_matches(']');
                props.insert(key.to_string(), value.to_string());
            }
        }

        Ok(props)
    }

    async fn get_battery_level(&self, serial: &str) -> Result<u32> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("shell")
            .arg("dumpsys")
            .arg("battery")
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.trim().starts_with("level:") {
                if let Some(level) = line.split(':').nth(1) {
                    return Ok(level.trim().parse()?);
                }
            }
        }

        Err(anyhow::anyhow!("Could not get battery level"))
    }

    /// Install an APK on the device
    pub async fn install_apk(&self, serial: &str, apk_path: &Path, replace: bool) -> Result<()> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.arg("-s")
           .arg(serial)
           .arg("install");

        if replace {
            cmd.arg("-r");
        }

        cmd.arg(apk_path);

        let output = cmd.output().await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install APK: {}", error));
        }

        Ok(())
    }

    /// Uninstall an app from the device
    pub async fn uninstall_app(&self, serial: &str, package_name: &str) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("uninstall")
            .arg(package_name)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to uninstall app: {}", error));
        }

        Ok(())
    }

    /// Take a screenshot
    pub async fn screenshot(&self, serial: &str, output_path: &Path) -> Result<()> {
        let temp_path = "/sdcard/screenshot.png";

        // Take screenshot on device
        Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("shell")
            .arg("screencap")
            .arg("-p")
            .arg(temp_path)
            .output()
            .await?;

        // Pull to local
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("pull")
            .arg(temp_path)
            .arg(output_path)
            .output()
            .await?;

        // Clean up on device
        Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("shell")
            .arg("rm")
            .arg(temp_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to pull screenshot"));
        }

        Ok(())
    }

    /// Start screen recording
    pub async fn start_screen_recording(
        &self,
        serial: &str,
        output_path: &str,
        max_duration_secs: Option<u32>,
    ) -> Result<tokio::process::Child> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.arg("-s")
           .arg(serial)
           .arg("shell")
           .arg("screenrecord");

        if let Some(duration) = max_duration_secs {
            cmd.arg("--time-limit").arg(duration.to_string());
        }

        cmd.arg(output_path)
           .stdout(Stdio::null())
           .stderr(Stdio::null());

        Ok(cmd.spawn()?)
    }

    /// Get logcat output
    pub async fn logcat(
        &self,
        serial: &str,
        filter: Option<&str>,
        limit: Option<u32>,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.arg("-s")
           .arg(serial)
           .arg("logcat")
           .arg("-d"); // Dump and exit

        if let Some(l) = limit {
            cmd.arg("-t").arg(l.to_string());
        }

        if let Some(f) = filter {
            cmd.arg(f);
        }

        let output = cmd.output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Clear logcat buffer
    pub async fn clear_logcat(&self, serial: &str) -> Result<()> {
        Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("logcat")
            .arg("-c")
            .output()
            .await?;
        Ok(())
    }

    /// Push a file to the device
    pub async fn push_file(&self, serial: &str, local_path: &Path, remote_path: &str) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("push")
            .arg(local_path)
            .arg(remote_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to push file: {}", error));
        }

        Ok(())
    }

    /// Pull a file from the device
    pub async fn pull_file(&self, serial: &str, remote_path: &str, local_path: &Path) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("pull")
            .arg(remote_path)
            .arg(local_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to pull file: {}", error));
        }

        Ok(())
    }

    /// Execute a shell command on the device
    pub async fn shell(&self, serial: &str, command: &str) -> Result<String> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("shell")
            .arg(command)
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Reboot the device
    pub async fn reboot(&self, serial: &str, mode: RebootMode) -> Result<()> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.arg("-s")
           .arg(serial)
           .arg("reboot");

        match mode {
            RebootMode::Normal => {}
            RebootMode::Bootloader => { cmd.arg("bootloader"); }
            RebootMode::Recovery => { cmd.arg("recovery"); }
        }

        cmd.output().await?;
        Ok(())
    }

    /// List installed packages
    pub async fn list_packages(&self, serial: &str, filter: PackageFilter) -> Result<Vec<String>> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.arg("-s")
           .arg(serial)
           .arg("shell")
           .arg("pm")
           .arg("list")
           .arg("packages");

        match filter {
            PackageFilter::All => {}
            PackageFilter::System => { cmd.arg("-s"); }
            PackageFilter::ThirdParty => { cmd.arg("-3"); }
        }

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let packages: Vec<String> = stdout
            .lines()
            .filter_map(|line| line.strip_prefix("package:"))
            .map(|s| s.to_string())
            .collect();

        Ok(packages)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RebootMode {
    Normal,
    Bootloader,
    Recovery,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PackageFilter {
    All,
    System,
    ThirdParty,
}

impl Default for AndroidDeviceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create AndroidDeviceManager")
    }
}
