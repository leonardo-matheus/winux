//! File search provider

use crate::config::Config;
use crate::search::{SearchCategory, SearchResult, SearchResultKind};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tracing::{debug, warn};

/// File search provider using locate/mlocate
pub struct FileSearcher {
    config: Arc<Config>,
    matcher: SkimMatcherV2,
    use_locate: bool,
}

impl FileSearcher {
    /// Create new file searcher
    pub fn new(config: Arc<Config>) -> Self {
        // Check if locate is available
        let use_locate = Command::new("locate")
            .arg("--version")
            .output()
            .is_ok();

        if use_locate {
            debug!("Using locate for file search");
        } else {
            debug!("locate not available, using fallback search");
        }

        Self {
            config,
            matcher: SkimMatcherV2::default(),
            use_locate,
        }
    }

    /// Refresh file index (trigger updatedb if available)
    pub fn refresh(&mut self) {
        if self.use_locate {
            // Note: updatedb typically requires root, so we just log
            debug!("File index refresh requested (requires updatedb)");
        }
    }

    /// Search for files
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.len() < 2 {
            return vec![];
        }

        if self.use_locate {
            self.search_with_locate(query)
        } else {
            self.search_fallback(query)
        }
    }

    /// Search using locate command
    fn search_with_locate(&self, query: &str) -> Vec<SearchResult> {
        let output = Command::new("locate")
            .arg("-i") // Case insensitive
            .arg("-l")
            .arg("20") // Limit results
            .arg("--regex")
            .arg(format!(".*{}.*", regex::escape(query)))
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .filter(|line| self.is_valid_path(line))
                    .filter_map(|line| self.path_to_result(PathBuf::from(line), query))
                    .take(5)
                    .collect()
            }
            Err(e) => {
                warn!("locate command failed: {}", e);
                vec![]
            }
        }
    }

    /// Fallback search (basic file system search)
    fn search_fallback(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Search in home directory and common locations
        let search_paths = vec![
            dirs::home_dir(),
            dirs::document_dir(),
            dirs::download_dir(),
            dirs::desktop_dir(),
        ];

        for base_path in search_paths.into_iter().flatten() {
            if let Ok(entries) = std::fs::read_dir(&base_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Skip hidden files unless query starts with .
                    if name.starts_with('.') && !query.starts_with('.') {
                        continue;
                    }

                    if let Some(score) = self.matcher.fuzzy_match(&name.to_lowercase(), &query_lower) {
                        if score > 0 {
                            if let Some(result) = self.path_to_result(path, query) {
                                results.push((score, result));
                            }
                        }
                    }
                }
            }
        }

        // Sort by score and return top results
        results.sort_by(|a, b| b.0.cmp(&a.0));
        results.into_iter().take(5).map(|(_, r)| r).collect()
    }

    /// Check if path should be included in results
    fn is_valid_path(&self, path: &str) -> bool {
        let path = PathBuf::from(path);

        // Check excluded paths
        for excluded in &self.config.search.excluded_paths {
            if path.starts_with(excluded) {
                return false;
            }
        }

        // Check if path exists
        path.exists()
    }

    /// Convert path to search result
    fn path_to_result(&self, path: PathBuf, query: &str) -> Option<SearchResult> {
        let name = path.file_name()?.to_string_lossy().to_string();

        // Determine icon based on file type
        let icon = self.get_file_icon(&path);

        // Calculate score based on name match
        let score = self
            .matcher
            .fuzzy_match(&name.to_lowercase(), &query.to_lowercase())
            .unwrap_or(0)
            .min(100) as u32;

        // Get parent directory for subtitle
        let subtitle = path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "File".to_string());

        Some(SearchResult {
            id: format!("file:{}", path.display()),
            title: name,
            subtitle,
            icon,
            category: SearchCategory::Files,
            kind: SearchResultKind::File { path },
            score,
            from_history: false,
        })
    }

    /// Get appropriate icon for file type
    fn get_file_icon(&self, path: &PathBuf) -> String {
        if path.is_dir() {
            return "folder-symbolic".to_string();
        }

        // Get extension
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            // Documents
            "pdf" => "application-pdf-symbolic",
            "doc" | "docx" | "odt" => "x-office-document-symbolic",
            "xls" | "xlsx" | "ods" => "x-office-spreadsheet-symbolic",
            "ppt" | "pptx" | "odp" => "x-office-presentation-symbolic",
            "txt" | "md" | "rst" => "text-x-generic-symbolic",

            // Images
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" => "image-x-generic-symbolic",

            // Audio
            "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "audio-x-generic-symbolic",

            // Video
            "mp4" | "mkv" | "avi" | "mov" | "webm" => "video-x-generic-symbolic",

            // Archives
            "zip" | "tar" | "gz" | "xz" | "7z" | "rar" => "package-x-generic-symbolic",

            // Code
            "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go" | "rb" => {
                "text-x-script-symbolic"
            }

            // Executables
            "sh" | "bash" | "bin" | "exe" | "appimage" => "application-x-executable-symbolic",

            // Default
            _ => "text-x-generic-symbolic",
        }
        .to_string()
    }
}
