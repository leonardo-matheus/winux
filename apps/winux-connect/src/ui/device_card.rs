//! Device card widget for displaying device information

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::protocol::{Device, DeviceStatus};

/// Device card widget showing device info and actions
pub struct DeviceCard {
    widget: adw::ExpanderRow,
}

impl DeviceCard {
    pub fn new(device: &Device) -> Self {
        let row = adw::ExpanderRow::builder()
            .title(&device.name)
            .subtitle(device.status.as_str())
            .build();

        // Device icon
        let icon = gtk4::Image::from_icon_name(device.device_type.icon_name());
        row.add_prefix(&icon);

        // Status indicator
        let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        status_box.set_valign(gtk4::Align::Center);

        // Battery indicator if available
        if let Some(battery) = device.battery {
            let battery_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

            let battery_icon = Self::get_battery_icon(battery);
            let icon = gtk4::Image::from_icon_name(battery_icon);
            let label = gtk4::Label::new(Some(&format!("{}%", battery)));
            label.add_css_class("dim-label");

            battery_box.append(&icon);
            battery_box.append(&label);

            status_box.append(&battery_box);
        }

        // Connection status
        if device.status == DeviceStatus::Connected {
            let connected_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            connected_icon.add_css_class("success");
            status_box.append(&connected_icon);
        }

        row.add_suffix(&status_box);

        // Expandable content with actions
        Self::add_device_actions(&row, device);

        Self { widget: row }
    }

    fn get_battery_icon(level: u8) -> &'static str {
        if level > 80 {
            "battery-level-100-symbolic"
        } else if level > 60 {
            "battery-level-80-symbolic"
        } else if level > 40 {
            "battery-level-60-symbolic"
        } else if level > 20 {
            "battery-level-40-symbolic"
        } else if level > 10 {
            "battery-level-20-symbolic"
        } else {
            "battery-level-10-symbolic"
        }
    }

    fn add_device_actions(row: &adw::ExpanderRow, device: &Device) {
        // Connect/Disconnect action
        let connect_action = if device.status == DeviceStatus::Connected {
            adw::ActionRow::builder()
                .title("Desconectar")
                .activatable(true)
                .build()
        } else {
            adw::ActionRow::builder()
                .title("Conectar")
                .activatable(true)
                .build()
        };
        connect_action.add_prefix(&gtk4::Image::from_icon_name(
            if device.status == DeviceStatus::Connected {
                "network-offline-symbolic"
            } else {
                "network-transmit-symbolic"
            }
        ));
        row.add_row(&connect_action);

        // Send file action
        let send_file = adw::ActionRow::builder()
            .title("Enviar Arquivo")
            .subtitle("Transferir arquivo para este dispositivo")
            .activatable(true)
            .build();
        send_file.add_prefix(&gtk4::Image::from_icon_name("document-send-symbolic"));
        row.add_row(&send_file);

        // Find phone action
        let find_phone = adw::ActionRow::builder()
            .title("Localizar Telefone")
            .subtitle("Fazer o telefone tocar")
            .activatable(true)
            .build();
        find_phone.add_prefix(&gtk4::Image::from_icon_name("find-location-symbolic"));
        row.add_row(&find_phone);

        // Share clipboard action
        let share_clipboard = adw::ActionRow::builder()
            .title("Enviar Clipboard")
            .subtitle("Enviar conteudo da area de transferencia")
            .activatable(true)
            .build();
        share_clipboard.add_prefix(&gtk4::Image::from_icon_name("edit-paste-symbolic"));
        row.add_row(&share_clipboard);

        // Device info
        let info = adw::ActionRow::builder()
            .title("Informacoes")
            .subtitle(&format!("IP: {} | Porta: {}", device.ip_address, device.port))
            .build();
        info.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        row.add_row(&info);

        // Capabilities
        let caps_text = device.capabilities.iter()
            .map(|c| c.replace("kdeconnect.", ""))
            .collect::<Vec<_>>()
            .join(", ");
        let capabilities = adw::ActionRow::builder()
            .title("Recursos")
            .subtitle(&caps_text)
            .build();
        capabilities.add_prefix(&gtk4::Image::from_icon_name("applications-system-symbolic"));
        row.add_row(&capabilities);

        // Unpair action
        let unpair = adw::ActionRow::builder()
            .title("Desparear")
            .subtitle("Remover pareamento com este dispositivo")
            .activatable(true)
            .build();
        unpair.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        unpair.add_css_class("error");
        row.add_row(&unpair);
    }

    pub fn widget(&self) -> adw::ExpanderRow {
        self.widget.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::DeviceType;

    fn setup_gtk() {
        let _ = gtk4::init();
    }

    #[test]
    fn test_battery_icon() {
        assert_eq!(DeviceCard::get_battery_icon(100), "battery-level-100-symbolic");
        assert_eq!(DeviceCard::get_battery_icon(50), "battery-level-60-symbolic");
        assert_eq!(DeviceCard::get_battery_icon(5), "battery-level-10-symbolic");
    }
}
