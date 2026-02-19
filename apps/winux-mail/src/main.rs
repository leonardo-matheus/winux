// Winux Mail - Modern email client for Winux OS
// Copyright (c) 2026 Winux OS Project

mod backend;
mod data;
mod ui;
mod views;
mod window;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.mail";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(|app| {
        let window = window::MailWindow::new(app);
        window.present();
    });

    // Handle mailto: URLs
    app.connect_open(|app, files, _| {
        let window = window::MailWindow::new(app);

        for file in files {
            if let Some(uri) = file.uri().map(|u| u.to_string()) {
                if uri.starts_with("mailto:") {
                    window.compose_mailto(&uri);
                }
            }
        }

        window.present();
    });

    app.run()
}
