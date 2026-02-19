//! lsblk command parser
//!
//! Parses output from lsblk to get block device information.
//! This is a reliable fallback when UDisks2 is not available.

use std::process::Command;
use serde::{Deserialize, Serialize};

/// Block device information
#[derive(Debug, Clone, Default)]
pub struct BlockDevice {
    pub name: String,
    pub device_type: String, // disk, part, loop, rom
    pub size: u64,
    pub model: String,
    pub serial: String,
    pub vendor: String,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub filesystem: Option<String>,
    pub mount_point: Option<String>,
    pub is_removable: bool,
    pub is_rotational: bool,
    pub is_readonly: bool,
    pub smart_healthy: Option<bool>,
}

/// Partition information
#[derive(Debug, Clone, Default)]
pub struct Partition {
    pub name: String,
    pub size: u64,
    pub start: u64,
    pub filesystem: Option<String>,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub mount_point: Option<String>,
    pub partition_type: Option<String>,
    pub flags: Vec<String>,
}

/// Parser for lsblk output
pub struct LsblkParser;

impl LsblkParser {
    pub fn new() -> Self {
        Self
    }

    /// Get all block devices using lsblk -J
    pub fn get_devices(&self) -> Vec<BlockDevice> {
        let output = match Command::new("lsblk")
            .args([
                "-J",           // JSON output
                "-b",           // Size in bytes
                "-o", "NAME,TYPE,SIZE,MODEL,SERIAL,VENDOR,LABEL,UUID,FSTYPE,MOUNTPOINT,RM,RO,ROTA",
            ])
            .output()
        {
            Ok(o) => o,
            Err(_) => return self.get_devices_fallback(),
        };

        if !output.status.success() {
            return self.get_devices_fallback();
        }

        self.parse_json_output(&output.stdout)
    }

    /// Fallback method using plain lsblk output
    fn get_devices_fallback(&self) -> Vec<BlockDevice> {
        let output = match Command::new("lsblk")
            .args([
                "-b",
                "-P", // Key=value pairs
                "-o", "NAME,TYPE,SIZE,MODEL,SERIAL,VENDOR,LABEL,UUID,FSTYPE,MOUNTPOINT,RM,RO,ROTA",
            ])
            .output()
        {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        self.parse_pairs_output(&String::from_utf8_lossy(&output.stdout))
    }

    /// Parse JSON output from lsblk -J
    fn parse_json_output(&self, data: &[u8]) -> Vec<BlockDevice> {
        #[derive(Deserialize)]
        struct LsblkOutput {
            blockdevices: Vec<LsblkDevice>,
        }

        #[derive(Deserialize)]
        struct LsblkDevice {
            name: String,
            #[serde(rename = "type")]
            device_type: Option<String>,
            size: Option<u64>,
            model: Option<String>,
            serial: Option<String>,
            vendor: Option<String>,
            label: Option<String>,
            uuid: Option<String>,
            fstype: Option<String>,
            mountpoint: Option<String>,
            rm: Option<bool>,
            ro: Option<bool>,
            rota: Option<bool>,
            children: Option<Vec<LsblkDevice>>,
        }

        let output: LsblkOutput = match serde_json::from_slice(data) {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        fn convert_device(dev: &LsblkDevice) -> BlockDevice {
            BlockDevice {
                name: dev.name.clone(),
                device_type: dev.device_type.clone().unwrap_or_default(),
                size: dev.size.unwrap_or(0),
                model: dev.model.clone().unwrap_or_default().trim().to_string(),
                serial: dev.serial.clone().unwrap_or_default().trim().to_string(),
                vendor: dev.vendor.clone().unwrap_or_default().trim().to_string(),
                label: dev.label.clone(),
                uuid: dev.uuid.clone(),
                filesystem: dev.fstype.clone(),
                mount_point: dev.mountpoint.clone(),
                is_removable: dev.rm.unwrap_or(false),
                is_rotational: dev.rota.unwrap_or(true),
                is_readonly: dev.ro.unwrap_or(false),
                smart_healthy: None, // Will be filled by SMART check
            }
        }

        let mut devices = Vec::new();

        for dev in &output.blockdevices {
            devices.push(convert_device(dev));

            // Add children (partitions)
            if let Some(children) = &dev.children {
                for child in children {
                    devices.push(convert_device(child));
                }
            }
        }

        devices
    }

    /// Parse key=value pairs output from lsblk -P
    fn parse_pairs_output(&self, data: &str) -> Vec<BlockDevice> {
        let mut devices = Vec::new();

        for line in data.lines() {
            let mut device = BlockDevice::default();

            // Parse key="value" pairs
            for pair in line.split_whitespace() {
                if let Some((key, value)) = pair.split_once('=') {
                    let value = value.trim_matches('"');
                    match key {
                        "NAME" => device.name = value.to_string(),
                        "TYPE" => device.device_type = value.to_string(),
                        "SIZE" => device.size = value.parse().unwrap_or(0),
                        "MODEL" => device.model = value.to_string(),
                        "SERIAL" => device.serial = value.to_string(),
                        "VENDOR" => device.vendor = value.to_string(),
                        "LABEL" => {
                            if !value.is_empty() {
                                device.label = Some(value.to_string());
                            }
                        }
                        "UUID" => {
                            if !value.is_empty() {
                                device.uuid = Some(value.to_string());
                            }
                        }
                        "FSTYPE" => {
                            if !value.is_empty() {
                                device.filesystem = Some(value.to_string());
                            }
                        }
                        "MOUNTPOINT" => {
                            if !value.is_empty() {
                                device.mount_point = Some(value.to_string());
                            }
                        }
                        "RM" => device.is_removable = value == "1",
                        "RO" => device.is_readonly = value == "1",
                        "ROTA" => device.is_rotational = value == "1",
                        _ => {}
                    }
                }
            }

            if !device.name.is_empty() {
                devices.push(device);
            }
        }

        devices
    }

    /// Get a specific device by name
    pub fn get_device(&self, name: &str) -> Option<BlockDevice> {
        self.get_devices().into_iter().find(|d| d.name == name)
    }

    /// Get partitions for a specific disk
    pub fn get_partitions(&self, disk_name: &str) -> Vec<Partition> {
        let output = match Command::new("lsblk")
            .args([
                "-J",
                "-b",
                "-o", "NAME,SIZE,START,FSTYPE,LABEL,UUID,MOUNTPOINT,PARTTYPE,PARTFLAGS",
                &format!("/dev/{}", disk_name),
            ])
            .output()
        {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        #[derive(Deserialize)]
        struct LsblkOutput {
            blockdevices: Vec<LsblkPartDevice>,
        }

        #[derive(Deserialize)]
        struct LsblkPartDevice {
            name: String,
            size: Option<u64>,
            start: Option<u64>,
            fstype: Option<String>,
            label: Option<String>,
            uuid: Option<String>,
            mountpoint: Option<String>,
            parttype: Option<String>,
            partflags: Option<String>,
            children: Option<Vec<LsblkPartDevice>>,
        }

        let output: LsblkOutput = match serde_json::from_slice(&output.stdout) {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        let mut partitions = Vec::new();

        for dev in &output.blockdevices {
            if let Some(children) = &dev.children {
                for child in children {
                    partitions.push(Partition {
                        name: child.name.clone(),
                        size: child.size.unwrap_or(0),
                        start: child.start.unwrap_or(0),
                        filesystem: child.fstype.clone(),
                        label: child.label.clone(),
                        uuid: child.uuid.clone(),
                        mount_point: child.mountpoint.clone(),
                        partition_type: child.parttype.clone(),
                        flags: child.partflags.as_ref()
                            .map(|f| f.split(',').map(|s| s.to_string()).collect())
                            .unwrap_or_default(),
                    });
                }
            }
        }

        partitions
    }

    /// Get disk usage statistics
    pub fn get_disk_usage(&self, mount_point: &str) -> Option<DiskUsage> {
        let output = Command::new("df")
            .args(["-B1", mount_point])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().nth(1)?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 4 {
            Some(DiskUsage {
                total: parts[1].parse().unwrap_or(0),
                used: parts[2].parse().unwrap_or(0),
                available: parts[3].parse().unwrap_or(0),
            })
        } else {
            None
        }
    }
}

impl Default for LsblkParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Disk usage information
#[derive(Debug, Clone)]
pub struct DiskUsage {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

impl DiskUsage {
    /// Get usage as a percentage (0.0 - 1.0)
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.used as f64 / self.total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pairs() {
        let parser = LsblkParser::new();
        let data = r#"NAME="sda" TYPE="disk" SIZE="500107862016" MODEL="Samsung SSD 870" SERIAL="S5XXXX" VENDOR="" LABEL="" UUID="" FSTYPE="" MOUNTPOINT="" RM="0" RO="0" ROTA="0"
NAME="sda1" TYPE="part" SIZE="536870912" MODEL="" SERIAL="" VENDOR="" LABEL="EFI" UUID="1234-5678" FSTYPE="vfat" MOUNTPOINT="/boot/efi" RM="0" RO="0" ROTA="0""#;

        let devices = parser.parse_pairs_output(data);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].name, "sda");
        assert_eq!(devices[0].device_type, "disk");
        assert!(!devices[0].is_rotational);
        assert_eq!(devices[1].name, "sda1");
        assert_eq!(devices[1].filesystem, Some("vfat".to_string()));
    }
}
