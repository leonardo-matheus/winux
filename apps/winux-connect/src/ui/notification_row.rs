//! Notification row widget for displaying phone notifications

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::pages::notifications::PhoneNotification;

/// Notification row widget
pub struct NotificationRow {
    widget: adw::ActionRow,
}

impl NotificationRow {
    pub fn new(notification: &PhoneNotification) -> Self {
        let row = adw::ActionRow::builder()
            .title(&notification.title)
            .subtitle(&notification.body)
            .activatable(true)
            .build();

        // App icon
        let icon = Self::get_app_icon(&notification.app_name);
        row.add_prefix(&gtk4::Image::from_icon_name(icon));

        // App name and time label
        let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        info_box.set_valign(gtk4::Align::Center);

        let app_label = gtk4::Label::new(Some(&notification.app_name));
        app_label.add_css_class("caption");
        app_label.add_css_class("dim-label");
        app_label.set_halign(gtk4::Align::End);
        info_box.append(&app_label);

        let time_label = gtk4::Label::new(Some(&notification.timestamp));
        time_label.add_css_class("caption");
        time_label.add_css_class("dim-label");
        time_label.set_halign(gtk4::Align::End);
        info_box.append(&time_label);

        row.add_suffix(&info_box);

        // Dismiss button
        if notification.dismissible {
            let dismiss_btn = gtk4::Button::from_icon_name("window-close-symbolic");
            dismiss_btn.set_valign(gtk4::Align::Center);
            dismiss_btn.add_css_class("flat");
            dismiss_btn.set_tooltip_text(Some("Dispensar"));
            row.add_suffix(&dismiss_btn);
        }

        Self { widget: row }
    }

    fn get_app_icon(app_name: &str) -> &'static str {
        match app_name.to_lowercase().as_str() {
            "whatsapp" => "chat-symbolic",
            "telegram" => "chat-symbolic",
            "gmail" | "email" | "mail" => "mail-unread-symbolic",
            "instagram" => "camera-photo-symbolic",
            "twitter" | "x" => "user-available-symbolic",
            "facebook" => "system-users-symbolic",
            "calendar" | "calendario" => "x-office-calendar-symbolic",
            "phone" | "telefone" => "call-start-symbolic",
            "messages" | "mensagens" | "sms" => "mail-unread-symbolic",
            "bank" | "banco" => "wallet-symbolic",
            "spotify" | "music" | "musica" => "audio-x-generic-symbolic",
            _ => "preferences-system-notifications-symbolic",
        }
    }

    pub fn widget(&self) -> adw::ActionRow {
        self.widget.clone()
    }
}

/// Create a notification row with reply support
pub struct NotificationRowWithReply {
    widget: adw::ExpanderRow,
}

impl NotificationRowWithReply {
    pub fn new(notification: &PhoneNotification) -> Self {
        let row = adw::ExpanderRow::builder()
            .title(&notification.title)
            .subtitle(&notification.body)
            .build();

        // App icon
        let icon = NotificationRow::get_app_icon(&notification.app_name);
        row.add_prefix(&gtk4::Image::from_icon_name(icon));

        // App name and time
        let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        info_box.set_valign(gtk4::Align::Center);

        let app_label = gtk4::Label::new(Some(&notification.app_name));
        app_label.add_css_class("caption");
        app_label.add_css_class("dim-label");
        info_box.append(&app_label);

        let time_label = gtk4::Label::new(Some(&notification.timestamp));
        time_label.add_css_class("caption");
        time_label.add_css_class("dim-label");
        info_box.append(&time_label);

        row.add_suffix(&info_box);

        // Reply row
        let reply_row = adw::ActionRow::new();

        let reply_entry = gtk4::Entry::builder()
            .placeholder_text("Digite uma resposta...")
            .hexpand(true)
            .build();

        let send_btn = gtk4::Button::from_icon_name("send-symbolic");
        send_btn.add_css_class("suggested-action");

        let reply_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        reply_box.set_margin_start(12);
        reply_box.set_margin_end(12);
        reply_box.set_margin_top(8);
        reply_box.set_margin_bottom(8);
        reply_box.append(&reply_entry);
        reply_box.append(&send_btn);

        reply_row.set_child(Some(&reply_box));
        row.add_row(&reply_row);

        // Actions row
        for action in &notification.actions {
            let action_row = adw::ActionRow::builder()
                .title(action)
                .activatable(true)
                .build();
            row.add_row(&action_row);
        }

        // Dismiss row
        let dismiss_row = adw::ActionRow::builder()
            .title("Dispensar")
            .activatable(true)
            .build();
        dismiss_row.add_prefix(&gtk4::Image::from_icon_name("window-close-symbolic"));
        row.add_row(&dismiss_row);

        Self { widget: row }
    }

    pub fn widget(&self) -> adw::ExpanderRow {
        self.widget.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_icon_mapping() {
        assert_eq!(NotificationRow::get_app_icon("WhatsApp"), "chat-symbolic");
        assert_eq!(NotificationRow::get_app_icon("Gmail"), "mail-unread-symbolic");
        assert_eq!(NotificationRow::get_app_icon("Unknown"), "preferences-system-notifications-symbolic");
    }
}
