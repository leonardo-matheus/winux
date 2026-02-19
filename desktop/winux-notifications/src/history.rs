//! Notification history management
//!
//! Stores and manages notification history for the notification center.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::notification::Notification;

/// Maximum number of notifications to keep in history
const MAX_HISTORY_SIZE: usize = 500;

/// How long to keep notifications (in days)
const MAX_HISTORY_AGE_DAYS: i64 = 7;

/// Notification history storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationHistory {
    /// All notifications, keyed by ID
    notifications: Vec<Notification>,
    /// Index for grouping by app
    #[serde(skip)]
    app_index: HashMap<String, Vec<usize>>,
}

impl Default for NotificationHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            app_index: HashMap::new(),
        }
    }

    /// Get the history file path
    fn history_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("org", "winux", "notifications")
            .context("Failed to determine data directory")?;

        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).context("Failed to create data directory")?;

        Ok(data_dir.join("history.json"))
    }

    /// Load history from disk
    pub fn load() -> Result<Self> {
        let history_path = Self::history_path()?;

        if history_path.exists() {
            info!("Loading notification history from {:?}", history_path);
            let content = fs::read_to_string(&history_path)
                .context("Failed to read history file")?;

            let mut history: NotificationHistory = serde_json::from_str(&content)
                .context("Failed to parse history file")?;

            // Rebuild the app index
            history.rebuild_index();

            // Prune old notifications
            history.prune();

            debug!("Loaded {} notifications from history", history.notifications.len());
            Ok(history)
        } else {
            info!("No history file found, starting fresh");
            Ok(Self::new())
        }
    }

    /// Save history to disk
    pub fn save(&self) -> Result<()> {
        let history_path = Self::history_path()?;

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize history")?;

        fs::write(&history_path, content)
            .context("Failed to write history file")?;

        debug!("Saved {} notifications to history", self.notifications.len());
        Ok(())
    }

    /// Rebuild the app index
    fn rebuild_index(&mut self) {
        self.app_index.clear();
        for (idx, notification) in self.notifications.iter().enumerate() {
            self.app_index
                .entry(notification.app_name.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }
    }

    /// Add a notification to history
    pub fn add(&mut self, notification: Notification) {
        // Don't add transient notifications
        if notification.is_transient() {
            debug!("Skipping transient notification");
            return;
        }

        // Check if this replaces an existing notification
        if notification.replaces_id > 0 {
            if let Some(pos) = self.notifications.iter().position(|n| n.id == notification.replaces_id) {
                self.notifications[pos] = notification;
                self.rebuild_index();
                return;
            }
        }

        // Add to index
        let idx = self.notifications.len();
        self.app_index
            .entry(notification.app_name.clone())
            .or_insert_with(Vec::new)
            .push(idx);

        // Add notification
        self.notifications.push(notification);

        // Prune if over limit
        if self.notifications.len() > MAX_HISTORY_SIZE {
            self.prune();
        }
    }

    /// Remove a notification from history
    pub fn remove(&mut self, id: u32) -> Option<Notification> {
        if let Some(pos) = self.notifications.iter().position(|n| n.id == id) {
            let notification = self.notifications.remove(pos);
            self.rebuild_index();
            Some(notification)
        } else {
            None
        }
    }

    /// Get a notification by ID
    pub fn get(&self, id: u32) -> Option<&Notification> {
        self.notifications.iter().find(|n| n.id == id)
    }

    /// Get a mutable notification by ID
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Notification> {
        self.notifications.iter_mut().find(|n| n.id == id)
    }

    /// Mark a notification as read
    pub fn mark_read(&mut self, id: u32) {
        if let Some(notification) = self.get_mut(id) {
            notification.read = true;
        }
    }

    /// Mark all notifications as read
    pub fn mark_all_read(&mut self) {
        for notification in &mut self.notifications {
            notification.read = true;
        }
    }

    /// Get all notifications (most recent first)
    pub fn all(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.iter().rev()
    }

    /// Get unread notifications
    pub fn unread(&self) -> impl Iterator<Item = &Notification> {
        self.notifications.iter().filter(|n| !n.read).rev()
    }

    /// Get notifications for a specific app
    pub fn by_app(&self, app_name: &str) -> Vec<&Notification> {
        if let Some(indices) = self.app_index.get(app_name) {
            indices
                .iter()
                .filter_map(|&idx| self.notifications.get(idx))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all app names with notifications
    pub fn apps(&self) -> Vec<&String> {
        self.app_index.keys().collect()
    }

    /// Get notifications grouped by app
    pub fn grouped_by_app(&self) -> HashMap<&String, Vec<&Notification>> {
        let mut groups: HashMap<&String, Vec<&Notification>> = HashMap::new();

        for notification in &self.notifications {
            groups
                .entry(&notification.app_name)
                .or_insert_with(Vec::new)
                .push(notification);
        }

        // Sort each group by timestamp (most recent first)
        for notifications in groups.values_mut() {
            notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }

        groups
    }

    /// Get the number of notifications
    pub fn len(&self) -> usize {
        self.notifications.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    /// Get the number of unread notifications
    pub fn unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.read).count()
    }

    /// Clear all notifications
    pub fn clear(&mut self) {
        self.notifications.clear();
        self.app_index.clear();
    }

    /// Clear notifications for a specific app
    pub fn clear_app(&mut self, app_name: &str) {
        self.notifications.retain(|n| n.app_name != app_name);
        self.rebuild_index();
    }

    /// Prune old and excess notifications
    fn prune(&mut self) {
        let cutoff = Local::now() - Duration::days(MAX_HISTORY_AGE_DAYS);

        // Remove old notifications
        self.notifications.retain(|n| n.timestamp > cutoff);

        // Remove excess notifications (keep most recent)
        if self.notifications.len() > MAX_HISTORY_SIZE {
            // Sort by timestamp descending
            self.notifications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            self.notifications.truncate(MAX_HISTORY_SIZE);
        }

        self.rebuild_index();
    }

    /// Get notifications from the last N hours
    pub fn recent(&self, hours: i64) -> Vec<&Notification> {
        let cutoff = Local::now() - Duration::hours(hours);
        self.notifications
            .iter()
            .filter(|n| n.timestamp > cutoff)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::{NotificationHints, Urgency};

    fn create_test_notification(id: u32, app_name: &str) -> Notification {
        Notification {
            id,
            app_name: app_name.to_string(),
            app_icon: "test-icon".to_string(),
            summary: "Test Summary".to_string(),
            body: "Test Body".to_string(),
            actions: vec![],
            hints: NotificationHints::default(),
            expire_timeout: -1,
            timestamp: Local::now(),
            replaces_id: 0,
            read: false,
        }
    }

    #[test]
    fn test_add_and_get() {
        let mut history = NotificationHistory::new();
        let notification = create_test_notification(1, "TestApp");

        history.add(notification.clone());

        assert_eq!(history.len(), 1);
        assert!(history.get(1).is_some());
        assert_eq!(history.get(1).unwrap().app_name, "TestApp");
    }

    #[test]
    fn test_remove() {
        let mut history = NotificationHistory::new();
        history.add(create_test_notification(1, "TestApp"));
        history.add(create_test_notification(2, "TestApp"));

        let removed = history.remove(1);
        assert!(removed.is_some());
        assert_eq!(history.len(), 1);
        assert!(history.get(1).is_none());
    }

    #[test]
    fn test_grouped_by_app() {
        let mut history = NotificationHistory::new();
        history.add(create_test_notification(1, "App1"));
        history.add(create_test_notification(2, "App1"));
        history.add(create_test_notification(3, "App2"));

        let grouped = history.grouped_by_app();
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&"App1".to_string()).unwrap().len(), 2);
        assert_eq!(grouped.get(&"App2".to_string()).unwrap().len(), 1);
    }

    #[test]
    fn test_mark_read() {
        let mut history = NotificationHistory::new();
        history.add(create_test_notification(1, "TestApp"));

        assert_eq!(history.unread_count(), 1);

        history.mark_read(1);
        assert_eq!(history.unread_count(), 0);
    }
}
