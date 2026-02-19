//! Hourly forecast view

use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use chrono::{Local, Timelike};

use crate::data::WeatherData;
use crate::ui::ForecastRow;

/// Hourly forecast view widget
pub struct HourlyView {
    container: ScrolledWindow,
    content_box: Box,
}

impl HourlyView {
    pub fn new() -> Self {
        let content_box = Box::new(Orientation::Vertical, 0);
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(16);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);

        // Title
        let title = Label::new(Some("Previsao por Hora"));
        title.add_css_class("title-3");
        title.set_xalign(0.0);
        title.set_margin_bottom(16);
        content_box.append(&title);

        // Horizontal scroll for hours
        let hours_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(160)
            .build();

        let hours_box = Box::new(Orientation::Horizontal, 12);
        hours_box.set_margin_top(8);
        hours_box.set_margin_bottom(8);
        hours_box.add_css_class("hourly-forecast-box");

        hours_scroll.set_child(Some(&hours_box));
        content_box.append(&hours_scroll);

        // Precipitation section
        let precip_title = Label::new(Some("Probabilidade de Chuva"));
        precip_title.add_css_class("title-4");
        precip_title.set_xalign(0.0);
        precip_title.set_margin_top(24);
        precip_title.set_margin_bottom(12);
        content_box.append(&precip_title);

        // Precipitation chart placeholder
        let precip_box = Box::new(Orientation::Horizontal, 4);
        precip_box.set_homogeneous(true);
        precip_box.add_css_class("precipitation-chart");
        content_box.append(&precip_box);

        let container = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&content_box)
            .build();

        Self {
            container,
            content_box,
        }
    }

    /// Get the widget to display
    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    /// Update the view with new weather data
    pub fn update(&self, data: &WeatherData) {
        // Clear existing content and rebuild
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        // Title
        let title = Label::new(Some("Previsao por Hora"));
        title.add_css_class("title-3");
        title.set_xalign(0.0);
        title.set_margin_bottom(16);
        self.content_box.append(&title);

        // Horizontal scroll for hours
        let hours_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(140)
            .build();

        let hours_box = Box::new(Orientation::Horizontal, 8);
        hours_box.set_margin_top(8);
        hours_box.set_margin_bottom(8);

        // Add hourly forecasts
        let now = chrono::Utc::now();
        for forecast in data.hourly.iter().filter(|h| h.time >= now).take(24) {
            let hour_widget = create_hour_widget(forecast);
            hours_box.append(&hour_widget);
        }

        hours_scroll.set_child(Some(&hours_box));
        self.content_box.append(&hours_scroll);

        // Precipitation section
        let precip_title = Label::new(Some("Probabilidade de Chuva"));
        precip_title.add_css_class("title-4");
        precip_title.set_xalign(0.0);
        precip_title.set_margin_top(24);
        precip_title.set_margin_bottom(12);
        self.content_box.append(&precip_title);

        // Precipitation bars
        let precip_box = Box::new(Orientation::Horizontal, 2);
        precip_box.set_homogeneous(true);

        for forecast in data.hourly.iter().filter(|h| h.time >= now).take(12) {
            let bar = create_precipitation_bar(forecast.precipitation_probability);
            precip_box.append(&bar);
        }

        self.content_box.append(&precip_box);

        // Legend
        let legend_box = Box::new(Orientation::Horizontal, 16);
        legend_box.set_margin_top(8);
        legend_box.set_halign(gtk4::Align::Center);

        for hour in data.hourly.iter().filter(|h| h.time >= now).take(12).step_by(3) {
            let local_time = hour.time.with_timezone(&Local);
            let label = Label::new(Some(&format!("{:02}h", local_time.hour())));
            label.add_css_class("dim-label");
            label.add_css_class("caption");
            legend_box.append(&label);
        }

        self.content_box.append(&legend_box);
    }
}

fn create_hour_widget(forecast: &crate::data::HourlyForecast) -> Box {
    let container = Box::new(Orientation::Vertical, 4);
    container.set_margin_start(8);
    container.set_margin_end(8);
    container.add_css_class("hour-forecast-item");

    // Time
    let local_time = forecast.time.with_timezone(&Local);
    let time_label = Label::new(Some(&format!("{:02}h", local_time.hour())));
    time_label.add_css_class("caption");
    time_label.add_css_class("dim-label");

    // Icon
    let icon = Image::from_icon_name(forecast.condition.icon_name(forecast.is_day));
    icon.set_pixel_size(32);

    // Temperature
    let temp_label = Label::new(Some(&format!("{:.0}°", forecast.temperature)));
    temp_label.add_css_class("heading");

    // Precipitation probability (if > 0)
    let precip_label = if forecast.precipitation_probability > 0 {
        let label = Label::new(Some(&format!("{}%", forecast.precipitation_probability)));
        label.add_css_class("caption");
        label.add_css_class("accent");
        label
    } else {
        Label::new(None)
    };

    container.append(&time_label);
    container.append(&icon);
    container.append(&temp_label);
    container.append(&precip_label);

    container
}

fn create_precipitation_bar(probability: i32) -> Box {
    let container = Box::new(Orientation::Vertical, 0);
    container.set_valign(gtk4::Align::End);
    container.set_size_request(20, 60);

    let height = (probability as f64 / 100.0 * 50.0) as i32;
    let bar = Box::new(Orientation::Vertical, 0);
    bar.set_size_request(-1, height.max(2));
    bar.set_valign(gtk4::Align::End);

    if probability > 60 {
        bar.add_css_class("precip-bar-high");
    } else if probability > 30 {
        bar.add_css_class("precip-bar-medium");
    } else {
        bar.add_css_class("precip-bar-low");
    }

    container.append(&bar);
    container
}
