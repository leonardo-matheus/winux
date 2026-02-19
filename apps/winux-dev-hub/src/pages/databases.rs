// Winux Dev Hub - Databases Page
// Copyright (c) 2026 Winux OS Project
//
// Local database management

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, StatusPage, SwitchRow};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Database {
    pub name: String,
    pub db_type: DatabaseType,
    pub status: DatabaseStatus,
    pub port: u16,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MariaDB,
    MongoDB,
    Redis,
    SQLite,
    Elasticsearch,
}

impl DatabaseType {
    pub fn icon(&self) -> &str {
        match self {
            DatabaseType::PostgreSQL => "drive-multidisk-symbolic",
            DatabaseType::MySQL | DatabaseType::MariaDB => "drive-multidisk-symbolic",
            DatabaseType::MongoDB => "drive-multidisk-symbolic",
            DatabaseType::Redis => "network-server-symbolic",
            DatabaseType::SQLite => "document-save-symbolic",
            DatabaseType::Elasticsearch => "system-search-symbolic",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::MySQL | DatabaseType::MariaDB => 3306,
            DatabaseType::MongoDB => 27017,
            DatabaseType::Redis => 6379,
            DatabaseType::SQLite => 0,
            DatabaseType::Elasticsearch => 9200,
        }
    }

    pub fn connection_string_template(&self) -> &str {
        match self {
            DatabaseType::PostgreSQL => "postgresql://user:password@localhost:5432/database",
            DatabaseType::MySQL | DatabaseType::MariaDB => "mysql://user:password@localhost:3306/database",
            DatabaseType::MongoDB => "mongodb://localhost:27017/database",
            DatabaseType::Redis => "redis://localhost:6379",
            DatabaseType::SQLite => "sqlite:///path/to/database.db",
            DatabaseType::Elasticsearch => "http://localhost:9200",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseStatus {
    Running,
    Stopped,
    NotInstalled,
}

impl DatabaseStatus {
    pub fn icon(&self) -> &str {
        match self {
            DatabaseStatus::Running => "emblem-ok-symbolic",
            DatabaseStatus::Stopped => "media-playback-stop-symbolic",
            DatabaseStatus::NotInstalled => "dialog-warning-symbolic",
        }
    }

    pub fn css_class(&self) -> &str {
        match self {
            DatabaseStatus::Running => "success",
            DatabaseStatus::Stopped => "warning",
            DatabaseStatus::NotInstalled => "dim-label",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            DatabaseStatus::Running => "Ativo",
            DatabaseStatus::Stopped => "Parado",
            DatabaseStatus::NotInstalled => "Nao instalado",
        }
    }
}

pub fn create_page() -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Header
    let header_group = PreferencesGroup::new();

    let status = StatusPage::builder()
        .icon_name("drive-multidisk-symbolic")
        .title("Databases")
        .description("Gerencie seus bancos de dados locais")
        .build();

    // Quick actions
    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_bottom(24);

    let start_all_btn = Button::with_label("Iniciar Todos");
    start_all_btn.add_css_class("suggested-action");
    start_all_btn.add_css_class("pill");
    actions_box.append(&start_all_btn);

    let stop_all_btn = Button::with_label("Parar Todos");
    stop_all_btn.add_css_class("pill");
    actions_box.append(&stop_all_btn);

    status.set_child(Some(&actions_box));

    let status_box = Box::new(Orientation::Vertical, 0);
    status_box.append(&status);
    header_group.add(&status_box);
    page.add(&header_group);

    // PostgreSQL
    let pg_status = check_service_status("postgresql");
    let pg_version = get_command_output("psql", &["--version"]);
    let pg_group = create_database_section(
        "PostgreSQL",
        DatabaseType::PostgreSQL,
        pg_status,
        &pg_version,
    );
    page.add(&pg_group);

    // MySQL/MariaDB
    let mysql_status = check_service_status("mysql").or(check_service_status("mariadb"));
    let mysql_version = get_command_output("mysql", &["--version"]);
    let mysql_group = create_database_section(
        "MySQL / MariaDB",
        DatabaseType::MySQL,
        mysql_status,
        &mysql_version,
    );
    page.add(&mysql_group);

    // MongoDB
    let mongo_status = check_service_status("mongod");
    let mongo_version = get_command_output("mongod", &["--version"]);
    let mongo_group = create_database_section(
        "MongoDB",
        DatabaseType::MongoDB,
        mongo_status,
        &mongo_version.map(|v| v.lines().next().unwrap_or("").to_string()),
    );
    page.add(&mongo_group);

    // Redis
    let redis_status = check_service_status("redis");
    let redis_version = get_command_output("redis-server", &["--version"]);
    let redis_group = create_database_section(
        "Redis",
        DatabaseType::Redis,
        redis_status,
        &redis_version.map(|v| {
            v.split_whitespace()
                .find(|s| s.starts_with("v="))
                .map(|s| s.replace("v=", ""))
                .unwrap_or_default()
        }),
    );
    page.add(&redis_group);

    // Elasticsearch
    let es_status = check_service_status("elasticsearch");
    let es_group = create_database_section(
        "Elasticsearch",
        DatabaseType::Elasticsearch,
        es_status,
        &None,
    );
    page.add(&es_group);

    // Quick Connect Strings
    let connect_group = PreferencesGroup::builder()
        .title("Connection Strings")
        .description("Copie as strings de conexao rapidamente")
        .build();

    let connection_templates = [
        ("PostgreSQL", "postgresql://user:password@localhost:5432/mydb"),
        ("MySQL", "mysql://user:password@localhost:3306/mydb"),
        ("MongoDB", "mongodb://localhost:27017/mydb"),
        ("Redis", "redis://localhost:6379"),
        ("SQLite", "sqlite:///home/user/data/app.db"),
    ];

    for (name, conn_str) in connection_templates {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(conn_str)
            .build();

        let copy_btn = Button::from_icon_name("edit-copy-symbolic");
        copy_btn.add_css_class("flat");
        copy_btn.set_valign(gtk4::Align::Center);
        copy_btn.set_tooltip_text(Some("Copiar"));
        row.add_suffix(&copy_btn);

        connect_group.add(&row);
    }

    page.add(&connect_group);

    // Database Tools
    let tools_group = PreferencesGroup::builder()
        .title("Ferramentas")
        .description("Clientes e utilitarios de banco de dados")
        .build();

    let tools = [
        ("DBeaver", "Cliente universal de banco de dados", "dbeaver"),
        ("pgAdmin", "Administracao PostgreSQL", "pgadmin4"),
        ("MongoDB Compass", "GUI para MongoDB", "mongodb-compass"),
        ("Redis Insight", "GUI para Redis", "redisinsight"),
        ("Beekeeper Studio", "Cliente SQL moderno", "beekeeper-studio"),
    ];

    for (name, desc, cmd) in tools {
        let installed = Command::new("which").arg(cmd).output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let row = ActionRow::builder()
            .title(name)
            .subtitle(desc)
            .activatable(installed)
            .build();

        row.add_prefix(&gtk4::Image::from_icon_name("application-x-executable-symbolic"));

        if installed {
            let open_btn = Button::with_label("Abrir");
            open_btn.add_css_class("flat");
            open_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&open_btn);
        } else {
            let install_btn = Button::with_label("Instalar");
            install_btn.add_css_class("flat");
            install_btn.add_css_class("suggested-action");
            install_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&install_btn);
        }

        tools_group.add(&row);
    }

    page.add(&tools_group);

    // Docker Database Images
    let docker_group = PreferencesGroup::builder()
        .title("Containers de Database")
        .description("Inicie bancos de dados via Docker rapidamente")
        .build();

    let docker_images = [
        ("postgres:latest", "PostgreSQL via Docker", "5432"),
        ("mysql:latest", "MySQL via Docker", "3306"),
        ("mongo:latest", "MongoDB via Docker", "27017"),
        ("redis:latest", "Redis via Docker", "6379"),
        ("elasticsearch:8.11.0", "Elasticsearch via Docker", "9200"),
    ];

    for (image, desc, port) in docker_images {
        let row = ActionRow::builder()
            .title(image)
            .subtitle(&format!("{} - Porta {}", desc, port))
            .build();

        row.add_prefix(&gtk4::Image::from_icon_name("application-x-firmware-symbolic"));

        let run_btn = Button::with_label("Run");
        run_btn.add_css_class("suggested-action");
        run_btn.add_css_class("flat");
        run_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&run_btn);

        docker_group.add(&row);
    }

    page.add(&docker_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn check_service_status(service: &str) -> Option<DatabaseStatus> {
    // Check if service is installed
    let is_installed = Command::new("systemctl")
        .args(["list-unit-files", &format!("{}.service", service)])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(service))
        .unwrap_or(false);

    if !is_installed {
        return Some(DatabaseStatus::NotInstalled);
    }

    // Check if service is running
    let is_running = Command::new("systemctl")
        .args(["is-active", service])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    if is_running {
        Some(DatabaseStatus::Running)
    } else {
        Some(DatabaseStatus::Stopped)
    }
}

fn get_command_output(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn create_database_section(
    name: &str,
    db_type: DatabaseType,
    status: Option<DatabaseStatus>,
    version: &Option<String>,
) -> PreferencesGroup {
    let group = PreferencesGroup::builder()
        .title(name)
        .build();

    let status = status.unwrap_or(DatabaseStatus::NotInstalled);

    // Main status row
    let main_row = ExpanderRow::builder()
        .title(name)
        .subtitle(version.clone().unwrap_or_else(|| status.display_name().to_string()))
        .build();

    main_row.add_prefix(&gtk4::Image::from_icon_name(db_type.icon()));

    let status_icon = gtk4::Image::from_icon_name(status.icon());
    status_icon.add_css_class(status.css_class());
    main_row.add_suffix(&status_icon);

    // Connection info
    let port_row = ActionRow::builder()
        .title("Porta")
        .subtitle(&db_type.default_port().to_string())
        .build();
    main_row.add_row(&port_row);

    let conn_row = ActionRow::builder()
        .title("Connection String")
        .subtitle(db_type.connection_string_template())
        .build();

    let copy_btn = Button::from_icon_name("edit-copy-symbolic");
    copy_btn.add_css_class("flat");
    copy_btn.set_valign(gtk4::Align::Center);
    conn_row.add_suffix(&copy_btn);
    main_row.add_row(&conn_row);

    // Actions based on status
    match status {
        DatabaseStatus::Running => {
            let stop_row = ActionRow::builder()
                .title("Parar Servico")
                .activatable(true)
                .build();
            stop_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-stop-symbolic"));
            main_row.add_row(&stop_row);

            let restart_row = ActionRow::builder()
                .title("Reiniciar Servico")
                .activatable(true)
                .build();
            restart_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));
            main_row.add_row(&restart_row);

            let logs_row = ActionRow::builder()
                .title("Ver Logs")
                .activatable(true)
                .build();
            logs_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
            main_row.add_row(&logs_row);

            let shell_row = ActionRow::builder()
                .title("Abrir Shell")
                .subtitle("Conectar via linha de comando")
                .activatable(true)
                .build();
            shell_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
            main_row.add_row(&shell_row);
        }
        DatabaseStatus::Stopped => {
            let start_row = ActionRow::builder()
                .title("Iniciar Servico")
                .activatable(true)
                .build();
            start_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
            main_row.add_row(&start_row);

            let enable_row = ActionRow::builder()
                .title("Habilitar no Boot")
                .activatable(true)
                .build();
            enable_row.add_prefix(&gtk4::Image::from_icon_name("emblem-default-symbolic"));
            main_row.add_row(&enable_row);
        }
        DatabaseStatus::NotInstalled => {
            let install_row = ActionRow::builder()
                .title("Instalar")
                .subtitle("Instalar via gerenciador de pacotes")
                .activatable(true)
                .build();
            install_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
            main_row.add_row(&install_row);

            let docker_row = ActionRow::builder()
                .title("Usar via Docker")
                .subtitle("Executar em container")
                .activatable(true)
                .build();
            docker_row.add_prefix(&gtk4::Image::from_icon_name("application-x-firmware-symbolic"));
            main_row.add_row(&docker_row);
        }
    }

    group.add(&main_row);
    group
}
