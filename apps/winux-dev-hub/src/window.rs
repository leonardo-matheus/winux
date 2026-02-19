// Winux Dev Hub - Main Window
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher, NavigationSplitView};

use crate::pages;

pub fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    let stack = ViewStack::new();
    stack.set_vexpand(true);

    // Dashboard Page - Project Overview
    let dashboard_page = pages::dashboard::create_page();
    stack.add_titled(&dashboard_page, Some("dashboard"), "Projetos")
        .set_icon_name(Some("folder-templates-symbolic"));

    // Environments Page - Environment Variables & Profiles
    let environments_page = pages::environments::create_page();
    stack.add_titled(&environments_page, Some("environments"), "Ambientes")
        .set_icon_name(Some("preferences-other-symbolic"));

    // Toolchains Page - Development Tools
    let toolchains_page = pages::toolchains::create_page();
    stack.add_titled(&toolchains_page, Some("toolchains"), "Toolchains")
        .set_icon_name(Some("applications-engineering-symbolic"));

    // Containers Page - Docker/Podman
    let containers_page = pages::containers::create_page();
    stack.add_titled(&containers_page, Some("containers"), "Containers")
        .set_icon_name(Some("application-x-firmware-symbolic"));

    // Databases Page - Local DBs
    let databases_page = pages::databases::create_page();
    stack.add_titled(&databases_page, Some("databases"), "Databases")
        .set_icon_name(Some("drive-multidisk-symbolic"));

    // Services Page - System Services
    let services_page = pages::services::create_page();
    stack.add_titled(&services_page, Some("services"), "Servicos")
        .set_icon_name(Some("system-run-symbolic"));

    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    // Add refresh button
    let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some("Atualizar informacoes"));
    header.pack_end(&refresh_btn);

    // Add settings button
    let settings_btn = gtk4::Button::from_icon_name("emblem-system-symbolic");
    settings_btn.set_tooltip_text(Some("Configuracoes"));
    header.pack_end(&settings_btn);

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&stack);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Dev Hub")
        .default_width(1200)
        .default_height(800)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));

    // Apply dark theme preference
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    window.present();
}
