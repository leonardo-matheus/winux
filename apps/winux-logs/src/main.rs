// Winux Logs - System Log Viewer
// Copyright (c) 2026 Winux OS Project
//
// View and analyze system logs from:
// - systemd journal (journalctl)
// - Kernel messages (dmesg)
// - Traditional syslog (/var/log)
// - Application-specific logs

mod sources;
mod filters;
mod ui;
mod window;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Logs";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(window::build_ui);
    app.run()
}
