//! Overview page - Firewall status, toggle, and presets
//!
//! Features:
//! - Firewall status (active/inactive)
//! - Toggle on/off
//! - Default policy (allow/deny incoming/outgoing)
//! - Quick presets (home, public, server)
//! - Statistics summary

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow, StatusPage};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::ufw::UfwBackend;

/// Overview page for firewall status and controls
pub struct OverviewPage {
    widget: ScrolledWindow,
}

impl OverviewPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Visao Geral");
        page.set_icon_name(Some("security-high-symbolic"));

        // Firewall status state
        let firewall_active = Rc::new(RefCell::new(true));

        // Status Group
        let status_group = PreferencesGroup::builder()
            .title("Status do Firewall")
            .description("Controle o estado do firewall do sistema")
            .build();

        // Main firewall toggle
        let firewall_switch = SwitchRow::builder()
            .title("Firewall Ativo")
            .subtitle("Protecao de rede ativada")
            .active(true)
            .build();

        let status_icon = Image::from_icon_name("security-high-symbolic");
        status_icon.add_css_class("success");
        firewall_switch.add_prefix(&status_icon);

        firewall_switch.connect_active_notify({
            let active = firewall_active.clone();
            move |switch| {
                let is_active = switch.is_active();
                *active.borrow_mut() = is_active;
                if is_active {
                    tracing::info!("Enabling firewall...");
                    // UfwBackend::enable();
                } else {
                    tracing::info!("Disabling firewall...");
                    // UfwBackend::disable();
                }
            }
        });

        status_group.add(&firewall_switch);

        // Status info row
        let status_info = ActionRow::builder()
            .title("Estado")
            .subtitle("Ativo desde 2 horas atras")
            .build();

        let uptime_label = Label::new(Some("2h 34m"));
        uptime_label.add_css_class("dim-label");
        status_info.add_suffix(&uptime_label);

        status_group.add(&status_info);

        page.add(&status_group);

        // Default Policy Group
        let policy_group = PreferencesGroup::builder()
            .title("Politica Padrao")
            .description("Defina o comportamento padrao para conexoes")
            .build();

        // Incoming policy
        let incoming_policy = ComboRow::builder()
            .title("Conexoes de Entrada")
            .subtitle("Politica para trafego recebido")
            .build();

        let incoming_model = gtk4::StringList::new(&["Negar (Deny)", "Rejeitar (Reject)", "Permitir (Allow)"]);
        incoming_policy.set_model(Some(&incoming_model));
        incoming_policy.set_selected(0); // Default: deny

        let incoming_icon = Image::from_icon_name("go-down-symbolic");
        incoming_policy.add_prefix(&incoming_icon);

        incoming_policy.connect_selected_notify(|row| {
            let selected = row.selected();
            let policy = match selected {
                0 => "deny",
                1 => "reject",
                2 => "allow",
                _ => "deny",
            };
            tracing::info!("Setting incoming policy to: {}", policy);
            // UfwBackend::set_default_incoming(policy);
        });

        policy_group.add(&incoming_policy);

        // Outgoing policy
        let outgoing_policy = ComboRow::builder()
            .title("Conexoes de Saida")
            .subtitle("Politica para trafego enviado")
            .build();

        let outgoing_model = gtk4::StringList::new(&["Permitir (Allow)", "Negar (Deny)", "Rejeitar (Reject)"]);
        outgoing_policy.set_model(Some(&outgoing_model));
        outgoing_policy.set_selected(0); // Default: allow

        let outgoing_icon = Image::from_icon_name("go-up-symbolic");
        outgoing_policy.add_prefix(&outgoing_icon);

        outgoing_policy.connect_selected_notify(|row| {
            let selected = row.selected();
            let policy = match selected {
                0 => "allow",
                1 => "deny",
                2 => "reject",
                _ => "allow",
            };
            tracing::info!("Setting outgoing policy to: {}", policy);
            // UfwBackend::set_default_outgoing(policy);
        });

        policy_group.add(&outgoing_policy);

        page.add(&policy_group);

        // Presets Group
        let presets_group = PreferencesGroup::builder()
            .title("Presets de Seguranca")
            .description("Configuracoes rapidas para diferentes cenarios")
            .build();

        // Home preset
        let home_preset = ActionRow::builder()
            .title("Modo Casa")
            .subtitle("Permite descoberta de rede local, compartilhamento de arquivos")
            .activatable(true)
            .build();

        let home_icon = Image::from_icon_name("user-home-symbolic");
        home_preset.add_prefix(&home_icon);

        let home_btn = Button::with_label("Aplicar");
        home_btn.add_css_class("flat");
        home_btn.set_valign(gtk4::Align::Center);
        home_btn.connect_clicked(|_| {
            tracing::info!("Applying home preset...");
            // Apply home-friendly rules
        });
        home_preset.add_suffix(&home_btn);

        presets_group.add(&home_preset);

        // Public preset
        let public_preset = ActionRow::builder()
            .title("Modo Publico")
            .subtitle("Mais restritivo, ideal para redes publicas/WiFi")
            .activatable(true)
            .build();

        let public_icon = Image::from_icon_name("network-wireless-symbolic");
        public_preset.add_prefix(&public_icon);

        let public_btn = Button::with_label("Aplicar");
        public_btn.add_css_class("flat");
        public_btn.set_valign(gtk4::Align::Center);
        public_btn.connect_clicked(|_| {
            tracing::info!("Applying public preset...");
            // Apply restrictive rules
        });
        public_preset.add_suffix(&public_btn);

        presets_group.add(&public_preset);

        // Server preset
        let server_preset = ActionRow::builder()
            .title("Modo Servidor")
            .subtitle("Permite apenas portas especificas (SSH, HTTP, HTTPS)")
            .activatable(true)
            .build();

        let server_icon = Image::from_icon_name("network-server-symbolic");
        server_preset.add_prefix(&server_icon);

        let server_btn = Button::with_label("Aplicar");
        server_btn.add_css_class("flat");
        server_btn.set_valign(gtk4::Align::Center);
        server_btn.connect_clicked(|_| {
            tracing::info!("Applying server preset...");
            // Apply server rules
        });
        server_preset.add_suffix(&server_btn);

        presets_group.add(&server_preset);

        // Custom preset
        let custom_expander = ExpanderRow::builder()
            .title("Preset Personalizado")
            .subtitle("Crie sua propria configuracao")
            .build();

        let custom_name = adw::EntryRow::builder()
            .title("Nome do Preset")
            .build();
        custom_expander.add_row(&custom_name);

        let save_preset_row = ActionRow::builder()
            .build();

        let save_preset_btn = Button::with_label("Salvar Configuracao Atual");
        save_preset_btn.add_css_class("suggested-action");
        save_preset_btn.set_halign(gtk4::Align::Center);
        save_preset_btn.set_margin_top(8);
        save_preset_btn.set_margin_bottom(8);
        save_preset_row.set_child(Some(&save_preset_btn));

        custom_expander.add_row(&save_preset_row);

        presets_group.add(&custom_expander);

        page.add(&presets_group);

        // Statistics Group
        let stats_group = PreferencesGroup::builder()
            .title("Estatisticas")
            .description("Resumo de atividade do firewall")
            .build();

        let blocked_row = ActionRow::builder()
            .title("Conexoes Bloqueadas")
            .subtitle("Nas ultimas 24 horas")
            .build();

        let blocked_count = Label::new(Some("127"));
        blocked_count.add_css_class("title-2");
        blocked_count.add_css_class("error");
        blocked_row.add_suffix(&blocked_count);

        stats_group.add(&blocked_row);

        let allowed_row = ActionRow::builder()
            .title("Conexoes Permitidas")
            .subtitle("Nas ultimas 24 horas")
            .build();

        let allowed_count = Label::new(Some("2,847"));
        allowed_count.add_css_class("title-2");
        allowed_count.add_css_class("success");
        allowed_row.add_suffix(&allowed_count);

        stats_group.add(&allowed_row);

        let rules_row = ActionRow::builder()
            .title("Regras Ativas")
            .subtitle("Total de regras configuradas")
            .build();

        let rules_count = Label::new(Some("12"));
        rules_count.add_css_class("title-2");
        rules_row.add_suffix(&rules_count);

        stats_group.add(&rules_row);

        let view_logs_row = ActionRow::builder()
            .title("Ver Logs Completos")
            .subtitle("Abrir pagina de logs detalhados")
            .activatable(true)
            .build();

        let logs_icon = Image::from_icon_name("go-next-symbolic");
        view_logs_row.add_suffix(&logs_icon);

        stats_group.add(&view_logs_row);

        page.add(&stats_group);

        // IPv6 Group
        let ipv6_group = PreferencesGroup::builder()
            .title("IPv6")
            .description("Configuracoes para protocolo IPv6")
            .build();

        let ipv6_switch = SwitchRow::builder()
            .title("Habilitar Firewall IPv6")
            .subtitle("Aplicar regras tambem para trafego IPv6")
            .active(true)
            .build();

        ipv6_switch.connect_active_notify(|switch| {
            let enabled = switch.is_active();
            tracing::info!("IPv6 firewall: {}", if enabled { "enabled" } else { "disabled" });
        });

        ipv6_group.add(&ipv6_switch);

        page.add(&ipv6_group);

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

impl Default for OverviewPage {
    fn default() -> Self {
        Self::new()
    }
}
