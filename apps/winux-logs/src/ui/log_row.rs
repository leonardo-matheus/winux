// Winux Logs - Log Row Component
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    Box, Label, Orientation, ListBoxRow, Image, Align, Separator,
};
use libadwaita as adw;
use adw::prelude::*;

use crate::sources::{LogEntry, LogLevel};
use crate::ui::{level_css_class, level_icon, truncate_string, escape_markup};

/// Create a log entry row for the list
pub fn create_log_row(entry: &LogEntry) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("log-row");
    row.set_selectable(true);
    row.set_activatable(true);

    let content = Box::new(Orientation::Horizontal, 8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.set_margin_top(4);
    content.set_margin_bottom(4);

    // Level indicator (icon)
    let level_icon_widget = Image::from_icon_name(level_icon(&entry.level));
    level_icon_widget.set_pixel_size(16);
    level_icon_widget.add_css_class(level_css_class(&entry.level));
    level_icon_widget.set_tooltip_text(Some(entry.level.display_name()));

    // Timestamp
    let time_label = Label::new(Some(&entry.formatted_time()));
    time_label.add_css_class("log-timestamp");
    time_label.add_css_class("dim-label");
    time_label.set_width_request(150);
    time_label.set_xalign(0.0);

    // Level text
    let level_label = Label::new(Some(entry.level.display_name()));
    level_label.add_css_class(level_css_class(&entry.level));
    level_label.set_width_request(70);
    level_label.set_xalign(0.0);

    // Unit
    let unit_text = entry.unit_display();
    let unit_label = Label::new(Some(&truncate_string(&unit_text, 25)));
    unit_label.add_css_class("log-unit");
    unit_label.set_width_request(180);
    unit_label.set_xalign(0.0);
    unit_label.set_tooltip_text(Some(&unit_text));

    // Message
    let message_text = truncate_string(&entry.message, 200);
    let message_label = Label::new(Some(&message_text));
    message_label.add_css_class("log-message");
    message_label.set_hexpand(true);
    message_label.set_xalign(0.0);
    message_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    message_label.set_tooltip_text(Some(&entry.message));

    content.append(&level_icon_widget);
    content.append(&time_label);
    content.append(&level_label);
    content.append(&unit_label);
    content.append(&message_label);

    // Add stack trace indicator if present
    if entry.has_stack_trace() {
        let trace_icon = Image::from_icon_name("dialog-error-symbolic");
        trace_icon.set_pixel_size(12);
        trace_icon.add_css_class("error");
        trace_icon.set_tooltip_text(Some("Contem stack trace"));
        content.append(&trace_icon);
    }

    row.set_child(Some(&content));
    row
}

/// Create a compact log row (for smaller displays)
pub fn create_compact_log_row(entry: &LogEntry) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("log-row");
    row.set_selectable(true);

    let content = Box::new(Orientation::Vertical, 2);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.set_margin_top(4);
    content.set_margin_bottom(4);

    // First line: timestamp + level
    let header_box = Box::new(Orientation::Horizontal, 8);

    let level_icon_widget = Image::from_icon_name(level_icon(&entry.level));
    level_icon_widget.set_pixel_size(14);
    level_icon_widget.add_css_class(level_css_class(&entry.level));

    let time_label = Label::new(Some(&entry.formatted_time()));
    time_label.add_css_class("log-timestamp");
    time_label.add_css_class("dim-label");

    let unit_label = Label::new(Some(&entry.unit_display()));
    unit_label.add_css_class("log-unit");
    unit_label.set_hexpand(true);
    unit_label.set_halign(Align::End);

    header_box.append(&level_icon_widget);
    header_box.append(&time_label);
    header_box.append(&unit_label);

    // Second line: message
    let message_label = Label::new(Some(&entry.message));
    message_label.add_css_class("log-message");
    message_label.set_xalign(0.0);
    message_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    message_label.set_max_width_chars(100);

    content.append(&header_box);
    content.append(&message_label);

    row.set_child(Some(&content));
    row
}

/// Create a detailed log row (with more information)
pub fn create_detailed_log_row(entry: &LogEntry) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("log-row");
    row.set_selectable(true);

    let content = Box::new(Orientation::Vertical, 4);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(8);
    content.set_margin_bottom(8);

    // Header line
    let header_box = Box::new(Orientation::Horizontal, 12);

    let level_box = create_level_badge(&entry.level);

    let time_label = Label::new(Some(&entry.formatted_time_precise()));
    time_label.add_css_class("log-timestamp");

    let unit_label = Label::new(Some(&entry.unit_display()));
    unit_label.add_css_class("log-unit");

    header_box.append(&level_box);
    header_box.append(&time_label);
    header_box.append(&unit_label);

    // PID info if available
    if let Some(pid) = entry.pid {
        let pid_label = Label::new(Some(&format!("PID: {}", pid)));
        pid_label.add_css_class("dim-label");
        header_box.append(&pid_label);
    }

    // Message
    let message_label = Label::new(Some(&entry.message));
    message_label.add_css_class("log-message");
    message_label.set_xalign(0.0);
    message_label.set_wrap(true);
    message_label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    message_label.set_selectable(true);

    content.append(&header_box);
    content.append(&message_label);

    // Stack trace if present
    if let Some(ref trace) = entry.stack_trace {
        let trace_label = Label::new(Some(trace));
        trace_label.add_css_class("stack-trace");
        trace_label.set_xalign(0.0);
        trace_label.set_wrap(true);
        trace_label.set_selectable(true);
        content.append(&trace_label);
    }

    row.set_child(Some(&content));
    row
}

/// Create a level badge widget
fn create_level_badge(level: &LogLevel) -> Box {
    let badge = Box::new(Orientation::Horizontal, 4);
    badge.add_css_class("pill");
    badge.add_css_class(level_css_class(level));

    let icon = Image::from_icon_name(level_icon(level));
    icon.set_pixel_size(12);

    let label = Label::new(Some(level.display_name()));
    label.add_css_class("caption");

    badge.append(&icon);
    badge.append(&label);

    badge
}

/// Create a highlighted message label (for search results)
pub fn create_highlighted_message(message: &str, highlight_pattern: Option<&str>) -> Label {
    let label = Label::new(None);
    label.add_css_class("log-message");
    label.set_xalign(0.0);
    label.set_use_markup(true);

    let markup = if let Some(pattern) = highlight_pattern {
        crate::filters::search::highlight_matches(message, pattern, false)
    } else {
        escape_markup(message)
    };

    label.set_markup(&markup);
    label
}

/// Row view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowViewMode {
    Normal,
    Compact,
    Detailed,
}

/// Create a log row with specified view mode
pub fn create_log_row_with_mode(entry: &LogEntry, mode: RowViewMode) -> ListBoxRow {
    match mode {
        RowViewMode::Normal => create_log_row(entry),
        RowViewMode::Compact => create_compact_log_row(entry),
        RowViewMode::Detailed => create_detailed_log_row(entry),
    }
}
