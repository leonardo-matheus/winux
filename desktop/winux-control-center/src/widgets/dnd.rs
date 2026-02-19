//! Do Not Disturb widget
//!
//! Provides a toggle for enabling/disabling notification interruptions.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, Orientation};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// Do Not Disturb control widget
pub struct DoNotDisturbWidget {
    container: Button,
    icon: Image,
    status: Label,
    enabled: Rc<Cell<bool>>,
}

impl DoNotDisturbWidget {
    /// Create a new Do Not Disturb widget
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
        let icon = Image::from_icon_name(Self::dnd_icon(initial_state));
        icon.add_css_class("tile-icon");
        icon.set_pixel_size(24);
        icon.set_halign(gtk::Align::Start);
        content.append(&icon);

        // Label
        let label = Label::new(Some("Do Not Disturb"));
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
        let icon_clone = icon.clone();

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

            icon_clone.set_icon_name(Some(Self::dnd_icon(new_state)));
            Self::set_dnd_mode(new_state);

            info!("Do Not Disturb toggled: {}", new_state);
        });

        Self {
            container,
            icon,
            status,
            enabled,
        }
    }

    /// Get the icon name based on DND state
    fn dnd_icon(enabled: bool) -> &'static str {
        if enabled {
            "notifications-disabled-symbolic"
        } else {
            "user-available-symbolic"
        }
    }

    /// Enable/disable DND mode in the system
    fn set_dnd_mode(enabled: bool) {
        // Use D-Bus to toggle notification daemon DND mode
        // For GNOME:
        let value = if enabled { "true" } else { "false" };
        let _ = std::process::Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.notifications",
                "show-banners",
                if enabled { "false" } else { "true" },
            ])
            .spawn();

        info!("System DND mode set to: {}", enabled);
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Button {
        &self.container
    }

    /// Check if DND is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Set DND state programmatically
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.set(enabled);
        if enabled {
            self.container.add_css_class("active");
            self.status.set_text("On");
        } else {
            self.container.remove_css_class("active");
            self.status.set_text("Off");
        }
        self.icon.set_icon_name(Some(Self::dnd_icon(enabled)));
    }
}
