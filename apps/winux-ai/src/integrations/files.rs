// File Manager - File system operations and analysis

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::fs;

pub struct FileManager;

impl FileManager {
    /// Read file content with size limit
    pub fn read_file(path: &Path, max_size: Option<u64>) -> Result<String> {
        let max_size = max_size.unwrap_or(1_000_000); // 1MB default
        let metadata = fs::metadata(path)?;

        if metadata.len() > max_size {
            let content = fs::read_to_string(path)?;
            let truncated: String = content.chars().take(max_size as usize).collect();
            Ok(format!("{}\n\n[... truncated, file size: {} bytes ...]", truncated, metadata.len()))
        } else {
            fs::read_to_string(path).map_err(|e| anyhow!("Failed to read file: {}", e))
        }
    }

    /// Get file info
    pub fn get_file_info(path: &Path) -> Result<FileInfo> {
        let metadata = fs::metadata(path)?;
        let mime = mime_guess::from_path(path).first_or_octet_stream();

        Ok(FileInfo {
            path: path.to_path_buf(),
            name: path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            extension: path.extension()
                .map(|e| e.to_string_lossy().to_string()),
            size_bytes: metadata.len(),
            is_dir: metadata.is_dir(),
            is_symlink: metadata.is_symlink(),
            mime_type: mime.to_string(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
        })
    }

    /// List directory contents
    pub fn list_directory(path: &Path, max_entries: Option<usize>) -> Result<Vec<FileInfo>> {
        let max_entries = max_entries.unwrap_or(100);
        let mut entries = Vec::new();

        for entry in fs::read_dir(path)?.take(max_entries) {
            if let Ok(entry) = entry {
                if let Ok(info) = Self::get_file_info(&entry.path()) {
                    entries.push(info);
                }
            }
        }

        // Sort: directories first, then by name
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(entries)
    }

    /// Search for files by pattern
    pub fn search_files(dir: &Path, pattern: &str, max_results: usize) -> Result<Vec<PathBuf>> {
        let pattern = pattern.to_lowercase();
        let mut results = Vec::new();

        Self::search_recursive(dir, &pattern, &mut results, max_results)?;
        Ok(results)
    }

    fn search_recursive(dir: &Path, pattern: &str, results: &mut Vec<PathBuf>, max_results: usize) -> Result<()> {
        if results.len() >= max_results {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                if name.contains(pattern) {
                    results.push(path.clone());
                    if results.len() >= max_results {
                        return Ok(());
                    }
                }

                if path.is_dir() && !name.starts_with('.') {
                    let _ = Self::search_recursive(&path, pattern, results, max_results);
                }
            }
        }

        Ok(())
    }

    /// Get recent files in a directory
    pub fn get_recent_files(dir: &Path, limit: usize) -> Result<Vec<FileInfo>> {
        let mut files: Vec<FileInfo> = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| Self::get_file_info(&e.path()).ok())
            .filter(|f| !f.is_dir)
            .collect();

        // Sort by modified time (most recent first)
        files.sort_by(|a, b| {
            match (&b.modified, &a.modified) {
                (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        files.truncate(limit);
        Ok(files)
    }

    /// Check if file is a text file
    pub fn is_text_file(path: &Path) -> bool {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let mime_str = mime.as_ref();

        mime_str.starts_with("text/")
            || mime_str == "application/json"
            || mime_str == "application/xml"
            || mime_str == "application/javascript"
            || mime_str == "application/x-sh"
            || mime_str == "application/toml"
            || mime_str == "application/yaml"
    }

    /// Check if file is an image
    pub fn is_image_file(path: &Path) -> bool {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        mime.as_ref().starts_with("image/")
    }

    /// Get file content preview
    pub fn get_preview(path: &Path, lines: usize) -> Result<String> {
        let content = fs::read_to_string(path)?;
        let preview: String = content
            .lines()
            .take(lines)
            .collect::<Vec<_>>()
            .join("\n");

        if content.lines().count() > lines {
            Ok(format!("{}\n\n[... {} more lines ...]", preview, content.lines().count() - lines))
        } else {
            Ok(preview)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub mime_type: String,
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

impl FileInfo {
    pub fn size_human_readable(&self) -> String {
        let bytes = self.size_bytes as f64;
        if bytes < 1024.0 {
            format!("{} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
