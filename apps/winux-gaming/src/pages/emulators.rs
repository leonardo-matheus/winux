// Emulators page - RetroArch, Dolphin, PCSX2, RPCS3, etc.
// Unified emulation management for retro gaming

use gtk4::prelude::*;
use gtk4::{
    Box, Button, FlowBox, Frame, Image, Label, ListBox, ListBoxRow,
    Orientation, ProgressBar, ScrolledWindow,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow};

pub fn create_emulators_page() -> ScrolledWindow {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Header
    let header = create_emulators_header();
    main_box.append(&header);

    // Quick launch section for recent ROMs
    let recent = create_recent_roms_section();
    main_box.append(&recent);

    // Emulators grid
    let emulators = create_emulators_grid();
    main_box.append(&emulators);

    // RetroArch settings
    let retroarch = create_retroarch_section();
    main_box.append(&retroarch);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_emulators_header() -> Box {
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
        .label("Emuladores")
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    // Spacer
    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Scan ROMs button
    let scan_btn = Button::builder()
        .label("Escanear ROMs")
        .css_classes(vec!["flat"])
        .build();
    header.append(&scan_btn);

    // Add emulator button
    let add_btn = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Adicionar emulador")
        .css_classes(vec!["flat"])
        .build();
    header.append(&add_btn);

    header
}

fn create_recent_roms_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    // Section header
    let title = Label::builder()
        .label("Jogados Recentemente")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    section.append(&title);

    // ROMs flow
    let flow_box = FlowBox::builder()
        .homogeneous(true)
        .column_spacing(15)
        .row_spacing(15)
        .min_children_per_line(2)
        .max_children_per_line(6)
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    let recent_roms = [
        ("Super Mario World", "SNES", "RetroArch", "SMW"),
        ("The Legend of Zelda: OoT", "N64", "RetroArch", "ZOT"),
        ("Pokemon FireRed", "GBA", "RetroArch", "PFR"),
        ("Sonic Adventure 2", "Dreamcast", "Flycast", "SA2"),
        ("Mario Kart: Double Dash", "GameCube", "Dolphin", "MKD"),
        ("Persona 4 Golden", "PS Vita", "Vita3K", "P4G"),
    ];

    for (name, system, emulator, icon_text) in recent_roms {
        let card = create_rom_card(name, system, emulator, icon_text);
        flow_box.insert(&card, -1);
    }

    section.append(&flow_box);
    section
}

fn create_rom_card(name: &str, system: &str, emulator: &str, icon_text: &str) -> Frame {
    let card = Frame::builder()
        .css_classes(vec!["card", "game-card"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .width_request(160)
        .build();

    // ROM cover placeholder
    let cover_box = Box::builder()
        .width_request(140)
        .height_request(100)
        .css_classes(vec!["card"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let cover_label = Label::builder()
        .label(icon_text)
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();
    cover_box.append(&cover_label);
    content.append(&cover_box);

    // ROM name
    let name_label = Label::builder()
        .label(name)
        .css_classes(vec!["game-title"])
        .halign(gtk4::Align::Center)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    content.append(&name_label);

    // System and emulator
    let info_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .halign(gtk4::Align::Center)
        .build();

    let system_label = Label::builder()
        .label(system)
        .css_classes(vec!["dim-label", "caption"])
        .build();
    info_box.append(&system_label);

    let sep = Label::builder()
        .label("|")
        .css_classes(vec!["dim-label", "caption"])
        .build();
    info_box.append(&sep);

    let emu_label = Label::builder()
        .label(emulator)
        .css_classes(vec!["dim-label", "caption"])
        .build();
    info_box.append(&emu_label);

    content.append(&info_box);

    // Play button
    let play_btn = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .css_classes(vec!["suggested-action", "circular"])
        .halign(gtk4::Align::Center)
        .build();
    content.append(&play_btn);

    card.set_child(Some(&content));
    card
}

fn create_emulators_grid() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    // Section header
    let title = Label::builder()
        .label("Emuladores Instalados")
        .css_classes(vec!["title-3"])
        .halign(gtk4::Align::Start)
        .build();
    section.append(&title);

    let page = PreferencesPage::new();

    // RetroArch (multi-system)
    let retroarch_group = PreferencesGroup::builder()
        .title("RetroArch")
        .description("Emulador multi-sistema com cores")
        .build();

    let cores = [
        ("bsnes", "Super Nintendo", "54 jogos"),
        ("mupen64plus", "Nintendo 64", "23 jogos"),
        ("mgba", "Game Boy Advance", "87 jogos"),
        ("genesis_plus_gx", "Mega Drive/Genesis", "42 jogos"),
        ("pcsx_rearmed", "PlayStation", "31 jogos"),
        ("flycast", "Dreamcast", "15 jogos"),
    ];

    for (core, system, games) in cores {
        let row = ActionRow::builder()
            .title(system)
            .subtitle(&format!("Core: {} | {}", core, games))
            .activatable(true)
            .build();

        let play_btn = Button::builder()
            .icon_name("media-playback-start-symbolic")
            .css_classes(vec!["flat"])
            .valign(gtk4::Align::Center)
            .tooltip_text("Abrir biblioteca")
            .build();
        row.add_suffix(&play_btn);

        let settings_btn = Button::builder()
            .icon_name("emblem-system-symbolic")
            .css_classes(vec!["flat"])
            .valign(gtk4::Align::Center)
            .tooltip_text("Configuracoes do core")
            .build();
        row.add_suffix(&settings_btn);

        retroarch_group.add(&row);
    }

    page.add(&retroarch_group);

    // Standalone emulators
    let standalone_group = PreferencesGroup::builder()
        .title("Emuladores Standalone")
        .description("Emuladores dedicados para sistemas especificos")
        .build();

    let emulators = [
        ("Dolphin", "GameCube/Wii", "18 jogos", true),
        ("PCSX2", "PlayStation 2", "45 jogos", true),
        ("RPCS3", "PlayStation 3", "12 jogos", true),
        ("Ryujinx", "Nintendo Switch", "8 jogos", true),
        ("Vita3K", "PlayStation Vita", "5 jogos", false),
        ("Cemu", "Wii U", "6 jogos", true),
    ];

    for (name, system, games, installed) in emulators {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(&format!("{} | {}", system, games))
            .activatable(true)
            .build();

        if installed {
            let status = Image::from_icon_name("emblem-ok-symbolic");
            status.add_css_class("success");
            row.add_prefix(&status);

            let play_btn = Button::builder()
                .icon_name("media-playback-start-symbolic")
                .css_classes(vec!["flat"])
                .valign(gtk4::Align::Center)
                .build();
            row.add_suffix(&play_btn);

            let settings_btn = Button::builder()
                .icon_name("emblem-system-symbolic")
                .css_classes(vec!["flat"])
                .valign(gtk4::Align::Center)
                .build();
            row.add_suffix(&settings_btn);
        } else {
            let install_btn = Button::builder()
                .label("Instalar")
                .css_classes(vec!["suggested-action"])
                .valign(gtk4::Align::Center)
                .build();
            row.add_suffix(&install_btn);
        }

        standalone_group.add(&row);
    }

    page.add(&standalone_group);

    section.append(&page);
    section
}

fn create_retroarch_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_bottom(20)
        .build();

    let page = PreferencesPage::new();

    // RetroArch global settings
    let settings_group = PreferencesGroup::builder()
        .title("Configuracoes RetroArch")
        .build();

    // ROM directories
    let rom_dirs = ExpanderRow::builder()
        .title("Diretorios de ROMs")
        .subtitle("Pastas escaneadas para jogos")
        .build();

    let dirs = [
        "~/ROMs",
        "~/Games/ROMs",
        "/media/external/ROMs",
    ];

    for dir in dirs {
        let row = ActionRow::builder()
            .title(dir)
            .build();

        let remove_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(vec!["flat"])
            .valign(gtk4::Align::Center)
            .build();
        row.add_suffix(&remove_btn);

        rom_dirs.add_row(&row);
    }

    let add_dir = ActionRow::builder()
        .title("Adicionar diretorio...")
        .activatable(true)
        .build();
    add_dir.add_suffix(&Image::from_icon_name("list-add-symbolic"));
    rom_dirs.add_row(&add_dir);

    settings_group.add(&rom_dirs);

    // Video settings
    let video_driver = adw::ComboRow::builder()
        .title("Driver de Video")
        .build();
    let drivers = gtk4::StringList::new(&["Vulkan", "OpenGL", "GLCore"]);
    video_driver.set_model(Some(&drivers));
    video_driver.set_selected(0);
    settings_group.add(&video_driver);

    // Shaders
    let shaders = SwitchRow::builder()
        .title("Shaders")
        .subtitle("Aplicar filtros visuais (CRT, Scanlines, etc)")
        .active(true)
        .build();
    settings_group.add(&shaders);

    // Rewind
    let rewind = SwitchRow::builder()
        .title("Rewind")
        .subtitle("Permite voltar no tempo durante o jogo")
        .active(true)
        .build();
    settings_group.add(&rewind);

    // Save states
    let auto_save = SwitchRow::builder()
        .title("Auto-save")
        .subtitle("Salvar estado automaticamente ao sair")
        .active(true)
        .build();
    settings_group.add(&auto_save);

    page.add(&settings_group);

    // Controller mapping
    let controller_group = PreferencesGroup::builder()
        .title("Controles")
        .build();

    let controller_row = ActionRow::builder()
        .title("Mapeamento de Controles")
        .subtitle("Configurar botoes do gamepad")
        .activatable(true)
        .build();
    controller_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    controller_group.add(&controller_row);

    let hotkeys_row = ActionRow::builder()
        .title("Atalhos")
        .subtitle("Configurar hotkeys (salvar, carregar, etc)")
        .activatable(true)
        .build();
    hotkeys_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    controller_group.add(&hotkeys_row);

    page.add(&controller_group);

    // Core updates
    let update_group = PreferencesGroup::builder()
        .title("Atualizacoes")
        .build();

    let update_cores = ActionRow::builder()
        .title("Atualizar Cores")
        .subtitle("Baixar ultimas versoes dos cores")
        .activatable(true)
        .build();

    let update_btn = Button::builder()
        .label("Verificar")
        .css_classes(vec!["suggested-action"])
        .valign(gtk4::Align::Center)
        .build();
    update_cores.add_suffix(&update_btn);
    update_group.add(&update_cores);

    let update_assets = ActionRow::builder()
        .title("Atualizar Assets")
        .subtitle("Baixar overlays, shaders e temas")
        .activatable(true)
        .build();
    update_assets.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    update_group.add(&update_assets);

    page.add(&update_group);

    section.append(&page);
    section
}
