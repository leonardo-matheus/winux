// Winux Mobile Studio - Main Window
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation, Paned};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::pages;
use crate::ui;

pub fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    let stack = ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    // Projects Page - Project Management
    let projects_page = pages::projects::create_page();
    stack.add_titled(&projects_page, Some("projects"), "Projetos")
        .set_icon_name(Some("folder-templates-symbolic"));

    // Android Page - Android Builds
    let android_page = pages::android::create_page();
    stack.add_titled(&android_page, Some("android"), "Android")
        .set_icon_name(Some("phone-symbolic"));

    // iOS Page - iOS/Swift Builds
    let ios_page = pages::ios::create_page();
    stack.add_titled(&ios_page, Some("ios"), "iOS")
        .set_icon_name(Some("phone-apple-iphone-symbolic"));

    // Flutter Page - Flutter Projects
    let flutter_page = pages::flutter::create_page();
    stack.add_titled(&flutter_page, Some("flutter"), "Flutter")
        .set_icon_name(Some("applications-science-symbolic"));

    // React Native Page - React Native Projects
    let react_native_page = pages::react_native::create_page();
    stack.add_titled(&react_native_page, Some("react_native"), "React Native")
        .set_icon_name(Some("applications-internet-symbolic"));

    // Emulator Page - Manage Emulators
    let emulator_page = pages::emulator::create_page();
    stack.add_titled(&emulator_page, Some("emulator"), "Emuladores")
        .set_icon_name(Some("computer-symbolic"));

    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    // Add device list button
    let devices_btn = gtk4::Button::from_icon_name("phone-symbolic");
    devices_btn.set_tooltip_text(Some("Dispositivos conectados"));
    header.pack_start(&devices_btn);

    // Add refresh button
    let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some("Atualizar"));
    header.pack_end(&refresh_btn);

    // Add settings button
    let settings_btn = gtk4::Button::from_icon_name("emblem-system-symbolic");
    settings_btn.set_tooltip_text(Some("Configuracoes"));
    header.pack_end(&settings_btn);

    // Main layout with paned view (content + terminal)
    let paned = Paned::new(Orientation::Vertical);
    paned.set_wide_handle(true);

    // Top: Main content
    let content_box = Box::new(Orientation::Horizontal, 0);

    // Device list sidebar
    let device_sidebar = ui::device_list::create_device_sidebar();
    content_box.append(&device_sidebar);

    // Separator
    let separator = gtk4::Separator::new(Orientation::Vertical);
    content_box.append(&separator);

    // Main stack content
    content_box.append(&stack);

    paned.set_start_child(Some(&content_box));
    paned.set_resize_start_child(true);
    paned.set_shrink_start_child(false);

    // Bottom: Build output and terminal
    let bottom_panel = ui::build_output::create_build_panel();
    paned.set_end_child(Some(&bottom_panel));
    paned.set_resize_end_child(false);
    paned.set_shrink_end_child(true);
    paned.set_position(500);

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&paned);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Mobile Studio")
        .default_width(1400)
        .default_height(900)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));

    // Apply dark theme preference
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    window.present();
}
