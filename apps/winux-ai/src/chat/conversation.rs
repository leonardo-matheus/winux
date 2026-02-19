// Conversation history management

use super::{Message, MessageRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub metadata: ConversationMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub total_tokens: u64,
    pub total_messages: usize,
    pub tags: Vec<String>,
    pub starred: bool,
}

impl Conversation {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: "New Conversation".to_string(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            model: "gpt-4o".to_string(),
            system_prompt: None,
            metadata: ConversationMetadata::default(),
        }
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
        self.metadata.total_messages = self.messages.len();
    }

    pub fn get_messages_for_api(&self) -> Vec<serde_json::Value> {
        let mut api_messages = Vec::new();

        // Add system prompt if present
        if let Some(ref system_prompt) = self.system_prompt {
            api_messages.push(serde_json::json!({
                "role": "system",
                "content": system_prompt
            }));
        } else {
            // Default system prompt
            api_messages.push(serde_json::json!({
                "role": "system",
                "content": "You are a helpful AI assistant integrated into Winux OS. You help users with coding, system administration, file management, and general questions. When providing code, use markdown code blocks with appropriate language tags. Be concise but thorough."
            }));
        }

        // Add conversation messages
        for message in &self.messages {
            api_messages.push(message.to_api_format());
        }

        api_messages
    }

    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.title);
        md.push_str(&format!("*Created: {}*\n\n", self.created_at.format("%Y-%m-%d %H:%M")));
        md.push_str("---\n\n");

        for message in &self.messages {
            let role_str = match message.role {
                MessageRole::User => "**You**",
                MessageRole::Assistant => "**AI Assistant**",
                MessageRole::System => "**System**",
            };
            md.push_str(&format!("{}\n\n{}\n\n---\n\n", role_str, message.get_text()));
        }

        md
    }

    pub fn get_last_n_messages(&self, n: usize) -> Vec<&Message> {
        self.messages.iter().rev().take(n).rev().collect()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.metadata.total_messages = 0;
        self.updated_at = Utc::now();
    }

    pub fn update_title_from_first_message(&mut self) {
        if let Some(first_user_msg) = self.messages.iter().find(|m| m.role == MessageRole::User) {
            let text = first_user_msg.get_text();
            let words: Vec<&str> = text.split_whitespace().take(6).collect();
            self.title = if words.len() >= 6 {
                format!("{}...", words.join(" "))
            } else {
                words.join(" ")
            };

            // Limit title length
            if self.title.len() > 60 {
                self.title = format!("{}...", &self.title[..57]);
            }
        }
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_creation() {
        let conv = Conversation::new();
        assert!(conv.messages.is_empty());
        assert_eq!(conv.title, "New Conversation");
    }

    #[test]
    fn test_add_message() {
        let mut conv = Conversation::new();
        let msg = Message::new(MessageRole::User, "Hello".to_string());
        conv.add_message(msg);
        assert_eq!(conv.messages.len(), 1);
    }
}
