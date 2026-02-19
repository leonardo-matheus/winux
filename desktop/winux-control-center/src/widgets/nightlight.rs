//! Night Light widget
//!
//! Provides a toggle for enabling blue light filter / night shift mode.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Adjustment, Box, Button, Image, Label, Orientation, Popover, Scale, Separator};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// Night Light control widget
pub struct NightLightWidget {
    container: Button,
    icon: Image,
    status: Label,
    enabled: Rc<Cell<bool>>,
    temperature: Rc<Cell<u32>>,
}

impl NightLightWidget {
    /// Create a new Night Light widget
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
        let icon = Image::from_icon_name(Self::night_light_icon(initial_state));
        icon.add_css_class("tile-icon");
        icon.set_pixel_size(24);
        icon.set_halign(gtk::Align::Start);
        content.append(&icon);

        // Label
        let label = Label::new(Some("Night Light"));
        label.add_css_class("tile-label");
        label.set_halign(gtk::Align::Start);
        content.append(&label);

        // Status
        let status = Label::new(Some(if initial_state { "On" } else { "Off" }));
        status.add_css_class("tile-status");
        status.set_halign(gtk::Align::Start);
        content.append(&status);

        // Expand button for settings
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
        let temperature = Rc::new(Cell::new(4500u32)); // Default color temperature

        // Create settings popover
        let temperature_clone = temperature.clone();
        let popover = Self::create_settings_popover(&temperature_clone);
        popover.set_parent(&expand_btn);

        let popover_clone = popover.clone();
        expand_btn.connect_clicked(move |_| {
            popover_clone.popup();
        });

        // Connect main click handler for toggle
        let enabled_clone = enabled.clone();
        let container_clone = container.clone();
        let status_clone = status.clone();
        let icon_clone = icon.clone();
        let temperature_clone2 = temperature.clone();

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

            icon_clone.set_icon_name(Some(Self::night_light_icon(new_state)));
            Self::set_night_light(new_state, temperature_clone2.get());

            info!("Night Light toggled: {}", new_state);
        });

        Self {
            container,
            icon,
            status,
            enabled,
            temperature,
        }
    }

    /// Create settings popover for night light configuration
    fn create_settings_popover(temperature: &Rc<Cell<u32>>) -> Popover {
        let popover = Popover::new();
        popover.add_css_class("network-list");

        let content = Box::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_width_request(260);

        // Header
        let title = Label::new(Some("Night Light"));
        title.add_css_class("title-4");
        title.set_halign(gtk::Align::Start);
        content.append(&title);

        let description = Label::new(Some("Reduce blue light to help you sleep better"));
        description.add_css_class("dim-label");
        description.add_css_class("body");
        description.set_halign(gtk::Align::Start);
        description.set_wrap(true);
        description.set_xalign(0.0);
        content.append(&description);

        content.append(&Separator::new(Orientation::Horizontal));

        // Color temperature slider
        let temp_box = Box::new(Orientation::Vertical, 4);

        let temp_header = Box::new(Orientation::Horizontal, 8);
        let temp_label = Label::new(Some("Color Temperature"));
        temp_label.set_hexpand(true);
        temp_label.set_halign(gtk::Align::Start);
        temp_header.append(&temp_label);

        let temp_value = Label::new(Some(&format!("{}K", temperature.get())));
        temp_value.add_css_class("dim-label");
        temp_header.append(&temp_value);

        temp_box.append(&temp_header);

        // Temperature slider (1000K - 10000K, lower = warmer/more orange)
        let adjustment = Adjustment::new(
            temperature.get() as f64,
            1000.0,  // Very warm
            6500.0,  // Normal daylight
            100.0,
            500.0,
            0.0,
        );

        let slider = Scale::new(Orientation::Horizontal, Some(&adjustment));
        slider.add_css_class("control-slider");
        slider.set_draw_value(false);

        // Add marks for reference
        slider.add_mark(1000.0, gtk::PositionType::Bottom, Some("Warm"));
        slider.add_mark(6500.0, gtk::PositionType::Bottom, Some("Cool"));

        let temp_value_clone = temp_value.clone();
        let temp_clone = temperature.clone();

        slider.connect_value_changed(move |scale| {
            let value = scale.value() as u32;
            temp_clone.set(value);
            temp_value_clone.set_text(&format!("{}K", value));
            Self::set_night_light(true, value);
        });

        temp_box.append(&slider);
        content.append(&temp_box);

        content.append(&Separator::new(Orientation::Horizontal));

        // Schedule option
        let schedule_box = Box::new(Orientation::Horizontal, 8);

        let schedule_label = Label::new(Some("Sunset to Sunrise"));
        schedule_label.set_hexpand(true);
        schedule_label.set_halign(gtk::Align::Start);
        schedule_box.append(&schedule_label);

        let schedule_switch = gtk::Switch::new();
        schedule_switch.set_active(false);
        schedule_switch.connect_state_set(|_, state| {
            info!("Night Light schedule: {}", state);
            gtk::glib::Propagation::Proceed
        });
        schedule_box.append(&schedule_switch);

        content.append(&schedule_box);

        // Settings button
        let settings_btn = Button::builder()
            .label("Display Settings...")
            .build();
        settings_btn.add_css_class("flat");
        settings_btn.set_margin_top(8);
        settings_btn.connect_clicked(|_| {
            info!("Opening display settings");
            let _ = std::process::Command::new("gnome-control-center")
                .arg("display")
                .spawn();
        });
        content.append(&settings_btn);

        popover.set_child(Some(&content));
        popover
    }

    /// Get the icon name based on night light state
    fn night_light_icon(enabled: bool) -> &'static str {
        if enabled {
            "night-light-symbolic"
        } else {
            "display-brightness-symbolic"
        }
    }

    /// Enable/disable night light in the system
    fn set_night_light(enabled: bool, temperature: u32) {
        if enabled {
            // Use gammastep, redshift, or GNOME Night Light
            // Try GNOME first
            let _ = std::process::Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.settings-daemon.plugins.color",
                    "night-light-enabled",
                    "true",
                ])
                .spawn();

            let _ = std::process::Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.settings-daemon.plugins.color",
                    "night-light-temperature",
                    &temperature.to_string(),
                ])
                .spawn();

            info!("Night Light enabled with temperature {}K", temperature);
        } else {
            let _ = std::process::Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.settings-daemon.plugins.color",
                    "night-light-enabled",
                    "false",
                ])
                .spawn();

            info!("Night Light disabled");
        }
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Button {
        &self.container
    }

    /// Check if night light is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Get the current color temperature
    pub fn temperature(&self) -> u32 {
        self.temperature.get()
    }

    /// Set night light state programmatically
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.set(enabled);
        if enabled {
            self.container.add_css_class("active");
            self.status.set_text("On");
        } else {
            self.container.remove_css_class("active");
            self.status.set_text("Off");
        }
        self.icon.set_icon_name(Some(Self::night_light_icon(enabled)));
    }
}
