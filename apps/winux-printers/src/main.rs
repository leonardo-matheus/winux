// Winux Printers - Printer Manager
// Copyright (c) 2026 Winux OS Project
//
// Complete printer management with:
// - List and manage configured printers
// - Automatic printer discovery (Avahi/mDNS)
// - CUPS integration for print jobs
// - Printer configuration and settings
// - Print queue management

mod window;
mod pages;
mod cups;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Printers";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::PrinterWindow::new(app);
        window.present();
    });

    app.run()
}
