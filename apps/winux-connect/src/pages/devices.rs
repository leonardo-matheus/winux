//! Devices page - List of paired and discovered devices

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::protocol::{ConnectionManager, Device, DeviceType, DeviceStatus};
use crate::ui::DeviceCard;

/// Paired and discovered devices page
pub struct DevicesPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<ConnectionManager>>,
}

impl DevicesPage {
    pub fn new(manager: Rc<RefCell<ConnectionManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Dispositivos");
        page.set_icon_name(Some("phone-symbolic"));

        // Connected devices group
        let connected_group = adw::PreferencesGroup::builder()
            .title("Dispositivos Conectados")
            .description("Dispositivos atualmente conectados")
            .build();

        // Sample connected devices
        let connected_devices = vec![
            Device::new(
                "abc123",
                "Samsung Galaxy S24",
                DeviceType::Phone,
                DeviceStatus::Connected,
                Some(78),
                "192.168.1.105",
            ),
            Device::new(
                "def456",
                "iPad Pro",
                DeviceType::Tablet,
                DeviceStatus::Connected,
                Some(95),
                "192.168.1.110",
            ),
        ];

        for device in &connected_devices {
            let card = DeviceCard::new(device);
            connected_group.add(&card.widget());
        }

        if connected_devices.is_empty() {
            let empty_row = adw::ActionRow::builder()
                .title("Nenhum dispositivo conectado")
                .subtitle("Conecte um dispositivo pareado ou descubra novos")
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
            Device::new(
                "ghi789",
                "Pixel 8 Pro",
                DeviceType::Phone,
                DeviceStatus::Paired,
                None,
                "192.168.1.120",
            ),
            Device::new(
                "jkl012",
                "OnePlus 12",
                DeviceType::Phone,
                DeviceStatus::Paired,
                None,
                "192.168.1.125",
            ),
        ];

        for device in &paired_devices {
            let card = DeviceCard::new(device);
            paired_group.add(&card.widget());
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

        // Discovered devices group
        let discovered_group = adw::PreferencesGroup::builder()
            .title("Dispositivos Descobertos")
            .description("Dispositivos encontrados na rede")
            .build();

        let discovered_devices = vec![
            Device::new(
                "mno345",
                "iPhone 15 de Maria",
                DeviceType::Phone,
                DeviceStatus::Discovered,
                None,
                "192.168.1.130",
            ),
        ];

        for device in &discovered_devices {
            let row = adw::ActionRow::builder()
                .title(&device.name)
                .subtitle(&format!("IP: {}", device.ip_address))
                .activatable(true)
                .build();

            row.add_prefix(&gtk4::Image::from_icon_name(device.device_type.icon_name()));

            let pair_button = gtk4::Button::builder()
                .label("Parear")
                .valign(gtk4::Align::Center)
                .build();
            pair_button.add_css_class("suggested-action");
            row.add_suffix(&pair_button);

            discovered_group.add(&row);
        }

        // Scanning indicator
        let scanning_row = adw::ActionRow::builder()
            .title("Buscando dispositivos...")
            .sensitive(false)
            .build();
        let spinner = gtk4::Spinner::new();
        spinner.set_spinning(true);
        scanning_row.add_prefix(&spinner);
        discovered_group.add(&scanning_row);

        page.add(&discovered_group);

        // Quick actions
        let actions_group = adw::PreferencesGroup::builder()
            .title("Acoes Rapidas")
            .build();

        let add_ip_row = adw::ActionRow::builder()
            .title("Adicionar por IP")
            .subtitle("Conectar manualmente a um dispositivo")
            .activatable(true)
            .build();
        add_ip_row.add_prefix(&gtk4::Image::from_icon_name("network-server-symbolic"));
        add_ip_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        actions_group.add(&add_ip_row);

        let qr_row = adw::ActionRow::builder()
            .title("Parear com QR Code")
            .subtitle("Escanear QR code para pareamento rapido")
            .activatable(true)
            .build();
        qr_row.add_prefix(&gtk4::Image::from_icon_name("view-barcode-symbolic"));
        qr_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        actions_group.add(&qr_row);

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

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
