// Winux Clock - Clock Application for Winux OS
// Copyright (c) 2026 Winux OS Project
//
// Features:
// - World Clock with multiple timezones
// - Alarms with repeat and snooze
// - Stopwatch with laps
// - Timer with presets

mod window;
mod tabs;
mod data;
mod notifications;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Clock";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        window::build_window(app);
    });

    app.run()
}
