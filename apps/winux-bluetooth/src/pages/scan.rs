//! Device discovery/scan page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bluez::{BluetoothManager, BluetoothDevice, DeviceType};

/// Scan page for discovering nearby Bluetooth devices
pub struct ScanPage {
    widget: gtk4::ScrolledWindow,
    manager: Rc<RefCell<BluetoothManager>>,
}

impl ScanPage {
    pub fn new(manager: Rc<RefCell<BluetoothManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Descobrir");
        page.set_icon_name(Some("system-search-symbolic"));

        // Scan controls group
        let controls_group = adw::PreferencesGroup::builder()
            .title("Busca de Dispositivos")
            .description("Encontre dispositivos Bluetooth proximos")
            .build();

        // Scan status row
        let scan_row = adw::ActionRow::builder()
            .title("Buscando dispositivos...")
            .subtitle("Certifique-se que o dispositivo esta em modo de pareamento")
            .build();

        let spinner = gtk4::Spinner::new();
        spinner.set_spinning(false);
        scan_row.add_prefix(&spinner);

        let scan_btn = gtk4::Button::with_label("Iniciar Busca");
        scan_btn.add_css_class("suggested-action");
        scan_btn.set_valign(gtk4::Align::Center);

        let spinner_clone = spinner.clone();
        let btn_clone = scan_btn.clone();
        scan_btn.connect_clicked(move |_| {
            if spinner_clone.is_spinning() {
                spinner_clone.set_spinning(false);
                btn_clone.set_label("Iniciar Busca");
                btn_clone.remove_css_class("destructive-action");
                btn_clone.add_css_class("suggested-action");
            } else {
                spinner_clone.set_spinning(true);
                btn_clone.set_label("Parar Busca");
                btn_clone.remove_css_class("suggested-action");
                btn_clone.add_css_class("destructive-action");
            }
        });

        scan_row.add_suffix(&scan_btn);
        controls_group.add(&scan_row);

        page.add(&controls_group);

        // Filter group
        let filter_group = adw::PreferencesGroup::builder()
            .title("Filtrar por Tipo")
            .build();

        let filter_row = adw::ComboRow::builder()
            .title("Tipo de Dispositivo")
            .subtitle("Mostrar apenas dispositivos especificos")
            .build();
        let filters = gtk4::StringList::new(&[
            "Todos os Dispositivos",
            "Audio (Fones, Caixas)",
            "Entrada (Teclado, Mouse)",
            "Telefones",
            "Computadores",
            "Outros",
        ]);
        filter_row.set_model(Some(&filters));
        filter_group.add(&filter_row);

        page.add(&filter_group);

        // Discovered devices group
        let discovered_group = adw::PreferencesGroup::builder()
            .title("Dispositivos Encontrados")
            .description("Clique para parear")
            .build();

        // Sample discovered devices
        let discovered_devices = vec![
            BluetoothDevice::new(
                "AA:11:BB:22:CC:33",
                "AirPods Pro",
                DeviceType::Headphones,
                false,
                false,
                None,
            ),
            BluetoothDevice::new(
                "DD:44:EE:55:FF:66",
                "JBL Flip 5",
                DeviceType::Speaker,
                false,
                false,
                None,
            ),
            BluetoothDevice::new(
                "77:88:99:AA:BB:CC",
                "iPhone 15",
                DeviceType::Phone,
                false,
                false,
                None,
            ),
            BluetoothDevice::new(
                "11:AA:22:BB:33:CC",
                "Logitech G Pro",
                DeviceType::Mouse,
                false,
                false,
                None,
            ),
            BluetoothDevice::new(
                "44:DD:55:EE:66:FF",
                "Smartwatch",
                DeviceType::Watch,
                false,
                false,
                None,
            ),
            BluetoothDevice::new(
                "99:00:88:11:77:22",
                "Dispositivo Desconhecido",
                DeviceType::Unknown,
                false,
                false,
                None,
            ),
        ];

        for device in &discovered_devices {
            let row = Self::create_discovered_row(device);
            discovered_group.add(&row);
        }

        page.add(&discovered_group);

        // Tips group
        let tips_group = adw::PreferencesGroup::builder()
            .title("Dicas")
            .build();

        let tip1 = adw::ActionRow::builder()
            .title("Modo de Pareamento")
            .subtitle("Coloque o dispositivo em modo de pareamento antes de buscar")
            .build();
        tip1.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        tips_group.add(&tip1);

        let tip2 = adw::ActionRow::builder()
            .title("Distancia")
            .subtitle("Mantenha os dispositivos proximos durante o pareamento")
            .build();
        tip2.add_prefix(&gtk4::Image::from_icon_name("network-wireless-signal-excellent-symbolic"));
        tips_group.add(&tip2);

        page.add(&tips_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    fn create_discovered_row(device: &BluetoothDevice) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&device.name)
            .subtitle(&format!("{} - {}", device.device_type.display_name(), device.address))
            .activatable(true)
            .build();

        // Device type icon
        let icon = gtk4::Image::from_icon_name(device.device_type.icon_name());
        row.add_prefix(&icon);

        // Signal strength indicator (simulated)
        let signal_icon = gtk4::Image::from_icon_name("network-wireless-signal-good-symbolic");
        signal_icon.set_tooltip_text(Some("Sinal: Bom"));
        row.add_suffix(&signal_icon);

        // Pair button
        let pair_btn = gtk4::Button::with_label("Parear");
        pair_btn.add_css_class("flat");
        pair_btn.set_valign(gtk4::Align::Center);

        pair_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            btn.set_label("Pareando...");
            // In real implementation, this would trigger pairing
        });

        row.add_suffix(&pair_btn);

        row
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
