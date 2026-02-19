//! TAR archive support (including .tar.gz, .tar.bz2, .tar.xz, .tar.zst)

use super::{ArchiveEntry, ArchiveFormat, CompressionMethod};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

/// TAR archive handler
pub struct TarArchive {
    path: std::path::PathBuf,
    format: ArchiveFormat,
    entries_cache: Vec<ArchiveEntry>,
}

impl TarArchive {
    /// Open an existing TAR archive
    pub fn open(path: &Path, format: ArchiveFormat) -> Result<Self> {
        let mut archive = Self {
            path: path.to_path_buf(),
            format,
            entries_cache: Vec::new(),
        };

        // Cache entries on open
        archive.entries_cache = archive.read_entries()?;

        Ok(archive)
    }

    /// Create a new TAR archive
    pub fn create(path: &Path, format: ArchiveFormat) -> Result<Self> {
        // Create empty archive
        let file = File::create(path).context("Failed to create TAR file")?;

        match format {
            ArchiveFormat::Tar => {
                let builder = tar::Builder::new(file);
                builder.into_inner()?;
            }
            ArchiveFormat::TarGz => {
                let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
                let builder = tar::Builder::new(encoder);
                builder.into_inner()?.finish()?;
            }
            ArchiveFormat::TarBz2 => {
                let encoder = bzip2::write::BzEncoder::new(file, bzip2::Compression::default());
                let builder = tar::Builder::new(encoder);
                builder.into_inner()?.finish()?;
            }
            ArchiveFormat::TarXz => {
                let encoder = xz2::write::XzEncoder::new(file, 6);
                let builder = tar::Builder::new(encoder);
                builder.into_inner()?.finish()?;
            }
            ArchiveFormat::Zstd => {
                let encoder = zstd::stream::Encoder::new(file, 3)?;
                let builder = tar::Builder::new(encoder);
                builder.into_inner()?.finish()?;
            }
            _ => return Err(anyhow::anyhow!("Invalid TAR format")),
        }

        Ok(Self {
            path: path.to_path_buf(),
            format,
            entries_cache: Vec::new(),
        })
    }

    /// Read all entries from the archive
    fn read_entries(&self) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        let mut entries = Vec::new();

        match self.format {
            ArchiveFormat::Tar => {
                let mut archive = tar::Archive::new(reader);
                self.collect_entries(&mut archive, &mut entries)?;
            }
            ArchiveFormat::TarGz => {
                let decoder = flate2::read::GzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.collect_entries(&mut archive, &mut entries)?;
            }
            ArchiveFormat::TarBz2 => {
                let decoder = bzip2::read::BzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.collect_entries(&mut archive, &mut entries)?;
            }
            ArchiveFormat::TarXz => {
                let decoder = xz2::read::XzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.collect_entries(&mut archive, &mut entries)?;
            }
            ArchiveFormat::Zstd => {
                let decoder = zstd::stream::Decoder::new(reader)?;
                let mut archive = tar::Archive::new(decoder);
                self.collect_entries(&mut archive, &mut entries)?;
            }
            _ => return Err(anyhow::anyhow!("Invalid TAR format")),
        }

        Ok(entries)
    }

    fn collect_entries<R: Read>(&self, archive: &mut tar::Archive<R>, entries: &mut Vec<ArchiveEntry>) -> Result<()> {
        for entry_result in archive.entries()? {
            let entry = entry_result?;
            let header = entry.header();
            let path = entry.path()?.to_string_lossy().to_string();

            let entry = ArchiveEntry {
                path: path.clone(),
                name: path.rsplit('/').next().unwrap_or(&path).to_string(),
                is_directory: header.entry_type().is_dir(),
                uncompressed_size: header.size()?,
                compressed_size: header.size()?, // TAR doesn't store compressed size per entry
                modified_time: header.mtime().ok().map(|t| t as i64),
                compression_method: Some(self.get_compression_method()),
                is_encrypted: false,
                crc32: None,
                permissions: header.mode().ok(),
            };

            entries.push(entry);
        }

        Ok(())
    }

    fn get_compression_method(&self) -> CompressionMethod {
        match self.format {
            ArchiveFormat::Tar => CompressionMethod::Store,
            ArchiveFormat::TarGz => CompressionMethod::Deflate,
            ArchiveFormat::TarBz2 => CompressionMethod::BZip2,
            ArchiveFormat::TarXz => CompressionMethod::Xz,
            ArchiveFormat::Zstd => CompressionMethod::Zstd,
            _ => CompressionMethod::Unknown(0),
        }
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
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        match self.format {
            ArchiveFormat::Tar => {
                let mut archive = tar::Archive::new(reader);
                self.extract_entry_from_archive(&mut archive, entry, dest)?;
            }
            ArchiveFormat::TarGz => {
                let decoder = flate2::read::GzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.extract_entry_from_archive(&mut archive, entry, dest)?;
            }
            ArchiveFormat::TarBz2 => {
                let decoder = bzip2::read::BzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.extract_entry_from_archive(&mut archive, entry, dest)?;
            }
            ArchiveFormat::TarXz => {
                let decoder = xz2::read::XzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.extract_entry_from_archive(&mut archive, entry, dest)?;
            }
            ArchiveFormat::Zstd => {
                let decoder = zstd::stream::Decoder::new(reader)?;
                let mut archive = tar::Archive::new(decoder);
                self.extract_entry_from_archive(&mut archive, entry, dest)?;
            }
            _ => return Err(anyhow::anyhow!("Invalid TAR format")),
        }

        Ok(())
    }

    fn extract_entry_from_archive<R: Read>(
        &self,
        archive: &mut tar::Archive<R>,
        target_entry: &ArchiveEntry,
        dest: &Path,
    ) -> Result<()> {
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?.to_string_lossy().to_string();

            if path == target_entry.path {
                let output_path = dest.join(&path);

                if target_entry.is_directory {
                    std::fs::create_dir_all(&output_path)?;
                } else {
                    if let Some(parent) = output_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    entry.unpack(&output_path)?;
                }

                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Entry not found: {}", target_entry.path))
    }

    /// Extract all entries
    pub fn extract_all(&self, dest: &Path) -> Result<()> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        std::fs::create_dir_all(dest)?;

        match self.format {
            ArchiveFormat::Tar => {
                let mut archive = tar::Archive::new(reader);
                archive.unpack(dest)?;
            }
            ArchiveFormat::TarGz => {
                let decoder = flate2::read::GzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            ArchiveFormat::TarBz2 => {
                let decoder = bzip2::read::BzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            ArchiveFormat::TarXz => {
                let decoder = xz2::read::XzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            ArchiveFormat::Zstd => {
                let decoder = zstd::stream::Decoder::new(reader)?;
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            _ => return Err(anyhow::anyhow!("Invalid TAR format")),
        }

        Ok(())
    }

    /// Add a file to the archive
    pub fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        // TAR archives need to be rebuilt to add files
        let temp_path = self.path.with_extension("tar.tmp");

        // Create new archive with existing entries + new file
        let temp_file = File::create(&temp_path)?;

        match self.format {
            ArchiveFormat::Tar => {
                let mut builder = tar::Builder::new(temp_file);
                self.rebuild_with_new_file(&mut builder, file_path, archive_path)?;
                builder.into_inner()?;
            }
            ArchiveFormat::TarGz => {
                let encoder = flate2::write::GzEncoder::new(temp_file, flate2::Compression::default());
                let mut builder = tar::Builder::new(encoder);
                self.rebuild_with_new_file(&mut builder, file_path, archive_path)?;
                builder.into_inner()?.finish()?;
            }
            ArchiveFormat::TarBz2 => {
                let encoder = bzip2::write::BzEncoder::new(temp_file, bzip2::Compression::default());
                let mut builder = tar::Builder::new(encoder);
                self.rebuild_with_new_file(&mut builder, file_path, archive_path)?;
                builder.into_inner()?.finish()?;
            }
            ArchiveFormat::TarXz => {
                let encoder = xz2::write::XzEncoder::new(temp_file, 6);
                let mut builder = tar::Builder::new(encoder);
                self.rebuild_with_new_file(&mut builder, file_path, archive_path)?;
                builder.into_inner()?.finish()?;
            }
            ArchiveFormat::Zstd => {
                let encoder = zstd::stream::Encoder::new(temp_file, 3)?;
                let mut builder = tar::Builder::new(encoder);
                self.rebuild_with_new_file(&mut builder, file_path, archive_path)?;
                builder.into_inner()?.finish()?;
            }
            _ => return Err(anyhow::anyhow!("Invalid TAR format")),
        }

        // Replace original
        std::fs::rename(&temp_path, &self.path)?;

        // Refresh cache
        self.entries_cache = self.read_entries()?;

        Ok(())
    }

    fn rebuild_with_new_file<W: Write>(
        &self,
        builder: &mut tar::Builder<W>,
        new_file_path: &Path,
        new_archive_path: &str,
    ) -> Result<()> {
        // First, extract and re-add all existing entries
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        // Create temp directory for extraction
        let temp_dir = tempfile::tempdir()?;

        match self.format {
            ArchiveFormat::Tar => {
                let mut archive = tar::Archive::new(reader);
                archive.unpack(temp_dir.path())?;
            }
            ArchiveFormat::TarGz => {
                let decoder = flate2::read::GzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(temp_dir.path())?;
            }
            ArchiveFormat::TarBz2 => {
                let decoder = bzip2::read::BzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(temp_dir.path())?;
            }
            ArchiveFormat::TarXz => {
                let decoder = xz2::read::XzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(temp_dir.path())?;
            }
            ArchiveFormat::Zstd => {
                let decoder = zstd::stream::Decoder::new(reader)?;
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(temp_dir.path())?;
            }
            _ => return Err(anyhow::anyhow!("Invalid TAR format")),
        }

        // Add existing entries
        builder.append_dir_all("", temp_dir.path())?;

        // Add new file
        let mut new_file = File::open(new_file_path)?;
        builder.append_file(new_archive_path, &mut new_file)?;

        Ok(())
    }

    /// Test archive integrity
    pub fn test_integrity(&self) -> Result<bool> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        let result = match self.format {
            ArchiveFormat::Tar => {
                let mut archive = tar::Archive::new(reader);
                self.verify_entries(&mut archive)
            }
            ArchiveFormat::TarGz => {
                let decoder = flate2::read::GzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.verify_entries(&mut archive)
            }
            ArchiveFormat::TarBz2 => {
                let decoder = bzip2::read::BzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.verify_entries(&mut archive)
            }
            ArchiveFormat::TarXz => {
                let decoder = xz2::read::XzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.verify_entries(&mut archive)
            }
            ArchiveFormat::Zstd => {
                let decoder = zstd::stream::Decoder::new(reader)?;
                let mut archive = tar::Archive::new(decoder);
                self.verify_entries(&mut archive)
            }
            _ => return Err(anyhow::anyhow!("Invalid TAR format")),
        };

        result
    }

    fn verify_entries<R: Read>(&self, archive: &mut tar::Archive<R>) -> Result<bool> {
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            // Try to read the entry to verify integrity
            let mut buffer = Vec::new();
            if let Err(e) = entry.read_to_end(&mut buffer) {
                eprintln!("Integrity check failed: {}", e);
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
        let reader = BufReader::new(file);

        match self.format {
            ArchiveFormat::Tar => {
                let mut archive = tar::Archive::new(reader);
                self.read_entry_text(&mut archive, entry, max_size)
            }
            ArchiveFormat::TarGz => {
                let decoder = flate2::read::GzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.read_entry_text(&mut archive, entry, max_size)
            }
            ArchiveFormat::TarBz2 => {
                let decoder = bzip2::read::BzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.read_entry_text(&mut archive, entry, max_size)
            }
            ArchiveFormat::TarXz => {
                let decoder = xz2::read::XzDecoder::new(reader);
                let mut archive = tar::Archive::new(decoder);
                self.read_entry_text(&mut archive, entry, max_size)
            }
            ArchiveFormat::Zstd => {
                let decoder = zstd::stream::Decoder::new(reader)?;
                let mut archive = tar::Archive::new(decoder);
                self.read_entry_text(&mut archive, entry, max_size)
            }
            _ => Err(anyhow::anyhow!("Invalid TAR format")),
        }
    }

    fn read_entry_text<R: Read>(
        &self,
        archive: &mut tar::Archive<R>,
        target_entry: &ArchiveEntry,
        max_size: usize,
    ) -> Result<String> {
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?.to_string_lossy().to_string();

            if path == target_entry.path {
                let size = std::cmp::min(target_entry.uncompressed_size as usize, max_size);
                let mut buffer = vec![0u8; size];
                entry.read_exact(&mut buffer)?;
                return String::from_utf8(buffer).context("File is not valid UTF-8 text");
            }
        }

        Err(anyhow::anyhow!("Entry not found: {}", target_entry.path))
    }
}
