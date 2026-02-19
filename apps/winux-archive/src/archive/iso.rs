//! ISO disc image support

use super::{ArchiveEntry, CompressionMethod};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// ISO image handler
pub struct IsoArchive {
    path: std::path::PathBuf,
    entries_cache: Vec<ArchiveEntry>,
    volume_label: String,
}

impl IsoArchive {
    /// Open an existing ISO image
    pub fn open(path: &Path) -> Result<Self> {
        let mut archive = Self {
            path: path.to_path_buf(),
            entries_cache: Vec::new(),
            volume_label: String::new(),
        };

        archive.read_iso_info()?;

        Ok(archive)
    }

    /// Read ISO information and entries
    fn read_iso_info(&mut self) -> Result<()> {
        let mut file = File::open(&self.path)?;

        // Read Primary Volume Descriptor (at sector 16, 2048 bytes per sector)
        let pvd_offset = 16 * 2048;
        file.seek(SeekFrom::Start(pvd_offset))?;

        let mut pvd = [0u8; 2048];
        file.read_exact(&mut pvd)?;

        // Check for CD001 signature
        if &pvd[1..6] != b"CD001" {
            return Err(anyhow::anyhow!("Invalid ISO9660 format"));
        }

        // Read volume label (offset 40, length 32)
        self.volume_label = String::from_utf8_lossy(&pvd[40..72])
            .trim()
            .to_string();

        // Read root directory record (offset 156, length 34)
        let root_dir_record = &pvd[156..190];

        // Location of root directory (offset 2 in record, LSB)
        let root_location = u32::from_le_bytes([
            root_dir_record[2],
            root_dir_record[3],
            root_dir_record[4],
            root_dir_record[5],
        ]);

        // Size of root directory
        let root_size = u32::from_le_bytes([
            root_dir_record[10],
            root_dir_record[11],
            root_dir_record[12],
            root_dir_record[13],
        ]);

        // Read root directory entries
        self.entries_cache = self.read_directory(&mut file, root_location, root_size, "")?;

        Ok(())
    }

    /// Read directory entries recursively
    fn read_directory(
        &self,
        file: &mut File,
        location: u32,
        size: u32,
        parent_path: &str,
    ) -> Result<Vec<ArchiveEntry>> {
        let mut entries = Vec::new();
        let offset = (location as u64) * 2048;

        file.seek(SeekFrom::Start(offset))?;

        let mut buffer = vec![0u8; size as usize];
        file.read_exact(&mut buffer)?;

        let mut pos = 0;
        while pos < buffer.len() {
            let record_length = buffer[pos] as usize;
            if record_length == 0 {
                // Move to next sector
                pos = ((pos / 2048) + 1) * 2048;
                continue;
            }

            if pos + record_length > buffer.len() {
                break;
            }

            let record = &buffer[pos..pos + record_length];

            // Extended attribute length
            let _ext_attr_len = record[1];

            // Location of extent (LSB)
            let extent_location = u32::from_le_bytes([
                record[2], record[3], record[4], record[5],
            ]);

            // Data length (LSB)
            let data_length = u32::from_le_bytes([
                record[10], record[11], record[12], record[13],
            ]);

            // File flags
            let flags = record[25];
            let is_directory = (flags & 0x02) != 0;

            // File identifier length
            let name_length = record[32] as usize;

            // File identifier
            let name_bytes = &record[33..33 + name_length];
            let mut name = String::from_utf8_lossy(name_bytes).to_string();

            // Skip . and .. entries
            if name == "\x00" || name == "\x01" {
                pos += record_length;
                continue;
            }

            // Remove version number (;1)
            if let Some(idx) = name.find(';') {
                name.truncate(idx);
            }

            // Build full path
            let full_path = if parent_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", parent_path, name)
            };

            // Get timestamp (recording date/time at offset 18)
            let timestamp = self.parse_iso_timestamp(&record[18..25]);

            let entry = ArchiveEntry {
                path: full_path.clone(),
                name: name.clone(),
                is_directory,
                uncompressed_size: data_length as u64,
                compressed_size: data_length as u64,
                modified_time: timestamp,
                compression_method: Some(CompressionMethod::Store),
                is_encrypted: false,
                crc32: None,
                permissions: None,
            };

            entries.push(entry);

            // Recursively read subdirectories
            if is_directory {
                let subdir_entries = self.read_directory(
                    file,
                    extent_location,
                    data_length,
                    &full_path,
                )?;
                entries.extend(subdir_entries);
            }

            pos += record_length;
        }

        Ok(entries)
    }

    /// Parse ISO 9660 timestamp
    fn parse_iso_timestamp(&self, bytes: &[u8]) -> Option<i64> {
        if bytes.len() < 7 {
            return None;
        }

        let year = 1900 + bytes[0] as i64;
        let month = bytes[1] as i64;
        let day = bytes[2] as i64;
        let hour = bytes[3] as i64;
        let minute = bytes[4] as i64;
        let second = bytes[5] as i64;
        // bytes[6] is timezone offset in 15-minute units from GMT

        // Simplified timestamp calculation
        Some((year - 1970) * 31536000 + month * 2592000 + day * 86400 + hour * 3600 + minute * 60 + second)
    }

    /// List entries in the archive
    pub fn list_entries(&self, prefix: &str) -> Result<Vec<ArchiveEntry>> {
        if prefix.is_empty() {
            Ok(self.entries_cache.clone())
        } else {
            Ok(self.entries_cache
                .iter()
                .filter(|e| e.path.starts_with(prefix))
                .cloned()
                .collect())
        }
    }

    /// Extract a single entry
    pub fn extract_entry(&self, entry: &ArchiveEntry, dest: &Path) -> Result<()> {
        let output_path = dest.join(&entry.path);

        if entry.is_directory {
            std::fs::create_dir_all(&output_path)?;
            return Ok(());
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Find entry location by re-reading the ISO
        let mut file = File::open(&self.path)?;
        let (location, size) = self.find_entry_location(&mut file, &entry.path)?;

        // Extract file data
        let offset = (location as u64) * 2048;
        file.seek(SeekFrom::Start(offset))?;

        let mut buffer = vec![0u8; size as usize];
        file.read_exact(&mut buffer)?;

        std::fs::write(&output_path, buffer)?;

        Ok(())
    }

    /// Find the location of an entry in the ISO
    fn find_entry_location(&self, file: &mut File, path: &str) -> Result<(u32, u32)> {
        // Read Primary Volume Descriptor
        file.seek(SeekFrom::Start(16 * 2048))?;
        let mut pvd = [0u8; 2048];
        file.read_exact(&mut pvd)?;

        let root_dir_record = &pvd[156..190];
        let mut current_location = u32::from_le_bytes([
            root_dir_record[2],
            root_dir_record[3],
            root_dir_record[4],
            root_dir_record[5],
        ]);
        let mut current_size = u32::from_le_bytes([
            root_dir_record[10],
            root_dir_record[11],
            root_dir_record[12],
            root_dir_record[13],
        ]);

        let path_parts: Vec<&str> = path.split('/').collect();

        for part in &path_parts {
            let (loc, size, is_dir) = self.find_in_directory(file, current_location, current_size, part)?;
            current_location = loc;
            current_size = size;
        }

        Ok((current_location, current_size))
    }

    fn find_in_directory(
        &self,
        file: &mut File,
        location: u32,
        size: u32,
        name: &str,
    ) -> Result<(u32, u32, bool)> {
        let offset = (location as u64) * 2048;
        file.seek(SeekFrom::Start(offset))?;

        let mut buffer = vec![0u8; size as usize];
        file.read_exact(&mut buffer)?;

        let mut pos = 0;
        while pos < buffer.len() {
            let record_length = buffer[pos] as usize;
            if record_length == 0 {
                pos = ((pos / 2048) + 1) * 2048;
                continue;
            }

            if pos + record_length > buffer.len() {
                break;
            }

            let record = &buffer[pos..pos + record_length];

            let extent_location = u32::from_le_bytes([
                record[2], record[3], record[4], record[5],
            ]);

            let data_length = u32::from_le_bytes([
                record[10], record[11], record[12], record[13],
            ]);

            let flags = record[25];
            let is_directory = (flags & 0x02) != 0;

            let name_length = record[32] as usize;
            let name_bytes = &record[33..33 + name_length];
            let mut entry_name = String::from_utf8_lossy(name_bytes).to_string();

            if let Some(idx) = entry_name.find(';') {
                entry_name.truncate(idx);
            }

            if entry_name == name {
                return Ok((extent_location, data_length, is_directory));
            }

            pos += record_length;
        }

        Err(anyhow::anyhow!("Entry not found: {}", name))
    }

    /// Extract all entries
    pub fn extract_all(&self, dest: &Path) -> Result<()> {
        for entry in &self.entries_cache {
            self.extract_entry(entry, dest)?;
        }
        Ok(())
    }

    /// Test archive integrity
    pub fn test_integrity(&self) -> Result<bool> {
        // Try to read all entries
        let mut file = File::open(&self.path)?;

        for entry in &self.entries_cache {
            if !entry.is_directory {
                match self.find_entry_location(&mut file, &entry.path) {
                    Ok((loc, size)) => {
                        let offset = (loc as u64) * 2048;
                        file.seek(SeekFrom::Start(offset))?;

                        let mut buffer = vec![0u8; size as usize];
                        if let Err(e) = file.read_exact(&mut buffer) {
                            eprintln!("Integrity check failed for {}: {}", entry.path, e);
                            return Ok(false);
                        }
                    }
                    Err(e) => {
                        eprintln!("Integrity check failed: {}", e);
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Read file content as text
    pub fn read_text(&self, entry: &ArchiveEntry, max_size: usize) -> Result<String> {
        if entry.is_directory {
            return Err(anyhow::anyhow!("Cannot read directory as text"));
        }

        let mut file = File::open(&self.path)?;
        let (location, size) = self.find_entry_location(&mut file, &entry.path)?;

        let read_size = std::cmp::min(size as usize, max_size);
        let offset = (location as u64) * 2048;
        file.seek(SeekFrom::Start(offset))?;

        let mut buffer = vec![0u8; read_size];
        file.read_exact(&mut buffer)?;

        String::from_utf8(buffer).context("File is not valid UTF-8 text")
    }

    /// Get volume label
    pub fn volume_label(&self) -> &str {
        &self.volume_label
    }
}
