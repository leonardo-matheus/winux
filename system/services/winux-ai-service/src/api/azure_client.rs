//! Azure OpenAI HTTP client implementation

use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::config::Config;

/// Message role for chat completions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Chat message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,
}

/// Message content - can be text or multimodal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentPart>),
}

/// Content part for multimodal messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

/// Image URL structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Chat completion request
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Chat completion response
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// Response choice
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Option<ResponseMessage>,
    pub delta: Option<DeltaContent>,
    pub finish_reason: Option<String>,
}

/// Response message
#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
}

/// Delta content for streaming
#[derive(Debug, Deserialize)]
pub struct DeltaContent {
    pub content: Option<String>,
}

/// Token usage information
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming chunk data
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub request_id: String,
    pub content: String,
    pub done: bool,
}

/// Azure OpenAI client
pub struct AzureOpenAIClient {
    client: Client,
    config: Arc<Config>,
}

impl AzureOpenAIClient {
    /// Create a new Azure OpenAI client
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.azure.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }

    /// Send a chat completion request
    pub async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let url = self.config.chat_completions_url();

        let response = self.client
            .post(&url)
            .header("api-key", &self.config.azure.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Azure OpenAI")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure OpenAI API error ({}): {}", status, error_text);
        }

        let result = response
            .json::<ChatCompletionResponse>()
            .await
            .context("Failed to parse Azure OpenAI response")?;

        Ok(result)
    }

    /// Send a streaming chat completion request
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
        request_id: String,
    ) -> Result<mpsc::Receiver<StreamChunk>> {
        request.stream = Some(true);

        let url = self.config.chat_completions_url();

        let response = self.client
            .post(&url)
            .header("api-key", &self.config.azure.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send streaming request to Azure OpenAI")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure OpenAI API error ({}): {}", status, error_text);
        }

        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(Self::process_stream(response, request_id, tx));

        Ok(rx)
    }

    /// Process the SSE stream
    async fn process_stream(response: Response, request_id: String, tx: mpsc::Sender<StreamChunk>) {
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));

                    // Process complete SSE events
                    while let Some(pos) = buffer.find("\n\n") {
                        let event = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();

                        for line in event.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];

                                if data == "[DONE]" {
                                    let _ = tx.send(StreamChunk {
                                        request_id: request_id.clone(),
                                        content: String::new(),
                                        done: true,
                                    }).await;
                                    return;
                                }

                                if let Ok(parsed) = serde_json::from_str::<ChatCompletionResponse>(data) {
                                    if let Some(choice) = parsed.choices.first() {
                                        if let Some(delta) = &choice.delta {
                                            if let Some(content) = &delta.content {
                                                let _ = tx.send(StreamChunk {
                                                    request_id: request_id.clone(),
                                                    content: content.clone(),
                                                    done: false,
                                                }).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                    break;
                }
            }
        }

        // Send final done signal if not already sent
        let _ = tx.send(StreamChunk {
            request_id,
            content: String::new(),
            done: true,
        }).await;
    }

    /// Simple text completion
    pub async fn complete(&self, prompt: &str, model: &str) -> Result<String> {
        let request = ChatCompletionRequest {
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: MessageContent::Text(prompt.to_string()),
            }],
            max_tokens: Some(self.config.azure.max_tokens),
            temperature: Some(0.7),
            stream: None,
        };

        let response = self.chat_completion(request).await?;

        response.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response content from model"))
    }

    /// Chat with message history
    pub async fn chat(&self, messages: Vec<(String, String)>, model: &str) -> Result<String> {
        let chat_messages: Vec<ChatMessage> = messages
            .into_iter()
            .map(|(role, content)| {
                let role = match role.to_lowercase().as_str() {
                    "system" => MessageRole::System,
                    "assistant" => MessageRole::Assistant,
                    _ => MessageRole::User,
                };
                ChatMessage {
                    role,
                    content: MessageContent::Text(content),
                }
            })
            .collect();

        let request = ChatCompletionRequest {
            messages: chat_messages,
            max_tokens: Some(self.config.azure.max_tokens),
            temperature: Some(0.7),
            stream: None,
        };

        let response = self.chat_completion(request).await?;

        response.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response content from model"))
    }

    /// Analyze an image with a prompt
    pub async fn analyze_image(&self, image_base64: &str, prompt: &str) -> Result<String> {
        let image_url = format!("data:image/jpeg;base64,{}", image_base64);

        let request = ChatCompletionRequest {
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: MessageContent::Multimodal(vec![
                    ContentPart::Text { text: prompt.to_string() },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: image_url,
                            detail: Some("auto".to_string()),
                        },
                    },
                ]),
            }],
            max_tokens: Some(self.config.azure.max_tokens),
            temperature: Some(0.7),
            stream: None,
        };

        let response = self.chat_completion(request).await?;

        response.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response content from model"))
    }
}
