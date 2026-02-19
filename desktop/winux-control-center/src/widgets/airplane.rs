//! Airplane Mode widget
//!
//! Provides a toggle for enabling/disabling all wireless connections.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, Orientation};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// Airplane Mode control widget
pub struct AirplaneModeWidget {
    container: Button,
    icon: Image,
    status: Label,
    enabled: Rc<Cell<bool>>,
}

impl AirplaneModeWidget {
    /// Create a new Airplane Mode widget
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
        let icon = Image::from_icon_name("airplane-mode-symbolic");
        icon.add_css_class("tile-icon");
        icon.set_pixel_size(24);
        icon.set_halign(gtk::Align::Start);
        content.append(&icon);

        // Label
        let label = Label::new(Some("Airplane Mode"));
        label.add_css_class("tile-label");
        label.set_halign(gtk::Align::Start);
        content.append(&label);

        // Status
        let status = Label::new(Some(if initial_state { "On" } else { "Off" }));
        status.add_css_class("tile-status");
        status.set_halign(gtk::Align::Start);
        content.append(&status);

        container.set_child(Some(&content));

        let enabled = Rc::new(Cell::new(initial_state));

        // Connect click handler
        let enabled_clone = enabled.clone();
        let container_clone = container.clone();
        let status_clone = status.clone();

        container.connect_clicked(move |_| {
            let new_state = !enabled_clone.get();
            enabled_clone.set(new_state);

            if new_state {
                container_clone.add_css_class("active");
                status_clone.set_text("On");
            } else {
                container_clone.remove_css_class("active");
                status_clone.set_text("Off");
            }

            Self::set_airplane_mode(new_state);
            info!("Airplane Mode toggled: {}", new_state);
        });

        Self {
            container,
            icon,
            status,
            enabled,
        }
    }

    /// Enable/disable airplane mode in the system
    fn set_airplane_mode(enabled: bool) {
        // Use rfkill to toggle all wireless devices
        let action = if enabled { "block" } else { "unblock" };

        // Block/unblock all wireless
        let _ = std::process::Command::new("rfkill")
            .args([action, "all"])
            .spawn();

        // Also try NetworkManager
        let nm_state = if enabled { "off" } else { "on" };
        let _ = std::process::Command::new("nmcli")
            .args(["radio", "all", nm_state])
            .spawn();

        info!("Airplane mode set to: {}", enabled);
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Button {
        &self.container
    }

    /// Check if airplane mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Set airplane mode state programmatically
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.set(enabled);
        if enabled {
            self.container.add_css_class("active");
            self.status.set_text("On");
        } else {
            self.container.remove_css_class("active");
            self.status.set_text("Off");
        }
    }
}
