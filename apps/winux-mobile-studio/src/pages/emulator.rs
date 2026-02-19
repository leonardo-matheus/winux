// Winux Mobile Studio - Emulator Management Page
// Copyright (c) 2026 Winux OS Project
//
// Emulator management:
// - List Android emulators (AVD)
// - Create/delete emulators
// - Start/stop emulators
// - Take screenshots
// - Screen recording

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, Frame, Grid, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, Image, DropDown, StringList, ProgressBar,
};
use libadwaita as adw;
use adw::prelude::*;

#[derive(Clone, Debug)]
pub struct Emulator {
    pub name: String,
    pub device: String,
    pub api_level: u32,
    pub target: String,
    pub status: EmulatorStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EmulatorStatus {
    Running,
    Stopped,
    Creating,
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

    // Running emulators section
    let running_section = create_running_section();
    content.append(&running_section);

    // Available emulators section
    let available_section = create_available_section();
    content.append(&available_section);

    // Create new emulator section
    let create_section = create_new_emulator_section();
    content.append(&create_section);

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

    let icon = Image::from_icon_name("computer-symbolic");
    icon.set_pixel_size(24);
    header.append(&icon);

    let title = Label::builder()
        .label("Emuladores")
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // SDK Manager button
    let sdk_btn = Button::builder()
        .label("SDK Manager")
        .css_classes(vec!["flat"])
        .build();
    header.append(&sdk_btn);

    // Refresh button
    let refresh_btn = Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some("Atualizar lista"));
    header.append(&refresh_btn);

    header
}

fn create_running_section() -> Frame {
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
        .label("Emuladores em Execucao")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let count_label = Label::builder()
        .label("1 ativo")
        .css_classes(vec!["success"])
        .build();
    header.append(&count_label);

    content.append(&header);

    // Running emulator card
    let running_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .margin_top(10)
        .build();

    // Emulator preview placeholder
    let preview = Frame::builder()
        .width_request(120)
        .height_request(200)
        .css_classes(vec!["card"])
        .build();

    let preview_content = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .build();

    let phone_icon = Image::from_icon_name("phone-symbolic");
    phone_icon.set_pixel_size(48);
    preview_content.append(&phone_icon);

    let preview_label = Label::builder()
        .label("Pixel 7")
        .css_classes(vec!["caption"])
        .margin_top(10)
        .build();
    preview_content.append(&preview_label);

    preview.set_child(Some(&preview_content));
    running_box.append(&preview);

    // Emulator info and controls
    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .build();

    let name_label = Label::builder()
        .label("Pixel_7_API_34")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&name_label);

    let details_grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(5)
        .build();

    let details = vec![
        ("Dispositivo:", "Pixel 7"),
        ("API Level:", "34 (Android 14)"),
        ("ABI:", "x86_64"),
        ("RAM:", "2048 MB"),
        ("Status:", "Rodando"),
    ];

    for (i, (label, value)) in details.iter().enumerate() {
        let label_widget = Label::builder()
            .label(*label)
            .css_classes(vec!["dim-label"])
            .halign(gtk4::Align::End)
            .build();
        details_grid.attach(&label_widget, 0, i as i32, 1, 1);

        let value_widget = Label::builder()
            .label(*value)
            .halign(gtk4::Align::Start)
            .build();
        details_grid.attach(&value_widget, 1, i as i32, 1, 1);
    }

    info_box.append(&details_grid);

    // Controls
    let controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .build();

    let screenshot_btn = Button::builder()
        .tooltip_text("Screenshot")
        .build();
    screenshot_btn.set_child(Some(&Image::from_icon_name("camera-photo-symbolic")));
    controls.append(&screenshot_btn);

    let record_btn = Button::builder()
        .tooltip_text("Gravar tela")
        .build();
    record_btn.set_child(Some(&Image::from_icon_name("media-record-symbolic")));
    controls.append(&record_btn);

    let rotate_btn = Button::builder()
        .tooltip_text("Rotacionar")
        .build();
    rotate_btn.set_child(Some(&Image::from_icon_name("object-rotate-right-symbolic")));
    controls.append(&rotate_btn);

    let volume_btn = Button::builder()
        .tooltip_text("Volume")
        .build();
    volume_btn.set_child(Some(&Image::from_icon_name("audio-speakers-symbolic")));
    controls.append(&volume_btn);

    let spacer = Box::builder().hexpand(true).build();
    controls.append(&spacer);

    let stop_btn = Button::builder()
        .label("Parar")
        .css_classes(vec!["destructive-action"])
        .build();
    controls.append(&stop_btn);

    info_box.append(&controls);
    running_box.append(&info_box);

    content.append(&running_box);

    frame.set_child(Some(&content));
    frame
}

fn create_available_section() -> Frame {
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
        .label("Emuladores Disponiveis")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    content.append(&header);

    let list = ListBox::builder()
        .css_classes(vec!["boxed-list"])
        .margin_top(10)
        .build();

    let emulators = vec![
        ("Pixel_7_API_34", "Pixel 7", "34", "Rodando"),
        ("Pixel_6_API_33", "Pixel 6", "33", "Parado"),
        ("Nexus_5X_API_30", "Nexus 5X", "30", "Parado"),
        ("Tablet_API_34", "Pixel Tablet", "34", "Parado"),
    ];

    for (name, device, api, status) in emulators {
        let row = create_emulator_row(name, device, api, status);
        list.append(&row);
    }

    content.append(&list);

    frame.set_child(Some(&content));
    frame
}

fn create_emulator_row(name: &str, device: &str, api: &str, status: &str) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let icon = Image::from_icon_name("phone-symbolic");
    icon.set_pixel_size(32);
    row_box.append(&icon);

    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let name_label = Label::builder()
        .label(name)
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&name_label);

    let details = format!("{} - API {}", device, api);
    let details_label = Label::builder()
        .label(&details)
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&details_label);

    row_box.append(&info_box);

    // Status
    let status_classes = if status == "Rodando" {
        vec!["success", "caption"]
    } else {
        vec!["dim-label", "caption"]
    };
    let status_label = Label::builder()
        .label(status)
        .css_classes(status_classes)
        .build();
    row_box.append(&status_label);

    // Actions
    if status == "Parado" {
        let start_btn = Button::builder()
            .icon_name("media-playback-start-symbolic")
            .css_classes(vec!["flat"])
            .tooltip_text("Iniciar")
            .build();
        row_box.append(&start_btn);
    }

    let edit_btn = Button::builder()
        .icon_name("document-edit-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Editar")
        .build();
    row_box.append(&edit_btn);

    let delete_btn = Button::builder()
        .icon_name("user-trash-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Excluir")
        .build();
    row_box.append(&delete_btn);

    ListBoxRow::builder()
        .child(&row_box)
        .build()
}

fn create_new_emulator_section() -> Frame {
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
        .label("Criar Novo Emulador")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    content.append(&title);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .margin_top(10)
        .build();

    // Name
    let name_label = Label::builder()
        .label("Nome:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&name_label, 0, 0, 1, 1);

    let name_entry = Entry::builder()
        .placeholder_text("Nome do emulador")
        .hexpand(true)
        .build();
    grid.attach(&name_entry, 1, 0, 2, 1);

    // Device
    let device_label = Label::builder()
        .label("Dispositivo:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&device_label, 0, 1, 1, 1);

    let device_list = StringList::new(&[
        "Pixel 8",
        "Pixel 8 Pro",
        "Pixel 7",
        "Pixel 7 Pro",
        "Pixel 6",
        "Pixel Tablet",
        "Nexus 5X",
        "Generic Phone",
        "Generic Tablet",
    ]);
    let device = DropDown::new(Some(device_list), gtk4::Expression::NONE);
    device.set_hexpand(true);
    grid.attach(&device, 1, 1, 2, 1);

    // API Level
    let api_label = Label::builder()
        .label("API Level:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&api_label, 0, 2, 1, 1);

    let api_list = StringList::new(&[
        "34 - Android 14 (UpsideDownCake)",
        "33 - Android 13 (Tiramisu)",
        "32 - Android 12L",
        "31 - Android 12 (Snow Cone)",
        "30 - Android 11 (Red Velvet Cake)",
        "29 - Android 10 (Quince Tart)",
    ]);
    let api = DropDown::new(Some(api_list), gtk4::Expression::NONE);
    api.set_hexpand(true);
    grid.attach(&api, 1, 2, 2, 1);

    // System Image
    let image_label = Label::builder()
        .label("System Image:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&image_label, 0, 3, 1, 1);

    let image_list = StringList::new(&[
        "x86_64 (Recomendado)",
        "x86",
        "arm64-v8a",
        "armeabi-v7a",
    ]);
    let image = DropDown::new(Some(image_list), gtk4::Expression::NONE);
    image.set_hexpand(true);
    grid.attach(&image, 1, 3, 2, 1);

    // RAM
    let ram_label = Label::builder()
        .label("RAM:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&ram_label, 0, 4, 1, 1);

    let ram_list = StringList::new(&[
        "1024 MB",
        "1536 MB",
        "2048 MB (Recomendado)",
        "3072 MB",
        "4096 MB",
    ]);
    let ram = DropDown::new(Some(ram_list), gtk4::Expression::NONE);
    ram.set_selected(2);
    ram.set_hexpand(true);
    grid.attach(&ram, 1, 4, 2, 1);

    // Storage
    let storage_label = Label::builder()
        .label("Armazenamento:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&storage_label, 0, 5, 1, 1);

    let storage_list = StringList::new(&[
        "2 GB",
        "4 GB",
        "6 GB (Recomendado)",
        "8 GB",
        "16 GB",
    ]);
    let storage = DropDown::new(Some(storage_list), gtk4::Expression::NONE);
    storage.set_selected(2);
    storage.set_hexpand(true);
    grid.attach(&storage, 1, 5, 2, 1);

    content.append(&grid);

    // Create button
    let actions = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(15)
        .build();

    let create_btn = Button::builder()
        .label("Criar Emulador")
        .css_classes(vec!["suggested-action"])
        .build();
    actions.append(&create_btn);

    let download_label = Label::builder()
        .label("System images serao baixados se necessario")
        .css_classes(vec!["dim-label", "caption"])
        .hexpand(true)
        .halign(gtk4::Align::End)
        .build();
    actions.append(&download_label);

    content.append(&actions);

    frame.set_child(Some(&content));
    frame
}
