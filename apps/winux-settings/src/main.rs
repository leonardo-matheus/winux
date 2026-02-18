//! Winux Settings - System Settings Application for Winux OS
//!
//! A comprehensive settings application built with GTK4/Adwaita featuring:
//! - Display settings
//! - Sound configuration
//! - Network management
//! - Appearance customization
//! - Gaming optimizations

mod pages;

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use pages::{
    AppearancePage, DisplayPage, GamingPage, NetworkPage, SoundPage,
};

/// Application ID for Winux Settings
const APP_ID: &str = "org.winux.settings";

fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Winux Settings");

    // Initialize GTK
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);

    Ok(())
}

fn build_ui(app: &adw::Application) {
    // Create main window
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Settings")
        .default_width(900)
        .default_height(700)
        .build();

    // Create navigation split view
    let split_view = adw::NavigationSplitView::new();

    // Create sidebar
    let sidebar = create_sidebar();
    split_view.set_sidebar(Some(&sidebar));

    // Create content stack
    let content_stack = gtk4::Stack::new();
    content_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
    content_stack.set_transition_duration(200);

    // Add pages
    let display_page = DisplayPage::new();
    content_stack.add_titled(display_page.widget(), Some("display"), "Display");

    let sound_page = SoundPage::new();
    content_stack.add_titled(sound_page.widget(), Some("sound"), "Sound");

    let network_page = NetworkPage::new();
    content_stack.add_titled(network_page.widget(), Some("network"), "Network");

    let appearance_page = AppearancePage::new();
    content_stack.add_titled(appearance_page.widget(), Some("appearance"), "Appearance");

    let gaming_page = GamingPage::new();
    content_stack.add_titled(gaming_page.widget(), Some("gaming"), "Gaming");

    // Create content navigation page
    let content_page = adw::NavigationPage::builder()
        .title("Settings")
        .child(&content_stack)
        .build();

    split_view.set_content(Some(&content_page));

    // Connect sidebar selection to stack
    if let Some(list_box) = find_list_box(&sidebar) {
        let stack = content_stack.clone();
        list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let name = row.widget_name();
                stack.set_visible_child_name(&name);
            }
        });

        // Select first row by default
        if let Some(first_row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&first_row));
        }
    }

    // Create toolbar view with header
    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    // Search button
    let search_btn = gtk4::ToggleButton::new();
    search_btn.set_icon_name("system-search-symbolic");
    search_btn.set_tooltip_text(Some("Search settings"));
    header.pack_end(&search_btn);

    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&split_view));

    window.set_content(Some(&toolbar_view));
    window.present();
}

fn create_sidebar() -> adw::NavigationPage {
    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    list_box.add_css_class("navigation-sidebar");

    // Settings categories
    let categories = [
        ("display", "video-display-symbolic", "Display", "Screen resolution, refresh rate, HDR"),
        ("sound", "audio-speakers-symbolic", "Sound", "Volume, output devices, effects"),
        ("network", "network-wireless-symbolic", "Network", "Wi-Fi, Ethernet, VPN"),
        ("appearance", "applications-graphics-symbolic", "Appearance", "Themes, colors, fonts"),
        ("gaming", "input-gaming-symbolic", "Gaming", "Performance modes, optimizations"),
    ];

    for (id, icon, title, subtitle) in categories {
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();

        row.add_prefix(&gtk4::Image::from_icon_name(icon));
        row.set_activatable(true);
        row.set_widget_name(id);

        list_box.append(&row);
    }

    // Wrap in scrolled window
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.set_child(Some(&list_box));

    // Create navigation page
    adw::NavigationPage::builder()
        .title("Settings")
        .child(&scrolled)
        .build()
}

fn find_list_box(page: &adw::NavigationPage) -> Option<gtk4::ListBox> {
    let child = page.child()?;
    let scrolled = child.downcast_ref::<gtk4::ScrolledWindow>()?;
    let viewport_child = scrolled.child()?;

    // Try direct cast first
    if let Some(list_box) = viewport_child.downcast_ref::<gtk4::ListBox>() {
        return Some(list_box.clone());
    }

    // Try viewport
    if let Some(viewport) = viewport_child.downcast_ref::<gtk4::Viewport>() {
        if let Some(child) = viewport.child() {
            return child.downcast::<gtk4::ListBox>().ok();
        }
    }

    None
}
