//! ZIP archive support

use super::{ArchiveEntry, CompressionMethod};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::read::ZipArchive as ZipReader;
use zip::write::ZipWriter;
use zip::{CompressionMethod as ZipCompression, ZipArchive as ZipFile};

/// ZIP archive handler
pub struct ZipArchive {
    reader: Option<ZipReader<File>>,
    path: std::path::PathBuf,
}

impl ZipArchive {
    /// Open an existing ZIP archive
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).context("Failed to open ZIP file")?;
        let reader = ZipReader::new(file).context("Failed to read ZIP archive")?;

        Ok(Self {
            reader: Some(reader),
            path: path.to_path_buf(),
        })
    }

    /// Create a new ZIP archive
    pub fn create(path: &Path) -> Result<Self> {
        // Create empty file
        let file = File::create(path).context("Failed to create ZIP file")?;
        let mut writer = ZipWriter::new(file);
        writer.finish()?;

        // Reopen as reader
        Self::open(path)
    }

    /// List entries in the archive
    pub fn list_entries(&self, prefix: &str) -> Result<Vec<ArchiveEntry>> {
        let reader = self.reader.as_ref().ok_or_else(|| anyhow::anyhow!("Archive not open"))?;

        let mut entries = Vec::new();

        for i in 0..reader.len() {
            let file = reader.by_index_raw(i)?;
            let name = file.name();

            if !prefix.is_empty() && !name.starts_with(prefix) {
                continue;
            }

            let entry = ArchiveEntry {
                path: name.to_string(),
                name: name.rsplit('/').next().unwrap_or(name).to_string(),
                is_directory: file.is_dir(),
                uncompressed_size: file.size(),
                compressed_size: file.compressed_size(),
                modified_time: file.last_modified().map(|dt| {
                    // Convert DOS datetime to Unix timestamp
                    let year = dt.year() as i64;
                    let month = dt.month() as i64;
                    let day = dt.day() as i64;
                    let hour = dt.hour() as i64;
                    let minute = dt.minute() as i64;
                    let second = dt.second() as i64;
                    // Simplified timestamp calculation
                    (year - 1970) * 31536000 + month * 2592000 + day * 86400 + hour * 3600 + minute * 60 + second
                }),
                compression_method: Some(match file.compression() {
                    ZipCompression::Stored => CompressionMethod::Store,
                    ZipCompression::Deflated => CompressionMethod::Deflate,
                    ZipCompression::Bzip2 => CompressionMethod::BZip2,
                    ZipCompression::Zstd => CompressionMethod::Zstd,
                    _ => CompressionMethod::Unknown(0),
                }),
                is_encrypted: file.encrypted(),
                crc32: Some(file.crc32()),
                permissions: file.unix_mode(),
            };

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Extract a single entry
    pub fn extract_entry(&self, entry: &ArchiveEntry, dest: &Path, password: Option<&str>) -> Result<()> {
        let file = File::open(&self.path)?;
        let mut archive = ZipReader::new(file)?;

        let mut zip_file = if let Some(pwd) = password {
            archive.by_name_decrypt(&entry.path, pwd.as_bytes())?
                .map_err(|_| anyhow::anyhow!("Invalid password"))?
        } else {
            archive.by_name(&entry.path)?
        };

        let output_path = dest.join(&entry.path);

        if entry.is_directory {
            std::fs::create_dir_all(&output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut zip_file, &mut output_file)?;

            // Set permissions on Unix
            #[cfg(unix)]
            if let Some(mode) = entry.permissions {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&output_path, std::fs::Permissions::from_mode(mode))?;
            }
        }

        Ok(())
    }

    /// Extract all entries
    pub fn extract_all(&self, dest: &Path, password: Option<&str>) -> Result<()> {
        let entries = self.list_entries("")?;

        for entry in &entries {
            self.extract_entry(entry, dest, password)?;
        }

        Ok(())
    }

    /// Add a file to the archive
    pub fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        // Read existing archive
        let existing_file = File::open(&self.path)?;
        let mut existing_archive = ZipReader::new(existing_file)?;

        // Create temp file for new archive
        let temp_path = self.path.with_extension("zip.tmp");
        let new_file = File::create(&temp_path)?;
        let mut writer = ZipWriter::new(new_file);

        // Copy existing entries
        for i in 0..existing_archive.len() {
            let file = existing_archive.by_index_raw(i)?;
            writer.raw_copy_file(file)?;
        }

        // Add new file
        let options = zip::write::FileOptions::default()
            .compression_method(ZipCompression::Deflated);

        let mut source = File::open(file_path)?;
        writer.start_file(archive_path, options)?;
        std::io::copy(&mut source, &mut writer)?;

        writer.finish()?;

        // Replace original with new
        std::fs::rename(&temp_path, &self.path)?;

        // Reopen
        let file = File::open(&self.path)?;
        self.reader = Some(ZipReader::new(file)?);

        Ok(())
    }

    /// Remove an entry from the archive
    pub fn remove_entry(&mut self, entry_path: &str) -> Result<()> {
        // Read existing archive
        let existing_file = File::open(&self.path)?;
        let mut existing_archive = ZipReader::new(existing_file)?;

        // Create temp file for new archive
        let temp_path = self.path.with_extension("zip.tmp");
        let new_file = File::create(&temp_path)?;
        let mut writer = ZipWriter::new(new_file);

        // Copy all entries except the one to remove
        for i in 0..existing_archive.len() {
            let file = existing_archive.by_index_raw(i)?;
            if file.name() != entry_path && !file.name().starts_with(&format!("{}/", entry_path)) {
                writer.raw_copy_file(file)?;
            }
        }

        writer.finish()?;

        // Replace original with new
        std::fs::rename(&temp_path, &self.path)?;

        // Reopen
        let file = File::open(&self.path)?;
        self.reader = Some(ZipReader::new(file)?);

        Ok(())
    }

    /// Test archive integrity
    pub fn test_integrity(&self) -> Result<bool> {
        let file = File::open(&self.path)?;
        let mut archive = ZipReader::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            // Try to read entire file to verify CRC
            let mut buffer = Vec::new();
            if let Err(e) = file.read_to_end(&mut buffer) {
                eprintln!("Integrity check failed for {}: {}", file.name(), e);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Read file content as text
    pub fn read_text(&self, entry: &ArchiveEntry, max_size: usize) -> Result<String> {
        if entry.is_directory {
            return Err(anyhow::anyhow!("Cannot read directory as text"));
        }

        let file = File::open(&self.path)?;
        let mut archive = ZipReader::new(file)?;
        let mut zip_file = archive.by_name(&entry.path)?;

        let size = std::cmp::min(entry.uncompressed_size as usize, max_size);
        let mut buffer = vec![0u8; size];
        zip_file.read_exact(&mut buffer)?;

        String::from_utf8(buffer).context("File is not valid UTF-8 text")
    }
}

/// Options for creating ZIP archives
#[derive(Debug, Clone)]
pub struct ZipOptions {
    pub compression_method: ZipCompression,
    pub compression_level: Option<u32>,
    pub password: Option<String>,
    pub comment: Option<String>,
}

impl Default for ZipOptions {
    fn default() -> Self {
        Self {
            compression_method: ZipCompression::Deflated,
            compression_level: Some(6),
            password: None,
            comment: None,
        }
    }
}
