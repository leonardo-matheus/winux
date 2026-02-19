// Winux Notes - Note Data Structure
// Copyright (c) 2026 Winux OS Project

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Available note colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NoteColor {
    #[default]
    Default,
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Blue,
    Purple,
    Pink,
    Brown,
    Gray,
}

impl NoteColor {
    pub fn to_css_class(&self) -> &'static str {
        match self {
            NoteColor::Default => "note-default",
            NoteColor::Red => "note-red",
            NoteColor::Orange => "note-orange",
            NoteColor::Yellow => "note-yellow",
            NoteColor::Green => "note-green",
            NoteColor::Teal => "note-teal",
            NoteColor::Blue => "note-blue",
            NoteColor::Purple => "note-purple",
            NoteColor::Pink => "note-pink",
            NoteColor::Brown => "note-brown",
            NoteColor::Gray => "note-gray",
        }
    }

    pub fn to_hex(&self) -> &'static str {
        match self {
            NoteColor::Default => "#2d2d2d",
            NoteColor::Red => "#5c2b29",
            NoteColor::Orange => "#614a19",
            NoteColor::Yellow => "#635d19",
            NoteColor::Green => "#345920",
            NoteColor::Teal => "#16504b",
            NoteColor::Blue => "#2d555e",
            NoteColor::Purple => "#42275e",
            NoteColor::Pink => "#5b2245",
            NoteColor::Brown => "#442f19",
            NoteColor::Gray => "#3c3f43",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "red" => NoteColor::Red,
            "orange" => NoteColor::Orange,
            "yellow" => NoteColor::Yellow,
            "green" => NoteColor::Green,
            "teal" => NoteColor::Teal,
            "blue" => NoteColor::Blue,
            "purple" => NoteColor::Purple,
            "pink" => NoteColor::Pink,
            "brown" => NoteColor::Brown,
            "gray" | "grey" => NoteColor::Gray,
            _ => NoteColor::Default,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            NoteColor::Default => "default",
            NoteColor::Red => "red",
            NoteColor::Orange => "orange",
            NoteColor::Yellow => "yellow",
            NoteColor::Green => "green",
            NoteColor::Teal => "teal",
            NoteColor::Blue => "blue",
            NoteColor::Purple => "purple",
            NoteColor::Pink => "pink",
            NoteColor::Brown => "brown",
            NoteColor::Gray => "gray",
        }
    }

    pub fn all() -> Vec<NoteColor> {
        vec![
            NoteColor::Default,
            NoteColor::Red,
            NoteColor::Orange,
            NoteColor::Yellow,
            NoteColor::Green,
            NoteColor::Teal,
            NoteColor::Blue,
            NoteColor::Purple,
            NoteColor::Pink,
            NoteColor::Brown,
            NoteColor::Gray,
        ]
    }
}

/// A note item in a checklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub text: String,
    pub checked: bool,
}

impl ChecklistItem {
    pub fn new(text: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text: text.to_string(),
            checked: false,
        }
    }
}

/// A note with all its metadata and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub notebook_id: Option<String>,
    pub color: NoteColor,
    pub pinned: bool,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub checklist: Vec<ChecklistItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(title: &str, content: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: content.to_string(),
            notebook_id: None,
            color: NoteColor::Default,
            pinned: false,
            favorite: false,
            tags: Vec::new(),
            checklist: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_notebook(mut self, notebook_id: &str) -> Self {
        self.notebook_id = Some(notebook_id.to_string());
        self
    }

    pub fn with_color(mut self, color: NoteColor) -> Self {
        self.color = color;
        self
    }

    pub fn with_pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    pub fn with_favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    pub fn add_checklist_item(&mut self, text: &str) {
        self.checklist.push(ChecklistItem::new(text));
        self.updated_at = Utc::now();
    }

    pub fn toggle_checklist_item(&mut self, item_id: &str) {
        if let Some(item) = self.checklist.iter_mut().find(|i| i.id == item_id) {
            item.checked = !item.checked;
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_checklist_item(&mut self, item_id: &str) {
        self.checklist.retain(|i| i.id != item_id);
        self.updated_at = Utc::now();
    }

    pub fn update_content(&mut self, title: &str, content: &str) {
        self.title = title.to_string();
        self.content = content.to_string();
        self.updated_at = Utc::now();
    }

    /// Get a preview of the content (first N characters)
    pub fn content_preview(&self, max_len: usize) -> String {
        if self.content.len() <= max_len {
            self.content.clone()
        } else {
            format!("{}...", &self.content[..max_len.min(self.content.len())])
        }
    }

    /// Format the updated_at timestamp for display
    pub fn formatted_date(&self) -> String {
        self.updated_at.format("%b %d, %Y").to_string()
    }

    /// Format relative time (e.g., "2 hours ago")
    pub fn relative_time(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.updated_at);

        if duration.num_minutes() < 1 {
            "Just now".to_string()
        } else if duration.num_hours() < 1 {
            let mins = duration.num_minutes();
            format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if duration.num_days() < 1 {
            let hours = duration.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if duration.num_days() < 7 {
            let days = duration.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else {
            self.formatted_date()
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Self::new("Untitled", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = Note::new("Test Note", "This is a test");
        assert_eq!(note.title, "Test Note");
        assert_eq!(note.content, "This is a test");
        assert!(!note.pinned);
        assert!(!note.favorite);
        assert!(note.tags.is_empty());
    }

    #[test]
    fn test_note_builder() {
        let note = Note::new("Test", "Content")
            .with_color(NoteColor::Blue)
            .with_pinned(true)
            .with_tags(vec!["work".to_string(), "important".to_string()]);

        assert_eq!(note.color, NoteColor::Blue);
        assert!(note.pinned);
        assert_eq!(note.tags.len(), 2);
    }

    #[test]
    fn test_checklist() {
        let mut note = Note::new("Shopping", "");
        note.add_checklist_item("Milk");
        note.add_checklist_item("Bread");

        assert_eq!(note.checklist.len(), 2);
        assert!(!note.checklist[0].checked);

        let item_id = note.checklist[0].id.clone();
        note.toggle_checklist_item(&item_id);
        assert!(note.checklist[0].checked);
    }
}
