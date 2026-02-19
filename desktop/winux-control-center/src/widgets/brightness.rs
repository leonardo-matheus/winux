//! Brightness control widget
//!
//! Provides a slider for controlling screen brightness.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Adjustment, Box, Image, Label, Orientation, Scale};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

/// Brightness control widget with slider
pub struct BrightnessWidget {
    container: Box,
    slider: Scale,
    icon: Image,
    level: Rc<Cell<u32>>,
}

impl BrightnessWidget {
    /// Create a new brightness widget
    pub fn new(initial_level: u32) -> Self {
        let container = Box::new(Orientation::Vertical, 8);
        container.add_css_class("slider-tile");

        // Header with icon and label
        let header = Box::new(Orientation::Horizontal, 8);

        let icon = Image::from_icon_name(Self::brightness_icon(initial_level));
        icon.add_css_class("slider-icon");
        icon.set_pixel_size(20);
        header.append(&icon);

        let label = Label::new(Some("Brightness"));
        label.add_css_class("slider-label");
        label.set_hexpand(true);
        label.set_halign(gtk::Align::Start);
        header.append(&label);

        // Percentage label
        let percentage = Label::new(Some(&format!("{}%", initial_level)));
        percentage.add_css_class("slider-label");
        header.append(&percentage);

        container.append(&header);

        // Slider
        let adjustment = Adjustment::new(
            initial_level as f64, // value
            0.0,                  // lower
            100.0,                // upper
            1.0,                  // step increment
            10.0,                 // page increment
            0.0,                  // page size
        );

        let slider = Scale::new(Orientation::Horizontal, Some(&adjustment));
        slider.add_css_class("control-slider");
        slider.set_draw_value(false);
        slider.set_hexpand(true);

        container.append(&slider);

        let level = Rc::new(Cell::new(initial_level));

        // Connect value change handler
        let level_clone = level.clone();
        let icon_clone = icon.clone();
        let percentage_clone = percentage.clone();

        slider.connect_value_changed(move |scale| {
            let value = scale.value() as u32;
            level_clone.set(value);
            percentage_clone.set_text(&format!("{}%", value));
            icon_clone.set_icon_name(Some(Self::brightness_icon(value)));

            // Here we would actually set the brightness
            Self::set_system_brightness(value);
        });

        Self {
            container,
            slider,
            icon,
            level,
        }
    }

    /// Get the icon name based on brightness level
    fn brightness_icon(level: u32) -> &'static str {
        match level {
            0..=25 => "display-brightness-low-symbolic",
            26..=75 => "display-brightness-medium-symbolic",
            _ => "display-brightness-symbolic",
        }
    }

    /// Set the system brightness (platform-specific implementation)
    fn set_system_brightness(level: u32) {
        info!("Setting brightness to {}%", level);

        // Try using brightnessctl on Linux
        let _ = std::process::Command::new("brightnessctl")
            .arg("set")
            .arg(format!("{}%", level))
            .spawn();

        // Alternative: try xbacklight
        // let _ = std::process::Command::new("xbacklight")
        //     .arg("-set")
        //     .arg(format!("{}", level))
        //     .spawn();
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Get the current brightness level
    pub fn level(&self) -> u32 {
        self.level.get()
    }

    /// Set the brightness level programmatically
    pub fn set_level(&self, level: u32) {
        let clamped = level.min(100);
        self.level.set(clamped);
        self.slider.set_value(clamped as f64);
    }
}
