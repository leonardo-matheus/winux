// Winux Mobile Studio - Device List Sidebar
// Copyright (c) 2026 Winux OS Project
//
// Sidebar widget showing connected devices (Android and iOS)

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Image, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, Separator,
};
use libadwaita as adw;
use adw::prelude::*;

pub fn create_device_sidebar() -> Box {
    let sidebar = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(220)
        .css_classes(vec!["sidebar"])
        .build();

    // Header
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(15)
        .margin_end(10)
        .margin_top(15)
        .margin_bottom(10)
        .build();

    let title = Label::builder()
        .label("Dispositivos")
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    header.append(&title);

    let refresh_btn = Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(vec!["flat", "circular"])
        .tooltip_text("Atualizar")
        .build();
    header.append(&refresh_btn);

    sidebar.append(&header);

    // Android section
    let android_section = create_section("Android", get_sample_android_devices());
    sidebar.append(&android_section);

    // iOS section
    let ios_section = create_section("iOS", get_sample_ios_devices());
    sidebar.append(&ios_section);

    // Emulators section
    let emulator_section = create_section("Emuladores", get_sample_emulators());
    sidebar.append(&emulator_section);

    // Spacer
    let spacer = Box::builder()
        .vexpand(true)
        .build();
    sidebar.append(&spacer);

    // Bottom actions
    let bottom_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(15)
        .build();

    let start_emulator_btn = Button::builder()
        .css_classes(vec!["flat"])
        .build();

    let btn_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    btn_content.append(&Image::from_icon_name("media-playback-start-symbolic"));
    btn_content.append(&Label::new(Some("Iniciar Emulador")));
    start_emulator_btn.set_child(Some(&btn_content));

    bottom_box.append(&start_emulator_btn);

    let wireless_btn = Button::builder()
        .css_classes(vec!["flat"])
        .build();

    let wireless_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    wireless_content.append(&Image::from_icon_name("network-wireless-symbolic"));
    wireless_content.append(&Label::new(Some("Conectar WiFi")));
    wireless_btn.set_child(Some(&wireless_content));

    bottom_box.append(&wireless_btn);

    sidebar.append(&bottom_box);

    sidebar
}

fn create_section(title: &str, devices: Vec<DeviceInfo>) -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .margin_top(10)
        .build();

    // Section header
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(15)
        .margin_end(10)
        .build();

    let icon = match title {
        "Android" => "phone-symbolic",
        "iOS" => "phone-apple-iphone-symbolic",
        _ => "computer-symbolic",
    };

    let icon_widget = Image::from_icon_name(icon);
    icon_widget.set_pixel_size(16);
    header.append(&icon_widget);

    let title_label = Label::builder()
        .label(title)
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    header.append(&title_label);

    let count_label = Label::builder()
        .label(&format!("{}", devices.len()))
        .css_classes(vec!["dim-label", "caption"])
        .build();
    header.append(&count_label);

    section.append(&header);

    // Device list
    let list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(vec!["navigation-sidebar"])
        .margin_start(5)
        .margin_end(5)
        .build();

    for device in devices {
        let row = create_device_row(&device);
        list.append(&row);
    }

    section.append(&list);
    section
}

struct DeviceInfo {
    name: String,
    model: String,
    status: DeviceStatus,
    battery: Option<u32>,
}

#[derive(Clone, Copy)]
enum DeviceStatus {
    Connected,
    Offline,
    Running,
}

fn create_device_row(device: &DeviceInfo) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(5)
        .margin_end(5)
        .build();

    // Status indicator
    let status_color = match device.status {
        DeviceStatus::Connected => "success",
        DeviceStatus::Running => "success",
        DeviceStatus::Offline => "dim-label",
    };

    let status_dot = Label::builder()
        .label("\u{2022}") // Bullet point
        .css_classes(vec![status_color])
        .build();
    row_box.append(&status_dot);

    // Device info
    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(1)
        .hexpand(true)
        .build();

    let name_label = Label::builder()
        .label(&device.name)
        .css_classes(vec!["caption"])
        .halign(gtk4::Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    info_box.append(&name_label);

    let model_label = Label::builder()
        .label(&device.model)
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    model_label.set_attributes(Some(&{
        let attrs = gtk4::pango::AttrList::new();
        attrs.insert(gtk4::pango::AttrInt::new_scale(0.8));
        attrs
    }));
    info_box.append(&model_label);

    row_box.append(&info_box);

    // Battery indicator (if available)
    if let Some(battery) = device.battery {
        let battery_icon = if battery > 80 {
            "battery-full-symbolic"
        } else if battery > 50 {
            "battery-good-symbolic"
        } else if battery > 20 {
            "battery-low-symbolic"
        } else {
            "battery-empty-symbolic"
        };

        let battery_widget = Image::from_icon_name(battery_icon);
        battery_widget.set_pixel_size(14);
        battery_widget.set_tooltip_text(Some(&format!("{}%", battery)));
        row_box.append(&battery_widget);
    }

    ListBoxRow::builder()
        .child(&row_box)
        .build()
}

fn get_sample_android_devices() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo {
            name: "Samsung Galaxy S23".to_string(),
            model: "SM-S911B".to_string(),
            status: DeviceStatus::Connected,
            battery: Some(78),
        },
    ]
}

fn get_sample_ios_devices() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo {
            name: "iPhone 15 Pro".to_string(),
            model: "A2849".to_string(),
            status: DeviceStatus::Connected,
            battery: Some(92),
        },
    ]
}

fn get_sample_emulators() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo {
            name: "Pixel 7 API 34".to_string(),
            model: "Emulator".to_string(),
            status: DeviceStatus::Running,
            battery: None,
        },
        DeviceInfo {
            name: "Pixel 6 API 33".to_string(),
            model: "Emulator".to_string(),
            status: DeviceStatus::Offline,
            battery: None,
        },
    ]
}
