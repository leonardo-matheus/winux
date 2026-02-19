// Winux Firewall - Firewall Management Center
// Copyright (c) 2026 Winux OS Project
//
// Complete firewall management with:
// - UFW integration (primary)
// - FirewallD support (alternative)
// - Rule management (add, edit, delete)
// - Application profiles
// - Connection logs
// - Security presets

mod window;
mod pages;
mod backend;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Firewall";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::FirewallWindow::new(app);
        window.present();
    });

    app.run()
}
