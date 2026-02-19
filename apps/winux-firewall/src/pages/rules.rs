//! Rules page - Manage firewall rules
//!
//! Features:
//! - List existing rules
//! - Add new rules
//! - Edit/delete rules
//! - Rule ordering
//! - Import/export rules

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ScrolledWindow, Separator};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::ufw::{FirewallRule, Protocol, Direction, Action};
use crate::ui::RuleRow;

/// Rules management page
pub struct RulesPage {
    widget: ScrolledWindow,
}

impl RulesPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Regras");
        page.set_icon_name(Some("view-list-symbolic"));

        // Add Rule Group
        let add_group = PreferencesGroup::builder()
            .title("Nova Regra")
            .description("Adicione uma nova regra de firewall")
            .build();

        let add_expander = ExpanderRow::builder()
            .title("Criar Regra")
            .subtitle("Clique para expandir o formulario")
            .build();

        let add_icon = Image::from_icon_name("list-add-symbolic");
        add_expander.add_prefix(&add_icon);

        // Port entry
        let port_entry = EntryRow::builder()
            .title("Porta")
            .text("")
            .build();
        port_entry.set_input_purpose(gtk4::InputPurpose::Number);
        add_expander.add_row(&port_entry);

        // Service name (alternative to port)
        let service_entry = EntryRow::builder()
            .title("Ou Nome do Servico")
            .text("")
            .build();
        add_expander.add_row(&service_entry);

        // Protocol selection
        let protocol_row = ComboRow::builder()
            .title("Protocolo")
            .subtitle("TCP, UDP ou ambos")
            .build();

        let protocol_model = gtk4::StringList::new(&["TCP", "UDP", "TCP/UDP"]);
        protocol_row.set_model(Some(&protocol_model));
        protocol_row.set_selected(0);

        add_expander.add_row(&protocol_row);

        // Direction selection
        let direction_row = ComboRow::builder()
            .title("Direcao")
            .subtitle("Entrada ou saida")
            .build();

        let direction_model = gtk4::StringList::new(&["Entrada (IN)", "Saida (OUT)", "Ambas"]);
        direction_row.set_model(Some(&direction_model));
        direction_row.set_selected(0);

        add_expander.add_row(&direction_row);

        // Action selection
        let action_row = ComboRow::builder()
            .title("Acao")
            .subtitle("O que fazer com o trafego")
            .build();

        let action_model = gtk4::StringList::new(&["Permitir (ALLOW)", "Negar (DENY)", "Rejeitar (REJECT)"]);
        action_row.set_model(Some(&action_model));
        action_row.set_selected(0);

        add_expander.add_row(&action_row);

        // Source IP (optional)
        let source_expander = ExpanderRow::builder()
            .title("Filtros Avancados")
            .subtitle("IP de origem/destino (opcional)")
            .build();

        let source_ip_entry = EntryRow::builder()
            .title("IP de Origem")
            .text("")
            .build();
        source_expander.add_row(&source_ip_entry);

        let dest_ip_entry = EntryRow::builder()
            .title("IP de Destino")
            .text("")
            .build();
        source_expander.add_row(&dest_ip_entry);

        let interface_entry = EntryRow::builder()
            .title("Interface de Rede")
            .text("")
            .build();
        source_expander.add_row(&interface_entry);

        add_expander.add_row(&source_expander);

        // Comment
        let comment_entry = EntryRow::builder()
            .title("Comentario")
            .text("")
            .build();
        add_expander.add_row(&comment_entry);

        // Add button
        let add_btn_row = ActionRow::builder()
            .build();

        let add_btn = Button::with_label("Adicionar Regra");
        add_btn.add_css_class("suggested-action");
        add_btn.set_halign(gtk4::Align::Center);
        add_btn.set_margin_top(8);
        add_btn.set_margin_bottom(8);

        add_btn.connect_clicked({
            let port = port_entry.clone();
            let service = service_entry.clone();
            let protocol = protocol_row.clone();
            let direction = direction_row.clone();
            let action = action_row.clone();
            let source = source_ip_entry.clone();
            let dest = dest_ip_entry.clone();
            let comment = comment_entry.clone();
            move |_| {
                let port_text = port.text().to_string();
                let service_text = service.text().to_string();
                let proto = match protocol.selected() {
                    0 => "tcp",
                    1 => "udp",
                    _ => "tcp/udp",
                };
                let dir = match direction.selected() {
                    0 => "in",
                    1 => "out",
                    _ => "both",
                };
                let act = match action.selected() {
                    0 => "allow",
                    1 => "deny",
                    _ => "reject",
                };

                tracing::info!(
                    "Adding rule: port={}, service={}, proto={}, dir={}, action={}",
                    port_text, service_text, proto, dir, act
                );

                // UfwBackend::add_rule(...);
            }
        });

        add_btn_row.set_child(Some(&add_btn));
        add_expander.add_row(&add_btn_row);

        add_group.add(&add_expander);
        page.add(&add_group);

        // Quick Add Group
        let quick_group = PreferencesGroup::builder()
            .title("Adicao Rapida")
            .description("Regras comuns com um clique")
            .build();

        let common_services = [
            ("SSH", "22", "tcp", "Acesso remoto seguro"),
            ("HTTP", "80", "tcp", "Servidor web"),
            ("HTTPS", "443", "tcp", "Servidor web seguro"),
            ("FTP", "21", "tcp", "Transferencia de arquivos"),
            ("DNS", "53", "udp", "Resolucao de nomes"),
            ("SMTP", "25", "tcp", "Email (envio)"),
            ("POP3", "110", "tcp", "Email (recebimento)"),
            ("IMAP", "143", "tcp", "Email (IMAP)"),
            ("MySQL", "3306", "tcp", "Banco de dados MySQL"),
            ("PostgreSQL", "5432", "tcp", "Banco de dados PostgreSQL"),
        ];

        for (name, port, proto, desc) in common_services {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("Porta {}/{} - {}", port, proto.to_uppercase(), desc))
                .activatable(true)
                .build();

            let port_label = Label::new(Some(port));
            port_label.add_css_class("dim-label");
            row.add_suffix(&port_label);

            let allow_btn = Button::with_label("Permitir");
            allow_btn.add_css_class("success");
            allow_btn.add_css_class("flat");
            allow_btn.set_valign(gtk4::Align::Center);

            let port_clone = port.to_string();
            let proto_clone = proto.to_string();
            let name_clone = name.to_string();
            allow_btn.connect_clicked(move |_| {
                tracing::info!("Quick allow: {} port {}/{}", name_clone, port_clone, proto_clone);
                // UfwBackend::allow_port(&port_clone, &proto_clone);
            });

            let deny_btn = Button::with_label("Negar");
            deny_btn.add_css_class("error");
            deny_btn.add_css_class("flat");
            deny_btn.set_valign(gtk4::Align::Center);

            let port_clone = port.to_string();
            let proto_clone = proto.to_string();
            let name_clone = name.to_string();
            deny_btn.connect_clicked(move |_| {
                tracing::info!("Quick deny: {} port {}/{}", name_clone, port_clone, proto_clone);
                // UfwBackend::deny_port(&port_clone, &proto_clone);
            });

            row.add_suffix(&allow_btn);
            row.add_suffix(&deny_btn);

            quick_group.add(&row);
        }

        page.add(&quick_group);

        // Existing Rules Group
        let rules_group = PreferencesGroup::builder()
            .title("Regras Existentes")
            .description("Gerencie regras configuradas")
            .build();

        // Sample rules for demonstration
        let sample_rules = [
            ("[ 1] 22/tcp", "ALLOW", "Anywhere", "SSH"),
            ("[ 2] 80/tcp", "ALLOW", "Anywhere", "HTTP"),
            ("[ 3] 443/tcp", "ALLOW", "Anywhere", "HTTPS"),
            ("[ 4] 3306/tcp", "DENY", "Anywhere", "MySQL (blocked)"),
            ("[ 5] 8080/tcp", "ALLOW", "192.168.1.0/24", "Dev server (local only)"),
        ];

        for (rule_num, action, from, comment) in sample_rules {
            let row = ExpanderRow::builder()
                .title(rule_num)
                .subtitle(&format!("{} from {} - {}", action, from, comment))
                .build();

            // Action indicator
            let action_icon = if action == "ALLOW" {
                let icon = Image::from_icon_name("emblem-ok-symbolic");
                icon.add_css_class("success");
                icon
            } else {
                let icon = Image::from_icon_name("action-unavailable-symbolic");
                icon.add_css_class("error");
                icon
            };
            row.add_prefix(&action_icon);

            // Edit row
            let edit_row = ActionRow::builder()
                .title("Editar Regra")
                .activatable(true)
                .build();

            let edit_icon = Image::from_icon_name("document-edit-symbolic");
            edit_row.add_prefix(&edit_icon);

            let rule_clone = rule_num.to_string();
            edit_row.connect_activated(move |_| {
                tracing::info!("Editing rule: {}", rule_clone);
            });
            row.add_row(&edit_row);

            // Move up/down
            let order_row = ActionRow::builder()
                .title("Ordenar")
                .build();

            let up_btn = Button::from_icon_name("go-up-symbolic");
            up_btn.add_css_class("flat");
            up_btn.set_tooltip_text(Some("Mover para cima"));
            up_btn.connect_clicked({
                let rule = rule_num.to_string();
                move |_| {
                    tracing::info!("Moving rule up: {}", rule);
                }
            });

            let down_btn = Button::from_icon_name("go-down-symbolic");
            down_btn.add_css_class("flat");
            down_btn.set_tooltip_text(Some("Mover para baixo"));
            down_btn.connect_clicked({
                let rule = rule_num.to_string();
                move |_| {
                    tracing::info!("Moving rule down: {}", rule);
                }
            });

            let order_box = Box::new(Orientation::Horizontal, 4);
            order_box.append(&up_btn);
            order_box.append(&down_btn);
            order_row.add_suffix(&order_box);

            row.add_row(&order_row);

            // Delete row
            let delete_row = ActionRow::builder()
                .title("Excluir Regra")
                .activatable(true)
                .build();
            delete_row.add_css_class("error");

            let delete_icon = Image::from_icon_name("user-trash-symbolic");
            delete_row.add_prefix(&delete_icon);

            let rule_clone = rule_num.to_string();
            delete_row.connect_activated(move |_| {
                tracing::info!("Deleting rule: {}", rule_clone);
                // UfwBackend::delete_rule(&rule_clone);
            });
            row.add_row(&delete_row);

            rules_group.add(&row);
        }

        // Refresh rules button
        let refresh_row = ActionRow::builder()
            .title("Atualizar Lista")
            .subtitle("Recarregar regras do sistema")
            .activatable(true)
            .build();

        let refresh_icon = Image::from_icon_name("view-refresh-symbolic");
        refresh_row.add_prefix(&refresh_icon);

        refresh_row.connect_activated(|_| {
            tracing::info!("Refreshing rules list...");
            // UfwBackend::list_rules();
        });

        rules_group.add(&refresh_row);

        page.add(&rules_group);

        // Import/Export Group
        let io_group = PreferencesGroup::builder()
            .title("Importar / Exportar")
            .description("Backup e restauracao de regras")
            .build();

        let export_row = ActionRow::builder()
            .title("Exportar Regras")
            .subtitle("Salvar configuracao atual em arquivo")
            .activatable(true)
            .build();

        let export_icon = Image::from_icon_name("document-save-symbolic");
        export_row.add_prefix(&export_icon);

        export_row.connect_activated(|_| {
            tracing::info!("Exporting rules...");
        });

        io_group.add(&export_row);

        let import_row = ActionRow::builder()
            .title("Importar Regras")
            .subtitle("Carregar configuracao de arquivo")
            .activatable(true)
            .build();

        let import_icon = Image::from_icon_name("document-open-symbolic");
        import_row.add_prefix(&import_icon);

        import_row.connect_activated(|_| {
            tracing::info!("Importing rules...");
        });

        io_group.add(&import_row);

        let reset_row = ActionRow::builder()
            .title("Redefinir para Padrao")
            .subtitle("Remove todas as regras e restaura configuracao inicial")
            .activatable(true)
            .build();
        reset_row.add_css_class("error");

        let reset_icon = Image::from_icon_name("edit-undo-symbolic");
        reset_row.add_prefix(&reset_icon);

        reset_row.connect_activated(|_| {
            tracing::info!("Resetting firewall to defaults...");
            // UfwBackend::reset();
        });

        io_group.add(&reset_row);

        page.add(&io_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}

impl Default for RulesPage {
    fn default() -> Self {
        Self::new()
    }
}
