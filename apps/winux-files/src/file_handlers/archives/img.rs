//! Raw disk image (.img) file handler
//!
//! Provides functionality to:
//! - Get information about disk images
//! - Mount disk images
//! - Extract contents

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header, format_size,
};
use std::path::Path;

/// Common filesystem signatures
const FAT_BOOT_SIG: [u8; 2] = [0x55, 0xAA];
const EXT_MAGIC: [u8; 2] = [0x53, 0xEF]; // At offset 0x438
const NTFS_MAGIC: &[u8] = b"NTFS    ";

/// Partition table types
#[derive(Debug, Clone)]
pub enum PartitionType {
    MBR,
    GPT,
    None,
    Unknown,
}

/// Filesystem types
#[derive(Debug, Clone)]
pub enum FilesystemType {
    Fat12,
    Fat16,
    Fat32,
    Ntfs,
    Ext2,
    Ext3,
    Ext4,
    Iso9660,
    Unknown,
}

impl FilesystemType {
    fn to_string(&self) -> &'static str {
        match self {
            FilesystemType::Fat12 => "FAT12",
            FilesystemType::Fat16 => "FAT16",
            FilesystemType::Fat32 => "FAT32",
            FilesystemType::Ntfs => "NTFS",
            FilesystemType::Ext2 => "ext2",
            FilesystemType::Ext3 => "ext3",
            FilesystemType::Ext4 => "ext4",
            FilesystemType::Iso9660 => "ISO 9660",
            FilesystemType::Unknown => "Unknown",
        }
    }
}

/// Detect filesystem type from image
pub fn detect_filesystem(path: &Path) -> FileHandlerResult<FilesystemType> {
    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    let mut file = File::open(path)?;

    // Read first sector
    let mut boot_sector = [0u8; 512];
    file.read_exact(&mut boot_sector)?;

    // Check for boot signature
    if boot_sector[510..512] == FAT_BOOT_SIG {
        // Check for NTFS
        if &boot_sector[3..11] == NTFS_MAGIC {
            return Ok(FilesystemType::Ntfs);
        }

        // Check for FAT
        let fs_type = &boot_sector[54..62];
        if fs_type.starts_with(b"FAT12") {
            return Ok(FilesystemType::Fat12);
        } else if fs_type.starts_with(b"FAT16") {
            return Ok(FilesystemType::Fat16);
        }

        let fs_type32 = &boot_sector[82..90];
        if fs_type32.starts_with(b"FAT32") {
            return Ok(FilesystemType::Fat32);
        }
    }

    // Check for ext2/3/4 (superblock at offset 1024)
    file.seek(SeekFrom::Start(0x438))?;
    let mut ext_magic = [0u8; 2];
    if file.read_exact(&mut ext_magic).is_ok() && ext_magic == EXT_MAGIC {
        // Read more of superblock to determine ext version
        file.seek(SeekFrom::Start(0x45C))?; // Feature compat flags
        let mut features = [0u8; 4];
        if file.read_exact(&mut features).is_ok() {
            let feature_compat = u32::from_le_bytes(features);
            file.seek(SeekFrom::Start(0x460))?; // Feature incompat flags
            if file.read_exact(&mut features).is_ok() {
                let feature_incompat = u32::from_le_bytes(features);

                // EXT4_FEATURE_INCOMPAT_EXTENTS = 0x40
                // EXT3_FEATURE_COMPAT_HAS_JOURNAL = 0x4
                if feature_incompat & 0x40 != 0 {
                    return Ok(FilesystemType::Ext4);
                } else if feature_compat & 0x4 != 0 {
                    return Ok(FilesystemType::Ext3);
                } else {
                    return Ok(FilesystemType::Ext2);
                }
            }
        }
        return Ok(FilesystemType::Ext2);
    }

    // Check for ISO 9660
    file.seek(SeekFrom::Start(0x8001))?;
    let mut iso_magic = [0u8; 5];
    if file.read_exact(&mut iso_magic).is_ok() && &iso_magic == b"CD001" {
        return Ok(FilesystemType::Iso9660);
    }

    Ok(FilesystemType::Unknown)
}

/// Detect partition table type
pub fn detect_partition_table(path: &Path) -> FileHandlerResult<PartitionType> {
    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    let mut file = File::open(path)?;
    let mut buffer = [0u8; 512];
    file.read_exact(&mut buffer)?;

    // Check for GPT (signature "EFI PART" at sector 1)
    file.seek(SeekFrom::Start(512))?;
    let mut gpt_sig = [0u8; 8];
    if file.read_exact(&mut gpt_sig).is_ok() && &gpt_sig == b"EFI PART" {
        return Ok(PartitionType::GPT);
    }

    // Check for MBR (signature 0x55AA at end of first sector)
    if buffer[510] == 0x55 && buffer[511] == 0xAA {
        // Check if any partition entries exist
        let has_partitions = buffer[446..510]
            .chunks(16)
            .any(|entry| entry[4] != 0); // Partition type byte

        if has_partitions {
            return Ok(PartitionType::MBR);
        }
    }

    Ok(PartitionType::None)
}

/// Get information about a disk image
pub fn get_img_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Disk Image");

    let path_str = path.to_str().unwrap_or("");

    // Detect partition table
    if let Ok(pt_type) = detect_partition_table(path) {
        let pt_str = match pt_type {
            PartitionType::MBR => "MBR",
            PartitionType::GPT => "GPT",
            PartitionType::None => "None (raw filesystem)",
            PartitionType::Unknown => "Unknown",
        };
        info.add_property("Partition Table", pt_str);
    }

    // Detect filesystem
    if let Ok(fs_type) = detect_filesystem(path) {
        info.add_property("Filesystem", fs_type.to_string());
    }

    // Use file command for additional info
    if command_exists("file") {
        if let Ok(output) = run_command("file", &[path_str]) {
            if let Some(desc) = output.split(':').nth(1) {
                info.add_property("File Type", desc.trim());
            }
        }
    }

    // Try fdisk for partition info
    if command_exists("fdisk") {
        if let Ok(output) = run_command("fdisk", &["-l", path_str]) {
            // Count partitions
            let partitions = output.lines()
                .filter(|l| l.starts_with(path_str) || l.contains("Device"))
                .count();
            if partitions > 1 {
                info.add_property("Partitions", &(partitions - 1).to_string());
            }

            // Look for disk size
            for line in output.lines() {
                if line.contains("Disk") && line.contains("bytes") {
                    if let Some(size_part) = line.split(',').next() {
                        info.add_property("Disk Info", size_part.trim());
                    }
                    break;
                }
            }
        }
    }

    // Try blkid for filesystem details
    if command_exists("blkid") {
        if let Ok(output) = run_command("blkid", &[path_str]) {
            for part in output.split_whitespace() {
                if part.starts_with("LABEL=") {
                    info.add_property("Label", part.strip_prefix("LABEL=").unwrap_or("").trim_matches('"'));
                } else if part.starts_with("UUID=") {
                    info.add_property("UUID", part.strip_prefix("UUID=").unwrap_or("").trim_matches('"'));
                }
            }
        }
    }

    Ok(info)
}

/// Mount a disk image
pub fn mount_img(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("img");
    let mount_point = format!("/tmp/img_mount_{}_{}", stem, std::process::id());

    std::fs::create_dir_all(&mount_point)?;

    // Try udisksctl (preferred, handles permissions well)
    if command_exists("udisksctl") {
        // Setup loop device
        if let Ok(output) = run_command("udisksctl", &["loop-setup", "-f", path_str]) {
            // Parse loop device from output
            for line in output.lines() {
                if line.contains("/dev/loop") {
                    if let Some(dev) = line.split_whitespace()
                        .find(|s| s.starts_with("/dev/loop")) {
                        let dev = dev.trim_end_matches('.');

                        // Mount the loop device
                        if let Ok(mount_output) = run_command("udisksctl", &["mount", "-b", dev]) {
                            return Ok(format!("Mounted:\n{}\n{}", output, mount_output));
                        }
                    }
                }
            }
            return Ok(format!("Loop device created:\n{}", output));
        }
    }

    // Try mount with loop option
    if command_exists("mount") {
        // Detect filesystem to use correct options
        let fs_type = detect_filesystem(path).unwrap_or(FilesystemType::Unknown);
        let fs_opt = match fs_type {
            FilesystemType::Fat12 | FilesystemType::Fat16 | FilesystemType::Fat32 => "vfat",
            FilesystemType::Ntfs => "ntfs-3g",
            FilesystemType::Ext2 | FilesystemType::Ext3 | FilesystemType::Ext4 => "ext4",
            FilesystemType::Iso9660 => "iso9660",
            FilesystemType::Unknown => "auto",
        };

        let output = run_command("pkexec", &[
            "mount", "-o", "loop,ro",
            "-t", fs_opt,
            path_str, &mount_point
        ])?;
        return Ok(format!("Mounted at {}\n{}", mount_point, output));
    }

    Err(FileHandlerError::NotSupported(
        "Image mounting requires udisksctl or mount".to_string()
    ))
}

/// Extract contents from a disk image
pub fn extract_img(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_extracted", stem));

    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_str().unwrap_or(".");

    // Try 7z (works with many image formats)
    if command_exists("7z") {
        let output = run_command("7z", &[
            "x", "-y",
            &format!("-o{}", output_dir_str),
            path_str
        ])?;
        return Ok(format!("Extracted to {}\n{}", output_dir.display(), output));
    }

    // Try mounting and copying
    let mount_point = format!("/tmp/img_extract_{}", std::process::id());
    std::fs::create_dir_all(&mount_point)?;

    if command_exists("mount") {
        if run_command("pkexec", &[
            "mount", "-o", "loop,ro",
            path_str, &mount_point
        ]).is_ok() {
            // Copy contents
            let copy_result = if command_exists("cp") {
                run_command("cp", &["-r", &format!("{}/.", mount_point), output_dir_str])
            } else {
                Err(FileHandlerError::NotSupported("cp not found".to_string()))
            };

            // Unmount
            let _ = run_command("pkexec", &["umount", &mount_point]);
            let _ = std::fs::remove_dir(&mount_point);

            if copy_result.is_ok() {
                return Ok(format!("Extracted to {}", output_dir.display()));
            }
        }
    }

    Err(FileHandlerError::NotSupported(
        "Image extraction requires 7z or mount+cp".to_string()
    ))
}

/// Convert image format (e.g., raw to qcow2)
pub fn convert_image(input: &Path, output: &Path, format: &str) -> FileHandlerResult<String> {
    let input_str = input.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid input path".to_string()))?;
    let output_str = output.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid output path".to_string()))?;

    if command_exists("qemu-img") {
        let result = run_command("qemu-img", &[
            "convert", "-f", "raw",
            "-O", format,
            input_str, output_str
        ])?;
        return Ok(format!("Converted to {}\n{}", output_str, result));
    }

    Err(FileHandlerError::NotSupported(
        "Image conversion requires qemu-img".to_string()
    ))
}

/// Get image size info
pub fn get_image_size(path: &Path) -> FileHandlerResult<(u64, u64)> {
    // Returns (virtual_size, actual_size)
    let metadata = std::fs::metadata(path)?;
    let actual_size = metadata.len();

    // For raw images, virtual == actual
    // For sparse images, we'd need to read the format header
    Ok((actual_size, actual_size))
}
