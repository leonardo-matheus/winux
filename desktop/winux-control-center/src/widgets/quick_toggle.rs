//! Generic quick toggle widget
//!
//! A reusable toggle tile component for the control center.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, Orientation};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// A generic quick toggle widget with icon, label, and status
pub struct QuickToggleWidget {
    container: Button,
    icon: Image,
    label: Label,
    status: Label,
    active: Rc<Cell<bool>>,
}

impl QuickToggleWidget {
    /// Create a new quick toggle widget
    pub fn new(icon_name: &str, label_text: &str, status_text: &str, initial_state: bool) -> Self {
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
        let icon = Image::from_icon_name(icon_name);
        icon.add_css_class("tile-icon");
        icon.set_pixel_size(24);
        icon.set_halign(gtk::Align::Start);
        content.append(&icon);

        // Label
        let label = Label::new(Some(label_text));
        label.add_css_class("tile-label");
        label.set_halign(gtk::Align::Start);
        content.append(&label);

        // Status
        let status = Label::new(Some(status_text));
        status.add_css_class("tile-status");
        status.set_halign(gtk::Align::Start);
        content.append(&status);

        container.set_child(Some(&content));

        let active = Rc::new(Cell::new(initial_state));

        // Connect click handler
        let active_clone = active.clone();
        let container_clone = container.clone();
        let status_clone = status.clone();
        let label_text = label_text.to_string();

        container.connect_clicked(move |_| {
            let new_state = !active_clone.get();
            active_clone.set(new_state);

            if new_state {
                container_clone.add_css_class("active");
                status_clone.set_text("On");
            } else {
                container_clone.remove_css_class("active");
                status_clone.set_text("Off");
            }

            info!("{} toggled: {}", label_text, new_state);
        });

        Self {
            container,
            icon,
            label,
            status,
            active,
        }
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Button {
        &self.container
    }

    /// Get the current state
    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    /// Set the state programmatically
    pub fn set_active(&self, active: bool) {
        self.active.set(active);
        if active {
            self.container.add_css_class("active");
            self.status.set_text("On");
        } else {
            self.container.remove_css_class("active");
            self.status.set_text("Off");
        }
    }

    /// Update the status text
    pub fn set_status(&self, text: &str) {
        self.status.set_text(text);
    }

    /// Update the icon
    pub fn set_icon(&self, icon_name: &str) {
        self.icon.set_icon_name(Some(icon_name));
    }
}
