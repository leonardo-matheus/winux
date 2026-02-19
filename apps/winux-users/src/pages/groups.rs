//! Groups management page

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, EntryRow, PreferencesGroup, PreferencesPage, ExpanderRow, StatusPage};

use crate::backend::AccountsService;

use std::cell::RefCell;
use std::rc::Rc;

/// Groups management page
pub struct GroupsPage {
    widget: ScrolledWindow,
    accounts_service: Rc<RefCell<AccountsService>>,
}

impl GroupsPage {
    pub fn new(accounts_service: Rc<RefCell<AccountsService>>) -> Self {
        let page = PreferencesPage::new();
        page.set_title("Grupos");
        page.set_icon_name(Some("system-users-symbolic"));

        // Header
        let header_group = PreferencesGroup::new();

        let status = StatusPage::builder()
            .icon_name("system-users-symbolic")
            .title("Gerenciamento de Grupos")
            .description("Organize usuarios em grupos para gerenciar permissoes")
            .build();

        let header_box = Box::new(Orientation::Vertical, 12);
        header_box.append(&status);

        // Add group button
        let add_btn = Button::with_label("Criar Grupo");
        add_btn.add_css_class("suggested-action");
        add_btn.add_css_class("pill");
        add_btn.set_halign(gtk4::Align::Center);
        add_btn.connect_clicked(|_| {
            tracing::info!("Create group clicked");
            // TODO: Show create group dialog
        });
        header_box.append(&add_btn);
        header_box.set_margin_bottom(24);

        header_group.add(&header_box);
        page.add(&header_group);

        // System groups
        let system_group = PreferencesGroup::builder()
            .title("Grupos do Sistema")
            .description("Grupos com permissoes especiais")
            .build();

        let system_groups = [
            ("wheel", "Administradores com acesso sudo", vec!["root", "admin"]),
            ("sudo", "Usuarios com permissao sudo", vec!["admin"]),
            ("audio", "Acesso a dispositivos de audio", vec!["user1", "user2"]),
            ("video", "Acesso a dispositivos de video", vec!["user1", "user2"]),
            ("storage", "Acesso a dispositivos de armazenamento", vec!["user1"]),
            ("network", "Gerenciamento de rede", vec!["admin"]),
            ("docker", "Acesso ao Docker", vec!["developer"]),
            ("libvirt", "Acesso a virtualizacao", vec!["developer"]),
            ("kvm", "Acesso ao KVM", vec!["developer"]),
        ];

        for (name, description, members) in system_groups {
            let expander = ExpanderRow::builder()
                .title(name)
                .subtitle(description)
                .build();

            expander.add_prefix(&gtk4::Image::from_icon_name("system-users-symbolic"));

            // Member count
            let count_label = gtk4::Label::new(Some(&format!("{} membros", members.len())));
            count_label.add_css_class("dim-label");
            expander.add_suffix(&count_label);

            // Add members as rows
            for member in &members {
                let member_row = ActionRow::builder()
                    .title(*member)
                    .build();

                let avatar = adw::Avatar::new(32, Some(*member), true);
                member_row.add_prefix(&avatar);

                let remove_btn = Button::from_icon_name("list-remove-symbolic");
                remove_btn.set_tooltip_text(Some("Remover do grupo"));
                remove_btn.add_css_class("flat");
                remove_btn.set_valign(gtk4::Align::Center);
                member_row.add_suffix(&remove_btn);

                expander.add_row(&member_row);
            }

            // Add member row
            let add_member_row = ActionRow::builder()
                .title("Adicionar membro...")
                .activatable(true)
                .build();
            add_member_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
            expander.add_row(&add_member_row);

            system_group.add(&expander);
        }

        page.add(&system_group);

        // Custom groups
        let custom_group = PreferencesGroup::builder()
            .title("Grupos Personalizados")
            .description("Grupos criados pelo usuario")
            .build();

        let custom_groups = [
            ("projeto-alpha", "Equipe do Projeto Alpha", vec!["alice", "bob", "carol"]),
            ("desenvolvimento", "Desenvolvedores", vec!["alice", "david"]),
        ];

        for (name, description, members) in custom_groups {
            let expander = ExpanderRow::builder()
                .title(name)
                .subtitle(description)
                .build();

            expander.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));

            // Member count
            let count_label = gtk4::Label::new(Some(&format!("{} membros", members.len())));
            count_label.add_css_class("dim-label");
            expander.add_suffix(&count_label);

            // Edit button
            let edit_btn = Button::from_icon_name("document-edit-symbolic");
            edit_btn.set_tooltip_text(Some("Editar grupo"));
            edit_btn.add_css_class("flat");
            edit_btn.set_valign(gtk4::Align::Center);
            expander.add_suffix(&edit_btn);

            // Delete button
            let delete_btn = Button::from_icon_name("user-trash-symbolic");
            delete_btn.set_tooltip_text(Some("Excluir grupo"));
            delete_btn.add_css_class("flat");
            delete_btn.set_valign(gtk4::Align::Center);
            expander.add_suffix(&delete_btn);

            // Add members as rows
            for member in &members {
                let member_row = ActionRow::builder()
                    .title(*member)
                    .build();

                let avatar = adw::Avatar::new(32, Some(*member), true);
                member_row.add_prefix(&avatar);

                let remove_btn = Button::from_icon_name("list-remove-symbolic");
                remove_btn.set_tooltip_text(Some("Remover do grupo"));
                remove_btn.add_css_class("flat");
                remove_btn.set_valign(gtk4::Align::Center);
                member_row.add_suffix(&remove_btn);

                expander.add_row(&member_row);
            }

            // Add member row
            let add_member_row = ActionRow::builder()
                .title("Adicionar membro...")
                .activatable(true)
                .build();
            add_member_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
            expander.add_row(&add_member_row);

            custom_group.add(&expander);
        }

        // No custom groups message
        if custom_groups.is_empty() {
            let no_groups_row = ActionRow::builder()
                .title("Nenhum grupo personalizado")
                .subtitle("Clique em 'Criar Grupo' para adicionar")
                .build();
            no_groups_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
            custom_group.add(&no_groups_row);
        }

        page.add(&custom_group);

        // Group permissions info
        let info_group = PreferencesGroup::builder()
            .title("Sobre Grupos")
            .build();

        let info_items = [
            ("wheel/sudo", "Permite executar comandos como administrador"),
            ("audio", "Acesso direto a dispositivos de som"),
            ("video", "Acesso a aceleracao de video e GPU"),
            ("docker", "Executar containers sem sudo"),
            ("libvirt", "Gerenciar maquinas virtuais"),
        ];

        for (group, description) in info_items {
            let row = ActionRow::builder()
                .title(group)
                .subtitle(description)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
            info_group.add(&row);
        }

        page.add(&info_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        GroupsPage {
            widget: scrolled,
            accounts_service,
        }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    pub fn refresh(&self) {
        tracing::info!("Refreshing groups list");
        // TODO: Reload groups from system
    }
}

/// Group information
#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
    pub description: String,
    pub is_system: bool,
}

impl GroupInfo {
    /// Parse groups from /etc/group
    pub fn from_etc_group() -> Vec<GroupInfo> {
        let mut groups = Vec::new();

        if let Ok(content) = std::fs::read_to_string("/etc/group") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let name = parts[0].to_string();
                    let gid: u32 = parts[2].parse().unwrap_or(0);
                    let members: Vec<String> = parts[3]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();

                    let is_system = gid < 1000;

                    groups.push(GroupInfo {
                        name,
                        gid,
                        members,
                        description: String::new(),
                        is_system,
                    });
                }
            }
        }

        groups
    }
}
