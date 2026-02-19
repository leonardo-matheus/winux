// Steam integration page
// Import and manage Steam library, launch games with Proton

use gtk4::prelude::*;
use gtk4::{
    Box, Button, FlowBox, Frame, Image, Label, ListBox, ListBoxRow,
    Orientation, ProgressBar, ScrolledWindow, Spinner,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};

use crate::ui::game_card::{GameInfo, Platform, create_game_card};

pub fn create_steam_page() -> ScrolledWindow {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Steam header
    let header = create_steam_header();
    main_box.append(&header);

    // Connection status
    let status = create_connection_status();
    main_box.append(&status);

    // Steam library section
    let library = create_steam_library();
    main_box.append(&library);

    // Proton settings section
    let proton = create_proton_section();
    main_box.append(&proton);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_steam_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(10)
        .build();

    // Steam logo placeholder
    let logo_box = Box::builder()
        .width_request(48)
        .height_request(48)
        .css_classes(vec!["card"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();
    let logo_label = Label::builder()
        .label("S")
        .css_classes(vec!["title-1"])
        .build();
    logo_box.append(&logo_label);
    header.append(&logo_box);

    // Title and subtitle
    let title_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();

    let title = Label::builder()
        .label("Steam")
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    title_box.append(&title);

    let subtitle = Label::builder()
        .label("Biblioteca Steam via Proton")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    title_box.append(&subtitle);

    header.append(&title_box);

    // Spacer
    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Sync button
    let sync_btn = Button::builder()
        .icon_name("emblem-synchronizing-symbolic")
        .tooltip_text("Sincronizar biblioteca")
        .css_classes(vec!["flat"])
        .build();
    header.append(&sync_btn);

    // Open Steam button
    let open_btn = Button::builder()
        .label("Abrir Steam")
        .css_classes(vec!["suggested-action"])
        .build();
    header.append(&open_btn);

    header
}

fn create_connection_status() -> Box {
    let status_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    let status_icon = Image::from_icon_name("emblem-ok-symbolic");
    status_icon.add_css_class("success");
    status_box.append(&status_icon);

    let status_label = Label::builder()
        .label("Conectado como: SteamUser123")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    status_box.append(&status_label);

    let spacer = Box::builder().hexpand(true).build();
    status_box.append(&spacer);

    let games_count = Label::builder()
        .label("186 jogos na biblioteca")
        .css_classes(vec!["dim-label"])
        .build();
    status_box.append(&games_count);

    status_box
}

fn create_steam_library() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    // Section header
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let title = Label::builder()
        .label("Biblioteca Steam")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let filter_btn = Button::builder()
        .label("Filtrar")
        .css_classes(vec!["flat"])
        .build();
    header.append(&filter_btn);

    section.append(&header);

    // Games flow box
    let flow_box = FlowBox::builder()
        .homogeneous(true)
        .column_spacing(15)
        .row_spacing(15)
        .min_children_per_line(2)
        .max_children_per_line(6)
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    let games = get_steam_games();
    for game in games {
        let card = create_game_card(&game);
        flow_box.insert(&card, -1);
    }

    section.append(&flow_box);
    section
}

fn create_proton_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    let page = PreferencesPage::new();

    // Proton settings group
    let proton_group = PreferencesGroup::builder()
        .title("Configuracoes do Proton")
        .description("Gerenciar versoes do Proton para jogos Windows")
        .build();

    // Default Proton version
    let default_proton = adw::ComboRow::builder()
        .title("Versao Padrao do Proton")
        .subtitle("Usado para novos jogos")
        .build();
    let proton_versions = gtk4::StringList::new(&[
        "Proton Experimental",
        "Proton 8.0-4",
        "Proton 7.0-6",
        "Proton-GE 8-25",
        "Wine-GE 8-26",
    ]);
    default_proton.set_model(Some(&proton_versions));
    default_proton.set_selected(0);
    proton_group.add(&default_proton);

    // Steam Play for all games
    let steam_play = SwitchRow::builder()
        .title("Steam Play para Todos os Jogos")
        .subtitle("Usar Proton em jogos sem suporte oficial")
        .active(true)
        .build();
    proton_group.add(&steam_play);

    // Shader cache
    let shader_cache = SwitchRow::builder()
        .title("Pre-compilar Shaders")
        .subtitle("Baixar shaders compilados da Valve")
        .active(true)
        .build();
    proton_group.add(&shader_cache);

    // DXVK async
    let dxvk_async = SwitchRow::builder()
        .title("DXVK Async")
        .subtitle("Compilacao assincrona de shaders (pode causar stuttering inicial)")
        .active(false)
        .build();
    proton_group.add(&dxvk_async);

    page.add(&proton_group);

    // Proton versions manager
    let versions_group = PreferencesGroup::builder()
        .title("Versoes Instaladas")
        .build();

    let versions = [
        ("Proton Experimental", "Ultima versao experimental", true),
        ("Proton 8.0-4", "Versao estavel atual", true),
        ("Proton-GE 8-25", "GloriousEggroll custom", true),
        ("Proton 7.0-6", "Versao anterior", false),
    ];

    for (name, desc, is_latest) in versions {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(desc)
            .build();

        if is_latest {
            let badge = Label::builder()
                .label("Atual")
                .css_classes(vec!["success", "caption"])
                .valign(gtk4::Align::Center)
                .build();
            row.add_suffix(&badge);
        }

        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(vec!["flat"])
            .valign(gtk4::Align::Center)
            .build();
        row.add_suffix(&delete_btn);

        versions_group.add(&row);
    }

    // Add version button
    let add_version_row = ActionRow::builder()
        .title("Adicionar Versao do Proton")
        .subtitle("Baixar ou instalar versao customizada")
        .activatable(true)
        .build();
    add_version_row.add_suffix(&Image::from_icon_name("list-add-symbolic"));
    versions_group.add(&add_version_row);

    page.add(&versions_group);

    section.append(&page);
    section
}

fn get_steam_games() -> Vec<GameInfo> {
    vec![
        GameInfo {
            id: "cyberpunk2077".to_string(),
            name: "Cyberpunk 2077".to_string(),
            platform: Platform::Steam,
            installed: true,
            playtime_hours: 87.5,
            last_played: Some("Hoje".to_string()),
            cover_icon: "CP".to_string(),
            native: false,
        },
        GameInfo {
            id: "eldenring".to_string(),
            name: "Elden Ring".to_string(),
            platform: Platform::Steam,
            installed: true,
            playtime_hours: 200.0,
            last_played: Some("3 dias".to_string()),
            cover_icon: "ER".to_string(),
            native: false,
        },
        GameInfo {
            id: "baldursgate3".to_string(),
            name: "Baldur's Gate 3".to_string(),
            platform: Platform::Steam,
            installed: true,
            playtime_hours: 156.0,
            last_played: Some("Ontem".to_string()),
            cover_icon: "BG".to_string(),
            native: true,
        },
        GameInfo {
            id: "celeste".to_string(),
            name: "Celeste".to_string(),
            platform: Platform::Steam,
            installed: true,
            playtime_hours: 25.0,
            last_played: Some("1 semana".to_string()),
            cover_icon: "CE".to_string(),
            native: true,
        },
        GameInfo {
            id: "stardew".to_string(),
            name: "Stardew Valley".to_string(),
            platform: Platform::Steam,
            installed: true,
            playtime_hours: 150.0,
            last_played: Some("5 dias".to_string()),
            cover_icon: "SV".to_string(),
            native: true,
        },
        GameInfo {
            id: "hollowknight_steam".to_string(),
            name: "Hollow Knight".to_string(),
            platform: Platform::Steam,
            installed: false,
            playtime_hours: 0.0,
            last_played: None,
            cover_icon: "HK".to_string(),
            native: true,
        },
    ]
}
