//! Winux Archive - Archive manager for Winux OS
//!
//! Supports multiple archive formats:
//! - ZIP (create/extract with encryption support)
//! - RAR (extract, create via unrar)
//! - TAR, TAR.GZ, TAR.BZ2, TAR.XZ
//! - 7z (create/extract)
//! - ISO (extract/mount)
//! - ZSTD

mod window;
mod archive;
mod operations;
mod ui;

use gtk4::prelude::*;
use gtk4::{glib, Application};
use libadwaita as adw;

const APP_ID: &str = "org.winux.Archive";

fn main() -> glib::ExitCode {
    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(|app| {
        window::build_window(app, None);
    });

    app.connect_open(|app, files, _| {
        if let Some(file) = files.first() {
            if let Some(path) = file.path() {
                window::build_window(app, Some(path));
            }
        }
    });

    app.run()
}
