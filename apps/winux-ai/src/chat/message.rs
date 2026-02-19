// Message types for chat

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: Vec<MessageContent>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content: vec![MessageContent::Text { text: content }],
            timestamp: Utc::now(),
            image_path: None,
            file_path: None,
        }
    }

    pub fn with_image(role: MessageRole, text: String, image_path: String) -> Self {
        // Read and encode image as base64
        let image_content = if let Ok(bytes) = std::fs::read(&image_path) {
            let mime = mime_guess::from_path(&image_path)
                .first_or_octet_stream()
                .to_string();
            let base64_data = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &bytes
            );
            format!("data:{};base64,{}", mime, base64_data)
        } else {
            String::new()
        };

        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content: vec![
                MessageContent::Text { text },
                MessageContent::ImageUrl {
                    image_url: ImageUrl {
                        url: image_content,
                        detail: Some("auto".to_string()),
                    },
                },
            ],
            timestamp: Utc::now(),
            image_path: Some(image_path),
            file_path: None,
        }
    }

    pub fn with_file(role: MessageRole, text: String, file_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content: vec![MessageContent::Text { text }],
            timestamp: Utc::now(),
            image_path: None,
            file_path: Some(file_path),
        }
    }

    pub fn get_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| match c {
                MessageContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn has_image(&self) -> bool {
        self.content.iter().any(|c| matches!(c, MessageContent::ImageUrl { .. }))
    }

    /// Convert to API format
    pub fn to_api_format(&self) -> serde_json::Value {
        if self.has_image() {
            serde_json::json!({
                "role": self.role,
                "content": self.content
            })
        } else {
            serde_json::json!({
                "role": self.role,
                "content": self.get_text()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(MessageRole::User, "Hello".to_string());
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.get_text(), "Hello");
    }
}
