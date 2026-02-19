// Winux Dev Hub - Environments Page
// Copyright (c) 2026 Winux OS Project
//
// Environment variables management with profiles

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, ListBox, Orientation, ScrolledWindow, TextView};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, EntryRow, ExpanderRow, PreferencesGroup, PreferencesPage,
    SwitchRow, ComboRow, ToastOverlay,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EnvProfile {
    pub name: String,
    pub description: String,
    pub variables: HashMap<String, String>,
    pub active: bool,
}

pub fn create_page() -> ScrolledWindow {
    let page = PreferencesPage::new();

    // Active Profile Section
    let profile_group = PreferencesGroup::builder()
        .title("Perfil Ativo")
        .description("Selecione o perfil de ambiente")
        .build();

    let profile_row = ComboRow::builder()
        .title("Perfil")
        .subtitle("Conjunto de variaveis de ambiente")
        .build();

    let profiles = gtk4::StringList::new(&[
        "Development (dev)",
        "Staging (staging)",
        "Production (prod)",
        "Custom",
    ]);
    profile_row.set_model(Some(&profiles));
    profile_group.add(&profile_row);

    // Quick switch buttons
    let switch_box = Box::new(Orientation::Horizontal, 8);
    switch_box.set_halign(gtk4::Align::Center);
    switch_box.set_margin_top(12);
    switch_box.set_margin_bottom(12);

    for (name, class) in [("Dev", "suggested-action"), ("Staging", ""), ("Prod", "destructive-action")] {
        let btn = Button::with_label(name);
        btn.add_css_class("pill");
        if !class.is_empty() {
            btn.add_css_class(class);
        }
        switch_box.append(&btn);
    }

    let switch_row = ActionRow::new();
    switch_row.set_child(Some(&switch_box));
    profile_group.add(&switch_row);

    page.add(&profile_group);

    // Environment Variables Section
    let env_group = PreferencesGroup::builder()
        .title("Variaveis de Ambiente")
        .description("Variaveis do sistema atual")
        .build();

    // Development variables
    let dev_expander = ExpanderRow::builder()
        .title("Desenvolvimento")
        .subtitle("Variaveis comuns de desenvolvimento")
        .build();

    let dev_vars = [
        ("NODE_ENV", "development"),
        ("DEBUG", "true"),
        ("LOG_LEVEL", "debug"),
        ("API_URL", "http://localhost:3000"),
        ("DATABASE_URL", "postgres://localhost/dev_db"),
    ];

    for (name, value) in dev_vars {
        add_env_variable_row(&dev_expander, name, value);
    }
    env_group.add(&dev_expander);

    // Runtime variables
    let runtime_expander = ExpanderRow::builder()
        .title("Runtime")
        .subtitle("Variaveis de execucao")
        .build();

    let runtime_vars = [
        ("PATH", &env::var("PATH").unwrap_or_default()[..50.min(env::var("PATH").unwrap_or_default().len())]),
        ("HOME", &env::var("HOME").unwrap_or_default()),
        ("SHELL", &env::var("SHELL").unwrap_or_default()),
        ("LANG", &env::var("LANG").unwrap_or_else(|_| "pt_BR.UTF-8".to_string())),
        ("EDITOR", &env::var("EDITOR").unwrap_or_else(|_| "code".to_string())),
    ];

    for (name, value) in runtime_vars {
        add_env_variable_row(&runtime_expander, name, value);
    }
    env_group.add(&runtime_expander);

    // Custom variables
    let custom_expander = ExpanderRow::builder()
        .title("Personalizadas")
        .subtitle("Suas variaveis customizadas")
        .build();

    let add_var_btn = Button::with_label("+ Adicionar Variavel");
    add_var_btn.add_css_class("flat");
    let add_row = ActionRow::new();
    add_row.set_child(Some(&add_var_btn));
    custom_expander.add_row(&add_row);

    env_group.add(&custom_expander);

    page.add(&env_group);

    // .env File Manager
    let dotenv_group = PreferencesGroup::builder()
        .title("Arquivos .env")
        .description("Gerencie arquivos de ambiente do projeto")
        .build();

    let dotenv_files = [
        (".env", "Variaveis padrao", true),
        (".env.local", "Overrides locais", true),
        (".env.development", "Ambiente de desenvolvimento", false),
        (".env.production", "Ambiente de producao", false),
        (".env.test", "Ambiente de testes", false),
    ];

    for (filename, desc, exists) in dotenv_files {
        let row = ActionRow::builder()
            .title(filename)
            .subtitle(desc)
            .activatable(true)
            .build();

        let icon = if exists {
            "emblem-ok-symbolic"
        } else {
            "list-add-symbolic"
        };
        row.add_prefix(&gtk4::Image::from_icon_name(icon));

        if exists {
            let edit_btn = Button::from_icon_name("document-edit-symbolic");
            edit_btn.add_css_class("flat");
            edit_btn.set_valign(gtk4::Align::Center);
            edit_btn.set_tooltip_text(Some("Editar arquivo"));
            row.add_suffix(&edit_btn);
        }

        row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        dotenv_group.add(&row);
    }

    // Template generator
    let template_row = ActionRow::builder()
        .title("Gerar .env de Template")
        .subtitle("Crie um arquivo .env a partir de um template")
        .activatable(true)
        .build();
    template_row.add_prefix(&gtk4::Image::from_icon_name("document-new-symbolic"));
    template_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    dotenv_group.add(&template_row);

    page.add(&dotenv_group);

    // Secret Management
    let secrets_group = PreferencesGroup::builder()
        .title("Gerenciamento de Segredos")
        .description("Armazene credenciais de forma segura")
        .build();

    let keyring_row = SwitchRow::builder()
        .title("Usar Keyring do Sistema")
        .subtitle("Armazena segredos no keyring do GNOME")
        .active(true)
        .build();
    secrets_group.add(&keyring_row);

    let encrypt_row = SwitchRow::builder()
        .title("Criptografar .env.local")
        .subtitle("Usa GPG para criptografar variaveis sensiveis")
        .active(false)
        .build();
    secrets_group.add(&encrypt_row);

    let vault_row = ActionRow::builder()
        .title("Integrar com Vault")
        .subtitle("HashiCorp Vault para gerenciamento de segredos")
        .activatable(true)
        .build();
    vault_row.add_prefix(&gtk4::Image::from_icon_name("dialog-password-symbolic"));
    vault_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
    secrets_group.add(&vault_row);

    page.add(&secrets_group);

    // Export/Import
    let export_group = PreferencesGroup::builder()
        .title("Exportar / Importar")
        .build();

    let export_row = ActionRow::builder()
        .title("Exportar Configuracoes")
        .subtitle("Salve suas variaveis em um arquivo")
        .activatable(true)
        .build();
    export_row.add_prefix(&gtk4::Image::from_icon_name("document-save-symbolic"));
    export_group.add(&export_row);

    let import_row = ActionRow::builder()
        .title("Importar Configuracoes")
        .subtitle("Carregue variaveis de um arquivo")
        .activatable(true)
        .build();
    import_row.add_prefix(&gtk4::Image::from_icon_name("document-open-symbolic"));
    export_group.add(&import_row);

    page.add(&export_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn add_env_variable_row(expander: &ExpanderRow, name: &str, value: &str) {
    let row = ActionRow::builder()
        .title(name)
        .subtitle(if value.len() > 50 { &value[..50] } else { value })
        .build();

    let copy_btn = Button::from_icon_name("edit-copy-symbolic");
    copy_btn.add_css_class("flat");
    copy_btn.set_valign(gtk4::Align::Center);
    copy_btn.set_tooltip_text(Some("Copiar valor"));
    row.add_suffix(&copy_btn);

    let edit_btn = Button::from_icon_name("document-edit-symbolic");
    edit_btn.add_css_class("flat");
    edit_btn.set_valign(gtk4::Align::Center);
    edit_btn.set_tooltip_text(Some("Editar"));
    row.add_suffix(&edit_btn);

    expander.add_row(&row);
}
