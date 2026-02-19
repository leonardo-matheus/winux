// Translator - Language translation capabilities

use crate::ai::{AzureOpenAIClient, SystemPrompts};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

pub struct Translator {
    client: Arc<AzureOpenAIClient>,
}

impl Translator {
    pub fn new(client: Arc<AzureOpenAIClient>) -> Self {
        Self { client }
    }

    /// Translate text to a target language
    pub async fn translate(&self, text: &str, target_language: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::translator()
            }),
            json!({
                "role": "user",
                "content": format!("Translate the following text to {}:\n\n{}", target_language, text)
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Translate with source language specified
    pub async fn translate_from(&self, text: &str, source_language: &str, target_language: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::translator()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Translate the following text from {} to {}:\n\n{}",
                    source_language, target_language, text
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Detect language and optionally translate
    pub async fn detect_language(&self, text: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::translator()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Detect the language of this text and provide your confidence level:\n\n{}",
                    text
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Translate and explain nuances
    pub async fn translate_with_context(&self, text: &str, target_language: &str, context: Option<&str>) -> Result<String> {
        let context_info = context
            .map(|c| format!("\n\nContext: {}", c))
            .unwrap_or_default();

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::translator()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Translate to {} and explain any cultural nuances or alternative translations:{}\n\nText:\n{}",
                    target_language, context_info, text
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Translate code comments and documentation
    pub async fn translate_code_docs(&self, code: &str, target_language: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::translator()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Translate the comments and documentation strings in this code to {}. Keep the code itself unchanged:\n\n```\n{}\n```",
                    target_language, code
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// List supported languages
    pub fn supported_languages() -> Vec<(&'static str, &'static str)> {
        vec![
            ("en", "English"),
            ("pt", "Portuguese"),
            ("es", "Spanish"),
            ("fr", "French"),
            ("de", "German"),
            ("it", "Italian"),
            ("nl", "Dutch"),
            ("ru", "Russian"),
            ("zh", "Chinese"),
            ("ja", "Japanese"),
            ("ko", "Korean"),
            ("ar", "Arabic"),
            ("hi", "Hindi"),
            ("tr", "Turkish"),
            ("pl", "Polish"),
            ("vi", "Vietnamese"),
            ("th", "Thai"),
            ("id", "Indonesian"),
            ("sv", "Swedish"),
            ("da", "Danish"),
            ("fi", "Finnish"),
            ("no", "Norwegian"),
            ("cs", "Czech"),
            ("el", "Greek"),
            ("he", "Hebrew"),
            ("uk", "Ukrainian"),
            ("ro", "Romanian"),
            ("hu", "Hungarian"),
        ]
    }
}
