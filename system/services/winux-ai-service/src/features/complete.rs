//! Text completion feature

use anyhow::Result;
use std::sync::Arc;

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::cache::{ResponseCache, response_cache::CachedResponse};

/// Text completion feature
pub struct CompleteFeature {
    client: Arc<AzureOpenAIClient>,
    cache: Arc<ResponseCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl CompleteFeature {
    /// Create a new complete feature
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

    /// Complete text based on a prompt
    pub async fn complete(&self, prompt: &str, model: &str) -> Result<String> {
        // Check cache first
        let cache_key = ResponseCache::generate_key("complete", &[prompt, model]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for completion request");
            return Ok(cached.content);
        }

        // Estimate tokens (rough estimate: 1 token per 4 characters)
        let estimated_tokens = (prompt.len() / 4 + 1000) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Make API request
        tracing::info!("Sending completion request to Azure OpenAI");
        let response = self.client.complete(prompt, model).await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here
    // Require actual API credentials to run
}
