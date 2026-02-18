//! Winux Store - Application store for Winux OS
//!
//! A modern application store with support for Flatpak and APT packages,
//! providing a unified interface for discovering, installing, and managing
//! applications on Winux OS.

mod app;
mod backend;
mod ui;

use adw::prelude::*;
use gtk4::{gio, glib};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use app::StoreApplication;

/// Application ID for D-Bus and desktop integration
const APP_ID: &str = "com.winux.Store";

fn main() -> glib::ExitCode {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!("Starting Winux Store");

    // Register resources
    gio::resources_register_include!("winux-store.gresource")
        .unwrap_or_else(|_| {
            info!("No compiled resources found, using defaults");
        });

    // Create and run the application
    let app = StoreApplication::new(APP_ID);
    app.run()
}
