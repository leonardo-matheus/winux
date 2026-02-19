// Main window for Winux Bluetooth

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bluez::BluetoothManager;
use crate::pages::{DevicesPage, ScanPage, TransferPage, SettingsPage};

pub struct BluetoothWindow {
    window: adw::ApplicationWindow,
}

impl BluetoothWindow {
    pub fn new(app: &Application) -> Self {
        let manager = Rc::new(RefCell::new(BluetoothManager::new()));

        let header = adw::HeaderBar::new();

        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        // Devices page (paired devices)
        let devices_page = DevicesPage::new(manager.clone());
        stack.add_titled(devices_page.widget(), Some("devices"), "Dispositivos")
            .set_icon_name(Some("bluetooth-active-symbolic"));

        // Scan page (discover new devices)
        let scan_page = ScanPage::new(manager.clone());
        stack.add_titled(scan_page.widget(), Some("scan"), "Descobrir")
            .set_icon_name(Some("system-search-symbolic"));

        // Transfer page (file transfer)
        let transfer_page = TransferPage::new(manager.clone());
        stack.add_titled(transfer_page.widget(), Some("transfer"), "Transferir")
            .set_icon_name(Some("folder-download-symbolic"));

        // Settings page
        let settings_page = SettingsPage::new(manager.clone());
        stack.add_titled(settings_page.widget(), Some("settings"), "Configuracoes")
            .set_icon_name(Some("emblem-system-symbolic"));

        let switcher = adw::ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        // Bluetooth toggle in header
        let bt_switch = gtk4::Switch::new();
        bt_switch.set_active(true);
        bt_switch.set_valign(gtk4::Align::Center);
        bt_switch.set_tooltip_text(Some("Ligar/Desligar Bluetooth"));

        let manager_clone = manager.clone();
        bt_switch.connect_state_set(move |_, state| {
            manager_clone.borrow_mut().set_powered(state);
            glib::Propagation::Proceed
        });

        header.pack_end(&bt_switch);

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Bluetooth")
            .default_width(800)
            .default_height(600)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
