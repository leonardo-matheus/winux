//! Winux Documents - Document Viewer for Winux OS
//!
//! Supports PDF, EPUB, DjVu, XPS, and comic book formats (CBZ/CBR)

mod window;
mod viewer;
mod features;
mod ui;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.documents";

fn main() -> gtk::glib::ExitCode {
    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "winux_documents=info");
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|_| {
        adw::init().expect("Failed to initialize libadwaita");
    });

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
