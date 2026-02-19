// Winux Updater - System Update Manager
// Copyright (c) 2026 Winux OS Project
//
// Complete system updater with:
// - APT package updates
// - Flatpak updates
// - Snap updates
// - Firmware updates (fwupd)
// - Driver management
// - Update history and rollback

mod window;
mod pages;
mod backend;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Updater";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::UpdaterWindow::new(app);
        window.present();
    });

    app.run()
}
