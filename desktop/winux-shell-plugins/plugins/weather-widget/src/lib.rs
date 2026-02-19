//! Weather Widget Plugin
//!
//! Shows current weather conditions in the panel.

use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use winux_shell_plugins::prelude::*;

/// Weather data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WeatherData {
    /// Temperature in Celsius
    temperature: f64,
    /// Weather condition
    condition: String,
    /// Weather icon code
    icon: String,
    /// Location name
    location: String,
    /// Humidity percentage
    humidity: u32,
    /// Wind speed in km/h
    wind_speed: f64,
    /// Last update timestamp
    last_update: Option<chrono::DateTime<chrono::Utc>>,
}

/// Weather widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeatherConfig {
    /// Location (city name or coordinates)
    location: String,
    /// Use metric units
    metric: bool,
    /// Update interval in minutes
    update_interval: u32,
    /// Show humidity
    show_humidity: bool,
    /// Show wind
    show_wind: bool,
    /// API key (optional, uses free tier if not set)
    api_key: Option<String>,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            location: "auto".to_string(),
            metric: true,
            update_interval: 30,
            show_humidity: false,
            show_wind: false,
            api_key: None,
        }
    }
}

/// Weather widget plugin
pub struct WeatherWidgetPlugin {
    config: WeatherConfig,
    weather: Arc<RwLock<WeatherData>>,
    last_fetch: Option<std::time::Instant>,
}

impl Default for WeatherWidgetPlugin {
    fn default() -> Self {
        Self {
            config: WeatherConfig::default(),
            weather: Arc::new(RwLock::new(WeatherData::default())),
            last_fetch: None,
        }
    }
}

impl WeatherWidgetPlugin {
    /// Get weather icon name from condition code
    fn get_icon_name(condition: &str) -> &'static str {
        match condition.to_lowercase().as_str() {
            s if s.contains("clear") || s.contains("sunny") => "weather-clear-symbolic",
            s if s.contains("cloud") && s.contains("few") => "weather-few-clouds-symbolic",
            s if s.contains("cloud") => "weather-overcast-symbolic",
            s if s.contains("rain") || s.contains("drizzle") => "weather-showers-symbolic",
            s if s.contains("thunder") || s.contains("storm") => "weather-storm-symbolic",
            s if s.contains("snow") => "weather-snow-symbolic",
            s if s.contains("fog") || s.contains("mist") => "weather-fog-symbolic",
            s if s.contains("wind") => "weather-windy-symbolic",
            _ => "weather-severe-alert-symbolic",
        }
    }

    /// Fetch weather data (mock implementation)
    fn fetch_weather(&mut self) -> Result<WeatherData, String> {
        // In a real implementation, this would call a weather API
        // For now, return mock data

        let weather = WeatherData {
            temperature: 22.0,
            condition: "Partly Cloudy".to_string(),
            icon: "weather-few-clouds".to_string(),
            location: if self.config.location == "auto" {
                "Current Location".to_string()
            } else {
                self.config.location.clone()
            },
            humidity: 65,
            wind_speed: 12.5,
            last_update: Some(chrono::Utc::now()),
        };

        *self.weather.write().unwrap() = weather.clone();
        self.last_fetch = Some(std::time::Instant::now());

        Ok(weather)
    }

    /// Check if weather data needs update
    fn needs_update(&self) -> bool {
        match self.last_fetch {
            None => true,
            Some(last) => {
                let elapsed = last.elapsed();
                elapsed.as_secs() > (self.config.update_interval as u64 * 60)
            }
        }
    }
}

impl Plugin for WeatherWidgetPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "org.winux.weather-widget".into(),
            name: "Weather Widget".into(),
            version: Version::new(1, 0, 0),
            description: "Shows current weather conditions in the panel".into(),
            authors: vec!["Winux Team".into()],
            homepage: Some("https://winux.org/plugins/weather".into()),
            license: Some("MIT".into()),
            min_api_version: Version::new(1, 0, 0),
            capabilities: vec![PluginCapability::PanelWidget, PluginCapability::Network],
            permissions: {
                let mut perms = PermissionSet::new();
                perms.add(Permission::Network);
                perms.add(Permission::PanelWidgets);
                perms.add(Permission::Location);
                perms.add(Permission::OwnData);
                perms
            },
            icon: Some("weather-few-clouds-symbolic".into()),
            category: Some("Utilities".into()),
            keywords: vec!["weather".into(), "temperature".into(), "forecast".into()],
            ..Default::default()
        }
    }

    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        // Load config
        let config_path = ctx.config_file("config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.config = config;
                }
            }
        }

        // Initial weather fetch
        let _ = self.fetch_weather();

        log::info!("Weather widget initialized for location: {}", self.config.location);
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        log::info!("Weather widget shutting down");
        Ok(())
    }

    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        Some(Box::new(WeatherPanelWidget {
            weather: self.weather.clone(),
            config: self.config.clone(),
        }))
    }

    fn settings_provider(&self) -> Option<Box<dyn SettingsProvider>> {
        Some(Box::new(WeatherSettingsProvider {
            config: self.config.clone(),
        }))
    }

    fn wants_updates(&self) -> bool {
        true
    }

    fn update_interval(&self) -> u32 {
        60000 // Check every minute
    }

    fn update(&mut self) -> PluginResult<()> {
        if self.needs_update() {
            let _ = self.fetch_weather();
        }
        Ok(())
    }
}

/// Panel widget for weather
struct WeatherPanelWidget {
    weather: Arc<RwLock<WeatherData>>,
    config: WeatherConfig,
}

impl PanelWidget for WeatherPanelWidget {
    fn id(&self) -> &str {
        "weather-widget"
    }

    fn name(&self) -> &str {
        "Weather"
    }

    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    fn size(&self) -> WidgetSize {
        WidgetSize::Small
    }

    fn priority(&self) -> i32 {
        10
    }

    fn state(&self) -> WidgetState {
        let weather = self.weather.read().unwrap();
        let icon = WeatherWidgetPlugin::get_icon_name(&weather.condition);

        let label = if self.config.metric {
            format!("{}°C", weather.temperature as i32)
        } else {
            let fahrenheit = weather.temperature * 9.0 / 5.0 + 32.0;
            format!("{}°F", fahrenheit as i32)
        };

        WidgetState::with_icon(icon)
            .label(&label)
            .tooltip(&format!("{} - {}", weather.location, weather.condition))
    }

    fn build_widget(&self) -> gtk::Widget {
        let weather = self.weather.read().unwrap();

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        hbox.set_valign(gtk::Align::Center);
        hbox.add_css_class("weather-widget");

        // Weather icon
        let icon_name = WeatherWidgetPlugin::get_icon_name(&weather.condition);
        let icon = gtk::Image::from_icon_name(icon_name);
        icon.set_pixel_size(16);
        hbox.append(&icon);

        // Temperature
        let temp_label = if self.config.metric {
            format!("{}°", weather.temperature as i32)
        } else {
            let fahrenheit = weather.temperature * 9.0 / 5.0 + 32.0;
            format!("{}°", fahrenheit as i32)
        };
        let label = gtk::Label::new(Some(&temp_label));
        label.add_css_class("weather-temp");
        hbox.append(&label);

        // Tooltip with more details
        let tooltip = format!(
            "{}\n{}\nHumidity: {}%\nWind: {} km/h",
            weather.location,
            weather.condition,
            weather.humidity,
            weather.wind_speed
        );
        hbox.set_tooltip_text(Some(&tooltip));

        hbox.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        WidgetAction::ShowPopup
    }

    fn popup_config(&self) -> Option<PopupConfig> {
        Some(PopupConfig {
            width: 280,
            height: 320,
            ..Default::default()
        })
    }

    fn build_popup(&self) -> Option<gtk::Widget> {
        let weather = self.weather.read().unwrap();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
        vbox.set_margin_top(16);
        vbox.set_margin_bottom(16);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);
        vbox.add_css_class("weather-popup");

        // Location
        let location_label = gtk::Label::new(Some(&weather.location));
        location_label.add_css_class("title-3");
        vbox.append(&location_label);

        // Current weather
        let current_box = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        current_box.set_halign(gtk::Align::Center);

        let icon_name = WeatherWidgetPlugin::get_icon_name(&weather.condition);
        let icon = gtk::Image::from_icon_name(icon_name);
        icon.set_pixel_size(64);
        current_box.append(&icon);

        let temp_label = gtk::Label::new(Some(&format!("{}°", weather.temperature as i32)));
        temp_label.add_css_class("title-1");
        current_box.append(&temp_label);

        vbox.append(&current_box);

        // Condition
        let condition_label = gtk::Label::new(Some(&weather.condition));
        condition_label.add_css_class("title-4");
        vbox.append(&condition_label);

        // Details
        let details_box = gtk::Box::new(gtk::Orientation::Horizontal, 24);
        details_box.set_halign(gtk::Align::Center);
        details_box.set_margin_top(12);

        // Humidity
        let humidity_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let humidity_icon = gtk::Image::from_icon_name("weather-showers-symbolic");
        humidity_icon.set_pixel_size(24);
        humidity_box.append(&humidity_icon);
        let humidity_label = gtk::Label::new(Some(&format!("{}%", weather.humidity)));
        humidity_box.append(&humidity_label);
        let humidity_title = gtk::Label::new(Some("Humidity"));
        humidity_title.add_css_class("dim-label");
        humidity_box.append(&humidity_title);
        details_box.append(&humidity_box);

        // Wind
        let wind_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let wind_icon = gtk::Image::from_icon_name("weather-windy-symbolic");
        wind_icon.set_pixel_size(24);
        wind_box.append(&wind_icon);
        let wind_label = gtk::Label::new(Some(&format!("{} km/h", weather.wind_speed as i32)));
        wind_box.append(&wind_label);
        let wind_title = gtk::Label::new(Some("Wind"));
        wind_title.add_css_class("dim-label");
        wind_box.append(&wind_title);
        details_box.append(&wind_box);

        vbox.append(&details_box);

        // Last update
        if let Some(last_update) = weather.last_update {
            let update_label = gtk::Label::new(Some(&format!(
                "Updated: {}",
                last_update.format("%H:%M")
            )));
            update_label.add_css_class("dim-label");
            update_label.set_margin_top(12);
            vbox.append(&update_label);
        }

        Some(vbox.upcast())
    }
}

/// Settings provider for weather widget
struct WeatherSettingsProvider {
    config: WeatherConfig,
}

impl SettingsProvider for WeatherSettingsProvider {
    fn id(&self) -> &str {
        "weather-widget-settings"
    }

    fn pages(&self) -> Vec<SettingsPage> {
        vec![SettingsPage::new("weather", "Weather Widget", "weather-few-clouds-symbolic")
            .with_description("Configure weather widget settings")
            .add_group(
                SettingGroup::new("Location")
                    .add(
                        Setting::text("location", "Location", &self.config.location)
                            .with_description("City name or 'auto' for automatic location"),
                    )
            )
            .add_group(
                SettingGroup::new("Display")
                    .add(
                        Setting::toggle("metric", "Use Metric Units", self.config.metric)
                            .with_description("Show temperature in Celsius"),
                    )
                    .add(
                        Setting::toggle("show_humidity", "Show Humidity", self.config.show_humidity),
                    )
                    .add(
                        Setting::toggle("show_wind", "Show Wind Speed", self.config.show_wind),
                    )
            )
            .add_group(
                SettingGroup::new("Updates")
                    .add(
                        Setting::slider("update_interval", "Update Interval (minutes)", 5.0, 120.0, 5.0, self.config.update_interval as f64),
                    )
            )]
    }

    fn load(&mut self) -> std::collections::HashMap<String, SettingValue> {
        let mut values = std::collections::HashMap::new();
        values.insert("location".to_string(), SettingValue::String(self.config.location.clone()));
        values.insert("metric".to_string(), SettingValue::Bool(self.config.metric));
        values.insert("show_humidity".to_string(), SettingValue::Bool(self.config.show_humidity));
        values.insert("show_wind".to_string(), SettingValue::Bool(self.config.show_wind));
        values.insert("update_interval".to_string(), SettingValue::Float(self.config.update_interval as f64));
        values
    }

    fn save(&mut self, key: &str, value: SettingValue) -> Result<(), String> {
        match key {
            "location" => {
                if let Some(v) = value.as_string() {
                    self.config.location = v.to_string();
                }
            }
            "metric" => {
                if let Some(v) = value.as_bool() {
                    self.config.metric = v;
                }
            }
            "show_humidity" => {
                if let Some(v) = value.as_bool() {
                    self.config.show_humidity = v;
                }
            }
            "show_wind" => {
                if let Some(v) = value.as_bool() {
                    self.config.show_wind = v;
                }
            }
            "update_interval" => {
                if let Some(v) = value.as_float() {
                    self.config.update_interval = v as u32;
                }
            }
            _ => return Err(format!("Unknown setting: {}", key)),
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<(), String> {
        self.config = WeatherConfig::default();
        Ok(())
    }

    fn reset_setting(&mut self, key: &str) -> Result<(), String> {
        let default = WeatherConfig::default();
        match key {
            "location" => self.config.location = default.location,
            "metric" => self.config.metric = default.metric,
            "show_humidity" => self.config.show_humidity = default.show_humidity,
            "show_wind" => self.config.show_wind = default.show_wind,
            "update_interval" => self.config.update_interval = default.update_interval,
            _ => return Err(format!("Unknown setting: {}", key)),
        }
        Ok(())
    }
}

// Plugin entry point
winux_shell_plugins::declare_plugin!(WeatherWidgetPlugin, WeatherWidgetPlugin::default);
