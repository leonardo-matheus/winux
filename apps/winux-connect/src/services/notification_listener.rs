//! D-Bus notification proxy for capturing desktop notifications
//!
//! Listens to org.freedesktop.Notifications interface to capture
//! notifications and forward them to connected devices.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Notification listener for D-Bus notifications
pub struct NotificationListener {
    running: Arc<RwLock<bool>>,
    notifications: Arc<RwLock<Vec<DesktopNotification>>>,
    app_filter: Arc<RwLock<HashMap<String, bool>>>,
}

impl NotificationListener {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            notifications: Arc::new(RwLock::new(Vec::new())),
            app_filter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start listening for notifications
    pub fn start(&self) -> Result<(), String> {
        *self.running.write().unwrap() = true;

        // In production, this would:
        // 1. Connect to session D-Bus
        // 2. Register as notification proxy
        // 3. Listen for NotificationClosed and ActionInvoked signals

        tracing::info!("Notification listener started");
        Ok(())
    }

    /// Stop listening for notifications
    pub fn stop(&self) {
        *self.running.write().unwrap() = false;
        tracing::info!("Notification listener stopped");
    }

    /// Check if listener is running
    pub fn is_running(&self) -> bool {
        *self.running.read().unwrap()
    }

    /// Get recent notifications
    pub fn get_notifications(&self) -> Vec<DesktopNotification> {
        self.notifications.read().unwrap().clone()
    }

    /// Clear notifications
    pub fn clear_notifications(&self) {
        self.notifications.write().unwrap().clear();
    }

    /// Add a notification (called from D-Bus handler)
    pub fn add_notification(&self, notification: DesktopNotification) {
        // Check app filter
        if let Some(allowed) = self.app_filter.read().unwrap().get(&notification.app_name) {
            if !*allowed {
                return;
            }
        }

        // Add to list (keep last 100)
        let mut notifications = self.notifications.write().unwrap();
        notifications.insert(0, notification);
        notifications.truncate(100);
    }

    /// Set app filter (which apps can send notifications)
    pub fn set_app_filter(&self, app_name: &str, allowed: bool) {
        self.app_filter.write().unwrap().insert(app_name.to_string(), allowed);
    }

    /// Get app filter settings
    pub fn get_app_filter(&self) -> HashMap<String, bool> {
        self.app_filter.read().unwrap().clone()
    }

    /// Dismiss a notification by ID
    pub fn dismiss_notification(&self, id: u32) -> Result<(), String> {
        // In production, would call CloseNotification on D-Bus
        tracing::info!("Dismissing notification {}", id);
        Ok(())
    }

    /// Invoke an action on a notification
    pub fn invoke_action(&self, id: u32, action_key: &str) -> Result<(), String> {
        // In production, would call ActionInvoked on D-Bus
        tracing::info!("Invoking action {} on notification {}", action_key, id);
        Ok(())
    }

    /// Reply to a notification (for notifications with reply support)
    pub fn reply_to_notification(&self, id: u32, reply: &str) -> Result<(), String> {
        // Some notifications support inline reply
        tracing::info!("Replying to notification {}: {}", id, reply);
        Ok(())
    }
}

impl Default for NotificationListener {
    fn default() -> Self {
        Self::new()
    }
}

/// Desktop notification data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesktopNotification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<NotificationAction>,
    pub hints: HashMap<String, String>,
    pub expire_timeout: i32,
    pub timestamp: i64,
    pub urgency: NotificationUrgency,
}

impl DesktopNotification {
    pub fn new(id: u32, app_name: &str, summary: &str, body: &str) -> Self {
        Self {
            id,
            app_name: app_name.to_string(),
            app_icon: String::new(),
            summary: summary.to_string(),
            body: body.to_string(),
            actions: Vec::new(),
            hints: HashMap::new(),
            expire_timeout: -1,
            timestamp: chrono::Utc::now().timestamp(),
            urgency: NotificationUrgency::Normal,
        }
    }
}

/// Notification action
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationAction {
    pub key: String,
    pub label: String,
}

/// Notification urgency level
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}

/// D-Bus interface for notifications (org.freedesktop.Notifications)
#[allow(dead_code)]
mod dbus {
    pub const NOTIFICATION_SERVICE: &str = "org.freedesktop.Notifications";
    pub const NOTIFICATION_PATH: &str = "/org/freedesktop/Notifications";
    pub const NOTIFICATION_INTERFACE: &str = "org.freedesktop.Notifications";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_listener_creation() {
        let listener = NotificationListener::new();
        assert!(!listener.is_running());
    }

    #[test]
    fn test_notification_listener_start_stop() {
        let listener = NotificationListener::new();
        listener.start().unwrap();
        assert!(listener.is_running());
        listener.stop();
        assert!(!listener.is_running());
    }

    #[test]
    fn test_add_notification() {
        let listener = NotificationListener::new();
        let notification = DesktopNotification::new(1, "TestApp", "Test Title", "Test Body");
        listener.add_notification(notification);

        let notifications = listener.get_notifications();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].app_name, "TestApp");
    }

    #[test]
    fn test_app_filter() {
        let listener = NotificationListener::new();
        listener.set_app_filter("BlockedApp", false);

        let notification = DesktopNotification::new(1, "BlockedApp", "Test", "Test");
        listener.add_notification(notification);

        // Should not be added due to filter
        let notifications = listener.get_notifications();
        assert!(notifications.is_empty());
    }
}
