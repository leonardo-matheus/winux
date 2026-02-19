//! Style management for notifications
//!
//! Handles CSS styling for notification popups and center.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gdk;
use tracing::debug;

/// CSS styles for notifications
const NOTIFICATION_CSS: &str = r#"
/* Notification Popup Window */
.notification-popup {
    background-color: alpha(@window_bg_color, 0.95);
    border-radius: 12px;
    box-shadow: 0 4px 12px alpha(black, 0.3);
    margin: 8px;
}

.notification-popup.notification-low {
    border-left: 4px solid @blue_3;
}

.notification-popup.notification-normal {
    border-left: 4px solid @accent_bg_color;
}

.notification-popup.notification-critical {
    border-left: 4px solid @error_bg_color;
    background-color: alpha(@error_bg_color, 0.1);
}

/* Notification Content */
.notification-content {
    padding: 12px;
}

.notification-header {
    margin-bottom: 8px;
}

.notification-icon {
    margin-right: 12px;
}

.notification-title {
    font-weight: bold;
    font-size: 14px;
}

.notification-app-name {
    font-size: 11px;
    color: alpha(@window_fg_color, 0.7);
}

.notification-time {
    font-size: 10px;
    color: alpha(@window_fg_color, 0.5);
}

.notification-body {
    font-size: 13px;
    color: alpha(@window_fg_color, 0.9);
}

.notification-body-scrolled {
    max-height: 100px;
}

/* Close Button */
.notification-close {
    min-width: 24px;
    min-height: 24px;
    padding: 2px;
    border-radius: 50%;
    background: transparent;
    opacity: 0.5;
    transition: opacity 200ms ease;
}

.notification-close:hover {
    opacity: 1;
    background-color: alpha(@window_fg_color, 0.1);
}

/* Action Buttons */
.notification-actions {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid alpha(@window_fg_color, 0.1);
}

.notification-action-button {
    padding: 6px 16px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    background-color: alpha(@accent_bg_color, 0.1);
    color: @accent_fg_color;
    transition: background-color 200ms ease;
}

.notification-action-button:hover {
    background-color: alpha(@accent_bg_color, 0.2);
}

.notification-action-button.default {
    background-color: @accent_bg_color;
    color: @accent_fg_color;
}

.notification-action-button.default:hover {
    background-color: shade(@accent_bg_color, 1.1);
}

/* Progress Bar */
.notification-progress {
    margin-top: 8px;
    min-height: 6px;
    border-radius: 3px;
}

.notification-progress trough {
    min-height: 6px;
    border-radius: 3px;
    background-color: alpha(@window_fg_color, 0.1);
}

.notification-progress progress {
    min-height: 6px;
    border-radius: 3px;
    background-color: @accent_bg_color;
}

/* Notification Center */
.notification-center {
    background-color: @window_bg_color;
}

.notification-center-header {
    padding: 16px;
    border-bottom: 1px solid alpha(@window_fg_color, 0.1);
}

.notification-center-title {
    font-size: 18px;
    font-weight: bold;
}

.notification-center-subtitle {
    font-size: 12px;
    color: alpha(@window_fg_color, 0.7);
}

.notification-center-list {
    padding: 8px;
}

.notification-center-empty {
    padding: 48px;
    color: alpha(@window_fg_color, 0.5);
}

.notification-center-empty-icon {
    font-size: 64px;
    margin-bottom: 16px;
    opacity: 0.3;
}

.notification-center-empty-text {
    font-size: 16px;
}

/* Group Header */
.notification-group-header {
    padding: 8px 12px;
    margin-top: 12px;
    font-weight: bold;
    font-size: 12px;
    color: alpha(@window_fg_color, 0.7);
}

.notification-group-header:first-child {
    margin-top: 0;
}

/* DND Toggle */
.dnd-toggle {
    padding: 8px 16px;
    border-radius: 8px;
    margin: 8px;
}

.dnd-toggle.active {
    background-color: alpha(@purple_3, 0.2);
}

.dnd-toggle-icon {
    margin-right: 8px;
}

/* Clear All Button */
.clear-all-button {
    color: @error_color;
    font-size: 12px;
}

.clear-all-button:hover {
    background-color: alpha(@error_color, 0.1);
}

/* History Item */
.notification-history-item {
    padding: 12px;
    margin: 4px 0;
    border-radius: 8px;
    background-color: alpha(@window_fg_color, 0.03);
    transition: background-color 200ms ease;
}

.notification-history-item:hover {
    background-color: alpha(@window_fg_color, 0.06);
}

.notification-history-item.unread {
    background-color: alpha(@accent_bg_color, 0.1);
}

.notification-history-item.unread:hover {
    background-color: alpha(@accent_bg_color, 0.15);
}

/* Animations */
@keyframes slide-in-right {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

@keyframes slide-out-right {
    from {
        transform: translateX(0);
        opacity: 1;
    }
    to {
        transform: translateX(100%);
        opacity: 0;
    }
}

@keyframes fade-in {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}

@keyframes fade-out {
    from {
        opacity: 1;
    }
    to {
        opacity: 0;
    }
}

.notification-slide-in {
    animation: slide-in-right 200ms ease-out;
}

.notification-slide-out {
    animation: slide-out-right 200ms ease-in;
}

.notification-fade-in {
    animation: fade-in 200ms ease-out;
}

.notification-fade-out {
    animation: fade-out 200ms ease-in;
}
"#;

/// Style manager for notification UI
pub struct StyleManager {
    provider: gtk::CssProvider,
}

impl StyleManager {
    pub fn new() -> Self {
        Self {
            provider: gtk::CssProvider::new(),
        }
    }

    /// Load CSS styles
    pub fn load_css(&self) {
        self.provider.load_from_string(NOTIFICATION_CSS);

        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &self.provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            debug!("CSS styles loaded");
        }
    }

    /// Get the CSS provider
    pub fn provider(&self) -> &gtk::CssProvider {
        &self.provider
    }
}

impl Default for StyleManager {
    fn default() -> Self {
        Self::new()
    }
}
