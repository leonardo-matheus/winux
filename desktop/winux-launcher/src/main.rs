//! Winux Launcher - Spotlight/Alfred-style application launcher
//!
//! A fast, elegant launcher with support for:
//! - Application search
//! - File search
//! - Calculator
//! - Unit conversions
//! - Web search
//! - System commands

mod config;
mod search;
mod ui;
mod window;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use libadwaita as adw;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use window::LauncherWindow;

/// Application ID
const APP_ID: &str = "org.winux.Launcher";

fn main() -> glib::ExitCode {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Winux Launcher");

    // Load configuration
    let config = Arc::new(Config::load().unwrap_or_default());

    // Create GTK application
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    // Store config in application
    app.set_resource_base_path(Some("/org/winux/launcher"));

    let config_clone = config.clone();
    app.connect_startup(move |app| {
        setup_css();
        setup_actions(app);
    });

    let config_clone = config.clone();
    app.connect_activate(move |app| {
        // Check if window already exists
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }

        // Create new launcher window
        let window = LauncherWindow::new(app, config_clone.clone());
        window.present();
    });

    // Run application
    app.run()
}

/// Setup CSS styling
fn setup_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Setup application actions
fn setup_actions(app: &adw::Application) {
    // Quit action
    let quit_action = gio::SimpleAction::new("quit", None);
    quit_action.connect_activate(glib::clone!(
        #[weak]
        app,
        move |_, _| {
            app.quit();
        }
    ));
    app.add_action(&quit_action);

    // Toggle visibility action
    let toggle_action = gio::SimpleAction::new("toggle", None);
    toggle_action.connect_activate(glib::clone!(
        #[weak]
        app,
        move |_, _| {
            if let Some(window) = app.active_window() {
                if window.is_visible() {
                    window.hide();
                } else {
                    window.present();
                }
            }
        }
    ));
    app.add_action(&toggle_action);

    // Set keyboard shortcuts
    app.set_accels_for_action("app.quit", &["<Control>q"]);
    app.set_accels_for_action("app.toggle", &["<Super>space", "<Control>space"]);
}
