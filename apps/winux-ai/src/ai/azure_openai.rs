// Azure OpenAI Client implementation

use crate::chat::{StreamingResponse, streaming::StreamHandler};
use crate::ai::models::Model;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use futures::StreamExt;

// API configuration loaded from environment variables or config file
// Set WINUX_AI_API_KEY environment variable or configure in ~/.config/winux/ai.toml
fn get_api_key() -> String {
    std::env::var("WINUX_AI_API_KEY")
        .or_else(|_| {
            let config_path = dirs::config_dir()
                .map(|p| p.join("winux/ai.toml"))
                .unwrap_or_default();
            std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| {
                    content.lines()
                        .find(|l| l.starts_with("api_key"))
                        .and_then(|l| l.split('=').nth(1))
                        .map(|s| s.trim().trim_matches('"').to_string())
                })
                .ok_or(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| String::new())
}

fn get_endpoint() -> String {
    std::env::var("WINUX_AI_ENDPOINT")
        .unwrap_or_else(|_| "https://api.openai.azure.com".to_string())
}

const GPT4O_PATH: &str = "/openai/deployments/gpt-4o/chat/completions?api-version=2025-01-01-preview";
const O1_PATH: &str = "/openai/deployments/o1/chat/completions?api-version=2025-01-01-preview";

pub struct AzureOpenAIClient {
    client: Client,
    current_model: Arc<RwLock<Model>>,
    temperature: Arc<RwLock<f32>>,
    max_tokens: Arc<RwLock<u32>>,
}

impl AzureOpenAIClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            current_model: Arc::new(RwLock::new(Model::Gpt4o)),
            temperature: Arc::new(RwLock::new(0.7)),
            max_tokens: Arc::new(RwLock::new(4096)),
        }
    }

    pub fn set_model(&self, model: Model) {
        *self.current_model.write() = model;
    }

    pub fn get_model(&self) -> Model {
        *self.current_model.read()
    }

    pub fn set_temperature(&self, temp: f32) {
        *self.temperature.write() = temp.clamp(0.0, 2.0);
    }

    pub fn set_max_tokens(&self, tokens: u32) {
        *self.max_tokens.write() = tokens.clamp(256, 128000);
    }

    fn get_endpoint(&self) -> &str {
        match *self.current_model.read() {
            Model::Gpt4o => GPT4O_ENDPOINT,
            Model::O1 => O1_ENDPOINT,
        }
    }

    /// Send a chat completion request (non-streaming)
    pub async fn chat_completion(&self, messages: Vec<Value>) -> Result<String> {
        let model = *self.current_model.read();
        let temperature = *self.temperature.read();
        let max_tokens = *self.max_tokens.read();

        let mut body = json!({
            "messages": messages,
            "max_tokens": max_tokens,
        });

        // o1 model doesn't support temperature
        if model != Model::O1 {
            body["temperature"] = json!(temperature);
        }

        let response = self.client
            .post(self.get_endpoint())
            .header("api-key", API_KEY)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API Error: {}", error_text));
        }

        let response_json: Value = response.json().await?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();

        Ok(content)
    }

    /// Send a chat completion request with streaming
    pub async fn chat_completion_stream(&self, messages: Vec<Value>) -> Result<mpsc::Receiver<String>> {
        let model = *self.current_model.read();
        let temperature = *self.temperature.read();
        let max_tokens = *self.max_tokens.read();

        let mut body = json!({
            "messages": messages,
            "max_tokens": max_tokens,
            "stream": true,
        });

        // o1 model doesn't support temperature or streaming
        if model == Model::O1 {
            // o1 doesn't support streaming, fall back to regular completion
            let content = self.chat_completion(messages).await?;
            let (tx, rx) = mpsc::channel(1);
            tokio::spawn(async move {
                let _ = tx.send(content).await;
            });
            return Ok(rx);
        }

        body["temperature"] = json!(temperature);

        let response = self.client
            .post(self.get_endpoint())
            .header("api-key", API_KEY)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API Error: {}", error_text));
        }

        let (tx, rx) = mpsc::channel(100);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut handler = StreamHandler::new(tx);
            while let Some(chunk_result) = stream.next().await {
                if let Ok(chunk) = chunk_result {
                    if let Err(e) = handler.process_bytes(&chunk).await {
                        tracing::error!("Stream processing error: {}", e);
                        break;
                    }
                }
            }
            let _ = handler.flush().await;
        });

        Ok(rx)
    }

    /// Vision API - analyze image with GPT-4o
    pub async fn analyze_image(&self, image_base64: &str, prompt: &str) -> Result<String> {
        // Force GPT-4o for vision
        let messages = vec![json!({
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": prompt
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": image_base64,
                        "detail": "auto"
                    }
                }
            ]
        })];

        let body = json!({
            "messages": messages,
            "max_tokens": 4096,
            "temperature": 0.7,
        });

        let response = self.client
            .post(GPT4O_ENDPOINT)
            .header("api-key", API_KEY)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Vision API Error: {}", error_text));
        }

        let response_json: Value = response.json().await?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();

        Ok(content)
    }
}

impl Default for AzureOpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AzureOpenAIClient::new();
        assert_eq!(client.get_model(), Model::Gpt4o);
    }

    #[test]
    fn test_model_switch() {
        let client = AzureOpenAIClient::new();
        client.set_model(Model::O1);
        assert_eq!(client.get_model(), Model::O1);
    }
}
