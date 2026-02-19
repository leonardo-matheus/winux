// Winux Logs - Detail Panel Component
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    Box, Label, Orientation, ScrolledWindow, PolicyType, Button,
    Separator, TextView, TextBuffer, WrapMode, Frame,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, ActionRow};

use crate::sources::LogEntry;
use crate::ui::{level_css_class, level_icon};

/// Create the detail panel for showing full log entry information
pub fn create_detail_panel() -> Box {
    let panel = Box::new(Orientation::Vertical, 0);
    panel.set_width_request(350);

    // Header
    let header = Box::new(Orientation::Horizontal, 8);
    header.set_margin_start(12);
    header.set_margin_end(12);
    header.set_margin_top(12);
    header.set_margin_bottom(8);

    let title = Label::new(Some("Detalhes"));
    title.add_css_class("title-3");
    title.set_hexpand(true);
    title.set_xalign(0.0);

    let copy_button = Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copiar entrada")
        .build();
    copy_button.add_css_class("flat");

    header.append(&title);
    header.append(&copy_button);

    panel.append(&header);
    panel.append(&Separator::new(Orientation::Horizontal));

    // Content area (scrollable)
    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.add_css_class("log-detail-view");

    // Placeholder
    let placeholder = create_placeholder();
    content.append(&placeholder);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&content)
        .vexpand(true)
        .build();

    panel.append(&scroll);

    panel
}

/// Create placeholder for empty detail view
fn create_placeholder() -> Box {
    let placeholder = Box::new(Orientation::Vertical, 8);
    placeholder.set_valign(gtk4::Align::Center);
    placeholder.set_halign(gtk4::Align::Center);
    placeholder.set_vexpand(true);

    let icon = gtk4::Image::from_icon_name("document-properties-symbolic");
    icon.set_pixel_size(48);
    icon.add_css_class("dim-label");

    let label = Label::new(Some("Selecione um log"));
    label.add_css_class("dim-label");

    placeholder.append(&icon);
    placeholder.append(&label);

    placeholder
}

/// Create detail view content for a log entry
pub fn create_entry_detail(entry: &LogEntry) -> Box {
    let content = Box::new(Orientation::Vertical, 16);
    content.add_css_class("log-detail-view");

    // Basic info section
    let basic_group = PreferencesGroup::builder()
        .title("Informacoes Basicas")
        .build();

    // Timestamp
    let time_row = ActionRow::builder()
        .title("Timestamp")
        .subtitle(&entry.formatted_time_precise())
        .build();
    time_row.add_prefix(&gtk4::Image::from_icon_name("preferences-system-time-symbolic"));
    basic_group.add(&time_row);

    // Level
    let level_row = ActionRow::builder()
        .title("Nivel")
        .subtitle(entry.level.display_name())
        .build();
    let level_icon_widget = gtk4::Image::from_icon_name(level_icon(&entry.level));
    level_icon_widget.add_css_class(level_css_class(&entry.level));
    level_row.add_prefix(&level_icon_widget);
    basic_group.add(&level_row);

    // Unit
    if let Some(ref unit) = entry.unit {
        let unit_row = ActionRow::builder()
            .title("Unit")
            .subtitle(unit)
            .build();
        unit_row.add_prefix(&gtk4::Image::from_icon_name("system-run-symbolic"));
        basic_group.add(&unit_row);
    }

    // PID
    if let Some(pid) = entry.pid {
        let pid_row = ActionRow::builder()
            .title("PID")
            .subtitle(&pid.to_string())
            .build();
        pid_row.add_prefix(&gtk4::Image::from_icon_name("utilities-system-monitor-symbolic"));
        basic_group.add(&pid_row);
    }

    // UID
    if let Some(uid) = entry.uid {
        let uid_row = ActionRow::builder()
            .title("UID")
            .subtitle(&uid.to_string())
            .build();
        uid_row.add_prefix(&gtk4::Image::from_icon_name("system-users-symbolic"));
        basic_group.add(&uid_row);
    }

    // Source
    if !entry.source.is_empty() {
        let source_row = ActionRow::builder()
            .title("Fonte")
            .subtitle(&entry.source)
            .build();
        source_row.add_prefix(&gtk4::Image::from_icon_name("folder-documents-symbolic"));
        basic_group.add(&source_row);
    }

    // Hostname
    if let Some(ref hostname) = entry.hostname {
        let host_row = ActionRow::builder()
            .title("Hostname")
            .subtitle(hostname)
            .build();
        host_row.add_prefix(&gtk4::Image::from_icon_name("computer-symbolic"));
        basic_group.add(&host_row);
    }

    content.append(&basic_group);

    // Message section
    let message_frame = Frame::builder()
        .label("Mensagem")
        .build();

    let message_view = create_message_view(&entry.message);
    message_frame.set_child(Some(&message_view));

    content.append(&message_frame);

    // Stack trace section (if present)
    if let Some(ref trace) = entry.stack_trace {
        let trace_frame = Frame::builder()
            .label("Stack Trace")
            .build();

        let trace_view = create_stack_trace_view(trace);
        trace_frame.set_child(Some(&trace_view));

        content.append(&trace_frame);
    }

    // Extra fields section
    if !entry.extra_fields.is_empty() {
        let extra_group = PreferencesGroup::builder()
            .title("Campos Adicionais")
            .description("Metadados do journal")
            .build();

        let mut fields: Vec<(&String, &String)> = entry.extra_fields.iter().collect();
        fields.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in fields {
            let field_row = ActionRow::builder()
                .title(key)
                .subtitle(&crate::ui::truncate_string(value, 100))
                .build();
            field_row.set_tooltip_text(Some(value));
            extra_group.add(&field_row);
        }

        content.append(&extra_group);
    }

    // Boot info
    if entry.boot_id.is_some() || entry.machine_id.is_some() {
        let boot_group = PreferencesGroup::builder()
            .title("Identificadores")
            .build();

        if let Some(ref boot_id) = entry.boot_id {
            let boot_row = ActionRow::builder()
                .title("Boot ID")
                .subtitle(boot_id)
                .build();
            boot_group.add(&boot_row);
        }

        if let Some(ref machine_id) = entry.machine_id {
            let machine_row = ActionRow::builder()
                .title("Machine ID")
                .subtitle(machine_id)
                .build();
            boot_group.add(&machine_row);
        }

        content.append(&boot_group);
    }

    // Raw message section
    if let Some(ref raw) = entry.raw_message {
        let raw_group = PreferencesGroup::builder()
            .title("Entrada Raw")
            .description("JSON original do journal")
            .build();

        let raw_view = create_raw_view(raw);
        raw_group.add(&raw_view);

        content.append(&raw_group);
    }

    // Action buttons
    let actions_box = Box::new(Orientation::Horizontal, 8);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_top(16);

    let copy_btn = Button::with_label("Copiar");
    copy_btn.set_icon_name("edit-copy-symbolic");
    copy_btn.add_css_class("pill");

    let export_btn = Button::with_label("Exportar");
    export_btn.set_icon_name("document-save-symbolic");
    export_btn.add_css_class("pill");

    actions_box.append(&copy_btn);
    actions_box.append(&export_btn);

    content.append(&actions_box);

    content
}

/// Create a scrollable message view
fn create_message_view(message: &str) -> ScrolledWindow {
    let buffer = TextBuffer::new(None::<&gtk4::TextTagTable>);
    buffer.set_text(message);

    let text_view = TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(WrapMode::WordChar)
        .monospace(true)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();

    ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&text_view)
        .min_content_height(100)
        .max_content_height(200)
        .build()
}

/// Create a stack trace view with syntax highlighting
fn create_stack_trace_view(trace: &str) -> ScrolledWindow {
    let buffer = TextBuffer::new(None::<&gtk4::TextTagTable>);
    buffer.set_text(trace);

    let text_view = TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(WrapMode::None)
        .monospace(true)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();
    text_view.add_css_class("stack-trace");

    ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&text_view)
        .min_content_height(150)
        .max_content_height(300)
        .build()
}

/// Create a raw JSON view
fn create_raw_view(raw: &str) -> ScrolledWindow {
    // Pretty print JSON if possible
    let formatted = if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw) {
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| raw.to_string())
    } else {
        raw.to_string()
    };

    let buffer = TextBuffer::new(None::<&gtk4::TextTagTable>);
    buffer.set_text(&formatted);

    let text_view = TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(WrapMode::None)
        .monospace(true)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();

    ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&text_view)
        .min_content_height(100)
        .max_content_height(200)
        .build()
}

/// Update the detail panel with a new entry
pub fn update_detail_panel(panel: &Box, entry: Option<&LogEntry>) {
    // Remove existing content (skip header)
    let mut child = panel.first_child();
    let mut children_to_remove = Vec::new();

    while let Some(widget) = child {
        let next = widget.next_sibling();
        // Skip first two children (header and separator)
        if panel.observe_children().n_items() > 2 {
            children_to_remove.push(widget.clone());
        }
        child = next;
    }

    for widget in children_to_remove {
        panel.remove(&widget);
    }

    // Add new content
    let content = if let Some(entry) = entry {
        create_entry_detail(entry)
    } else {
        let placeholder = create_placeholder();
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .child(&placeholder)
            .vexpand(true)
            .build();

        let wrapper = Box::new(Orientation::Vertical, 0);
        wrapper.append(&scroll);
        wrapper
    };

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&content)
        .vexpand(true)
        .build();

    panel.append(&scroll);
}
