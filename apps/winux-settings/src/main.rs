// Winux Settings - System Configuration Center
// Copyright (c) 2026 Winux OS Project
//
// Complete system settings with:
// - Network (WiFi, Ethernet, VPN)
// - Bluetooth
// - Performance Mode
// - Appearance
// - Display
// - Sound
// - About

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Label, ListBox, Orientation, Scale, Switch,
    ComboBoxText, Spinner, ProgressBar,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, ApplicationWindow, HeaderBar, PreferencesGroup, PreferencesPage,
    StatusPage, SwitchRow, ViewStack, ViewSwitcher, ExpanderRow, ComboRow,
    SpinRow,
};
use std::process::Command;
use std::sync::Arc;
use std::cell::RefCell;

const APP_ID: &str = "org.winux.Settings";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    let stack = ViewStack::new();
    stack.set_vexpand(true);

    // Network Page
    let network_page = create_network_page();
    stack.add_titled(&network_page, Some("network"), "Rede")
        .set_icon_name(Some("network-wireless-symbolic"));

    // Bluetooth Page
    let bluetooth_page = create_bluetooth_page();
    stack.add_titled(&bluetooth_page, Some("bluetooth"), "Bluetooth")
        .set_icon_name(Some("bluetooth-active-symbolic"));

    // Performance Page
    let performance_page = create_performance_page();
    stack.add_titled(&performance_page, Some("performance"), "Desempenho")
        .set_icon_name(Some("speedometer-symbolic"));

    // Appearance Page
    let appearance_page = create_appearance_page();
    stack.add_titled(&appearance_page, Some("appearance"), "Aparencia")
        .set_icon_name(Some("preferences-desktop-appearance-symbolic"));

    // Display Page
    let display_page = create_display_page();
    stack.add_titled(&display_page, Some("display"), "Tela")
        .set_icon_name(Some("preferences-desktop-display-symbolic"));

    // Sound Page
    let sound_page = create_sound_page();
    stack.add_titled(&sound_page, Some("sound"), "Som")
        .set_icon_name(Some("audio-speakers-symbolic"));

    // Power Page
    let power_page = create_power_page();
    stack.add_titled(&power_page, Some("power"), "Energia")
        .set_icon_name(Some("battery-full-symbolic"));

    // Language Page
    let language_page = create_language_page();
    stack.add_titled(&language_page, Some("language"), "Idioma")
        .set_icon_name(Some("preferences-desktop-locale-symbolic"));

    // About Page
    let about_page = create_about_page();
    stack.add_titled(&about_page, Some("about"), "Sobre")
        .set_icon_name(Some("help-about-symbolic"));

    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&stack);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Configuracoes")
        .default_width(1000)
        .default_height(700)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));
    window.present();
}

fn create_network_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // WiFi Section
    let wifi_group = PreferencesGroup::builder()
        .title("Wi-Fi")
        .description("Conecte-se a redes sem fio")
        .build();

    let wifi_switch = SwitchRow::builder()
        .title("Wi-Fi")
        .subtitle("Habilitar conexao sem fio")
        .active(true)
        .build();
    wifi_group.add(&wifi_switch);

    // Available networks
    let networks_expander = ExpanderRow::builder()
        .title("Redes Disponiveis")
        .subtitle("Clique para ver redes proximas")
        .build();

    let network_names = ["Casa_5G", "Vizinho_Net", "Cafe_WiFi", "Winux_Office"];
    for name in network_names {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(if name == "Casa_5G" { "Conectado" } else { "Disponivel" })
            .build();

        let signal_icon = gtk4::Image::from_icon_name("network-wireless-signal-excellent-symbolic");
        row.add_prefix(&signal_icon);

        if name != "Casa_5G" {
            let connect_btn = Button::with_label("Conectar");
            connect_btn.set_valign(gtk4::Align::Center);
            connect_btn.add_css_class("flat");
            row.add_suffix(&connect_btn);
        } else {
            let check = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            check.add_css_class("success");
            row.add_suffix(&check);
        }

        networks_expander.add_row(&row);
    }

    wifi_group.add(&networks_expander);
    page.add(&wifi_group);

    // Ethernet Section
    let eth_group = PreferencesGroup::builder()
        .title("Ethernet")
        .description("Conexao com fio")
        .build();

    let eth_row = ActionRow::builder()
        .title("Ethernet (eth0)")
        .subtitle("Cabo nao conectado")
        .build();

    let eth_icon = gtk4::Image::from_icon_name("network-wired-symbolic");
    eth_row.add_prefix(&eth_icon);
    eth_group.add(&eth_row);

    page.add(&eth_group);

    // VPN Section
    let vpn_group = PreferencesGroup::builder()
        .title("VPN")
        .description("Redes privadas virtuais")
        .build();

    let add_vpn = ActionRow::builder()
        .title("Adicionar VPN...")
        .activatable(true)
        .build();
    add_vpn.add_suffix(&gtk4::Image::from_icon_name("list-add-symbolic"));
    vpn_group.add(&add_vpn);

    page.add(&vpn_group);

    // Proxy Section
    let proxy_group = PreferencesGroup::builder()
        .title("Proxy")
        .build();

    let proxy_row = ComboRow::builder()
        .title("Modo de Proxy")
        .subtitle("Configuracao de proxy de rede")
        .build();
    let proxy_model = gtk4::StringList::new(&["Nenhum", "Manual", "Automatico"]);
    proxy_row.set_model(Some(&proxy_model));
    proxy_group.add(&proxy_row);

    page.add(&proxy_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_bluetooth_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Bluetooth toggle
    let bt_group = PreferencesGroup::builder()
        .title("Bluetooth")
        .build();

    let bt_switch = SwitchRow::builder()
        .title("Bluetooth")
        .subtitle("Habilitar Bluetooth")
        .active(true)
        .build();
    bt_group.add(&bt_switch);

    let visibility = SwitchRow::builder()
        .title("Visibilidade")
        .subtitle("Permitir que outros dispositivos encontrem este computador")
        .active(false)
        .build();
    bt_group.add(&visibility);

    page.add(&bt_group);

    // Connected devices
    let connected_group = PreferencesGroup::builder()
        .title("Dispositivos Conectados")
        .build();

    let headphones = ActionRow::builder()
        .title("Fones Bluetooth")
        .subtitle("Conectado - Audio")
        .build();
    headphones.add_prefix(&gtk4::Image::from_icon_name("audio-headphones-symbolic"));

    let disconnect_btn = Button::with_label("Desconectar");
    disconnect_btn.add_css_class("flat");
    disconnect_btn.set_valign(gtk4::Align::Center);
    headphones.add_suffix(&disconnect_btn);
    connected_group.add(&headphones);

    page.add(&connected_group);

    // Available devices
    let available_group = PreferencesGroup::builder()
        .title("Dispositivos Disponiveis")
        .description("Clique em um dispositivo para parear")
        .build();

    let devices = [
        ("Mouse MX Master", "mouse-symbolic"),
        ("Teclado K380", "input-keyboard-symbolic"),
        ("Galaxy Buds", "audio-headphones-symbolic"),
    ];

    for (name, icon) in devices {
        let row = ActionRow::builder()
            .title(name)
            .subtitle("Nao pareado")
            .activatable(true)
            .build();
        row.add_prefix(&gtk4::Image::from_icon_name(icon));

        let pair_btn = Button::with_label("Parear");
        pair_btn.add_css_class("flat");
        pair_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&pair_btn);
        available_group.add(&row);
    }

    let scan_btn = Button::with_label("Buscar Dispositivos");
    scan_btn.add_css_class("pill");
    scan_btn.set_halign(gtk4::Align::Center);
    scan_btn.set_margin_top(12);

    let scan_box = Box::new(Orientation::Vertical, 0);
    scan_box.append(&scan_btn);
    available_group.add(&scan_box);

    page.add(&available_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_performance_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Performance mode
    let mode_group = PreferencesGroup::builder()
        .title("Modo de Desempenho")
        .description("Ajuste o equilibrio entre desempenho e consumo de energia")
        .build();

    let modes = [
        ("Economico", "Maximiza duracao da bateria, reduz desempenho"),
        ("Balanceado", "Equilibrio entre desempenho e eficiencia"),
        ("Alto Desempenho", "Maximo desempenho, maior consumo de energia"),
        ("Gaming", "Otimizado para jogos com GameMode habilitado"),
    ];

    for (i, (name, desc)) in modes.iter().enumerate() {
        let row = ActionRow::builder()
            .title(*name)
            .subtitle(*desc)
            .activatable(true)
            .build();

        let radio = gtk4::CheckButton::new();
        if i == 1 {
            radio.set_active(true);
        }
        row.add_prefix(&radio);

        let icon = match i {
            0 => "battery-level-90-symbolic",
            1 => "speedometer-symbolic",
            2 => "utilities-system-monitor-symbolic",
            3 => "input-gaming-symbolic",
            _ => "preferences-system-symbolic",
        };
        row.add_suffix(&gtk4::Image::from_icon_name(icon));
        mode_group.add(&row);
    }

    page.add(&mode_group);

    // CPU Governor
    let cpu_group = PreferencesGroup::builder()
        .title("Processador")
        .description("Configuracoes avancadas de CPU")
        .build();

    let governor_row = ComboRow::builder()
        .title("Governor da CPU")
        .subtitle("Politica de escalonamento de frequencia")
        .build();
    let governors = gtk4::StringList::new(&["powersave", "ondemand", "performance", "schedutil"]);
    governor_row.set_model(Some(&governors));
    governor_row.set_selected(1);
    cpu_group.add(&governor_row);

    let turbo_row = SwitchRow::builder()
        .title("Turbo Boost")
        .subtitle("Permitir frequencias acima do base clock")
        .active(true)
        .build();
    cpu_group.add(&turbo_row);

    page.add(&cpu_group);

    // GPU Settings
    let gpu_group = PreferencesGroup::builder()
        .title("Placa de Video")
        .description("Configuracoes de GPU")
        .build();

    let gpu_profile = ComboRow::builder()
        .title("Perfil de Energia")
        .build();
    let profiles = gtk4::StringList::new(&["Automatico", "Baixo Consumo", "Alto Desempenho"]);
    gpu_profile.set_model(Some(&profiles));
    gpu_group.add(&gpu_profile);

    let vsync_row = SwitchRow::builder()
        .title("V-Sync Global")
        .subtitle("Sincronizacao vertical para toda a GPU")
        .active(true)
        .build();
    gpu_group.add(&vsync_row);

    page.add(&gpu_group);

    // Memory
    let mem_group = PreferencesGroup::builder()
        .title("Memoria")
        .build();

    let swap_row = SpinRow::builder()
        .title("Swappiness")
        .subtitle("Tendencia de usar swap (0-100)")
        .adjustment(&gtk4::Adjustment::new(60.0, 0.0, 100.0, 1.0, 10.0, 0.0))
        .build();
    mem_group.add(&swap_row);

    let zram_row = SwitchRow::builder()
        .title("ZRAM")
        .subtitle("Compressao de memoria RAM")
        .active(true)
        .build();
    mem_group.add(&zram_row);

    page.add(&mem_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_appearance_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Theme
    let theme_group = PreferencesGroup::builder()
        .title("Tema")
        .build();

    let style_row = ComboRow::builder()
        .title("Estilo")
        .build();
    let styles = gtk4::StringList::new(&["Escuro", "Claro", "Automatico"]);
    style_row.set_model(Some(&styles));
    theme_group.add(&style_row);

    page.add(&theme_group);

    // Accent color
    let accent_group = PreferencesGroup::builder()
        .title("Cor de Destaque")
        .build();

    let colors = ["Azul", "Verde", "Roxo", "Rosa", "Laranja"];
    for color in colors {
        let row = ActionRow::builder()
            .title(color)
            .activatable(true)
            .build();
        row.add_prefix(&gtk4::CheckButton::new());
        accent_group.add(&row);
    }

    page.add(&accent_group);

    // Fonts
    let font_group = PreferencesGroup::builder()
        .title("Fontes")
        .build();

    let interface_font = ActionRow::builder()
        .title("Fonte da Interface")
        .subtitle("Noto Sans 11")
        .activatable(true)
        .build();
    interface_font.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    font_group.add(&interface_font);

    let mono_font = ActionRow::builder()
        .title("Fonte Monoespaco")
        .subtitle("JetBrains Mono 10")
        .activatable(true)
        .build();
    mono_font.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    font_group.add(&mono_font);

    page.add(&font_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_display_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Resolution
    let display_group = PreferencesGroup::builder()
        .title("Tela Principal")
        .build();

    let resolution_row = ComboRow::builder()
        .title("Resolucao")
        .build();
    let resolutions = gtk4::StringList::new(&["3840x2160", "2560x1440", "1920x1080", "1366x768"]);
    resolution_row.set_model(Some(&resolutions));
    resolution_row.set_selected(2);
    display_group.add(&resolution_row);

    let refresh_row = ComboRow::builder()
        .title("Taxa de Atualizacao")
        .build();
    let rates = gtk4::StringList::new(&["60 Hz", "75 Hz", "120 Hz", "144 Hz", "165 Hz"]);
    refresh_row.set_model(Some(&rates));
    display_group.add(&refresh_row);

    let scale_row = ComboRow::builder()
        .title("Escala")
        .build();
    let scales = gtk4::StringList::new(&["100%", "125%", "150%", "175%", "200%"]);
    scale_row.set_model(Some(&scales));
    display_group.add(&scale_row);

    page.add(&display_group);

    // Night Light
    let night_group = PreferencesGroup::builder()
        .title("Luz Noturna")
        .description("Reduz luz azul para conforto visual")
        .build();

    let night_switch = SwitchRow::builder()
        .title("Luz Noturna")
        .active(false)
        .build();
    night_group.add(&night_switch);

    let schedule_row = ComboRow::builder()
        .title("Horario")
        .build();
    let schedules = gtk4::StringList::new(&["Por do Sol ao Nascer", "Manual", "Desligado"]);
    schedule_row.set_model(Some(&schedules));
    night_group.add(&schedule_row);

    page.add(&night_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_sound_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Output
    let output_group = PreferencesGroup::builder()
        .title("Saida de Audio")
        .build();

    let output_row = ComboRow::builder()
        .title("Dispositivo de Saida")
        .build();
    let outputs = gtk4::StringList::new(&["Alto-falantes Internos", "HDMI Audio", "Fones Bluetooth"]);
    output_row.set_model(Some(&outputs));
    output_group.add(&output_row);

    let vol_row = ActionRow::builder()
        .title("Volume")
        .build();

    let vol_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    vol_scale.set_value(75.0);
    vol_scale.set_hexpand(true);
    vol_scale.set_size_request(200, -1);
    vol_row.add_suffix(&vol_scale);
    output_group.add(&vol_row);

    page.add(&output_group);

    // Input
    let input_group = PreferencesGroup::builder()
        .title("Entrada de Audio")
        .build();

    let input_row = ComboRow::builder()
        .title("Microfone")
        .build();
    let inputs = gtk4::StringList::new(&["Microfone Interno", "USB Microphone"]);
    input_row.set_model(Some(&inputs));
    input_group.add(&input_row);

    let input_vol_row = ActionRow::builder()
        .title("Nivel do Microfone")
        .build();

    let input_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    input_scale.set_value(80.0);
    input_scale.set_hexpand(true);
    input_scale.set_size_request(200, -1);
    input_vol_row.add_suffix(&input_scale);
    input_group.add(&input_vol_row);

    page.add(&input_group);

    // Alerts
    let alerts_group = PreferencesGroup::builder()
        .title("Sons do Sistema")
        .build();

    let alerts_switch = SwitchRow::builder()
        .title("Sons de Alerta")
        .active(true)
        .build();
    alerts_group.add(&alerts_switch);

    page.add(&alerts_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_power_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Battery
    let battery_group = PreferencesGroup::builder()
        .title("Bateria")
        .build();

    let battery_row = ActionRow::builder()
        .title("Nivel da Bateria")
        .subtitle("Carregando - 75%")
        .build();
    battery_row.add_prefix(&gtk4::Image::from_icon_name("battery-level-80-charging-symbolic"));

    let progress = ProgressBar::new();
    progress.set_fraction(0.75);
    progress.set_valign(gtk4::Align::Center);
    progress.set_size_request(100, -1);
    battery_row.add_suffix(&progress);
    battery_group.add(&battery_row);

    page.add(&battery_group);

    // Power saving
    let power_group = PreferencesGroup::builder()
        .title("Economia de Energia")
        .build();

    let auto_brightness = SwitchRow::builder()
        .title("Brilho Automatico")
        .subtitle("Ajusta brilho baseado na luz ambiente")
        .active(true)
        .build();
    power_group.add(&auto_brightness);

    let dim_row = SwitchRow::builder()
        .title("Escurecer Tela")
        .subtitle("Reduz brilho quando inativo")
        .active(true)
        .build();
    power_group.add(&dim_row);

    page.add(&power_group);

    // Suspend
    let suspend_group = PreferencesGroup::builder()
        .title("Suspensao")
        .build();

    let blank_row = ComboRow::builder()
        .title("Desligar Tela")
        .subtitle("Tempo de inatividade para desligar tela")
        .build();
    let times = gtk4::StringList::new(&["1 minuto", "2 minutos", "5 minutos", "10 minutos", "Nunca"]);
    blank_row.set_model(Some(&times));
    blank_row.set_selected(2);
    suspend_group.add(&blank_row);

    let suspend_row = ComboRow::builder()
        .title("Suspensao Automatica")
        .build();
    let suspend_times = gtk4::StringList::new(&["15 minutos", "30 minutos", "1 hora", "Nunca"]);
    suspend_row.set_model(Some(&suspend_times));
    suspend_row.set_selected(1);
    suspend_group.add(&suspend_row);

    page.add(&suspend_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_about_page() -> gtk4::ScrolledWindow {
    let content = Box::new(Orientation::Vertical, 0);

    let status = StatusPage::builder()
        .icon_name("computer-symbolic")
        .title("Winux OS")
        .description("Developer Edition")
        .build();

    content.append(&status);

    let page = PreferencesPage::new();

    // System info
    let sys_group = PreferencesGroup::builder()
        .title("Sistema")
        .build();

    let info = [
        ("Nome do Dispositivo", get_hostname()),
        ("Versao", "Winux OS 1.0 Aurora".to_string()),
        ("Kernel", get_kernel_version()),
        ("Tipo do Sistema", "64-bit".to_string()),
    ];

    for (label, value) in info {
        let row = ActionRow::builder()
            .title(label)
            .subtitle(&value)
            .build();
        sys_group.add(&row);
    }

    page.add(&sys_group);

    // Hardware
    let hw_group = PreferencesGroup::builder()
        .title("Hardware")
        .build();

    let hw_info = [
        ("Processador", get_cpu_info()),
        ("Memoria", get_memory_info()),
        ("Graficos", get_gpu_info()),
        ("Disco", get_disk_info()),
    ];

    for (label, value) in hw_info {
        let row = ActionRow::builder()
            .title(label)
            .subtitle(&value)
            .build();
        hw_group.add(&row);
    }

    page.add(&hw_group);

    // Links
    let links_group = PreferencesGroup::builder()
        .title("Sobre")
        .build();

    let website = ActionRow::builder()
        .title("Website")
        .subtitle("https://winux.org")
        .activatable(true)
        .build();
    website.add_suffix(&gtk4::Image::from_icon_name("external-link-symbolic"));
    links_group.add(&website);

    let github = ActionRow::builder()
        .title("Codigo Fonte")
        .subtitle("github.com/winux-os")
        .activatable(true)
        .build();
    github.add_suffix(&gtk4::Image::from_icon_name("external-link-symbolic"));
    links_group.add(&github);

    page.add(&links_group);

    content.append(&page);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&content)
        .build();

    scrolled
}

// Helper functions
fn get_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "winux".to_string())
        .trim()
        .to_string()
}

fn get_kernel_version() -> String {
    Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_cpu_info() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(name) = line.split(':').nth(1) {
                    return name.trim().to_string();
                }
            }
        }
    }
    "Unknown CPU".to_string()
}

fn get_memory_info() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal") {
                if let Some(mem) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = mem.parse::<u64>() {
                        let gb = kb as f64 / 1024.0 / 1024.0;
                        return format!("{:.1} GB", gb);
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_gpu_info() -> String {
    if let Ok(output) = Command::new("lspci").arg("-nn").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("VGA") || line.contains("3D") {
                if let Some(start) = line.find(": ") {
                    let info = &line[start + 2..];
                    return info.split('[').next().unwrap_or(info).trim().to_string();
                }
            }
        }
    }
    "Unknown GPU".to_string()
}

fn get_disk_info() -> String {
    if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return format!("{} total, {} disponivel", parts[1], parts[3]);
            }
        }
    }
    "Unknown".to_string()
}

fn create_language_page() -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // System Language
    let lang_group = PreferencesGroup::builder()
        .title("Idioma do Sistema")
        .description("Selecione o idioma principal")
        .build();

    let lang_row = ComboRow::builder()
        .title("Idioma")
        .subtitle("Idioma da interface do sistema")
        .build();
    let languages = gtk4::StringList::new(&[
        "Portugues (Brasil) - pt_BR",
        "English (USA) - en_US",
    ]);
    lang_row.set_model(Some(&languages));
    lang_group.add(&lang_row);

    let format_row = ComboRow::builder()
        .title("Formato Regional")
        .subtitle("Formato de data, hora e numeros")
        .build();
    let formats = gtk4::StringList::new(&[
        "Brasil (DD/MM/AAAA, 24h)",
        "Estados Unidos (MM/DD/YYYY, 12h)",
    ]);
    format_row.set_model(Some(&formats));
    lang_group.add(&format_row);

    page.add(&lang_group);

    // Keyboard Layout
    let keyboard_group = PreferencesGroup::builder()
        .title("Teclado")
        .description("Layout e configuracoes de teclado")
        .build();

    let layout_row = ComboRow::builder()
        .title("Layout do Teclado")
        .build();
    let layouts = gtk4::StringList::new(&[
        "Portugues (Brasil) - ABNT2",
        "Portugues (Brasil) - ABNT",
        "English (US) - QWERTY",
        "English (US) - International",
    ]);
    layout_row.set_model(Some(&layouts));
    keyboard_group.add(&layout_row);

    let compose_row = SwitchRow::builder()
        .title("Teclas de Composicao")
        .subtitle("Permite digitar caracteres especiais")
        .active(true)
        .build();
    keyboard_group.add(&compose_row);

    page.add(&keyboard_group);

    // Input Method
    let input_group = PreferencesGroup::builder()
        .title("Metodo de Entrada")
        .build();

    let ibus_row = ComboRow::builder()
        .title("Motor de Entrada")
        .build();
    let engines = gtk4::StringList::new(&["IBus", "Fcitx5", "Nenhum"]);
    ibus_row.set_model(Some(&engines));
    input_group.add(&ibus_row);

    page.add(&input_group);

    // Apply button
    let apply_group = PreferencesGroup::new();
    let apply_row = ActionRow::builder()
        .title("Aplicar Alteracoes")
        .subtitle("Requer logout para aplicar completamente")
        .activatable(true)
        .build();

    let apply_btn = Button::with_label("Aplicar");
    apply_btn.add_css_class("suggested-action");
    apply_btn.set_valign(gtk4::Align::Center);
    apply_btn.connect_clicked(|_| {
        // Apply language changes
        if let Err(e) = std::process::Command::new("notify-send")
            .args(["Winux Settings", "Configuracoes de idioma aplicadas. Faca logout para completar."])
            .spawn() {
            eprintln!("Failed to send notification: {}", e);
        }
    });
    apply_row.add_suffix(&apply_btn);
    apply_group.add(&apply_row);

    page.add(&apply_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}
