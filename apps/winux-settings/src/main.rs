// Winux Settings - System configuration center
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Label, ListBox, ListBoxRow, Orientation, HeaderBar, Stack, StackSidebar, Separator};
use libadwaita as adw;
use adw::prelude::*;

const APP_ID: &str = "org.winux.settings";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();
    header.set_title_widget(Some(&Label::new(Some("Settings"))));

    let stack = Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);

    // WiFi page
    let wifi_page = create_page("Network", &[
        ("Wi-Fi", "Connect to wireless networks"),
        ("Ethernet", "Wired connection settings"),
        ("VPN", "Virtual private networks"),
        ("Proxy", "Network proxy settings"),
    ]);
    stack.add_titled(&wifi_page, Some("network"), "Network");

    // Bluetooth page
    let bt_page = create_page("Bluetooth", &[
        ("Devices", "Paired devices"),
        ("Visibility", "Device visibility settings"),
    ]);
    stack.add_titled(&bt_page, Some("bluetooth"), "Bluetooth");

    // Appearance page
    let appearance_page = create_page("Appearance", &[
        ("Theme", "Light or dark mode"),
        ("Wallpaper", "Desktop background"),
        ("Accent Color", "System accent color"),
        ("Icons", "Icon theme"),
    ]);
    stack.add_titled(&appearance_page, Some("appearance"), "Appearance");

    // Sound page
    let sound_page = create_page("Sound", &[
        ("Output", "Speakers and headphones"),
        ("Input", "Microphones"),
        ("Volume", "System volume levels"),
        ("Alerts", "System sounds"),
    ]);
    stack.add_titled(&sound_page, Some("sound"), "Sound");

    // Display page
    let display_page = create_page("Displays", &[
        ("Resolution", "Screen resolution"),
        ("Refresh Rate", "Monitor refresh rate"),
        ("Night Light", "Reduce blue light"),
        ("Scale", "Display scaling"),
    ]);
    stack.add_titled(&display_page, Some("displays"), "Displays");

    // About page
    let about_page = create_page("About", &[
        ("Device Name", "winux-desktop"),
        ("OS", "Winux OS 1.0 Aurora"),
        ("Kernel", "Linux 6.8"),
        ("Desktop", "Winux Shell"),
    ]);
    stack.add_titled(&about_page, Some("about"), "About");

    let sidebar = StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.set_width_request(200);

    let sep = Separator::new(Orientation::Vertical);

    let main_box = Box::new(Orientation::Horizontal, 0);
    main_box.append(&sidebar);
    main_box.append(&sep);
    main_box.append(&stack);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Settings")
        .default_width(900)
        .default_height(600)
        .build();

    window.set_titlebar(Some(&header));
    window.set_child(Some(&main_box));

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    window.present();
}

fn create_page(title: &str, items: &[(&str, &str)]) -> Box {
    let page = Box::new(Orientation::Vertical, 12);
    page.set_margin_top(24);
    page.set_margin_bottom(24);
    page.set_margin_start(24);
    page.set_margin_end(24);
    page.set_hexpand(true);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("title-1");
    title_label.set_xalign(0.0);
    page.append(&title_label);

    let list = ListBox::new();
    list.add_css_class("boxed-list");

    for (name, desc) in items {
        let row = adw::ActionRow::builder()
            .title(*name)
            .subtitle(*desc)
            .activatable(true)
            .build();
        list.append(&row);
    }

    page.append(&list);
    page
}
