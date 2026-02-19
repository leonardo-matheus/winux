//! Backend modules for disk operations

mod udisks;
mod lsblk;
mod parted;

pub use udisks::{UDisksClient, UDisksDrive, UDisksBlockDevice};
pub use lsblk::{LsblkParser, BlockDevice, Partition};
pub use parted::PartedManager;

use std::process::Command;

/// SMART information for a disk
#[derive(Debug, Clone)]
pub struct SmartInfo {
    pub healthy: bool,
    pub temperature: Option<u32>,
    pub power_on_hours: Option<u64>,
    pub power_cycle_count: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub uncorrectable_sectors: Option<u64>,
}

/// Main disk manager that combines all backends
pub struct DiskManager {
    lsblk: LsblkParser,
}

impl DiskManager {
    pub fn new() -> Self {
        Self {
            lsblk: LsblkParser::new(),
        }
    }

    /// Get all block devices
    pub fn get_block_devices(&self) -> Vec<BlockDevice> {
        self.lsblk.get_devices()
    }

    /// Get a specific device by name
    pub fn get_device(&self, name: &str) -> Option<BlockDevice> {
        self.lsblk.get_device(name)
    }

    /// Get partition info as a BlockDevice
    pub fn get_partition_info(&self, name: &str) -> Option<BlockDevice> {
        let devices = self.lsblk.get_devices();
        for device in devices {
            if device.name == name {
                return Some(device);
            }
        }
        None
    }

    /// Get partitions for a disk
    pub fn get_partitions(&self, disk_name: &str) -> Vec<Partition> {
        self.lsblk.get_partitions(disk_name)
    }

    /// Get SMART information for a disk
    pub fn get_smart_info(&self, disk_name: &str) -> Option<SmartInfo> {
        // Try to get SMART info using smartctl
        let output = Command::new("smartctl")
            .args(["-A", "-H", "-j", &format!("/dev/{}", disk_name)])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;

        let healthy = json.get("smart_status")
            .and_then(|s| s.get("passed"))
            .and_then(|p| p.as_bool())
            .unwrap_or(true);

        let temperature = json.get("temperature")
            .and_then(|t| t.get("current"))
            .and_then(|c| c.as_u64())
            .map(|t| t as u32);

        // Parse SMART attributes
        let attrs = json.get("ata_smart_attributes")
            .and_then(|a| a.get("table"))
            .and_then(|t| t.as_array());

        let mut power_on_hours = None;
        let mut power_cycle_count = None;
        let mut reallocated_sectors = None;
        let mut pending_sectors = None;
        let mut uncorrectable_sectors = None;

        if let Some(attrs) = attrs {
            for attr in attrs {
                let id = attr.get("id").and_then(|i| i.as_u64()).unwrap_or(0);
                let raw_value = attr.get("raw")
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.as_u64());

                match id {
                    5 => reallocated_sectors = raw_value,
                    9 => power_on_hours = raw_value,
                    12 => power_cycle_count = raw_value,
                    197 => pending_sectors = raw_value,
                    198 => uncorrectable_sectors = raw_value,
                    _ => {}
                }
            }
        }

        Some(SmartInfo {
            healthy,
            temperature,
            power_on_hours,
            power_cycle_count,
            reallocated_sectors,
            pending_sectors,
            uncorrectable_sectors,
        })
    }

    /// Mount a partition
    pub fn mount_partition(&self, partition: &str, mount_point: Option<&str>) -> Result<String, String> {
        let args = if let Some(mp) = mount_point {
            vec![&format!("/dev/{}", partition), mp]
        } else {
            // Use udisksctl for auto-mounting
            return self.udisks_mount(partition);
        };

        let output = Command::new("pkexec")
            .arg("mount")
            .args(&args)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(mount_point.unwrap_or("").to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Mount using UDisks2
    fn udisks_mount(&self, partition: &str) -> Result<String, String> {
        let output = Command::new("udisksctl")
            .args(["mount", "-b", &format!("/dev/{}", partition)])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Extract mount point from output like "Mounted /dev/sda1 at /run/media/user/label"
            if let Some(idx) = stdout.find(" at ") {
                let mount_point = stdout[idx + 4..].trim().to_string();
                return Ok(mount_point);
            }
            Ok(stdout.to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Unmount a partition
    pub fn unmount_partition(&self, partition: &str) -> Result<(), String> {
        let output = Command::new("udisksctl")
            .args(["unmount", "-b", &format!("/dev/{}", partition)])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Format a partition
    pub fn format_partition(
        &self,
        partition: &str,
        filesystem: &str,
        label: Option<&str>,
        options: FormatOptions,
    ) -> Result<(), String> {
        // This should use pkexec for elevated privileges
        let mkfs_cmd = match filesystem {
            "ext4" => "mkfs.ext4",
            "ext3" => "mkfs.ext3",
            "ext2" => "mkfs.ext2",
            "btrfs" => "mkfs.btrfs",
            "xfs" => "mkfs.xfs",
            "ntfs" => "mkfs.ntfs",
            "vfat" | "fat32" => "mkfs.vfat",
            "exfat" => "mkfs.exfat",
            "swap" => "mkswap",
            _ => return Err(format!("Unsupported filesystem: {}", filesystem)),
        };

        let mut args = vec!["-f".to_string()]; // Force

        if let Some(l) = label {
            match filesystem {
                "ext4" | "ext3" | "ext2" => {
                    args.push("-L".to_string());
                    args.push(l.to_string());
                }
                "btrfs" => {
                    args.push("-L".to_string());
                    args.push(l.to_string());
                }
                "xfs" => {
                    args.push("-L".to_string());
                    args.push(l.to_string());
                }
                "ntfs" => {
                    args.push("-L".to_string());
                    args.push(l.to_string());
                }
                "vfat" | "fat32" => {
                    args.push("-n".to_string());
                    args.push(l.to_string());
                }
                "exfat" => {
                    args.push("-n".to_string());
                    args.push(l.to_string());
                }
                _ => {}
            }
        }

        // Add filesystem-specific options
        if filesystem == "ext4" && !options.quick {
            args.push("-c".to_string()); // Check for bad blocks
        }

        args.push(format!("/dev/{}", partition));

        let output = Command::new("pkexec")
            .arg(mkfs_cmd)
            .args(&args)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Run disk benchmark
    pub fn benchmark_disk(&self, disk: &str, size_mb: u32) -> Result<BenchmarkResult, String> {
        // Write test using dd
        let write_output = Command::new("dd")
            .args([
                &format!("if=/dev/zero"),
                &format!("of=/tmp/winux_benchmark_test"),
                "bs=1M",
                &format!("count={}", size_mb),
                "conv=fdatasync",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let write_speed = Self::parse_dd_speed(&String::from_utf8_lossy(&write_output.stderr));

        // Read test
        let read_output = Command::new("dd")
            .args([
                "if=/tmp/winux_benchmark_test",
                "of=/dev/null",
                "bs=1M",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let read_speed = Self::parse_dd_speed(&String::from_utf8_lossy(&read_output.stderr));

        // Cleanup
        let _ = Command::new("rm").arg("/tmp/winux_benchmark_test").output();

        Ok(BenchmarkResult {
            read_speed_mbps: read_speed,
            write_speed_mbps: write_speed,
            iops_read: None,
            iops_write: None,
        })
    }

    fn parse_dd_speed(output: &str) -> f64 {
        // Parse output like "... 500 MB/s" or "... 500 MB/s"
        if let Some(idx) = output.find("MB/s") {
            let before = &output[..idx];
            let parts: Vec<&str> = before.split_whitespace().collect();
            if let Some(speed) = parts.last() {
                if let Ok(s) = speed.replace(',', ".").parse::<f64>() {
                    return s;
                }
            }
        }
        0.0
    }
}

impl Default for DiskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format options
#[derive(Default)]
pub struct FormatOptions {
    pub quick: bool,
    pub check_bad_blocks: bool,
    pub overwrite_zeros: bool,
    pub encrypt: bool,
}

/// Benchmark results
pub struct BenchmarkResult {
    pub read_speed_mbps: f64,
    pub write_speed_mbps: f64,
    pub iops_read: Option<u64>,
    pub iops_write: Option<u64>,
}
