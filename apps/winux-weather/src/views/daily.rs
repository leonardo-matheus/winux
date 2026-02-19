//! Daily forecast view

use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation, ScrolledWindow, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage};

use crate::data::WeatherData;
use crate::ui::WeatherCard;

/// Daily forecast view widget
pub struct DailyView {
    container: ScrolledWindow,
    page: PreferencesPage,
}

impl DailyView {
    pub fn new() -> Self {
        let page = PreferencesPage::new();

        let container = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { container, page }
    }

    /// Get the widget to display
    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    /// Update the view with new weather data
    pub fn update(&self, data: &WeatherData) {
        // Clear existing groups
        while let Some(child) = self.page.first_child() {
            self.page.remove(&child);
        }

        // 7-Day forecast group
        let daily_group = PreferencesGroup::builder()
            .title("Previsao para 7 Dias")
            .build();

        // Calculate min/max for the week to normalize temperature bars
        let week_min = data.daily.iter()
            .map(|d| d.temperature_min)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let week_max = data.daily.iter()
            .map(|d| d.temperature_max)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(40.0);
        let temp_range = week_max - week_min;

        for forecast in data.next_7_days() {
            let row = create_daily_row(forecast, week_min, temp_range);
            daily_group.add(&row);
        }

        self.page.add(&daily_group);

        // Summary group
        let summary_group = PreferencesGroup::builder()
            .title("Resumo da Semana")
            .build();

        // Calculate averages
        let avg_max: f64 = data.daily.iter().map(|d| d.temperature_max).sum::<f64>() / data.daily.len() as f64;
        let avg_min: f64 = data.daily.iter().map(|d| d.temperature_min).sum::<f64>() / data.daily.len() as f64;
        let total_precip: f64 = data.daily.iter().map(|d| d.precipitation_sum).sum();
        let rainy_days = data.daily.iter().filter(|d| d.precipitation_probability_max > 50).count();

        let temp_row = ActionRow::builder()
            .title("Temperatura Media")
            .subtitle(&format!("Max: {:.0}° / Min: {:.0}°", avg_max, avg_min))
            .build();
        temp_row.add_prefix(&Image::from_icon_name("weather-clear-symbolic"));
        summary_group.add(&temp_row);

        let precip_row = ActionRow::builder()
            .title("Precipitacao Total")
            .subtitle(&format!("{:.1} mm", total_precip))
            .build();
        precip_row.add_prefix(&Image::from_icon_name("weather-showers-symbolic"));
        summary_group.add(&precip_row);

        let rainy_row = ActionRow::builder()
            .title("Dias com Chuva")
            .subtitle(&format!("{} de {} dias", rainy_days, data.daily.len()))
            .build();
        rainy_row.add_prefix(&Image::from_icon_name("weather-showers-scattered-symbolic"));
        summary_group.add(&rainy_row);

        self.page.add(&summary_group);
    }
}

fn create_daily_row(forecast: &crate::data::DailyForecast, week_min: f64, temp_range: f64) -> ActionRow {
    let row = ActionRow::builder()
        .title(&forecast.day_name())
        .subtitle(&format!("{}", forecast.formatted_date()))
        .build();

    // Weather icon
    let icon = Image::from_icon_name(forecast.condition.icon_name(true));
    icon.set_pixel_size(32);
    row.add_prefix(&icon);

    // Temperature range indicator
    let temp_box = Box::new(Orientation::Horizontal, 8);
    temp_box.set_valign(gtk4::Align::Center);

    // Min temp
    let min_label = Label::new(Some(&format!("{:.0}°", forecast.temperature_min)));
    min_label.add_css_class("dim-label");
    min_label.set_width_chars(4);

    // Temperature bar
    let bar_container = Box::new(Orientation::Horizontal, 0);
    bar_container.set_size_request(80, 6);
    bar_container.add_css_class("temp-bar-container");

    // Calculate bar position and width
    let left_offset = if temp_range > 0.0 {
        ((forecast.temperature_min - week_min) / temp_range * 80.0) as i32
    } else {
        0
    };
    let bar_width = if temp_range > 0.0 {
        ((forecast.temperature_max - forecast.temperature_min) / temp_range * 80.0) as i32
    } else {
        80
    };

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_size_request(left_offset, -1);

    let bar = Box::new(Orientation::Horizontal, 0);
    bar.set_size_request(bar_width.max(10), 6);
    bar.add_css_class("temp-bar");

    bar_container.append(&spacer);
    bar_container.append(&bar);

    // Max temp
    let max_label = Label::new(Some(&format!("{:.0}°", forecast.temperature_max)));
    max_label.set_width_chars(4);

    temp_box.append(&min_label);
    temp_box.append(&bar_container);
    temp_box.append(&max_label);

    // Precipitation probability
    if forecast.precipitation_probability_max > 0 {
        let precip_box = Box::new(Orientation::Horizontal, 4);
        precip_box.set_valign(gtk4::Align::Center);
        precip_box.set_margin_start(8);

        let drop_icon = Image::from_icon_name("weather-showers-symbolic");
        drop_icon.set_pixel_size(12);
        drop_icon.add_css_class("dim-label");

        let precip_label = Label::new(Some(&format!("{}%", forecast.precipitation_probability_max)));
        precip_label.add_css_class("caption");
        precip_label.add_css_class("accent");

        precip_box.append(&drop_icon);
        precip_box.append(&precip_label);

        temp_box.append(&precip_box);
    }

    row.add_suffix(&temp_box);

    row
}
