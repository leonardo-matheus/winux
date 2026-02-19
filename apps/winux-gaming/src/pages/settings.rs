// Gaming settings page
// GameMode, MangoHud, performance profiles, per-game settings

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Image, Label, ListBox, ListBoxRow,
    Orientation, Scale, ScrolledWindow, Switch,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, SpinRow, SwitchRow};

pub fn create_settings_page() -> ScrolledWindow {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Header
    let header = create_settings_header();
    main_box.append(&header);

    // Settings page content
    let page = create_settings_content();
    main_box.append(&page);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_settings_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(10)
        .build();

    let title = Label::builder()
        .label("Configuracoes de Gaming")
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    // Spacer
    let spacer = Box::builder().hexpand(true).build();
    header.append(&spacer);

    // Reset defaults button
    let reset_btn = Button::builder()
        .label("Restaurar Padroes")
        .css_classes(vec!["flat"])
        .build();
    header.append(&reset_btn);

    header
}

fn create_settings_content() -> PreferencesPage {
    let page = PreferencesPage::new();

    // GameMode section
    let gamemode_group = create_gamemode_section();
    page.add(&gamemode_group);

    // MangoHud section
    let mangohud_group = create_mangohud_section();
    page.add(&mangohud_group);

    // Performance profiles
    let performance_group = create_performance_section();
    page.add(&performance_group);

    // Wine/Proton global settings
    let wine_group = create_wine_section();
    page.add(&wine_group);

    // Launcher integrations
    let integrations_group = create_integrations_section();
    page.add(&integrations_group);

    // Game-specific settings
    let per_game_group = create_per_game_section();
    page.add(&per_game_group);

    page
}

fn create_gamemode_section() -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title("GameMode")
        .description("Otimizacoes automaticas de sistema durante jogos")
        .build();

    // Enable GameMode
    let enable_gamemode = SwitchRow::builder()
        .title("Habilitar GameMode")
        .subtitle("Ativa automaticamente ao iniciar jogos")
        .active(true)
        .build();
    group.add(&enable_gamemode);

    // CPU Governor
    let cpu_governor = adw::ComboRow::builder()
        .title("Governor da CPU")
        .subtitle("Politica de frequencia durante jogos")
        .build();
    let governors = gtk4::StringList::new(&["performance", "schedutil", "ondemand"]);
    cpu_governor.set_model(Some(&governors));
    cpu_governor.set_selected(0);
    group.add(&cpu_governor);

    // GPU Performance mode
    let gpu_mode = SwitchRow::builder()
        .title("GPU Performance Mode")
        .subtitle("Maximizar clock da GPU durante jogos")
        .active(true)
        .build();
    group.add(&gpu_mode);

    // Disable compositor
    let disable_compositor = SwitchRow::builder()
        .title("Desabilitar Compositor")
        .subtitle("Desativa efeitos visuais para menor latencia")
        .active(false)
        .build();
    group.add(&disable_compositor);

    // Process priority
    let nice_value = SpinRow::builder()
        .title("Prioridade do Processo")
        .subtitle("Nice value para jogos (-20 a 19)")
        .adjustment(&gtk4::Adjustment::new(-10.0, -20.0, 19.0, 1.0, 5.0, 0.0))
        .build();
    group.add(&nice_value);

    // IO Priority
    let io_priority = adw::ComboRow::builder()
        .title("Prioridade de I/O")
        .build();
    let io_classes = gtk4::StringList::new(&["Realtime", "Best-effort", "Idle"]);
    io_priority.set_model(Some(&io_classes));
    io_priority.set_selected(0);
    group.add(&io_priority);

    group
}

fn create_mangohud_section() -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title("MangoHud")
        .description("Overlay de performance em tempo real")
        .build();

    // Enable MangoHud
    let enable_mangohud = SwitchRow::builder()
        .title("Habilitar MangoHud")
        .subtitle("Mostrar overlay de performance")
        .active(true)
        .build();
    group.add(&enable_mangohud);

    // Preset
    let preset = adw::ComboRow::builder()
        .title("Preset")
        .subtitle("Nivel de detalhes do overlay")
        .build();
    let presets = gtk4::StringList::new(&["Minimal", "Default", "Full", "Custom"]);
    preset.set_model(Some(&presets));
    preset.set_selected(1);
    group.add(&preset);

    // Position
    let position = adw::ComboRow::builder()
        .title("Posicao")
        .build();
    let positions = gtk4::StringList::new(&[
        "Canto Superior Esquerdo",
        "Canto Superior Direito",
        "Canto Inferior Esquerdo",
        "Canto Inferior Direito",
    ]);
    position.set_model(Some(&positions));
    position.set_selected(0);
    group.add(&position);

    // Custom options expander
    let custom_options = ExpanderRow::builder()
        .title("Opcoes Personalizadas")
        .subtitle("Configurar metricas exibidas")
        .build();

    let metrics = [
        ("FPS", "Quadros por segundo", true),
        ("Frametime", "Tempo de frame em ms", true),
        ("CPU Usage", "Uso de CPU por core", true),
        ("GPU Usage", "Uso da GPU", true),
        ("VRAM", "Memoria de video", false),
        ("RAM", "Memoria do sistema", false),
        ("CPU Temp", "Temperatura da CPU", true),
        ("GPU Temp", "Temperatura da GPU", true),
        ("Fan Speed", "Velocidade das ventoinhas", false),
        ("Power Draw", "Consumo de energia", false),
    ];

    for (name, desc, enabled) in metrics {
        let row = SwitchRow::builder()
            .title(name)
            .subtitle(desc)
            .active(enabled)
            .build();
        custom_options.add_row(&row);
    }

    group.add(&custom_options);

    // Log to file
    let log_to_file = SwitchRow::builder()
        .title("Salvar Logs")
        .subtitle("Gravar metricas em arquivo para analise")
        .active(false)
        .build();
    group.add(&log_to_file);

    // Toggle hotkey
    let hotkey = ActionRow::builder()
        .title("Tecla de Atalho")
        .subtitle("Shift+F12")
        .activatable(true)
        .build();
    hotkey.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    group.add(&hotkey);

    group
}

fn create_performance_section() -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title("Perfis de Performance")
        .description("Presets de otimizacao para diferentes cenarios")
        .build();

    let profiles = [
        ("Bateria", "Economiza energia, menor desempenho", "battery-level-90-symbolic"),
        ("Balanceado", "Equilibrio entre performance e consumo", "speedometer-symbolic"),
        ("Performance", "Maximo desempenho, maior consumo", "utilities-system-monitor-symbolic"),
        ("Gaming", "Otimizado para jogos com GameMode", "input-gaming-symbolic"),
    ];

    for (i, (name, desc, icon)) in profiles.iter().enumerate() {
        let row = ActionRow::builder()
            .title(*name)
            .subtitle(*desc)
            .activatable(true)
            .build();

        row.add_prefix(&Image::from_icon_name(*icon));

        let radio = gtk4::CheckButton::new();
        if i == 3 {
            radio.set_active(true);
        }
        row.add_suffix(&radio);

        group.add(&row);
    }

    // Frame limiter
    let frame_limiter = adw::ComboRow::builder()
        .title("Limitador de FPS")
        .subtitle("Limitar FPS globalmente")
        .build();
    let limits = gtk4::StringList::new(&["Sem limite", "30 FPS", "60 FPS", "90 FPS", "120 FPS", "144 FPS", "V-Sync"]);
    frame_limiter.set_model(Some(&limits));
    frame_limiter.set_selected(0);
    group.add(&frame_limiter);

    group
}

fn create_wine_section() -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title("Wine/Proton Global")
        .description("Configuracoes padrao para jogos Windows")
        .build();

    // Default Wine
    let default_wine = adw::ComboRow::builder()
        .title("Wine Padrao")
        .subtitle("Usado quando nenhum especifico e definido")
        .build();
    let wines = gtk4::StringList::new(&[
        "Wine-GE 8-26",
        "Wine 9.0",
        "Proton-GE 8-25",
        "Proton Experimental",
    ]);
    default_wine.set_model(Some(&wines));
    default_wine.set_selected(0);
    group.add(&default_wine);

    // DXVK
    let dxvk = SwitchRow::builder()
        .title("DXVK")
        .subtitle("Traducao DirectX 9-11 para Vulkan")
        .active(true)
        .build();
    group.add(&dxvk);

    // VKD3D
    let vkd3d = SwitchRow::builder()
        .title("VKD3D-Proton")
        .subtitle("Traducao DirectX 12 para Vulkan")
        .active(true)
        .build();
    group.add(&vkd3d);

    // DXVK Async
    let dxvk_async = SwitchRow::builder()
        .title("DXVK Async")
        .subtitle("Compilacao assincrona de shaders")
        .active(false)
        .build();
    group.add(&dxvk_async);

    // FSR
    let fsr = SwitchRow::builder()
        .title("AMD FSR Global")
        .subtitle("FidelityFX Super Resolution para todos os jogos")
        .active(false)
        .build();
    group.add(&fsr);

    // Esync/Fsync
    let sync = adw::ComboRow::builder()
        .title("Sincronizacao")
        .subtitle("Metodo de sincronizacao de threads")
        .build();
    let sync_options = gtk4::StringList::new(&["Fsync", "Esync", "Wineserver"]);
    sync.set_model(Some(&sync_options));
    sync.set_selected(0);
    group.add(&sync);

    // Environment variables
    let env_vars = ActionRow::builder()
        .title("Variaveis de Ambiente")
        .subtitle("Configurar variaveis customizadas")
        .activatable(true)
        .build();
    env_vars.add_suffix(&Image::from_icon_name("go-next-symbolic"));
    group.add(&env_vars);

    group
}

fn create_integrations_section() -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title("Integracoes")
        .description("Configurar launchers e servicos externos")
        .build();

    // Steam
    let steam = ActionRow::builder()
        .title("Steam")
        .subtitle("Configurado - 186 jogos")
        .build();
    steam.add_prefix(&Image::from_icon_name("emblem-ok-symbolic"));
    let steam_btn = Button::builder()
        .label("Configurar")
        .css_classes(vec!["flat"])
        .valign(gtk4::Align::Center)
        .build();
    steam.add_suffix(&steam_btn);
    group.add(&steam);

    // Heroic
    let heroic = ActionRow::builder()
        .title("Heroic Games Launcher")
        .subtitle("Configurado - GOG + Epic")
        .build();
    heroic.add_prefix(&Image::from_icon_name("emblem-ok-symbolic"));
    let heroic_btn = Button::builder()
        .label("Configurar")
        .css_classes(vec!["flat"])
        .valign(gtk4::Align::Center)
        .build();
    heroic.add_suffix(&heroic_btn);
    group.add(&heroic);

    // Lutris
    let lutris = ActionRow::builder()
        .title("Lutris")
        .subtitle("Nao instalado")
        .build();
    let lutris_btn = Button::builder()
        .label("Instalar")
        .css_classes(vec!["suggested-action"])
        .valign(gtk4::Align::Center)
        .build();
    lutris.add_suffix(&lutris_btn);
    group.add(&lutris);

    // Discord integration
    let discord = SwitchRow::builder()
        .title("Discord Rich Presence")
        .subtitle("Mostrar jogo atual no Discord")
        .active(true)
        .build();
    group.add(&discord);

    group
}

fn create_per_game_section() -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title("Configuracoes por Jogo")
        .description("Override de configuracoes para jogos especificos")
        .build();

    let games_with_overrides = [
        ("Cyberpunk 2077", "Proton-GE 8-25, FSR, MangoHud"),
        ("Elden Ring", "Proton 8.0-4, GameMode"),
        ("Red Dead Redemption 2", "Proton-GE 8-25, DXVK Async"),
    ];

    for (game, settings) in games_with_overrides {
        let row = ActionRow::builder()
            .title(game)
            .subtitle(settings)
            .activatable(true)
            .build();

        let edit_btn = Button::builder()
            .icon_name("document-edit-symbolic")
            .css_classes(vec!["flat"])
            .valign(gtk4::Align::Center)
            .build();
        row.add_suffix(&edit_btn);

        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(vec!["flat"])
            .valign(gtk4::Align::Center)
            .build();
        row.add_suffix(&delete_btn);

        group.add(&row);
    }

    // Add game override
    let add_override = ActionRow::builder()
        .title("Adicionar Configuracao")
        .subtitle("Criar override para um jogo especifico")
        .activatable(true)
        .build();
    add_override.add_suffix(&Image::from_icon_name("list-add-symbolic"));
    group.add(&add_override);

    group
}
