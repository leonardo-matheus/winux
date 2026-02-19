//! Winux AI Service - System-wide AI daemon
//!
//! This service provides AI capabilities to all applications in the Winux system
//! via D-Bus interface. It connects to Azure OpenAI API and exposes methods for:
//! - Text completion
//! - Chat conversations
//! - Summarization
//! - Translation
//! - Code analysis
//! - Image analysis (vision)
//!
//! # Configuration
//! Configuration is loaded from `/etc/winux/ai-service.toml`
//!
//! # D-Bus Interface
//! Service name: `com.winux.AI`
//! Object path: `/com/winux/AI`

mod api;
mod cache;
mod config;
mod dbus;
mod features;

use std::sync::Arc;
use anyhow::{Context, Result};
use tokio::signal::unix::{signal, SignalKind};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use zbus::connection::Builder;

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::cache::ResponseCache;
use crate::config::Config;
use crate::dbus::{AIService, interface::{SERVICE_NAME, OBJECT_PATH}};

/// Main entry point for the winux-ai-service daemon
#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()
        .context("Failed to load configuration")?;

    // Initialize logging
    init_logging(&config.service.log_level)?;

    tracing::info!("Starting Winux AI Service v{}", env!("CARGO_PKG_VERSION"));

    // Create shared configuration
    let config = Arc::new(config);

    // Initialize components
    let client = Arc::new(
        AzureOpenAIClient::new(config.clone())
            .context("Failed to create Azure OpenAI client")?
    );

    let cache = Arc::new(ResponseCache::new(config.cache.clone()));
    tracing::info!("Response cache initialized (enabled: {})", cache.is_enabled());

    let rate_limiter = Arc::new(
        RateLimiter::new(config.rate_limit.clone())
            .context("Failed to create rate limiter")?
    );
    tracing::info!("Rate limiter initialized (enabled: {})", rate_limiter.is_enabled());

    // Create AI service
    let ai_service = AIService::new(client, cache, rate_limiter);

    // Connect to D-Bus
    let connection = match config.service.bus_type.as_str() {
        "session" => {
            tracing::info!("Connecting to session bus");
            Builder::session()?
                .name(SERVICE_NAME)?
                .serve_at(OBJECT_PATH, ai_service)?
                .build()
                .await?
        }
        _ => {
            tracing::info!("Connecting to system bus");
            Builder::system()?
                .name(SERVICE_NAME)?
                .serve_at(OBJECT_PATH, ai_service)?
                .build()
                .await?
        }
    };

    tracing::info!(
        "D-Bus service registered: {} at {}",
        SERVICE_NAME,
        OBJECT_PATH
    );

    // Wait for shutdown signal
    wait_for_shutdown().await;

    tracing::info!("Shutting down Winux AI Service");
    drop(connection);

    Ok(())
}

/// Initialize logging with tracing
fn init_logging(log_level: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    // Try to use journald for system services
    let journald_layer = tracing_journald::layer()
        .ok();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer);

    if let Some(journald) = journald_layer {
        subscriber.with(journald).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

/// Wait for termination signals (SIGTERM, SIGINT)
async fn wait_for_shutdown() {
    let mut sigterm = signal(SignalKind::terminate())
        .expect("Failed to register SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt())
        .expect("Failed to register SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM");
        }
        _ = sigint.recv() => {
            tracing::info!("Received SIGINT");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.service.bus_type, "system");
        assert!(config.cache.enabled);
    }
}
