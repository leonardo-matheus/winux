//! Winux Terminal - GPU-accelerated Terminal Emulator for Winux OS
//!
//! A modern terminal emulator built with GTK4/Adwaita and VTE featuring:
//! - GPU acceleration
//! - Multiple tabs
//! - Customizable themes
//! - Split panes
//! - Configurable shortcuts

mod config;
mod tabs;
mod terminal;
mod themes;

use anyhow::Result;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use config::Config;
use tabs::TabManager;
use terminal::TerminalWidget;
use themes::ThemeManager;

/// Application ID for Winux Terminal
const APP_ID: &str = "org.winux.terminal";

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Winux Terminal");

    // Load configuration
    let config = Config::load().unwrap_or_default();

    // Initialize GTK
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(move |app| {
        build_ui(app, &config);
    });

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);

    Ok(())
}

fn build_ui(app: &adw::Application, config: &Config) {
    // Load theme
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme(&config.theme);

    // Create main window
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Winux Terminal")
        .default_width(config.window.width)
        .default_height(config.window.height)
        .build();

    // Create toolbar view
    let toolbar_view = adw::ToolbarView::new();

    // Create header bar
    let header = adw::HeaderBar::new();

    // New tab button
    let new_tab_btn = gtk4::Button::from_icon_name("tab-new-symbolic");
    new_tab_btn.set_tooltip_text(Some("New Tab (Ctrl+Shift+T)"));
    header.pack_start(&new_tab_btn);

    // Menu button
    let menu_btn = gtk4::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    menu_btn.set_tooltip_text(Some("Menu"));
    header.pack_end(&menu_btn);

    toolbar_view.add_top_bar(&header);

    // Create tab manager
    let tab_manager = TabManager::new(config.clone(), theme.clone());

    // Add initial tab
    tab_manager.add_tab(None);

    // Connect new tab button
    let tab_mgr_clone = tab_manager.clone();
    new_tab_btn.connect_clicked(move |_| {
        tab_mgr_clone.add_tab(None);
    });

    // Set content
    toolbar_view.set_content(Some(tab_manager.widget()));

    window.set_content(Some(&toolbar_view));

    // Apply CSS
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&generate_css(config, &theme));

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Setup keyboard shortcuts
    setup_shortcuts(&window, &tab_manager);

    window.present();
}

fn setup_shortcuts(window: &adw::ApplicationWindow, tab_manager: &TabManager) {
    let controller = gtk4::EventControllerKey::new();

    let tab_mgr = tab_manager.clone();
    controller.connect_key_pressed(move |_, key, _code, modifiers| {
        let ctrl_shift = gtk4::gdk::ModifierType::CONTROL_MASK
            | gtk4::gdk::ModifierType::SHIFT_MASK;

        if modifiers.contains(ctrl_shift) {
            match key {
                gtk4::gdk::Key::T | gtk4::gdk::Key::t => {
                    tab_mgr.add_tab(None);
                    return gtk4::glib::Propagation::Stop;
                }
                gtk4::gdk::Key::W | gtk4::gdk::Key::w => {
                    tab_mgr.close_current_tab();
                    return gtk4::glib::Propagation::Stop;
                }
                gtk4::gdk::Key::C | gtk4::gdk::Key::c => {
                    tab_mgr.copy_selection();
                    return gtk4::glib::Propagation::Stop;
                }
                gtk4::gdk::Key::V | gtk4::gdk::Key::v => {
                    tab_mgr.paste();
                    return gtk4::glib::Propagation::Stop;
                }
                _ => {}
            }
        }

        gtk4::glib::Propagation::Proceed
    });

    window.add_controller(controller);
}

fn generate_css(config: &Config, theme: &themes::Theme) -> String {
    format!(
        r#"
        .terminal-view {{
            background-color: {};
            padding: 4px;
        }}

        vte-terminal {{
            padding: 8px;
        }}

        .tab-bar {{
            background-color: {};
        }}

        .tab-button {{
            border-radius: 6px;
            padding: 4px 12px;
            margin: 4px;
        }}

        .tab-button:checked {{
            background-color: {};
        }}
        "#,
        theme.background,
        theme.background,
        theme.selection,
    )
}
