// Summarizer - Summarize texts and documents

use crate::ai::{AzureOpenAIClient, SystemPrompts};
use anyhow::{anyhow, Result};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

pub struct Summarizer {
    client: Arc<AzureOpenAIClient>,
}

#[derive(Debug, Clone, Copy)]
pub enum SummaryLength {
    Brief,    // 1-2 sentences
    Short,    // 1 paragraph
    Medium,   // 2-3 paragraphs
    Detailed, // Comprehensive with sections
}

impl Summarizer {
    pub fn new(client: Arc<AzureOpenAIClient>) -> Self {
        Self { client }
    }

    /// Summarize text
    pub async fn summarize(&self, text: &str, length: SummaryLength) -> Result<String> {
        let length_instruction = match length {
            SummaryLength::Brief => "Provide a very brief summary in 1-2 sentences.",
            SummaryLength::Short => "Provide a short summary in one paragraph.",
            SummaryLength::Medium => "Provide a medium-length summary in 2-3 paragraphs.",
            SummaryLength::Detailed => "Provide a detailed summary with sections and bullet points.",
        };

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::summarizer()
            }),
            json!({
                "role": "user",
                "content": format!("{}\n\nText to summarize:\n\n{}", length_instruction, text)
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Summarize with specific focus
    pub async fn summarize_with_focus(&self, text: &str, focus: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::summarizer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Summarize the following text, focusing specifically on: {}\n\nText:\n\n{}",
                    focus, text
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Extract key points as bullet list
    pub async fn extract_key_points(&self, text: &str, max_points: Option<usize>) -> Result<String> {
        let limit = max_points
            .map(|n| format!("List up to {} key points.", n))
            .unwrap_or_else(|| "List all key points.".to_string());

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::summarizer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Extract the key points from this text as a bullet list. {}\n\nText:\n\n{}",
                    limit, text
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Summarize a file
    pub async fn summarize_file(&self, path: &Path, length: SummaryLength) -> Result<String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;

        // Truncate if too long
        let content = if content.len() > 50000 {
            format!("{}...\n\n[Content truncated due to length]", &content[..50000])
        } else {
            content
        };

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let length_instruction = match length {
            SummaryLength::Brief => "Provide a very brief summary in 1-2 sentences.",
            SummaryLength::Short => "Provide a short summary in one paragraph.",
            SummaryLength::Medium => "Provide a medium-length summary in 2-3 paragraphs.",
            SummaryLength::Detailed => "Provide a detailed summary with sections and bullet points.",
        };

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::summarizer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "{}\n\nFile: {}\n\nContent:\n\n{}",
                    length_instruction, file_name, content
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Create an abstract for academic/technical content
    pub async fn create_abstract(&self, text: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::summarizer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Create a formal abstract (150-250 words) for the following content:\n\n{}",
                    text
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Generate TL;DR
    pub async fn tldr(&self, text: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": "Provide extremely concise TL;DR summaries. Maximum 2 sentences."
            }),
            json!({
                "role": "user",
                "content": format!("TL;DR:\n\n{}", text)
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Summarize conversation/meeting notes
    pub async fn summarize_conversation(&self, text: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::summarizer()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Summarize this conversation/meeting. Include:\n- Main topics discussed\n- Key decisions made\n- Action items\n- Open questions\n\nConversation:\n\n{}",
                    text
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }
}
