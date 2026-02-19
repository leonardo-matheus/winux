// Main window for Winux Gaming
// Steam Deck-inspired interface with large cards and gamepad navigation

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Image, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, SearchEntry, Separator, Stack, StackSidebar,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::pages;

pub fn build_ui(app: &Application) {
    // Force dark theme for gaming aesthetic
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

    let header = HeaderBar::new();

    // Create main stack for pages
    let stack = ViewStack::new();
    stack.set_vexpand(true);

    // Library page - unified game library
    let library_page = pages::library::create_library_page();
    stack.add_titled(&library_page, Some("library"), "Biblioteca")
        .set_icon_name(Some("applications-games-symbolic"));

    // Steam store integration
    let steam_page = pages::store_steam::create_steam_page();
    stack.add_titled(&steam_page, Some("steam"), "Steam")
        .set_icon_name(Some("steam-symbolic"));

    // GOG integration
    let gog_page = pages::store_gog::create_gog_page();
    stack.add_titled(&gog_page, Some("gog"), "GOG")
        .set_icon_name(Some("folder-games-symbolic"));

    // Epic Games integration
    let epic_page = pages::store_epic::create_epic_page();
    stack.add_titled(&epic_page, Some("epic"), "Epic")
        .set_icon_name(Some("gamepad-symbolic"));

    // Emulators page
    let emulators_page = pages::emulators::create_emulators_page();
    stack.add_titled(&emulators_page, Some("emulators"), "Emuladores")
        .set_icon_name(Some("media-optical-symbolic"));

    // Settings page
    let settings_page = pages::settings::create_settings_page();
    stack.add_titled(&settings_page, Some("settings"), "Config")
        .set_icon_name(Some("emblem-system-symbolic"));

    // View switcher in header
    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    // Add menu button
    let menu_button = Button::builder()
        .icon_name("open-menu-symbolic")
        .css_classes(vec!["flat"])
        .build();
    header.pack_end(&menu_button);

    // Add quick launch button
    let quick_launch = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .tooltip_text("Iniciar ultimo jogo")
        .css_classes(vec!["suggested-action", "circular"])
        .build();
    header.pack_start(&quick_launch);

    // Main content box
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&stack);

    // Create window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Gaming")
        .default_width(1400)
        .default_height(900)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));

    // Add custom CSS for gaming theme
    load_css();

    window.present();
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        r#"
        /* Winux Gaming Theme - Steam Deck inspired */

        .game-card {
            background: linear-gradient(180deg, #2a2a3a 0%, #1a1a2e 100%);
            border-radius: 12px;
            padding: 0;
            transition: all 200ms ease;
        }

        .game-card:hover {
            background: linear-gradient(180deg, #3a3a4a 0%, #2a2a3e 100%);
            transform: scale(1.02);
        }

        .game-card:focus {
            outline: 2px solid @accent_color;
            outline-offset: 2px;
        }

        .game-title {
            font-weight: bold;
            font-size: 14px;
        }

        .game-platform {
            font-size: 11px;
            opacity: 0.7;
        }

        .featured-banner {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 16px;
            min-height: 200px;
        }

        .performance-overlay {
            background: rgba(0, 0, 0, 0.8);
            border-radius: 8px;
            padding: 8px 12px;
            font-family: monospace;
        }

        .stat-value {
            font-size: 24px;
            font-weight: bold;
            color: #00ff88;
        }

        .stat-label {
            font-size: 10px;
            opacity: 0.7;
        }

        .platform-badge {
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 10px;
            font-weight: bold;
        }

        .platform-steam {
            background: #1b2838;
            color: #66c0f4;
        }

        .platform-gog {
            background: #4a1b6d;
            color: #f5f5f5;
        }

        .platform-epic {
            background: #2a2a2a;
            color: #f5f5f5;
        }

        .platform-native {
            background: #2d5016;
            color: #b8e986;
        }

        .sidebar-gaming {
            background: #0d0d15;
        }

        .play-button {
            background: linear-gradient(180deg, #00d26a 0%, #00a854 100%);
            color: white;
            font-weight: bold;
            padding: 12px 32px;
            border-radius: 8px;
            font-size: 16px;
        }

        .play-button:hover {
            background: linear-gradient(180deg, #00e676 0%, #00c853 100%);
        }

        .install-button {
            background: linear-gradient(180deg, #3498db 0%, #2980b9 100%);
            color: white;
        }
        "#,
    );

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
