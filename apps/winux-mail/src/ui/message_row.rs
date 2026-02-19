// Winux Mail - Message Row Widget
// Copyright (c) 2026 Winux OS Project

use crate::data::message::Message;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, CheckButton, Image, Label, ListBoxRow, Orientation, Overlay,
};
use libadwaita as adw;
use pango::EllipsizeMode;

/// A row widget displaying a message in the message list
pub struct MessageRow {
    pub row: ListBoxRow,
    pub check: CheckButton,
    pub starred_btn: gtk4::Button,
    pub message_id: String,
}

impl MessageRow {
    pub fn new(message: &Message) -> Self {
        let row = ListBoxRow::builder()
            .css_classes(if message.is_read() {
                vec!["message-row", "read"]
            } else {
                vec!["message-row", "unread"]
            })
            .build();

        let overlay = Overlay::new();

        let main_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Checkbox (hidden by default, shown on hover)
        let check = CheckButton::builder()
            .visible(false)
            .build();

        main_box.append(&check);

        // Avatar/initials
        let avatar = adw::Avatar::builder()
            .size(40)
            .text(message.sender_name())
            .show_initials(true)
            .build();

        main_box.append(&avatar);

        // Content
        let content_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .spacing(2)
            .build();

        // Top row: sender + date
        let top_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let sender_label = Label::builder()
            .label(message.sender_name())
            .css_classes(if message.is_read() {
                vec![]
            } else {
                vec!["heading"]
            })
            .halign(gtk4::Align::Start)
            .ellipsize(EllipsizeMode::End)
            .hexpand(true)
            .build();

        let date_label = Label::builder()
            .label(&message.formatted_date())
            .css_classes(vec!["dim-label", "caption"])
            .halign(gtk4::Align::End)
            .build();

        top_row.append(&sender_label);
        top_row.append(&date_label);

        // Subject row
        let subject_label = Label::builder()
            .label(&message.subject)
            .css_classes(if message.is_read() {
                vec![]
            } else {
                vec!["heading"]
            })
            .halign(gtk4::Align::Start)
            .ellipsize(EllipsizeMode::End)
            .build();

        // Preview row with icons
        let preview_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        let preview_label = Label::builder()
            .label(&message.short_preview(100))
            .css_classes(vec!["dim-label", "caption"])
            .halign(gtk4::Align::Start)
            .ellipsize(EllipsizeMode::End)
            .hexpand(true)
            .build();

        preview_row.append(&preview_label);

        // Icons
        if message.has_attachments() {
            let attach_icon = Image::builder()
                .icon_name("mail-attachment-symbolic")
                .css_classes(vec!["dim-label"])
                .build();
            preview_row.append(&attach_icon);
        }

        if message.is_replied() {
            let reply_icon = Image::builder()
                .icon_name("mail-reply-sender-symbolic")
                .css_classes(vec!["dim-label"])
                .build();
            preview_row.append(&reply_icon);
        }

        content_box.append(&top_row);
        content_box.append(&subject_label);
        content_box.append(&preview_row);

        main_box.append(&content_box);

        // Star button
        let starred_btn = gtk4::Button::builder()
            .icon_name(if message.starred {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            })
            .css_classes(vec!["flat", "circular"])
            .valign(gtk4::Align::Center)
            .build();

        if message.starred {
            starred_btn.add_css_class("starred");
        }

        main_box.append(&starred_btn);

        overlay.set_child(Some(&main_box));

        // Unread indicator
        if !message.is_read() {
            let indicator = GtkBox::builder()
                .width_request(4)
                .css_classes(vec!["accent-bg"])
                .valign(gtk4::Align::Fill)
                .halign(gtk4::Align::Start)
                .build();
            overlay.add_overlay(&indicator);
        }

        row.set_child(Some(&overlay));

        Self {
            row,
            check,
            starred_btn,
            message_id: message.id.clone(),
        }
    }

    pub fn set_selected(&self, selected: bool) {
        self.check.set_active(selected);
        self.check.set_visible(selected);
    }

    pub fn is_selected(&self) -> bool {
        self.check.is_active()
    }

    pub fn update_starred(&self, starred: bool) {
        self.starred_btn.set_icon_name(if starred {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        });

        if starred {
            self.starred_btn.add_css_class("starred");
        } else {
            self.starred_btn.remove_css_class("starred");
        }
    }

    pub fn mark_read(&self) {
        self.row.remove_css_class("unread");
        self.row.add_css_class("read");
    }

    pub fn mark_unread(&self) {
        self.row.remove_css_class("read");
        self.row.add_css_class("unread");
    }
}

/// Compact message row for thread view
pub struct CompactMessageRow {
    pub row: ListBoxRow,
    pub message_id: String,
}

impl CompactMessageRow {
    pub fn new(message: &Message, is_expanded: bool) -> Self {
        let row = ListBoxRow::builder()
            .css_classes(vec!["compact-message-row"])
            .build();

        let main_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // Avatar
        let avatar = adw::Avatar::builder()
            .size(24)
            .text(message.sender_name())
            .show_initials(true)
            .build();

        main_box.append(&avatar);

        // Sender
        let sender_label = Label::builder()
            .label(message.sender_name())
            .css_classes(if message.is_read() {
                vec![]
            } else {
                vec!["heading"]
            })
            .halign(gtk4::Align::Start)
            .width_chars(15)
            .max_width_chars(15)
            .ellipsize(EllipsizeMode::End)
            .build();

        main_box.append(&sender_label);

        // Preview or content indicator
        let content_label = Label::builder()
            .label(if is_expanded {
                &message.short_preview(50)
            } else {
                "(Click to expand)"
            })
            .css_classes(vec!["dim-label"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .ellipsize(EllipsizeMode::End)
            .build();

        main_box.append(&content_label);

        // Date
        let date_label = Label::builder()
            .label(&message.formatted_date())
            .css_classes(vec!["dim-label", "caption"])
            .halign(gtk4::Align::End)
            .build();

        main_box.append(&date_label);

        // Attachment icon
        if message.has_attachments() {
            let icon = Image::builder()
                .icon_name("mail-attachment-symbolic")
                .css_classes(vec!["dim-label"])
                .build();
            main_box.append(&icon);
        }

        row.set_child(Some(&main_box));

        Self {
            row,
            message_id: message.id.clone(),
        }
    }
}

/// Message row for search results
pub struct SearchResultRow {
    pub row: ListBoxRow,
    pub message_id: String,
    pub folder: String,
}

impl SearchResultRow {
    pub fn new(message: &Message, highlight: &str) -> Self {
        let row = ListBoxRow::builder()
            .css_classes(vec!["search-result-row"])
            .build();

        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Top row
        let top_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let avatar = adw::Avatar::builder()
            .size(32)
            .text(message.sender_name())
            .show_initials(true)
            .build();

        let sender_label = Label::builder()
            .label(message.sender_name())
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        let date_label = Label::builder()
            .label(&message.formatted_date())
            .css_classes(vec!["dim-label", "caption"])
            .build();

        top_row.append(&avatar);
        top_row.append(&sender_label);
        top_row.append(&date_label);

        // Subject
        let subject_label = Label::builder()
            .label(&message.subject)
            .halign(gtk4::Align::Start)
            .ellipsize(EllipsizeMode::End)
            .build();

        // Preview with highlighted match
        let preview = Self::highlight_text(&message.preview, highlight);
        let preview_label = Label::builder()
            .label(&preview)
            .use_markup(true)
            .css_classes(vec!["dim-label"])
            .halign(gtk4::Align::Start)
            .ellipsize(EllipsizeMode::End)
            .build();

        // Folder badge
        let folder_label = Label::builder()
            .label(&message.folder)
            .css_classes(vec!["caption", "pill"])
            .halign(gtk4::Align::Start)
            .build();

        main_box.append(&top_row);
        main_box.append(&subject_label);
        main_box.append(&preview_label);
        main_box.append(&folder_label);

        row.set_child(Some(&main_box));

        Self {
            row,
            message_id: message.id.clone(),
            folder: message.folder.clone(),
        }
    }

    fn highlight_text(text: &str, highlight: &str) -> String {
        if highlight.is_empty() {
            return glib::markup_escape_text(text).to_string();
        }

        let escaped = glib::markup_escape_text(text);
        let highlight_escaped = glib::markup_escape_text(highlight);

        // Case-insensitive replace with markup
        let re = regex::RegexBuilder::new(&regex::escape(&highlight_escaped))
            .case_insensitive(true)
            .build();

        if let Ok(re) = re {
            re.replace_all(&escaped, "<b>$0</b>").to_string()
        } else {
            escaped.to_string()
        }
    }
}
