//! Winux Shell - Desktop shell with dock and app launcher
//! 
//! This is the main desktop shell component providing:
//! - Application dock for quick access to favorite apps
//! - Application launcher/overview
//! - Desktop background management

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Orientation};
use libadwaita as adw;
use adw::prelude::*;

const APP_ID: &str = "org.winux.Shell";

fn main() -> gtk::glib::ExitCode {
    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // Create the main shell window (covers the desktop)
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Shell")
        .default_width(1920)
        .default_height(1080)
        .decorated(false)
        .build();

    // Main container
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    // Desktop area (spacer)
    let desktop_area = Box::new(Orientation::Vertical, 0);
    desktop_area.set_vexpand(true);
    main_box.append(&desktop_area);

    // Dock at the bottom
    let dock = build_dock();
    main_box.append(&dock);

    window.set_child(Some(&main_box));
    window.present();

    println!("[winux-shell] Desktop shell initialized");
    println!("[winux-shell] Dock loaded with {} items", 5);
}

fn build_dock() -> Box {
    let dock = Box::new(Orientation::Horizontal, 8);
    dock.set_halign(gtk::Align::Center);
    dock.set_margin_bottom(12);
    dock.set_margin_top(8);
    dock.add_css_class("dock");

    // Add some placeholder dock items
    let apps = vec![
        ("Files", "system-file-manager"),
        ("Terminal", "utilities-terminal"),
        ("Browser", "web-browser"),
        ("Settings", "preferences-system"),
        ("Apps", "view-app-grid-symbolic"),
    ];

    for (name, icon) in apps {
        let button = Button::builder()
            .icon_name(icon)
            .tooltip_text(name)
            .build();
        
        button.add_css_class("dock-item");
        button.set_size_request(48, 48);
        
        let app_name = name.to_string();
        button.connect_clicked(move |_| {
            println!("[winux-shell] Dock item clicked: {}", app_name);
            if app_name == "Apps" {
                println!("[winux-shell] Opening app launcher...");
            }
        });
        
        dock.append(&button);
    }

    dock
}
