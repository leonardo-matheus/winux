//! Image analysis (vision) feature

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::cache::{ResponseCache, response_cache::CachedResponse};

/// Vision/Image analysis feature
pub struct VisionFeature {
    client: Arc<AzureOpenAIClient>,
    cache: Arc<ResponseCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl VisionFeature {
    /// Create a new vision feature
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

    /// Analyze an image from a file path
    pub async fn analyze_image(&self, image_path: &str, prompt: &str) -> Result<String> {
        let path = Path::new(image_path);

        // Validate file exists
        if !path.exists() {
            anyhow::bail!("Image file not found: {}", image_path);
        }

        // Check cache (using file path and prompt as key)
        // Note: This won't invalidate if the file changes - for production,
        // consider using file hash instead
        let cache_key = ResponseCache::generate_key("analyze_image", &[image_path, prompt]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for image analysis request");
            return Ok(cached.content);
        }

        // Read and encode image
        let image_data = fs::read(path)
            .await
            .with_context(|| format!("Failed to read image file: {}", image_path))?;

        let image_base64 = BASE64.encode(&image_data);

        // Estimate tokens (images use more tokens)
        let estimated_tokens = 1500_u32; // Vision requests typically use more tokens

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Make API request
        tracing::info!("Sending image analysis request to Azure OpenAI for: {}", image_path);
        let response = self.client.analyze_image(&image_base64, prompt).await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Analyze an image from base64 data
    pub async fn analyze_image_base64(&self, image_base64: &str, prompt: &str) -> Result<String> {
        // Check cache
        // Use a hash of the base64 data for the cache key (first 100 chars to avoid huge keys)
        let image_hash = &image_base64[..image_base64.len().min(100)];
        let cache_key = ResponseCache::generate_key("analyze_image_b64", &[image_hash, prompt]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for base64 image analysis request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = 1500_u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Make API request
        tracing::info!("Sending base64 image analysis request to Azure OpenAI");
        let response = self.client.analyze_image(image_base64, prompt).await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Describe an image
    pub async fn describe(&self, image_path: &str) -> Result<String> {
        self.analyze_image(
            image_path,
            "Describe this image in detail. Include information about the main subjects, \
            colors, composition, and any text or notable elements visible.",
        ).await
    }

    /// Extract text from an image (OCR)
    pub async fn extract_text(&self, image_path: &str) -> Result<String> {
        self.analyze_image(
            image_path,
            "Extract and transcribe all text visible in this image. \
            Maintain the original formatting as much as possible. \
            If no text is visible, respond with 'No text found in image.'",
        ).await
    }

    /// Identify objects in an image
    pub async fn identify_objects(&self, image_path: &str) -> Result<String> {
        self.analyze_image(
            image_path,
            "List all identifiable objects in this image. \
            For each object, provide a brief description and estimated location \
            (e.g., 'in the foreground', 'top-left corner').",
        ).await
    }

    /// Get accessibility description for an image
    pub async fn accessibility_description(&self, image_path: &str) -> Result<String> {
        self.analyze_image(
            image_path,
            "Provide an accessibility-focused description of this image suitable for \
            screen readers. Be concise but comprehensive, focusing on the most \
            important visual information.",
        ).await
    }
}
