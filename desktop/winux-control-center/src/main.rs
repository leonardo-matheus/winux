//! Winux Control Center - iOS/macOS style quick settings panel
//!
//! A modern control center inspired by iOS/macOS providing quick access to:
//! - WiFi and Bluetooth controls
//! - Volume and brightness sliders
//! - Do Not Disturb, Night Light, and Airplane mode
//! - Media playback controls
//! - Battery status and power mode
//!
//! Features:
//! - Smooth animations and blur background
//! - Grid layout with rounded tiles
//! - Layer shell support for Wayland

mod config;
mod window;
mod widgets;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{gdk, Application};
use libadwaita as adw;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub const APP_ID: &str = "org.winux.ControlCenter";

fn main() -> gtk::glib::ExitCode {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!("Starting Winux Control Center");

    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    // Load CSS
    load_css();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        window::build_control_center(app);
    });

    // Handle command line for toggle behavior
    app.connect_startup(|_| {
        info!("Control Center startup complete");
    });

    app.run()
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    info!("CSS styles loaded");
}
