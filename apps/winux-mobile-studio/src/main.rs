// Winux Mobile Studio - Mobile Development IDE
// Copyright (c) 2026 Winux OS Project
//
// A comprehensive mobile development environment providing:
// - Project management for Flutter, React Native, Android, iOS/Swift
// - Build systems for APK, AAB, IPA
// - Device management via ADB and libimobiledevice
// - Emulator management
// - Integrated terminal with build output

mod window;
mod pages;
mod builders;
mod devices;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.MobileStudio";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
