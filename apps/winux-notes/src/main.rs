// Winux Notes - Note-taking application for Winux OS
// Copyright (c) 2026 Winux OS Project

mod data;
mod editor;
mod ui;
mod views;
mod window;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.notes";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        let window = window::NotesWindow::new(app);
        window.present();
    });
    app.run()
}
