// GOG Galaxy integration page
// Uses Heroic Games Launcher for GOG support

use gtk4::prelude::*;
use gtk4::{
    Box, Button, FlowBox, Frame, Image, Label, ListBox,
    Orientation, ScrolledWindow,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};

use crate::ui::game_card::{GameInfo, Platform, create_game_card};

pub fn create_gog_page() -> ScrolledWindow {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // GOG header
    let header = create_gog_header();
    main_box.append(&header);

    // Connection status
    let status = create_connection_status();
    main_box.append(&status);

    // GOG library section
    let library = create_gog_library();
    main_box.append(&library);

    // GOG settings
    let settings = create_gog_settings();
    main_box.append(&settings);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_gog_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(10)
        .build();

    // GOG logo placeholder
    let logo_box = Box::builder()
        .width_request(48)
        .height_request(48)
        .css_classes(vec!["card"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();
    let logo_label = Label::builder()
        .label("G")
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
        .label("GOG Galaxy")
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    title_box.append(&title);

    let subtitle = Label::builder()
        .label("Via Heroic Games Launcher - DRM-free gaming")
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

    // Open Heroic button
    let open_btn = Button::builder()
        .label("Abrir Heroic")
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
        .label("Conectado como: GOGUser456")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    status_box.append(&status_label);

    let spacer = Box::builder().hexpand(true).build();
    status_box.append(&spacer);

    let heroic_status = Label::builder()
        .label("Heroic v2.11.0")
        .css_classes(vec!["dim-label"])
        .build();
    status_box.append(&heroic_status);

    let separator = Label::builder()
        .label("|")
        .css_classes(vec!["dim-label"])
        .margin_start(10)
        .margin_end(10)
        .build();
    status_box.append(&separator);

    let games_count = Label::builder()
        .label("47 jogos na biblioteca")
        .css_classes(vec!["dim-label"])
        .build();
    status_box.append(&games_count);

    status_box
}

fn create_gog_library() -> Box {
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
        .label("Biblioteca GOG")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let native_only_btn = Button::builder()
        .label("Apenas Nativos")
        .css_classes(vec!["flat"])
        .build();
    header.append(&native_only_btn);

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

    let games = get_gog_games();
    for game in games {
        let card = create_game_card(&game);
        flow_box.insert(&card, -1);
    }

    section.append(&flow_box);
    section
}

fn create_gog_settings() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    let page = PreferencesPage::new();

    // Heroic settings
    let heroic_group = PreferencesGroup::builder()
        .title("Configuracoes do Heroic")
        .description("Opcoes de integracao com Heroic Games Launcher")
        .build();

    // Wine prefix
    let wine_prefix = ActionRow::builder()
        .title("Prefixo Wine Padrao")
        .subtitle("~/.wine/gog")
        .activatable(true)
        .build();
    wine_prefix.add_suffix(&Image::from_icon_name("folder-symbolic"));
    heroic_group.add(&wine_prefix);

    // Default Wine version
    let wine_version = adw::ComboRow::builder()
        .title("Versao do Wine")
        .subtitle("Usado para jogos Windows")
        .build();
    let wine_versions = gtk4::StringList::new(&[
        "Wine-GE 8-26",
        "Wine 9.0",
        "Wine 8.0",
        "Proton-GE 8-25",
    ]);
    wine_version.set_model(Some(&wine_versions));
    wine_version.set_selected(0);
    heroic_group.add(&wine_version);

    // Auto sync
    let auto_sync = SwitchRow::builder()
        .title("Sincronizacao Automatica")
        .subtitle("Sincronizar biblioteca ao iniciar")
        .active(true)
        .build();
    heroic_group.add(&auto_sync);

    // Cloud saves
    let cloud_saves = SwitchRow::builder()
        .title("Saves na Nuvem")
        .subtitle("Sincronizar saves com GOG Galaxy Cloud")
        .active(true)
        .build();
    heroic_group.add(&cloud_saves);

    page.add(&heroic_group);

    // Download settings
    let download_group = PreferencesGroup::builder()
        .title("Downloads")
        .build();

    // Install location
    let install_loc = ActionRow::builder()
        .title("Pasta de Instalacao")
        .subtitle("~/Games/GOG")
        .activatable(true)
        .build();
    install_loc.add_suffix(&Image::from_icon_name("folder-symbolic"));
    download_group.add(&install_loc);

    // Limit bandwidth
    let bandwidth = adw::SpinRow::builder()
        .title("Limite de Download (MB/s)")
        .subtitle("0 = sem limite")
        .adjustment(&gtk4::Adjustment::new(0.0, 0.0, 1000.0, 1.0, 10.0, 0.0))
        .build();
    download_group.add(&bandwidth);

    page.add(&download_group);

    section.append(&page);
    section
}

fn get_gog_games() -> Vec<GameInfo> {
    vec![
        GameInfo {
            id: "witcher3".to_string(),
            name: "The Witcher 3: Wild Hunt".to_string(),
            platform: Platform::GOG,
            installed: true,
            playtime_hours: 180.0,
            last_played: Some("2 meses".to_string()),
            cover_icon: "W3".to_string(),
            native: true,
        },
        GameInfo {
            id: "disco".to_string(),
            name: "Disco Elysium".to_string(),
            platform: Platform::GOG,
            installed: true,
            playtime_hours: 42.0,
            last_played: Some("1 mes".to_string()),
            cover_icon: "DE".to_string(),
            native: true,
        },
        GameInfo {
            id: "hollowknight".to_string(),
            name: "Hollow Knight".to_string(),
            platform: Platform::GOG,
            installed: true,
            playtime_hours: 45.0,
            last_played: Some("2 dias".to_string()),
            cover_icon: "HK".to_string(),
            native: true,
        },
        GameInfo {
            id: "deadcells".to_string(),
            name: "Dead Cells".to_string(),
            platform: Platform::GOG,
            installed: true,
            playtime_hours: 80.0,
            last_played: Some("2 semanas".to_string()),
            cover_icon: "DC".to_string(),
            native: true,
        },
        GameInfo {
            id: "divinity2".to_string(),
            name: "Divinity: Original Sin 2".to_string(),
            platform: Platform::GOG,
            installed: false,
            playtime_hours: 0.0,
            last_played: None,
            cover_icon: "D2".to_string(),
            native: true,
        },
        GameInfo {
            id: "pathfinder".to_string(),
            name: "Pathfinder: Wrath of the Righteous".to_string(),
            platform: Platform::GOG,
            installed: true,
            playtime_hours: 120.0,
            last_played: Some("1 semana".to_string()),
            cover_icon: "PF".to_string(),
            native: true,
        },
    ]
}
