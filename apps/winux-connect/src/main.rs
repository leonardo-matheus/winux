// Winux Connect - Smartphone Integration
// Copyright (c) 2026 Winux OS Project
//
// KDE Connect compatible protocol for:
// - Device discovery via mDNS
// - Secure pairing with QR code and PIN
// - Bidirectional notification mirroring
// - SMS send/receive
// - File transfer with drag & drop
// - Clipboard sync
// - Remote media control (MPRIS)
// - Find phone (ring)
// - Battery level monitoring
// - Screen mirroring via scrcpy

mod window;
mod pages;
mod protocol;
mod services;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Connect";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::ConnectWindow::new(app);
        window.present();
    });

    app.run()
}
