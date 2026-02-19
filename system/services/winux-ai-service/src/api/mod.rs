//! API module for Azure OpenAI integration

pub mod azure_client;
pub mod rate_limiter;

pub use azure_client::AzureOpenAIClient;
pub use rate_limiter::RateLimiter;
