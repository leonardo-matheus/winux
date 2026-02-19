//! Device row widget for displaying Bluetooth devices

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bluez::{BluetoothDevice, DeviceType, ConnectionState};

/// Custom widget for displaying a Bluetooth device in a list
pub struct DeviceRow {
    row: adw::ActionRow,
    device: Rc<RefCell<BluetoothDevice>>,
}

impl DeviceRow {
    /// Create a new device row
    pub fn new(device: BluetoothDevice) -> Self {
        let row = adw::ActionRow::builder()
            .title(&device.name)
            .activatable(true)
            .build();

        // Device icon based on type
        let icon = gtk4::Image::from_icon_name(device.device_type.icon_name());
        row.add_prefix(&icon);

        // Connection status
        let status = if device.connected {
            "Conectado"
        } else if device.paired {
            "Pareado"
        } else {
            "Disponivel"
        };
        row.set_subtitle(status);

        // Battery indicator if available
        if let Some(battery) = device.battery {
            let battery_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

            let battery_icon = Self::battery_icon_for_level(battery);
            let icon = gtk4::Image::from_icon_name(battery_icon);
            let label = gtk4::Label::new(Some(&format!("{}%", battery)));
            label.add_css_class("dim-label");

            battery_box.append(&icon);
            battery_box.append(&label);
            battery_box.set_valign(gtk4::Align::Center);

            row.add_suffix(&battery_box);
        }

        // Connection status icon
        if device.connected {
            let status_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            status_icon.add_css_class("success");
            row.add_suffix(&status_icon);
        }

        // Action button
        let action_btn = if device.connected {
            let btn = gtk4::Button::with_label("Desconectar");
            btn.add_css_class("flat");
            btn
        } else if device.paired {
            let btn = gtk4::Button::with_label("Conectar");
            btn.add_css_class("flat");
            btn
        } else {
            let btn = gtk4::Button::with_label("Parear");
            btn.add_css_class("suggested-action");
            btn.add_css_class("flat");
            btn
        };
        action_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&action_btn);

        // Navigate arrow
        let arrow = gtk4::Image::from_icon_name("go-next-symbolic");
        row.add_suffix(&arrow);

        Self {
            row,
            device: Rc::new(RefCell::new(device)),
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &adw::ActionRow {
        &self.row
    }

    /// Get the device
    pub fn device(&self) -> std::cell::Ref<'_, BluetoothDevice> {
        self.device.borrow()
    }

    /// Update device information
    pub fn update(&self, device: BluetoothDevice) {
        *self.device.borrow_mut() = device;
        self.refresh_ui();
    }

    /// Refresh UI from current device state
    fn refresh_ui(&self) {
        let device = self.device.borrow();
        self.row.set_title(&device.name);

        let status = if device.connected {
            "Conectado"
        } else if device.paired {
            "Pareado"
        } else {
            "Disponivel"
        };
        self.row.set_subtitle(status);
    }

    /// Get battery icon for level
    fn battery_icon_for_level(level: u8) -> &'static str {
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

    /// Create a compact device row (for discovered devices)
    pub fn new_compact(device: &BluetoothDevice) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&device.name)
            .subtitle(&format!("{} - {}", device.device_type.display_name(), device.address))
            .activatable(true)
            .build();

        // Device icon
        let icon = gtk4::Image::from_icon_name(device.device_type.icon_name());
        row.add_prefix(&icon);

        // Signal strength indicator
        if let Some(rssi) = device.rssi {
            let signal_icon = Self::signal_icon_for_rssi(rssi);
            let icon = gtk4::Image::from_icon_name(signal_icon);
            icon.set_tooltip_text(Some(&format!("Sinal: {} dBm", rssi)));
            row.add_suffix(&icon);
        }

        // Pair button
        let pair_btn = gtk4::Button::with_label("Parear");
        pair_btn.add_css_class("flat");
        pair_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&pair_btn);

        row
    }

    /// Get signal icon for RSSI value
    fn signal_icon_for_rssi(rssi: i16) -> &'static str {
        if rssi > -50 {
            "network-wireless-signal-excellent-symbolic"
        } else if rssi > -60 {
            "network-wireless-signal-good-symbolic"
        } else if rssi > -70 {
            "network-wireless-signal-ok-symbolic"
        } else {
            "network-wireless-signal-weak-symbolic"
        }
    }

    /// Create an expandable device row with detailed actions
    pub fn new_expandable(device: &BluetoothDevice, connected: bool) -> adw::ExpanderRow {
        let row = adw::ExpanderRow::builder()
            .title(&device.name)
            .subtitle(if connected { "Conectado" } else { "Nao conectado" })
            .build();

        // Device icon
        let icon = gtk4::Image::from_icon_name(device.device_type.icon_name());
        row.add_prefix(&icon);

        // Battery
        if let Some(battery) = device.battery {
            let battery_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
            let icon = gtk4::Image::from_icon_name(Self::battery_icon_for_level(battery));
            let label = gtk4::Label::new(Some(&format!("{}%", battery)));
            label.add_css_class("dim-label");
            battery_box.append(&icon);
            battery_box.append(&label);
            battery_box.set_valign(gtk4::Align::Center);
            row.add_suffix(&battery_box);
        }

        // Status icon
        if connected {
            let status_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            status_icon.add_css_class("success");
            row.add_suffix(&status_icon);
        }

        // Connection action
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

        // Audio profiles for audio devices
        if device.is_audio_device() {
            let profile_row = adw::ComboRow::builder()
                .title("Perfil de Audio")
                .build();
            let profiles = gtk4::StringList::new(&[
                "A2DP (Alta Qualidade)",
                "HFP (Headset)",
            ]);
            profile_row.set_model(Some(&profiles));
            row.add_row(&profile_row);
        }

        // Properties
        let props_row = adw::ActionRow::builder()
            .title("Propriedades")
            .subtitle(&format!("MAC: {}", device.address))
            .activatable(true)
            .build();
        props_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        row.add_row(&props_row);

        // Remove device
        let remove_row = adw::ActionRow::builder()
            .title("Remover Dispositivo")
            .activatable(true)
            .build();
        remove_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        remove_row.add_css_class("error");
        row.add_row(&remove_row);

        row
    }
}

/// Device list widget for managing multiple device rows
pub struct DeviceList {
    list_box: gtk4::ListBox,
    devices: Rc<RefCell<Vec<DeviceRow>>>,
}

impl DeviceList {
    /// Create a new device list
    pub fn new() -> Self {
        let list_box = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(vec!["boxed-list".to_string()])
            .build();

        Self {
            list_box,
            devices: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &gtk4::ListBox {
        &self.list_box
    }

    /// Add a device to the list
    pub fn add_device(&self, device: BluetoothDevice) {
        let row = DeviceRow::new(device);
        self.list_box.append(row.widget());
        self.devices.borrow_mut().push(row);
    }

    /// Clear all devices
    pub fn clear(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        self.devices.borrow_mut().clear();
    }

    /// Update a device by address
    pub fn update_device(&self, address: &str, device: BluetoothDevice) {
        for row in self.devices.borrow().iter() {
            if row.device().address == address {
                row.update(device);
                break;
            }
        }
    }

    /// Remove a device by address
    pub fn remove_device(&self, address: &str) {
        let mut devices = self.devices.borrow_mut();
        if let Some(pos) = devices.iter().position(|r| r.device().address == address) {
            let row = devices.remove(pos);
            self.list_box.remove(row.widget());
        }
    }
}

impl Default for DeviceList {
    fn default() -> Self {
        Self::new()
    }
}
