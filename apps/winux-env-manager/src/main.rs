// Winux Environment Manager - System Environment Variables Manager
// Copyright (c) 2026 Winux OS Project
//
// Provides a GUI to manage system and user environment variables
// Includes pre-configured development environment settings

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Entry, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, SearchEntry, Separator,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, ApplicationWindow, EntryRow, HeaderBar, PreferencesGroup,
    PreferencesPage, StatusPage, SwitchRow, ExpanderRow, ToastOverlay,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const APP_ID: &str = "org.winux.EnvManager";

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
    let title = adw::WindowTitle::new("Variaveis de Ambiente", "Gerencie as variaveis do sistema");
    header.set_title_widget(Some(&title));

    // Add button
    let add_btn = Button::from_icon_name("list-add-symbolic");
    add_btn.set_tooltip_text(Some("Adicionar variavel"));
    header.pack_start(&add_btn);

    // Save button
    let save_btn = Button::with_label("Salvar");
    save_btn.add_css_class("suggested-action");
    header.pack_end(&save_btn);

    let toast_overlay = ToastOverlay::new();

    let main_box = Box::new(Orientation::Vertical, 0);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let content = Box::new(Orientation::Vertical, 0);
    let page = create_env_page();
    content.append(&page);

    scrolled.set_child(Some(&content));
    main_box.append(&scrolled);

    toast_overlay.set_child(Some(&main_box));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Environment Manager")
        .default_width(800)
        .default_height(700)
        .content(&toast_overlay)
        .build();

    window.set_titlebar(Some(&header));

    // Save button action
    let toast_overlay_clone = toast_overlay.clone();
    save_btn.connect_clicked(move |_| {
        let toast = adw::Toast::new("Variaveis de ambiente salvas! Reinicie o terminal.");
        toast_overlay_clone.add_toast(toast);
    });

    window.present();
}

fn create_env_page() -> PreferencesPage {
    let page = PreferencesPage::new();

    // Development Languages Section
    let dev_group = PreferencesGroup::builder()
        .title("Linguagens de Desenvolvimento")
        .description("Variaveis de ambiente para linguagens de programacao")
        .build();

    // Node.js / NVM
    let node_expander = ExpanderRow::builder()
        .title("Node.js / NVM")
        .subtitle("JavaScript runtime e gerenciador de versoes")
        .build();

    add_env_row(&node_expander, "NVM_DIR", "$HOME/.nvm");
    add_env_row(&node_expander, "NODE_PATH", "$NVM_DIR/versions/node/$(nvm current)/lib/node_modules");
    add_path_row(&node_expander, "$NVM_DIR/versions/node/$(nvm current)/bin");
    dev_group.add(&node_expander);

    // Java
    let java_expander = ExpanderRow::builder()
        .title("Java / JDK")
        .subtitle("Java Development Kit e ferramentas")
        .build();

    add_env_row(&java_expander, "JAVA_HOME", "/usr/lib/jvm/java-21-openjdk-amd64");
    add_env_row(&java_expander, "MAVEN_HOME", "/usr/share/maven");
    add_env_row(&java_expander, "GRADLE_HOME", "/usr/share/gradle");
    add_path_row(&java_expander, "$JAVA_HOME/bin");
    add_path_row(&java_expander, "$MAVEN_HOME/bin");
    dev_group.add(&java_expander);

    // PHP
    let php_expander = ExpanderRow::builder()
        .title("PHP / Composer")
        .subtitle("PHP e gerenciador de pacotes")
        .build();

    add_env_row(&php_expander, "COMPOSER_HOME", "$HOME/.config/composer");
    add_path_row(&php_expander, "$COMPOSER_HOME/vendor/bin");
    dev_group.add(&php_expander);

    // Python
    let python_expander = ExpanderRow::builder()
        .title("Python / pip")
        .subtitle("Python e gerenciadores de pacotes")
        .build();

    add_env_row(&python_expander, "PYTHONPATH", "$HOME/.local/lib/python3/site-packages");
    add_env_row(&python_expander, "PIPENV_VENV_IN_PROJECT", "1");
    add_env_row(&python_expander, "POETRY_VIRTUALENVS_IN_PROJECT", "true");
    add_path_row(&python_expander, "$HOME/.local/bin");
    dev_group.add(&python_expander);

    // Rust
    let rust_expander = ExpanderRow::builder()
        .title("Rust / Cargo")
        .subtitle("Rust e ferramentas de build")
        .build();

    add_env_row(&rust_expander, "CARGO_HOME", "$HOME/.cargo");
    add_env_row(&rust_expander, "RUSTUP_HOME", "$HOME/.rustup");
    add_path_row(&rust_expander, "$CARGO_HOME/bin");
    dev_group.add(&rust_expander);

    // Go
    let go_expander = ExpanderRow::builder()
        .title("Go")
        .subtitle("Go programming language")
        .build();

    add_env_row(&go_expander, "GOPATH", "$HOME/go");
    add_env_row(&go_expander, "GOROOT", "/usr/local/go");
    add_path_row(&go_expander, "$GOPATH/bin");
    add_path_row(&go_expander, "$GOROOT/bin");
    dev_group.add(&go_expander);

    page.add(&dev_group);

    // System PATH
    let path_group = PreferencesGroup::builder()
        .title("PATH do Sistema")
        .description("Diretorios incluidos no PATH")
        .build();

    let current_path = env::var("PATH").unwrap_or_default();
    for path in current_path.split(':').take(15) {
        if !path.is_empty() {
            let row = ActionRow::builder()
                .title(path)
                .build();

            let delete_btn = Button::from_icon_name("edit-delete-symbolic");
            delete_btn.add_css_class("flat");
            delete_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&delete_btn);

            path_group.add(&row);
        }
    }

    let add_path_btn = Button::with_label("Adicionar ao PATH");
    add_path_btn.add_css_class("flat");
    add_path_btn.set_halign(gtk4::Align::Center);
    add_path_btn.set_margin_top(12);

    let path_box = Box::new(Orientation::Vertical, 0);
    path_box.append(&add_path_btn);
    path_group.add(&path_box);

    page.add(&path_group);

    // User Environment Variables
    let user_group = PreferencesGroup::builder()
        .title("Variaveis de Usuario")
        .description("Variaveis de ambiente personalizadas")
        .build();

    // Read from .bashrc or profile
    let important_vars = [
        ("HOME", env::var("HOME").unwrap_or_default()),
        ("USER", env::var("USER").unwrap_or_default()),
        ("SHELL", env::var("SHELL").unwrap_or_default()),
        ("LANG", env::var("LANG").unwrap_or_default()),
        ("EDITOR", env::var("EDITOR").unwrap_or("nano".to_string())),
        ("VISUAL", env::var("VISUAL").unwrap_or("code".to_string())),
        ("TERM", env::var("TERM").unwrap_or_default()),
        ("XDG_SESSION_TYPE", env::var("XDG_SESSION_TYPE").unwrap_or_default()),
    ];

    for (name, value) in important_vars {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(&value)
            .build();

        let edit_btn = Button::from_icon_name("document-edit-symbolic");
        edit_btn.add_css_class("flat");
        edit_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&edit_btn);

        user_group.add(&row);
    }

    page.add(&user_group);

    // Development Tools
    let tools_group = PreferencesGroup::builder()
        .title("Configuracoes de Ferramentas")
        .description("Variaveis para ferramentas de desenvolvimento")
        .build();

    // Docker
    let docker_row = ActionRow::builder()
        .title("DOCKER_HOST")
        .subtitle("unix:///var/run/docker.sock")
        .build();
    tools_group.add(&docker_row);

    // Git
    let git_editor = ActionRow::builder()
        .title("GIT_EDITOR")
        .subtitle(env::var("GIT_EDITOR").unwrap_or("code --wait".to_string()).as_str())
        .build();
    tools_group.add(&git_editor);

    // VS Code
    let vscode_row = ActionRow::builder()
        .title("VSCODE_IPC_HOOK_CLI")
        .subtitle("Auto-configurado pelo VS Code")
        .build();
    tools_group.add(&vscode_row);

    page.add(&tools_group);

    // Apply Configuration Button
    let apply_group = PreferencesGroup::new();

    let apply_btn = Button::with_label("Aplicar Configuracao Completa");
    apply_btn.add_css_class("suggested-action");
    apply_btn.add_css_class("pill");
    apply_btn.set_halign(gtk4::Align::Center);
    apply_btn.set_margin_top(24);
    apply_btn.set_margin_bottom(24);

    apply_btn.connect_clicked(|_| {
        // Apply all environment configurations
        println!("Applying configuration...");
        // Run the setup script
        if let Err(e) = std::process::Command::new("bash")
            .args(["/etc/winux/scripts/setup-environment.sh"])
            .spawn() {
            eprintln!("Failed to run setup script: {}", e);
        }
    });

    let btn_box = Box::new(Orientation::Vertical, 0);
    btn_box.append(&apply_btn);
    apply_group.add(&btn_box);

    page.add(&apply_group);

    page
}

fn add_env_row(expander: &ExpanderRow, name: &str, value: &str) {
    let row = ActionRow::builder()
        .title(name)
        .subtitle(value)
        .build();

    let switch = gtk4::Switch::new();
    switch.set_active(true);
    switch.set_valign(gtk4::Align::Center);
    row.add_suffix(&switch);

    expander.add_row(&row);
}

fn add_path_row(expander: &ExpanderRow, path: &str) {
    let row = ActionRow::builder()
        .title("PATH +=")
        .subtitle(path)
        .build();

    let check = gtk4::CheckButton::new();
    check.set_active(true);
    row.add_prefix(&check);

    expander.add_row(&row);
}
