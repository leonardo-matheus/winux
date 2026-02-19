// Winux Mobile Studio - iOS Build Page
// Copyright (c) 2026 Winux OS Project
//
// iOS build management (limited without Mac):
// - Compile Swift for Linux
// - Create .deb for jailbreak
// - Theos for tweaks
// - IPA building (requires Mac or workarounds)

use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, Entry, Frame, Grid, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, Image, InfoBar, MessageType, DropDown, StringList,
};
use libadwaita as adw;
use adw::prelude::*;

#[derive(Clone, Debug)]
pub struct IOSBuildConfig {
    pub target: IOSTarget,
    pub swift_version: String,
    pub deployment_target: String,
    pub code_sign: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IOSTarget {
    Simulator,
    Device,
    JailbreakDeb,
    TheosTweak,
}

pub fn create_page() -> Box {
    let page = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Header
    let header = create_header();
    page.append(&header);

    // Warning about limited iOS support
    let warning = create_warning_banner();
    page.append(&warning);

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

    // Swift compilation section
    let swift_section = create_swift_section();
    content.append(&swift_section);

    // Theos/Jailbreak section
    let theos_section = create_theos_section();
    content.append(&theos_section);

    // IPA section
    let ipa_section = create_ipa_section();
    content.append(&ipa_section);

    // Tools status
    let tools_section = create_tools_status();
    content.append(&tools_section);

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

    let icon = Image::from_icon_name("phone-apple-iphone-symbolic");
    icon.set_pixel_size(24);
    header.append(&icon);

    let title = Label::builder()
        .label("iOS / Swift Build")
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Swift version info
    let swift_label = Label::builder()
        .label("Swift: 5.9 | Theos: Instalado")
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&swift_label);

    header
}

fn create_warning_banner() -> InfoBar {
    let info_bar = InfoBar::builder()
        .message_type(MessageType::Warning)
        .show_close_button(true)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(10)
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .build();

    let title = Label::builder()
        .label("Suporte iOS Limitado no Linux")
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .build();
    content.append(&title);

    let message = Label::builder()
        .label("Build de IPA oficial requer macOS. Aqui voce pode compilar Swift, criar tweaks com Theos, e gerar .deb para dispositivos jailbreak.")
        .wrap(true)
        .halign(gtk4::Align::Start)
        .build();
    content.append(&message);

    info_bar.add_child(&content);
    info_bar
}

fn create_swift_section() -> Frame {
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
        .label("Compilar Swift")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let docs_btn = Button::builder()
        .label("Documentacao Swift")
        .css_classes(vec!["flat"])
        .build();
    header.append(&docs_btn);

    content.append(&header);

    let description = Label::builder()
        .label("Compile codigo Swift para Linux usando Swift for Linux. Util para bibliotecas compartilhadas e ferramentas de linha de comando.")
        .wrap(true)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label"])
        .build();
    content.append(&description);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .margin_top(10)
        .build();

    // Source file/project
    let source_label = Label::builder()
        .label("Projeto/Arquivo:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&source_label, 0, 0, 1, 1);

    let source_entry = Entry::builder()
        .placeholder_text("Caminho para Package.swift ou arquivo .swift")
        .hexpand(true)
        .build();
    grid.attach(&source_entry, 1, 0, 1, 1);

    let browse_btn = Button::builder()
        .label("Selecionar...")
        .build();
    grid.attach(&browse_btn, 2, 0, 1, 1);

    // Build configuration
    let config_label = Label::builder()
        .label("Configuracao:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&config_label, 0, 1, 1, 1);

    let config_list = StringList::new(&["Debug", "Release"]);
    let config = DropDown::new(Some(config_list), gtk4::Expression::NONE);
    config.set_hexpand(true);
    grid.attach(&config, 1, 1, 2, 1);

    content.append(&grid);

    // Build button
    let build_btn = Button::builder()
        .label("Compilar Swift")
        .css_classes(vec!["suggested-action"])
        .margin_top(10)
        .halign(gtk4::Align::Start)
        .build();
    content.append(&build_btn);

    frame.set_child(Some(&content));
    frame
}

fn create_theos_section() -> Frame {
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
        .label("Theos - Tweaks para Jailbreak")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    content.append(&title);

    let description = Label::builder()
        .label("Crie tweaks e extensoes para iOS usando Theos. Gera pacotes .deb para instalacao via Cydia/Sileo.")
        .wrap(true)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label"])
        .build();
    content.append(&description);

    let grid = Grid::builder()
        .column_spacing(15)
        .row_spacing(10)
        .margin_top(10)
        .build();

    // Project type
    let type_label = Label::builder()
        .label("Tipo de Projeto:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&type_label, 0, 0, 1, 1);

    let type_list = StringList::new(&[
        "Tweak",
        "Application",
        "PreferenceBundle",
        "Tool",
        "Library",
    ]);
    let project_type = DropDown::new(Some(type_list), gtk4::Expression::NONE);
    project_type.set_hexpand(true);
    grid.attach(&project_type, 1, 0, 2, 1);

    // Bundle ID
    let bundle_label = Label::builder()
        .label("Bundle ID:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&bundle_label, 0, 1, 1, 1);

    let bundle_entry = Entry::builder()
        .placeholder_text("com.example.mytweak")
        .hexpand(true)
        .build();
    grid.attach(&bundle_entry, 1, 1, 2, 1);

    // Target iOS version
    let target_label = Label::builder()
        .label("iOS Target:")
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&target_label, 0, 2, 1, 1);

    let target_list = StringList::new(&["14.0", "15.0", "16.0", "17.0"]);
    let target = DropDown::new(Some(target_list), gtk4::Expression::NONE);
    target.set_hexpand(true);
    grid.attach(&target, 1, 2, 2, 1);

    content.append(&grid);

    // Actions
    let actions = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(10)
        .build();

    let new_btn = Button::builder()
        .label("Novo Projeto Theos")
        .css_classes(vec!["suggested-action"])
        .build();
    actions.append(&new_btn);

    let build_btn = Button::builder()
        .label("Build .deb")
        .build();
    actions.append(&build_btn);

    let deploy_btn = Button::builder()
        .label("Deploy no Device")
        .build();
    actions.append(&deploy_btn);

    content.append(&actions);

    frame.set_child(Some(&content));
    frame
}

fn create_ipa_section() -> Frame {
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
        .label("IPA Build (Experimental)")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let beta_label = Label::builder()
        .label("BETA")
        .css_classes(vec!["warning"])
        .build();
    header.append(&beta_label);

    content.append(&header);

    let description = Label::builder()
        .label("Opcoes experimentais para criar IPAs sem Mac. Requer ferramentas adicionais e pode nao funcionar para todos os projetos.")
        .wrap(true)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label"])
        .build();
    content.append(&description);

    let options = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(10)
        .build();

    let option1 = CheckButton::with_label("Usar xcbuild (alternativa ao xcodebuild)");
    options.append(&option1);

    let option2 = CheckButton::with_label("Usar toolchain cross-compilation");
    options.append(&option2);

    let option3 = CheckButton::with_label("Re-assinar IPA existente");
    option3.set_active(true);
    options.append(&option3);

    content.append(&options);

    let actions = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(10)
        .build();

    let resign_btn = Button::builder()
        .label("Re-assinar IPA")
        .css_classes(vec!["suggested-action"])
        .build();
    actions.append(&resign_btn);

    let install_btn = Button::builder()
        .label("Instalar via libimobiledevice")
        .build();
    actions.append(&install_btn);

    content.append(&actions);

    frame.set_child(Some(&content));
    frame
}

fn create_tools_status() -> Frame {
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
        .label("Ferramentas Instaladas")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    content.append(&title);

    let list = ListBox::builder()
        .css_classes(vec!["boxed-list"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    let tools = vec![
        ("Swift for Linux", "5.9.2", true),
        ("Theos", "3.0", true),
        ("ldid", "2.1.5", true),
        ("libimobiledevice", "1.3.0", true),
        ("ideviceinstaller", "1.1.1", true),
        ("xcbuild", "-", false),
    ];

    for (name, version, installed) in tools {
        let row = create_tool_row(name, version, installed);
        list.append(&row);
    }

    content.append(&list);

    let install_btn = Button::builder()
        .label("Instalar Ferramentas Faltando")
        .css_classes(vec!["flat"])
        .halign(gtk4::Align::Start)
        .margin_top(10)
        .build();
    content.append(&install_btn);

    frame.set_child(Some(&content));
    frame
}

fn create_tool_row(name: &str, version: &str, installed: bool) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let status_icon = if installed {
        Image::from_icon_name("emblem-ok-symbolic")
    } else {
        Image::from_icon_name("dialog-warning-symbolic")
    };
    row_box.append(&status_icon);

    let name_label = Label::builder()
        .label(name)
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();
    row_box.append(&name_label);

    let version_label = Label::builder()
        .label(version)
        .css_classes(vec!["dim-label"])
        .build();
    row_box.append(&version_label);

    ListBoxRow::builder()
        .child(&row_box)
        .activatable(false)
        .build()
}
