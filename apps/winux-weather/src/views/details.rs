//! Weather details view

use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation, ScrolledWindow, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage};
use chrono::{Local, Timelike};

use crate::data::{WeatherData, UvLevel};

/// Details view widget
pub struct DetailsView {
    container: ScrolledWindow,
    page: PreferencesPage,
}

impl DetailsView {
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

        let current = &data.current;

        // Conditions group
        let conditions_group = PreferencesGroup::builder()
            .title("Condicoes Atuais")
            .build();

        // Feels like
        let feels_row = ActionRow::builder()
            .title("Sensacao Termica")
            .subtitle(&format!("{:.0}°C", current.apparent_temperature))
            .build();
        feels_row.add_prefix(&Image::from_icon_name("weather-clear-symbolic"));
        conditions_group.add(&feels_row);

        // Humidity
        let humidity_row = ActionRow::builder()
            .title("Umidade")
            .subtitle(&format!("{}%", current.humidity))
            .build();
        humidity_row.add_prefix(&Image::from_icon_name("weather-fog-symbolic"));

        let humidity_bar = create_progress_bar(current.humidity as f64 / 100.0);
        humidity_row.add_suffix(&humidity_bar);
        conditions_group.add(&humidity_row);

        // Cloud cover
        let cloud_row = ActionRow::builder()
            .title("Cobertura de Nuvens")
            .subtitle(&format!("{}%", current.cloud_cover))
            .build();
        cloud_row.add_prefix(&Image::from_icon_name("weather-overcast-symbolic"));

        let cloud_bar = create_progress_bar(current.cloud_cover as f64 / 100.0);
        cloud_row.add_suffix(&cloud_bar);
        conditions_group.add(&cloud_row);

        self.page.add(&conditions_group);

        // Wind group
        let wind_group = PreferencesGroup::builder()
            .title("Vento")
            .build();

        // Wind speed
        let wind_speed_row = ActionRow::builder()
            .title("Velocidade")
            .subtitle(&format!("{:.1} km/h", current.wind_speed))
            .build();
        wind_speed_row.add_prefix(&Image::from_icon_name("weather-windy-symbolic"));
        wind_group.add(&wind_speed_row);

        // Wind direction
        let wind_dir_row = ActionRow::builder()
            .title("Direcao")
            .subtitle(&format!(
                "{} ({:.0}°)",
                current.wind_direction.full_name(),
                current.wind_direction.degrees
            ))
            .build();
        wind_dir_row.add_prefix(&Image::from_icon_name("object-rotate-right-symbolic"));
        wind_group.add(&wind_dir_row);

        // Wind gusts
        if current.wind_gusts > 0.0 {
            let gusts_row = ActionRow::builder()
                .title("Rajadas")
                .subtitle(&format!("{:.1} km/h", current.wind_gusts))
                .build();
            gusts_row.add_prefix(&Image::from_icon_name("weather-storm-symbolic"));
            wind_group.add(&gusts_row);
        }

        self.page.add(&wind_group);

        // Atmosphere group
        let atmosphere_group = PreferencesGroup::builder()
            .title("Atmosfera")
            .build();

        // Pressure
        let pressure_row = ActionRow::builder()
            .title("Pressao")
            .subtitle(&format!("{:.0} hPa", current.pressure))
            .build();
        pressure_row.add_prefix(&Image::from_icon_name("speedometer-symbolic"));
        atmosphere_group.add(&pressure_row);

        // UV Index
        let uv_level = UvLevel::from_index(current.uv_index);
        let uv_row = ActionRow::builder()
            .title("Indice UV")
            .subtitle(&format!("{:.0} - {}", current.uv_index, uv_level.description()))
            .build();
        uv_row.add_prefix(&Image::from_icon_name("weather-clear-symbolic"));

        let uv_indicator = create_uv_indicator(current.uv_index);
        uv_row.add_suffix(&uv_indicator);
        atmosphere_group.add(&uv_row);

        // UV recommendation
        let uv_rec_row = ActionRow::builder()
            .title("Recomendacao")
            .subtitle(uv_level.recommendation())
            .build();
        uv_rec_row.add_prefix(&Image::from_icon_name("dialog-information-symbolic"));
        atmosphere_group.add(&uv_rec_row);

        // Visibility
        let visibility_km = current.visibility / 1000.0;
        let visibility_row = ActionRow::builder()
            .title("Visibilidade")
            .subtitle(&format!("{:.1} km", visibility_km))
            .build();
        visibility_row.add_prefix(&Image::from_icon_name("view-reveal-symbolic"));
        atmosphere_group.add(&visibility_row);

        self.page.add(&atmosphere_group);

        // Sun group
        let sun_group = PreferencesGroup::builder()
            .title("Sol")
            .build();

        // Sunrise
        let sunrise_local = data.sunrise.with_timezone(&Local);
        let sunrise_row = ActionRow::builder()
            .title("Nascer do Sol")
            .subtitle(&format!("{:02}:{:02}", sunrise_local.hour(), sunrise_local.minute()))
            .build();
        sunrise_row.add_prefix(&Image::from_icon_name("weather-clear-symbolic"));
        sun_group.add(&sunrise_row);

        // Sunset
        let sunset_local = data.sunset.with_timezone(&Local);
        let sunset_row = ActionRow::builder()
            .title("Por do Sol")
            .subtitle(&format!("{:02}:{:02}", sunset_local.hour(), sunset_local.minute()))
            .build();
        sunset_row.add_prefix(&Image::from_icon_name("weather-clear-night-symbolic"));
        sun_group.add(&sunset_row);

        // Day length
        let day_length = data.sunset.signed_duration_since(data.sunrise);
        let hours = day_length.num_hours();
        let minutes = day_length.num_minutes() % 60;
        let daylight_row = ActionRow::builder()
            .title("Duracao do Dia")
            .subtitle(&format!("{}h {}min", hours, minutes))
            .build();
        daylight_row.add_prefix(&Image::from_icon_name("daytime-sunrise-symbolic"));
        sun_group.add(&daylight_row);

        self.page.add(&sun_group);

        // Precipitation group
        if current.precipitation > 0.0 {
            let precip_group = PreferencesGroup::builder()
                .title("Precipitacao")
                .build();

            let precip_row = ActionRow::builder()
                .title("Precipitacao Atual")
                .subtitle(&format!("{:.1} mm", current.precipitation))
                .build();
            precip_row.add_prefix(&Image::from_icon_name("weather-showers-symbolic"));
            precip_group.add(&precip_row);

            self.page.add(&precip_group);
        }
    }
}

fn create_progress_bar(fraction: f64) -> Box {
    let container = Box::new(Orientation::Vertical, 0);
    container.set_valign(gtk4::Align::Center);
    container.set_size_request(60, -1);

    let bar = ProgressBar::new();
    bar.set_fraction(fraction);
    bar.add_css_class("osd");

    container.append(&bar);
    container
}

fn create_uv_indicator(uv_index: f64) -> Box {
    let container = Box::new(Orientation::Horizontal, 4);
    container.set_valign(gtk4::Align::Center);

    // Create colored segments
    let levels = [
        (3.0, "uv-low"),
        (6.0, "uv-moderate"),
        (8.0, "uv-high"),
        (11.0, "uv-very-high"),
        (15.0, "uv-extreme"),
    ];

    for (threshold, class) in levels {
        let segment = Box::new(Orientation::Horizontal, 0);
        segment.set_size_request(10, 8);
        segment.add_css_class(class);

        if uv_index >= threshold - 3.0 && uv_index < threshold {
            segment.add_css_class("uv-active");
        }

        container.append(&segment);
    }

    container
}
