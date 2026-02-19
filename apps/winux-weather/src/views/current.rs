//! Current weather view

use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::api::Location;
use crate::data::WeatherData;

/// Current weather view widget
pub struct CurrentWeatherView {
    container: Box,
    location_label: Label,
    temperature_label: Label,
    condition_icon: Image,
    condition_label: Label,
    feels_like_label: Label,
    high_low_label: Label,
}

impl CurrentWeatherView {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 8);
        container.set_valign(gtk4::Align::Center);
        container.set_halign(gtk4::Align::Center);
        container.set_margin_top(32);
        container.set_margin_bottom(32);
        container.set_margin_start(24);
        container.set_margin_end(24);

        // Location name
        let location_label = Label::new(Some("Carregando..."));
        location_label.add_css_class("title-2");
        location_label.set_margin_bottom(8);

        // Weather icon (large)
        let condition_icon = Image::from_icon_name("weather-clear-symbolic");
        condition_icon.set_pixel_size(128);
        condition_icon.add_css_class("weather-icon-large");
        condition_icon.set_margin_top(16);
        condition_icon.set_margin_bottom(8);

        // Temperature (very large)
        let temperature_label = Label::new(Some("--"));
        temperature_label.add_css_class("temperature-large");
        temperature_label.set_margin_bottom(4);

        // Condition description
        let condition_label = Label::new(Some("--"));
        condition_label.add_css_class("title-3");
        condition_label.set_margin_bottom(8);

        // Feels like
        let feels_like_label = Label::new(Some("Sensacao termica: --"));
        feels_like_label.add_css_class("dim-label");

        // High/Low for today
        let high_low_label = Label::new(Some("Max: -- / Min: --"));
        high_low_label.add_css_class("dim-label");

        container.append(&location_label);
        container.append(&condition_icon);
        container.append(&temperature_label);
        container.append(&condition_label);
        container.append(&feels_like_label);
        container.append(&high_low_label);

        Self {
            container,
            location_label,
            temperature_label,
            condition_icon,
            condition_label,
            feels_like_label,
            high_low_label,
        }
    }

    /// Get the widget to display
    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    /// Update the view with new weather data
    pub fn update(&self, data: &WeatherData, location: &Location) {
        // Update location
        self.location_label.set_text(&format!("{}, {}", location.name, location.country));

        // Update temperature
        self.temperature_label.set_text(&format!("{:.0}°", data.current.temperature));

        // Update icon
        let icon_name = data.current.condition.icon_name(data.current.is_day);
        self.condition_icon.set_from_icon_name(Some(icon_name));

        // Update condition
        self.condition_label.set_text(data.current.condition.description());

        // Update feels like
        self.feels_like_label.set_text(&format!(
            "Sensacao termica: {:.0}°",
            data.current.apparent_temperature
        ));

        // Update high/low from today's daily forecast
        if let Some(today) = data.daily.first() {
            self.high_low_label.set_text(&format!(
                "Max: {:.0}° / Min: {:.0}°",
                today.temperature_max,
                today.temperature_min
            ));
        }
    }
}
