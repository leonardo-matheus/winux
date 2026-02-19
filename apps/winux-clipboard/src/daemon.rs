//! Clipboard monitoring daemon

use anyhow::{Context, Result};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::config::{security, Config};
use crate::history::{ClipboardHistory, ClipboardItem, ContentType};
use crate::storage::Storage;

/// Events from the clipboard daemon
#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    /// New item added to clipboard
    NewItem(ClipboardItem),
    /// Clipboard cleared
    Cleared,
    /// Error occurred
    Error(String),
}

/// Messages to control the daemon
#[derive(Debug)]
pub enum DaemonCommand {
    /// Stop the daemon
    Stop,
    /// Get current clipboard content
    GetCurrent,
    /// Set clipboard content
    SetContent(String, ContentType),
    /// Clear clipboard
    Clear,
}

/// Clipboard daemon that monitors wl-clipboard
pub struct ClipboardDaemon {
    config: Config,
    history: Arc<RwLock<ClipboardHistory>>,
    storage: Arc<RwLock<Storage>>,
    event_tx: mpsc::Sender<ClipboardEvent>,
    command_rx: mpsc::Receiver<DaemonCommand>,
}

impl ClipboardDaemon {
    /// Create a new clipboard daemon
    pub fn new(
        config: Config,
        history: Arc<RwLock<ClipboardHistory>>,
        storage: Arc<RwLock<Storage>>,
        event_tx: mpsc::Sender<ClipboardEvent>,
        command_rx: mpsc::Receiver<DaemonCommand>,
    ) -> Self {
        Self {
            config,
            history,
            storage,
            event_tx,
            command_rx,
        }
    }

    /// Run the daemon
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting clipboard daemon");

        // Start clipboard watcher
        let watcher_handle = self.start_watcher();

        // Process commands
        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                DaemonCommand::Stop => {
                    info!("Stopping clipboard daemon");
                    break;
                }
                DaemonCommand::GetCurrent => {
                    if let Err(e) = self.get_current_clipboard().await {
                        error!("Failed to get clipboard: {}", e);
                    }
                }
                DaemonCommand::SetContent(content, content_type) => {
                    if let Err(e) = self.set_clipboard(&content, &content_type).await {
                        error!("Failed to set clipboard: {}", e);
                    }
                }
                DaemonCommand::Clear => {
                    if let Err(e) = self.clear_clipboard().await {
                        error!("Failed to clear clipboard: {}", e);
                    }
                }
            }
        }

        watcher_handle.abort();
        Ok(())
    }

    /// Start watching clipboard for changes
    fn start_watcher(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let history = self.history.clone();
        let storage = self.storage.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                match Self::watch_clipboard_once(&config, &history, &storage, &event_tx).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Clipboard watcher error: {}", e);
                        let _ = event_tx.send(ClipboardEvent::Error(e.to_string())).await;
                        // Wait before retrying
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        })
    }

    /// Watch for a single clipboard change
    async fn watch_clipboard_once(
        config: &Config,
        history: &Arc<RwLock<ClipboardHistory>>,
        storage: &Arc<RwLock<Storage>>,
        event_tx: &mpsc::Sender<ClipboardEvent>,
    ) -> Result<()> {
        // Use wl-paste with --watch to monitor clipboard changes
        let mut child = Command::new("wl-paste")
            .args(["--watch", "echo", "CLIPBOARD_CHANGED"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start wl-paste --watch")?;

        let stdout = child.stdout.take().context("No stdout")?;
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await? {
            if line == "CLIPBOARD_CHANGED" {
                debug!("Clipboard changed, fetching content");

                // Get the content type first
                let mime_types = Self::get_mime_types().await?;
                debug!("Available MIME types: {:?}", mime_types);

                // Determine content type and fetch appropriate data
                let item = if mime_types.iter().any(|m| m.starts_with("image/")) {
                    Self::fetch_image(config, storage).await?
                } else if mime_types.contains(&"text/html".to_string()) && config.store_html {
                    Self::fetch_html(config).await?
                } else if mime_types.contains(&"text/uri-list".to_string()) && config.store_files {
                    Self::fetch_files(config).await?
                } else {
                    Self::fetch_text(config).await?
                };

                if let Some(item) = item {
                    // Add to history
                    let mut history = history.write().await;
                    let id = history.add(item.clone());

                    // Save history
                    let storage = storage.read().await;
                    if let Err(e) = storage.save_history(&history) {
                        error!("Failed to save history: {}", e);
                    }

                    // Notify listeners
                    let _ = event_tx.send(ClipboardEvent::NewItem(item)).await;

                    debug!("Added clipboard item with ID {}", id);
                }
            }
        }

        Ok(())
    }

    /// Get available MIME types from clipboard
    async fn get_mime_types() -> Result<Vec<String>> {
        let output = Command::new("wl-paste")
            .args(["--list-types"])
            .output()
            .await
            .context("Failed to get MIME types")?;

        let types: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(types)
    }

    /// Fetch text content from clipboard
    async fn fetch_text(config: &Config) -> Result<Option<ClipboardItem>> {
        let output = Command::new("wl-paste")
            .args(["--no-newline"])
            .output()
            .await
            .context("Failed to get text from clipboard")?;

        if !output.status.success() || output.stdout.is_empty() {
            return Ok(None);
        }

        let content = String::from_utf8_lossy(&output.stdout).to_string();

        // Check size limit
        if content.len() > config.max_text_size {
            warn!("Text content too large, skipping");
            return Ok(None);
        }

        // Check for password-like content
        if config.skip_passwords && security::looks_like_password(&content) {
            debug!("Content looks like password, skipping");
            return Ok(None);
        }

        // Get source application
        let source_app = Self::get_focused_app().await;

        // Check if app is in ignore list
        if let Some(ref app) = source_app {
            if config.ignored_apps.iter().any(|a| app.to_lowercase().contains(&a.to_lowercase())) {
                debug!("App {} is in ignore list, skipping", app);
                return Ok(None);
            }
            if security::is_password_manager(app) {
                debug!("App {} is a password manager, skipping", app);
                return Ok(None);
            }
        }

        Ok(Some(ClipboardItem::new_text(0, content, source_app)))
    }

    /// Fetch HTML content from clipboard
    async fn fetch_html(config: &Config) -> Result<Option<ClipboardItem>> {
        // Get HTML content
        let html_output = Command::new("wl-paste")
            .args(["--type", "text/html"])
            .output()
            .await
            .context("Failed to get HTML from clipboard")?;

        if !html_output.status.success() || html_output.stdout.is_empty() {
            return Self::fetch_text(config).await;
        }

        let html = String::from_utf8_lossy(&html_output.stdout).to_string();

        // Also get plain text version
        let text_output = Command::new("wl-paste")
            .args(["--no-newline"])
            .output()
            .await
            .ok();

        let text = text_output
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string());

        let source_app = Self::get_focused_app().await;

        Ok(Some(ClipboardItem::new_html(0, html, text, source_app)))
    }

    /// Fetch image from clipboard
    async fn fetch_image(config: &Config, storage: &Arc<RwLock<Storage>>) -> Result<Option<ClipboardItem>> {
        // Get image as PNG
        let output = Command::new("wl-paste")
            .args(["--type", "image/png"])
            .output()
            .await
            .context("Failed to get image from clipboard")?;

        if !output.status.success() || output.stdout.is_empty() {
            return Ok(None);
        }

        let data = output.stdout;

        // Check size limit
        if data.len() > config.max_image_size {
            warn!("Image too large, skipping");
            return Ok(None);
        }

        // Get image dimensions
        let size = image::load_from_memory(&data)
            .map(|img| (img.width(), img.height()))
            .unwrap_or((0, 0));

        // Save image to storage
        let storage = storage.read().await;
        let path = storage.save_image(&data, "png")?;

        let source_app = Self::get_focused_app().await;

        Ok(Some(ClipboardItem::new_image(0, path, size, data.len(), source_app)))
    }

    /// Fetch files from clipboard
    async fn fetch_files(_config: &Config) -> Result<Option<ClipboardItem>> {
        let output = Command::new("wl-paste")
            .args(["--type", "text/uri-list"])
            .output()
            .await
            .context("Failed to get files from clipboard")?;

        if !output.status.success() || output.stdout.is_empty() {
            return Ok(None);
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let paths: Vec<String> = content
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| {
                // Convert file:// URIs to paths
                l.strip_prefix("file://")
                    .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
                    .unwrap_or_else(|| l.to_string())
            })
            .collect();

        if paths.is_empty() {
            return Ok(None);
        }

        let source_app = Self::get_focused_app().await;

        Ok(Some(ClipboardItem::new_files(0, paths, source_app)))
    }

    /// Get the currently focused application name
    async fn get_focused_app() -> Option<String> {
        // Try to get from Wayland compositor via D-Bus or other means
        // This is compositor-specific, so we use a fallback approach

        // Try wlrctl or similar
        let output = Command::new("wlrctl")
            .args(["toplevel", "focus"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }

        None
    }

    /// Get current clipboard content
    async fn get_current_clipboard(&self) -> Result<()> {
        if let Some(item) = Self::fetch_text(&self.config).await? {
            let _ = self.event_tx.send(ClipboardEvent::NewItem(item)).await;
        }
        Ok(())
    }

    /// Set clipboard content
    async fn set_clipboard(&self, content: &str, content_type: &ContentType) -> Result<()> {
        match content_type {
            ContentType::Text => {
                let mut child = Command::new("wl-copy")
                    .stdin(Stdio::piped())
                    .spawn()
                    .context("Failed to start wl-copy")?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(content.as_bytes()).await?;
                }

                child.wait().await?;
            }
            ContentType::Html => {
                let mut child = Command::new("wl-copy")
                    .args(["--type", "text/html"])
                    .stdin(Stdio::piped())
                    .spawn()
                    .context("Failed to start wl-copy")?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(content.as_bytes()).await?;
                }

                child.wait().await?;
            }
            ContentType::Image => {
                // Content is the image path
                let data = std::fs::read(content)?;
                let mut child = Command::new("wl-copy")
                    .args(["--type", "image/png"])
                    .stdin(Stdio::piped())
                    .spawn()
                    .context("Failed to start wl-copy")?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(&data).await?;
                }

                child.wait().await?;
            }
            ContentType::Files => {
                let uris: Vec<String> = content
                    .lines()
                    .map(|p| format!("file://{}", urlencoding::encode(p)))
                    .collect();

                let mut child = Command::new("wl-copy")
                    .args(["--type", "text/uri-list"])
                    .stdin(Stdio::piped())
                    .spawn()
                    .context("Failed to start wl-copy")?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(uris.join("\n").as_bytes()).await?;
                }

                child.wait().await?;
            }
            ContentType::Rtf => {
                let mut child = Command::new("wl-copy")
                    .args(["--type", "text/rtf"])
                    .stdin(Stdio::piped())
                    .spawn()
                    .context("Failed to start wl-copy")?;

                if let Some(stdin) = child.stdin.as_mut() {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(content.as_bytes()).await?;
                }

                child.wait().await?;
            }
        }

        Ok(())
    }

    /// Clear clipboard
    async fn clear_clipboard(&self) -> Result<()> {
        Command::new("wl-copy")
            .args(["--clear"])
            .output()
            .await
            .context("Failed to clear clipboard")?;

        let _ = self.event_tx.send(ClipboardEvent::Cleared).await;
        Ok(())
    }
}

/// URL encoding/decoding utilities
mod urlencoding {
    use std::borrow::Cow;

    pub fn encode(input: &str) -> String {
        let mut result = String::new();
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }

    pub fn decode(input: &str) -> Result<Cow<'_, str>, ()> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                result.push('%');
                result.push_str(&hex);
            } else {
                result.push(c);
            }
        }

        Ok(Cow::Owned(result))
    }
}
