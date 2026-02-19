// Winux Disks - Disk Management Utility
// Copyright (c) 2026 Winux OS Project
//
// A comprehensive disk management tool with:
// - Disk overview and SMART status
// - Partition visualization
// - Mount/unmount operations
// - Format with multiple filesystems
// - Partition creation/deletion
// - Benchmark tools
// - Disk imaging

mod window;
mod pages;
mod backend;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Disks";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::DisksWindow::new(app);
        window.present();
    });

    app.run()
}
