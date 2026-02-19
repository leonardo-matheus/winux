// Code Assistant - Help with coding tasks

use crate::ai::{AzureOpenAIClient, SystemPrompts};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

pub struct CodeAssistant {
    client: Arc<AzureOpenAIClient>,
}

impl CodeAssistant {
    pub fn new(client: Arc<AzureOpenAIClient>) -> Self {
        Self { client }
    }

    /// Explain code
    pub async fn explain(&self, code: &str, language: Option<&str>) -> Result<String> {
        let lang_context = language.map(|l| format!(" ({})", l)).unwrap_or_default();
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!("Explain the following code{}:\n\n```{}\n{}\n```",
                    lang_context,
                    language.unwrap_or(""),
                    code)
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Review code for issues and improvements
    pub async fn review(&self, code: &str, language: Option<&str>) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Review the following code for bugs, security issues, and potential improvements:\n\n```{}\n{}\n```",
                    language.unwrap_or(""),
                    code
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Suggest refactoring improvements
    pub async fn refactor(&self, code: &str, language: Option<&str>, goal: Option<&str>) -> Result<String> {
        let goal_text = goal.map(|g| format!(" Goal: {}", g)).unwrap_or_default();
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Suggest refactoring improvements for this code.{}\n\n```{}\n{}\n```",
                    goal_text,
                    language.unwrap_or(""),
                    code
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Generate tests for code
    pub async fn generate_tests(&self, code: &str, language: Option<&str>, framework: Option<&str>) -> Result<String> {
        let framework_text = framework.map(|f| format!(" using {}", f)).unwrap_or_default();
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Generate comprehensive tests{} for the following code:\n\n```{}\n{}\n```",
                    framework_text,
                    language.unwrap_or(""),
                    code
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Add documentation/comments to code
    pub async fn document(&self, code: &str, language: Option<&str>) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Add comprehensive documentation and comments to this code:\n\n```{}\n{}\n```",
                    language.unwrap_or(""),
                    code
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Fix a bug in code
    pub async fn fix_bug(&self, code: &str, error_message: Option<&str>, language: Option<&str>) -> Result<String> {
        let error_context = error_message
            .map(|e| format!("\n\nError message:\n```\n{}\n```", e))
            .unwrap_or_default();

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Fix the bug in this code:\n\n```{}\n{}\n```{}",
                    language.unwrap_or(""),
                    code,
                    error_context
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Convert code between languages
    pub async fn convert(&self, code: &str, from_lang: &str, to_lang: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Convert this {} code to {}:\n\n```{}\n{}\n```",
                    from_lang, to_lang, from_lang, code
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Optimize code for performance
    pub async fn optimize(&self, code: &str, language: Option<&str>) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::code_assistant()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Optimize this code for better performance:\n\n```{}\n{}\n```",
                    language.unwrap_or(""),
                    code
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }
}
