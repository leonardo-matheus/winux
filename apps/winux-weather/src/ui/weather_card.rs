//! Weather card widget

use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::data::{WeatherCondition, DailyForecast};

/// A card displaying weather information
pub struct WeatherCard {
    container: Box,
}

impl WeatherCard {
    /// Create a new weather card for a daily forecast
    pub fn new_daily(forecast: &DailyForecast) -> Self {
        let container = Box::new(Orientation::Vertical, 8);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.add_css_class("card");
        container.add_css_class("weather-card");
        container.set_size_request(100, -1);

        // Day name
        let day_label = Label::new(Some(&forecast.day_name()));
        day_label.add_css_class("heading");

        // Date
        let date_label = Label::new(Some(&forecast.formatted_date()));
        date_label.add_css_class("caption");
        date_label.add_css_class("dim-label");

        // Weather icon
        let icon = Image::from_icon_name(forecast.condition.icon_name(true));
        icon.set_pixel_size(48);
        icon.set_margin_top(8);
        icon.set_margin_bottom(8);

        // Temperature range
        let temp_box = Box::new(Orientation::Horizontal, 4);
        temp_box.set_halign(gtk4::Align::Center);

        let max_label = Label::new(Some(&format!("{:.0}°", forecast.temperature_max)));
        max_label.add_css_class("heading");

        let min_label = Label::new(Some(&format!("{:.0}°", forecast.temperature_min)));
        min_label.add_css_class("caption");
        min_label.add_css_class("dim-label");

        temp_box.append(&max_label);
        temp_box.append(&min_label);

        // Precipitation probability
        if forecast.precipitation_probability_max > 0 {
            let precip_box = Box::new(Orientation::Horizontal, 4);
            precip_box.set_halign(gtk4::Align::Center);
            precip_box.set_margin_top(4);

            let drop_icon = Image::from_icon_name("weather-showers-symbolic");
            drop_icon.set_pixel_size(12);

            let precip_label = Label::new(Some(&format!("{}%", forecast.precipitation_probability_max)));
            precip_label.add_css_class("caption");
            precip_label.add_css_class("accent");

            precip_box.append(&drop_icon);
            precip_box.append(&precip_label);

            container.append(&day_label);
            container.append(&date_label);
            container.append(&icon);
            container.append(&temp_box);
            container.append(&precip_box);
        } else {
            container.append(&day_label);
            container.append(&date_label);
            container.append(&icon);
            container.append(&temp_box);
        }

        Self { container }
    }

    /// Create a compact weather card
    pub fn new_compact(
        title: &str,
        value: &str,
        icon_name: &str,
        subtitle: Option<&str>,
    ) -> Self {
        let container = Box::new(Orientation::Vertical, 4);
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.set_margin_top(8);
        container.set_margin_bottom(8);
        container.add_css_class("card");
        container.add_css_class("weather-card-compact");
        container.set_size_request(80, -1);

        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(24);

        let title_label = Label::new(Some(title));
        title_label.add_css_class("caption");
        title_label.add_css_class("dim-label");

        let value_label = Label::new(Some(value));
        value_label.add_css_class("heading");

        container.append(&icon);
        container.append(&title_label);
        container.append(&value_label);

        if let Some(sub) = subtitle {
            let sub_label = Label::new(Some(sub));
            sub_label.add_css_class("caption");
            sub_label.add_css_class("dim-label");
            container.append(&sub_label);
        }

        Self { container }
    }

    /// Get the widget
    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }
}
