//! Archive format definitions and detection

use std::path::Path;

/// Supported archive formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// ZIP archive
    Zip,
    /// RAR archive
    Rar,
    /// Plain TAR archive
    Tar,
    /// Gzip compressed TAR
    TarGz,
    /// Bzip2 compressed TAR
    TarBz2,
    /// XZ compressed TAR
    TarXz,
    /// 7-Zip archive
    SevenZip,
    /// ISO disc image
    Iso,
    /// Zstandard compressed
    Zstd,
}

impl ArchiveFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        let path_str = path.to_string_lossy().to_lowercase();

        // Check compound extensions first
        if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
            return Some(Self::TarGz);
        }
        if path_str.ends_with(".tar.bz2") || path_str.ends_with(".tbz2") || path_str.ends_with(".tbz") {
            return Some(Self::TarBz2);
        }
        if path_str.ends_with(".tar.xz") || path_str.ends_with(".txz") {
            return Some(Self::TarXz);
        }
        if path_str.ends_with(".tar.zst") || path_str.ends_with(".tar.zstd") {
            return Some(Self::Zstd);
        }

        // Check simple extensions
        let extension = path.extension()?.to_string_lossy().to_lowercase();

        match extension.as_str() {
            "zip" | "zipx" | "jar" | "war" | "ear" | "apk" | "ipa" | "epub" => Some(Self::Zip),
            "rar" | "cbr" => Some(Self::Rar),
            "tar" => Some(Self::Tar),
            "gz" | "gzip" => Some(Self::TarGz),
            "bz2" | "bzip2" => Some(Self::TarBz2),
            "xz" | "lzma" => Some(Self::TarXz),
            "7z" => Some(Self::SevenZip),
            "iso" | "img" | "nrg" | "mdf" => Some(Self::Iso),
            "zst" | "zstd" => Some(Self::Zstd),
            _ => None,
        }
    }

    /// Detect format from file magic bytes
    pub fn from_magic(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        // ZIP magic: PK\x03\x04 or PK\x05\x06 (empty) or PK\x07\x08
        if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
            return Some(Self::Zip);
        }

        // RAR magic: Rar!\x1a\x07
        if bytes.starts_with(b"Rar!\x1a\x07") {
            return Some(Self::Rar);
        }

        // 7z magic: 7z\xbc\xaf\x27\x1c
        if bytes.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
            return Some(Self::SevenZip);
        }

        // GZip magic: \x1f\x8b
        if bytes.starts_with(&[0x1F, 0x8B]) {
            return Some(Self::TarGz);
        }

        // BZip2 magic: BZ
        if bytes.starts_with(b"BZ") {
            return Some(Self::TarBz2);
        }

        // XZ magic: \xfd7zXZ\x00
        if bytes.starts_with(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]) {
            return Some(Self::TarXz);
        }

        // Zstd magic: 0x28 0xB5 0x2F 0xFD
        if bytes.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
            return Some(Self::Zstd);
        }

        // ISO magic: CD001 at offset 32769
        // Note: This would need more bytes to check properly

        // TAR - check for ustar magic at offset 257
        if bytes.len() > 262 && &bytes[257..262] == b"ustar" {
            return Some(Self::Tar);
        }

        None
    }

    /// Get human-readable format name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Zip => "ZIP",
            Self::Rar => "RAR",
            Self::Tar => "TAR",
            Self::TarGz => "TAR.GZ",
            Self::TarBz2 => "TAR.BZ2",
            Self::TarXz => "TAR.XZ",
            Self::SevenZip => "7-Zip",
            Self::Iso => "ISO",
            Self::Zstd => "ZSTD",
        }
    }

    /// Get default file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::Rar => "rar",
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
            Self::TarBz2 => "tar.bz2",
            Self::TarXz => "tar.xz",
            Self::SevenZip => "7z",
            Self::Iso => "iso",
            Self::Zstd => "tar.zst",
        }
    }

    /// Check if format supports creating new archives
    pub fn supports_create(&self) -> bool {
        match self {
            Self::Zip | Self::Tar | Self::TarGz | Self::TarBz2 | Self::TarXz
            | Self::SevenZip | Self::Zstd => true,
            Self::Rar | Self::Iso => false,
        }
    }

    /// Check if format supports adding files
    pub fn supports_add(&self) -> bool {
        match self {
            Self::Zip | Self::SevenZip => true,
            _ => false,
        }
    }

    /// Check if format supports removing files
    pub fn supports_remove(&self) -> bool {
        match self {
            Self::Zip | Self::SevenZip => true,
            _ => false,
        }
    }

    /// Check if format supports encryption
    pub fn supports_encryption(&self) -> bool {
        match self {
            Self::Zip | Self::Rar | Self::SevenZip => true,
            _ => false,
        }
    }

    /// Check if format supports split volumes
    pub fn supports_split(&self) -> bool {
        match self {
            Self::Zip | Self::Rar | Self::SevenZip => true,
            _ => false,
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Zip => "application/zip",
            Self::Rar => "application/vnd.rar",
            Self::Tar => "application/x-tar",
            Self::TarGz => "application/gzip",
            Self::TarBz2 => "application/x-bzip2",
            Self::TarXz => "application/x-xz",
            Self::SevenZip => "application/x-7z-compressed",
            Self::Iso => "application/x-iso9660-image",
            Self::Zstd => "application/zstd",
        }
    }

    /// Get icon name for this format
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Zip => "application-zip",
            Self::Rar => "application-x-rar",
            Self::Tar | Self::TarGz | Self::TarBz2 | Self::TarXz => "application-x-tar",
            Self::SevenZip => "application-x-7z-compressed",
            Self::Iso => "application-x-cd-image",
            Self::Zstd => "application-x-compressed-tar",
        }
    }
}

/// Compression methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    /// No compression (store only)
    Store,
    /// Deflate (ZIP, GZ)
    Deflate,
    /// Deflate64
    Deflate64,
    /// BZip2
    BZip2,
    /// LZMA
    Lzma,
    /// LZMA2
    Lzma2,
    /// XZ
    Xz,
    /// PPMd
    Ppmd,
    /// Zstandard
    Zstd,
    /// Unknown method
    Unknown(u16),
}

impl CompressionMethod {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Store => "Store",
            Self::Deflate => "Deflate",
            Self::Deflate64 => "Deflate64",
            Self::BZip2 => "BZip2",
            Self::Lzma => "LZMA",
            Self::Lzma2 => "LZMA2",
            Self::Xz => "XZ",
            Self::Ppmd => "PPMd",
            Self::Zstd => "Zstandard",
            Self::Unknown(_) => "Unknown",
        }
    }

    /// Create from ZIP compression method code
    pub fn from_zip_method(method: u16) -> Self {
        match method {
            0 => Self::Store,
            8 => Self::Deflate,
            9 => Self::Deflate64,
            12 => Self::BZip2,
            14 => Self::Lzma,
            93 => Self::Zstd,
            98 => Self::Ppmd,
            _ => Self::Unknown(method),
        }
    }
}

/// Compression level (1-9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionLevel(u8);

impl CompressionLevel {
    pub const FASTEST: Self = Self(1);
    pub const FAST: Self = Self(3);
    pub const NORMAL: Self = Self(5);
    pub const MAXIMUM: Self = Self(7);
    pub const ULTRA: Self = Self(9);

    pub fn new(level: u8) -> Self {
        Self(level.clamp(1, 9))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::NORMAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.zip")),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tar.gz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.7z")),
            Some(ArchiveFormat::SevenZip)
        );
    }

    #[test]
    fn test_magic_detection() {
        assert_eq!(
            ArchiveFormat::from_magic(b"PK\x03\x04test"),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(
            ArchiveFormat::from_magic(b"Rar!\x1a\x07"),
            Some(ArchiveFormat::Rar)
        );
    }
}
