// Winux Accessibility - System Accessibility Center
// Copyright (c) 2026 Winux OS Project
//
// Complete accessibility settings with:
// - Vision (High contrast, Large text, Screen reader)
// - Hearing (Visual alerts, Captions, Mono audio)
// - Typing (On-screen keyboard, Sticky keys, Slow keys)
// - Pointing (Mouse keys, Click assist, Hover click)
// - Zoom (Magnifier, Zoom level, Follow cursor)

mod window;
mod pages;
mod settings;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Accessibility";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::AccessibilityWindow::new(app);
        window.present();
    });

    app.run()
}
