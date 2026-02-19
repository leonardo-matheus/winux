//! Archive handling module
//!
//! Provides unified interface for working with various archive formats.

mod formats;
mod zip;
mod tar;
mod rar;
mod p7zip;
mod iso;

pub use formats::{ArchiveFormat, CompressionMethod};
pub use self::zip::ZipArchive;
pub use self::tar::TarArchive;
pub use self::rar::RarArchive;
pub use self::p7zip::SevenZipArchive;
pub use self::iso::IsoArchive;

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Represents an entry within an archive
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// Full path within the archive
    pub path: String,
    /// File name only
    pub name: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Uncompressed size in bytes
    pub uncompressed_size: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Modification time (Unix timestamp)
    pub modified_time: Option<i64>,
    /// Compression method used
    pub compression_method: Option<CompressionMethod>,
    /// Whether the entry is encrypted
    pub is_encrypted: bool,
    /// CRC32 checksum
    pub crc32: Option<u32>,
    /// Unix permissions (if available)
    pub permissions: Option<u32>,
}

impl ArchiveEntry {
    /// Get the compression ratio as a percentage
    pub fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            return 0.0;
        }
        (1.0 - (self.compressed_size as f64 / self.uncompressed_size as f64)) * 100.0
    }

    /// Get parent directory path
    pub fn parent_path(&self) -> Option<&str> {
        self.path.rfind('/').map(|i| &self.path[..i])
    }
}

/// Unified archive interface
pub struct Archive {
    inner: ArchiveInner,
    path: PathBuf,
    format: ArchiveFormat,
}

enum ArchiveInner {
    Zip(ZipArchive),
    Tar(TarArchive),
    Rar(RarArchive),
    SevenZip(SevenZipArchive),
    Iso(IsoArchive),
}

impl Archive {
    /// Open an archive from a file path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let format = ArchiveFormat::from_path(path)
            .ok_or_else(|| anyhow::anyhow!("Unknown archive format"))?;

        let inner = match format {
            ArchiveFormat::Zip => ArchiveInner::Zip(ZipArchive::open(path)?),
            ArchiveFormat::Tar
            | ArchiveFormat::TarGz
            | ArchiveFormat::TarBz2
            | ArchiveFormat::TarXz
            | ArchiveFormat::Zstd => ArchiveInner::Tar(TarArchive::open(path, format)?),
            ArchiveFormat::Rar => ArchiveInner::Rar(RarArchive::open(path)?),
            ArchiveFormat::SevenZip => ArchiveInner::SevenZip(SevenZipArchive::open(path)?),
            ArchiveFormat::Iso => ArchiveInner::Iso(IsoArchive::open(path)?),
        };

        Ok(Self {
            inner,
            path: path.to_path_buf(),
            format,
        })
    }

    /// Create a new archive
    pub fn create(path: impl AsRef<Path>, format: ArchiveFormat) -> Result<Self> {
        let path = path.as_ref();

        let inner = match format {
            ArchiveFormat::Zip => ArchiveInner::Zip(ZipArchive::create(path)?),
            ArchiveFormat::Tar
            | ArchiveFormat::TarGz
            | ArchiveFormat::TarBz2
            | ArchiveFormat::TarXz
            | ArchiveFormat::Zstd => ArchiveInner::Tar(TarArchive::create(path, format)?),
            ArchiveFormat::SevenZip => ArchiveInner::SevenZip(SevenZipArchive::create(path)?),
            ArchiveFormat::Rar => return Err(anyhow::anyhow!("RAR creation requires proprietary tools")),
            ArchiveFormat::Iso => return Err(anyhow::anyhow!("ISO creation not supported")),
        };

        Ok(Self {
            inner,
            path: path.to_path_buf(),
            format,
        })
    }

    /// Get archive format
    pub fn format(&self) -> ArchiveFormat {
        self.format
    }

    /// Get archive file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// List all entries in the archive
    pub fn list_entries(&self, prefix: &str) -> Result<Vec<ArchiveEntry>> {
        match &self.inner {
            ArchiveInner::Zip(a) => a.list_entries(prefix),
            ArchiveInner::Tar(a) => a.list_entries(prefix),
            ArchiveInner::Rar(a) => a.list_entries(prefix),
            ArchiveInner::SevenZip(a) => a.list_entries(prefix),
            ArchiveInner::Iso(a) => a.list_entries(prefix),
        }
    }

    /// Extract a single entry
    pub fn extract_entry(&self, entry: &ArchiveEntry, dest: &Path, password: Option<&str>) -> Result<()> {
        match &self.inner {
            ArchiveInner::Zip(a) => a.extract_entry(entry, dest, password),
            ArchiveInner::Tar(a) => a.extract_entry(entry, dest),
            ArchiveInner::Rar(a) => a.extract_entry(entry, dest, password),
            ArchiveInner::SevenZip(a) => a.extract_entry(entry, dest, password),
            ArchiveInner::Iso(a) => a.extract_entry(entry, dest),
        }
    }

    /// Extract all entries
    pub fn extract_all(&self, dest: &Path, password: Option<&str>) -> Result<()> {
        match &self.inner {
            ArchiveInner::Zip(a) => a.extract_all(dest, password),
            ArchiveInner::Tar(a) => a.extract_all(dest),
            ArchiveInner::Rar(a) => a.extract_all(dest, password),
            ArchiveInner::SevenZip(a) => a.extract_all(dest, password),
            ArchiveInner::Iso(a) => a.extract_all(dest),
        }
    }

    /// Add a file to the archive
    pub fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        match &mut self.inner {
            ArchiveInner::Zip(a) => a.add_file(file_path, archive_path),
            ArchiveInner::Tar(a) => a.add_file(file_path, archive_path),
            ArchiveInner::SevenZip(a) => a.add_file(file_path, archive_path),
            ArchiveInner::Rar(_) => Err(anyhow::anyhow!("RAR modification requires proprietary tools")),
            ArchiveInner::Iso(_) => Err(anyhow::anyhow!("ISO modification not supported")),
        }
    }

    /// Remove an entry from the archive
    pub fn remove_entry(&mut self, entry_path: &str) -> Result<()> {
        match &mut self.inner {
            ArchiveInner::Zip(a) => a.remove_entry(entry_path),
            ArchiveInner::Tar(_) => Err(anyhow::anyhow!("TAR archives don't support removing entries")),
            ArchiveInner::SevenZip(a) => a.remove_entry(entry_path),
            ArchiveInner::Rar(_) => Err(anyhow::anyhow!("RAR modification requires proprietary tools")),
            ArchiveInner::Iso(_) => Err(anyhow::anyhow!("ISO modification not supported")),
        }
    }

    /// Test archive integrity
    pub fn test_integrity(&self) -> Result<bool> {
        match &self.inner {
            ArchiveInner::Zip(a) => a.test_integrity(),
            ArchiveInner::Tar(a) => a.test_integrity(),
            ArchiveInner::Rar(a) => a.test_integrity(),
            ArchiveInner::SevenZip(a) => a.test_integrity(),
            ArchiveInner::Iso(a) => a.test_integrity(),
        }
    }

    /// Read file content as text (for preview)
    pub fn read_text(&self, entry: &ArchiveEntry, max_size: usize) -> Result<String> {
        match &self.inner {
            ArchiveInner::Zip(a) => a.read_text(entry, max_size),
            ArchiveInner::Tar(a) => a.read_text(entry, max_size),
            ArchiveInner::Rar(a) => a.read_text(entry, max_size),
            ArchiveInner::SevenZip(a) => a.read_text(entry, max_size),
            ArchiveInner::Iso(a) => a.read_text(entry, max_size),
        }
    }

    /// Get archive statistics
    pub fn statistics(&self) -> ArchiveStatistics {
        let entries = self.list_entries("").unwrap_or_default();

        let file_count = entries.iter().filter(|e| !e.is_directory).count();
        let dir_count = entries.iter().filter(|e| e.is_directory).count();
        let total_uncompressed: u64 = entries.iter().map(|e| e.uncompressed_size).sum();
        let total_compressed: u64 = entries.iter().map(|e| e.compressed_size).sum();

        ArchiveStatistics {
            file_count,
            directory_count: dir_count,
            total_uncompressed_size: total_uncompressed,
            total_compressed_size: total_compressed,
            compression_ratio: if total_uncompressed > 0 {
                (1.0 - (total_compressed as f64 / total_uncompressed as f64)) * 100.0
            } else {
                0.0
            },
            format: self.format,
            has_encryption: entries.iter().any(|e| e.is_encrypted),
        }
    }
}

/// Archive statistics
#[derive(Debug, Clone)]
pub struct ArchiveStatistics {
    pub file_count: usize,
    pub directory_count: usize,
    pub total_uncompressed_size: u64,
    pub total_compressed_size: u64,
    pub compression_ratio: f64,
    pub format: ArchiveFormat,
    pub has_encryption: bool,
}
