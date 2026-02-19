//! Chat completion feature

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::api::azure_client::{ChatCompletionRequest, ChatMessage, MessageRole, MessageContent, StreamChunk};
use crate::cache::{ResponseCache, response_cache::CachedResponse};

/// Chat completion feature
pub struct ChatFeature {
    client: Arc<AzureOpenAIClient>,
    cache: Arc<ResponseCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl ChatFeature {
    /// Create a new chat feature
    pub fn new(
        client: Arc<AzureOpenAIClient>,
        cache: Arc<ResponseCache>,
        rate_limiter: Arc<RateLimiter>,
    ) -> Self {
        Self {
            client,
            cache,
            rate_limiter,
        }
    }

    /// Chat with message history
    /// Messages are tuples of (role, content) where role is "user", "assistant", or "system"
    pub async fn chat(&self, messages: Vec<(String, String)>, model: &str) -> Result<String> {
        // Generate cache key from all messages
        let messages_str: Vec<String> = messages
            .iter()
            .map(|(r, c)| format!("{}:{}", r, c))
            .collect();
        let messages_joined = messages_str.join("|");
        let cache_key = ResponseCache::generate_key("chat", &[&messages_joined, model]);

        // Check cache
        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for chat request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let total_chars: usize = messages.iter().map(|(_, c)| c.len()).sum();
        let estimated_tokens = (total_chars / 4 + 1000) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Make API request
        tracing::info!("Sending chat request to Azure OpenAI with {} messages", messages.len());
        let response = self.client.chat(messages, model).await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Chat with streaming response
    pub async fn chat_stream(
        &self,
        messages: Vec<(String, String)>,
        model: &str,
        request_id: String,
    ) -> Result<mpsc::Receiver<StreamChunk>> {
        // Estimate tokens
        let total_chars: usize = messages.iter().map(|(_, c)| c.len()).sum();
        let estimated_tokens = (total_chars / 4 + 1000) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Convert messages
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
            max_tokens: None,
            temperature: Some(0.7),
            stream: Some(true),
        };

        tracing::info!("Sending streaming chat request to Azure OpenAI");
        self.client.chat_completion_stream(request, request_id).await
    }
}
