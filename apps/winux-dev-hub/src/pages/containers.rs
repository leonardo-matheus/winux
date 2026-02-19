// Winux Dev Hub - Containers Page
// Copyright (c) 2026 Winux OS Project
//
// Docker/Podman container management

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, TextView};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub ports: String,
    pub created: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Paused,
    Restarting,
    Exited,
}

impl ContainerStatus {
    pub fn icon(&self) -> &str {
        match self {
            ContainerStatus::Running => "media-playback-start-symbolic",
            ContainerStatus::Stopped => "media-playback-stop-symbolic",
            ContainerStatus::Paused => "media-playback-pause-symbolic",
            ContainerStatus::Restarting => "view-refresh-symbolic",
            ContainerStatus::Exited => "process-stop-symbolic",
        }
    }

    pub fn css_class(&self) -> &str {
        match self {
            ContainerStatus::Running => "success",
            ContainerStatus::Stopped | ContainerStatus::Exited => "error",
            ContainerStatus::Paused => "warning",
            ContainerStatus::Restarting => "accent",
        }
    }
}

pub fn create_page() -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Header Status
    let header_group = PreferencesGroup::new();

    let (docker_available, runtime_name) = detect_container_runtime();

    let status = StatusPage::builder()
        .icon_name("application-x-firmware-symbolic")
        .title("Containers")
        .description(if docker_available {
            format!("{} esta ativo", runtime_name).as_str()
        } else {
            "Docker/Podman nao detectado"
        })
        .build();

    // Quick actions
    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_bottom(24);

    let pull_btn = Button::with_label("Pull Image");
    pull_btn.add_css_class("pill");
    actions_box.append(&pull_btn);

    let run_btn = Button::with_label("Run Container");
    run_btn.add_css_class("suggested-action");
    run_btn.add_css_class("pill");
    actions_box.append(&run_btn);

    let compose_btn = Button::with_label("Docker Compose");
    compose_btn.add_css_class("pill");
    actions_box.append(&compose_btn);

    status.set_child(Some(&actions_box));

    let status_box = Box::new(Orientation::Vertical, 0);
    status_box.append(&status);
    header_group.add(&status_box);
    page.add(&header_group);

    // Runtime Status
    let runtime_group = PreferencesGroup::builder()
        .title("Runtime")
        .description("Status do runtime de containers")
        .build();

    // Docker daemon status
    let docker_status = get_daemon_status("docker");
    let docker_row = ActionRow::builder()
        .title("Docker Daemon")
        .subtitle(if docker_status { "Ativo" } else { "Inativo" })
        .build();
    docker_row.add_prefix(&gtk4::Image::from_icon_name("application-x-firmware-symbolic"));

    let docker_status_icon = gtk4::Image::from_icon_name(
        if docker_status { "emblem-ok-symbolic" } else { "dialog-warning-symbolic" }
    );
    if docker_status {
        docker_status_icon.add_css_class("success");
    } else {
        docker_status_icon.add_css_class("warning");
    }
    docker_row.add_suffix(&docker_status_icon);

    if !docker_status {
        let start_btn = Button::with_label("Iniciar");
        start_btn.add_css_class("suggested-action");
        start_btn.add_css_class("flat");
        start_btn.set_valign(gtk4::Align::Center);
        docker_row.add_suffix(&start_btn);
    }
    runtime_group.add(&docker_row);

    // Podman status
    let podman_status = get_daemon_status("podman");
    let podman_row = ActionRow::builder()
        .title("Podman")
        .subtitle(if podman_status { "Disponivel" } else { "Nao instalado" })
        .build();
    podman_row.add_prefix(&gtk4::Image::from_icon_name("application-x-firmware-symbolic"));

    let podman_status_icon = gtk4::Image::from_icon_name(
        if podman_status { "emblem-ok-symbolic" } else { "dialog-warning-symbolic" }
    );
    if podman_status {
        podman_status_icon.add_css_class("success");
    } else {
        podman_status_icon.add_css_class("dim-label");
    }
    podman_row.add_suffix(&podman_status_icon);
    runtime_group.add(&podman_row);

    page.add(&runtime_group);

    // Running Containers
    let running_group = PreferencesGroup::builder()
        .title("Containers em Execucao")
        .description("Containers ativos no momento")
        .build();

    let containers = list_containers();
    let running_containers: Vec<_> = containers.iter()
        .filter(|c| c.status == ContainerStatus::Running)
        .collect();

    if running_containers.is_empty() {
        let empty_row = ActionRow::builder()
            .title("Nenhum container em execucao")
            .subtitle("Inicie um container ou use docker-compose up")
            .build();
        empty_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        running_group.add(&empty_row);
    } else {
        for container in running_containers {
            let row = create_container_row(container);
            running_group.add(&row);
        }
    }

    page.add(&running_group);

    // Stopped Containers
    let stopped_group = PreferencesGroup::builder()
        .title("Containers Parados")
        .description("Containers que podem ser iniciados")
        .build();

    let stopped_containers: Vec<_> = containers.iter()
        .filter(|c| c.status != ContainerStatus::Running)
        .collect();

    if stopped_containers.is_empty() {
        let empty_row = ActionRow::builder()
            .title("Nenhum container parado")
            .build();
        stopped_group.add(&empty_row);
    } else {
        for container in stopped_containers.iter().take(5) {
            let row = create_container_row(container);
            stopped_group.add(&row);
        }
    }

    page.add(&stopped_group);

    // Images
    let images_group = PreferencesGroup::builder()
        .title("Imagens")
        .description("Imagens Docker disponiveis localmente")
        .build();

    let images = list_images();
    if images.is_empty() {
        let empty_row = ActionRow::builder()
            .title("Nenhuma imagem local")
            .subtitle("Use 'docker pull' para baixar imagens")
            .build();
        images_group.add(&empty_row);
    } else {
        for (image_name, tag, size) in images.iter().take(8) {
            let row = ActionRow::builder()
                .title(image_name)
                .subtitle(&format!("Tag: {} | Tamanho: {}", tag, size))
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));

            let run_btn = Button::from_icon_name("media-playback-start-symbolic");
            run_btn.add_css_class("flat");
            run_btn.set_valign(gtk4::Align::Center);
            run_btn.set_tooltip_text(Some("Executar"));
            row.add_suffix(&run_btn);

            let delete_btn = Button::from_icon_name("edit-delete-symbolic");
            delete_btn.add_css_class("flat");
            delete_btn.set_valign(gtk4::Align::Center);
            delete_btn.set_tooltip_text(Some("Remover"));
            row.add_suffix(&delete_btn);

            images_group.add(&row);
        }
    }

    page.add(&images_group);

    // Docker Compose
    let compose_group = PreferencesGroup::builder()
        .title("Docker Compose")
        .description("Projetos compose detectados")
        .build();

    let compose_files = find_compose_files();
    if compose_files.is_empty() {
        let empty_row = ActionRow::builder()
            .title("Nenhum arquivo docker-compose encontrado")
            .subtitle("Crie um docker-compose.yml no seu projeto")
            .build();
        compose_group.add(&empty_row);
    } else {
        for (path, name) in compose_files.iter().take(5) {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(path)
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("text-x-generic-symbolic"));

            let up_btn = Button::with_label("Up");
            up_btn.add_css_class("suggested-action");
            up_btn.add_css_class("flat");
            up_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&up_btn);

            let down_btn = Button::with_label("Down");
            down_btn.add_css_class("flat");
            down_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&down_btn);

            compose_group.add(&row);
        }
    }

    page.add(&compose_group);

    // Settings
    let settings_group = PreferencesGroup::builder()
        .title("Configuracoes")
        .build();

    let autostart_row = SwitchRow::builder()
        .title("Iniciar Docker no boot")
        .subtitle("Inicia o daemon Docker automaticamente")
        .active(true)
        .build();
    settings_group.add(&autostart_row);

    let prune_row = ActionRow::builder()
        .title("Limpar Sistema")
        .subtitle("Remove containers, imagens e volumes nao utilizados")
        .activatable(true)
        .build();
    prune_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));

    let prune_btn = Button::with_label("Prune");
    prune_btn.add_css_class("destructive-action");
    prune_btn.add_css_class("flat");
    prune_btn.set_valign(gtk4::Align::Center);
    prune_row.add_suffix(&prune_btn);
    settings_group.add(&prune_row);

    page.add(&settings_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn detect_container_runtime() -> (bool, String) {
    // Check Docker first
    if Command::new("docker").arg("info").output().map(|o| o.status.success()).unwrap_or(false) {
        return (true, "Docker".to_string());
    }

    // Check Podman
    if Command::new("podman").arg("info").output().map(|o| o.status.success()).unwrap_or(false) {
        return (true, "Podman".to_string());
    }

    (false, "Nenhum".to_string())
}

fn get_daemon_status(runtime: &str) -> bool {
    Command::new(runtime)
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn list_containers() -> Vec<Container> {
    let output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}|{{.CreatedAt}}"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 6 {
                        let status = parse_container_status(parts[3]);
                        Some(Container {
                            id: parts[0].to_string(),
                            name: parts[1].to_string(),
                            image: parts[2].to_string(),
                            status,
                            ports: parts[4].to_string(),
                            created: parts[5].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn parse_container_status(status_str: &str) -> ContainerStatus {
    let status_lower = status_str.to_lowercase();
    if status_lower.contains("up") {
        ContainerStatus::Running
    } else if status_lower.contains("paused") {
        ContainerStatus::Paused
    } else if status_lower.contains("restarting") {
        ContainerStatus::Restarting
    } else if status_lower.contains("exited") {
        ContainerStatus::Exited
    } else {
        ContainerStatus::Stopped
    }
}

fn create_container_row(container: &Container) -> ExpanderRow {
    let row = ExpanderRow::builder()
        .title(&container.name)
        .subtitle(&format!("{} - {}", container.image, container.ports))
        .build();

    row.add_prefix(&gtk4::Image::from_icon_name(container.status.icon()));

    // Status badge
    let status_label = Label::new(Some(match container.status {
        ContainerStatus::Running => "Running",
        ContainerStatus::Stopped => "Stopped",
        ContainerStatus::Paused => "Paused",
        ContainerStatus::Restarting => "Restarting",
        ContainerStatus::Exited => "Exited",
    }));
    status_label.add_css_class("badge");
    status_label.add_css_class(container.status.css_class());
    row.add_suffix(&status_label);

    // Details
    let id_row = ActionRow::builder()
        .title("Container ID")
        .subtitle(&container.id)
        .build();
    row.add_row(&id_row);

    let created_row = ActionRow::builder()
        .title("Criado em")
        .subtitle(&container.created)
        .build();
    row.add_row(&created_row);

    // Actions based on status
    if container.status == ContainerStatus::Running {
        let stop_row = ActionRow::builder()
            .title("Parar")
            .activatable(true)
            .build();
        stop_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-stop-symbolic"));
        row.add_row(&stop_row);

        let restart_row = ActionRow::builder()
            .title("Reiniciar")
            .activatable(true)
            .build();
        restart_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));
        row.add_row(&restart_row);

        let logs_row = ActionRow::builder()
            .title("Ver Logs")
            .activatable(true)
            .build();
        logs_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        row.add_row(&logs_row);

        let exec_row = ActionRow::builder()
            .title("Shell Interativo")
            .subtitle("Abre um terminal dentro do container")
            .activatable(true)
            .build();
        exec_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        row.add_row(&exec_row);
    } else {
        let start_row = ActionRow::builder()
            .title("Iniciar")
            .activatable(true)
            .build();
        start_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
        row.add_row(&start_row);

        let remove_row = ActionRow::builder()
            .title("Remover")
            .activatable(true)
            .build();
        remove_row.add_prefix(&gtk4::Image::from_icon_name("edit-delete-symbolic"));
        row.add_row(&remove_row);
    }

    row
}

fn list_images() -> Vec<(String, String, String)> {
    let output = Command::new("docker")
        .args(["images", "--format", "{{.Repository}}|{{.Tag}}|{{.Size}}"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        Some((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn find_compose_files() -> Vec<(String, String)> {
    let home = dirs::home_dir().unwrap_or_default();
    let search_dirs = vec![
        home.join("Projetos"),
        home.join("Projects"),
        home.join("dev"),
        home.join("workspace"),
    ];

    let mut compose_files = Vec::new();

    for dir in search_dirs {
        if dir.exists() {
            if let Ok(entries) = walkdir::WalkDir::new(&dir)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                for entry in entries {
                    let path = entry.path();
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy();
                        if filename_str == "docker-compose.yml"
                            || filename_str == "docker-compose.yaml"
                            || filename_str == "compose.yml"
                            || filename_str == "compose.yaml"
                        {
                            let parent_name = path
                                .parent()
                                .and_then(|p| p.file_name())
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Projeto".to_string());

                            compose_files.push((
                                path.to_string_lossy().to_string(),
                                parent_name,
                            ));
                        }
                    }
                }
            }
        }
    }

    compose_files
}
