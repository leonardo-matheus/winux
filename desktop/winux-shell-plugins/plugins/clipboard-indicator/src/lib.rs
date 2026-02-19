//! Clipboard Indicator Plugin
//!
//! Shows clipboard history in the panel and allows quick access to previous copies.

use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use winux_shell_plugins::prelude::*;

/// Maximum number of items to keep in history
const MAX_HISTORY: usize = 50;

/// A clipboard history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardEntry {
    /// Entry content (text)
    content: String,
    /// When it was copied
    timestamp: chrono::DateTime<chrono::Local>,
    /// Whether it's pinned
    pinned: bool,
    /// Content preview (truncated)
    preview: String,
}

impl ClipboardEntry {
    fn new(content: String) -> Self {
        let preview = if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.clone()
        };

        Self {
            content,
            timestamp: chrono::Local::now(),
            pinned: false,
            preview,
        }
    }
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardConfig {
    /// Maximum history size
    max_history: usize,
    /// Show notifications on copy
    show_notifications: bool,
    /// Keyboard shortcut to open history
    shortcut: String,
    /// Clear history on logout
    clear_on_logout: bool,
    /// Synchronize with winux-clipboard app
    sync_with_app: bool,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            max_history: MAX_HISTORY,
            show_notifications: false,
            shortcut: "Super+V".to_string(),
            clear_on_logout: false,
            sync_with_app: true,
        }
    }
}

/// Clipboard indicator plugin
pub struct ClipboardIndicatorPlugin {
    config: ClipboardConfig,
    history: Arc<RwLock<VecDeque<ClipboardEntry>>>,
    last_content: Arc<RwLock<String>>,
}

impl Default for ClipboardIndicatorPlugin {
    fn default() -> Self {
        Self {
            config: ClipboardConfig::default(),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_HISTORY))),
            last_content: Arc::new(RwLock::new(String::new())),
        }
    }
}

impl ClipboardIndicatorPlugin {
    /// Add content to history
    fn add_to_history(&self, content: String) {
        if content.is_empty() {
            return;
        }

        let mut history = self.history.write().unwrap();

        // Check if this content already exists (move to top if so)
        if let Some(pos) = history.iter().position(|e| e.content == content && !e.pinned) {
            let entry = history.remove(pos).unwrap();
            history.push_front(ClipboardEntry {
                timestamp: chrono::Local::now(),
                ..entry
            });
            return;
        }

        // Add new entry
        let entry = ClipboardEntry::new(content);
        history.push_front(entry);

        // Trim history (keep pinned items)
        while history.len() > self.config.max_history {
            if let Some(pos) = history.iter().rposition(|e| !e.pinned) {
                history.remove(pos);
            } else {
                break;
            }
        }
    }

    /// Clear history (except pinned items)
    fn clear_history(&self) {
        let mut history = self.history.write().unwrap();
        history.retain(|e| e.pinned);
    }

    /// Toggle pin on an entry
    fn toggle_pin(&self, index: usize) {
        let mut history = self.history.write().unwrap();
        if let Some(entry) = history.get_mut(index) {
            entry.pinned = !entry.pinned;
        }
    }

    /// Remove an entry
    fn remove_entry(&self, index: usize) {
        let mut history = self.history.write().unwrap();
        history.remove(index);
    }

    /// Copy entry to clipboard
    fn copy_entry(&self, index: usize) -> Option<String> {
        let history = self.history.read().unwrap();
        history.get(index).map(|e| e.content.clone())
    }
}

impl Plugin for ClipboardIndicatorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "org.winux.clipboard-indicator".into(),
            name: "Clipboard Indicator".into(),
            version: Version::new(1, 0, 0),
            description: "Shows clipboard history in the panel".into(),
            authors: vec!["Winux Team".into()],
            homepage: Some("https://winux.org/plugins/clipboard".into()),
            license: Some("MIT".into()),
            min_api_version: Version::new(1, 0, 0),
            capabilities: vec![
                PluginCapability::PanelWidget,
                PluginCapability::KeyboardShortcuts,
            ],
            permissions: {
                let mut perms = PermissionSet::new();
                perms.add(Permission::Clipboard);
                perms.add(Permission::ClipboardWrite);
                perms.add(Permission::PanelWidgets);
                perms.add(Permission::KeyboardShortcuts);
                perms.add(Permission::OwnData);
                perms
            },
            icon: Some("edit-paste-symbolic".into()),
            category: Some("Utilities".into()),
            keywords: vec!["clipboard".into(), "copy".into(), "paste".into(), "history".into()],
            ..Default::default()
        }
    }

    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        // Load config
        let config_path = ctx.config_file("config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.config = config;
                }
            }
        }

        // Load history
        let history_path = ctx.data_file("history.json");
        if history_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&history_path) {
                if let Ok(history) = serde_json::from_str(&content) {
                    *self.history.write().unwrap() = history;
                }
            }
        }

        log::info!("Clipboard indicator initialized with {} items", self.history.read().unwrap().len());
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        log::info!("Clipboard indicator shutting down");
        Ok(())
    }

    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        Some(Box::new(ClipboardPanelWidget {
            history: self.history.clone(),
            config: self.config.clone(),
        }))
    }

    fn command_provider(&self) -> Option<Box<dyn CommandProvider>> {
        Some(Box::new(ClipboardCommandProvider {
            history: self.history.clone(),
        }))
    }

    fn wants_updates(&self) -> bool {
        true
    }

    fn update_interval(&self) -> u32 {
        500 // Check clipboard every 500ms
    }

    fn update(&mut self) -> PluginResult<()> {
        // In a real implementation, we would monitor the clipboard here
        // For now, this is a placeholder
        Ok(())
    }
}

/// Panel widget for clipboard
struct ClipboardPanelWidget {
    history: Arc<RwLock<VecDeque<ClipboardEntry>>>,
    config: ClipboardConfig,
}

impl PanelWidget for ClipboardPanelWidget {
    fn id(&self) -> &str {
        "clipboard-indicator"
    }

    fn name(&self) -> &str {
        "Clipboard"
    }

    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    fn size(&self) -> WidgetSize {
        WidgetSize::Minimal
    }

    fn priority(&self) -> i32 {
        15
    }

    fn state(&self) -> WidgetState {
        let history = self.history.read().unwrap();
        let count = history.len();

        WidgetState::with_icon("edit-paste-symbolic")
            .badge(if count > 0 { &count.to_string() } else { "" })
            .tooltip(&format!("{} items in clipboard history", count))
    }

    fn build_widget(&self) -> gtk::Widget {
        let history = self.history.read().unwrap();

        let button = gtk::Button::new();
        button.set_has_frame(false);
        button.add_css_class("clipboard-indicator");

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);

        let icon = gtk::Image::from_icon_name("edit-paste-symbolic");
        icon.set_pixel_size(16);
        hbox.append(&icon);

        if !history.is_empty() {
            let badge = gtk::Label::new(Some(&history.len().to_string()));
            badge.add_css_class("badge");
            hbox.append(&badge);
        }

        button.set_child(Some(&hbox));
        button.set_tooltip_text(Some(&format!(
            "{} items in clipboard history\nClick to view history",
            history.len()
        )));

        button.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        WidgetAction::ShowPopup
    }

    fn popup_config(&self) -> Option<PopupConfig> {
        Some(PopupConfig {
            width: 350,
            height: 450,
            ..Default::default()
        })
    }

    fn build_popup(&self) -> Option<gtk::Widget> {
        let history = self.history.read().unwrap();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.add_css_class("clipboard-popup");

        // Header
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.set_margin_top(12);
        header.set_margin_bottom(8);
        header.set_margin_start(12);
        header.set_margin_end(12);

        let title = gtk::Label::new(Some("Clipboard History"));
        title.add_css_class("title-3");
        title.set_halign(gtk::Align::Start);
        title.set_hexpand(true);
        header.append(&title);

        let clear_button = gtk::Button::from_icon_name("user-trash-symbolic");
        clear_button.set_tooltip_text(Some("Clear history"));
        clear_button.add_css_class("flat");
        header.append(&clear_button);

        vbox.append(&header);

        // Separator
        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        vbox.append(&sep);

        // List
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let list = gtk::ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk::SelectionMode::None);

        if history.is_empty() {
            let empty = gtk::Label::new(Some("No items in clipboard history"));
            empty.add_css_class("dim-label");
            empty.set_margin_top(24);
            empty.set_margin_bottom(24);
            list.append(&empty);
        } else {
            for (i, entry) in history.iter().enumerate().take(20) {
                let row = self.build_history_row(i, entry);
                list.append(&row);
            }
        }

        scrolled.set_child(Some(&list));
        vbox.append(&scrolled);

        // Footer
        let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        footer.set_margin_top(8);
        footer.set_margin_bottom(12);
        footer.set_margin_start(12);
        footer.set_margin_end(12);

        let settings_button = gtk::Button::with_label("Open Clipboard Manager");
        settings_button.connect_clicked(|_| {
            let _ = std::process::Command::new("winux-clipboard").spawn();
        });
        settings_button.set_hexpand(true);
        footer.append(&settings_button);

        vbox.append(&footer);

        Some(vbox.upcast())
    }
}

impl ClipboardPanelWidget {
    fn build_history_row(&self, _index: usize, entry: &ClipboardEntry) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.add_css_class("clipboard-row");

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);

        // Pin indicator
        if entry.pinned {
            let pin_icon = gtk::Image::from_icon_name("view-pin-symbolic");
            pin_icon.set_pixel_size(16);
            pin_icon.add_css_class("dim-label");
            hbox.append(&pin_icon);
        }

        // Content
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        content_box.set_hexpand(true);

        let preview = gtk::Label::new(Some(&entry.preview.replace('\n', " ")));
        preview.set_halign(gtk::Align::Start);
        preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
        preview.set_max_width_chars(40);
        content_box.append(&preview);

        let time = gtk::Label::new(Some(&entry.timestamp.format("%H:%M").to_string()));
        time.add_css_class("dim-label");
        time.add_css_class("caption");
        time.set_halign(gtk::Align::Start);
        content_box.append(&time);

        hbox.append(&content_box);

        // Actions
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);

        let copy_button = gtk::Button::from_icon_name("edit-copy-symbolic");
        copy_button.set_tooltip_text(Some("Copy"));
        copy_button.add_css_class("flat");
        actions.append(&copy_button);

        let pin_button = gtk::Button::from_icon_name(if entry.pinned {
            "view-pin-symbolic"
        } else {
            "view-pin-symbolic"
        });
        pin_button.set_tooltip_text(Some(if entry.pinned { "Unpin" } else { "Pin" }));
        pin_button.add_css_class("flat");
        actions.append(&pin_button);

        let delete_button = gtk::Button::from_icon_name("edit-delete-symbolic");
        delete_button.set_tooltip_text(Some("Delete"));
        delete_button.add_css_class("flat");
        actions.append(&delete_button);

        hbox.append(&actions);

        row.set_child(Some(&hbox));
        row
    }
}

/// Command provider for clipboard shortcuts
struct ClipboardCommandProvider {
    history: Arc<RwLock<VecDeque<ClipboardEntry>>>,
}

impl CommandProvider for ClipboardCommandProvider {
    fn id(&self) -> &str {
        "clipboard-commands"
    }

    fn commands(&self) -> Vec<Command> {
        vec![
            Command::new("clipboard.show", "Show Clipboard History")
                .with_description("Open the clipboard history popup")
                .with_icon("edit-paste-symbolic")
                .with_shortcut("Super+V")
                .with_category("Clipboard"),
            Command::new("clipboard.clear", "Clear Clipboard History")
                .with_description("Remove all items from clipboard history")
                .with_icon("user-trash-symbolic")
                .with_category("Clipboard")
                .confirm(Some("Clear all clipboard history?")),
            Command::new("clipboard.paste_previous", "Paste Previous")
                .with_description("Paste the second-to-last copied item")
                .with_icon("edit-paste-symbolic")
                .with_shortcut("Super+Shift+V")
                .with_category("Clipboard"),
        ]
    }

    fn execute(&mut self, command_id: &str, _context: &CommandContext) -> CommandResult {
        match command_id {
            "clipboard.show" => CommandResult::Opened, // Shell will handle popup
            "clipboard.clear" => {
                let mut history = self.history.write().unwrap();
                history.retain(|e| e.pinned);
                CommandResult::Message("Clipboard history cleared".to_string())
            }
            "clipboard.paste_previous" => {
                let history = self.history.read().unwrap();
                if history.len() >= 2 {
                    // In real impl, would paste the content
                    CommandResult::Success
                } else {
                    CommandResult::Error("No previous item in history".to_string())
                }
            }
            _ => CommandResult::Error(format!("Unknown command: {}", command_id)),
        }
    }
}

// Plugin entry point
winux_shell_plugins::declare_plugin!(ClipboardIndicatorPlugin, ClipboardIndicatorPlugin::default);
