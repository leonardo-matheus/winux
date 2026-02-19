// Winux Weather - Weather Application for Winux OS
// Copyright (c) 2026 Winux OS Project
//
// Features:
// - Current weather conditions
// - Hourly and daily forecasts
// - Multiple locations support
// - Dynamic backgrounds based on weather
// - Uses Open-Meteo API (free, no key required)

mod window;
mod views;
mod api;
mod data;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.weather";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
