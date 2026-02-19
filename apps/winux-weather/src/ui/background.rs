//! Dynamic weather background

use gtk4::prelude::*;
use gtk4::{Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::data::{WeatherData, WeatherCondition};

/// Dynamic background that changes based on weather and time
pub struct WeatherBackground {
    container: Box,
}

impl WeatherBackground {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);
        container.add_css_class("weather-background");
        container.add_css_class("weather-bg-day-clear");

        Self { container }
    }

    /// Get the widget
    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    /// Update the background based on weather
    pub fn update(&self, data: &WeatherData) {
        // Remove all previous background classes
        let bg_classes = [
            "weather-bg-day-clear",
            "weather-bg-night-clear",
            "weather-bg-day-cloudy",
            "weather-bg-night-cloudy",
            "weather-bg-day-rainy",
            "weather-bg-night-rainy",
            "weather-bg-day-stormy",
            "weather-bg-night-stormy",
            "weather-bg-day-snowy",
            "weather-bg-night-snowy",
            "weather-bg-day-foggy",
            "weather-bg-night-foggy",
        ];

        for class in bg_classes {
            self.container.remove_css_class(class);
        }

        // Determine the appropriate background class
        let is_day = data.current.is_day;
        let condition = &data.current.condition;

        let bg_class = match condition {
            WeatherCondition::ClearSky | WeatherCondition::MainlyClear => {
                if is_day { "weather-bg-day-clear" }
                else { "weather-bg-night-clear" }
            }
            WeatherCondition::PartlyCloudy | WeatherCondition::Overcast => {
                if is_day { "weather-bg-day-cloudy" }
                else { "weather-bg-night-cloudy" }
            }
            WeatherCondition::Fog | WeatherCondition::DepositingRimeFog => {
                if is_day { "weather-bg-day-foggy" }
                else { "weather-bg-night-foggy" }
            }
            c if c.is_rainy() => {
                if is_day { "weather-bg-day-rainy" }
                else { "weather-bg-night-rainy" }
            }
            c if c.is_snowy() => {
                if is_day { "weather-bg-day-snowy" }
                else { "weather-bg-night-snowy" }
            }
            c if c.is_stormy() => {
                if is_day { "weather-bg-day-stormy" }
                else { "weather-bg-night-stormy" }
            }
            _ => {
                if is_day { "weather-bg-day-clear" }
                else { "weather-bg-night-clear" }
            }
        };

        self.container.add_css_class(bg_class);
    }
}
