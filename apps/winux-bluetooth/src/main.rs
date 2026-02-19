// Winux Bluetooth - Bluetooth Manager
// Copyright (c) 2026 Winux OS Project
//
// Complete Bluetooth management with:
// - Paired devices management
// - Device discovery and pairing
// - File transfer (OBEX)
// - Audio profiles (A2DP, HFP)
// - Settings and configuration

mod window;
mod pages;
mod bluez;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Bluetooth";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::BluetoothWindow::new(app);
        window.present();
    });

    app.run()
}
