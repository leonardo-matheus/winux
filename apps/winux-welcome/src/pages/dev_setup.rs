// Winux Welcome - Development Setup
// Optional page for setting up development environment

use gtk4::prelude::*;
use gtk4::{Box, CheckButton, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, ExpanderRow, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::WelcomeState;

pub fn create_page(state: Rc<RefCell<WelcomeState>>) -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Header
    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(24);
    header_box.set_margin_bottom(12);
    header_box.set_halign(gtk4::Align::Center);

    let title = Label::new(Some("Setup de Desenvolvimento"));
    title.add_css_class("title-1");
    header_box.append(&title);

    let subtitle = Label::new(Some("Configure seu ambiente de desenvolvimento (opcional)"));
    subtitle.add_css_class("dim-label");
    header_box.append(&subtitle);

    let header_group = PreferencesGroup::new();
    header_group.add(&header_box);
    page.add(&header_group);

    // Skip section
    let skip_group = PreferencesGroup::new();
    let skip_row = SwitchRow::builder()
        .title("Configurar ambiente de desenvolvimento")
        .subtitle("Desative se voce nao e desenvolvedor")
        .active(true)
        .build();
    skip_group.add(&skip_row);
    page.add(&skip_group);

    // Programming Languages
    let languages_group = PreferencesGroup::builder()
        .title("Linguagens de Programacao")
        .description("Selecione as linguagens que voce utiliza")
        .build();

    let languages = [
        ("Rust", "Linguagem de sistemas segura e rapida", "rust", false),
        ("Python", "Linguagem versatil para scripting e data science", "python", true),
        (".NET / C#", "Framework da Microsoft para aplicacoes", "dotnet-sdk", false),
        ("Node.js", "Runtime JavaScript para backend", "nodejs", true),
        ("Go", "Linguagem simples e eficiente do Google", "go", false),
        ("Java / OpenJDK", "Plataforma Java para aplicacoes enterprise", "jdk-openjdk", false),
        ("PHP", "Linguagem popular para desenvolvimento web", "php", false),
        ("Ruby", "Linguagem elegante focada em produtividade", "ruby", false),
        ("Kotlin", "Linguagem moderna para JVM e Android", "kotlin", false),
        ("Swift", "Linguagem da Apple para iOS/macOS", "swift", false),
    ];

    for (name, desc, package, default) in languages {
        let row = create_toggle_row(name, desc, package, default, "dev_languages", state.clone());
        languages_group.add(&row);
    }

    page.add(&languages_group);

    // IDEs and Editors
    let ides_group = PreferencesGroup::builder()
        .title("IDEs e Editores")
        .description("Ambientes de desenvolvimento integrados")
        .build();

    let ides = [
        ("Visual Studio Code", "Editor leve e extensivel da Microsoft", "code", true),
        ("JetBrains Toolbox", "Gerenciador de IDEs JetBrains", "jetbrains-toolbox", false),
        ("Neovim", "Editor de texto avancado baseado em Vim", "neovim", false),
        ("Sublime Text", "Editor de texto rapido e elegante", "sublime-text", false),
        ("Eclipse", "IDE para Java e outras linguagens", "eclipse-java", false),
        ("Android Studio", "IDE oficial para desenvolvimento Android", "android-studio", false),
        ("Zed", "Editor moderno e colaborativo", "zed", false),
    ];

    for (name, desc, package, default) in ides {
        let row = create_toggle_row(name, desc, package, default, "dev_ides", state.clone());
        ides_group.add(&row);
    }

    page.add(&ides_group);

    // Tools
    let tools_group = PreferencesGroup::builder()
        .title("Ferramentas")
        .description("Utilitarios de desenvolvimento")
        .build();

    let tools = [
        ("Git", "Controle de versao distribuido", "git", true),
        ("Docker", "Containerizacao de aplicacoes", "docker", true),
        ("Podman", "Alternativa ao Docker sem daemon", "podman", false),
        ("GitHub CLI", "Ferramenta de linha de comando do GitHub", "github-cli", false),
        ("Postman", "Teste de APIs REST", "postman-bin", false),
        ("DBeaver", "Cliente de banco de dados universal", "dbeaver", false),
        ("Insomnia", "Cliente REST/GraphQL", "insomnia", false),
    ];

    for (name, desc, package, default) in tools {
        let row = create_toggle_row(name, desc, package, default, "dev_tools", state.clone());
        tools_group.add(&row);
    }

    page.add(&tools_group);

    // Database Section
    let db_group = PreferencesGroup::builder()
        .title("Bancos de Dados")
        .description("Sistemas de gerenciamento de banco de dados")
        .build();

    let databases = [
        ("PostgreSQL", "Banco de dados relacional avancado", "postgresql", false),
        ("MySQL/MariaDB", "Banco de dados relacional popular", "mariadb", false),
        ("MongoDB", "Banco de dados NoSQL documental", "mongodb-bin", false),
        ("Redis", "Banco de dados em memoria", "redis", false),
        ("SQLite", "Banco de dados embutido leve", "sqlite", true),
    ];

    for (name, desc, package, default) in databases {
        let row = create_toggle_row(name, desc, package, default, "dev_databases", state.clone());
        db_group.add(&row);
    }

    page.add(&db_group);

    // Info
    let info_group = PreferencesGroup::new();
    let info = Label::new(Some("Dica: Voce pode instalar mais ferramentas de desenvolvimento usando o Winux Dev Hub."));
    info.add_css_class("dim-label");
    info.add_css_class("caption");
    info.set_wrap(true);
    info.set_margin_top(12);
    info.set_margin_bottom(12);
    info_group.add(&info);
    page.add(&info_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_toggle_row(
    name: &str,
    description: &str,
    package: &str,
    default: bool,
    category: &str,
    state: Rc<RefCell<WelcomeState>>,
) -> ActionRow {
    let row = ActionRow::builder()
        .title(name)
        .subtitle(description)
        .activatable(true)
        .build();

    let check = CheckButton::new();
    check.set_active(default);
    check.set_valign(gtk4::Align::Center);
    row.add_suffix(&check);
    row.set_activatable_widget(Some(&check));

    let package_name = package.to_string();
    let category_name = category.to_string();
    let state_clone = state.clone();

    // Initialize default
    if default {
        let mut state = state.borrow_mut();
        match category {
            "dev_languages" => {
                if !state.dev_languages.contains(&package_name) {
                    state.dev_languages.push(package_name.clone());
                }
            }
            "dev_ides" => {
                if !state.dev_ides.contains(&package_name) {
                    state.dev_ides.push(package_name.clone());
                }
            }
            _ => {}
        }
    }

    check.connect_toggled(move |btn| {
        let mut state = state_clone.borrow_mut();
        let list = match category_name.as_str() {
            "dev_languages" => &mut state.dev_languages,
            "dev_ides" => &mut state.dev_ides,
            _ => return,
        };

        if btn.is_active() {
            if !list.contains(&package_name) {
                list.push(package_name.clone());
            }
        } else {
            list.retain(|x| x != &package_name);
        }
    });

    row
}
