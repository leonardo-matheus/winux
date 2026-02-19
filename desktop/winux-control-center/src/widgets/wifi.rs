//! WiFi control widget
//!
//! Provides WiFi toggle and network list functionality.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, ListBox, ListBoxRow, Orientation, Popover, Separator};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// WiFi network information
#[derive(Clone, Debug)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_strength: u8, // 0-100
    pub secured: bool,
    pub connected: bool,
}

/// WiFi control widget with toggle and network list
pub struct WifiWidget {
    container: Button,
    icon: Image,
    status: Label,
    enabled: Rc<Cell<bool>>,
    networks: Rc<std::cell::RefCell<Vec<WifiNetwork>>>,
}

impl WifiWidget {
    /// Create a new WiFi widget
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
        let icon = Image::from_icon_name("network-wireless-symbolic");
        icon.add_css_class("tile-icon");
        icon.set_pixel_size(24);
        icon.set_halign(gtk::Align::Start);
        content.append(&icon);

        // Label
        let label = Label::new(Some("Wi-Fi"));
        label.add_css_class("tile-label");
        label.set_halign(gtk::Align::Start);
        content.append(&label);

        // Status
        let status = Label::new(Some(if initial_state { "Connected" } else { "Off" }));
        status.add_css_class("tile-status");
        status.set_halign(gtk::Align::Start);
        content.append(&status);

        // Expand button for network list
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
        let networks = Rc::new(std::cell::RefCell::new(Self::get_mock_networks()));

        // Create popover for network list
        let popover = Self::create_network_popover(&networks.borrow());
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
                status_clone.set_text("Searching...");
                icon_clone.set_icon_name(Some("network-wireless-symbolic"));

                // Simulate finding networks
                let status = status_clone.clone();
                gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || {
                    status.set_text("Winux-Home");
                });
            } else {
                container_clone.remove_css_class("active");
                status_clone.set_text("Off");
                icon_clone.set_icon_name(Some("network-wireless-offline-symbolic"));
            }

            info!("WiFi toggled: {}", new_state);
        });

        Self {
            container,
            icon,
            status,
            enabled,
            networks,
        }
    }

    /// Create a popover with the network list
    fn create_network_popover(networks: &[WifiNetwork]) -> Popover {
        let popover = Popover::new();
        popover.add_css_class("network-list");

        let content = Box::new(Orientation::Vertical, 4);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);

        // Header
        let header = Box::new(Orientation::Horizontal, 8);
        let title = Label::new(Some("Wi-Fi Networks"));
        title.add_css_class("title-4");
        header.append(&title);
        content.append(&header);

        content.append(&Separator::new(Orientation::Horizontal));

        // Network list
        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .build();
        list.add_css_class("boxed-list");

        for network in networks {
            let row = Self::create_network_row(network);
            list.append(&row);
        }

        content.append(&list);

        // Settings button
        let settings_btn = Button::builder()
            .label("Wi-Fi Settings...")
            .build();
        settings_btn.add_css_class("flat");
        settings_btn.connect_clicked(|_| {
            info!("Opening WiFi settings");
            let _ = std::process::Command::new("gnome-control-center")
                .arg("wifi")
                .spawn();
        });
        content.append(&settings_btn);

        popover.set_child(Some(&content));
        popover
    }

    /// Create a row for a network in the list
    fn create_network_row(network: &WifiNetwork) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("network-item");
        if network.connected {
            row.add_css_class("connected");
        }

        let content = Box::new(Orientation::Horizontal, 12);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);

        // Signal strength icon
        let signal_icon = Self::signal_icon(network.signal_strength);
        let icon = Image::from_icon_name(&signal_icon);
        icon.add_css_class("signal-icon");
        if network.signal_strength > 60 {
            icon.add_css_class("strong");
        }
        content.append(&icon);

        // Network name and status
        let info_box = Box::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);

        let name = Label::new(Some(&network.ssid));
        name.add_css_class("network-name");
        name.set_halign(gtk::Align::Start);
        info_box.append(&name);

        if network.connected {
            let status = Label::new(Some("Connected"));
            status.add_css_class("network-status");
            status.set_halign(gtk::Align::Start);
            info_box.append(&status);
        }

        content.append(&info_box);

        // Lock icon if secured
        if network.secured {
            let lock = Image::from_icon_name("network-wireless-encrypted-symbolic");
            lock.set_opacity(0.5);
            content.append(&lock);
        }

        row.set_child(Some(&content));
        row
    }

    /// Get the signal strength icon name based on strength level
    fn signal_icon(strength: u8) -> String {
        match strength {
            0..=25 => "network-wireless-signal-weak-symbolic",
            26..=50 => "network-wireless-signal-ok-symbolic",
            51..=75 => "network-wireless-signal-good-symbolic",
            _ => "network-wireless-signal-excellent-symbolic",
        }
        .to_string()
    }

    /// Get mock network data for demonstration
    fn get_mock_networks() -> Vec<WifiNetwork> {
        vec![
            WifiNetwork {
                ssid: "Winux-Home".to_string(),
                signal_strength: 85,
                secured: true,
                connected: true,
            },
            WifiNetwork {
                ssid: "Neighbors-5G".to_string(),
                signal_strength: 60,
                secured: true,
                connected: false,
            },
            WifiNetwork {
                ssid: "CoffeeShop".to_string(),
                signal_strength: 45,
                secured: false,
                connected: false,
            },
            WifiNetwork {
                ssid: "Guest".to_string(),
                signal_strength: 30,
                secured: true,
                connected: false,
            },
        ]
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Button {
        &self.container
    }

    /// Check if WiFi is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Set the connection status
    pub fn set_connected(&self, ssid: Option<&str>) {
        if let Some(name) = ssid {
            self.status.set_text(name);
            self.container.add_css_class("active");
        } else {
            self.status.set_text("Not Connected");
        }
    }
}
