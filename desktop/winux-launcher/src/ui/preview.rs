//! Preview panel component

use crate::config::Config;
use crate::search::{SearchResult, SearchResultKind};
use gtk4::prelude::*;
use gtk4::{
    glib, Box as GtkBox, Button, Image, Label, Orientation, Revealer, RevealerTransitionType,
    Separator,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Preview panel widget
#[derive(Clone)]
pub struct PreviewPanel {
    revealer: Revealer,
    container: GtkBox,
    icon: Image,
    title: Label,
    subtitle: Label,
    description: Label,
    actions_box: GtkBox,
    current_result: Rc<RefCell<Option<SearchResult>>>,
    expanded: Rc<RefCell<bool>>,
    config: Arc<Config>,
}

impl PreviewPanel {
    /// Create new preview panel
    pub fn new(config: Arc<Config>) -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.add_css_class("preview-container");
        container.set_valign(gtk4::Align::Start);

        // Icon
        let icon = Image::new();
        icon.add_css_class("preview-icon");
        icon.set_pixel_size(64);
        icon.set_halign(gtk4::Align::Center);
        container.append(&icon);

        // Title
        let title = Label::new(None);
        title.add_css_class("preview-title");
        title.set_halign(gtk4::Align::Center);
        title.set_wrap(true);
        title.set_max_width_chars(30);
        container.append(&title);

        // Subtitle
        let subtitle = Label::new(None);
        subtitle.add_css_class("preview-subtitle");
        subtitle.set_halign(gtk4::Align::Center);
        subtitle.set_wrap(true);
        subtitle.set_max_width_chars(35);
        container.append(&subtitle);

        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        separator.set_margin_top(16);
        separator.set_margin_bottom(16);
        container.append(&separator);

        // Description
        let description = Label::new(None);
        description.add_css_class("preview-description");
        description.set_halign(gtk4::Align::Start);
        description.set_wrap(true);
        description.set_max_width_chars(35);
        description.set_xalign(0.0);
        container.append(&description);

        // Actions box
        let actions_box = GtkBox::new(Orientation::Horizontal, 8);
        actions_box.add_css_class("preview-actions");
        actions_box.set_halign(gtk4::Align::Center);
        actions_box.set_margin_top(16);
        container.append(&actions_box);

        // Wrap in revealer for animation
        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::SlideLeft);
        revealer.set_transition_duration(200);
        revealer.set_child(Some(&container));
        revealer.set_reveal_child(config.ui.show_preview);

        Self {
            revealer,
            container,
            icon,
            title,
            subtitle,
            description,
            actions_box,
            current_result: Rc::new(RefCell::new(None)),
            expanded: Rc::new(RefCell::new(config.ui.show_preview)),
            config,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &Revealer {
        &self.revealer
    }

    /// Show result in preview
    pub fn show_result(&self, result: &SearchResult) {
        // Update icon
        self.icon.set_from_icon_name(Some(&result.icon));

        // Update title
        self.title.set_text(&result.title);

        // Update subtitle
        self.subtitle.set_text(&result.subtitle);

        // Update description based on result kind
        let description = self.get_description(result);
        self.description.set_text(&description);

        // Update actions
        self.update_actions(result);

        // Store current result
        *self.current_result.borrow_mut() = Some(result.clone());

        // Show panel if expanded
        if *self.expanded.borrow() {
            self.revealer.set_reveal_child(true);
        }
    }

    /// Get description for result
    fn get_description(&self, result: &SearchResult) -> String {
        match &result.kind {
            SearchResultKind::Application {
                exec, categories, ..
            } => {
                let cats = if categories.is_empty() {
                    "Application".to_string()
                } else {
                    categories.join(", ")
                };
                format!("Categories: {}\n\nCommand: {}", cats, exec)
            }
            SearchResultKind::File { path } => {
                let metadata = std::fs::metadata(path).ok();
                let size = metadata
                    .as_ref()
                    .map(|m| self.format_size(m.len()))
                    .unwrap_or_else(|| "Unknown".to_string());

                let modified = metadata
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                        datetime.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                format!(
                    "Path: {}\n\nSize: {}\nModified: {}",
                    path.display(),
                    size,
                    modified
                )
            }
            SearchResultKind::Calculator { expression, result } => {
                format!("Expression: {}\n\nResult: {}", expression, result)
            }
            SearchResultKind::Conversion {
                from_value,
                from_unit,
                to_value,
                to_unit,
                ..
            } => {
                format!(
                    "Conversion\n\n{} {} = {} {}",
                    from_value, from_unit, to_value, to_unit
                )
            }
            SearchResultKind::WebSearch { engine, query, url } => {
                format!("Search {} for:\n\n\"{}\"\n\nURL: {}", engine, query, url)
            }
            SearchResultKind::Command { command } => {
                format!("System Command\n\nAction: {}", command)
            }
            SearchResultKind::Plugin { plugin_id, action } => {
                format!("Plugin: {}\n\nAction: {}", plugin_id, action)
            }
        }
    }

    /// Update action buttons
    fn update_actions(&self, result: &SearchResult) {
        // Clear existing actions
        while let Some(child) = self.actions_box.first_child() {
            self.actions_box.remove(&child);
        }

        // Add appropriate actions based on result kind
        match &result.kind {
            SearchResultKind::Application { .. } => {
                self.add_action_button("Open", "application-x-executable-symbolic");
            }
            SearchResultKind::File { path } => {
                self.add_action_button("Open", "document-open-symbolic");
                if path.is_file() {
                    self.add_action_button("Open Folder", "folder-open-symbolic");
                }
            }
            SearchResultKind::Calculator { .. } | SearchResultKind::Conversion { .. } => {
                self.add_action_button("Copy", "edit-copy-symbolic");
            }
            SearchResultKind::WebSearch { .. } => {
                self.add_action_button("Search", "web-browser-symbolic");
            }
            SearchResultKind::Command { .. } => {
                self.add_action_button("Run", "system-run-symbolic");
            }
            SearchResultKind::Plugin { .. } => {
                self.add_action_button("Execute", "system-run-symbolic");
            }
        }
    }

    /// Add action button
    fn add_action_button(&self, label: &str, icon: &str) {
        let button = Button::new();
        let content = GtkBox::new(Orientation::Horizontal, 4);

        let icon_widget = Image::from_icon_name(icon);
        icon_widget.set_pixel_size(16);
        content.append(&icon_widget);

        let label_widget = Label::new(Some(label));
        content.append(&label_widget);

        button.set_child(Some(&content));
        button.add_css_class("preview-action-btn");

        self.actions_box.append(&button);
    }

    /// Format file size
    fn format_size(&self, bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }

    /// Clear preview
    pub fn clear(&self) {
        self.icon.clear();
        self.title.set_text("");
        self.subtitle.set_text("");
        self.description.set_text("");

        while let Some(child) = self.actions_box.first_child() {
            self.actions_box.remove(&child);
        }

        *self.current_result.borrow_mut() = None;
    }

    /// Toggle expanded state
    pub fn toggle_expanded(&self) {
        let expanded = !*self.expanded.borrow();
        *self.expanded.borrow_mut() = expanded;
        self.revealer.set_reveal_child(expanded);
    }

    /// Set expanded state
    pub fn set_expanded(&self, expanded: bool) {
        *self.expanded.borrow_mut() = expanded;
        self.revealer.set_reveal_child(expanded);
    }

    /// Check if expanded
    pub fn is_expanded(&self) -> bool {
        *self.expanded.borrow()
    }
}
