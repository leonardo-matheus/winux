// Epic Games Store integration page
// Uses Heroic Games Launcher for Epic Games support

use gtk4::prelude::*;
use gtk4::{
    Box, Button, FlowBox, Frame, Image, Label,
    Orientation, ProgressBar, ScrolledWindow,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, SwitchRow};

use crate::ui::game_card::{GameInfo, Platform, create_game_card};

pub fn create_epic_page() -> ScrolledWindow {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Epic header
    let header = create_epic_header();
    main_box.append(&header);

    // Connection status
    let status = create_connection_status();
    main_box.append(&status);

    // Free games section
    let free_games = create_free_games_section();
    main_box.append(&free_games);

    // Epic library section
    let library = create_epic_library();
    main_box.append(&library);

    // Epic settings
    let settings = create_epic_settings();
    main_box.append(&settings);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_epic_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(10)
        .build();

    // Epic logo placeholder
    let logo_box = Box::builder()
        .width_request(48)
        .height_request(48)
        .css_classes(vec!["card"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();
    let logo_label = Label::builder()
        .label("E")
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
        .label("Epic Games")
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    title_box.append(&title);

    let subtitle = Label::builder()
        .label("Via Heroic Games Launcher")
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
        .label("Conectado como: EpicGamer789")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    status_box.append(&status_label);

    let spacer = Box::builder().hexpand(true).build();
    status_box.append(&spacer);

    let games_count = Label::builder()
        .label("124 jogos na biblioteca")
        .css_classes(vec!["dim-label"])
        .build();
    status_box.append(&games_count);

    status_box
}

fn create_free_games_section() -> Box {
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
        .label("Jogos Gratis da Semana")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    let timer_label = Label::builder()
        .label("Expira em 3d 14h")
        .css_classes(vec!["dim-label"])
        .build();
    header.append(&timer_label);

    section.append(&header);

    // Free games banner
    let banner = Frame::builder()
        .css_classes(vec!["card", "featured-banner"])
        .build();

    let banner_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(20)
        .margin_start(25)
        .margin_end(25)
        .margin_top(25)
        .margin_bottom(25)
        .build();

    // Current free game
    let game_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .build();

    let free_badge = Label::builder()
        .label("GRATIS")
        .css_classes(vec!["success", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    game_box.append(&free_badge);

    let game_title = Label::builder()
        .label("Control Ultimate Edition")
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    game_box.append(&game_title);

    let game_desc = Label::builder()
        .label("Action-adventure com poderes sobrenaturais. Inclui todos DLCs.")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .wrap(true)
        .build();
    game_box.append(&game_desc);

    let claim_btn = Button::builder()
        .label("Resgatar Gratis")
        .css_classes(vec!["suggested-action", "pill"])
        .halign(gtk4::Align::Start)
        .margin_top(10)
        .build();
    game_box.append(&claim_btn);

    banner_content.append(&game_box);

    // Upcoming free game
    let upcoming_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .width_request(200)
        .build();

    let upcoming_label = Label::builder()
        .label("Proximo:")
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    upcoming_box.append(&upcoming_label);

    let upcoming_title = Label::builder()
        .label("Ghostrunner")
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .build();
    upcoming_box.append(&upcoming_title);

    let upcoming_date = Label::builder()
        .label("Em 4 dias")
        .css_classes(vec!["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    upcoming_box.append(&upcoming_date);

    banner_content.append(&upcoming_box);

    banner.set_child(Some(&banner_content));
    section.append(&banner);

    section
}

fn create_epic_library() -> Box {
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
        .label("Biblioteca Epic")
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

    let games = get_epic_games();
    for game in games {
        let card = create_game_card(&game);
        flow_box.insert(&card, -1);
    }

    section.append(&flow_box);
    section
}

fn create_epic_settings() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    let page = PreferencesPage::new();

    // Epic settings via Heroic
    let epic_group = PreferencesGroup::builder()
        .title("Configuracoes Epic Games")
        .description("Opcoes de integracao via Heroic")
        .build();

    // EGL Sync
    let egl_sync = SwitchRow::builder()
        .title("Sincronizar com EGL")
        .subtitle("Sincronizar com Epic Games Launcher (se instalado)")
        .active(false)
        .build();
    epic_group.add(&egl_sync);

    // Alternative Legendary
    let use_legendary = SwitchRow::builder()
        .title("Usar Legendary")
        .subtitle("Cliente CLI alternativo para Epic Games")
        .active(true)
        .build();
    epic_group.add(&use_legendary);

    // Auto-update games
    let auto_update = SwitchRow::builder()
        .title("Atualizar Automaticamente")
        .subtitle("Baixar atualizacoes automaticamente")
        .active(true)
        .build();
    epic_group.add(&auto_update);

    // Wine/Proton settings
    let wine_version = adw::ComboRow::builder()
        .title("Runtime Padrao")
        .subtitle("Wine/Proton para jogos Windows")
        .build();
    let versions = gtk4::StringList::new(&[
        "Wine-GE 8-26",
        "Proton-GE 8-25",
        "Wine 9.0",
        "Proton Experimental",
    ]);
    wine_version.set_model(Some(&versions));
    wine_version.set_selected(0);
    epic_group.add(&wine_version);

    page.add(&epic_group);

    // Cloud saves
    let cloud_group = PreferencesGroup::builder()
        .title("Saves na Nuvem")
        .build();

    let cloud_sync = SwitchRow::builder()
        .title("Sincronizacao de Saves")
        .subtitle("Sincronizar saves com Epic Cloud")
        .active(true)
        .build();
    cloud_group.add(&cloud_sync);

    let sync_on_launch = SwitchRow::builder()
        .title("Sincronizar ao Iniciar")
        .subtitle("Baixar saves antes de iniciar o jogo")
        .active(true)
        .build();
    cloud_group.add(&sync_on_launch);

    page.add(&cloud_group);

    section.append(&page);
    section
}

fn get_epic_games() -> Vec<GameInfo> {
    vec![
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
            id: "control".to_string(),
            name: "Control".to_string(),
            platform: Platform::Epic,
            installed: false,
            playtime_hours: 0.0,
            last_played: None,
            cover_icon: "CT".to_string(),
            native: false,
        },
        GameInfo {
            id: "gta5".to_string(),
            name: "Grand Theft Auto V".to_string(),
            platform: Platform::Epic,
            installed: true,
            playtime_hours: 250.0,
            last_played: Some("1 mes".to_string()),
            cover_icon: "GTA".to_string(),
            native: false,
        },
        GameInfo {
            id: "alanwake".to_string(),
            name: "Alan Wake Remastered".to_string(),
            platform: Platform::Epic,
            installed: false,
            playtime_hours: 0.0,
            last_played: None,
            cover_icon: "AW".to_string(),
            native: false,
        },
        GameInfo {
            id: "borderlands3".to_string(),
            name: "Borderlands 3".to_string(),
            platform: Platform::Epic,
            installed: true,
            playtime_hours: 65.0,
            last_played: Some("2 semanas".to_string()),
            cover_icon: "B3".to_string(),
            native: true,
        },
    ]
}
