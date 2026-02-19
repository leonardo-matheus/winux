//! 7-Zip archive support

use super::{ArchiveEntry, CompressionMethod};
use anyhow::{Context, Result};
use sevenz_rust::{Archive as SzArchive, BlockDecoder};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// 7-Zip archive handler
pub struct SevenZipArchive {
    path: std::path::PathBuf,
    entries_cache: Vec<ArchiveEntry>,
}

impl SevenZipArchive {
    /// Open an existing 7z archive
    pub fn open(path: &Path) -> Result<Self> {
        let mut archive = Self {
            path: path.to_path_buf(),
            entries_cache: Vec::new(),
        };

        archive.entries_cache = archive.read_entries()?;

        Ok(archive)
    }

    /// Create a new 7z archive
    pub fn create(path: &Path) -> Result<Self> {
        // Create empty 7z archive
        let file = File::create(path).context("Failed to create 7z file")?;

        // sevenz-rust doesn't support creating empty archives directly
        // We need to use the command line tool for creation
        drop(file);

        Ok(Self {
            path: path.to_path_buf(),
            entries_cache: Vec::new(),
        })
    }

    /// Read all entries from the archive
    fn read_entries(&self) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let archive = SzArchive::read(reader, &[])
            .context("Failed to read 7z archive")?;

        let mut entries = Vec::new();

        for (idx, entry) in archive.files.iter().enumerate() {
            let path = entry.name().to_string();
            let name = path.rsplit('/').next().unwrap_or(&path).to_string();

            let entry = ArchiveEntry {
                path,
                name,
                is_directory: entry.is_directory(),
                uncompressed_size: entry.size(),
                compressed_size: entry.size(), // Actual compressed size not easily available
                modified_time: entry.last_modified()
                    .map(|dt| dt.timestamp())
                    .unwrap_or(None),
                compression_method: Some(CompressionMethod::Lzma2),
                is_encrypted: entry.has_stream() && archive.folders.get(0)
                    .map(|f| f.has_crypt())
                    .unwrap_or(false),
                crc32: entry.crc(),
                permissions: None,
            };

            entries.push(entry);
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
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        let password_bytes: Vec<u8> = password
            .map(|p| p.as_bytes().to_vec())
            .unwrap_or_default();

        let archive = SzArchive::read(reader, &password_bytes)
            .context("Failed to read 7z archive")?;

        let output_path = dest.join(&entry.path);

        if entry.is_directory {
            std::fs::create_dir_all(&output_path)?;
            return Ok(());
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Find entry index
        let entry_idx = archive.files.iter()
            .position(|e| e.name() == entry.path)
            .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;

        // Extract using sevenz-rust
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        sevenz_rust::decompress_file_with_extract_fn(
            reader,
            &password_bytes,
            |file_entry, reader, _| {
                if file_entry.name() == entry.path {
                    let mut output = File::create(&output_path)?;
                    std::io::copy(reader, &mut output)?;
                }
                Ok(true)
            },
        ).context("Failed to extract entry")?;

        Ok(())
    }

    /// Extract all entries
    pub fn extract_all(&self, dest: &Path, password: Option<&str>) -> Result<()> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        let password_bytes: Vec<u8> = password
            .map(|p| p.as_bytes().to_vec())
            .unwrap_or_default();

        std::fs::create_dir_all(dest)?;

        sevenz_rust::decompress_file_with_extract_fn(
            reader,
            &password_bytes,
            |entry, reader, _| {
                let output_path = dest.join(entry.name());

                if entry.is_directory() {
                    std::fs::create_dir_all(&output_path)?;
                } else {
                    if let Some(parent) = output_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut output = File::create(&output_path)?;
                    std::io::copy(reader, &mut output)?;
                }
                Ok(true)
            },
        ).context("Failed to extract archive")?;

        Ok(())
    }

    /// Add a file to the archive (requires rebuilding)
    pub fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        // 7z archives need to be rebuilt to add files
        // Use command line tool for this operation
        let output = std::process::Command::new("7z")
            .args(["a", "-y", self.path.to_str().unwrap()])
            .arg(format!("-ir!{}", file_path.display()))
            .output()
            .context("Failed to run 7z command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "7z add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Refresh cache
        self.entries_cache = self.read_entries()?;

        Ok(())
    }

    /// Remove an entry from the archive
    pub fn remove_entry(&mut self, entry_path: &str) -> Result<()> {
        let output = std::process::Command::new("7z")
            .args(["d", "-y", self.path.to_str().unwrap(), entry_path])
            .output()
            .context("Failed to run 7z command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "7z delete failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Refresh cache
        self.entries_cache = self.read_entries()?;

        Ok(())
    }

    /// Test archive integrity
    pub fn test_integrity(&self) -> Result<bool> {
        // Try to read all entries
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        match sevenz_rust::decompress_file_with_extract_fn(
            reader,
            &[],
            |_, reader, _| {
                // Just read through the data without saving
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)?;
                Ok(true)
            },
        ) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Integrity check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Read file content as text
    pub fn read_text(&self, entry: &ArchiveEntry, max_size: usize) -> Result<String> {
        if entry.is_directory {
            return Err(anyhow::anyhow!("Cannot read directory as text"));
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut result = String::new();

        sevenz_rust::decompress_file_with_extract_fn(
            reader,
            &[],
            |file_entry, reader, _| {
                if file_entry.name() == entry.path {
                    let mut buffer = vec![0u8; max_size];
                    let bytes_read = reader.take(max_size as u64).read(&mut buffer)?;
                    buffer.truncate(bytes_read);
                    result = String::from_utf8(buffer)
                        .unwrap_or_else(|_| "<Binary content>".to_string());
                }
                Ok(true)
            },
        )?;

        Ok(result)
    }
}
