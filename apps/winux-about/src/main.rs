// Winux About - System Information Application
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Grid, Label, Orientation, Picture, Separator,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, StatusPage, ActionRow, PreferencesGroup};
use sysinfo::{System, CpuRefreshKind, MemoryRefreshKind, RefreshKind};
use std::fs;

const APP_ID: &str = "org.winux.About";

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

    let main_box = Box::new(Orientation::Vertical, 0);

    // Logo and title section
    let status_page = StatusPage::builder()
        .icon_name("computer-symbolic")
        .title("Winux OS")
        .description("Developer Edition")
        .build();

    // System info section
    let content = Box::new(Orientation::Vertical, 24);
    content.set_margin_top(12);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);

    // Get system information
    let sys_info = get_system_info();

    // OS Information Group
    let os_group = PreferencesGroup::builder()
        .title("Sistema Operacional")
        .build();

    add_info_row(&os_group, "Nome", &sys_info.os_name);
    add_info_row(&os_group, "Versao", &sys_info.os_version);
    add_info_row(&os_group, "Codename", &sys_info.os_codename);
    add_info_row(&os_group, "Kernel", &sys_info.kernel);
    add_info_row(&os_group, "Arquitetura", &sys_info.arch);

    content.append(&os_group);

    // Hardware Information Group
    let hw_group = PreferencesGroup::builder()
        .title("Hardware")
        .build();

    add_info_row(&hw_group, "Processador", &sys_info.cpu);
    add_info_row(&hw_group, "Nucleos/Threads", &sys_info.cpu_cores);
    add_info_row(&hw_group, "Memoria RAM", &sys_info.memory);
    add_info_row(&hw_group, "GPU", &sys_info.gpu);
    add_info_row(&hw_group, "Armazenamento", &sys_info.storage);

    content.append(&hw_group);

    // Desktop Information Group
    let desktop_group = PreferencesGroup::builder()
        .title("Ambiente de Trabalho")
        .build();

    add_info_row(&desktop_group, "Sessao", &sys_info.desktop_session);
    add_info_row(&desktop_group, "Servidor Grafico", &sys_info.display_server);
    add_info_row(&desktop_group, "Hostname", &sys_info.hostname);
    add_info_row(&desktop_group, "Usuario", &sys_info.username);

    content.append(&desktop_group);

    // Build information
    let build_group = PreferencesGroup::builder()
        .title("Build")
        .build();

    add_info_row(&build_group, "Data do Build", &sys_info.build_date);
    add_info_row(&build_group, "Licenca", "MIT / GPL-3.0");
    add_info_row(&build_group, "Website", "https://winux.org");

    content.append(&build_group);

    // Scrolled window for content
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&content)
        .vexpand(true)
        .build();

    main_box.append(&status_page);
    main_box.append(&scrolled);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Sobre o Winux")
        .default_width(500)
        .default_height(700)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));
    window.present();
}

fn add_info_row(group: &PreferencesGroup, title: &str, value: &str) {
    let row = ActionRow::builder()
        .title(title)
        .subtitle(value)
        .build();
    group.add(&row);
}

struct SystemInfo {
    os_name: String,
    os_version: String,
    os_codename: String,
    kernel: String,
    arch: String,
    cpu: String,
    cpu_cores: String,
    memory: String,
    gpu: String,
    storage: String,
    desktop_session: String,
    display_server: String,
    hostname: String,
    username: String,
    build_date: String,
}

fn get_system_info() -> SystemInfo {
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
    );
    sys.refresh_all();

    // Read Winux release info
    let (os_version, os_codename, build_date) = read_winux_release();

    // Get CPU info
    let cpu_name = sys.cpus().first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let physical_cores = sys.physical_core_count().unwrap_or(0);
    let logical_cores = sys.cpus().len();

    // Get memory info
    let total_memory = sys.total_memory();
    let memory_str = format_bytes(total_memory);

    // Get GPU info (from lspci or sysfs)
    let gpu = get_gpu_info();

    // Get storage info
    let storage = get_storage_info();

    // Get desktop session info
    let desktop_session = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Winux Shell".to_string());

    let display_server = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        "Wayland"
    } else {
        "X11"
    }.to_string();

    // Get hostname
    let hostname = System::host_name().unwrap_or_else(|| "winux".to_string());

    // Get username
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    // Get kernel version
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

    // Get architecture
    let arch = System::cpu_arch().map(|s| s.to_string()).unwrap_or_else(|| "x86_64".to_string());

    SystemInfo {
        os_name: "Winux OS".to_string(),
        os_version,
        os_codename,
        kernel,
        arch,
        cpu: cpu_name,
        cpu_cores: format!("{} cores / {} threads", physical_cores, logical_cores),
        memory: memory_str,
        gpu,
        storage,
        desktop_session,
        display_server,
        hostname,
        username,
        build_date,
    }
}

fn read_winux_release() -> (String, String, String) {
    let release_file = "/etc/winux-release";
    let mut version = "1.0".to_string();
    let mut codename = "Aurora".to_string();
    let mut build_date = "Unknown".to_string();

    if let Ok(content) = fs::read_to_string(release_file) {
        for line in content.lines() {
            if line.starts_with("DISTRIB_RELEASE=") {
                version = line.split('=').nth(1).unwrap_or("1.0").to_string();
            } else if line.starts_with("DISTRIB_CODENAME=") {
                codename = line.split('=').nth(1).unwrap_or("aurora").to_string();
            } else if line.starts_with("BUILD_DATE=") {
                build_date = line.split('=').nth(1).unwrap_or("Unknown").to_string();
            }
        }
    }

    (version, codename, build_date)
}

fn get_gpu_info() -> String {
    // Try to read from lspci
    if let Ok(output) = std::process::Command::new("lspci")
        .args(["-nn"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("VGA") || line.contains("3D") || line.contains("Display") {
                // Extract GPU name
                if let Some(start) = line.find(": ") {
                    let gpu_info = &line[start + 2..];
                    // Clean up the string
                    let gpu_clean = gpu_info
                        .split('[')
                        .next()
                        .unwrap_or(gpu_info)
                        .trim();
                    if !gpu_clean.is_empty() {
                        return gpu_clean.to_string();
                    }
                }
            }
        }
    }

    // Fallback
    "Integrated Graphics".to_string()
}

fn get_storage_info() -> String {
    // Get root partition size
    if let Ok(output) = std::process::Command::new("df")
        .args(["-h", "/"])
        .output()
    {
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

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
