//! Backup card widget - displays backup information

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::backends::{BackupMetadata, BackupType};

/// A card widget displaying backup information
pub struct BackupCard {
    widget: Box,
}

impl BackupCard {
    /// Create a new backup card from metadata
    pub fn new(metadata: &BackupMetadata) -> Self {
        let card = Box::new(Orientation::Horizontal, 12);
        card.set_margin_top(8);
        card.set_margin_bottom(8);
        card.set_margin_start(12);
        card.set_margin_end(12);
        card.add_css_class("card");

        // Icon based on backup type
        let icon_name = match metadata.backup_type {
            BackupType::System => "computer-symbolic",
            BackupType::Home => "user-home-symbolic",
            BackupType::Custom => "folder-symbolic",
            BackupType::Config => "applications-system-symbolic",
        };
        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(48);
        icon.set_margin_start(12);
        icon.set_margin_top(12);
        icon.set_margin_bottom(12);
        card.append(&icon);

        // Info section
        let info_box = Box::new(Orientation::Vertical, 4);
        info_box.set_hexpand(true);
        info_box.set_margin_top(12);
        info_box.set_margin_bottom(12);

        let name_label = Label::new(Some(&metadata.name));
        name_label.set_halign(gtk4::Align::Start);
        name_label.add_css_class("heading");
        info_box.append(&name_label);

        let date_str = metadata.timestamp.format("%d/%m/%Y %H:%M").to_string();
        let date_label = Label::new(Some(&date_str));
        date_label.set_halign(gtk4::Align::Start);
        date_label.add_css_class("dim-label");
        info_box.append(&date_label);

        let size_str = format_size(metadata.size_bytes);
        let files_str = format!("{} arquivos", metadata.file_count);
        let details_label = Label::new(Some(&format!("{} - {}", size_str, files_str)));
        details_label.set_halign(gtk4::Align::Start);
        details_label.add_css_class("caption");
        info_box.append(&details_label);

        card.append(&info_box);

        // Status indicators
        let status_box = Box::new(Orientation::Vertical, 4);
        status_box.set_margin_top(12);
        status_box.set_margin_bottom(12);

        // Encryption indicator
        if metadata.encrypted {
            let encrypted_icon = Image::from_icon_name("channel-secure-symbolic");
            encrypted_icon.set_tooltip_text(Some("Criptografado"));
            status_box.append(&encrypted_icon);
        }

        // Verified indicator
        if metadata.verified {
            let verified_icon = Image::from_icon_name("emblem-ok-symbolic");
            verified_icon.set_tooltip_text(Some("Verificado"));
            verified_icon.add_css_class("success");
            status_box.append(&verified_icon);
        }

        card.append(&status_box);

        // Action buttons
        let actions_box = Box::new(Orientation::Vertical, 4);
        actions_box.set_margin_top(8);
        actions_box.set_margin_bottom(8);
        actions_box.set_margin_end(12);
        actions_box.set_valign(gtk4::Align::Center);

        let restore_btn = Button::from_icon_name("edit-undo-symbolic");
        restore_btn.set_tooltip_text(Some("Restaurar"));
        restore_btn.add_css_class("flat");
        restore_btn.add_css_class("circular");
        actions_box.append(&restore_btn);

        let browse_btn = Button::from_icon_name("folder-open-symbolic");
        browse_btn.set_tooltip_text(Some("Navegar"));
        browse_btn.add_css_class("flat");
        browse_btn.add_css_class("circular");
        actions_box.append(&browse_btn);

        let delete_btn = Button::from_icon_name("user-trash-symbolic");
        delete_btn.set_tooltip_text(Some("Excluir"));
        delete_btn.add_css_class("flat");
        delete_btn.add_css_class("circular");
        actions_box.append(&delete_btn);

        card.append(&actions_box);

        Self { widget: card }
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.widget
    }
}

/// Format bytes to human readable string
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Create a minimal backup row for lists
pub fn create_backup_row(metadata: &BackupMetadata) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(&metadata.name)
        .subtitle(&format!(
            "{} - {}",
            metadata.timestamp.format("%d/%m/%Y %H:%M"),
            format_size(metadata.size_bytes)
        ))
        .activatable(true)
        .build();

    // Icon based on backup type
    let icon_name = match metadata.backup_type {
        BackupType::System => "computer-symbolic",
        BackupType::Home => "user-home-symbolic",
        BackupType::Custom => "folder-symbolic",
        BackupType::Config => "applications-system-symbolic",
    };
    row.add_prefix(&Image::from_icon_name(icon_name));

    // Status icon
    let status_icon = if metadata.verified {
        "emblem-ok-symbolic"
    } else {
        "dialog-question-symbolic"
    };
    row.add_suffix(&Image::from_icon_name(status_icon));

    row.add_suffix(&Image::from_icon_name("go-next-symbolic"));

    row
}
