// Terminal Helper - Assist with command-line operations

use crate::ai::{AzureOpenAIClient, SystemPrompts};
use crate::integrations::SystemInfo;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

pub struct TerminalHelper {
    client: Arc<AzureOpenAIClient>,
}

impl TerminalHelper {
    pub fn new(client: Arc<AzureOpenAIClient>) -> Self {
        Self { client }
    }

    /// Get help for a command
    pub async fn explain_command(&self, command: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::terminal_helper()
            }),
            json!({
                "role": "user",
                "content": format!("Explain what this command does:\n\n```bash\n{}\n```", command)
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Suggest a command for a task
    pub async fn suggest_command(&self, task: &str) -> Result<String> {
        let system_info = SystemInfo::collect().summary();

        let messages = vec![
            json!({
                "role": "system",
                "content": format!(
                    "{}\n\nSystem context:\n{}",
                    SystemPrompts::terminal_helper(),
                    system_info
                )
            }),
            json!({
                "role": "user",
                "content": format!("Suggest a command to: {}", task)
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Fix a command that isn't working
    pub async fn fix_command(&self, command: &str, error: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::terminal_helper()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "This command failed:\n\n```bash\n{}\n```\n\nError:\n```\n{}\n```\n\nHow do I fix it?",
                    command, error
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Convert command between systems (e.g., macOS to Linux)
    pub async fn convert_command(&self, command: &str, from_system: &str, to_system: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::terminal_helper()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Convert this {} command to work on {}:\n\n```bash\n{}\n```",
                    from_system, to_system, command
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Create a shell script for a complex task
    pub async fn create_script(&self, description: &str, shell: Option<&str>) -> Result<String> {
        let shell_type = shell.unwrap_or("bash");

        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::terminal_helper()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Create a {} script that: {}\n\nInclude error handling and comments.",
                    shell_type, description
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Explain the output of a command
    pub async fn explain_output(&self, command: &str, output: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::terminal_helper()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Explain the output of this command:\n\nCommand:\n```bash\n{}\n```\n\nOutput:\n```\n{}\n```",
                    command, output
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Find a command by describing what you want to do
    pub async fn find_command(&self, description: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::terminal_helper()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "What Linux command can I use to: {}\n\nProvide multiple options if available.",
                    description
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }

    /// Check if a command is safe to run
    pub async fn check_safety(&self, command: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": SystemPrompts::terminal_helper()
            }),
            json!({
                "role": "user",
                "content": format!(
                    "Is this command safe to run? Explain any risks:\n\n```bash\n{}\n```",
                    command
                )
            }),
        ];

        self.client.chat_completion(messages).await
    }
}
