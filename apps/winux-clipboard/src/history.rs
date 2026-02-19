//! Clipboard history management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Unique identifier for clipboard items
pub type ItemId = u64;

/// Content type of clipboard item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// Plain text content
    Text,
    /// HTML content (may also have text representation)
    Html,
    /// Image (PNG, JPEG, etc.)
    Image,
    /// File paths (URI list)
    Files,
    /// Rich text format
    Rtf,
}

impl ContentType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ContentType::Text => "Text",
            ContentType::Html => "HTML",
            ContentType::Image => "Image",
            ContentType::Files => "Files",
            ContentType::Rtf => "Rich Text",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            ContentType::Text => "text-x-generic-symbolic",
            ContentType::Html => "text-html-symbolic",
            ContentType::Image => "image-x-generic-symbolic",
            ContentType::Files => "folder-documents-symbolic",
            ContentType::Rtf => "x-office-document-symbolic",
        }
    }
}

/// A single clipboard item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier
    pub id: ItemId,

    /// Content type
    pub content_type: ContentType,

    /// Main content (text, file paths, or image hash)
    pub content: String,

    /// Preview text (truncated for display)
    pub preview: String,

    /// HTML content if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,

    /// Image path if content is image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,

    /// Image dimensions if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_size: Option<(u32, u32)>,

    /// File paths if content is files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_paths: Option<Vec<String>>,

    /// Source application (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_app: Option<String>,

    /// When the item was copied
    pub timestamp: DateTime<Utc>,

    /// Whether this item is pinned/favorited
    pub pinned: bool,

    /// Number of times this item has been used
    pub use_count: u32,

    /// Tags for organization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Content size in bytes
    pub size_bytes: usize,
}

impl ClipboardItem {
    /// Create a new text item
    pub fn new_text(id: ItemId, content: String, source_app: Option<String>) -> Self {
        let preview = Self::make_preview(&content, 100);
        let size_bytes = content.len();

        Self {
            id,
            content_type: ContentType::Text,
            content,
            preview,
            html: None,
            image_path: None,
            image_size: None,
            file_paths: None,
            source_app,
            timestamp: Utc::now(),
            pinned: false,
            use_count: 0,
            tags: Vec::new(),
            size_bytes,
        }
    }

    /// Create a new HTML item
    pub fn new_html(
        id: ItemId,
        html: String,
        text: Option<String>,
        source_app: Option<String>,
    ) -> Self {
        let text_content = text.unwrap_or_else(|| Self::strip_html(&html));
        let preview = Self::make_preview(&text_content, 100);
        let size_bytes = html.len() + text_content.len();

        Self {
            id,
            content_type: ContentType::Html,
            content: text_content,
            preview,
            html: Some(html),
            image_path: None,
            image_size: None,
            file_paths: None,
            source_app,
            timestamp: Utc::now(),
            pinned: false,
            use_count: 0,
            tags: Vec::new(),
            size_bytes,
        }
    }

    /// Create a new image item
    pub fn new_image(
        id: ItemId,
        image_path: String,
        size: (u32, u32),
        file_size: usize,
        source_app: Option<String>,
    ) -> Self {
        let preview = format!("Image {}x{}", size.0, size.1);

        Self {
            id,
            content_type: ContentType::Image,
            content: String::new(),
            preview,
            html: None,
            image_path: Some(image_path),
            image_size: Some(size),
            file_paths: None,
            source_app,
            timestamp: Utc::now(),
            pinned: false,
            use_count: 0,
            tags: Vec::new(),
            size_bytes: file_size,
        }
    }

    /// Create a new files item
    pub fn new_files(id: ItemId, paths: Vec<String>, source_app: Option<String>) -> Self {
        let preview = if paths.len() == 1 {
            paths[0]
                .rsplit('/')
                .next()
                .unwrap_or(&paths[0])
                .to_string()
        } else {
            format!("{} files", paths.len())
        };

        let content = paths.join("\n");
        let size_bytes = content.len();

        Self {
            id,
            content_type: ContentType::Files,
            content,
            preview,
            html: None,
            image_path: None,
            image_size: None,
            file_paths: Some(paths),
            source_app,
            timestamp: Utc::now(),
            pinned: false,
            use_count: 0,
            tags: Vec::new(),
            size_bytes,
        }
    }

    /// Create a preview string from content
    fn make_preview(content: &str, max_len: usize) -> String {
        let trimmed = content.trim();
        let single_line = trimmed.lines().next().unwrap_or(trimmed);

        if single_line.len() > max_len {
            format!("{}...", &single_line[..max_len])
        } else if trimmed.lines().count() > 1 {
            format!("{}...", single_line)
        } else {
            single_line.to_string()
        }
    }

    /// Strip HTML tags for preview
    fn strip_html(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }

        // Decode common HTML entities
        result
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
    }

    /// Check if content matches search query
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.content.to_lowercase().contains(&query_lower)
            || self.preview.to_lowercase().contains(&query_lower)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
    }

    /// Format timestamp for display
    pub fn format_time(&self) -> String {
        let now = Utc::now();
        let diff = now.signed_duration_since(self.timestamp);

        if diff.num_seconds() < 60 {
            "Just now".to_string()
        } else if diff.num_minutes() < 60 {
            let mins = diff.num_minutes();
            if mins == 1 {
                "1 minute ago".to_string()
            } else {
                format!("{} minutes ago", mins)
            }
        } else if diff.num_hours() < 24 {
            let hours = diff.num_hours();
            if hours == 1 {
                "1 hour ago".to_string()
            } else {
                format!("{} hours ago", hours)
            }
        } else if diff.num_days() < 7 {
            let days = diff.num_days();
            if days == 1 {
                "Yesterday".to_string()
            } else {
                format!("{} days ago", days)
            }
        } else {
            self.timestamp.format("%b %d, %Y").to_string()
        }
    }

    /// Format size for display
    pub fn format_size(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Clipboard history manager
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClipboardHistory {
    /// List of clipboard items (newest first)
    items: Vec<ClipboardItem>,

    /// Next available ID
    next_id: ItemId,

    /// Maximum number of items to keep
    #[serde(skip)]
    max_items: usize,
}

impl ClipboardHistory {
    /// Create a new history with the given capacity
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
            max_items,
        }
    }

    /// Set maximum items
    pub fn set_max_items(&mut self, max: usize) {
        self.max_items = max;
        self.trim();
    }

    /// Get all items
    pub fn items(&self) -> &[ClipboardItem] {
        &self.items
    }

    /// Get pinned items
    pub fn pinned_items(&self) -> Vec<&ClipboardItem> {
        self.items.iter().filter(|i| i.pinned).collect()
    }

    /// Get item by ID
    pub fn get(&self, id: ItemId) -> Option<&ClipboardItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Get mutable item by ID
    pub fn get_mut(&mut self, id: ItemId) -> Option<&mut ClipboardItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Add a new item to history
    pub fn add(&mut self, mut item: ClipboardItem) -> ItemId {
        // Check for duplicates (same content)
        if let Some(existing_idx) = self.find_duplicate(&item) {
            // Move existing to front and update timestamp
            let mut existing = self.items.remove(existing_idx);
            existing.timestamp = Utc::now();
            existing.use_count += 1;
            let id = existing.id;
            self.items.insert(0, existing);
            return id;
        }

        item.id = self.next_id;
        self.next_id += 1;

        let id = item.id;
        self.items.insert(0, item);
        self.trim();

        id
    }

    /// Find duplicate item index
    fn find_duplicate(&self, item: &ClipboardItem) -> Option<usize> {
        self.items.iter().position(|i| {
            i.content_type == item.content_type && i.content == item.content
        })
    }

    /// Trim history to max size (keeping pinned items)
    fn trim(&mut self) {
        if self.max_items == 0 {
            return;
        }

        // Separate pinned and unpinned
        let pinned_count = self.items.iter().filter(|i| i.pinned).count();

        // Remove oldest unpinned items if over limit
        while self.items.len() > self.max_items + pinned_count {
            // Find last unpinned item
            if let Some(idx) = self.items.iter().rposition(|i| !i.pinned) {
                self.items.remove(idx);
            } else {
                break;
            }
        }
    }

    /// Remove an item by ID
    pub fn remove(&mut self, id: ItemId) -> Option<ClipboardItem> {
        if let Some(idx) = self.items.iter().position(|i| i.id == id) {
            Some(self.items.remove(idx))
        } else {
            None
        }
    }

    /// Toggle pin status
    pub fn toggle_pin(&mut self, id: ItemId) -> bool {
        if let Some(item) = self.get_mut(id) {
            item.pinned = !item.pinned;
            item.pinned
        } else {
            false
        }
    }

    /// Mark item as used (increment counter, move to front)
    pub fn mark_used(&mut self, id: ItemId) {
        if let Some(idx) = self.items.iter().position(|i| i.id == id) {
            let mut item = self.items.remove(idx);
            item.use_count += 1;
            self.items.insert(0, item);
        }
    }

    /// Search items
    pub fn search(&self, query: &str) -> Vec<&ClipboardItem> {
        if query.is_empty() {
            return self.items.iter().collect();
        }

        self.items
            .iter()
            .filter(|i| i.matches_search(query))
            .collect()
    }

    /// Filter by content type
    pub fn filter_by_type(&self, content_type: &ContentType) -> Vec<&ClipboardItem> {
        self.items
            .iter()
            .filter(|i| &i.content_type == content_type)
            .collect()
    }

    /// Clear all non-pinned items
    pub fn clear_unpinned(&mut self) {
        self.items.retain(|i| i.pinned);
    }

    /// Clear all items
    pub fn clear_all(&mut self) {
        self.items.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> HistoryStats {
        let mut stats = HistoryStats::default();

        for item in &self.items {
            stats.total_items += 1;
            stats.total_size += item.size_bytes;

            if item.pinned {
                stats.pinned_items += 1;
            }

            match item.content_type {
                ContentType::Text => stats.text_items += 1,
                ContentType::Html => stats.html_items += 1,
                ContentType::Image => stats.image_items += 1,
                ContentType::Files => stats.file_items += 1,
                ContentType::Rtf => stats.rtf_items += 1,
            }
        }

        stats
    }

    /// Get all unique tags
    pub fn all_tags(&self) -> HashSet<String> {
        self.items
            .iter()
            .flat_map(|i| i.tags.iter().cloned())
            .collect()
    }

    /// Export history to JSON
    pub fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.items).unwrap_or_default()
    }
}

/// Statistics about the clipboard history
#[derive(Debug, Default)]
pub struct HistoryStats {
    pub total_items: usize,
    pub pinned_items: usize,
    pub text_items: usize,
    pub html_items: usize,
    pub image_items: usize,
    pub file_items: usize,
    pub rtf_items: usize,
    pub total_size: usize,
}

impl HistoryStats {
    pub fn format_size(&self) -> String {
        if self.total_size < 1024 {
            format!("{} B", self.total_size)
        } else if self.total_size < 1024 * 1024 {
            format!("{:.1} KB", self.total_size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.total_size as f64 / (1024.0 * 1024.0))
        }
    }
}
