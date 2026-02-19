// Clipboard Manager - System clipboard integration

use anyhow::{anyhow, Result};
use arboard::Clipboard;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static CLIPBOARD: Lazy<Mutex<Option<Clipboard>>> = Lazy::new(|| {
    Mutex::new(Clipboard::new().ok())
});

pub struct ClipboardManager;

impl ClipboardManager {
    /// Get text from clipboard
    pub fn get_text() -> Result<String> {
        let mut guard = CLIPBOARD.lock().map_err(|_| anyhow!("Clipboard lock poisoned"))?;
        let clipboard = guard.as_mut().ok_or_else(|| anyhow!("Clipboard not available"))?;
        clipboard.get_text().map_err(|e| anyhow!("Failed to get clipboard text: {}", e))
    }

    /// Set text to clipboard
    pub fn set_text(text: &str) -> Result<()> {
        let mut guard = CLIPBOARD.lock().map_err(|_| anyhow!("Clipboard lock poisoned"))?;
        let clipboard = guard.as_mut().ok_or_else(|| anyhow!("Clipboard not available"))?;
        clipboard.set_text(text).map_err(|e| anyhow!("Failed to set clipboard text: {}", e))
    }

    /// Copy code block to clipboard
    pub fn copy_code(code: &str) -> Result<()> {
        Self::set_text(code)
    }

    /// Check if clipboard has text
    pub fn has_text() -> bool {
        Self::get_text().is_ok()
    }

    /// Get clipboard and process it
    pub fn get_and_process<F, T>(processor: F) -> Result<T>
    where
        F: FnOnce(&str) -> T,
    {
        let text = Self::get_text()?;
        Ok(processor(&text))
    }
}

/// Clipboard history manager (in-memory)
pub struct ClipboardHistory {
    items: Vec<ClipboardItem>,
    max_items: usize,
}

#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: Option<String>,
}

impl ClipboardHistory {
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
        }
    }

    pub fn add(&mut self, content: String, source: Option<String>) {
        // Don't add duplicates of the last item
        if let Some(last) = self.items.last() {
            if last.content == content {
                return;
            }
        }

        let item = ClipboardItem {
            content,
            timestamp: chrono::Utc::now(),
            source,
        };

        self.items.push(item);

        // Trim to max size
        if self.items.len() > self.max_items {
            self.items.remove(0);
        }
    }

    pub fn get_recent(&self, count: usize) -> Vec<&ClipboardItem> {
        self.items.iter().rev().take(count).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&ClipboardItem> {
        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| item.content.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
