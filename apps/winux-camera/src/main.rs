//! Winux Camera - Camera application for Winux OS
//!
//! Features:
//! - Real-time camera preview with PipeWire/V4L2 integration
//! - Photo capture with timer and burst mode
//! - Video recording with configurable resolution and FPS
//! - Image filters and effects
//! - Quick gallery access

mod window;
mod camera;
mod capture;
mod processing;
mod ui;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.camera";

fn main() -> gtk::glib::ExitCode {
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    log::info!("Starting Winux Camera");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        adw::init().expect("Failed to initialize libadwaita");
    });

    app.connect_activate(|app| {
        window::build_window(app);
    });

    app.run()
}
