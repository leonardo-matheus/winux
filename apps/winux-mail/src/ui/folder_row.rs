// Winux Mail - Folder Row Widget
// Copyright (c) 2026 Winux OS Project

use crate::data::folder::{Folder, FolderType};

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Image, Label, ListBoxRow, Orientation,
};
use libadwaita as adw;

/// A row widget displaying a folder in the folder list
pub struct FolderRow {
    pub row: ListBoxRow,
    pub badge: Label,
    pub folder_path: String,
}

impl FolderRow {
    pub fn new(icon_name: &str, name: &str, unread_count: u32) -> Self {
        let row = ListBoxRow::builder()
            .css_classes(vec!["folder-row"])
            .build();

        row.set_widget_name(name);

        let main_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        let icon = Image::builder()
            .icon_name(icon_name)
            .build();

        let label = Label::builder()
            .label(name)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();

        let badge = Label::builder()
            .label(&unread_count.to_string())
            .css_classes(vec!["badge", "numeric"])
            .visible(unread_count > 0)
            .build();

        main_box.append(&icon);
        main_box.append(&label);
        main_box.append(&badge);

        row.set_child(Some(&main_box));

        Self {
            row,
            badge,
            folder_path: name.to_string(),
        }
    }

    pub fn from_folder(folder: &Folder) -> Self {
        let row = Self::new(
            folder.icon(),
            folder.display_name(),
            folder.unread_count,
        );

        Self {
            row: row.row,
            badge: row.badge,
            folder_path: folder.path.clone(),
        }
    }

    pub fn update_count(&self, count: u32) {
        self.badge.set_label(&count.to_string());
        self.badge.set_visible(count > 0);
    }

    pub fn set_active(&self, active: bool) {
        if active {
            self.row.add_css_class("active");
        } else {
            self.row.remove_css_class("active");
        }
    }
}

/// Account row in folder sidebar
pub struct AccountRow {
    pub row: ListBoxRow,
    pub expander: gtk4::Expander,
    pub account_id: String,
}

impl AccountRow {
    pub fn new(account_name: &str, email: &str, account_id: &str, total_unread: u32) -> Self {
        let row = ListBoxRow::builder()
            .css_classes(vec!["account-row"])
            .selectable(false)
            .build();

        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        // Account header with expander
        let header_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(4)
            .build();

        let avatar = adw::Avatar::builder()
            .size(32)
            .text(account_name)
            .show_initials(true)
            .build();

        let info_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .build();

        let name_label = Label::builder()
            .label(account_name)
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .build();

        let email_label = Label::builder()
            .label(email)
            .css_classes(vec!["dim-label", "caption"])
            .halign(gtk4::Align::Start)
            .build();

        info_box.append(&name_label);
        info_box.append(&email_label);

        // Unread badge
        let badge = if total_unread > 0 {
            let b = Label::builder()
                .label(&total_unread.to_string())
                .css_classes(vec!["badge", "numeric"])
                .build();
            Some(b)
        } else {
            None
        };

        header_box.append(&avatar);
        header_box.append(&info_box);
        if let Some(b) = badge {
            header_box.append(&b);
        }

        let expander = gtk4::Expander::builder()
            .expanded(true)
            .build();

        expander.set_label_widget(Some(&header_box));

        main_box.append(&expander);

        row.set_child(Some(&main_box));

        Self {
            row,
            expander,
            account_id: account_id.to_string(),
        }
    }

    pub fn set_folders(&self, folders: &[Folder]) {
        let folder_list = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(24)
            .build();

        for folder in folders {
            let folder_row = FolderRow::from_folder(folder);
            folder_list.append(&folder_row.row);
        }

        self.expander.set_child(Some(&folder_list));
    }
}

/// Unified folder row (All Inboxes, etc.)
pub struct UnifiedFolderRow {
    pub row: ListBoxRow,
    pub badge: Label,
    pub folder_type: FolderType,
}

impl UnifiedFolderRow {
    pub fn new(folder_type: FolderType, total_unread: u32) -> Self {
        let name = format!("All {}", folder_type.display_name());

        let row = ListBoxRow::builder()
            .css_classes(vec!["folder-row", "unified"])
            .build();

        let main_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let icon = Image::builder()
            .icon_name(folder_type.icon_name())
            .build();

        let label = Label::builder()
            .label(&name)
            .css_classes(vec!["heading"])
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();

        let badge = Label::builder()
            .label(&total_unread.to_string())
            .css_classes(vec!["badge", "numeric"])
            .visible(total_unread > 0)
            .build();

        main_box.append(&icon);
        main_box.append(&label);
        main_box.append(&badge);

        row.set_child(Some(&main_box));

        Self {
            row,
            badge,
            folder_type,
        }
    }

    pub fn update_count(&self, count: u32) {
        self.badge.set_label(&count.to_string());
        self.badge.set_visible(count > 0);
    }
}

/// Separator row with label
pub struct SeparatorRow {
    pub row: ListBoxRow,
}

impl SeparatorRow {
    pub fn new(label: &str) -> Self {
        let row = ListBoxRow::builder()
            .selectable(false)
            .activatable(false)
            .css_classes(vec!["separator-row"])
            .build();

        let main_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(8)
            .margin_end(8)
            .margin_top(12)
            .margin_bottom(4)
            .build();

        let label = Label::builder()
            .label(label)
            .css_classes(vec!["dim-label", "caption"])
            .halign(gtk4::Align::Start)
            .build();

        main_box.append(&label);

        row.set_child(Some(&main_box));

        Self { row }
    }
}

/// Drag indicator for folder DnD
pub struct FolderDragIndicator {
    pub widget: GtkBox,
}

impl FolderDragIndicator {
    pub fn new(folder: &Folder) -> Self {
        let widget = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .css_classes(vec!["folder-drag-indicator", "card"])
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let icon = Image::builder()
            .icon_name(folder.icon())
            .build();

        let label = Label::builder()
            .label(folder.display_name())
            .build();

        let count = Label::builder()
            .label(&format!("({})", folder.total_count))
            .css_classes(vec!["dim-label"])
            .build();

        widget.append(&icon);
        widget.append(&label);
        widget.append(&count);

        Self { widget }
    }
}
