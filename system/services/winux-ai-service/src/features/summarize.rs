//! Text summarization feature

use anyhow::Result;
use std::sync::Arc;

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::cache::{ResponseCache, response_cache::CachedResponse};

/// Summarization feature
pub struct SummarizeFeature {
    client: Arc<AzureOpenAIClient>,
    cache: Arc<ResponseCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl SummarizeFeature {
    /// Create a new summarize feature
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

    /// Summarize text
    pub async fn summarize(&self, text: &str) -> Result<String> {
        // Check cache first
        let cache_key = ResponseCache::generate_key("summarize", &[text]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for summarization request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (text.len() / 4 + 500) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "Please provide a concise summary of the following text. \
            Focus on the main points and key information.\n\n\
            Text to summarize:\n{}\n\n\
            Summary:",
            text
        );

        // Make API request
        tracing::info!("Sending summarization request to Azure OpenAI");
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Summarize with custom instructions
    pub async fn summarize_with_instructions(&self, text: &str, instructions: &str) -> Result<String> {
        // Check cache
        let cache_key = ResponseCache::generate_key("summarize_custom", &[text, instructions]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for custom summarization request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (text.len() / 4 + 500) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "{}\n\nText:\n{}\n\nSummary:",
            instructions,
            text
        );

        // Make API request
        tracing::info!("Sending custom summarization request to Azure OpenAI");
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }
}
