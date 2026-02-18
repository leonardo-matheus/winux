//! System Tray module
//!
//! Implements the StatusNotifierItem (SNI) protocol for system tray icons.
//! This provides a modern D-Bus based tray similar to KDE's implementation.

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Box as GtkBox, Button, Image, MenuButton, Orientation, Popover};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use zbus::Connection;

/// Status Notifier Item representation
#[derive(Debug, Clone)]
pub struct StatusNotifierItem {
    /// Unique bus name
    pub bus_name: String,
    /// Object path
    pub object_path: String,
    /// Item ID
    pub id: String,
    /// Category (ApplicationStatus, Communications, SystemServices, Hardware)
    pub category: String,
    /// Status (Passive, Active, NeedsAttention)
    pub status: String,
    /// Icon name
    pub icon_name: Option<String>,
    /// Icon pixmap data (ARGB)
    pub icon_pixmap: Option<Vec<u8>>,
    /// Attention icon name
    pub attention_icon_name: Option<String>,
    /// Title
    pub title: Option<String>,
    /// Tooltip
    pub tooltip: Option<String>,
    /// Has menu
    pub has_menu: bool,
}

/// System tray widget
pub struct SystemTray {
    /// Main container
    container: GtkBox,
    /// Icon buttons by ID
    icons: Rc<RefCell<HashMap<String, Button>>>,
    /// Status notifier items
    items: Rc<RefCell<HashMap<String, StatusNotifierItem>>>,
    /// Quick settings button
    quick_settings_button: MenuButton,
    /// Network indicator
    network_button: Button,
    /// Volume indicator
    volume_button: Button,
    /// Battery indicator
    battery_button: Button,
}

impl SystemTray {
    /// Create a new system tray
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        container.add_css_class("system-tray");

        // Third-party tray icons area
        let icons_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .build();

        // System indicators
        let network_button = Self::create_indicator_button("network-wireless-symbolic", "Network");
        let volume_button = Self::create_indicator_button("audio-volume-high-symbolic", "Volume");
        let battery_button = Self::create_indicator_button("battery-good-symbolic", "Battery");

        // Quick settings button (combines all quick settings)
        let quick_settings_button = Self::create_quick_settings_button();

        // Build layout
        container.append(&icons_box);
        container.append(&network_button);
        container.append(&volume_button);
        container.append(&battery_button);
        container.append(&quick_settings_button);

        let tray = Self {
            container,
            icons: Rc::new(RefCell::new(HashMap::new())),
            items: Rc::new(RefCell::new(HashMap::new())),
            quick_settings_button,
            network_button,
            volume_button,
            battery_button,
        };

        // Setup D-Bus watcher for StatusNotifierItems
        tray.setup_sni_watcher();

        // Setup system indicators
        tray.setup_system_indicators();

        tray
    }

    /// Create an indicator button
    fn create_indicator_button(icon_name: &str, tooltip: &str) -> Button {
        let button = Button::builder()
            .tooltip_text(tooltip)
            .build();

        button.add_css_class("system-tray-icon");

        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(20);
        button.set_child(Some(&icon));

        button
    }

    /// Create the quick settings button with popover
    fn create_quick_settings_button() -> MenuButton {
        let button = MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("Quick Settings")
            .build();

        button.add_css_class("system-tray-icon");

        // Create quick settings popover
        let popover = Popover::new();
        let content = Self::create_quick_settings_content();
        popover.set_child(Some(&content));

        button.set_popover(Some(&popover));

        button
    }

    /// Create quick settings popover content
    fn create_quick_settings_content() -> GtkBox {
        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .width_request(300)
            .build();

        // Toggle buttons row
        let toggles_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .homogeneous(true)
            .build();

        let wifi_toggle = Self::create_quick_toggle("network-wireless-symbolic", "Wi-Fi", true);
        let bluetooth_toggle = Self::create_quick_toggle("bluetooth-active-symbolic", "Bluetooth", false);
        let dnd_toggle = Self::create_quick_toggle("notifications-disabled-symbolic", "Do Not Disturb", false);
        let night_toggle = Self::create_quick_toggle("night-light-symbolic", "Night Light", false);

        toggles_row.append(&wifi_toggle);
        toggles_row.append(&bluetooth_toggle);
        toggles_row.append(&dnd_toggle);
        toggles_row.append(&night_toggle);

        // Volume slider
        let volume_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let volume_icon = Image::from_icon_name("audio-volume-high-symbolic");
        volume_icon.set_pixel_size(20);

        let volume_scale = gtk4::Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 5.0);
        volume_scale.set_hexpand(true);
        volume_scale.set_value(75.0);

        volume_box.append(&volume_icon);
        volume_box.append(&volume_scale);

        // Brightness slider
        let brightness_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let brightness_icon = Image::from_icon_name("display-brightness-symbolic");
        brightness_icon.set_pixel_size(20);

        let brightness_scale = gtk4::Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 5.0);
        brightness_scale.set_hexpand(true);
        brightness_scale.set_value(80.0);

        brightness_box.append(&brightness_icon);
        brightness_box.append(&brightness_scale);

        // Settings button
        let settings_button = Button::builder()
            .label("All Settings")
            .margin_top(8)
            .build();

        settings_button.connect_clicked(|_| {
            // Launch settings app
            if let Some(app_info) = gio::DesktopAppInfo::new("org.winux.Settings.desktop") {
                let _ = app_info.launch(&[], gio::AppLaunchContext::NONE);
            }
        });

        content.append(&toggles_row);
        content.append(&gtk4::Separator::new(Orientation::Horizontal));
        content.append(&volume_box);
        content.append(&brightness_box);
        content.append(&settings_button);

        content
    }

    /// Create a quick toggle button
    fn create_quick_toggle(icon_name: &str, tooltip: &str, active: bool) -> gtk4::ToggleButton {
        let button = gtk4::ToggleButton::builder()
            .tooltip_text(tooltip)
            .active(active)
            .build();

        button.add_css_class("circular");

        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(20);
        button.set_child(Some(&icon));

        button
    }

    /// Setup StatusNotifierItem D-Bus watcher
    fn setup_sni_watcher(&self) {
        let items = Rc::clone(&self.items);
        let icons = Rc::clone(&self.icons);

        // Spawn async task to watch for SNI items
        glib::spawn_future_local(async move {
            if let Err(e) = Self::watch_sni_items(items, icons).await {
                error!("Failed to setup SNI watcher: {}", e);
            }
        });
    }

    /// Watch for StatusNotifierItem registrations
    async fn watch_sni_items(
        items: Rc<RefCell<HashMap<String, StatusNotifierItem>>>,
        icons: Rc<RefCell<HashMap<String, Button>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Connect to session bus
        let connection = Connection::session().await?;

        // TODO: Implement full SNI watcher
        // This would involve:
        // 1. Registering as StatusNotifierWatcher
        // 2. Listening for RegisterStatusNotifierItem signals
        // 3. Getting item properties via D-Bus
        // 4. Creating/updating icons

        debug!("SNI watcher initialized");

        Ok(())
    }

    /// Setup system indicators (battery, network, volume)
    fn setup_system_indicators(&self) {
        // Setup battery monitoring
        self.setup_battery_indicator();

        // Setup network monitoring
        self.setup_network_indicator();

        // Setup volume control
        self.setup_volume_indicator();
    }

    /// Setup battery indicator
    fn setup_battery_indicator(&self) {
        let button = self.battery_button.clone();

        // Use UPower D-Bus interface for battery info
        glib::spawn_future_local(async move {
            loop {
                // Update battery status
                if let Some((percentage, charging)) = Self::get_battery_status().await {
                    let icon_name = match (percentage, charging) {
                        (_, true) => "battery-good-charging-symbolic",
                        (p, _) if p > 80 => "battery-full-symbolic",
                        (p, _) if p > 50 => "battery-good-symbolic",
                        (p, _) if p > 20 => "battery-low-symbolic",
                        _ => "battery-caution-symbolic",
                    };

                    glib::idle_add_local_once(move || {
                        if let Some(image) = button.child().and_then(|c| c.downcast::<Image>().ok()) {
                            image.set_from_icon_name(Some(icon_name));
                        }
                        button.set_tooltip_text(Some(&format!("Battery: {}%", percentage)));
                    });
                }

                // Update every 30 seconds
                glib::timeout_future_seconds(30).await;
            }
        });
    }

    /// Get battery status from UPower
    async fn get_battery_status() -> Option<(u8, bool)> {
        // TODO: Implement UPower D-Bus query
        // For now, return placeholder
        Some((75, false))
    }

    /// Setup network indicator
    fn setup_network_indicator(&self) {
        let button = self.network_button.clone();

        button.connect_clicked(|_| {
            // TODO: Show network popover with connection options
            debug!("Network button clicked");
        });

        // Monitor NetworkManager for connection changes
        glib::spawn_future_local(async move {
            loop {
                if let Some((connected, wifi_strength)) = Self::get_network_status().await {
                    let icon_name = if connected {
                        match wifi_strength {
                            s if s > 75 => "network-wireless-signal-excellent-symbolic",
                            s if s > 50 => "network-wireless-signal-good-symbolic",
                            s if s > 25 => "network-wireless-signal-ok-symbolic",
                            _ => "network-wireless-signal-weak-symbolic",
                        }
                    } else {
                        "network-wireless-offline-symbolic"
                    };

                    glib::idle_add_local_once(move || {
                        if let Some(image) = button.child().and_then(|c| c.downcast::<Image>().ok()) {
                            image.set_from_icon_name(Some(icon_name));
                        }
                    });
                }

                glib::timeout_future_seconds(5).await;
            }
        });
    }

    /// Get network status from NetworkManager
    async fn get_network_status() -> Option<(bool, u8)> {
        // TODO: Implement NetworkManager D-Bus query
        Some((true, 80))
    }

    /// Setup volume indicator
    fn setup_volume_indicator(&self) {
        let button = self.volume_button.clone();

        button.connect_clicked(|_| {
            // TODO: Show volume popover with slider
            debug!("Volume button clicked");
        });

        // Monitor PulseAudio/PipeWire for volume changes
        glib::spawn_future_local(async move {
            loop {
                if let Some((volume, muted)) = Self::get_volume_status().await {
                    let icon_name = if muted {
                        "audio-volume-muted-symbolic"
                    } else {
                        match volume {
                            v if v > 66 => "audio-volume-high-symbolic",
                            v if v > 33 => "audio-volume-medium-symbolic",
                            v if v > 0 => "audio-volume-low-symbolic",
                            _ => "audio-volume-muted-symbolic",
                        }
                    };

                    glib::idle_add_local_once(move || {
                        if let Some(image) = button.child().and_then(|c| c.downcast::<Image>().ok()) {
                            image.set_from_icon_name(Some(icon_name));
                        }
                    });
                }

                glib::timeout_future_seconds(2).await;
            }
        });
    }

    /// Get volume status from PulseAudio/PipeWire
    async fn get_volume_status() -> Option<(u8, bool)> {
        // TODO: Implement PulseAudio/PipeWire query
        Some((75, false))
    }

    /// Add a status notifier item
    pub fn add_item(&mut self, item: StatusNotifierItem) {
        let id = item.id.clone();

        // Create button for the item
        let button = Button::builder()
            .tooltip_text(item.title.as_deref().unwrap_or(&item.id))
            .build();

        button.add_css_class("system-tray-icon");

        // Set icon
        let icon = if let Some(icon_name) = &item.icon_name {
            Image::from_icon_name(icon_name)
        } else {
            Image::from_icon_name("application-x-executable")
        };

        icon.set_pixel_size(20);
        button.set_child(Some(&icon));

        // Handle click
        let bus_name = item.bus_name.clone();
        let object_path = item.object_path.clone();

        button.connect_clicked(move |_| {
            debug!("Tray icon clicked: {}", bus_name);
            // TODO: Call Activate method on the item
        });

        // Add to container (before system indicators)
        // We need to insert at the beginning
        self.container.prepend(&button);

        // Store references
        self.items.borrow_mut().insert(id.clone(), item);
        self.icons.borrow_mut().insert(id, button);
    }

    /// Remove a status notifier item
    pub fn remove_item(&mut self, id: &str) {
        if let Some(button) = self.icons.borrow_mut().remove(id) {
            self.container.remove(&button);
        }
        self.items.borrow_mut().remove(id);
    }

    /// Update a status notifier item
    pub fn update_item(&mut self, id: &str, item: StatusNotifierItem) {
        if let Some(button) = self.icons.borrow().get(id) {
            // Update icon
            if let Some(icon_name) = &item.icon_name {
                if let Some(image) = button.child().and_then(|c| c.downcast::<Image>().ok()) {
                    image.set_from_icon_name(Some(icon_name));
                }
            }

            // Update tooltip
            button.set_tooltip_text(item.title.as_deref());
        }

        self.items.borrow_mut().insert(id.to_string(), item);
    }

    /// Get the main widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }
}

impl Default for SystemTray {
    fn default() -> Self {
        Self::new()
    }
}
