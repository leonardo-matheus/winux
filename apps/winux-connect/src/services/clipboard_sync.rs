//! Clipboard synchronization service
//!
//! Automatically syncs clipboard content between PC and connected devices.
//! Supports text, URLs, and optionally images.

use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Maximum clipboard history size
const MAX_HISTORY_SIZE: usize = 100;

/// Clipboard sync service
pub struct ClipboardSyncService {
    running: Arc<RwLock<bool>>,
    enabled: Arc<RwLock<bool>>,
    history: Arc<RwLock<VecDeque<ClipboardEntry>>>,
    current_content: Arc<RwLock<Option<String>>>,
    sync_images: bool,
    sync_files: bool,
}

impl ClipboardSyncService {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            enabled: Arc::new(RwLock::new(true)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            current_content: Arc::new(RwLock::new(None)),
            sync_images: true,
            sync_files: false,
        }
    }

    /// Start the clipboard sync service
    pub fn start(&self) -> Result<(), String> {
        *self.running.write().unwrap() = true;

        // In production, would:
        // 1. Connect to clipboard manager via D-Bus or GTK
        // 2. Listen for clipboard changes
        // 3. Start sync loop

        tracing::info!("Clipboard sync service started");
        Ok(())
    }

    /// Stop the clipboard sync service
    pub fn stop(&self) {
        *self.running.write().unwrap() = false;
        tracing::info!("Clipboard sync service stopped");
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        *self.running.read().unwrap()
    }

    /// Enable/disable clipboard sync
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().unwrap() = enabled;
    }

    /// Check if sync is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read().unwrap()
    }

    /// Set image sync preference
    pub fn set_sync_images(&mut self, sync: bool) {
        self.sync_images = sync;
    }

    /// Set file sync preference
    pub fn set_sync_files(&mut self, sync: bool) {
        self.sync_files = sync;
    }

    /// Get clipboard history
    pub fn get_history(&self) -> Vec<ClipboardEntry> {
        self.history.read().unwrap().iter().cloned().collect()
    }

    /// Clear clipboard history
    pub fn clear_history(&self) {
        self.history.write().unwrap().clear();
    }

    /// Get current clipboard content
    pub fn get_current(&self) -> Option<String> {
        self.current_content.read().unwrap().clone()
    }

    /// Set clipboard content (from local)
    pub fn set_content(&self, content: &str) -> Result<(), String> {
        let entry = ClipboardEntry {
            content: content.to_string(),
            content_type: self.detect_content_type(content),
            timestamp: chrono::Utc::now().timestamp(),
            source: ClipboardSource::Local,
        };

        self.add_to_history(entry);
        *self.current_content.write().unwrap() = Some(content.to_string());

        tracing::debug!("Clipboard content set locally");
        Ok(())
    }

    /// Receive clipboard content from remote device
    pub fn receive_content(&self, content: &str, device_id: &str) -> Result<(), String> {
        if !*self.enabled.read().unwrap() {
            return Ok(());
        }

        let entry = ClipboardEntry {
            content: content.to_string(),
            content_type: self.detect_content_type(content),
            timestamp: chrono::Utc::now().timestamp(),
            source: ClipboardSource::Remote(device_id.to_string()),
        };

        self.add_to_history(entry);
        *self.current_content.write().unwrap() = Some(content.to_string());

        // In production, would set system clipboard here
        tracing::debug!("Clipboard content received from {}", device_id);
        Ok(())
    }

    /// Send current clipboard to device
    pub fn send_to_device(&self, _device_id: &str) -> Result<(), String> {
        if let Some(content) = self.get_current() {
            // In production, would send via connection manager
            tracing::debug!("Sending clipboard content: {}", &content[..content.len().min(50)]);
            Ok(())
        } else {
            Err("No clipboard content".to_string())
        }
    }

    /// Copy item from history to clipboard
    pub fn copy_from_history(&self, index: usize) -> Result<(), String> {
        let history = self.history.read().unwrap();
        if let Some(entry) = history.get(index) {
            let content = entry.content.clone();
            drop(history);
            self.set_content(&content)
        } else {
            Err("Invalid history index".to_string())
        }
    }

    /// Delete item from history
    pub fn delete_from_history(&self, index: usize) -> Result<(), String> {
        let mut history = self.history.write().unwrap();
        if index < history.len() {
            history.remove(index);
            Ok(())
        } else {
            Err("Invalid history index".to_string())
        }
    }

    /// Add entry to history
    fn add_to_history(&self, entry: ClipboardEntry) {
        let mut history = self.history.write().unwrap();

        // Remove duplicate if exists
        history.retain(|e| e.content != entry.content);

        // Add to front
        history.push_front(entry);

        // Trim to max size
        while history.len() > MAX_HISTORY_SIZE {
            history.pop_back();
        }
    }

    /// Detect content type from string
    fn detect_content_type(&self, content: &str) -> ClipboardContentType {
        let trimmed = content.trim();

        // Check for URL
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return ClipboardContentType::Url;
        }

        // Check for file path
        if trimmed.starts_with("/") || trimmed.starts_with("file://") {
            if std::path::Path::new(trimmed).exists() {
                return ClipboardContentType::FilePath;
            }
        }

        // Check for email
        if trimmed.contains('@') && !trimmed.contains(' ') {
            return ClipboardContentType::Email;
        }

        // Check for phone number (simple check)
        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 10 && digits.len() <= 15 {
            return ClipboardContentType::PhoneNumber;
        }

        ClipboardContentType::Text
    }

    /// Check if content should be filtered (sensitive data)
    pub fn should_filter(&self, content: &str) -> bool {
        let lower = content.to_lowercase();

        // Filter password manager content
        if lower.contains("password") || lower.contains("senha") {
            return true;
        }

        // Filter credit card numbers (simple check for 16 digits)
        let digits: String = content.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 16 {
            return true;
        }

        // Filter CPF (Brazilian ID)
        if digits.len() == 11 {
            return true;
        }

        false
    }
}

impl Default for ClipboardSyncService {
    fn default() -> Self {
        Self::new()
    }
}

/// Clipboard entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub content: String,
    pub content_type: ClipboardContentType,
    pub timestamp: i64,
    pub source: ClipboardSource,
}

/// Clipboard content type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardContentType {
    Text,
    Url,
    Email,
    PhoneNumber,
    FilePath,
    Image,
}

impl ClipboardContentType {
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Text => "edit-paste-symbolic",
            Self::Url => "web-browser-symbolic",
            Self::Email => "mail-unread-symbolic",
            Self::PhoneNumber => "phone-symbolic",
            Self::FilePath => "text-x-generic-symbolic",
            Self::Image => "image-x-generic-symbolic",
        }
    }
}

/// Clipboard content source
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardSource {
    Local,
    Remote(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_service_creation() {
        let service = ClipboardSyncService::new();
        assert!(!service.is_running());
        assert!(service.is_enabled());
    }

    #[test]
    fn test_content_type_detection() {
        let service = ClipboardSyncService::new();

        assert_eq!(
            service.detect_content_type("https://example.com"),
            ClipboardContentType::Url
        );
        assert_eq!(
            service.detect_content_type("test@example.com"),
            ClipboardContentType::Email
        );
        assert_eq!(
            service.detect_content_type("+5511999991234"),
            ClipboardContentType::PhoneNumber
        );
        assert_eq!(
            service.detect_content_type("Hello, World!"),
            ClipboardContentType::Text
        );
    }

    #[test]
    fn test_clipboard_history() {
        let service = ClipboardSyncService::new();

        service.set_content("First").unwrap();
        service.set_content("Second").unwrap();
        service.set_content("Third").unwrap();

        let history = service.get_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].content, "Third");
        assert_eq!(history[2].content, "First");
    }

    #[test]
    fn test_duplicate_removal() {
        let service = ClipboardSyncService::new();

        service.set_content("Same").unwrap();
        service.set_content("Other").unwrap();
        service.set_content("Same").unwrap();

        let history = service.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].content, "Same");
    }

    #[test]
    fn test_sensitive_filter() {
        let service = ClipboardSyncService::new();

        assert!(service.should_filter("mypassword123"));
        assert!(service.should_filter("1234567890123456")); // Credit card
        assert!(!service.should_filter("Hello World"));
    }
}
