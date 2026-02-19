// Winux Mobile Studio - React Native Projects Page
// Copyright (c) 2026 Winux OS Project
//
// React Native project management:
// - Create new React Native projects
// - Build for Android/iOS
// - Metro bundler control
// - NPM/Yarn dependencies

use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, Entry, Frame, Grid, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, Image, DropDown, StringList,
};
use libadwaita as adw;
use adw::prelude::*;

pub fn create_page() -> Box {
    let page = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Header
    let header = create_header();
    page.append(&header);

    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(20)
        .margin_start(20)
        .margin_end(20)
        .margin_top(10)
        .margin_bottom(20)
        .build();

    // Environment status
    let env_section = create_environment_section();
    content.append(&env_section);

    // Metro bundler section
    let metro_section = create_metro_section();
    content.append(&metro_section);

    // Build section
    let build_section = create_build_section();
    content.append(&build_section);

    // Dependencies section
    let deps_section = create_dependencies_section();
    content.append(&deps_section);

    scrolled.set_child(Some(&content));
    page.append(&scrolled);

    page
}

fn create_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_top(15)
        .margin_bottom(15)
        .build();

    let icon = Image::from_icon_name("applications-internet-symbolic");
    icon.set_pixel_size(24);
    header.append(&icon);

    let title = Label::builder()
        .label("React Native")
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Version info
    let version_label = Label::builder()
        .label("React Native 0.73.2 | React 18.2.0")
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&version_label);

    header
}

fn create_environment_section() -> Frame {
    let frame = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    let title = Label::builder()
        .label("Ambiente de Desenvolvimento")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    content.append(&title);

    let list = ListBox::builder()
        .css_classes(vec!["boxed-list"])
        .selection_mode(gtk4::SelectionMode::None)
        .margin_top(10)
        .build();

    let tools = vec![
        ("Node.js", "20.11.0", true),
        ("npm", "10.2.4", true),
        ("Yarn", "1.22.21", true),
        ("Watchman", "4.9.0", true),
        ("Android SDK", "34", true),
        ("JDK", "17.0.9", true),
    ];

    for (name, version, ok) in tools {
        let row = create_tool_status_row(name, version, ok);
        list.append(&row);
    }

    content.append(&list);

    frame.set_child(Some(&content));
    frame
}

fn create_tool_status_row(name: &str, version: &str, ok: bool) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(10)
        .margin_end(10)
        .build();

    let icon = if ok {
        Image::from_icon_name("emblem-ok-symbolic")
    } else {
        Image::from_icon_name("dialog-warning-symbolic")
    };
    row_box.append(&icon);

    let name_label = Label::builder()
        .label(name)
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();
    row_box.append(&name_label);

    let version_label = Label::builder()
        .label(version)
        .css_classes(vec!["dim-label", "monospace"])
        .build();
    row_box.append(&version_label);

    ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .build()
}

fn create_metro_section() -> Frame {
    let frame = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let title = Label::builder()
        .label("Metro Bundler")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let status_label = Label::builder()
        .label("Parado")
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&status_label);

    content.append(&header);

    let description = Label::builder()
        .label("O Metro bundler serve o JavaScript da sua aplicacao durante o desenvolvimento.")
        .wrap(true)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label"])
        .margin_top(5)
        .build();
    content.append(&description);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .margin_top(15)
        .build();

    // Port
    let port_label = Label::builder()
        .label("Porta:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&port_label, 0, 0, 1, 1);

    let port_entry = Entry::builder()
        .text("8081")
        .width_request(100)
        .build();
    grid.attach(&port_entry, 1, 0, 1, 1);

    // Options
    let reset_cache = CheckButton::with_label("Reset cache on start");
    grid.attach(&reset_cache, 1, 1, 2, 1);

    content.append(&grid);

    // Controls
    let controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(15)
        .build();

    let start_btn = Button::builder()
        .label("Iniciar Metro")
        .css_classes(vec!["suggested-action"])
        .build();
    controls.append(&start_btn);

    let reload_btn = Button::builder()
        .label("Reload")
        .sensitive(false)
        .build();
    controls.append(&reload_btn);

    let open_debugger = Button::builder()
        .label("Abrir Debugger")
        .css_classes(vec!["flat"])
        .sensitive(false)
        .build();
    controls.append(&open_debugger);

    content.append(&controls);

    frame.set_child(Some(&content));
    frame
}

fn create_build_section() -> Frame {
    let frame = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    let title = Label::builder()
        .label("Build & Run")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    content.append(&title);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .margin_top(10)
        .build();

    // Platform
    let platform_label = Label::builder()
        .label("Plataforma:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&platform_label, 0, 0, 1, 1);

    let platform_list = StringList::new(&["Android", "iOS"]);
    let platform = DropDown::new(Some(platform_list), gtk4::Expression::NONE);
    platform.set_hexpand(true);
    grid.attach(&platform, 1, 0, 2, 1);

    // Variant
    let variant_label = Label::builder()
        .label("Variante:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&variant_label, 0, 1, 1, 1);

    let variant_list = StringList::new(&["Debug", "Release"]);
    let variant = DropDown::new(Some(variant_list), gtk4::Expression::NONE);
    variant.set_hexpand(true);
    grid.attach(&variant, 1, 1, 2, 1);

    // Device
    let device_label = Label::builder()
        .label("Dispositivo:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&device_label, 0, 2, 1, 1);

    let device_list = StringList::new(&[
        "Pixel 7 (emulator)",
        "Samsung Galaxy (USB)",
    ]);
    let device = DropDown::new(Some(device_list), gtk4::Expression::NONE);
    device.set_hexpand(true);
    grid.attach(&device, 1, 2, 2, 1);

    content.append(&grid);

    // Actions
    let actions = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(15)
        .build();

    let run_btn = Button::builder()
        .css_classes(vec!["suggested-action", "pill"])
        .build();
    let run_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    run_content.append(&Image::from_icon_name("media-playback-start-symbolic"));
    run_content.append(&Label::new(Some("Run")));
    run_btn.set_child(Some(&run_content));
    actions.append(&run_btn);

    let build_btn = Button::builder()
        .label("Build Only")
        .css_classes(vec!["pill"])
        .build();
    actions.append(&build_btn);

    let bundle_btn = Button::builder()
        .label("Bundle JS")
        .css_classes(vec!["pill"])
        .build();
    actions.append(&bundle_btn);

    let clean_btn = Button::builder()
        .label("Clean")
        .css_classes(vec!["pill"])
        .build();
    actions.append(&clean_btn);

    content.append(&actions);

    frame.set_child(Some(&content));
    frame
}

fn create_dependencies_section() -> Frame {
    let frame = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let title = Label::builder()
        .label("Dependencias (package.json)")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let add_btn = Button::builder()
        .label("Adicionar")
        .css_classes(vec!["flat"])
        .build();
    header.append(&add_btn);

    let install_btn = Button::builder()
        .label("npm install")
        .css_classes(vec!["flat"])
        .build();
    header.append(&install_btn);

    content.append(&header);

    let list = ListBox::builder()
        .css_classes(vec!["boxed-list"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    let deps = vec![
        ("react", "18.2.0", false),
        ("react-native", "0.73.2", false),
        ("@react-navigation/native", "^6.1.9", true),
        ("react-native-reanimated", "^3.6.1", true),
        ("axios", "^1.6.5", false),
    ];

    for (name, version, outdated) in deps {
        let row = create_dependency_row(name, version, outdated);
        list.append(&row);
    }

    content.append(&list);

    // Scripts
    let scripts_title = Label::builder()
        .label("Scripts NPM")
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .margin_top(15)
        .build();
    content.append(&scripts_title);

    let scripts_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(5)
        .build();

    let scripts = vec!["start", "test", "lint", "android", "ios"];
    for script in scripts {
        let btn = Button::builder()
            .label(&format!("npm run {}", script))
            .css_classes(vec!["flat", "caption"])
            .build();
        scripts_box.append(&btn);
    }

    content.append(&scripts_box);

    frame.set_child(Some(&content));
    frame
}

fn create_dependency_row(name: &str, version: &str, outdated: bool) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let name_label = Label::builder()
        .label(name)
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();
    row_box.append(&name_label);

    let version_label = Label::builder()
        .label(version)
        .css_classes(vec!["dim-label", "monospace"])
        .build();
    row_box.append(&version_label);

    if outdated {
        let update_icon = Image::from_icon_name("software-update-available-symbolic");
        update_icon.set_tooltip_text(Some("Atualizacao disponivel"));
        row_box.append(&update_icon);
    }

    ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .build()
}
