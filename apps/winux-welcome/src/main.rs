// Winux Welcome - Onboarding Experience
// Copyright (c) 2026 Winux OS Project
//
// First-run experience with:
// - Welcome screen with animated logo
// - Desktop mode selection (Windows/Mac/Linux style)
// - Appearance customization (theme, accent color, wallpaper)
// - Recommended apps installation
// - Development environment setup
// - Gaming setup
// - Privacy configuration
// - Summary and finish

mod window;
mod pages;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Welcome";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
