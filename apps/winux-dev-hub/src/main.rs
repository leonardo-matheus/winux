// Winux Dev Hub - Central Development Hub for Developers
// Copyright (c) 2026 Winux OS Project
//
// A comprehensive developer hub providing:
// - Project Dashboard with auto-detection
// - Environment variable management with profiles
// - Toolchain status and management
// - Container orchestration (Docker/Podman)
// - Database management
// - System services control

mod window;
mod pages;
mod widgets;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.DevHub";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
