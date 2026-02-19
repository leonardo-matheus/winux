// Winux Dev Hub - Project Card Widget
// Copyright (c) 2026 Winux OS Project
//
// Widget for displaying project information in a card format

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow};

use crate::pages::dashboard::{Project, ProjectType};

/// Creates a row widget to display a project
pub fn create_project_row(project: &Project) -> ExpanderRow {
    let row = ExpanderRow::builder()
        .title(&project.name)
        .subtitle(&format!("{} - {}", project.project_type.display_name(), project.last_modified))
        .build();

    // Project type icon
    row.add_prefix(&gtk4::Image::from_icon_name(project.project_type.icon_name()));

    // Path info
    let path_row = ActionRow::builder()
        .title("Caminho")
        .subtitle(&project.path.to_string_lossy())
        .build();

    let copy_path_btn = Button::from_icon_name("edit-copy-symbolic");
    copy_path_btn.add_css_class("flat");
    copy_path_btn.set_valign(gtk4::Align::Center);
    copy_path_btn.set_tooltip_text(Some("Copiar caminho"));
    path_row.add_suffix(&copy_path_btn);

    row.add_row(&path_row);

    // Type info
    let type_row = ActionRow::builder()
        .title("Tipo de Projeto")
        .subtitle(project.project_type.display_name())
        .build();
    type_row.add_prefix(&gtk4::Image::from_icon_name(project.project_type.icon_name()));
    row.add_row(&type_row);

    // Last modified
    let modified_row = ActionRow::builder()
        .title("Ultima modificacao")
        .subtitle(&project.last_modified)
        .build();
    modified_row.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));
    row.add_row(&modified_row);

    // Quick actions
    let actions_row = ActionRow::builder()
        .title("Acoes")
        .build();

    let actions_box = Box::new(Orientation::Horizontal, 8);
    actions_box.set_halign(gtk4::Align::End);
    actions_box.set_valign(gtk4::Align::Center);

    // Open in terminal
    let terminal_btn = Button::from_icon_name("utilities-terminal-symbolic");
    terminal_btn.add_css_class("flat");
    terminal_btn.set_tooltip_text(Some("Abrir Terminal"));
    actions_box.append(&terminal_btn);

    // Open in file manager
    let files_btn = Button::from_icon_name("system-file-manager-symbolic");
    files_btn.add_css_class("flat");
    files_btn.set_tooltip_text(Some("Abrir Arquivos"));
    actions_box.append(&files_btn);

    // Open in VS Code
    let code_btn = Button::from_icon_name("text-editor-symbolic");
    code_btn.add_css_class("flat");
    code_btn.set_tooltip_text(Some("Abrir no Editor"));
    actions_box.append(&code_btn);

    // Git status (if applicable)
    let git_btn = Button::from_icon_name("git-symbolic");
    git_btn.add_css_class("flat");
    git_btn.set_tooltip_text(Some("Status Git"));
    actions_box.append(&git_btn);

    actions_row.add_suffix(&actions_box);
    row.add_row(&actions_row);

    // Project-specific actions based on type
    add_project_specific_actions(&row, &project.project_type);

    row
}

/// Creates a compact card for the project
pub fn create_project_card(project: &Project) -> Box {
    let card = Box::new(Orientation::Vertical, 8);
    card.set_margin_start(12);
    card.set_margin_end(12);
    card.set_margin_top(12);
    card.set_margin_bottom(12);
    card.add_css_class("card");

    // Header with icon and name
    let header = Box::new(Orientation::Horizontal, 12);

    let icon = gtk4::Image::from_icon_name(project.project_type.icon_name());
    icon.set_pixel_size(48);
    header.append(&icon);

    let info = Box::new(Orientation::Vertical, 4);

    let name_label = Label::new(Some(&project.name));
    name_label.add_css_class("title-3");
    name_label.set_xalign(0.0);
    info.append(&name_label);

    let type_label = Label::new(Some(project.project_type.display_name()));
    type_label.add_css_class("dim-label");
    type_label.set_xalign(0.0);
    info.append(&type_label);

    header.append(&info);
    card.append(&header);

    // Path
    let path_label = Label::new(Some(&project.path.to_string_lossy()));
    path_label.add_css_class("dim-label");
    path_label.add_css_class("caption");
    path_label.set_xalign(0.0);
    path_label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
    card.append(&path_label);

    // Actions
    let actions = Box::new(Orientation::Horizontal, 8);
    actions.set_margin_top(8);

    let open_btn = Button::with_label("Abrir");
    open_btn.add_css_class("suggested-action");
    open_btn.add_css_class("pill");
    actions.append(&open_btn);

    let terminal_btn = Button::from_icon_name("utilities-terminal-symbolic");
    terminal_btn.add_css_class("circular");
    actions.append(&terminal_btn);

    let files_btn = Button::from_icon_name("system-file-manager-symbolic");
    files_btn.add_css_class("circular");
    actions.append(&files_btn);

    card.append(&actions);

    card
}

/// Adds project-type specific actions to the row
fn add_project_specific_actions(row: &ExpanderRow, project_type: &ProjectType) {
    match project_type {
        ProjectType::Rust => {
            let run_row = ActionRow::builder()
                .title("cargo run")
                .subtitle("Executar o projeto")
                .activatable(true)
                .build();
            run_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
            row.add_row(&run_row);

            let build_row = ActionRow::builder()
                .title("cargo build")
                .subtitle("Compilar o projeto")
                .activatable(true)
                .build();
            build_row.add_prefix(&gtk4::Image::from_icon_name("applications-engineering-symbolic"));
            row.add_row(&build_row);

            let test_row = ActionRow::builder()
                .title("cargo test")
                .subtitle("Executar testes")
                .activatable(true)
                .build();
            test_row.add_prefix(&gtk4::Image::from_icon_name("emblem-default-symbolic"));
            row.add_row(&test_row);
        }
        ProjectType::Node => {
            let install_row = ActionRow::builder()
                .title("npm install")
                .subtitle("Instalar dependencias")
                .activatable(true)
                .build();
            install_row.add_prefix(&gtk4::Image::from_icon_name("emblem-downloads-symbolic"));
            row.add_row(&install_row);

            let start_row = ActionRow::builder()
                .title("npm start")
                .subtitle("Iniciar aplicacao")
                .activatable(true)
                .build();
            start_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
            row.add_row(&start_row);

            let dev_row = ActionRow::builder()
                .title("npm run dev")
                .subtitle("Modo de desenvolvimento")
                .activatable(true)
                .build();
            dev_row.add_prefix(&gtk4::Image::from_icon_name("preferences-system-symbolic"));
            row.add_row(&dev_row);
        }
        ProjectType::Python => {
            let venv_row = ActionRow::builder()
                .title("Ativar venv")
                .subtitle("Ativar ambiente virtual")
                .activatable(true)
                .build();
            venv_row.add_prefix(&gtk4::Image::from_icon_name("preferences-other-symbolic"));
            row.add_row(&venv_row);

            let run_row = ActionRow::builder()
                .title("python main.py")
                .subtitle("Executar script principal")
                .activatable(true)
                .build();
            run_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
            row.add_row(&run_row);

            let pip_row = ActionRow::builder()
                .title("pip install -r requirements.txt")
                .subtitle("Instalar dependencias")
                .activatable(true)
                .build();
            pip_row.add_prefix(&gtk4::Image::from_icon_name("emblem-downloads-symbolic"));
            row.add_row(&pip_row);
        }
        ProjectType::Go => {
            let run_row = ActionRow::builder()
                .title("go run .")
                .subtitle("Executar o projeto")
                .activatable(true)
                .build();
            run_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
            row.add_row(&run_row);

            let build_row = ActionRow::builder()
                .title("go build")
                .subtitle("Compilar o projeto")
                .activatable(true)
                .build();
            build_row.add_prefix(&gtk4::Image::from_icon_name("applications-engineering-symbolic"));
            row.add_row(&build_row);

            let test_row = ActionRow::builder()
                .title("go test ./...")
                .subtitle("Executar testes")
                .activatable(true)
                .build();
            test_row.add_prefix(&gtk4::Image::from_icon_name("emblem-default-symbolic"));
            row.add_row(&test_row);
        }
        ProjectType::DotNet => {
            let run_row = ActionRow::builder()
                .title("dotnet run")
                .subtitle("Executar o projeto")
                .activatable(true)
                .build();
            run_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
            row.add_row(&run_row);

            let build_row = ActionRow::builder()
                .title("dotnet build")
                .subtitle("Compilar o projeto")
                .activatable(true)
                .build();
            build_row.add_prefix(&gtk4::Image::from_icon_name("applications-engineering-symbolic"));
            row.add_row(&build_row);

            let restore_row = ActionRow::builder()
                .title("dotnet restore")
                .subtitle("Restaurar pacotes NuGet")
                .activatable(true)
                .build();
            restore_row.add_prefix(&gtk4::Image::from_icon_name("emblem-downloads-symbolic"));
            row.add_row(&restore_row);
        }
        ProjectType::Java => {
            let maven_row = ActionRow::builder()
                .title("mvn clean install")
                .subtitle("Build com Maven")
                .activatable(true)
                .build();
            maven_row.add_prefix(&gtk4::Image::from_icon_name("applications-engineering-symbolic"));
            row.add_row(&maven_row);

            let gradle_row = ActionRow::builder()
                .title("./gradlew build")
                .subtitle("Build com Gradle")
                .activatable(true)
                .build();
            gradle_row.add_prefix(&gtk4::Image::from_icon_name("applications-engineering-symbolic"));
            row.add_row(&gradle_row);
        }
        ProjectType::Php => {
            let composer_row = ActionRow::builder()
                .title("composer install")
                .subtitle("Instalar dependencias")
                .activatable(true)
                .build();
            composer_row.add_prefix(&gtk4::Image::from_icon_name("emblem-downloads-symbolic"));
            row.add_row(&composer_row);

            let serve_row = ActionRow::builder()
                .title("php artisan serve")
                .subtitle("Servidor de desenvolvimento (Laravel)")
                .activatable(true)
                .build();
            serve_row.add_prefix(&gtk4::Image::from_icon_name("network-server-symbolic"));
            row.add_row(&serve_row);
        }
        ProjectType::Ruby => {
            let bundle_row = ActionRow::builder()
                .title("bundle install")
                .subtitle("Instalar gems")
                .activatable(true)
                .build();
            bundle_row.add_prefix(&gtk4::Image::from_icon_name("emblem-downloads-symbolic"));
            row.add_row(&bundle_row);

            let rails_row = ActionRow::builder()
                .title("rails server")
                .subtitle("Iniciar servidor Rails")
                .activatable(true)
                .build();
            rails_row.add_prefix(&gtk4::Image::from_icon_name("network-server-symbolic"));
            row.add_row(&rails_row);
        }
        ProjectType::Unknown => {
            // No specific actions for unknown projects
        }
    }
}
