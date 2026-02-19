// Winux Gaming - Unified Game Launcher
// Copyright (c) 2026 Winux OS Project
//
// A unified game launcher supporting:
// - Steam library integration
// - GOG Galaxy (via Heroic)
// - Epic Games (via Heroic)
// - Emulators (RetroArch, Dolphin, PCSX2, RPCS3)
// - Wine/Proton for Windows games
// - GameMode and MangoHud integration
// - Per-game optimization settings

mod window;
mod pages;
mod launchers;
mod optimization;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Gaming";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
