// Winux Dev Hub - Dashboard Page
// Copyright (c) 2026 Winux OS Project
//
// Project overview with automatic type detection

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, Grid};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, Clamp};
use std::path::PathBuf;
use std::fs;

use crate::widgets::project_card;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub last_modified: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    DotNet,
    Java,
    Php,
    Ruby,
    Unknown,
}

impl ProjectType {
    pub fn icon_name(&self) -> &str {
        match self {
            ProjectType::Rust => "application-x-executable-symbolic",
            ProjectType::Node => "text-x-javascript-symbolic",
            ProjectType::Python => "text-x-python-symbolic",
            ProjectType::Go => "text-x-generic-symbolic",
            ProjectType::DotNet => "application-x-addon-symbolic",
            ProjectType::Java => "application-x-java-symbolic",
            ProjectType::Php => "text-x-php-symbolic",
            ProjectType::Ruby => "text-x-ruby-symbolic",
            ProjectType::Unknown => "folder-symbolic",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Node => "Node.js",
            ProjectType::Python => "Python",
            ProjectType::Go => "Go",
            ProjectType::DotNet => ".NET",
            ProjectType::Java => "Java",
            ProjectType::Php => "PHP",
            ProjectType::Ruby => "Ruby",
            ProjectType::Unknown => "Projeto",
        }
    }
}

pub fn create_page() -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Header with status
    let header_group = PreferencesGroup::new();

    let status = StatusPage::builder()
        .icon_name("folder-templates-symbolic")
        .title("Dashboard de Projetos")
        .description("Seus projetos de desenvolvimento recentes")
        .build();

    // Quick actions
    let actions_box = Box::new(Orientation::Horizontal, 12);
    actions_box.set_halign(gtk4::Align::Center);
    actions_box.set_margin_bottom(24);

    let new_project_btn = Button::with_label("Novo Projeto");
    new_project_btn.add_css_class("suggested-action");
    new_project_btn.add_css_class("pill");
    actions_box.append(&new_project_btn);

    let open_folder_btn = Button::with_label("Abrir Pasta");
    open_folder_btn.add_css_class("pill");
    actions_box.append(&open_folder_btn);

    let scan_btn = Button::with_label("Escanear");
    scan_btn.add_css_class("pill");
    actions_box.append(&scan_btn);

    status.set_child(Some(&actions_box));

    let status_box = Box::new(Orientation::Vertical, 0);
    status_box.append(&status);
    header_group.add(&status_box);
    page.add(&header_group);

    // Recent projects section
    let recent_group = PreferencesGroup::builder()
        .title("Projetos Recentes")
        .description("Detectados automaticamente no seu sistema")
        .build();

    // Scan common project directories
    let projects = scan_projects();

    if projects.is_empty() {
        let empty_row = ActionRow::builder()
            .title("Nenhum projeto encontrado")
            .subtitle("Clique em 'Escanear' para buscar projetos ou 'Abrir Pasta' para adicionar")
            .build();
        empty_row.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));
        recent_group.add(&empty_row);
    } else {
        for project in projects.iter().take(10) {
            let row = project_card::create_project_row(project);
            recent_group.add(&row);
        }
    }

    page.add(&recent_group);

    // Project types summary
    let summary_group = PreferencesGroup::builder()
        .title("Resumo por Tipo")
        .description("Distribuicao dos seus projetos")
        .build();

    let type_counts = count_project_types(&projects);

    for (project_type, count) in type_counts {
        let row = ActionRow::builder()
            .title(project_type.display_name())
            .subtitle(&format!("{} projeto(s)", count))
            .build();
        row.add_prefix(&gtk4::Image::from_icon_name(project_type.icon_name()));

        let badge = Label::new(Some(&count.to_string()));
        badge.add_css_class("badge");
        badge.add_css_class("numeric");
        row.add_suffix(&badge);

        summary_group.add(&row);
    }

    page.add(&summary_group);

    // Quick links
    let links_group = PreferencesGroup::builder()
        .title("Acesso Rapido")
        .build();

    let links = [
        ("Terminal", "utilities-terminal-symbolic", "Abrir terminal no diretorio atual"),
        ("VS Code", "text-editor-symbolic", "Abrir Visual Studio Code"),
        ("Arquivos", "system-file-manager-symbolic", "Gerenciador de arquivos"),
        ("Git", "git-symbolic", "Abrir interface Git"),
    ];

    for (name, icon, desc) in links {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(desc)
            .activatable(true)
            .build();
        row.add_prefix(&gtk4::Image::from_icon_name(icon));
        row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        links_group.add(&row);
    }

    page.add(&links_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn scan_projects() -> Vec<Project> {
    let mut projects = Vec::new();

    // Common project directories
    let home = dirs::home_dir().unwrap_or_default();
    let search_dirs = vec![
        home.join("Projetos"),
        home.join("Projects"),
        home.join("dev"),
        home.join("Development"),
        home.join("workspace"),
        home.join("codigo"),
        home.join("repos"),
    ];

    for dir in search_dirs {
        if dir.exists() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(project) = detect_project(&path) {
                            projects.push(project);
                        }
                    }
                }
            }
        }
    }

    // Sort by name
    projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    projects
}

fn detect_project(path: &PathBuf) -> Option<Project> {
    let name = path.file_name()?.to_str()?.to_string();

    // Skip hidden directories
    if name.starts_with('.') {
        return None;
    }

    let project_type = detect_project_type(path);

    // Get last modified time
    let last_modified = fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Local> = t.into();
            datetime.format("%d/%m/%Y").to_string()
        })
        .unwrap_or_else(|| "Desconhecido".to_string());

    Some(Project {
        name,
        path: path.clone(),
        project_type,
        last_modified,
    })
}

fn detect_project_type(path: &PathBuf) -> ProjectType {
    // Check for Rust
    if path.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }

    // Check for Node.js
    if path.join("package.json").exists() {
        return ProjectType::Node;
    }

    // Check for Python
    if path.join("pyproject.toml").exists()
        || path.join("setup.py").exists()
        || path.join("requirements.txt").exists() {
        return ProjectType::Python;
    }

    // Check for Go
    if path.join("go.mod").exists() {
        return ProjectType::Go;
    }

    // Check for .NET
    if fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "csproj" || ext == "fsproj" || ext == "sln")
                        .unwrap_or(false)
                })
        })
        .unwrap_or(false)
    {
        return ProjectType::DotNet;
    }

    // Check for Java
    if path.join("pom.xml").exists()
        || path.join("build.gradle").exists()
        || path.join("build.gradle.kts").exists() {
        return ProjectType::Java;
    }

    // Check for PHP
    if path.join("composer.json").exists() {
        return ProjectType::Php;
    }

    // Check for Ruby
    if path.join("Gemfile").exists() {
        return ProjectType::Ruby;
    }

    ProjectType::Unknown
}

fn count_project_types(projects: &[Project]) -> Vec<(ProjectType, usize)> {
    let mut counts: std::collections::HashMap<ProjectType, usize> = std::collections::HashMap::new();

    for project in projects {
        if project.project_type != ProjectType::Unknown {
            *counts.entry(project.project_type.clone()).or_insert(0) += 1;
        }
    }

    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}
