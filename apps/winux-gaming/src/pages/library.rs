// Library page - Unified game library from all sources
// Shows all games from Steam, GOG, Epic, and native games

use gtk4::prelude::*;
use gtk4::{
    Box, Button, ComboBoxText, FlowBox, Frame, Image, Label,
    Orientation, ScrolledWindow, SearchEntry, ToggleButton,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage};

use crate::ui::game_card::{GameInfo, Platform, create_game_card};

pub fn create_library_page() -> ScrolledWindow {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Header with search and filters
    let header = create_library_header();
    main_box.append(&header);

    // Quick stats bar
    let stats_bar = create_stats_bar();
    main_box.append(&stats_bar);

    // Recently played section
    let recent_section = create_section("Jogados Recentemente", get_recent_games());
    main_box.append(&recent_section);

    // All games section
    let all_games_section = create_all_games_section();
    main_box.append(&all_games_section);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_library_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(10)
        .build();

    // Title
    let title = Label::builder()
        .label("Minha Biblioteca")
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    // Spacer
    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Search
    let search = SearchEntry::builder()
        .placeholder_text("Buscar jogos...")
        .width_request(300)
        .build();
    header.append(&search);

    // View toggle (grid/list)
    let view_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(vec!["linked"])
        .build();

    let grid_btn = ToggleButton::builder()
        .icon_name("view-grid-symbolic")
        .active(true)
        .build();
    let list_btn = ToggleButton::builder()
        .icon_name("view-list-symbolic")
        .group(&grid_btn)
        .build();

    view_box.append(&grid_btn);
    view_box.append(&list_btn);
    header.append(&view_box);

    // Filter dropdown
    let filter = ComboBoxText::new();
    filter.append_text("Todos");
    filter.append_text("Instalados");
    filter.append_text("Steam");
    filter.append_text("GOG");
    filter.append_text("Epic");
    filter.append_text("Nativos");
    filter.set_active(Some(0));
    header.append(&filter);

    // Sort dropdown
    let sort = ComboBoxText::new();
    sort.append_text("Nome");
    sort.append_text("Recentes");
    sort.append_text("Tempo de Jogo");
    sort.append_text("Plataforma");
    sort.set_active(Some(1));
    header.append(&sort);

    header
}

fn create_stats_bar() -> Box {
    let stats = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(30)
        .margin_start(20)
        .margin_end(20)
        .margin_top(10)
        .margin_bottom(20)
        .halign(gtk4::Align::Start)
        .build();

    let items = [
        ("256", "Jogos"),
        ("148", "Instalados"),
        ("1,847h", "Tempo Total"),
        ("42", "Conquistas"),
    ];

    for (value, label) in items {
        let item_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();

        let value_label = Label::builder()
            .label(value)
            .css_classes(vec!["title-2"])
            .build();
        let text_label = Label::builder()
            .label(label)
            .css_classes(vec!["dim-label", "caption"])
            .build();

        item_box.append(&value_label);
        item_box.append(&text_label);
        stats.append(&item_box);
    }

    stats
}

fn create_section(title: &str, games: Vec<GameInfo>) -> Box {
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

    let title_label = Label::builder()
        .label(title)
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title_label);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let see_all = Button::builder()
        .label("Ver Todos")
        .css_classes(vec!["flat"])
        .build();
    header.append(&see_all);

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

    for game in games {
        let card = create_game_card(&game);
        flow_box.insert(&card, -1);
    }

    section.append(&flow_box);
    section
}

fn create_all_games_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .vexpand(true)
        .build();

    // Section header
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let title_label = Label::builder()
        .label("Todos os Jogos")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title_label);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let count_label = Label::builder()
        .label("256 jogos")
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&count_label);

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

    let all_games = get_all_games();
    for game in all_games {
        let card = create_game_card(&game);
        flow_box.insert(&card, -1);
    }

    section.append(&flow_box);
    section
}

fn get_recent_games() -> Vec<GameInfo> {
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
            id: "rdr2".to_string(),
            name: "Red Dead Redemption 2".to_string(),
            platform: Platform::Epic,
            installed: true,
            playtime_hours: 120.0,
            last_played: Some("3 dias".to_string()),
            cover_icon: "RD".to_string(),
            native: false,
        },
    ]
}

fn get_all_games() -> Vec<GameInfo> {
    vec![
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
            id: "hades".to_string(),
            name: "Hades".to_string(),
            platform: Platform::Epic,
            installed: true,
            playtime_hours: 95.0,
            last_played: Some("1 semana".to_string()),
            cover_icon: "HA".to_string(),
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
            id: "terraria".to_string(),
            name: "Terraria".to_string(),
            platform: Platform::Steam,
            installed: false,
            playtime_hours: 0.0,
            last_played: None,
            cover_icon: "TE".to_string(),
            native: true,
        },
        GameInfo {
            id: "witcher3".to_string(),
            name: "The Witcher 3".to_string(),
            platform: Platform::GOG,
            installed: true,
            playtime_hours: 180.0,
            last_played: Some("2 meses".to_string()),
            cover_icon: "W3".to_string(),
            native: true,
        },
    ]
}
