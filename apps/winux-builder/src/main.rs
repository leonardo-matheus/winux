// Winux Builder - Cross-platform Application Builder
// Copyright (c) 2026 Winux OS Project
//
// Build applications for multiple platforms:
// - macOS: .app, .dmg, .pkg, .ipa
// - Windows: .exe, .msi
// - Linux: .deb, .rpm, .AppImage, .flatpak
//
// Supports: Rust, .NET, Electron, Flutter projects

mod window;
mod builders;
mod projects;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Builder";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
