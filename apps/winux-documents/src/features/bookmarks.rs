//! Document bookmarks management

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// Bookmark for a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub page: usize,
    pub label: Option<String>,
    pub created_at: i64,
}

/// Collection of bookmarks for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmarks {
    document_path: Option<String>,
    bookmarks: BTreeSet<usize>,
    labels: std::collections::HashMap<usize, String>,
    #[serde(skip)]
    dirty: bool,
}

impl Bookmarks {
    pub fn new() -> Self {
        Self {
            document_path: None,
            bookmarks: BTreeSet::new(),
            labels: std::collections::HashMap::new(),
            dirty: false,
        }
    }

    pub fn load_for_document(path: &Path) -> Self {
        let bookmarks_path = Self::bookmarks_file_path(path);

        if let Ok(content) = std::fs::read_to_string(&bookmarks_path) {
            if let Ok(bookmarks) = serde_json::from_str(&content) {
                return bookmarks;
            }
        }

        let mut bookmarks = Self::new();
        bookmarks.document_path = Some(path.to_string_lossy().to_string());
        bookmarks
    }

    fn bookmarks_file_path(doc_path: &Path) -> std::path::PathBuf {
        let bookmarks_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("winux-documents")
            .join("bookmarks");

        let _ = std::fs::create_dir_all(&bookmarks_dir);

        // Use hash of document path as filename
        let hash = format!("{:x}", hash_path(doc_path.to_string_lossy().as_bytes()));
        bookmarks_dir.join(format!("{}.json", hash))
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(ref doc_path) = self.document_path {
            let bookmarks_path = Self::bookmarks_file_path(Path::new(doc_path));
            let content = serde_json::to_string_pretty(self)?;
            std::fs::write(bookmarks_path, content)?;
        }
        Ok(())
    }

    /// Add a bookmark for a page
    pub fn add(&mut self, page: usize) {
        if self.bookmarks.insert(page) {
            self.dirty = true;
        }
    }

    /// Add a bookmark with a label
    pub fn add_with_label(&mut self, page: usize, label: String) {
        self.bookmarks.insert(page);
        self.labels.insert(page, label);
        self.dirty = true;
    }

    /// Remove a bookmark
    pub fn remove(&mut self, page: usize) -> bool {
        let removed = self.bookmarks.remove(&page);
        self.labels.remove(&page);
        if removed {
            self.dirty = true;
        }
        removed
    }

    /// Toggle bookmark for a page
    pub fn toggle(&mut self, page: usize) {
        if self.bookmarks.contains(&page) {
            self.remove(page);
        } else {
            self.add(page);
        }
    }

    /// Check if a page is bookmarked
    pub fn is_bookmarked(&self, page: usize) -> bool {
        self.bookmarks.contains(&page)
    }

    /// Get all bookmarked pages
    pub fn all_pages(&self) -> impl Iterator<Item = &usize> {
        self.bookmarks.iter()
    }

    /// Get bookmark count
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Get label for a bookmarked page
    pub fn get_label(&self, page: usize) -> Option<&str> {
        self.labels.get(&page).map(|s| s.as_str())
    }

    /// Set label for a bookmarked page
    pub fn set_label(&mut self, page: usize, label: String) {
        if self.bookmarks.contains(&page) {
            self.labels.insert(page, label);
            self.dirty = true;
        }
    }

    /// Get next bookmarked page after current
    pub fn next_bookmark(&self, current_page: usize) -> Option<usize> {
        self.bookmarks.range((current_page + 1)..).next().copied()
    }

    /// Get previous bookmarked page before current
    pub fn prev_bookmark(&self, current_page: usize) -> Option<usize> {
        if current_page == 0 {
            return None;
        }
        self.bookmarks.range(..current_page).next_back().copied()
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
        self.labels.clear();
        self.dirty = true;
    }

    /// Export bookmarks as list
    pub fn export(&self) -> Vec<Bookmark> {
        self.bookmarks
            .iter()
            .map(|&page| Bookmark {
                page,
                label: self.labels.get(&page).cloned(),
                created_at: 0, // Would need to track this
            })
            .collect()
    }

    /// Import bookmarks from list
    pub fn import(&mut self, bookmarks: Vec<Bookmark>) {
        for bm in bookmarks {
            self.bookmarks.insert(bm.page);
            if let Some(label) = bm.label {
                self.labels.insert(bm.page, label);
            }
        }
        self.dirty = true;
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Default for Bookmarks {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Bookmarks {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.save();
        }
    }
}

// Simple hash function for filename generation
fn hash_path(data: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for (i, &byte) in data.iter().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul(31u64.wrapping_pow(i as u32)));
    }
    hash
}
