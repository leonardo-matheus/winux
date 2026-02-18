//! Archive Module - Native support for compressed file formats
//!
//! Supports: zip, tar, tar.gz, tar.bz2, tar.xz, tar.zst, rar, 7z
//! Features: extract, compress, list_contents, get_info with progress callbacks

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported archive formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZst,
    Rar,
    SevenZip,
}

impl ArchiveFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
            Some(Self::TarGz)
        } else if path_str.ends_with(".tar.bz2") || path_str.ends_with(".tbz2") {
            Some(Self::TarBz2)
        } else if path_str.ends_with(".tar.xz") || path_str.ends_with(".txz") {
            Some(Self::TarXz)
        } else if path_str.ends_with(".tar.zst") || path_str.ends_with(".tzst") {
            Some(Self::TarZst)
        } else if path_str.ends_with(".tar") {
            Some(Self::Tar)
        } else if path_str.ends_with(".zip") {
            Some(Self::Zip)
        } else if path_str.ends_with(".rar") {
            Some(Self::Rar)
        } else if path_str.ends_with(".7z") {
            Some(Self::SevenZip)
        } else {
            None
        }
    }

    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Zip => ".zip",
            Self::Tar => ".tar",
            Self::TarGz => ".tar.gz",
            Self::TarBz2 => ".tar.bz2",
            Self::TarXz => ".tar.xz",
            Self::TarZst => ".tar.zst",
            Self::Rar => ".rar",
            Self::SevenZip => ".7z",
        }
    }

    /// Get MIME type for format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Zip => "application/zip",
            Self::Tar => "application/x-tar",
            Self::TarGz => "application/x-compressed-tar",
            Self::TarBz2 => "application/x-bzip-compressed-tar",
            Self::TarXz => "application/x-xz-compressed-tar",
            Self::TarZst => "application/x-zstd-compressed-tar",
            Self::Rar => "application/vnd.rar",
            Self::SevenZip => "application/x-7z-compressed",
        }
    }

    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Zip => "ZIP Archive",
            Self::Tar => "TAR Archive",
            Self::TarGz => "Gzipped TAR Archive",
            Self::TarBz2 => "Bzip2 TAR Archive",
            Self::TarXz => "XZ TAR Archive",
            Self::TarZst => "Zstd TAR Archive",
            Self::Rar => "RAR Archive",
            Self::SevenZip => "7-Zip Archive",
        }
    }

    /// Check if format supports compression level
    pub fn supports_compression_level(&self) -> bool {
        matches!(
            self,
            Self::Zip | Self::TarGz | Self::TarBz2 | Self::TarXz | Self::TarZst | Self::SevenZip
        )
    }

    /// Check if format supports password protection
    pub fn supports_password(&self) -> bool {
        matches!(self, Self::Zip | Self::Rar | Self::SevenZip)
    }
}

/// Compression level settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Store only, no compression
    Store,
    /// Fast compression, larger files
    Fast,
    /// Balanced compression
    Normal,
    /// Maximum compression, slower
    Best,
    /// Custom level (0-9)
    Custom(u32),
}

impl CompressionLevel {
    /// Convert to numeric level (0-9)
    pub fn to_level(&self) -> u32 {
        match self {
            Self::Store => 0,
            Self::Fast => 1,
            Self::Normal => 6,
            Self::Best => 9,
            Self::Custom(level) => (*level).min(9),
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Normal
    }
}

/// Entry in an archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    /// Path within the archive
    pub path: PathBuf,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Uncompressed size in bytes
    pub size: u64,
    /// Compressed size in bytes (if available)
    pub compressed_size: Option<u64>,
    /// Last modification time
    pub modified: Option<DateTime<Utc>>,
    /// Unix permissions (if available)
    pub permissions: Option<u32>,
    /// CRC32 checksum (if available)
    pub crc32: Option<u32>,
    /// Is the entry encrypted
    pub encrypted: bool,
}

impl ArchiveEntry {
    /// Get compression ratio as percentage
    pub fn compression_ratio(&self) -> Option<f64> {
        self.compressed_size.map(|compressed| {
            if self.size == 0 {
                100.0
            } else {
                (1.0 - (compressed as f64 / self.size as f64)) * 100.0
            }
        })
    }
}

/// Archive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    /// Archive file path
    pub path: PathBuf,
    /// Archive format
    pub format: ArchiveFormat,
    /// Total uncompressed size
    pub total_size: u64,
    /// Total compressed size (archive file size)
    pub compressed_size: u64,
    /// Number of files
    pub file_count: usize,
    /// Number of directories
    pub dir_count: usize,
    /// Whether the archive is password protected
    pub encrypted: bool,
    /// Archive comment (if any)
    pub comment: Option<String>,
}

impl ArchiveInfo {
    /// Overall compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.total_size == 0 {
            100.0
        } else {
            (1.0 - (self.compressed_size as f64 / self.total_size as f64)) * 100.0
        }
    }
}

/// Progress callback type
pub type ProgressCallback = Arc<dyn Fn(ArchiveProgress) + Send + Sync>;

/// Progress information during archive operations
#[derive(Debug, Clone)]
pub struct ArchiveProgress {
    /// Current operation
    pub operation: ArchiveOperation,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Current file index (1-based)
    pub current_index: usize,
    /// Total number of files
    pub total_files: usize,
    /// Bytes processed
    pub bytes_processed: u64,
    /// Total bytes to process
    pub total_bytes: u64,
}

impl ArchiveProgress {
    /// Get progress percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            if self.total_files == 0 {
                100.0
            } else {
                (self.current_index as f64 / self.total_files as f64) * 100.0
            }
        } else {
            (self.bytes_processed as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

/// Archive operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveOperation {
    Extract,
    Compress,
    List,
    Test,
}

/// Options for extraction
#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    /// Destination directory
    pub destination: PathBuf,
    /// Preserve file permissions
    pub preserve_permissions: bool,
    /// Overwrite existing files
    pub overwrite: bool,
    /// Extract specific paths only (empty = all)
    pub paths: Vec<PathBuf>,
    /// Password for encrypted archives
    pub password: Option<String>,
    /// Progress callback
    pub progress: Option<ProgressCallback>,
}

/// Options for compression
#[derive(Debug, Clone)]
pub struct CompressOptions {
    /// Output archive path
    pub output: PathBuf,
    /// Archive format
    pub format: ArchiveFormat,
    /// Compression level
    pub level: CompressionLevel,
    /// Password for encryption (if supported)
    pub password: Option<String>,
    /// Archive comment
    pub comment: Option<String>,
    /// Progress callback
    pub progress: Option<ProgressCallback>,
}

/// Archive manager for handling compressed files
pub struct ArchiveManager;

impl ArchiveManager {
    /// Create a new archive manager
    pub fn new() -> Self {
        Self
    }

    /// List contents of an archive
    pub fn list_contents(&self, path: &Path, password: Option<&str>) -> Result<Vec<ArchiveEntry>> {
        let format = ArchiveFormat::from_path(path)
            .ok_or_else(|| anyhow!("Unsupported archive format: {}", path.display()))?;

        match format {
            ArchiveFormat::Zip => self.list_zip(path, password),
            ArchiveFormat::Tar => self.list_tar(path),
            ArchiveFormat::TarGz => self.list_tar_gz(path),
            ArchiveFormat::TarBz2 => self.list_tar_bz2(path),
            ArchiveFormat::TarXz => self.list_tar_xz(path),
            ArchiveFormat::TarZst => self.list_tar_zst(path),
            ArchiveFormat::Rar => self.list_rar(path, password),
            ArchiveFormat::SevenZip => self.list_7z(path, password),
        }
    }

    /// Get archive information
    pub fn get_info(&self, path: &Path, password: Option<&str>) -> Result<ArchiveInfo> {
        let format = ArchiveFormat::from_path(path)
            .ok_or_else(|| anyhow!("Unsupported archive format: {}", path.display()))?;

        let entries = self.list_contents(path, password)?;
        let metadata = fs::metadata(path)?;

        let mut total_size = 0u64;
        let mut file_count = 0usize;
        let mut dir_count = 0usize;
        let mut encrypted = false;

        for entry in &entries {
            if entry.is_dir {
                dir_count += 1;
            } else {
                file_count += 1;
                total_size += entry.size;
            }
            if entry.encrypted {
                encrypted = true;
            }
        }

        let comment = match format {
            ArchiveFormat::Zip => self.get_zip_comment(path).ok(),
            _ => None,
        };

        Ok(ArchiveInfo {
            path: path.to_path_buf(),
            format,
            total_size,
            compressed_size: metadata.len(),
            file_count,
            dir_count,
            encrypted,
            comment,
        })
    }

    /// Extract an archive
    pub fn extract(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let format = ArchiveFormat::from_path(path)
            .ok_or_else(|| anyhow!("Unsupported archive format: {}", path.display()))?;

        // Create destination directory if it doesn't exist
        fs::create_dir_all(&options.destination)?;

        match format {
            ArchiveFormat::Zip => self.extract_zip(path, options),
            ArchiveFormat::Tar => self.extract_tar(path, options),
            ArchiveFormat::TarGz => self.extract_tar_gz(path, options),
            ArchiveFormat::TarBz2 => self.extract_tar_bz2(path, options),
            ArchiveFormat::TarXz => self.extract_tar_xz(path, options),
            ArchiveFormat::TarZst => self.extract_tar_zst(path, options),
            ArchiveFormat::Rar => self.extract_rar(path, options),
            ArchiveFormat::SevenZip => self.extract_7z(path, options),
        }
    }

    /// Create a compressed archive
    pub fn compress(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        match options.format {
            ArchiveFormat::Zip => self.compress_zip(paths, options),
            ArchiveFormat::Tar => self.compress_tar(paths, options),
            ArchiveFormat::TarGz => self.compress_tar_gz(paths, options),
            ArchiveFormat::TarBz2 => self.compress_tar_bz2(paths, options),
            ArchiveFormat::TarXz => self.compress_tar_xz(paths, options),
            ArchiveFormat::TarZst => self.compress_tar_zst(paths, options),
            ArchiveFormat::Rar => Err(anyhow!("RAR compression is not supported (read-only)")),
            ArchiveFormat::SevenZip => self.compress_7z(paths, options),
        }
    }

    // ========== ZIP Operations ==========

    fn list_zip(&self, path: &Path, _password: Option<&str>) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let archive = zip::ZipArchive::new(reader)?;

        let mut entries = Vec::new();
        for i in 0..archive.len() {
            let file = archive.by_index_raw(i)?;

            let entry = ArchiveEntry {
                path: PathBuf::from(file.name()),
                is_dir: file.is_dir(),
                size: file.size(),
                compressed_size: Some(file.compressed_size()),
                modified: file.last_modified().and_then(|dt| {
                    chrono::NaiveDate::from_ymd_opt(
                        dt.year() as i32,
                        dt.month() as u32,
                        dt.day() as u32,
                    )
                    .and_then(|date| {
                        date.and_hms_opt(dt.hour() as u32, dt.minute() as u32, dt.second() as u32)
                    })
                    .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
                }),
                permissions: file.unix_mode(),
                crc32: Some(file.crc32()),
                encrypted: file.encrypted(),
            };
            entries.push(entry);
        }

        Ok(entries)
    }

    fn get_zip_comment(&self, path: &Path) -> Result<String> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let archive = zip::ZipArchive::new(reader)?;

        let comment = archive.comment();
        if comment.is_empty() {
            Err(anyhow!("No comment"))
        } else {
            Ok(String::from_utf8_lossy(comment).to_string())
        }
    }

    fn extract_zip(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut archive = zip::ZipArchive::new(reader)?;

        let total_files = archive.len();
        let mut bytes_processed = 0u64;
        let total_bytes: u64 = (0..archive.len())
            .filter_map(|i| archive.by_index_raw(i).ok())
            .map(|f| f.size())
            .sum();

        for i in 0..archive.len() {
            let mut file = if let Some(ref pwd) = options.password {
                archive.by_index_decrypt(i, pwd.as_bytes())??
            } else {
                archive.by_index(i)?
            };

            let name = file.name().to_string();

            // Check if we should extract this file
            if !options.paths.is_empty() {
                let file_path = PathBuf::from(&name);
                if !options.paths.iter().any(|p| file_path.starts_with(p)) {
                    continue;
                }
            }

            let out_path = options.destination.join(&name);

            // Report progress
            if let Some(ref callback) = options.progress {
                callback(ArchiveProgress {
                    operation: ArchiveOperation::Extract,
                    current_file: Some(name.clone()),
                    current_index: i + 1,
                    total_files,
                    bytes_processed,
                    total_bytes,
                });
            }

            if file.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Check if file exists
                if out_path.exists() && !options.overwrite {
                    continue;
                }

                let mut out_file = File::create(&out_path)?;
                io::copy(&mut file, &mut out_file)?;

                // Preserve permissions on Unix
                #[cfg(unix)]
                if options.preserve_permissions {
                    if let Some(mode) = file.unix_mode() {
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
                    }
                }

                bytes_processed += file.size();
            }
        }

        Ok(())
    }

    fn compress_zip(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        let file = File::create(&options.output)?;
        let writer = BufWriter::new(file);
        let mut archive = zip::ZipWriter::new(writer);

        let zip_options = zip::write::SimpleFileOptions::default()
            .compression_method(match options.level {
                CompressionLevel::Store => zip::CompressionMethod::Stored,
                _ => zip::CompressionMethod::Deflated,
            })
            .compression_level(Some(options.level.to_level() as i64));

        // Collect all files to compress
        let files_to_compress = self.collect_files(paths)?;
        let total_files = files_to_compress.len();
        let total_bytes: u64 = files_to_compress
            .iter()
            .filter_map(|(p, _)| fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();

        let mut bytes_processed = 0u64;

        for (i, (file_path, archive_name)) in files_to_compress.iter().enumerate() {
            if let Some(ref callback) = options.progress {
                callback(ArchiveProgress {
                    operation: ArchiveOperation::Compress,
                    current_file: Some(archive_name.clone()),
                    current_index: i + 1,
                    total_files,
                    bytes_processed,
                    total_bytes,
                });
            }

            let metadata = fs::metadata(file_path)?;

            if metadata.is_dir() {
                archive.add_directory(archive_name, zip_options)?;
            } else {
                archive.start_file(archive_name, zip_options)?;
                let mut file = File::open(file_path)?;
                io::copy(&mut file, &mut archive)?;
                bytes_processed += metadata.len();
            }
        }

        if let Some(ref comment) = options.comment {
            archive.set_comment(comment);
        }

        archive.finish()?;
        Ok(())
    }

    // ========== TAR Operations ==========

    fn list_tar(&self, path: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.list_tar_from_reader(reader)
    }

    fn list_tar_gz(&self, path: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = flate2::read::GzDecoder::new(reader);
        self.list_tar_from_reader(decoder)
    }

    fn list_tar_bz2(&self, path: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = bzip2::read::BzDecoder::new(reader);
        self.list_tar_from_reader(decoder)
    }

    fn list_tar_xz(&self, path: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = xz2::read::XzDecoder::new(reader);
        self.list_tar_from_reader(decoder)
    }

    fn list_tar_zst(&self, path: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = zstd::stream::Decoder::new(reader)?;
        self.list_tar_from_reader(decoder)
    }

    fn list_tar_from_reader<R: Read>(&self, reader: R) -> Result<Vec<ArchiveEntry>> {
        let mut archive = tar::Archive::new(reader);
        let mut entries = Vec::new();

        for entry in archive.entries()? {
            let entry = entry?;
            let header = entry.header();

            let archive_entry = ArchiveEntry {
                path: entry.path()?.to_path_buf(),
                is_dir: header.entry_type().is_dir(),
                size: header.size()?,
                compressed_size: None,
                modified: header.mtime().ok().map(|t| {
                    DateTime::from_timestamp(t as i64, 0).unwrap_or_default()
                }),
                permissions: header.mode().ok(),
                crc32: None,
                encrypted: false,
            };
            entries.push(archive_entry);
        }

        Ok(entries)
    }

    fn extract_tar(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.extract_tar_from_reader(reader, options)
    }

    fn extract_tar_gz(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = flate2::read::GzDecoder::new(reader);
        self.extract_tar_from_reader(decoder, options)
    }

    fn extract_tar_bz2(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = bzip2::read::BzDecoder::new(reader);
        self.extract_tar_from_reader(decoder, options)
    }

    fn extract_tar_xz(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = xz2::read::XzDecoder::new(reader);
        self.extract_tar_from_reader(decoder, options)
    }

    fn extract_tar_zst(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let decoder = zstd::stream::Decoder::new(reader)?;
        self.extract_tar_from_reader(decoder, options)
    }

    fn extract_tar_from_reader<R: Read>(&self, reader: R, options: &ExtractOptions) -> Result<()> {
        let mut archive = tar::Archive::new(reader);
        archive.set_preserve_permissions(options.preserve_permissions);
        archive.set_overwrite(options.overwrite);

        archive.unpack(&options.destination)?;
        Ok(())
    }

    fn compress_tar(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        let file = File::create(&options.output)?;
        let writer = BufWriter::new(file);
        self.compress_tar_to_writer(paths, options, writer)
    }

    fn compress_tar_gz(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        let file = File::create(&options.output)?;
        let writer = BufWriter::new(file);
        let encoder = flate2::write::GzEncoder::new(writer,
            flate2::Compression::new(options.level.to_level()));
        self.compress_tar_to_writer(paths, options, encoder)?;
        Ok(())
    }

    fn compress_tar_bz2(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        let file = File::create(&options.output)?;
        let writer = BufWriter::new(file);
        let encoder = bzip2::write::BzEncoder::new(writer,
            bzip2::Compression::new(options.level.to_level()));
        self.compress_tar_to_writer(paths, options, encoder)?;
        Ok(())
    }

    fn compress_tar_xz(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        let file = File::create(&options.output)?;
        let writer = BufWriter::new(file);
        let encoder = xz2::write::XzEncoder::new(writer, options.level.to_level());
        self.compress_tar_to_writer(paths, options, encoder)?;
        Ok(())
    }

    fn compress_tar_zst(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        let file = File::create(&options.output)?;
        let writer = BufWriter::new(file);
        let encoder = zstd::stream::Encoder::new(writer, options.level.to_level() as i32)?
            .auto_finish();
        self.compress_tar_to_writer(paths, options, encoder)?;
        Ok(())
    }

    fn compress_tar_to_writer<W: Write>(&self, paths: &[PathBuf], options: &CompressOptions, writer: W) -> Result<()> {
        let mut archive = tar::Builder::new(writer);

        let files_to_compress = self.collect_files(paths)?;
        let total_files = files_to_compress.len();
        let total_bytes: u64 = files_to_compress
            .iter()
            .filter_map(|(p, _)| fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();

        let mut bytes_processed = 0u64;

        for (i, (file_path, archive_name)) in files_to_compress.iter().enumerate() {
            if let Some(ref callback) = options.progress {
                callback(ArchiveProgress {
                    operation: ArchiveOperation::Compress,
                    current_file: Some(archive_name.clone()),
                    current_index: i + 1,
                    total_files,
                    bytes_processed,
                    total_bytes,
                });
            }

            let metadata = fs::metadata(file_path)?;

            if metadata.is_dir() {
                archive.append_dir(archive_name, file_path)?;
            } else {
                let mut file = File::open(file_path)?;
                archive.append_file(archive_name, &mut file)?;
                bytes_processed += metadata.len();
            }
        }

        archive.finish()?;
        Ok(())
    }

    // ========== RAR Operations (read-only) ==========

    #[cfg(feature = "rar")]
    fn list_rar(&self, path: &Path, _password: Option<&str>) -> Result<Vec<ArchiveEntry>> {
        let archive = unrar::Archive::new(path)
            .open_for_listing()
            .map_err(|e| anyhow!("Failed to open RAR archive: {}", e))?;

        let mut entries = Vec::new();
        for entry in archive {
            let entry = entry.map_err(|e| anyhow!("Failed to read RAR entry: {}", e))?;

            let archive_entry = ArchiveEntry {
                path: entry.filename.clone(),
                is_dir: entry.is_directory(),
                size: entry.unpacked_size as u64,
                compressed_size: Some(entry.packed_size as u64),
                modified: None,
                permissions: None,
                crc32: Some(entry.crc),
                encrypted: entry.is_encrypted(),
            };
            entries.push(archive_entry);
        }

        Ok(entries)
    }

    #[cfg(not(feature = "rar"))]
    fn list_rar(&self, _path: &Path, _password: Option<&str>) -> Result<Vec<ArchiveEntry>> {
        Err(anyhow!("RAR support is not enabled. Compile with --features rar"))
    }

    #[cfg(feature = "rar")]
    fn extract_rar(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let mut archive = if let Some(ref pwd) = options.password {
            unrar::Archive::new(path)
                .open_for_processing_with_password(pwd)
                .map_err(|e| anyhow!("Failed to open RAR archive: {}", e))?
        } else {
            unrar::Archive::new(path)
                .open_for_processing()
                .map_err(|e| anyhow!("Failed to open RAR archive: {}", e))?
        };

        while let Some(header) = archive.read_header().map_err(|e| anyhow!("RAR error: {}", e))? {
            archive = if header.entry().is_file() {
                header.extract_to(&options.destination)
                    .map_err(|e| anyhow!("Failed to extract: {}", e))?
            } else {
                header.skip().map_err(|e| anyhow!("Failed to skip: {}", e))?
            };
        }

        Ok(())
    }

    #[cfg(not(feature = "rar"))]
    fn extract_rar(&self, _path: &Path, _options: &ExtractOptions) -> Result<()> {
        Err(anyhow!("RAR support is not enabled. Compile with --features rar"))
    }

    // ========== 7-Zip Operations ==========

    fn list_7z(&self, path: &Path, _password: Option<&str>) -> Result<Vec<ArchiveEntry>> {
        let mut archive = sevenz_rust::SevenZReader::open(path, sevenz_rust::Password::empty())
            .context("Failed to open 7z archive")?;

        let mut entries = Vec::new();

        archive.for_each_entries(|entry, _| {
            let archive_entry = ArchiveEntry {
                path: PathBuf::from(entry.name()),
                is_dir: entry.is_directory(),
                size: entry.size(),
                compressed_size: entry.compressed_size().map(|s| s as u64),
                modified: entry.last_modified_date().map(|t| {
                    DateTime::from_timestamp(t.timestamp(), 0).unwrap_or_default()
                }),
                permissions: None,
                crc32: entry.crc().map(|c| c as u32),
                encrypted: entry.has_stream() && entry.compressed_size().is_none(),
            };
            entries.push(archive_entry);
            Ok(true)
        })?;

        Ok(entries)
    }

    fn extract_7z(&self, path: &Path, options: &ExtractOptions) -> Result<()> {
        let password = options.password.as_deref()
            .map(|p| sevenz_rust::Password::from(p))
            .unwrap_or(sevenz_rust::Password::empty());

        sevenz_rust::decompress_file_with_password(path, &options.destination, password)
            .context("Failed to extract 7z archive")?;

        Ok(())
    }

    fn compress_7z(&self, paths: &[PathBuf], options: &CompressOptions) -> Result<()> {
        let mut encoder = sevenz_rust::SevenZWriter::create(&options.output)
            .context("Failed to create 7z archive")?;

        let files_to_compress = self.collect_files(paths)?;

        for (file_path, archive_name) in files_to_compress {
            let metadata = fs::metadata(&file_path)?;

            if metadata.is_dir() {
                // Create directory entry
                let entry = sevenz_rust::SevenZArchiveEntry::directory(archive_name);
                encoder.push_archive_entry(entry, None::<&mut std::io::Empty>)?;
            } else {
                let mut file = File::open(&file_path)?;
                let entry = sevenz_rust::SevenZArchiveEntry::file(archive_name.clone());
                encoder.push_archive_entry(entry, Some(&mut file))?;
            }
        }

        encoder.finish()?;
        Ok(())
    }

    // ========== Helper Functions ==========

    /// Collect all files to compress, returning (full_path, archive_name) pairs
    fn collect_files(&self, paths: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
        let mut files = Vec::new();

        for path in paths {
            if path.is_dir() {
                self.collect_dir_recursive(path, path.parent(), &mut files)?;
            } else {
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                files.push((path.clone(), name));
            }
        }

        Ok(files)
    }

    fn collect_dir_recursive(
        &self,
        dir: &Path,
        base: Option<&Path>,
        files: &mut Vec<(PathBuf, String)>,
    ) -> Result<()> {
        let base = base.unwrap_or(dir);

        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();

            let relative = path.strip_prefix(base)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if !relative.is_empty() {
                files.push((path.to_path_buf(), relative));
            }
        }

        Ok(())
    }
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ArchiveFormat::from_path(Path::new("file.zip")),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("file.tar.gz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("file.tgz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("file.7z")),
            Some(ArchiveFormat::SevenZip)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("file.txt")),
            None
        );
    }

    #[test]
    fn test_compression_level() {
        assert_eq!(CompressionLevel::Store.to_level(), 0);
        assert_eq!(CompressionLevel::Fast.to_level(), 1);
        assert_eq!(CompressionLevel::Normal.to_level(), 6);
        assert_eq!(CompressionLevel::Best.to_level(), 9);
        assert_eq!(CompressionLevel::Custom(5).to_level(), 5);
        assert_eq!(CompressionLevel::Custom(15).to_level(), 9); // Capped at 9
    }

    #[test]
    fn test_progress_percentage() {
        let progress = ArchiveProgress {
            operation: ArchiveOperation::Extract,
            current_file: Some("test.txt".to_string()),
            current_index: 5,
            total_files: 10,
            bytes_processed: 500,
            total_bytes: 1000,
        };

        assert!((progress.percentage() - 50.0).abs() < 0.001);
    }
}
