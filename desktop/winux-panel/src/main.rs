//! Winux Panel - Main taskbar and start menu component for Winux OS
//!
//! This is the primary shell component that provides:
//! - Taskbar with running applications
//! - Start menu with app launcher
//! - System tray with status indicators
//! - Clock widget

mod config;
mod start_menu;
mod system_tray;
mod taskbar;
mod widgets;

use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Application, CssProvider};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use libadwaita as adw;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use config::PanelConfig;
use taskbar::Taskbar;

/// Application ID for the Winux Panel
const APP_ID: &str = "org.winux.Panel";

/// Main application state
pub struct PanelState {
    pub config: PanelConfig,
    pub start_menu_visible: bool,
}

impl PanelState {
    pub fn new(config: PanelConfig) -> Self {
        Self {
            config,
            start_menu_visible: false,
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Winux Panel v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = PanelConfig::load().unwrap_or_else(|e| {
        error!("Failed to load config: {}, using defaults", e);
        PanelConfig::default()
    });

    // Create shared state
    let state = Arc::new(RwLock::new(PanelState::new(config)));

    // Initialize GTK
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    // Clone state for use in closures
    let state_clone = Arc::clone(&state);

    app.connect_startup(move |app| {
        // Initialize libadwaita
        adw::init().expect("Failed to initialize libadwaita");

        // Load custom CSS
        load_css();

        debug!("GTK application startup complete");
    });

    let state_activate = Arc::clone(&state);

    app.connect_activate(move |app| {
        info!("Activating Winux Panel");

        // Create main panel window
        let window = create_panel_window(app, Arc::clone(&state_activate));

        // Show the window
        window.present();

        info!("Winux Panel window presented");
    });

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    let exit_code = app.run_with_args(&args);

    info!("Winux Panel exiting with code: {:?}", exit_code);

    Ok(())
}

/// Load custom CSS styles for the panel
fn load_css() {
    let provider = CssProvider::new();

    // Panel CSS styles
    let css = r#"
        /* Winux Panel Styles */
        .winux-panel {
            background-color: alpha(@window_bg_color, 0.85);
            border-top: 1px solid alpha(@borders, 0.5);
        }

        .winux-taskbar {
            padding: 2px 8px;
        }

        .start-button {
            min-width: 48px;
            min-height: 40px;
            padding: 4px 12px;
            border-radius: 4px;
            background: transparent;
            transition: background-color 200ms ease;
        }

        .start-button:hover {
            background-color: alpha(@accent_bg_color, 0.3);
        }

        .start-button:active,
        .start-button:checked {
            background-color: alpha(@accent_bg_color, 0.5);
        }

        .start-button image {
            -gtk-icon-size: 24px;
        }

        .taskbar-button {
            min-width: 44px;
            min-height: 40px;
            padding: 4px;
            border-radius: 4px;
            background: transparent;
            transition: all 200ms ease;
        }

        .taskbar-button:hover {
            background-color: alpha(@window_fg_color, 0.1);
        }

        .taskbar-button.active {
            background-color: alpha(@accent_bg_color, 0.3);
            border-bottom: 3px solid @accent_bg_color;
        }

        .taskbar-button.pinned {
            opacity: 0.7;
        }

        .taskbar-button.pinned:hover {
            opacity: 1.0;
        }

        .system-tray {
            padding: 0 8px;
        }

        .system-tray-icon {
            min-width: 24px;
            min-height: 24px;
            padding: 4px;
            border-radius: 4px;
            transition: background-color 200ms ease;
        }

        .system-tray-icon:hover {
            background-color: alpha(@window_fg_color, 0.1);
        }

        .clock-widget {
            padding: 4px 12px;
            font-weight: 500;
        }

        .clock-time {
            font-size: 13px;
        }

        .clock-date {
            font-size: 11px;
            opacity: 0.8;
        }

        /* Start Menu Styles */
        .start-menu {
            background-color: alpha(@window_bg_color, 0.95);
            border-radius: 12px 12px 0 0;
            border: 1px solid alpha(@borders, 0.3);
            box-shadow: 0 -4px 24px alpha(black, 0.2);
        }

        .start-menu-search {
            margin: 16px;
            border-radius: 8px;
        }

        .start-menu-search entry {
            min-height: 40px;
            padding: 0 16px;
            border-radius: 8px;
            background-color: alpha(@view_bg_color, 0.5);
        }

        .pinned-apps-grid {
            padding: 8px 16px;
        }

        .pinned-app-button {
            min-width: 80px;
            min-height: 80px;
            padding: 8px;
            border-radius: 8px;
            background: transparent;
            transition: background-color 200ms ease;
        }

        .pinned-app-button:hover {
            background-color: alpha(@window_fg_color, 0.1);
        }

        .pinned-app-button image {
            -gtk-icon-size: 48px;
        }

        .pinned-app-button label {
            font-size: 11px;
            margin-top: 4px;
        }

        .all-apps-list {
            padding: 8px 0;
        }

        .app-list-item {
            padding: 8px 16px;
            border-radius: 0;
        }

        .app-list-item:hover {
            background-color: alpha(@window_fg_color, 0.05);
        }

        .app-list-item image {
            -gtk-icon-size: 32px;
            margin-right: 12px;
        }

        .user-area {
            padding: 12px 16px;
            border-top: 1px solid alpha(@borders, 0.2);
        }

        .power-options {
            padding: 8px 16px;
        }

        .power-button {
            min-width: 40px;
            min-height: 40px;
            border-radius: 8px;
            padding: 8px;
        }

        .power-button:hover {
            background-color: alpha(@error_bg_color, 0.2);
        }
    "#;

    provider.load_from_string(css);

    // Add provider to the default display
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        debug!("CSS styles loaded");
    } else {
        error!("Failed to get default display for CSS provider");
    }
}

/// Create the main panel window with layer shell configuration
fn create_panel_window(app: &Application, state: Arc<RwLock<PanelState>>) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Winux Panel")
        .build();

    // Configure layer shell for Wayland
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.auto_exclusive_zone_enable();

    // Anchor to bottom edge, spanning full width
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // Set panel height
    window.set_height_request(48);

    // Add panel CSS class
    window.add_css_class("winux-panel");

    // Create the taskbar
    let taskbar = Taskbar::new(state);
    window.set_child(Some(taskbar.widget()));

    window
}
