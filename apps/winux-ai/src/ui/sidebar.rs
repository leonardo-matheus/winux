// Sidebar - Conversation history list

use crate::database::{ConversationDatabase, ConversationSummary};
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, SearchEntry, SelectionMode, Align,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
pub struct Sidebar {
    pub widget: GtkBox,
    list_box: ListBox,
    search_entry: SearchEntry,
    database: Arc<ConversationDatabase>,
    on_conversation_selected: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
}

impl Sidebar {
    pub fn new(database: Arc<ConversationDatabase>) -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.add_css_class("sidebar");
        widget.set_width_request(280);

        // Header
        let header = GtkBox::new(Orientation::Horizontal, 8);
        header.add_css_class("sidebar-header");

        let title = Label::new(Some("Conversations"));
        title.add_css_class("title-4");
        title.set_hexpand(true);
        title.set_xalign(0.0);
        header.append(&title);

        // New conversation button
        let new_btn = Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("New Conversation")
            .build();
        new_btn.add_css_class("flat");
        new_btn.set_action_name(Some("win.new-conversation"));
        header.append(&new_btn);

        widget.append(&header);

        // Search
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search conversations...")
            .margin_start(8)
            .margin_end(8)
            .margin_bottom(8)
            .build();
        search_entry.add_css_class("search-entry");
        widget.append(&search_entry);

        // List
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .build();
        list_box.add_css_class("navigation-sidebar");

        scroll.set_child(Some(&list_box));
        widget.append(&scroll);

        let on_conversation_selected: Rc<RefCell<Option<Box<dyn Fn(String)>>>> =
            Rc::new(RefCell::new(None));

        // Handle selection
        let callback = on_conversation_selected.clone();
        list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(id) = row.widget_name().as_str().strip_prefix("conv-") {
                    if let Some(ref cb) = *callback.borrow() {
                        cb(id.to_string());
                    }
                }
            }
        });

        // Handle search
        let database_clone = database.clone();
        let list_box_clone = list_box.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text();
            Self::update_list_internal(&database_clone, &list_box_clone, Some(&query));
        });

        let sidebar = Self {
            widget,
            list_box,
            search_entry,
            database,
            on_conversation_selected,
        };

        sidebar.refresh();
        sidebar
    }

    /// Connect conversation selection callback
    pub fn connect_conversation_selected<F>(&self, callback: F)
    where
        F: Fn(String) + 'static,
    {
        *self.on_conversation_selected.borrow_mut() = Some(Box::new(callback));
    }

    /// Refresh the conversation list
    pub fn refresh(&self) {
        let query = self.search_entry.text();
        let query = if query.is_empty() { None } else { Some(query.as_str()) };
        Self::update_list_internal(&self.database, &self.list_box, query);
    }

    fn update_list_internal(
        database: &Arc<ConversationDatabase>,
        list_box: &ListBox,
        search_query: Option<&str>,
    ) {
        // Clear existing items
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        // Get conversations
        let conversations = if let Some(query) = search_query {
            if query.is_empty() {
                database.get_recent_conversations(50)
            } else {
                database.search_conversations(query, 50)
            }
        } else {
            database.get_recent_conversations(50)
        };

        // Add rows
        for conv in conversations {
            let row = Self::create_conversation_row(&conv);
            list_box.append(&row);
        }

        // Show empty state if no conversations
        if list_box.first_child().is_none() {
            let empty_label = Label::new(Some("No conversations yet"));
            empty_label.add_css_class("dim-label");
            empty_label.set_margin_top(24);
            let row = ListBoxRow::builder()
                .child(&empty_label)
                .activatable(false)
                .selectable(false)
                .build();
            list_box.append(&row);
        }
    }

    fn create_conversation_row(conv: &ConversationSummary) -> ListBoxRow {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.add_css_class("conversation-item");
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        // Content
        let content = GtkBox::new(Orientation::Vertical, 4);
        content.set_hexpand(true);

        // Title
        let title = Label::new(Some(&conv.title));
        title.add_css_class("conversation-title");
        title.set_xalign(0.0);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title.set_max_width_chars(25);
        content.append(&title);

        // Metadata
        let meta_box = GtkBox::new(Orientation::Horizontal, 8);

        // Parse and format date
        let date_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&conv.updated_at) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(dt);

            if duration.num_days() == 0 {
                if duration.num_hours() == 0 {
                    format!("{}m ago", duration.num_minutes().max(1))
                } else {
                    format!("{}h ago", duration.num_hours())
                }
            } else if duration.num_days() == 1 {
                "Yesterday".to_string()
            } else if duration.num_days() < 7 {
                format!("{}d ago", duration.num_days())
            } else {
                dt.format("%b %d").to_string()
            }
        } else {
            conv.updated_at.clone()
        };

        let date_label = Label::new(Some(&date_str));
        date_label.add_css_class("conversation-date");
        meta_box.append(&date_label);

        let msg_count = Label::new(Some(&format!("{} msgs", conv.message_count)));
        msg_count.add_css_class("conversation-date");
        meta_box.append(&msg_count);

        content.append(&meta_box);
        container.append(&content);

        // Delete button
        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Delete conversation")
            .build();
        delete_btn.add_css_class("flat");
        delete_btn.add_css_class("circular");
        delete_btn.set_visible(false);

        let conv_id = conv.id.clone();
        delete_btn.connect_clicked(move |btn| {
            btn.activate_action("win.delete-conversation", Some(&conv_id.to_variant()))
                .ok();
        });

        container.append(&delete_btn);

        // Show delete button on hover
        let motion_controller = gtk4::EventControllerMotion::new();
        let delete_btn_clone = delete_btn.clone();
        motion_controller.connect_enter(move |_, _, _| {
            delete_btn_clone.set_visible(true);
        });
        let delete_btn_clone = delete_btn.clone();
        motion_controller.connect_leave(move |_| {
            delete_btn_clone.set_visible(false);
        });
        container.add_controller(motion_controller);

        let row = ListBoxRow::builder()
            .child(&container)
            .build();
        row.set_widget_name(&format!("conv-{}", conv.id));

        row
    }

    /// Select a conversation by ID
    pub fn select_conversation(&self, id: &str) {
        let mut index = 0;
        while let Some(row) = self.list_box.row_at_index(index) {
            if row.widget_name() == format!("conv-{}", id) {
                self.list_box.select_row(Some(&row));
                break;
            }
            index += 1;
        }
    }
}
