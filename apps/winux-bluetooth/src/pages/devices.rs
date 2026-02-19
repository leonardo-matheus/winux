//! Paired devices page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bluez::{BluetoothManager, BluetoothDevice, DeviceType};
use crate::ui::DeviceRow;

/// Paired devices page - shows all paired/connected devices
pub struct DevicesPage {
    widget: gtk4::ScrolledWindow,
    manager: Rc<RefCell<BluetoothManager>>,
}

impl DevicesPage {
    pub fn new(manager: Rc<RefCell<BluetoothManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Dispositivos");
        page.set_icon_name(Some("bluetooth-active-symbolic"));

        // Connected devices group
        let connected_group = adw::PreferencesGroup::builder()
            .title("Dispositivos Conectados")
            .description("Dispositivos atualmente conectados")
            .build();

        // Sample connected devices
        let connected_devices = vec![
            BluetoothDevice::new(
                "00:11:22:33:44:55",
                "Fones Bluetooth XM5",
                DeviceType::Headphones,
                true,
                true,
                Some(85),
            ),
        ];

        for device in &connected_devices {
            let row = Self::create_device_row(device, true);
            connected_group.add(&row);
        }

        if connected_devices.is_empty() {
            let empty_row = adw::ActionRow::builder()
                .title("Nenhum dispositivo conectado")
                .subtitle("Conecte um dispositivo pareado ou descubra novos dispositivos")
                .sensitive(false)
                .build();
            connected_group.add(&empty_row);
        }

        page.add(&connected_group);

        // Paired devices group
        let paired_group = adw::PreferencesGroup::builder()
            .title("Dispositivos Pareados")
            .description("Dispositivos salvos que nao estao conectados")
            .build();

        let paired_devices = vec![
            BluetoothDevice::new(
                "AA:BB:CC:DD:EE:FF",
                "Mouse MX Master 3",
                DeviceType::Mouse,
                true,
                false,
                Some(100),
            ),
            BluetoothDevice::new(
                "11:22:33:44:55:66",
                "Teclado K380",
                DeviceType::Keyboard,
                true,
                false,
                None,
            ),
            BluetoothDevice::new(
                "22:33:44:55:66:77",
                "Galaxy Buds Pro",
                DeviceType::Headphones,
                true,
                false,
                Some(45),
            ),
            BluetoothDevice::new(
                "33:44:55:66:77:88",
                "Controle PS5",
                DeviceType::Gamepad,
                true,
                false,
                Some(75),
            ),
        ];

        for device in &paired_devices {
            let row = Self::create_device_row(device, false);
            paired_group.add(&row);
        }

        if paired_devices.is_empty() {
            let empty_row = adw::ActionRow::builder()
                .title("Nenhum dispositivo pareado")
                .subtitle("Descubra e pareie novos dispositivos")
                .sensitive(false)
                .build();
            paired_group.add(&empty_row);
        }

        page.add(&paired_group);

        // Quick actions group
        let actions_group = adw::PreferencesGroup::builder()
            .title("Acoes Rapidas")
            .build();

        let discover_row = adw::ActionRow::builder()
            .title("Descobrir Dispositivos")
            .subtitle("Buscar dispositivos Bluetooth proximos")
            .activatable(true)
            .build();
        discover_row.add_prefix(&gtk4::Image::from_icon_name("system-search-symbolic"));
        discover_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        actions_group.add(&discover_row);

        let send_file_row = adw::ActionRow::builder()
            .title("Enviar Arquivo")
            .subtitle("Transferir arquivo para dispositivo pareado")
            .activatable(true)
            .build();
        send_file_row.add_prefix(&gtk4::Image::from_icon_name("document-send-symbolic"));
        send_file_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        actions_group.add(&send_file_row);

        page.add(&actions_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    fn create_device_row(device: &BluetoothDevice, connected: bool) -> adw::ExpanderRow {
        let row = adw::ExpanderRow::builder()
            .title(&device.name)
            .subtitle(if connected { "Conectado" } else { "Nao conectado" })
            .build();

        // Device icon
        let icon = gtk4::Image::from_icon_name(device.device_type.icon_name());
        row.add_prefix(&icon);

        // Battery indicator if available
        if let Some(battery) = device.battery {
            let battery_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

            let battery_icon = if battery > 80 {
                "battery-level-100-symbolic"
            } else if battery > 60 {
                "battery-level-80-symbolic"
            } else if battery > 40 {
                "battery-level-60-symbolic"
            } else if battery > 20 {
                "battery-level-40-symbolic"
            } else {
                "battery-level-20-symbolic"
            };

            let icon = gtk4::Image::from_icon_name(battery_icon);
            let label = gtk4::Label::new(Some(&format!("{}%", battery)));
            label.add_css_class("dim-label");

            battery_box.append(&icon);
            battery_box.append(&label);
            battery_box.set_valign(gtk4::Align::Center);

            row.add_suffix(&battery_box);
        }

        // Connection status indicator
        if connected {
            let status_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            status_icon.add_css_class("success");
            row.add_suffix(&status_icon);
        }

        // Expandable actions
        let connect_row = adw::ActionRow::builder()
            .title(if connected { "Desconectar" } else { "Conectar" })
            .activatable(true)
            .build();
        let connect_icon = if connected {
            "network-offline-symbolic"
        } else {
            "network-transmit-symbolic"
        };
        connect_row.add_prefix(&gtk4::Image::from_icon_name(connect_icon));
        row.add_row(&connect_row);

        // Audio profile selection for audio devices
        if matches!(device.device_type, DeviceType::Headphones | DeviceType::Speaker) {
            let profile_row = adw::ComboRow::builder()
                .title("Perfil de Audio")
                .subtitle("Selecione o modo de audio")
                .build();
            let profiles = gtk4::StringList::new(&["A2DP (Alta Qualidade)", "HFP (Headset)", "HSP (Telefone)"]);
            profile_row.set_model(Some(&profiles));
            row.add_row(&profile_row);
        }

        // Properties row
        let props_row = adw::ActionRow::builder()
            .title("Propriedades")
            .subtitle(&format!("MAC: {}", device.address))
            .activatable(true)
            .build();
        props_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        row.add_row(&props_row);

        // Remove row
        let remove_row = adw::ActionRow::builder()
            .title("Remover Dispositivo")
            .subtitle("Desparear e esquecer este dispositivo")
            .activatable(true)
            .build();
        remove_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        remove_row.add_css_class("error");
        row.add_row(&remove_row);

        row
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
