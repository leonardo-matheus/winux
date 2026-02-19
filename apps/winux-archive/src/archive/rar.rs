//! RAR archive support (extraction only, requires unrar)

use super::{ArchiveEntry, CompressionMethod};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// RAR archive handler
pub struct RarArchive {
    path: std::path::PathBuf,
    entries_cache: Vec<ArchiveEntry>,
}

impl RarArchive {
    /// Open an existing RAR archive
    pub fn open(path: &Path) -> Result<Self> {
        // Check if unrar is available
        if !Self::check_unrar_available() {
            return Err(anyhow::anyhow!(
                "unrar is not installed. Please install unrar to work with RAR archives."
            ));
        }

        let mut archive = Self {
            path: path.to_path_buf(),
            entries_cache: Vec::new(),
        };

        archive.entries_cache = archive.read_entries()?;

        Ok(archive)
    }

    /// Check if unrar is available
    fn check_unrar_available() -> bool {
        Command::new("unrar")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Read entries using unrar command
    fn read_entries(&self) -> Result<Vec<ArchiveEntry>> {
        let output = Command::new("unrar")
            .args(["lt", "-v", self.path.to_str().unwrap()])
            .output()
            .context("Failed to run unrar")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "unrar failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_unrar_output(&output_str)
    }

    /// Parse unrar verbose listing output
    fn parse_unrar_output(&self, output: &str) -> Result<Vec<ArchiveEntry>> {
        let mut entries = Vec::new();
        let mut current_entry: Option<ArchiveEntryBuilder> = None;

        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("Name:") {
                // Save previous entry if exists
                if let Some(builder) = current_entry.take() {
                    if let Some(entry) = builder.build() {
                        entries.push(entry);
                    }
                }
                // Start new entry
                let name = line.strip_prefix("Name:").unwrap_or("").trim();
                current_entry = Some(ArchiveEntryBuilder::new(name.to_string()));
            } else if let Some(ref mut builder) = current_entry {
                if line.starts_with("Size:") {
                    if let Some(size_str) = line.strip_prefix("Size:") {
                        builder.uncompressed_size = size_str.trim().parse().unwrap_or(0);
                    }
                } else if line.starts_with("Packed size:") {
                    if let Some(size_str) = line.strip_prefix("Packed size:") {
                        builder.compressed_size = size_str.trim().parse().unwrap_or(0);
                    }
                } else if line.starts_with("Type:") {
                    builder.is_directory = line.contains("Directory");
                } else if line.starts_with("Flags:") {
                    builder.is_encrypted = line.contains("encrypted");
                } else if line.starts_with("CRC32:") {
                    if let Some(crc_str) = line.strip_prefix("CRC32:") {
                        builder.crc32 = u32::from_str_radix(crc_str.trim(), 16).ok();
                    }
                }
            }
        }

        // Save last entry
        if let Some(builder) = current_entry {
            if let Some(entry) = builder.build() {
                entries.push(entry);
            }
        }

        Ok(entries)
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
    pub fn extract_entry(&self, entry: &ArchiveEntry, dest: &Path, password: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("unrar");
        cmd.arg("x")
            .arg("-y") // Answer yes to all
            .arg("-o+"); // Overwrite existing

        if let Some(pwd) = password {
            cmd.arg(format!("-p{}", pwd));
        }

        cmd.arg(self.path.to_str().unwrap())
            .arg(&entry.path)
            .arg(dest.to_str().unwrap());

        let output = cmd.output().context("Failed to run unrar")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "unrar extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Extract all entries
    pub fn extract_all(&self, dest: &Path, password: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("unrar");
        cmd.arg("x")
            .arg("-y")
            .arg("-o+");

        if let Some(pwd) = password {
            cmd.arg(format!("-p{}", pwd));
        }

        cmd.arg(self.path.to_str().unwrap())
            .arg(dest.to_str().unwrap());

        let output = cmd.output().context("Failed to run unrar")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "unrar extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Test archive integrity
    pub fn test_integrity(&self) -> Result<bool> {
        let output = Command::new("unrar")
            .args(["t", self.path.to_str().unwrap()])
            .output()
            .context("Failed to run unrar")?;

        Ok(output.status.success())
    }

    /// Read file content as text
    pub fn read_text(&self, entry: &ArchiveEntry, max_size: usize) -> Result<String> {
        if entry.is_directory {
            return Err(anyhow::anyhow!("Cannot read directory as text"));
        }

        // Extract to temp directory
        let temp_dir = tempfile::tempdir()?;
        self.extract_entry(entry, temp_dir.path(), None)?;

        let file_path = temp_dir.path().join(&entry.path);
        let content = std::fs::read_to_string(&file_path)
            .context("Failed to read extracted file")?;

        if content.len() > max_size {
            Ok(content[..max_size].to_string())
        } else {
            Ok(content)
        }
    }
}

/// Builder for constructing ArchiveEntry from unrar output
struct ArchiveEntryBuilder {
    path: String,
    is_directory: bool,
    uncompressed_size: u64,
    compressed_size: u64,
    is_encrypted: bool,
    crc32: Option<u32>,
}

impl ArchiveEntryBuilder {
    fn new(path: String) -> Self {
        Self {
            path,
            is_directory: false,
            uncompressed_size: 0,
            compressed_size: 0,
            is_encrypted: false,
            crc32: None,
        }
    }

    fn build(self) -> Option<ArchiveEntry> {
        if self.path.is_empty() {
            return None;
        }

        let name = self.path
            .rsplit('/')
            .next()
            .unwrap_or(&self.path)
            .to_string();

        Some(ArchiveEntry {
            path: self.path,
            name,
            is_directory: self.is_directory,
            uncompressed_size: self.uncompressed_size,
            compressed_size: self.compressed_size,
            modified_time: None,
            compression_method: Some(CompressionMethod::Lzma), // RAR typically uses LZMA variant
            is_encrypted: self.is_encrypted,
            crc32: self.crc32,
            permissions: None,
        })
    }
}
