//! Custom user row widget

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::backend::{AccountsService, AccountType};

/// Custom user row widget with avatar and actions
pub struct UserRow {
    widget: adw::ActionRow,
    username: String,
    avatar: adw::Avatar,
}

impl UserRow {
    /// Create a new user row
    pub fn new(
        username: &str,
        real_name: &str,
        account_type: AccountType,
        is_current_user: bool,
        icon_path: Option<&std::path::Path>,
    ) -> Self {
        let row = adw::ActionRow::builder()
            .title(if real_name.is_empty() { username } else { real_name })
            .subtitle(Self::format_subtitle(username, account_type))
            .activatable(true)
            .build();

        // Avatar
        let avatar = adw::Avatar::new(48, Some(real_name), true);

        // Load custom icon if available
        if let Some(path) = icon_path {
            if let Ok(texture) = gdk4::Texture::from_filename(path) {
                avatar.set_custom_image(Some(&texture));
            }
        }

        row.add_prefix(&avatar);

        // Current user badge
        if is_current_user {
            let badge = Label::new(Some("Voce"));
            badge.add_css_class("dim-label");
            badge.add_css_class("caption");
            row.add_suffix(&badge);
        }

        // Admin badge
        if matches!(account_type, AccountType::Administrator) {
            let admin_icon = gtk4::Image::from_icon_name("security-high-symbolic");
            admin_icon.set_tooltip_text(Some("Administrador"));
            admin_icon.add_css_class("accent");
            row.add_suffix(&admin_icon);
        }

        // Edit button
        let edit_btn = Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_tooltip_text(Some("Editar usuario"));
        edit_btn.add_css_class("flat");
        edit_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&edit_btn);

        // Navigation arrow
        row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

        UserRow {
            widget: row,
            username: username.to_string(),
            avatar,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &adw::ActionRow {
        &self.widget
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Update the avatar
    pub fn set_avatar(&self, icon_path: &std::path::Path) {
        if let Ok(texture) = gdk4::Texture::from_filename(icon_path) {
            self.avatar.set_custom_image(Some(&texture));
        }
    }

    /// Update the real name
    pub fn set_real_name(&self, real_name: &str) {
        self.widget.set_title(real_name);
        self.avatar.set_text(Some(real_name));
    }

    /// Connect to edit button click
    pub fn connect_edit<F: Fn(&str) + 'static>(&self, callback: F) {
        let username = self.username.clone();
        // Note: We would need to get the edit button reference to connect this
        // For now, this is a placeholder for the implementation
        tracing::debug!("Edit callback set for user: {}", username);
    }

    /// Connect to row activation
    pub fn connect_activated<F: Fn(&str) + 'static>(&self, callback: F) {
        let username = self.username.clone();
        self.widget.connect_activated(move |_| {
            callback(&username);
        });
    }

    fn format_subtitle(username: &str, account_type: AccountType) -> String {
        let type_str = match account_type {
            AccountType::Administrator => "Administrador",
            AccountType::Standard => "Usuario Padrao",
        };
        format!("{} - {}", username, type_str)
    }
}

/// Create a simple user row without full features
pub fn create_simple_user_row(username: &str, real_name: &str, subtitle: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(if real_name.is_empty() { username } else { real_name })
        .subtitle(subtitle)
        .activatable(true)
        .build();

    let avatar = adw::Avatar::new(40, Some(real_name), true);
    row.add_prefix(&avatar);

    row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

    row
}

/// Create a user row with checkbox for selection
pub fn create_selectable_user_row(
    username: &str,
    real_name: &str,
    selected: bool,
) -> (adw::ActionRow, gtk4::CheckButton) {
    let row = adw::ActionRow::builder()
        .title(if real_name.is_empty() { username } else { real_name })
        .subtitle(username)
        .activatable(true)
        .build();

    let avatar = adw::Avatar::new(32, Some(real_name), true);
    row.add_prefix(&avatar);

    let check = gtk4::CheckButton::new();
    check.set_active(selected);
    row.add_suffix(&check);

    // Make row click toggle the checkbox
    let check_clone = check.clone();
    row.connect_activated(move |_| {
        check_clone.set_active(!check_clone.is_active());
    });

    (row, check)
}
