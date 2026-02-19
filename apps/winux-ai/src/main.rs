// Winux AI - AI Assistant for Winux OS
// Copyright (c) 2026 Winux OS Project
// Integrated with Azure OpenAI (GPT-4o, o1)

mod ai;
mod chat;
mod database;
mod features;
mod integrations;
mod ui;
mod window;

use gtk4::prelude::*;
use gtk4::{gio, Application};
use libadwaita as adw;

const APP_ID: &str = "org.winux.ai";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_activate(|app| {
        let window = window::AiWindow::new(app);
        window.present();
    });

    app.connect_command_line(|app, _| {
        app.activate();
        0
    });

    // Register global shortcut Super+A
    app.set_accels_for_action("app.show-window", &["<Super>a"]);

    app.run()
}
