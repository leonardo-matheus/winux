// Winux Fonts - Font Manager for Winux OS
// Copyright (c) 2026 Winux OS Project
//
// Features:
// - Browse installed fonts with filtering
// - Preview fonts with customizable text
// - Compare multiple fonts side by side
// - Install/uninstall fonts
// - View font details and glyph grid

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

mod window;
mod pages;
mod fonts;
mod ui;

const APP_ID: &str = "org.winux.Fonts";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
