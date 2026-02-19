// File Analyzer - Analyze and understand files

use crate::ai::{AzureOpenAIClient, SystemPrompts};
use anyhow::{anyhow, Result};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

pub struct FileAnalyzer {
    client: Arc<AzureOpenAIClient>,
}

impl FileAnalyzer {
    pub fn new(client: Arc<AzureOpenAIClient>) -> Self {
        Self { client }
    }

    /// Analyze a file and provide insights
    pub async fn analyze(&self, path: &Path) -> Result<String> {
        let content = self.read_file_content(path)?;
        let file_info = self.get_file_info(path);

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::file_analyzer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Analyze this file:\n\n{}\n\nContent:\n```\n{}\n```",
                    file_info,
                    content
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Explain what a file does
    pub async fn explain(&self, path: &Path) -> Result<String> {
        let content = self.read_file_content(path)?;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::file_analyzer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Explain what this {} file does:\n\n```{}\n{}\n```",
                    extension,
                    extension,
                    content
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Find potential issues in a file
    pub async fn find_issues(&self, path: &Path) -> Result<String> {
        let content = self.read_file_content(path)?;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::file_analyzer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Find potential issues, bugs, or security concerns in this file:\n\n```{}\n{}\n```",
                    extension,
                    content
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Suggest improvements for a file
    pub async fn suggest_improvements(&self, path: &Path) -> Result<String> {
        let content = self.read_file_content(path)?;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::file_analyzer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Suggest improvements for this file:\n\n```{}\n{}\n```",
                    extension,
                    content
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Compare two files
    pub async fn compare(&self, path1: &Path, path2: &Path) -> Result<String> {
        let content1 = self.read_file_content(path1)?;
        let content2 = self.read_file_content(path2)?;

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::file_analyzer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Compare these two files and explain the differences:\n\nFile 1 ({}):\n```\n{}\n```\n\nFile 2 ({}):\n```\n{}\n```",
                    path1.display(),
                    content1,
                    path2.display(),
                    content2
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Read file content with size limit
    fn read_file_content(&self, path: &Path) -> Result<String> {
        const MAX_SIZE: u64 = 100_000; // 100KB limit

        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_SIZE {
            // Read only the first part of large files
            let content = std::fs::read_to_string(path)?;
            let truncated: String = content.chars().take(MAX_SIZE as usize).collect();
            return Ok(format!("{}\n\n[... truncated, file too large ...]", truncated));
        }

        std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file: {}", e))
    }

    /// Get file metadata info
    fn get_file_info(&self, path: &Path) -> String {
        let mut info = Vec::new();

        info.push(format!("File: {}", path.display()));

        if let Some(ext) = path.extension() {
            info.push(format!("Extension: .{}", ext.to_string_lossy()));
        }

        if let Ok(metadata) = std::fs::metadata(path) {
            info.push(format!("Size: {} bytes", metadata.len()));

            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    let datetime = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0);
                    if let Some(dt) = datetime {
                        info.push(format!("Modified: {}", dt.format("%Y-%m-%d %H:%M:%S")));
                    }
                }
            }
        }

        // Detect language/type
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        info.push(format!("Type: {}", mime));

        info.join("\n")
    }
}
