// Winux Power - Power Management Application
// Copyright (c) 2026 Winux OS Project
//
// Complete power management with:
// - Battery status and health monitoring
// - Power profiles (Performance, Balanced, Power Saver)
// - Display power settings
// - USB/peripheral power management
// - Usage statistics and history

mod window;
mod pages;
mod backend;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Power";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::PowerWindow::new(app);
        window.present();
    });

    app.run()
}
