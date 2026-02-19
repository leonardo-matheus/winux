// Winux Notes - Notebook Data Structure
// Copyright (c) 2026 Winux OS Project

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A notebook (folder) for organizing notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Notebook {
    pub fn new(name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            icon: None,
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    pub fn rename(&mut self, new_name: &str) {
        self.name = new_name.to_string();
        self.updated_at = Utc::now();
    }

    pub fn set_description(&mut self, description: Option<&str>) {
        self.description = description.map(|s| s.to_string());
        self.updated_at = Utc::now();
    }

    pub fn set_icon(&mut self, icon: Option<&str>) {
        self.icon = icon.map(|s| s.to_string());
        self.updated_at = Utc::now();
    }

    /// Get the icon name for display (default if none set)
    pub fn display_icon(&self) -> &str {
        self.icon.as_deref().unwrap_or("folder-symbolic")
    }
}

impl Default for Notebook {
    fn default() -> Self {
        Self::new("New Notebook")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notebook_creation() {
        let notebook = Notebook::new("Work Notes");
        assert_eq!(notebook.name, "Work Notes");
        assert!(notebook.description.is_none());
        assert!(notebook.parent_id.is_none());
    }

    #[test]
    fn test_notebook_builder() {
        let notebook = Notebook::new("Projects")
            .with_description("Work projects and tasks")
            .with_icon("folder-documents-symbolic");

        assert_eq!(notebook.name, "Projects");
        assert_eq!(notebook.description, Some("Work projects and tasks".to_string()));
        assert_eq!(notebook.icon, Some("folder-documents-symbolic".to_string()));
    }

    #[test]
    fn test_notebook_rename() {
        let mut notebook = Notebook::new("Old Name");
        let original_updated = notebook.updated_at;

        // Wait a tiny bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        notebook.rename("New Name");
        assert_eq!(notebook.name, "New Name");
        assert!(notebook.updated_at > original_updated);
    }
}
