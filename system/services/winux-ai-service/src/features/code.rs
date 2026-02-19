//! Code assistance feature

use anyhow::Result;
use std::sync::Arc;

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::cache::{ResponseCache, response_cache::CachedResponse};

/// Code assistance feature
pub struct CodeFeature {
    client: Arc<AzureOpenAIClient>,
    cache: Arc<ResponseCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl CodeFeature {
    /// Create a new code feature
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

    /// Analyze code and provide insights
    pub async fn analyze(&self, code: &str, language: &str) -> Result<String> {
        // Check cache first
        let cache_key = ResponseCache::generate_key("analyze_code", &[code, language]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for code analysis request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (code.len() / 4 + 1000) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "Analyze the following {} code and provide:\n\
            1. A brief description of what the code does\n\
            2. Any potential issues or bugs\n\
            3. Suggestions for improvement\n\
            4. Security concerns (if any)\n\n\
            ```{}\n{}\n```\n\n\
            Analysis:",
            language, language, code
        );

        // Make API request
        tracing::info!("Sending code analysis request to Azure OpenAI for {} code", language);
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Explain code in simple terms
    pub async fn explain(&self, code: &str, language: &str) -> Result<String> {
        // Check cache
        let cache_key = ResponseCache::generate_key("explain_code", &[code, language]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for code explanation request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (code.len() / 4 + 800) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "Explain the following {} code in simple terms. \
            Break down what each part does and how it works.\n\n\
            ```{}\n{}\n```\n\n\
            Explanation:",
            language, language, code
        );

        // Make API request
        tracing::info!("Sending code explanation request to Azure OpenAI");
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Suggest improvements for code
    pub async fn improve(&self, code: &str, language: &str) -> Result<String> {
        // Check cache
        let cache_key = ResponseCache::generate_key("improve_code", &[code, language]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for code improvement request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (code.len() / 4 + 1500) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "Review and improve the following {} code. \
            Provide the improved version with comments explaining the changes. \
            Focus on:\n\
            - Code quality and readability\n\
            - Performance optimization\n\
            - Best practices\n\
            - Error handling\n\n\
            Original code:\n```{}\n{}\n```\n\n\
            Improved code:",
            language, language, code
        );

        // Make API request
        tracing::info!("Sending code improvement request to Azure OpenAI");
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Fix bugs in code
    pub async fn fix(&self, code: &str, language: &str, error_message: &str) -> Result<String> {
        // Check cache
        let cache_key = ResponseCache::generate_key("fix_code", &[code, language, error_message]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for code fix request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (code.len() / 4 + error_message.len() / 4 + 1000) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "The following {} code has an error. Fix the bug and explain what was wrong.\n\n\
            Code:\n```{}\n{}\n```\n\n\
            Error message:\n```\n{}\n```\n\n\
            Fixed code and explanation:",
            language, language, code, error_message
        );

        // Make API request
        tracing::info!("Sending code fix request to Azure OpenAI");
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Generate code documentation
    pub async fn document(&self, code: &str, language: &str) -> Result<String> {
        // Check cache
        let cache_key = ResponseCache::generate_key("document_code", &[code, language]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for code documentation request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (code.len() / 4 + 800) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "Generate comprehensive documentation for the following {} code. \
            Include:\n\
            - Function/class descriptions\n\
            - Parameter descriptions\n\
            - Return value descriptions\n\
            - Usage examples\n\n\
            Code:\n```{}\n{}\n```\n\n\
            Documentation:",
            language, language, code
        );

        // Make API request
        tracing::info!("Sending code documentation request to Azure OpenAI");
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }
}
