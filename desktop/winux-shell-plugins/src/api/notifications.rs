//! Notification handler API
//!
//! Allows plugins to intercept, modify, and respond to notifications.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Notification urgency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

impl Default for NotificationUrgency {
    fn default() -> Self {
        Self::Normal
    }
}

/// A notification that can be handled by plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique notification ID
    pub id: u32,
    /// Application name
    pub app_name: String,
    /// Application icon
    pub app_icon: String,
    /// Notification title/summary
    pub summary: String,
    /// Notification body
    pub body: String,
    /// Urgency level
    pub urgency: NotificationUrgency,
    /// Category (e.g., "email", "im.received")
    pub category: Option<String>,
    /// Desktop entry of sending application
    pub desktop_entry: Option<String>,
    /// Available action IDs
    pub actions: Vec<(String, String)>,
    /// Timeout in milliseconds
    pub timeout: i32,
    /// Whether this is a transient notification
    pub transient: bool,
    /// Whether notification is resident (stays until dismissed)
    pub resident: bool,
    /// Progress value if any
    pub progress: Option<i32>,
    /// Image path if any
    pub image_path: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Local>,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            id: 0,
            app_name: String::new(),
            app_icon: String::new(),
            summary: String::new(),
            body: String::new(),
            urgency: NotificationUrgency::Normal,
            category: None,
            desktop_entry: None,
            actions: Vec::new(),
            timeout: -1,
            transient: false,
            resident: false,
            progress: None,
            image_path: None,
            timestamp: chrono::Local::now(),
        }
    }
}

/// Filter for notifications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationFilter {
    /// Filter by app names (empty = all)
    pub app_names: HashSet<String>,
    /// Filter by categories (empty = all)
    pub categories: HashSet<String>,
    /// Filter by urgency (empty = all)
    pub urgencies: HashSet<NotificationUrgency>,
    /// Filter by desktop entries (empty = all)
    pub desktop_entries: HashSet<String>,
    /// Whether to include transient notifications
    pub include_transient: bool,
    /// Regex pattern for summary (optional)
    pub summary_pattern: Option<String>,
    /// Regex pattern for body (optional)
    pub body_pattern: Option<String>,
}

impl NotificationFilter {
    /// Create a new filter that matches all notifications
    pub fn all() -> Self {
        Self {
            include_transient: true,
            ..Default::default()
        }
    }

    /// Create a filter for specific app names
    pub fn for_apps(apps: &[&str]) -> Self {
        Self {
            app_names: apps.iter().map(|s| s.to_string()).collect(),
            include_transient: true,
            ..Default::default()
        }
    }

    /// Create a filter for specific categories
    pub fn for_categories(categories: &[&str]) -> Self {
        Self {
            categories: categories.iter().map(|s| s.to_string()).collect(),
            include_transient: true,
            ..Default::default()
        }
    }

    /// Add app name filter
    pub fn with_app(mut self, app: &str) -> Self {
        self.app_names.insert(app.to_string());
        self
    }

    /// Add category filter
    pub fn with_category(mut self, category: &str) -> Self {
        self.categories.insert(category.to_string());
        self
    }

    /// Add urgency filter
    pub fn with_urgency(mut self, urgency: NotificationUrgency) -> Self {
        self.urgencies.insert(urgency);
        self
    }

    /// Check if a notification matches this filter
    pub fn matches(&self, notification: &Notification) -> bool {
        // Check transient
        if notification.transient && !self.include_transient {
            return false;
        }

        // Check app name
        if !self.app_names.is_empty() && !self.app_names.contains(&notification.app_name) {
            return false;
        }

        // Check category
        if !self.categories.is_empty() {
            if let Some(cat) = &notification.category {
                if !self.categories.contains(cat) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check urgency
        if !self.urgencies.is_empty() && !self.urgencies.contains(&notification.urgency) {
            return false;
        }

        // Check desktop entry
        if !self.desktop_entries.is_empty() {
            if let Some(entry) = &notification.desktop_entry {
                if !self.desktop_entries.contains(entry) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check patterns
        if let Some(pattern) = &self.summary_pattern {
            if let Ok(re) = regex::Regex::new(pattern) {
                if !re.is_match(&notification.summary) {
                    return false;
                }
            }
        }

        if let Some(pattern) = &self.body_pattern {
            if let Ok(re) = regex::Regex::new(pattern) {
                if !re.is_match(&notification.body) {
                    return false;
                }
            }
        }

        true
    }
}

/// Result of handling a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationHandlerResult {
    /// Let the notification pass through unchanged
    Pass,
    /// Modify the notification
    Modify(Box<Notification>),
    /// Block the notification (don't show it)
    Block,
    /// Show a different notification instead
    Replace(Box<Notification>),
}

/// Reason for notification close
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseReason {
    /// Notification expired
    Expired,
    /// User dismissed the notification
    Dismissed,
    /// Notification was closed programmatically
    Closed,
    /// Unknown reason
    Unknown,
}

/// Trait for notification handlers
pub trait NotificationHandler: Send + Sync {
    /// Get handler ID
    fn id(&self) -> &str;

    /// Get handler name
    fn name(&self) -> &str;

    /// Get the notification filter
    fn filter(&self) -> NotificationFilter {
        NotificationFilter::all()
    }

    /// Get priority (higher = called first)
    fn priority(&self) -> i32 {
        0
    }

    /// Handle an incoming notification
    fn handle(&mut self, notification: &Notification) -> NotificationHandlerResult {
        let _ = notification;
        NotificationHandlerResult::Pass
    }

    /// Called when a notification is closed
    fn on_close(&mut self, notification_id: u32, reason: CloseReason) {
        let _ = (notification_id, reason);
    }

    /// Called when a notification action is invoked
    fn on_action(&mut self, notification_id: u32, action_key: &str) {
        let _ = (notification_id, action_key);
    }

    /// Called when notifications are cleared
    fn on_clear_all(&mut self) {}
}

/// Builder for creating notification handlers
pub struct NotificationHandlerBuilder {
    id: String,
    name: String,
    filter: NotificationFilter,
    priority: i32,
}

impl NotificationHandlerBuilder {
    /// Create a new builder
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            filter: NotificationFilter::all(),
            priority: 0,
        }
    }

    /// Set the filter
    pub fn filter(mut self, filter: NotificationFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set the priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Get the ID
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Get the name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the filter
    pub fn get_filter(&self) -> &NotificationFilter {
        &self.filter
    }

    /// Get the priority
    pub fn get_priority(&self) -> i32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matches_all() {
        let filter = NotificationFilter::all();
        let notification = Notification::default();
        assert!(filter.matches(&notification));
    }

    #[test]
    fn test_filter_by_app() {
        let filter = NotificationFilter::for_apps(&["Firefox"]);

        let mut notification = Notification::default();
        notification.app_name = "Firefox".to_string();
        assert!(filter.matches(&notification));

        notification.app_name = "Chrome".to_string();
        assert!(!filter.matches(&notification));
    }

    #[test]
    fn test_filter_by_urgency() {
        let filter = NotificationFilter::all()
            .with_urgency(NotificationUrgency::Critical);

        let mut notification = Notification::default();
        notification.urgency = NotificationUrgency::Critical;
        assert!(filter.matches(&notification));

        notification.urgency = NotificationUrgency::Low;
        assert!(!filter.matches(&notification));
    }
}
