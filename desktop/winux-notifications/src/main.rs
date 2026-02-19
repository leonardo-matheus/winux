//! Winux Notifications - Modern D-Bus notification daemon
//!
//! A modern notification daemon for Winux OS that implements the
//! org.freedesktop.Notifications specification with advanced features:
//!
//! - Modern popup notifications with animations
//! - Notification center for history
//! - Do Not Disturb mode
//! - Urgency levels and persistent notifications
//! - Sound notifications
//! - Progress bar support

mod config;
mod daemon;
mod history;
mod notification;
mod ui;

use anyhow::Result;
use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::NotificationConfig;
use crate::daemon::NotificationDaemon;
use crate::ui::NotificationApp;

const APP_ID: &str = "org.winux.Notifications";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .init();

    info!("Starting Winux Notification Daemon");

    // Load configuration
    let config = NotificationConfig::load().unwrap_or_default();
    info!("Configuration loaded");

    // Initialize GTK
    adw::init().expect("Failed to initialize libadwaita");

    // Create the application
    let app = NotificationApp::new(APP_ID, config);

    // Run the application
    let exit_code = app.run();

    info!("Notification daemon shutting down");
    std::process::exit(exit_code.into());
}
