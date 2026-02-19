// Winux Dev Hub - Toolchains Page
// Copyright (c) 2026 Winux OS Project
//
// Development toolchain management

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ProgressBar, ScrolledWindow, Spinner};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup, PreferencesPage, StatusPage};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Toolchain {
    pub name: String,
    pub version: Option<String>,
    pub installed: bool,
    pub update_available: Option<String>,
    pub icon: String,
}

pub fn create_page() -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Status Overview
    let status_group = PreferencesGroup::new();

    let status = StatusPage::builder()
        .icon_name("applications-engineering-symbolic")
        .title("Toolchains")
        .description("Gerencie suas ferramentas de desenvolvimento")
        .build();

    let status_box = Box::new(Orientation::Vertical, 0);
    status_box.append(&status);
    status_group.add(&status_box);
    page.add(&status_group);

    // Languages & Runtimes
    let languages_group = PreferencesGroup::builder()
        .title("Linguagens & Runtimes")
        .description("Status das ferramentas instaladas")
        .build();

    // Rust
    let rust_version = get_command_output("rustc", &["--version"]);
    let cargo_version = get_command_output("cargo", &["--version"]);
    let rust_row = create_toolchain_row(
        "Rust",
        &rust_version,
        &cargo_version.map(|v| format!("Cargo: {}", v)),
        "application-x-executable-symbolic",
        true,
    );
    languages_group.add(&rust_row);

    // Node.js
    let node_version = get_command_output("node", &["--version"]);
    let npm_version = get_command_output("npm", &["--version"]);
    let node_row = create_toolchain_row(
        "Node.js",
        &node_version,
        &npm_version.map(|v| format!("npm: {}", v)),
        "text-x-javascript-symbolic",
        node_version.is_some(),
    );
    languages_group.add(&node_row);

    // Python
    let python_version = get_command_output("python3", &["--version"]);
    let pip_version = get_command_output("pip3", &["--version"]);
    let python_row = create_toolchain_row(
        "Python",
        &python_version,
        &pip_version.map(|v| v.split_whitespace().take(2).collect::<Vec<_>>().join(" ")),
        "text-x-python-symbolic",
        python_version.is_some(),
    );
    languages_group.add(&python_row);

    // Go
    let go_version = get_command_output("go", &["version"]);
    let go_row = create_toolchain_row(
        "Go",
        &go_version.map(|v| v.replace("go version ", "")),
        &None,
        "text-x-generic-symbolic",
        go_version.is_some(),
    );
    languages_group.add(&go_row);

    // .NET
    let dotnet_version = get_command_output("dotnet", &["--version"]);
    let dotnet_row = create_toolchain_row(
        ".NET SDK",
        &dotnet_version,
        &None,
        "application-x-addon-symbolic",
        dotnet_version.is_some(),
    );
    languages_group.add(&dotnet_row);

    // Java
    let java_version = get_command_output("java", &["--version"])
        .or_else(|| get_command_output("java", &["-version"]));
    let java_row = create_toolchain_row(
        "Java (OpenJDK)",
        &java_version.map(|v| v.lines().next().unwrap_or("").to_string()),
        &None,
        "application-x-java-symbolic",
        java_version.is_some(),
    );
    languages_group.add(&java_row);

    // PHP
    let php_version = get_command_output("php", &["--version"]);
    let php_row = create_toolchain_row(
        "PHP",
        &php_version.map(|v| v.lines().next().unwrap_or("").to_string()),
        &None,
        "text-x-php-symbolic",
        php_version.is_some(),
    );
    languages_group.add(&php_row);

    // Ruby
    let ruby_version = get_command_output("ruby", &["--version"]);
    let ruby_row = create_toolchain_row(
        "Ruby",
        &ruby_version,
        &None,
        "text-x-ruby-symbolic",
        ruby_version.is_some(),
    );
    languages_group.add(&ruby_row);

    page.add(&languages_group);

    // Build Tools
    let build_group = PreferencesGroup::builder()
        .title("Ferramentas de Build")
        .description("Compiladores e sistemas de build")
        .build();

    // GCC
    let gcc_version = get_command_output("gcc", &["--version"]);
    let gcc_row = create_toolchain_row(
        "GCC",
        &gcc_version.map(|v| v.lines().next().unwrap_or("").to_string()),
        &None,
        "applications-engineering-symbolic",
        gcc_version.is_some(),
    );
    build_group.add(&gcc_row);

    // Clang
    let clang_version = get_command_output("clang", &["--version"]);
    let clang_row = create_toolchain_row(
        "Clang/LLVM",
        &clang_version.map(|v| v.lines().next().unwrap_or("").to_string()),
        &None,
        "applications-engineering-symbolic",
        clang_version.is_some(),
    );
    build_group.add(&clang_row);

    // CMake
    let cmake_version = get_command_output("cmake", &["--version"]);
    let cmake_row = create_toolchain_row(
        "CMake",
        &cmake_version.map(|v| v.lines().next().unwrap_or("").to_string()),
        &None,
        "applications-engineering-symbolic",
        cmake_version.is_some(),
    );
    build_group.add(&cmake_row);

    // Make
    let make_version = get_command_output("make", &["--version"]);
    let make_row = create_toolchain_row(
        "GNU Make",
        &make_version.map(|v| v.lines().next().unwrap_or("").to_string()),
        &None,
        "applications-engineering-symbolic",
        make_version.is_some(),
    );
    build_group.add(&make_row);

    page.add(&build_group);

    // Package Managers
    let pkg_group = PreferencesGroup::builder()
        .title("Gerenciadores de Pacotes")
        .build();

    let pkg_managers = [
        ("npm", "Node Package Manager"),
        ("yarn", "Yarn Package Manager"),
        ("pnpm", "PNPM Package Manager"),
        ("pip", "Python Package Installer"),
        ("cargo", "Rust Package Manager"),
        ("composer", "PHP Composer"),
        ("gem", "Ruby Gems"),
    ];

    for (cmd, name) in pkg_managers {
        let version = get_command_output(cmd, &["--version"]);
        let installed = version.is_some();

        let row = ActionRow::builder()
            .title(name)
            .subtitle(version.unwrap_or_else(|| "Nao instalado".to_string()).as_str())
            .build();

        let status_icon = if installed {
            "emblem-ok-symbolic"
        } else {
            "dialog-warning-symbolic"
        };

        row.add_prefix(&gtk4::Image::from_icon_name("package-x-generic-symbolic"));

        let status_img = gtk4::Image::from_icon_name(status_icon);
        if installed {
            status_img.add_css_class("success");
        } else {
            status_img.add_css_class("warning");
        }
        row.add_suffix(&status_img);

        pkg_group.add(&row);
    }

    page.add(&pkg_group);

    // Version Managers
    let version_mgr_group = PreferencesGroup::builder()
        .title("Gerenciadores de Versao")
        .description("Gerencie multiplas versoes de linguagens")
        .build();

    let version_managers = [
        ("nvm", "Node Version Manager", "nvm --version"),
        ("pyenv", "Python Version Manager", "pyenv --version"),
        ("rustup", "Rust Version Manager", "rustup --version"),
        ("sdkman", "Java SDK Manager", "sdk version"),
        ("rbenv", "Ruby Version Manager", "rbenv --version"),
    ];

    for (cmd, name, version_cmd) in version_managers {
        let parts: Vec<&str> = version_cmd.split_whitespace().collect();
        let version = if parts.len() >= 2 {
            get_command_output(parts[0], &parts[1..])
        } else {
            None
        };

        let row = ActionRow::builder()
            .title(name)
            .subtitle(version.clone().unwrap_or_else(|| "Nao instalado".to_string()).as_str())
            .activatable(true)
            .build();

        if version.is_some() {
            let manage_btn = Button::with_label("Gerenciar");
            manage_btn.add_css_class("flat");
            manage_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&manage_btn);
        } else {
            let install_btn = Button::with_label("Instalar");
            install_btn.add_css_class("flat");
            install_btn.add_css_class("suggested-action");
            install_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&install_btn);
        }

        version_mgr_group.add(&row);
    }

    page.add(&version_mgr_group);

    // Quick Install
    let install_group = PreferencesGroup::builder()
        .title("Instalacao Rapida")
        .description("Instale ferramentas com um clique")
        .build();

    let install_btn = Button::with_label("Abrir Winux Store");
    install_btn.add_css_class("suggested-action");
    install_btn.add_css_class("pill");
    install_btn.set_halign(gtk4::Align::Center);
    install_btn.set_margin_top(12);
    install_btn.set_margin_bottom(12);

    let btn_box = Box::new(Orientation::Vertical, 0);
    btn_box.append(&install_btn);
    install_group.add(&btn_box);

    page.add(&install_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn get_command_output(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let result = if stdout.is_empty() { stderr } else { stdout };
                Some(result.trim().to_string())
            } else {
                None
            }
        })
}

fn create_toolchain_row(
    name: &str,
    version: &Option<String>,
    extra_info: &Option<String>,
    icon: &str,
    installed: bool,
) -> ExpanderRow {
    let row = ExpanderRow::builder()
        .title(name)
        .subtitle(version.clone().unwrap_or_else(|| "Nao instalado".to_string()).as_str())
        .build();

    row.add_prefix(&gtk4::Image::from_icon_name(icon));

    // Status indicator
    let status_icon = if installed {
        "emblem-ok-symbolic"
    } else {
        "dialog-warning-symbolic"
    };

    let status_img = gtk4::Image::from_icon_name(status_icon);
    if installed {
        status_img.add_css_class("success");
    } else {
        status_img.add_css_class("warning");
    }
    row.add_suffix(&status_img);

    // Add extra info if available
    if let Some(info) = extra_info {
        let info_row = ActionRow::builder()
            .title("Detalhes")
            .subtitle(info)
            .build();
        row.add_row(&info_row);
    }

    // Action buttons
    if installed {
        let update_row = ActionRow::builder()
            .title("Atualizar")
            .subtitle("Verificar atualizacoes disponiveis")
            .activatable(true)
            .build();
        update_row.add_suffix(&gtk4::Image::from_icon_name("software-update-available-symbolic"));
        row.add_row(&update_row);

        let uninstall_row = ActionRow::builder()
            .title("Desinstalar")
            .subtitle("Remover do sistema")
            .activatable(true)
            .build();
        uninstall_row.add_suffix(&gtk4::Image::from_icon_name("edit-delete-symbolic"));
        row.add_row(&uninstall_row);
    } else {
        let install_row = ActionRow::builder()
            .title("Instalar")
            .subtitle("Instalar esta ferramenta")
            .activatable(true)
            .build();
        install_row.add_suffix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        row.add_row(&install_row);
    }

    row
}
