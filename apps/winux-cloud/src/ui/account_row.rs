//! Account row widget

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;

use crate::providers::{CloudAccount, ProviderType};

/// Account row widget for displaying cloud account information
pub struct AccountRow {
    widget: adw::ActionRow,
}

impl AccountRow {
    /// Create a new account row
    pub fn new(account: &CloudAccount) -> Self {
        let row = adw::ActionRow::builder()
            .title(&account.name)
            .activatable(true)
            .build();

        // Provider icon
        let icon = Image::from_icon_name(account.provider.icon_name());
        icon.set_pixel_size(32);
        row.add_prefix(&icon);

        // Subtitle with email and storage info
        let mut subtitle_parts = Vec::new();
        if let Some(email) = &account.email {
            subtitle_parts.push(email.clone());
        }
        if let Some(quota) = &account.quota {
            subtitle_parts.push(format!(
                "{} usado de {}",
                format_bytes(quota.used),
                format_bytes(quota.total)
            ));
        }
        row.set_subtitle(&subtitle_parts.join("\n"));

        // Sync status indicator
        if account.sync_enabled {
            let status_icon = Image::from_icon_name("emblem-ok-symbolic");
            status_icon.set_tooltip_text(Some("Sincronizado"));
            row.add_suffix(&status_icon);
        } else {
            let status_icon = Image::from_icon_name("media-playback-pause-symbolic");
            status_icon.set_tooltip_text(Some("Sincronizacao pausada"));
            row.add_suffix(&status_icon);
        }

        // Settings button
        let settings_btn = Button::from_icon_name("emblem-system-symbolic");
        settings_btn.add_css_class("flat");
        settings_btn.set_valign(gtk4::Align::Center);
        settings_btn.set_tooltip_text(Some("Configuracoes da conta"));
        row.add_suffix(&settings_btn);

        Self { widget: row }
    }

    /// Create a row for adding a new provider
    pub fn new_add_provider(provider_type: ProviderType, description: &str) -> Self {
        let row = adw::ActionRow::builder()
            .title(provider_type.display_name())
            .subtitle(description)
            .activatable(true)
            .build();

        let icon = Image::from_icon_name(provider_type.icon_name());
        icon.set_pixel_size(32);
        row.add_prefix(&icon);

        let connect_btn = Button::with_label("Conectar");
        connect_btn.add_css_class("suggested-action");
        connect_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&connect_btn);

        Self { widget: row }
    }

    /// Get the widget
    pub fn widget(&self) -> &adw::ActionRow {
        &self.widget
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
