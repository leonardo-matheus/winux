// Winux Network - Network Management Center
// Copyright (c) 2026 Winux OS Project
//
// Complete network management with:
// - WiFi networks (scan, connect, manage)
// - Ethernet connections
// - VPN (OpenVPN, WireGuard)
// - Hotspot/AP mode
// - Proxy settings
// - Advanced (DNS, routing, firewall)

mod window;
mod pages;
mod nm;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Network";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::NetworkWindow::new(app);
        window.present();
    });

    app.run()
}
