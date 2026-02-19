//! Notification data structures
//!
//! Defines the core notification types used throughout the daemon.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notification urgency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    /// Low urgency - can be hidden during DND
    Low = 0,
    /// Normal urgency - default level
    #[default]
    Normal = 1,
    /// Critical urgency - always shown, even during DND
    Critical = 2,
}

impl From<u8> for Urgency {
    fn from(value: u8) -> Self {
        match value {
            0 => Urgency::Low,
            2 => Urgency::Critical,
            _ => Urgency::Normal,
        }
    }
}

/// A notification action (button)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Action identifier
    pub id: String,
    /// Display label for the action
    pub label: String,
}

/// Notification hints/metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationHints {
    /// Urgency level
    pub urgency: Urgency,
    /// Category of notification
    pub category: Option<String>,
    /// Desktop entry of the sending application
    pub desktop_entry: Option<String>,
    /// Image data (base64 encoded if serialized)
    #[serde(skip)]
    pub image_data: Option<Vec<u8>>,
    /// Image path
    pub image_path: Option<String>,
    /// Sound file to play
    pub sound_file: Option<String>,
    /// Sound name from theme
    pub sound_name: Option<String>,
    /// Suppress sound
    pub suppress_sound: bool,
    /// Transient notification (don't save to history)
    pub transient: bool,
    /// X position hint
    pub x: Option<i32>,
    /// Y position hint
    pub y: Option<i32>,
    /// Resident notification (stays until explicitly closed)
    pub resident: bool,
    /// Action icons
    pub action_icons: bool,
    /// Progress value (0-100, or -1 for indeterminate)
    pub progress_value: Option<i32>,
}

/// A notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique notification ID
    pub id: u32,
    /// Application name
    pub app_name: String,
    /// Notification icon name or path
    pub app_icon: String,
    /// Summary/title
    pub summary: String,
    /// Body text (may contain markup)
    pub body: String,
    /// Available actions
    pub actions: Vec<NotificationAction>,
    /// Hints and metadata
    pub hints: NotificationHints,
    /// Expiration timeout in milliseconds (-1 for default, 0 for never)
    pub expire_timeout: i32,
    /// Timestamp when notification was received
    pub timestamp: DateTime<Local>,
    /// Whether this notification replaces another
    pub replaces_id: u32,
    /// Whether notification has been read/acknowledged
    pub read: bool,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        id: u32,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<(String, String)>,
        hints: HashMap<String, zvariant::OwnedValue>,
        expire_timeout: i32,
    ) -> Self {
        let notification_actions: Vec<NotificationAction> = actions
            .into_iter()
            .map(|(id, label)| NotificationAction { id, label })
            .collect();

        let notification_hints = Self::parse_hints(hints);

        Self {
            id,
            app_name,
            app_icon,
            summary,
            body,
            actions: notification_actions,
            hints: notification_hints,
            expire_timeout,
            timestamp: Local::now(),
            replaces_id,
            read: false,
        }
    }

    /// Parse D-Bus hints into our hints structure
    fn parse_hints(hints: HashMap<String, zvariant::OwnedValue>) -> NotificationHints {
        let mut result = NotificationHints::default();

        for (key, value) in hints {
            match key.as_str() {
                "urgency" => {
                    if let Ok(v) = value.downcast_ref::<u8>() {
                        result.urgency = Urgency::from(*v);
                    }
                }
                "category" => {
                    if let Ok(v) = value.downcast_ref::<&str>() {
                        result.category = Some(v.to_string());
                    }
                }
                "desktop-entry" => {
                    if let Ok(v) = value.downcast_ref::<&str>() {
                        result.desktop_entry = Some(v.to_string());
                    }
                }
                "image-path" | "image_path" => {
                    if let Ok(v) = value.downcast_ref::<&str>() {
                        result.image_path = Some(v.to_string());
                    }
                }
                "sound-file" => {
                    if let Ok(v) = value.downcast_ref::<&str>() {
                        result.sound_file = Some(v.to_string());
                    }
                }
                "sound-name" => {
                    if let Ok(v) = value.downcast_ref::<&str>() {
                        result.sound_name = Some(v.to_string());
                    }
                }
                "suppress-sound" => {
                    if let Ok(v) = value.downcast_ref::<bool>() {
                        result.suppress_sound = *v;
                    }
                }
                "transient" => {
                    if let Ok(v) = value.downcast_ref::<bool>() {
                        result.transient = *v;
                    }
                }
                "x" => {
                    if let Ok(v) = value.downcast_ref::<i32>() {
                        result.x = Some(*v);
                    }
                }
                "y" => {
                    if let Ok(v) = value.downcast_ref::<i32>() {
                        result.y = Some(*v);
                    }
                }
                "resident" => {
                    if let Ok(v) = value.downcast_ref::<bool>() {
                        result.resident = *v;
                    }
                }
                "action-icons" => {
                    if let Ok(v) = value.downcast_ref::<bool>() {
                        result.action_icons = *v;
                    }
                }
                "value" => {
                    if let Ok(v) = value.downcast_ref::<i32>() {
                        result.progress_value = Some(*v);
                    }
                }
                _ => {}
            }
        }

        result
    }

    /// Check if this notification should be shown during DND mode
    pub fn should_show_during_dnd(&self) -> bool {
        self.hints.urgency == Urgency::Critical
    }

    /// Check if this is a transient notification
    pub fn is_transient(&self) -> bool {
        self.hints.transient
    }

    /// Check if this is a progress notification
    pub fn is_progress(&self) -> bool {
        self.hints.progress_value.is_some()
    }

    /// Get the effective timeout in milliseconds
    pub fn effective_timeout(&self, default_timeout: u32) -> u32 {
        match self.expire_timeout {
            -1 => default_timeout,
            0 => 0, // Never expire
            t if t > 0 => t as u32,
            _ => default_timeout,
        }
    }

    /// Get CSS class based on urgency
    pub fn urgency_class(&self) -> &'static str {
        match self.hints.urgency {
            Urgency::Low => "notification-low",
            Urgency::Normal => "notification-normal",
            Urgency::Critical => "notification-critical",
        }
    }
}

/// Reason for closing a notification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    /// Notification expired
    Expired = 1,
    /// Notification was dismissed by user
    Dismissed = 2,
    /// Notification was closed via CloseNotification
    Closed = 3,
    /// Undefined/reserved
    Undefined = 4,
}

impl From<u32> for CloseReason {
    fn from(value: u32) -> Self {
        match value {
            1 => CloseReason::Expired,
            2 => CloseReason::Dismissed,
            3 => CloseReason::Closed,
            _ => CloseReason::Undefined,
        }
    }
}
