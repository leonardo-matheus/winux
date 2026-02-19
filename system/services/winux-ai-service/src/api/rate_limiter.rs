//! Rate limiting implementation for Azure OpenAI API

use anyhow::Result;
use governor::{Quota, RateLimiter as GovRateLimiter};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use std::num::NonZeroU32;
use std::sync::Arc;

use crate::config::RateLimitConfig;

/// Rate limiter for API requests
pub struct RateLimiter {
    /// Request rate limiter
    request_limiter: Option<GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    /// Token rate limiter
    token_limiter: Option<GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    /// Configuration
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Result<Self> {
        let request_limiter = if config.enabled {
            let requests_per_minute = NonZeroU32::new(config.requests_per_minute)
                .ok_or_else(|| anyhow::anyhow!("Invalid requests_per_minute"))?;

            Some(GovRateLimiter::direct(Quota::per_minute(requests_per_minute)))
        } else {
            None
        };

        // Token rate limiter - for simplicity, we'll track estimated tokens
        let token_limiter = if config.enabled {
            // Allow tokens_per_minute / 100 (assuming avg 100 tokens per request)
            let tokens_per_burst = config.tokens_per_minute / 100;
            let tokens_burst = NonZeroU32::new(tokens_per_burst.max(1))
                .ok_or_else(|| anyhow::anyhow!("Invalid tokens_per_minute"))?;

            Some(GovRateLimiter::direct(Quota::per_minute(tokens_burst)))
        } else {
            None
        };

        Ok(Self {
            request_limiter,
            token_limiter,
            config,
        })
    }

    /// Check if a request can proceed
    pub async fn check_request(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if let Some(limiter) = &self.request_limiter {
            limiter.until_ready().await;
        }

        Ok(())
    }

    /// Check if tokens can be consumed
    pub async fn check_tokens(&self, estimated_tokens: u32) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if let Some(limiter) = &self.token_limiter {
            // Convert tokens to "request units" (1 unit = 100 tokens)
            let units = (estimated_tokens / 100).max(1);

            for _ in 0..units {
                limiter.until_ready().await;
            }
        }

        Ok(())
    }

    /// Check both request and token limits
    pub async fn acquire(&self, estimated_tokens: u32) -> Result<()> {
        self.check_request().await?;
        self.check_tokens(estimated_tokens).await?;
        Ok(())
    }

    /// Get remaining requests (approximate)
    pub fn remaining_requests(&self) -> Option<u32> {
        self.request_limiter.as_ref().map(|_| {
            // Governor doesn't expose remaining quota easily
            // Return configured limit as approximation
            self.config.requests_per_minute
        })
    }

    /// Check if rate limiting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Wrapper for thread-safe rate limiter
pub type SharedRateLimiter = Arc<RateLimiter>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            requests_per_minute: 60,
            tokens_per_minute: 90000,
            enabled: false,
        };

        let limiter = RateLimiter::new(config).unwrap();
        assert!(!limiter.is_enabled());

        // Should not block
        limiter.check_request().await.unwrap();
        limiter.check_tokens(1000).await.unwrap();
    }

    #[tokio::test]
    async fn test_rate_limiter_enabled() {
        let config = RateLimitConfig {
            requests_per_minute: 60,
            tokens_per_minute: 90000,
            enabled: true,
        };

        let limiter = RateLimiter::new(config).unwrap();
        assert!(limiter.is_enabled());

        // First request should pass immediately
        limiter.check_request().await.unwrap();
    }
}
