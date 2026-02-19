//! Response caching implementation

use moka::future::Cache;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use std::time::Duration;

use crate::config::CacheConfig;

/// Cached response entry
#[derive(Debug, Clone)]
pub struct CachedResponse {
    /// The response content
    pub content: String,
    /// Token count (if known)
    pub tokens: Option<u32>,
}

/// Response cache for avoiding duplicate API calls
pub struct ResponseCache {
    /// Internal cache
    cache: Cache<String, CachedResponse>,
    /// Configuration
    config: CacheConfig,
}

impl ResponseCache {
    /// Create a new response cache
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_entries)
            .time_to_live(Duration::from_secs(config.ttl_secs))
            .build();

        Self { cache, config }
    }

    /// Generate a cache key from the request parameters
    pub fn generate_key(operation: &str, params: &[&str]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(operation.as_bytes());

        for param in params {
            hasher.update(b"|");
            hasher.update(param.as_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Get a cached response
    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        if !self.config.enabled {
            return None;
        }

        self.cache.get(key).await
    }

    /// Store a response in the cache
    pub async fn set(&self, key: String, response: CachedResponse) {
        if !self.config.enabled {
            return;
        }

        self.cache.insert(key, response).await;
    }

    /// Check if a key exists in the cache
    pub async fn contains(&self, key: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.cache.contains_key(key)
    }

    /// Remove a specific entry from the cache
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if caching is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.len(),
            max_entries: self.config.max_entries,
            ttl_secs: self.config.ttl_secs,
            enabled: self.config.enabled,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: u64,
    pub max_entries: u64,
    pub ttl_secs: u64,
    pub enabled: bool,
}

/// Wrapper for thread-safe cache
pub type SharedCache = Arc<ResponseCache>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_operations() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            ttl_secs: 3600,
        };

        let cache = ResponseCache::new(config);

        // Generate a key
        let key = ResponseCache::generate_key("complete", &["test prompt", "gpt-4o"]);
        assert!(!key.is_empty());

        // Store a response
        let response = CachedResponse {
            content: "Test response".to_string(),
            tokens: Some(10),
        };

        cache.set(key.clone(), response).await;

        // Retrieve the response
        let cached = cache.get(&key).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, "Test response");
    }

    #[tokio::test]
    async fn test_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            max_entries: 100,
            ttl_secs: 3600,
        };

        let cache = ResponseCache::new(config);

        let key = ResponseCache::generate_key("complete", &["test"]);
        let response = CachedResponse {
            content: "Test".to_string(),
            tokens: None,
        };

        cache.set(key.clone(), response).await;

        // Should not retrieve when disabled
        let cached = cache.get(&key).await;
        assert!(cached.is_none());
    }

    #[test]
    fn test_key_generation() {
        let key1 = ResponseCache::generate_key("complete", &["prompt1", "model1"]);
        let key2 = ResponseCache::generate_key("complete", &["prompt1", "model1"]);
        let key3 = ResponseCache::generate_key("complete", &["prompt2", "model1"]);

        // Same parameters should produce same key
        assert_eq!(key1, key2);

        // Different parameters should produce different key
        assert_ne!(key1, key3);
    }
}
