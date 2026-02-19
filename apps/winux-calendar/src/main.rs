// Winux Calendar - Calendar Application for Winux OS
// Copyright (c) 2026 Winux OS Project
//
// Features:
// - Month, Week, Day, and Agenda views
// - Event management (create, edit, delete)
// - Multiple calendars with colors
// - CalDAV synchronization
// - iCal import/export
// - Integrated tasks

mod window;
mod views;
mod data;
mod sync;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Calendar";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        window::build_ui(app);
    });

    app.run()
}
