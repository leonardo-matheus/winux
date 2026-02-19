//! Translation feature

use anyhow::Result;
use std::sync::Arc;

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::cache::{ResponseCache, response_cache::CachedResponse};

/// Translation feature
pub struct TranslateFeature {
    client: Arc<AzureOpenAIClient>,
    cache: Arc<ResponseCache>,
    rate_limiter: Arc<RateLimiter>,
}

impl TranslateFeature {
    /// Create a new translate feature
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

    /// Translate text from one language to another
    pub async fn translate(&self, text: &str, from_lang: &str, to_lang: &str) -> Result<String> {
        // Check cache first
        let cache_key = ResponseCache::generate_key("translate", &[text, from_lang, to_lang]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for translation request");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (text.len() / 4 + 500) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = if from_lang.is_empty() || from_lang.to_lowercase() == "auto" {
            format!(
                "Translate the following text to {}. \
                Provide only the translation without any explanation.\n\n\
                Text: {}\n\n\
                Translation:",
                Self::get_language_name(to_lang),
                text
            )
        } else {
            format!(
                "Translate the following text from {} to {}. \
                Provide only the translation without any explanation.\n\n\
                Text: {}\n\n\
                Translation:",
                Self::get_language_name(from_lang),
                Self::get_language_name(to_lang),
                text
            )
        };

        // Make API request
        tracing::info!("Sending translation request to Azure OpenAI ({} -> {})", from_lang, to_lang);
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: response.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(response)
    }

    /// Detect the language of text
    pub async fn detect_language(&self, text: &str) -> Result<String> {
        // Check cache
        let cache_key = ResponseCache::generate_key("detect_language", &[text]);

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for language detection");
            return Ok(cached.content);
        }

        // Estimate tokens
        let estimated_tokens = (text.len() / 4 + 100) as u32;

        // Apply rate limiting
        self.rate_limiter.acquire(estimated_tokens).await?;

        // Build prompt
        let prompt = format!(
            "Detect the language of the following text. \
            Respond with only the ISO 639-1 language code (e.g., 'en' for English, 'pt' for Portuguese).\n\n\
            Text: {}\n\n\
            Language code:",
            text
        );

        // Make API request
        tracing::info!("Sending language detection request to Azure OpenAI");
        let response = self.client.complete(&prompt, "gpt-4o").await?;

        // Clean up response (should be just the language code)
        let language_code = response.trim().to_lowercase();

        // Cache the response
        self.cache.set(cache_key, CachedResponse {
            content: language_code.clone(),
            tokens: Some(estimated_tokens),
        }).await;

        Ok(language_code)
    }

    /// Get human-readable language name from code
    fn get_language_name(code: &str) -> &str {
        match code.to_lowercase().as_str() {
            "en" => "English",
            "pt" | "pt-br" => "Portuguese",
            "es" => "Spanish",
            "fr" => "French",
            "de" => "German",
            "it" => "Italian",
            "nl" => "Dutch",
            "ru" => "Russian",
            "zh" | "zh-cn" => "Chinese (Simplified)",
            "zh-tw" => "Chinese (Traditional)",
            "ja" => "Japanese",
            "ko" => "Korean",
            "ar" => "Arabic",
            "hi" => "Hindi",
            "tr" => "Turkish",
            "pl" => "Polish",
            "sv" => "Swedish",
            "da" => "Danish",
            "no" => "Norwegian",
            "fi" => "Finnish",
            "cs" => "Czech",
            "el" => "Greek",
            "he" => "Hebrew",
            "th" => "Thai",
            "vi" => "Vietnamese",
            "id" => "Indonesian",
            "ms" => "Malay",
            _ => code,
        }
    }
}
