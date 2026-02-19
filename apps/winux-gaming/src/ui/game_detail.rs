// Game detail view
// Full page view with game info, settings, and launch options

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Frame, Image, Label, Notebook, Orientation,
    ScrolledWindow, Separator,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, SwitchRow};

use super::game_card::{GameInfo, Platform};

/// Create a detailed game view
pub fn create_game_detail(game: &GameInfo) -> Box {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Hero header with background
    let header = create_hero_header(game);
    main_box.append(&header);

    // Tabbed content
    let notebook = create_detail_tabs(game);
    main_box.append(&notebook);

    main_box
}

fn create_hero_header(game: &GameInfo) -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(30)
        .margin_start(30)
        .margin_end(30)
        .margin_top(30)
        .margin_bottom(30)
        .css_classes(vec!["featured-banner"])
        .build();

    // Large cover
    let cover_frame = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let cover = Box::builder()
        .width_request(200)
        .height_request(280)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let cover_label = Label::builder()
        .label(&game.cover_icon)
        .css_classes(vec!["title-1"])
        .build();
    cover.append(&cover_label);
    cover_frame.set_child(Some(&cover));
    header.append(&cover_frame);

    // Game info
    let info = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .valign(gtk4::Align::Center)
        .hexpand(true)
        .build();

    // Title
    let title = Label::builder()
        .label(&game.name)
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    info.append(&title);

    // Platform badges
    let badges = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let platform_badge = Label::builder()
        .label(game.platform.display_name())
        .css_classes(vec!["platform-badge", game.platform.css_class()])
        .build();
    badges.append(&platform_badge);

    if game.native {
        let native_badge = Label::builder()
            .label("Native Linux")
            .css_classes(vec!["platform-badge", "platform-native"])
            .build();
        badges.append(&native_badge);
    }

    if game.installed {
        let installed_badge = Label::builder()
            .label("Instalado")
            .css_classes(vec!["success", "caption"])
            .build();
        badges.append(&installed_badge);
    }

    info.append(&badges);

    // Stats
    let stats = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(30)
        .margin_top(12)
        .build();

    if game.playtime_hours > 0.0 {
        let playtime_box = create_stat_box(&game.playtime_formatted(), "Tempo de Jogo");
        stats.append(&playtime_box);
    }

    if let Some(ref last_played) = game.last_played {
        let last_played_box = create_stat_box(last_played, "Ultimo Acesso");
        stats.append(&last_played_box);
    }

    info.append(&stats);

    // Action buttons
    let buttons = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(20)
        .build();

    let play_btn = Button::builder()
        .label(if game.installed { "JOGAR" } else { "INSTALAR" })
        .css_classes(vec!["play-button"])
        .build();
    buttons.append(&play_btn);

    if game.installed {
        let update_btn = Button::builder()
            .label("Verificar Atualizacoes")
            .css_classes(vec!["flat"])
            .build();
        buttons.append(&update_btn);

        let uninstall_btn = Button::builder()
            .label("Desinstalar")
            .css_classes(vec!["flat", "destructive-action"])
            .build();
        buttons.append(&uninstall_btn);
    }

    info.append(&buttons);
    header.append(&info);

    header
}

fn create_stat_box(value: &str, label: &str) -> Box {
    let stat_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();

    let value_label = Label::builder()
        .label(value)
        .css_classes(vec!["stat-value"])
        .halign(gtk4::Align::Start)
        .build();
    stat_box.append(&value_label);

    let label_label = Label::builder()
        .label(label)
        .css_classes(vec!["stat-label"])
        .halign(gtk4::Align::Start)
        .build();
    stat_box.append(&label_label);

    stat_box
}

fn create_detail_tabs(game: &GameInfo) -> Notebook {
    let notebook = Notebook::builder()
        .tab_pos(gtk4::PositionType::Top)
        .vexpand(true)
        .build();

    // Overview tab
    let overview = create_overview_tab(game);
    notebook.append_page(&overview, Some(&Label::new(Some("Visao Geral"))));

    // Settings tab
    let settings = create_settings_tab(game);
    notebook.append_page(&settings, Some(&Label::new(Some("Configuracoes"))));

    // Wine/Proton tab (if applicable)
    if !game.native {
        let wine_tab = create_wine_tab(game);
        notebook.append_page(&wine_tab, Some(&Label::new(Some("Wine/Proton"))));
    }

    // Performance tab
    let performance = create_performance_tab(game);
    notebook.append_page(&performance, Some(&Label::new(Some("Performance"))));

    notebook
}

fn create_overview_tab(game: &GameInfo) -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Game info
    let info_group = PreferencesGroup::builder()
        .title("Informacoes")
        .build();

    let id_row = ActionRow::builder()
        .title("ID do Jogo")
        .subtitle(&game.id)
        .build();
    info_group.add(&id_row);

    let platform_row = ActionRow::builder()
        .title("Plataforma")
        .subtitle(game.platform.display_name())
        .build();
    info_group.add(&platform_row);

    let compat_row = ActionRow::builder()
        .title("Compatibilidade")
        .subtitle(if game.native { "Nativo Linux" } else { "Via Proton/Wine" })
        .build();
    info_group.add(&compat_row);

    page.add(&info_group);

    // Storage info
    if game.installed {
        let storage_group = PreferencesGroup::builder()
            .title("Armazenamento")
            .build();

        let size_row = ActionRow::builder()
            .title("Tamanho")
            .subtitle("45.2 GB") // Placeholder
            .build();
        storage_group.add(&size_row);

        let location_row = ActionRow::builder()
            .title("Localizacao")
            .subtitle("~/.local/share/Steam/steamapps/common/...")
            .activatable(true)
            .build();
        location_row.add_suffix(&Image::from_icon_name("folder-symbolic"));
        storage_group.add(&location_row);

        page.add(&storage_group);
    }

    // Quick actions
    let actions_group = PreferencesGroup::builder()
        .title("Acoes Rapidas")
        .build();

    let desktop_row = ActionRow::builder()
        .title("Criar Atalho na Area de Trabalho")
        .activatable(true)
        .build();
    desktop_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    actions_group.add(&desktop_row);

    let properties_row = ActionRow::builder()
        .title("Propriedades do Jogo")
        .activatable(true)
        .build();
    properties_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    actions_group.add(&properties_row);

    page.add(&actions_group);

    ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build()
}

fn create_settings_tab(game: &GameInfo) -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Launch options
    let launch_group = PreferencesGroup::builder()
        .title("Opcoes de Lancamento")
        .build();

    let gamemode_row = SwitchRow::builder()
        .title("GameMode")
        .subtitle("Ativar otimizacoes automaticas")
        .active(true)
        .build();
    launch_group.add(&gamemode_row);

    let mangohud_row = SwitchRow::builder()
        .title("MangoHud")
        .subtitle("Mostrar overlay de performance")
        .active(false)
        .build();
    launch_group.add(&mangohud_row);

    let args_row = ActionRow::builder()
        .title("Argumentos de Linha de Comando")
        .subtitle("Nenhum")
        .activatable(true)
        .build();
    args_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    launch_group.add(&args_row);

    page.add(&launch_group);

    // Environment variables
    let env_group = PreferencesGroup::builder()
        .title("Variaveis de Ambiente")
        .build();

    let env_row = ActionRow::builder()
        .title("Variaveis Customizadas")
        .subtitle("0 variaveis definidas")
        .activatable(true)
        .build();
    env_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    env_group.add(&env_row);

    page.add(&env_group);

    ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build()
}

fn create_wine_tab(game: &GameInfo) -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Proton/Wine version
    let version_group = PreferencesGroup::builder()
        .title("Versao do Wine/Proton")
        .build();

    let version_row = adw::ComboRow::builder()
        .title("Versao")
        .subtitle("Selecione a versao do Wine/Proton")
        .build();
    let versions = gtk4::StringList::new(&[
        "Usar Padrao",
        "Proton Experimental",
        "Proton 8.0-4",
        "Proton-GE 8-25",
        "Wine-GE 8-26",
    ]);
    version_row.set_model(Some(&versions));
    version_group.add(&version_row);

    page.add(&version_group);

    // Prefix management
    let prefix_group = PreferencesGroup::builder()
        .title("Prefixo Wine")
        .build();

    let prefix_path = ActionRow::builder()
        .title("Localizacao do Prefixo")
        .subtitle("~/.steam/steam/steamapps/compatdata/...")
        .activatable(true)
        .build();
    prefix_path.add_suffix(&Image::from_icon_name("folder-symbolic"));
    prefix_group.add(&prefix_path);

    let winecfg_row = ActionRow::builder()
        .title("Wine Configuration")
        .subtitle("Abrir winecfg")
        .activatable(true)
        .build();
    winecfg_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    prefix_group.add(&winecfg_row);

    let winetricks_row = ActionRow::builder()
        .title("Winetricks")
        .subtitle("Instalar componentes Windows")
        .activatable(true)
        .build();
    winetricks_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    prefix_group.add(&winetricks_row);

    let delete_prefix = ActionRow::builder()
        .title("Deletar Prefixo")
        .subtitle("Remove todos os dados do Wine para este jogo")
        .activatable(true)
        .build();
    delete_prefix.add_css_class("error");
    prefix_group.add(&delete_prefix);

    page.add(&prefix_group);

    // Compatibility options
    let compat_group = PreferencesGroup::builder()
        .title("Compatibilidade")
        .build();

    let dxvk_row = SwitchRow::builder()
        .title("DXVK")
        .subtitle("DirectX 9-11 para Vulkan")
        .active(true)
        .build();
    compat_group.add(&dxvk_row);

    let vkd3d_row = SwitchRow::builder()
        .title("VKD3D-Proton")
        .subtitle("DirectX 12 para Vulkan")
        .active(true)
        .build();
    compat_group.add(&vkd3d_row);

    let fsr_row = SwitchRow::builder()
        .title("AMD FSR")
        .subtitle("Upscaling via FidelityFX")
        .active(false)
        .build();
    compat_group.add(&fsr_row);

    page.add(&compat_group);

    ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build()
}

fn create_performance_tab(game: &GameInfo) -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Performance profile
    let profile_group = PreferencesGroup::builder()
        .title("Perfil de Performance")
        .build();

    let profile_row = adw::ComboRow::builder()
        .title("Perfil")
        .build();
    let profiles = gtk4::StringList::new(&[
        "Usar Padrao Global",
        "Balanceado",
        "Performance",
        "Economia de Energia",
    ]);
    profile_row.set_model(Some(&profiles));
    profile_group.add(&profile_row);

    page.add(&profile_group);

    // Frame limit
    let frame_group = PreferencesGroup::builder()
        .title("Limitacao de FPS")
        .build();

    let fps_row = adw::ComboRow::builder()
        .title("Limite de FPS")
        .build();
    let fps_options = gtk4::StringList::new(&[
        "Sem Limite",
        "30 FPS",
        "60 FPS",
        "90 FPS",
        "120 FPS",
        "144 FPS",
    ]);
    fps_row.set_model(Some(&fps_options));
    frame_group.add(&fps_row);

    let vsync_row = SwitchRow::builder()
        .title("V-Sync")
        .subtitle("Sincronizacao vertical")
        .active(false)
        .build();
    frame_group.add(&vsync_row);

    page.add(&frame_group);

    // MangoHud for this game
    let hud_group = PreferencesGroup::builder()
        .title("MangoHud")
        .build();

    let hud_enable = SwitchRow::builder()
        .title("Habilitar MangoHud")
        .active(false)
        .build();
    hud_group.add(&hud_enable);

    let hud_preset = adw::ComboRow::builder()
        .title("Preset")
        .build();
    let presets = gtk4::StringList::new(&["Minimal", "Default", "Full"]);
    hud_preset.set_model(Some(&presets));
    hud_preset.set_selected(1);
    hud_group.add(&hud_preset);

    page.add(&hud_group);

    ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build()
}
