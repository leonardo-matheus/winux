// Winux Logs - UI Module
// Copyright (c) 2026 Winux OS Project

pub mod log_view;
pub mod log_row;
pub mod detail;
pub mod filters;

use gtk4::prelude::*;
use gtk4::{CssProvider, StyleContext};
use gdk4::Display;

/// Custom CSS for log viewer styling
const CUSTOM_CSS: &str = r#"
    .log-row {
        padding: 4px 8px;
        border-bottom: 1px solid alpha(@borders, 0.5);
    }

    .log-row:hover {
        background-color: alpha(@theme_selected_bg_color, 0.1);
    }

    .log-row.selected {
        background-color: alpha(@theme_selected_bg_color, 0.3);
    }

    .log-level-emergency,
    .log-level-alert,
    .log-level-critical {
        color: @error_color;
        font-weight: bold;
    }

    .log-level-error {
        color: @error_color;
    }

    .log-level-warning {
        color: @warning_color;
    }

    .log-level-notice {
        color: @accent_color;
    }

    .log-level-info {
        color: @theme_fg_color;
    }

    .log-level-debug {
        color: alpha(@theme_fg_color, 0.6);
    }

    .log-timestamp {
        font-family: monospace;
        font-size: 0.9em;
        color: alpha(@theme_fg_color, 0.7);
    }

    .log-unit {
        font-family: monospace;
        font-size: 0.9em;
        color: @accent_color;
    }

    .log-message {
        font-family: monospace;
    }

    .log-detail-view {
        font-family: monospace;
        padding: 12px;
    }

    .log-detail-field {
        margin-bottom: 8px;
    }

    .log-detail-label {
        font-weight: bold;
        color: alpha(@theme_fg_color, 0.7);
    }

    .log-detail-value {
        margin-left: 8px;
    }

    .stack-trace {
        font-family: monospace;
        font-size: 0.85em;
        background-color: alpha(@error_color, 0.1);
        padding: 8px;
        border-radius: 4px;
        border: 1px solid alpha(@error_color, 0.3);
    }

    .filter-section {
        padding: 8px;
        border-bottom: 1px solid @borders;
    }

    .filter-chip {
        border-radius: 16px;
        padding: 4px 12px;
        margin: 2px;
    }

    .filter-chip.active {
        background-color: @accent_bg_color;
        color: @accent_fg_color;
    }

    .live-indicator {
        color: @success_color;
        animation: pulse 1s ease-in-out infinite;
    }

    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
    }

    .boot-selector {
        background-color: alpha(@theme_bg_color, 0.5);
        padding: 6px 12px;
        border-radius: 4px;
    }

    .status-bar {
        background-color: alpha(@theme_bg_color, 0.8);
        font-size: 0.85em;
    }
"#;

/// Initialize custom styling
pub fn init_styles() {
    let provider = CssProvider::new();
    provider.load_from_string(CUSTOM_CSS);

    if let Some(display) = Display::default() {
        StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Format a log level for display with appropriate styling class
pub fn level_css_class(level: &crate::sources::LogLevel) -> &'static str {
    use crate::sources::LogLevel;

    match level {
        LogLevel::Emergency => "log-level-emergency",
        LogLevel::Alert => "log-level-alert",
        LogLevel::Critical => "log-level-critical",
        LogLevel::Error => "log-level-error",
        LogLevel::Warning => "log-level-warning",
        LogLevel::Notice => "log-level-notice",
        LogLevel::Info => "log-level-info",
        LogLevel::Debug => "log-level-debug",
    }
}

/// Get icon name for log level
pub fn level_icon(level: &crate::sources::LogLevel) -> &'static str {
    use crate::sources::LogLevel;

    match level {
        LogLevel::Emergency | LogLevel::Alert | LogLevel::Critical => "dialog-error-symbolic",
        LogLevel::Error => "process-stop-symbolic",
        LogLevel::Warning => "dialog-warning-symbolic",
        LogLevel::Notice | LogLevel::Info => "dialog-information-symbolic",
        LogLevel::Debug => "system-run-symbolic",
    }
}

/// Truncate a string for display
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Escape Pango markup characters
pub fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
