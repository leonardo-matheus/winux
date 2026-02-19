//! Configuration module for winux-ai-service
//!
//! Loads configuration from /etc/winux/ai-service.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default configuration file path
pub const CONFIG_PATH: &str = "/etc/winux/ai-service.toml";

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Azure OpenAI configuration
    pub azure: AzureConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Service configuration
    pub service: ServiceConfig,
}

/// Azure OpenAI API configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureConfig {
    /// API key for authentication
    pub api_key: String,
    /// Base endpoint URL
    pub endpoint: String,
    /// GPT-4o deployment name
    pub gpt4o_deployment: String,
    /// API version
    pub api_version: String,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Maximum tokens for responses
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
    /// Tokens per minute
    #[serde(default = "default_tokens_per_minute")]
    pub tokens_per_minute: u32,
    /// Enable rate limiting
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Enable response caching
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Maximum cache entries
    #[serde(default = "default_max_entries")]
    pub max_entries: u64,
    /// TTL in seconds
    #[serde(default = "default_ttl")]
    pub ttl_secs: u64,
}

/// Service configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    /// D-Bus bus type (system or session)
    #[serde(default = "default_bus_type")]
    pub bus_type: String,
    /// Enable streaming responses
    #[serde(default = "default_true")]
    pub streaming_enabled: bool,
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

// Default value functions
fn default_timeout() -> u64 { 60 }
fn default_max_tokens() -> u32 { 4096 }
fn default_requests_per_minute() -> u32 { 60 }
fn default_tokens_per_minute() -> u32 { 90000 }
fn default_true() -> bool { true }
fn default_max_entries() -> u64 { 1000 }
fn default_ttl() -> u64 { 3600 }
fn default_bus_type() -> String { "system".to_string() }
fn default_log_level() -> String { "info".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            azure: AzureConfig {
                api_key: String::new(),
                endpoint: "https://conta-ma6t6uyn-eastus2.openai.azure.com".to_string(),
                gpt4o_deployment: "gpt-4o".to_string(),
                api_version: "2025-01-01-preview".to_string(),
                timeout_secs: default_timeout(),
                max_tokens: default_max_tokens(),
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: default_requests_per_minute(),
                tokens_per_minute: default_tokens_per_minute(),
                enabled: true,
            },
            cache: CacheConfig {
                enabled: true,
                max_entries: default_max_entries(),
                ttl_secs: default_ttl(),
            },
            service: ServiceConfig {
                bus_type: default_bus_type(),
                streaming_enabled: true,
                log_level: default_log_level(),
            },
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        Self::load_from(CONFIG_PATH)
    }

    /// Load configuration from a specific path
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            tracing::warn!("Configuration file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file: {:?}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse configuration file: {:?}", path))?;

        config.validate()?;

        Ok(config)
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        if self.azure.api_key.is_empty() {
            anyhow::bail!("Azure API key is required");
        }

        if self.azure.endpoint.is_empty() {
            anyhow::bail!("Azure endpoint is required");
        }

        Ok(())
    }

    /// Get the full URL for chat completions
    pub fn chat_completions_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.azure.endpoint,
            self.azure.gpt4o_deployment,
            self.azure.api_version
        )
    }

    /// Generate default configuration file content
    pub fn generate_default_config() -> String {
        r#"# Winux AI Service Configuration
# Location: /etc/winux/ai-service.toml

[azure]
# Azure OpenAI API key (required)
api_key = "YOUR_API_KEY_HERE"

# Azure OpenAI endpoint
endpoint = "https://conta-ma6t6uyn-eastus2.openai.azure.com"

# Deployment name for GPT-4o
gpt4o_deployment = "gpt-4o"

# API version
api_version = "2025-01-01-preview"

# Request timeout in seconds
timeout_secs = 60

# Maximum tokens for responses
max_tokens = 4096

[rate_limit]
# Enable rate limiting
enabled = true

# Maximum requests per minute
requests_per_minute = 60

# Maximum tokens per minute
tokens_per_minute = 90000

[cache]
# Enable response caching
enabled = true

# Maximum number of cached responses
max_entries = 1000

# Cache TTL in seconds (1 hour)
ttl_secs = 3600

[service]
# D-Bus bus type: "system" or "session"
bus_type = "system"

# Enable streaming responses via D-Bus signals
streaming_enabled = true

# Log level: "trace", "debug", "info", "warn", "error"
log_level = "info"
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.azure.timeout_secs, 60);
        assert_eq!(config.rate_limit.requests_per_minute, 60);
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_chat_completions_url() {
        let mut config = Config::default();
        config.azure.api_key = "test-key".to_string();

        let url = config.chat_completions_url();
        assert!(url.contains("gpt-4o"));
        assert!(url.contains("chat/completions"));
    }
}
