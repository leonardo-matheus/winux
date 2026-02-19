// Winux Dev Hub - Services Page
// Copyright (c) 2026 Winux OS Project
//
// System services management for development

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};
use std::process::Command;

use crate::widgets::service_row;

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub status: ServiceStatus,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Failed,
    Unknown,
}

impl ServiceStatus {
    pub fn icon(&self) -> &str {
        match self {
            ServiceStatus::Active => "emblem-ok-symbolic",
            ServiceStatus::Inactive => "media-playback-stop-symbolic",
            ServiceStatus::Failed => "dialog-error-symbolic",
            ServiceStatus::Unknown => "dialog-question-symbolic",
        }
    }

    pub fn css_class(&self) -> &str {
        match self {
            ServiceStatus::Active => "success",
            ServiceStatus::Inactive => "dim-label",
            ServiceStatus::Failed => "error",
            ServiceStatus::Unknown => "warning",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            ServiceStatus::Active => "Ativo",
            ServiceStatus::Inactive => "Inativo",
            ServiceStatus::Failed => "Falhou",
            ServiceStatus::Unknown => "Desconhecido",
        }
    }
}

pub fn create_page() -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Header
    let header_group = PreferencesGroup::new();

    let status = StatusPage::builder()
        .icon_name("system-run-symbolic")
        .title("Servicos do Sistema")
        .description("Gerencie servicos relacionados ao desenvolvimento")
        .build();

    // Quick actions
    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_bottom(24);

    let refresh_btn = Button::with_label("Atualizar Status");
    refresh_btn.add_css_class("pill");
    actions_box.append(&refresh_btn);

    let logs_btn = Button::with_label("Ver Logs");
    logs_btn.add_css_class("pill");
    actions_box.append(&logs_btn);

    status.set_child(Some(&actions_box));

    let status_box = Box::new(Orientation::Vertical, 0);
    status_box.append(&status);
    header_group.add(&status_box);
    page.add(&header_group);

    // Web Servers
    let web_group = PreferencesGroup::builder()
        .title("Servidores Web")
        .description("Apache, Nginx e servidores de aplicacao")
        .build();

    let web_services = [
        ("nginx", "Nginx", "Servidor web e proxy reverso de alta performance"),
        ("apache2", "Apache2", "Servidor web Apache HTTP"),
        ("httpd", "HTTPD", "Apache HTTP Server (alternativo)"),
        ("php-fpm", "PHP-FPM", "FastCGI Process Manager para PHP"),
        ("caddy", "Caddy", "Servidor web automatico com HTTPS"),
    ];

    for (service_name, display_name, description) in web_services {
        let row = service_row::create_service_row(service_name, display_name, description);
        web_group.add(&row);
    }

    page.add(&web_group);

    // Database Services
    let db_group = PreferencesGroup::builder()
        .title("Bancos de Dados")
        .description("Servicos de banco de dados")
        .build();

    let db_services = [
        ("postgresql", "PostgreSQL", "Banco de dados relacional avancado"),
        ("mysql", "MySQL", "Sistema de gerenciamento de banco de dados"),
        ("mariadb", "MariaDB", "Fork do MySQL com melhorias"),
        ("mongod", "MongoDB", "Banco de dados NoSQL orientado a documentos"),
        ("redis", "Redis", "Armazenamento de estrutura de dados em memoria"),
        ("elasticsearch", "Elasticsearch", "Motor de busca e analytics"),
    ];

    for (service_name, display_name, description) in db_services {
        let row = service_row::create_service_row(service_name, display_name, description);
        db_group.add(&row);
    }

    page.add(&db_group);

    // Container Services
    let container_group = PreferencesGroup::builder()
        .title("Containers")
        .description("Docker e servicos de virtualizacao")
        .build();

    let container_services = [
        ("docker", "Docker", "Plataforma de containerizacao de aplicacoes"),
        ("containerd", "containerd", "Runtime de containers"),
        ("podman", "Podman", "Container engine sem daemon"),
        ("libvirtd", "Libvirt", "API de virtualizacao"),
    ];

    for (service_name, display_name, description) in container_services {
        let row = service_row::create_service_row(service_name, display_name, description);
        container_group.add(&row);
    }

    page.add(&container_group);

    // Message Queue Services
    let mq_group = PreferencesGroup::builder()
        .title("Filas de Mensagens")
        .description("Servicos de mensageria e filas")
        .build();

    let mq_services = [
        ("rabbitmq-server", "RabbitMQ", "Message broker AMQP"),
        ("kafka", "Apache Kafka", "Plataforma de streaming distribuido"),
    ];

    for (service_name, display_name, description) in mq_services {
        let row = service_row::create_service_row(service_name, display_name, description);
        mq_group.add(&row);
    }

    page.add(&mq_group);

    // Development Tools
    let dev_group = PreferencesGroup::builder()
        .title("Ferramentas de Desenvolvimento")
        .description("Servicos auxiliares para desenvolvimento")
        .build();

    let dev_services = [
        ("ssh", "SSH Server", "Servidor OpenSSH para acesso remoto"),
        ("cups", "CUPS", "Sistema de impressao (util para PDFs)"),
        ("avahi-daemon", "Avahi", "Descoberta de servicos na rede local"),
        ("snapd", "Snapd", "Servico de pacotes Snap"),
        ("flatpak", "Flatpak", "Sistema de pacotes Flatpak"),
    ];

    for (service_name, display_name, description) in dev_services {
        let row = service_row::create_service_row(service_name, display_name, description);
        dev_group.add(&row);
    }

    page.add(&dev_group);

    // Network Services
    let net_group = PreferencesGroup::builder()
        .title("Rede")
        .description("Servicos de rede e conectividade")
        .build();

    let net_services = [
        ("NetworkManager", "NetworkManager", "Gerenciamento de conexoes de rede"),
        ("dnsmasq", "Dnsmasq", "DNS e DHCP leve"),
        ("named", "BIND", "Servidor DNS"),
        ("openvpn", "OpenVPN", "Servidor VPN"),
        ("wireguard", "WireGuard", "VPN moderna e rapida"),
    ];

    for (service_name, display_name, description) in net_services {
        let row = service_row::create_service_row(service_name, display_name, description);
        net_group.add(&row);
    }

    page.add(&net_group);

    // Quick Actions
    let actions_group = PreferencesGroup::builder()
        .title("Acoes Rapidas")
        .build();

    let systemctl_row = ActionRow::builder()
        .title("Abrir systemctl")
        .subtitle("Gerenciar todos os servicos via terminal")
        .activatable(true)
        .build();
    systemctl_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
    systemctl_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    actions_group.add(&systemctl_row);

    let journalctl_row = ActionRow::builder()
        .title("Ver Logs do Sistema")
        .subtitle("journalctl - logs do systemd")
        .activatable(true)
        .build();
    journalctl_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
    journalctl_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    actions_group.add(&journalctl_row);

    let reload_row = ActionRow::builder()
        .title("Recarregar Daemon")
        .subtitle("systemctl daemon-reload")
        .activatable(true)
        .build();
    reload_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));
    actions_group.add(&reload_row);

    page.add(&actions_group);

    // Custom Services
    let custom_group = PreferencesGroup::builder()
        .title("Servicos Personalizados")
        .description("Adicione seus proprios servicos para monitorar")
        .build();

    let add_row = ActionRow::builder()
        .title("Adicionar Servico")
        .subtitle("Monitore um servico systemd personalizado")
        .activatable(true)
        .build();
    add_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
    add_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    custom_group.add(&add_row);

    page.add(&custom_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}
