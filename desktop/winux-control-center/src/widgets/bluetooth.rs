//! Bluetooth control widget
//!
//! Provides Bluetooth toggle and paired device list functionality.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, ListBox, ListBoxRow, Orientation, Popover, Separator};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// Bluetooth device information
#[derive(Clone, Debug)]
pub struct BluetoothDevice {
    pub name: String,
    pub device_type: BluetoothDeviceType,
    pub connected: bool,
    pub battery_level: Option<u8>,
}

/// Type of Bluetooth device
#[derive(Clone, Debug)]
pub enum BluetoothDeviceType {
    Headphones,
    Speaker,
    Keyboard,
    Mouse,
    Phone,
    Watch,
    Other,
}

impl BluetoothDeviceType {
    fn icon_name(&self) -> &str {
        match self {
            BluetoothDeviceType::Headphones => "audio-headphones-symbolic",
            BluetoothDeviceType::Speaker => "audio-speakers-symbolic",
            BluetoothDeviceType::Keyboard => "input-keyboard-symbolic",
            BluetoothDeviceType::Mouse => "input-mouse-symbolic",
            BluetoothDeviceType::Phone => "phone-symbolic",
            BluetoothDeviceType::Watch => "watch-symbolic",
            BluetoothDeviceType::Other => "bluetooth-symbolic",
        }
    }
}

/// Bluetooth control widget with toggle and device list
pub struct BluetoothWidget {
    container: Button,
    icon: Image,
    status: Label,
    enabled: Rc<Cell<bool>>,
    devices: Rc<std::cell::RefCell<Vec<BluetoothDevice>>>,
}

impl BluetoothWidget {
    /// Create a new Bluetooth widget
    pub fn new(initial_state: bool) -> Self {
        let container = Button::builder()
            .build();
        container.add_css_class("control-tile");

        if initial_state {
            container.add_css_class("active");
        }

        let content = Box::new(Orientation::Vertical, 4);
        content.set_halign(gtk::Align::Start);
        content.set_valign(gtk::Align::Center);
        content.set_hexpand(true);

        // Icon
        let icon = Image::from_icon_name("bluetooth-active-symbolic");
        icon.add_css_class("tile-icon");
        icon.set_pixel_size(24);
        icon.set_halign(gtk::Align::Start);
        content.append(&icon);

        // Label
        let label = Label::new(Some("Bluetooth"));
        label.add_css_class("tile-label");
        label.set_halign(gtk::Align::Start);
        content.append(&label);

        // Status
        let status = Label::new(Some(if initial_state { "1 Connected" } else { "Off" }));
        status.add_css_class("tile-status");
        status.set_halign(gtk::Align::Start);
        content.append(&status);

        // Expand button for device list
        let expand_btn = Button::builder()
            .icon_name("go-next-symbolic")
            .build();
        expand_btn.add_css_class("expand-button");
        expand_btn.add_css_class("flat");
        expand_btn.set_valign(gtk::Align::Center);

        let main_content = Box::new(Orientation::Horizontal, 8);
        main_content.append(&content);
        main_content.append(&expand_btn);

        container.set_child(Some(&main_content));

        let enabled = Rc::new(Cell::new(initial_state));
        let devices = Rc::new(std::cell::RefCell::new(Self::get_mock_devices()));

        // Create popover for device list
        let popover = Self::create_device_popover(&devices.borrow());
        popover.set_parent(&expand_btn);

        // Connect expand button
        let popover_clone = popover.clone();
        expand_btn.connect_clicked(move |_| {
            popover_clone.popup();
        });

        // Connect main button click for toggle
        let enabled_clone = enabled.clone();
        let container_clone = container.clone();
        let status_clone = status.clone();
        let icon_clone = icon.clone();

        container.connect_clicked(move |_| {
            let new_state = !enabled_clone.get();
            enabled_clone.set(new_state);

            if new_state {
                container_clone.add_css_class("active");
                status_clone.set_text("On");
                icon_clone.set_icon_name(Some("bluetooth-active-symbolic"));
            } else {
                container_clone.remove_css_class("active");
                status_clone.set_text("Off");
                icon_clone.set_icon_name(Some("bluetooth-disabled-symbolic"));
            }

            info!("Bluetooth toggled: {}", new_state);
        });

        Self {
            container,
            icon,
            status,
            enabled,
            devices,
        }
    }

    /// Create a popover with the device list
    fn create_device_popover(devices: &[BluetoothDevice]) -> Popover {
        let popover = Popover::new();
        popover.add_css_class("network-list");

        let content = Box::new(Orientation::Vertical, 4);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);

        // Header
        let header = Box::new(Orientation::Horizontal, 8);
        let title = Label::new(Some("Bluetooth Devices"));
        title.add_css_class("title-4");
        header.append(&title);
        content.append(&header);

        content.append(&Separator::new(Orientation::Horizontal));

        // Device list
        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .build();
        list.add_css_class("boxed-list");

        let connected: Vec<_> = devices.iter().filter(|d| d.connected).collect();
        let available: Vec<_> = devices.iter().filter(|d| !d.connected).collect();

        // Connected devices section
        if !connected.is_empty() {
            let section_label = Label::new(Some("Connected"));
            section_label.add_css_class("dim-label");
            section_label.set_halign(gtk::Align::Start);
            section_label.set_margin_top(8);
            section_label.set_margin_start(4);
            content.append(&section_label);

            for device in connected {
                let row = Self::create_device_row(device);
                list.append(&row);
            }
        }

        // Available devices section
        if !available.is_empty() {
            let section_label = Label::new(Some("Other Devices"));
            section_label.add_css_class("dim-label");
            section_label.set_halign(gtk::Align::Start);
            section_label.set_margin_top(12);
            section_label.set_margin_start(4);
            content.append(&section_label);

            for device in available {
                let row = Self::create_device_row(device);
                list.append(&row);
            }
        }

        content.append(&list);

        // Settings button
        let settings_btn = Button::builder()
            .label("Bluetooth Settings...")
            .build();
        settings_btn.add_css_class("flat");
        settings_btn.connect_clicked(|_| {
            info!("Opening Bluetooth settings");
            let _ = std::process::Command::new("gnome-control-center")
                .arg("bluetooth")
                .spawn();
        });
        content.append(&settings_btn);

        popover.set_child(Some(&content));
        popover
    }

    /// Create a row for a device in the list
    fn create_device_row(device: &BluetoothDevice) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("network-item");
        if device.connected {
            row.add_css_class("connected");
        }

        let content = Box::new(Orientation::Horizontal, 12);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);

        // Device icon
        let icon = Image::from_icon_name(device.device_type.icon_name());
        icon.set_pixel_size(24);
        content.append(&icon);

        // Device name and status
        let info_box = Box::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);

        let name = Label::new(Some(&device.name));
        name.add_css_class("network-name");
        name.set_halign(gtk::Align::Start);
        info_box.append(&name);

        if device.connected {
            let mut status_text = "Connected".to_string();
            if let Some(battery) = device.battery_level {
                status_text = format!("Connected - {}%", battery);
            }
            let status = Label::new(Some(&status_text));
            status.add_css_class("network-status");
            status.set_halign(gtk::Align::Start);
            info_box.append(&status);
        }

        content.append(&info_box);

        // Battery indicator if available
        if let Some(battery) = device.battery_level {
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
            let battery_img = Image::from_icon_name(battery_icon);
            battery_img.set_opacity(0.6);
            content.append(&battery_img);
        }

        row.set_child(Some(&content));

        // Connect click handler
        let device_name = device.name.clone();
        let connected = device.connected;
        row.set_activatable(true);

        row
    }

    /// Get mock device data for demonstration
    fn get_mock_devices() -> Vec<BluetoothDevice> {
        vec![
            BluetoothDevice {
                name: "AirPods Pro".to_string(),
                device_type: BluetoothDeviceType::Headphones,
                connected: true,
                battery_level: Some(78),
            },
            BluetoothDevice {
                name: "Magic Keyboard".to_string(),
                device_type: BluetoothDeviceType::Keyboard,
                connected: false,
                battery_level: Some(45),
            },
            BluetoothDevice {
                name: "MX Master 3".to_string(),
                device_type: BluetoothDeviceType::Mouse,
                connected: false,
                battery_level: None,
            },
            BluetoothDevice {
                name: "iPhone".to_string(),
                device_type: BluetoothDeviceType::Phone,
                connected: false,
                battery_level: None,
            },
        ]
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Button {
        &self.container
    }

    /// Check if Bluetooth is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Update the connected device count
    pub fn update_connected_count(&self, count: usize) {
        if count > 0 {
            self.status.set_text(&format!("{} Connected", count));
        } else if self.enabled.get() {
            self.status.set_text("On");
        } else {
            self.status.set_text("Off");
        }
    }
}
