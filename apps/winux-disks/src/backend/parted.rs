//! Parted operations for partition management
//!
//! Provides partition creation, deletion, and resizing using parted.

use std::process::Command;
use thiserror::Error;

/// Parted manager for partition operations
pub struct PartedManager {
    disk: String,
}

impl PartedManager {
    /// Create a new parted manager for a disk
    pub fn new(disk: &str) -> Self {
        Self {
            disk: disk.to_string(),
        }
    }

    /// Get the partition table type (gpt, msdos, etc.)
    pub fn get_partition_table(&self) -> Result<String, PartedError> {
        let output = Command::new("parted")
            .args(["-s", &format!("/dev/{}", self.disk), "print"])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Partition Table:") {
                return Ok(line.split(':').nth(1).unwrap_or("unknown").trim().to_string());
            }
        }

        Ok("unknown".to_string())
    }

    /// Create a new partition table
    pub fn create_partition_table(&self, table_type: PartitionTableType) -> Result<(), PartedError> {
        let table_str = match table_type {
            PartitionTableType::Gpt => "gpt",
            PartitionTableType::Msdos => "msdos",
            PartitionTableType::Mac => "mac",
        };

        let output = Command::new("pkexec")
            .args([
                "parted", "-s",
                &format!("/dev/{}", self.disk),
                "mklabel", table_str,
            ])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// Create a new partition
    pub fn create_partition(&self, params: CreatePartitionParams) -> Result<(), PartedError> {
        let part_type = match params.partition_type {
            PartitionType::Primary => "primary",
            PartitionType::Extended => "extended",
            PartitionType::Logical => "logical",
        };

        let mut args = vec![
            "parted".to_string(),
            "-s".to_string(),
            format!("/dev/{}", self.disk),
            "mkpart".to_string(),
            part_type.to_string(),
        ];

        // Add filesystem type if specified (for alignment hints)
        if let Some(ref fs) = params.filesystem {
            args.push(fs.clone());
        }

        // Add start and end positions
        args.push(params.start.clone());
        args.push(params.end.clone());

        let output = Command::new("pkexec")
            .args(&args)
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            // Set partition name if provided (GPT only)
            if let Some(name) = params.name {
                let _ = self.set_partition_name(params.number.unwrap_or(1), &name);
            }

            // Set flags if provided
            for flag in params.flags {
                let _ = self.set_partition_flag(params.number.unwrap_or(1), &flag, true);
            }

            Ok(())
        } else {
            Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// Delete a partition
    pub fn delete_partition(&self, partition_number: u32) -> Result<(), PartedError> {
        let output = Command::new("pkexec")
            .args([
                "parted", "-s",
                &format!("/dev/{}", self.disk),
                "rm", &partition_number.to_string(),
            ])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// Resize a partition
    pub fn resize_partition(&self, partition_number: u32, new_end: &str) -> Result<(), PartedError> {
        let output = Command::new("pkexec")
            .args([
                "parted", "-s",
                &format!("/dev/{}", self.disk),
                "resizepart", &partition_number.to_string(), new_end,
            ])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// Set partition name (GPT only)
    pub fn set_partition_name(&self, partition_number: u32, name: &str) -> Result<(), PartedError> {
        let output = Command::new("pkexec")
            .args([
                "parted", "-s",
                &format!("/dev/{}", self.disk),
                "name", &partition_number.to_string(), name,
            ])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// Set or clear a partition flag
    pub fn set_partition_flag(&self, partition_number: u32, flag: &str, value: bool) -> Result<(), PartedError> {
        let flag_value = if value { "on" } else { "off" };

        let output = Command::new("pkexec")
            .args([
                "parted", "-s",
                &format!("/dev/{}", self.disk),
                "set", &partition_number.to_string(), flag, flag_value,
            ])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// Get partition information
    pub fn get_partitions(&self) -> Result<Vec<PartedPartition>, PartedError> {
        let output = Command::new("parted")
            .args([
                "-s", "-m",
                &format!("/dev/{}", self.disk),
                "unit", "B", "print",
            ])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut partitions = Vec::new();

        for line in stdout.lines().skip(2) { // Skip header lines
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 7 {
                if let Ok(number) = parts[0].parse::<u32>() {
                    partitions.push(PartedPartition {
                        number,
                        start: Self::parse_bytes(parts[1]),
                        end: Self::parse_bytes(parts[2]),
                        size: Self::parse_bytes(parts[3]),
                        filesystem: if parts[4].is_empty() { None } else { Some(parts[4].to_string()) },
                        name: if parts.len() > 5 && !parts[5].is_empty() { Some(parts[5].to_string()) } else { None },
                        flags: if parts.len() > 6 && !parts[6].is_empty() {
                            parts[6].split(',').map(|s| s.trim().to_string()).collect()
                        } else {
                            Vec::new()
                        },
                    });
                }
            }
        }

        Ok(partitions)
    }

    /// Get free space regions on the disk
    pub fn get_free_space(&self) -> Result<Vec<FreeSpace>, PartedError> {
        let output = Command::new("parted")
            .args([
                "-s", "-m",
                &format!("/dev/{}", self.disk),
                "unit", "B", "print", "free",
            ])
            .output()
            .map_err(|e| PartedError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(PartedError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut free_regions = Vec::new();

        for line in stdout.lines().skip(2) {
            if line.contains("free") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    free_regions.push(FreeSpace {
                        start: Self::parse_bytes(parts[1]),
                        end: Self::parse_bytes(parts[2]),
                        size: Self::parse_bytes(parts[3]),
                    });
                }
            }
        }

        Ok(free_regions)
    }

    /// Parse byte string (e.g., "1048576B" -> 1048576)
    fn parse_bytes(s: &str) -> u64 {
        s.trim_end_matches('B')
            .parse()
            .unwrap_or(0)
    }

    /// Align a position to optimal boundaries
    pub fn align_to_mebibyte(bytes: u64) -> u64 {
        const MIB: u64 = 1024 * 1024;
        ((bytes + MIB - 1) / MIB) * MIB
    }
}

/// Partition table types
#[derive(Debug, Clone, Copy)]
pub enum PartitionTableType {
    Gpt,
    Msdos,
    Mac,
}

/// Partition types
#[derive(Debug, Clone, Copy)]
pub enum PartitionType {
    Primary,
    Extended,
    Logical,
}

/// Parameters for creating a partition
#[derive(Debug, Clone)]
pub struct CreatePartitionParams {
    pub partition_type: PartitionType,
    pub filesystem: Option<String>,
    pub start: String, // e.g., "1MiB", "50%"
    pub end: String,   // e.g., "100MiB", "100%"
    pub name: Option<String>,
    pub number: Option<u32>,
    pub flags: Vec<String>,
}

impl Default for CreatePartitionParams {
    fn default() -> Self {
        Self {
            partition_type: PartitionType::Primary,
            filesystem: None,
            start: "1MiB".to_string(),
            end: "100%".to_string(),
            name: None,
            number: None,
            flags: Vec::new(),
        }
    }
}

/// Parted partition information
#[derive(Debug, Clone)]
pub struct PartedPartition {
    pub number: u32,
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub filesystem: Option<String>,
    pub name: Option<String>,
    pub flags: Vec<String>,
}

/// Free space region
#[derive(Debug, Clone)]
pub struct FreeSpace {
    pub start: u64,
    pub end: u64,
    pub size: u64,
}

/// Parted errors
#[derive(Debug, Error)]
pub enum PartedError {
    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Device busy: {0}")]
    DeviceBusy(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("No free space available")]
    NoFreeSpace,
}

/// Common partition flags
pub mod flags {
    pub const BOOT: &str = "boot";
    pub const ESP: &str = "esp";
    pub const SWAP: &str = "swap";
    pub const LVM: &str = "lvm";
    pub const RAID: &str = "raid";
    pub const HIDDEN: &str = "hidden";
    pub const LEGACY_BOOT: &str = "legacy_boot";
    pub const MSFTDATA: &str = "msftdata";
    pub const MSFTRES: &str = "msftres";
    pub const DIAG: &str = "diag";
}

/// Common GPT partition type GUIDs
pub mod gpt_types {
    pub const LINUX_FILESYSTEM: &str = "0FC63DAF-8483-4772-8E79-3D69D8477DE4";
    pub const LINUX_SWAP: &str = "0657FD6D-A4AB-43C4-84E5-0933C84B4F4F";
    pub const LINUX_HOME: &str = "933AC7E1-2EB4-4F13-B844-0E14E2AEF915";
    pub const LINUX_ROOT_X86_64: &str = "4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709";
    pub const EFI_SYSTEM: &str = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B";
    pub const MICROSOFT_BASIC_DATA: &str = "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7";
    pub const MICROSOFT_RESERVED: &str = "E3C9E316-0B5C-4DB8-817D-F92DF00215AE";
}
