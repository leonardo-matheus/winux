//! Logs page - Firewall activity logs
//!
//! Features:
//! - View blocked attempts
//! - Connection statistics
//! - Export logs
//! - Filter and search

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, ListBox, Orientation, ScrolledWindow, SearchEntry, Spinner};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, Duration};

/// Logs page for firewall activity
pub struct LogsPage {
    widget: ScrolledWindow,
}

impl LogsPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Logs");
        page.set_icon_name(Some("document-open-recent-symbolic"));

        // Logging Control Group
        let control_group = PreferencesGroup::builder()
            .title("Controle de Logs")
            .description("Configure o registro de atividades")
            .build();

        // Logging toggle
        let logging_switch = SwitchRow::builder()
            .title("Habilitar Logging")
            .subtitle("Registrar atividade do firewall")
            .active(true)
            .build();

        logging_switch.connect_active_notify(|switch| {
            let enabled = switch.is_active();
            tracing::info!("Firewall logging: {}", if enabled { "enabled" } else { "disabled" });
            // UfwBackend::set_logging(enabled);
        });

        control_group.add(&logging_switch);

        // Log level
        let log_level = ComboRow::builder()
            .title("Nivel de Log")
            .subtitle("Quantidade de detalhes registrados")
            .build();

        let level_model = gtk4::StringList::new(&["Baixo", "Medio", "Alto", "Completo"]);
        log_level.set_model(Some(&level_model));
        log_level.set_selected(1);

        log_level.connect_selected_notify(|row| {
            let level = match row.selected() {
                0 => "low",
                1 => "medium",
                2 => "high",
                3 => "full",
                _ => "medium",
            };
            tracing::info!("Setting log level to: {}", level);
            // UfwBackend::set_log_level(level);
        });

        control_group.add(&log_level);

        page.add(&control_group);

        // Statistics Group
        let stats_group = PreferencesGroup::builder()
            .title("Estatisticas")
            .description("Resumo de atividade")
            .build();

        // Time range selector
        let time_range = ComboRow::builder()
            .title("Periodo")
            .subtitle("Filtrar por intervalo de tempo")
            .build();

        let time_model = gtk4::StringList::new(&[
            "Ultima hora",
            "Ultimas 24 horas",
            "Ultimos 7 dias",
            "Ultimos 30 dias",
            "Todo o historico",
        ]);
        time_range.set_model(Some(&time_model));
        time_range.set_selected(1);

        stats_group.add(&time_range);

        // Statistics rows
        let blocked_row = ActionRow::builder()
            .title("Conexoes Bloqueadas")
            .build();

        let blocked_count = Label::new(Some("127"));
        blocked_count.add_css_class("title-1");
        blocked_count.add_css_class("error");
        blocked_row.add_suffix(&blocked_count);

        let blocked_icon = Image::from_icon_name("action-unavailable-symbolic");
        blocked_icon.add_css_class("error");
        blocked_row.add_prefix(&blocked_icon);

        stats_group.add(&blocked_row);

        let allowed_row = ActionRow::builder()
            .title("Conexoes Permitidas")
            .build();

        let allowed_count = Label::new(Some("2,847"));
        allowed_count.add_css_class("title-1");
        allowed_count.add_css_class("success");
        allowed_row.add_suffix(&allowed_count);

        let allowed_icon = Image::from_icon_name("emblem-ok-symbolic");
        allowed_icon.add_css_class("success");
        allowed_row.add_prefix(&allowed_icon);

        stats_group.add(&allowed_row);

        // Top blocked IPs
        let top_blocked = ExpanderRow::builder()
            .title("IPs Mais Bloqueados")
            .subtitle("Origens com mais tentativas rejeitadas")
            .build();

        let blocked_ips = [
            ("185.220.101.45", 45, "Tor Exit Node"),
            ("91.134.202.33", 32, "Scan de portas"),
            ("45.155.205.117", 28, "Botnet conhecido"),
            ("103.75.119.23", 19, "Tentativa SSH"),
            ("194.180.49.116", 15, "HTTP flood"),
        ];

        for (ip, count, reason) in blocked_ips {
            let ip_row = ActionRow::builder()
                .title(ip)
                .subtitle(reason)
                .build();

            let count_label = Label::new(Some(&format!("{} bloqueios", count)));
            count_label.add_css_class("error");
            count_label.add_css_class("dim-label");
            ip_row.add_suffix(&count_label);

            // Block permanently button
            let block_btn = Button::from_icon_name("list-add-symbolic");
            block_btn.add_css_class("flat");
            block_btn.set_tooltip_text(Some("Adicionar a lista de bloqueio permanente"));

            let ip_clone = ip.to_string();
            block_btn.connect_clicked(move |_| {
                tracing::info!("Adding {} to permanent block list", ip_clone);
            });
            ip_row.add_suffix(&block_btn);

            top_blocked.add_row(&ip_row);
        }

        stats_group.add(&top_blocked);

        // Top blocked ports
        let top_ports = ExpanderRow::builder()
            .title("Portas Mais Alvo")
            .subtitle("Portas com mais tentativas de acesso")
            .build();

        let blocked_ports = [
            ("22/tcp", 89, "SSH"),
            ("3389/tcp", 56, "RDP"),
            ("23/tcp", 34, "Telnet"),
            ("445/tcp", 28, "SMB"),
            ("3306/tcp", 21, "MySQL"),
        ];

        for (port, count, service) in blocked_ports {
            let port_row = ActionRow::builder()
                .title(port)
                .subtitle(service)
                .build();

            let count_label = Label::new(Some(&format!("{} tentativas", count)));
            count_label.add_css_class("warning");
            count_label.add_css_class("dim-label");
            port_row.add_suffix(&count_label);

            top_ports.add_row(&port_row);
        }

        stats_group.add(&top_ports);

        page.add(&stats_group);

        // Recent Logs Group
        let recent_group = PreferencesGroup::builder()
            .title("Logs Recentes")
            .description("Ultimas entradas do firewall")
            .build();

        // Search bar
        let search_row = ActionRow::builder()
            .build();

        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Filtrar logs (IP, porta, acao...)"));
        search_entry.set_hexpand(true);
        search_row.set_child(Some(&search_entry));

        recent_group.add(&search_row);

        // Filter options
        let filter_row = ActionRow::builder()
            .build();

        let filter_box = Box::new(Orientation::Horizontal, 8);
        filter_box.set_halign(gtk4::Align::Center);

        let show_blocked = gtk4::CheckButton::with_label("Bloqueados");
        show_blocked.set_active(true);
        filter_box.append(&show_blocked);

        let show_allowed = gtk4::CheckButton::with_label("Permitidos");
        show_allowed.set_active(true);
        filter_box.append(&show_allowed);

        let show_in = gtk4::CheckButton::with_label("Entrada");
        show_in.set_active(true);
        filter_box.append(&show_in);

        let show_out = gtk4::CheckButton::with_label("Saida");
        show_out.set_active(true);
        filter_box.append(&show_out);

        filter_row.set_child(Some(&filter_box));
        recent_group.add(&filter_row);

        // Sample log entries
        let log_entries = [
            ("14:32:15", "BLOCK", "IN", "185.220.101.45", "22/tcp", "SYN"),
            ("14:31:58", "ALLOW", "OUT", "192.168.1.100", "443/tcp", "ESTABLISHED"),
            ("14:31:45", "BLOCK", "IN", "91.134.202.33", "3389/tcp", "SYN"),
            ("14:31:30", "ALLOW", "IN", "192.168.1.1", "53/udp", "DNS query"),
            ("14:31:12", "BLOCK", "IN", "45.155.205.117", "22/tcp", "SYN"),
            ("14:30:55", "ALLOW", "OUT", "192.168.1.100", "80/tcp", "HTTP"),
            ("14:30:42", "BLOCK", "IN", "103.75.119.23", "23/tcp", "Telnet"),
            ("14:30:28", "ALLOW", "OUT", "192.168.1.100", "443/tcp", "HTTPS"),
            ("14:30:15", "BLOCK", "IN", "194.180.49.116", "80/tcp", "HTTP flood"),
            ("14:30:01", "ALLOW", "IN", "192.168.1.50", "22/tcp", "SSH local"),
        ];

        for (time, action, direction, ip, port, info) in log_entries {
            let row = ActionRow::builder()
                .title(&format!("[{}] {} {} {}", time, action, direction, ip))
                .subtitle(&format!("{} - {}", port, info))
                .build();

            let (icon_name, css_class) = if action == "BLOCK" {
                ("action-unavailable-symbolic", "error")
            } else {
                ("emblem-ok-symbolic", "success")
            };

            let icon = Image::from_icon_name(icon_name);
            icon.add_css_class(css_class);
            row.add_prefix(&icon);

            let dir_icon = if direction == "IN" {
                Image::from_icon_name("go-down-symbolic")
            } else {
                Image::from_icon_name("go-up-symbolic")
            };
            row.add_suffix(&dir_icon);

            recent_group.add(&row);
        }

        // Load more button
        let load_more_row = ActionRow::builder()
            .title("Carregar Mais")
            .subtitle("Exibir entradas anteriores")
            .activatable(true)
            .build();

        let more_spinner = Spinner::new();
        load_more_row.add_suffix(&more_spinner);

        let more_icon = Image::from_icon_name("view-more-symbolic");
        load_more_row.add_suffix(&more_icon);

        load_more_row.connect_activated({
            let spinner = more_spinner.clone();
            move |_| {
                spinner.start();
                tracing::info!("Loading more log entries...");
                glib::timeout_add_seconds_local_once(1, {
                    let spinner = spinner.clone();
                    move || spinner.stop()
                });
            }
        });

        recent_group.add(&load_more_row);

        page.add(&recent_group);

        // Export Group
        let export_group = PreferencesGroup::builder()
            .title("Exportar Logs")
            .description("Salvar logs para analise externa")
            .build();

        // Export format
        let export_format = ComboRow::builder()
            .title("Formato")
            .subtitle("Escolha o formato de exportacao")
            .build();

        let format_model = gtk4::StringList::new(&["Texto (TXT)", "CSV", "JSON", "Syslog"]);
        export_format.set_model(Some(&format_model));
        export_format.set_selected(0);

        export_group.add(&export_format);

        // Export button
        let export_row = ActionRow::builder()
            .title("Exportar")
            .subtitle("Salvar logs em arquivo")
            .activatable(true)
            .build();

        let export_icon = Image::from_icon_name("document-save-symbolic");
        export_row.add_prefix(&export_icon);

        export_row.connect_activated(|_| {
            tracing::info!("Exporting firewall logs...");
        });

        export_group.add(&export_row);

        // Clear logs
        let clear_row = ActionRow::builder()
            .title("Limpar Logs")
            .subtitle("Apagar todos os registros de log")
            .activatable(true)
            .build();
        clear_row.add_css_class("error");

        let clear_icon = Image::from_icon_name("user-trash-symbolic");
        clear_row.add_prefix(&clear_icon);

        clear_row.connect_activated(|_| {
            tracing::info!("Clearing firewall logs...");
        });

        export_group.add(&clear_row);

        page.add(&export_group);

        // Real-time monitoring
        let realtime_group = PreferencesGroup::builder()
            .title("Monitoramento em Tempo Real")
            .description("Acompanhe eventos conforme acontecem")
            .build();

        let realtime_switch = SwitchRow::builder()
            .title("Modo ao Vivo")
            .subtitle("Atualizar automaticamente novos eventos")
            .active(false)
            .build();

        realtime_switch.connect_active_notify(|switch| {
            let enabled = switch.is_active();
            tracing::info!("Real-time monitoring: {}", if enabled { "enabled" } else { "disabled" });
        });

        realtime_group.add(&realtime_switch);

        let sound_switch = SwitchRow::builder()
            .title("Alertas Sonoros")
            .subtitle("Tocar som ao bloquear conexao")
            .active(false)
            .build();

        realtime_group.add(&sound_switch);

        let notify_switch = SwitchRow::builder()
            .title("Notificacoes Desktop")
            .subtitle("Mostrar notificacao ao bloquear")
            .active(false)
            .build();

        realtime_group.add(&notify_switch);

        page.add(&realtime_group);

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

impl Default for LogsPage {
    fn default() -> Self {
        Self::new()
    }
}
