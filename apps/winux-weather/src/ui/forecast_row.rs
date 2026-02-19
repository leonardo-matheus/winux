//! Forecast row widget

use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::data::HourlyForecast;

/// A row displaying hourly forecast
pub struct ForecastRow {
    container: Box,
}

impl ForecastRow {
    /// Create a new forecast row for hourly data
    pub fn new_hourly(forecast: &HourlyForecast) -> Self {
        let container = Box::new(Orientation::Vertical, 4);
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.add_css_class("forecast-row");

        // Time
        let local_time = forecast.time.with_timezone(&chrono::Local);
        let time_label = Label::new(Some(&format!("{:02}h", local_time.hour())));
        time_label.add_css_class("caption");
        time_label.add_css_class("dim-label");

        // Icon
        let icon = Image::from_icon_name(forecast.condition.icon_name(forecast.is_day));
        icon.set_pixel_size(32);

        // Temperature
        let temp_label = Label::new(Some(&format!("{:.0}°", forecast.temperature)));
        temp_label.add_css_class("heading");

        container.append(&time_label);
        container.append(&icon);
        container.append(&temp_label);

        // Precipitation probability if > 0
        if forecast.precipitation_probability > 0 {
            let precip_label = Label::new(Some(&format!("{}%", forecast.precipitation_probability)));
            precip_label.add_css_class("caption");
            precip_label.add_css_class("accent");
            container.append(&precip_label);
        }

        Self { container }
    }

    /// Get the widget
    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }
}

use chrono::Timelike;
