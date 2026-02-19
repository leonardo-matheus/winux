// Winux Backup - Backup and Restore Application
// Copyright (c) 2026 Winux OS Project
//
// Complete backup solution with:
// - Local, Remote (rsync), Restic, and Cloud backends
// - System, Home, Custom folder backups
// - Incremental backups with deduplication
// - Scheduling and retention policies
// - Encryption and compression

mod window;
mod pages;
mod backends;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Backup";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::BackupWindow::new(app);
        window.present();
    });

    app.run()
}
