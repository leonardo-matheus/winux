// Winux Mobile Studio - Android Build Page
// Copyright (c) 2026 Winux OS Project
//
// Android build management:
// - Build APK (debug/release)
// - Build AAB for Play Store
// - Sign APK/AAB
// - Install on device via ADB

use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, ComboBoxText, Entry, Frame, Grid, Label,
    ListBox, ListBoxRow, Orientation, ProgressBar, ScrolledWindow,
    Separator, Image, TextView, DropDown, StringList,
};
use libadwaita as adw;
use adw::prelude::*;

#[derive(Clone, Debug)]
pub struct BuildConfig {
    pub build_type: BuildType,
    pub flavor: Option<String>,
    pub sign_config: Option<SignConfig>,
    pub minify: bool,
    pub split_apk: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuildType {
    Debug,
    Release,
    Profile,
}

#[derive(Clone, Debug)]
pub struct SignConfig {
    pub keystore_path: String,
    pub key_alias: String,
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

    // Build configuration section
    let build_section = create_build_section();
    content.append(&build_section);

    // APK signing section
    let signing_section = create_signing_section();
    content.append(&signing_section);

    // Build actions
    let actions_section = create_actions_section();
    content.append(&actions_section);

    // Build history
    let history_section = create_history_section();
    content.append(&history_section);

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

    let icon = Image::from_icon_name("phone-symbolic");
    icon.set_pixel_size(24);
    header.append(&icon);

    let title = Label::builder()
        .label("Android Build")
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // SDK info
    let sdk_label = Label::builder()
        .label("SDK: 34 | Build Tools: 34.0.0")
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&sdk_label);

    header
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
        .label("Configuracao de Build")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    content.append(&title);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .build();

    // Build type
    let build_label = Label::builder()
        .label("Tipo de Build:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&build_label, 0, 0, 1, 1);

    let build_type_list = StringList::new(&["Debug", "Release", "Profile"]);
    let build_type = DropDown::new(Some(build_type_list), gtk4::Expression::NONE);
    build_type.set_hexpand(true);
    grid.attach(&build_type, 1, 0, 2, 1);

    // Build variant/flavor
    let flavor_label = Label::builder()
        .label("Flavor:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&flavor_label, 0, 1, 1, 1);

    let flavor_list = StringList::new(&["(nenhum)", "dev", "staging", "production"]);
    let flavor = DropDown::new(Some(flavor_list), gtk4::Expression::NONE);
    flavor.set_hexpand(true);
    grid.attach(&flavor, 1, 1, 2, 1);

    // Output format
    let format_label = Label::builder()
        .label("Formato:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&format_label, 0, 2, 1, 1);

    let format_list = StringList::new(&["APK", "AAB (App Bundle)"]);
    let format = DropDown::new(Some(format_list), gtk4::Expression::NONE);
    format.set_hexpand(true);
    grid.attach(&format, 1, 2, 2, 1);

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

    let minify_check = CheckButton::with_label("Minificar codigo (ProGuard/R8)");
    options_box.append(&minify_check);

    let split_check = CheckButton::with_label("Gerar APKs separados por ABI");
    options_box.append(&split_check);

    let debug_symbols = CheckButton::with_label("Incluir simbolos de debug");
    debug_symbols.set_active(true);
    options_box.append(&debug_symbols);

    grid.attach(&options_box, 1, 3, 2, 1);

    content.append(&grid);
    frame.set_child(Some(&content));

    frame
}

fn create_signing_section() -> Frame {
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
        .label("Assinatura de APK")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let sign_check = CheckButton::with_label("Assinar APK");
    sign_check.set_active(true);
    header.append(&sign_check);

    content.append(&header);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .build();

    // Keystore
    let keystore_label = Label::builder()
        .label("Keystore:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&keystore_label, 0, 0, 1, 1);

    let keystore_entry = Entry::builder()
        .placeholder_text("Caminho para keystore")
        .hexpand(true)
        .build();
    grid.attach(&keystore_entry, 1, 0, 1, 1);

    let keystore_btn = Button::builder()
        .label("Selecionar...")
        .build();
    grid.attach(&keystore_btn, 2, 0, 1, 1);

    // Key alias
    let alias_label = Label::builder()
        .label("Key Alias:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&alias_label, 0, 1, 1, 1);

    let alias_entry = Entry::builder()
        .placeholder_text("Alias da chave")
        .hexpand(true)
        .build();
    grid.attach(&alias_entry, 1, 1, 2, 1);

    // Create new keystore
    let create_btn = Button::builder()
        .label("Criar Nova Keystore")
        .css_classes(vec!["flat"])
        .halign(gtk4::Align::Start)
        .build();
    grid.attach(&create_btn, 1, 2, 2, 1);

    content.append(&grid);
    frame.set_child(Some(&content));

    frame
}

fn create_actions_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let title = Label::builder()
        .label("Acoes de Build")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    section.append(&title);

    let buttons_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let build_btn = Button::builder()
        .css_classes(vec!["suggested-action", "pill"])
        .build();

    let build_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    build_content.append(&Image::from_icon_name("system-run-symbolic"));
    build_content.append(&Label::new(Some("Build APK")));
    build_btn.set_child(Some(&build_content));
    buttons_box.append(&build_btn);

    let install_btn = Button::builder()
        .css_classes(vec!["pill"])
        .build();

    let install_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    install_content.append(&Image::from_icon_name("document-save-symbolic"));
    install_content.append(&Label::new(Some("Instalar no Device")));
    install_btn.set_child(Some(&install_content));
    buttons_box.append(&install_btn);

    let clean_btn = Button::builder()
        .css_classes(vec!["pill"])
        .build();

    let clean_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    clean_content.append(&Image::from_icon_name("edit-clear-symbolic"));
    clean_content.append(&Label::new(Some("Clean Build")));
    clean_btn.set_child(Some(&clean_content));
    buttons_box.append(&clean_btn);

    section.append(&buttons_box);

    // Progress bar (hidden by default)
    let progress_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .margin_top(10)
        .build();

    let progress_label = Label::builder()
        .label("Pronto para build")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    progress_box.append(&progress_label);

    let progress = ProgressBar::builder()
        .fraction(0.0)
        .build();
    progress_box.append(&progress);

    section.append(&progress_box);

    section
}

fn create_history_section() -> Frame {
    let frame = Frame::builder()
        .css_classes(vec!["card"])
        .vexpand(true)
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let title = Label::builder()
        .label("Historico de Builds")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let clear_btn = Button::builder()
        .label("Limpar")
        .css_classes(vec!["flat"])
        .build();
    header.append(&clear_btn);

    content.append(&header);

    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let list = ListBox::builder()
        .css_classes(vec!["boxed-list"])
        .build();

    // Sample build history
    let builds = vec![
        ("app-debug.apk", "12.5 MB", "Sucesso", "10:35"),
        ("app-release.aab", "8.2 MB", "Sucesso", "09:20"),
        ("app-debug.apk", "12.4 MB", "Falhou", "Ontem"),
    ];

    for (name, size, status, time) in builds {
        let row = create_build_history_row(name, size, status, time);
        list.append(&row);
    }

    scrolled.set_child(Some(&list));
    content.append(&scrolled);

    frame.set_child(Some(&content));
    frame
}

fn create_build_history_row(name: &str, size: &str, status: &str, time: &str) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let status_icon = if status == "Sucesso" {
        Image::from_icon_name("emblem-ok-symbolic")
    } else {
        Image::from_icon_name("dialog-error-symbolic")
    };
    row_box.append(&status_icon);

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

    let details = format!("{} - {}", size, time);
    let details_label = Label::builder()
        .label(&details)
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    info_box.append(&details_label);

    row_box.append(&info_box);

    let open_btn = Button::builder()
        .icon_name("folder-open-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Abrir pasta")
        .build();
    row_box.append(&open_btn);

    ListBoxRow::builder()
        .child(&row_box)
        .build()
}
