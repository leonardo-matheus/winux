// Winux Mobile Studio - Flutter Projects Page
// Copyright (c) 2026 Winux OS Project
//
// Flutter project management:
// - Create new Flutter projects
// - Build for Android/iOS/Web/Desktop
// - Hot reload management
// - Pub dependencies

use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, Entry, Frame, Grid, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, Image, ProgressBar, DropDown, StringList,
};
use libadwaita as adw;
use adw::prelude::*;

#[derive(Clone, Debug)]
pub struct FlutterProject {
    pub name: String,
    pub path: String,
    pub flutter_version: String,
    pub platforms: Vec<FlutterPlatform>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlutterPlatform {
    Android,
    IOS,
    Web,
    Linux,
    Windows,
    MacOS,
}

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

    // Flutter SDK status
    let sdk_section = create_sdk_section();
    content.append(&sdk_section);

    // Build section
    let build_section = create_build_section();
    content.append(&build_section);

    // Run section
    let run_section = create_run_section();
    content.append(&run_section);

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

    let icon = Image::from_icon_name("applications-science-symbolic");
    icon.set_pixel_size(24);
    header.append(&icon);

    let title = Label::builder()
        .label("Flutter")
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Flutter version
    let version_label = Label::builder()
        .label("Flutter 3.19.0 | Dart 3.3.0")
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&version_label);

    // Flutter doctor button
    let doctor_btn = Button::builder()
        .label("Flutter Doctor")
        .css_classes(vec!["flat"])
        .build();
    header.append(&doctor_btn);

    header
}

fn create_sdk_section() -> Frame {
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
        .label("Flutter SDK")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let channel_list = StringList::new(&["stable", "beta", "dev", "master"]);
    let channel = DropDown::new(Some(channel_list), gtk4::Expression::NONE);
    header.append(&channel);

    let upgrade_btn = Button::builder()
        .label("Atualizar SDK")
        .css_classes(vec!["flat"])
        .build();
    header.append(&upgrade_btn);

    content.append(&header);

    // SDK Info grid
    let grid = Grid::builder()
        .column_spacing(20)
        .row_spacing(8)
        .margin_top(10)
        .build();

    let labels = vec![
        ("Flutter:", "3.19.0 (stable)"),
        ("Dart:", "3.3.0"),
        ("DevTools:", "2.31.0"),
        ("Caminho:", "~/.local/flutter"),
    ];

    for (i, (label, value)) in labels.iter().enumerate() {
        let label_widget = Label::builder()
            .label(*label)
            .css_classes(vec!["dim-label"])
            .halign(gtk4::Align::End)
            .build();
        grid.attach(&label_widget, 0, i as i32, 1, 1);

        let value_widget = Label::builder()
            .label(*value)
            .halign(gtk4::Align::Start)
            .build();
        grid.attach(&value_widget, 1, i as i32, 1, 1);
    }

    content.append(&grid);

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
        .label("Build")
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

    let platform_list = StringList::new(&["Android APK", "Android AAB", "iOS", "Web", "Linux", "Windows"]);
    let platform = DropDown::new(Some(platform_list), gtk4::Expression::NONE);
    platform.set_hexpand(true);
    grid.attach(&platform, 1, 0, 2, 1);

    // Build mode
    let mode_label = Label::builder()
        .label("Modo:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&mode_label, 0, 1, 1, 1);

    let mode_list = StringList::new(&["Debug", "Profile", "Release"]);
    let mode = DropDown::new(Some(mode_list), gtk4::Expression::NONE);
    mode.set_hexpand(true);
    grid.attach(&mode, 1, 1, 2, 1);

    // Flavor
    let flavor_label = Label::builder()
        .label("Flavor:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&flavor_label, 0, 2, 1, 1);

    let flavor_entry = Entry::builder()
        .placeholder_text("(opcional)")
        .hexpand(true)
        .build();
    grid.attach(&flavor_entry, 1, 2, 2, 1);

    // Options
    let options_label = Label::builder()
        .label("Opcoes:")
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Start)
        .build();
    grid.attach(&options_label, 0, 3, 1, 1);

    let options_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .build();

    let tree_shake = CheckButton::with_label("Tree shake icons");
    tree_shake.set_active(true);
    options_box.append(&tree_shake);

    let obfuscate = CheckButton::with_label("Obfuscate Dart code");
    options_box.append(&obfuscate);

    let split_debug = CheckButton::with_label("Split debug info");
    options_box.append(&split_debug);

    grid.attach(&options_box, 1, 3, 2, 1);

    content.append(&grid);

    // Build button
    let actions = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(15)
        .build();

    let build_btn = Button::builder()
        .label("Build")
        .css_classes(vec!["suggested-action", "pill"])
        .build();
    actions.append(&build_btn);

    let clean_btn = Button::builder()
        .label("Clean")
        .css_classes(vec!["pill"])
        .build();
    actions.append(&clean_btn);

    let analyze_btn = Button::builder()
        .label("Analyze")
        .css_classes(vec!["pill"])
        .build();
    actions.append(&analyze_btn);

    content.append(&actions);

    frame.set_child(Some(&content));
    frame
}

fn create_run_section() -> Frame {
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
        .label("Run & Debug")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let hot_reload_label = Label::builder()
        .label("Hot Reload: Ativo")
        .css_classes(vec!["success"])
        .build();
    header.append(&hot_reload_label);

    content.append(&header);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .margin_top(10)
        .build();

    // Device
    let device_label = Label::builder()
        .label("Dispositivo:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&device_label, 0, 0, 1, 1);

    let device_list = StringList::new(&[
        "Pixel 7 (emulator)",
        "Samsung Galaxy S23 (USB)",
        "Chrome (web)",
        "Linux (desktop)",
    ]);
    let device = DropDown::new(Some(device_list), gtk4::Expression::NONE);
    device.set_hexpand(true);
    grid.attach(&device, 1, 0, 2, 1);

    content.append(&grid);

    // Run controls
    let controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(15)
        .build();

    let run_btn = Button::builder()
        .css_classes(vec!["suggested-action", "circular"])
        .build();
    run_btn.set_child(Some(&Image::from_icon_name("media-playback-start-symbolic")));
    run_btn.set_tooltip_text(Some("Run"));
    controls.append(&run_btn);

    let hot_reload_btn = Button::builder()
        .css_classes(vec!["circular"])
        .build();
    hot_reload_btn.set_child(Some(&Image::from_icon_name("view-refresh-symbolic")));
    hot_reload_btn.set_tooltip_text(Some("Hot Reload (r)"));
    controls.append(&hot_reload_btn);

    let hot_restart_btn = Button::builder()
        .css_classes(vec!["circular"])
        .build();
    hot_restart_btn.set_child(Some(&Image::from_icon_name("system-reboot-symbolic")));
    hot_restart_btn.set_tooltip_text(Some("Hot Restart (R)"));
    controls.append(&hot_restart_btn);

    let stop_btn = Button::builder()
        .css_classes(vec!["destructive-action", "circular"])
        .build();
    stop_btn.set_child(Some(&Image::from_icon_name("media-playback-stop-symbolic")));
    stop_btn.set_tooltip_text(Some("Stop"));
    controls.append(&stop_btn);

    let spacer = Box::builder().hexpand(true).build();
    controls.append(&spacer);

    let devtools_btn = Button::builder()
        .label("Open DevTools")
        .css_classes(vec!["flat"])
        .build();
    controls.append(&devtools_btn);

    content.append(&controls);

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
        .label("Dependencias (pubspec.yaml)")
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

    let update_btn = Button::builder()
        .label("Pub Get")
        .css_classes(vec!["flat"])
        .build();
    header.append(&update_btn);

    content.append(&header);

    let list = ListBox::builder()
        .css_classes(vec!["boxed-list"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    let deps = vec![
        ("flutter_bloc", "^8.1.3", true),
        ("dio", "^5.4.0", true),
        ("shared_preferences", "^2.2.2", true),
        ("cached_network_image", "^3.3.1", false),
        ("flutter_test", "sdk: flutter", true),
    ];

    for (name, version, up_to_date) in deps {
        let row = create_dependency_row(name, version, up_to_date);
        list.append(&row);
    }

    content.append(&list);

    frame.set_child(Some(&content));
    frame
}

fn create_dependency_row(name: &str, version: &str, up_to_date: bool) -> ListBoxRow {
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

    if !up_to_date {
        let update_icon = Image::from_icon_name("software-update-available-symbolic");
        update_icon.set_tooltip_text(Some("Atualizacao disponivel"));
        row_box.append(&update_icon);
    }

    ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .build()
}
