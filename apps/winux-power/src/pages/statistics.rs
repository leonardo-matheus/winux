// Statistics and usage history page for Winux Power

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::PowerManager;
use crate::ui::HistoryGraph;

pub struct StatisticsPage {
    container: gtk4::ScrolledWindow,
}

impl StatisticsPage {
    pub fn new(manager: Rc<RefCell<PowerManager>>) -> Self {
        let page = adw::PreferencesPage::new();

        // Summary Group
        let summary_group = adw::PreferencesGroup::builder()
            .title("Resumo de Hoje")
            .description("Estatisticas do dia atual")
            .build();

        // Summary cards
        let cards_box = Box::new(Orientation::Horizontal, 12);
        cards_box.set_homogeneous(true);
        cards_box.set_margin_top(12);
        cards_box.set_margin_bottom(12);

        let card_data = [
            ("Tempo na Bateria", "3h 45min", "battery-symbolic"),
            ("Tempo na Tomada", "5h 12min", "ac-adapter-symbolic"),
            ("Ciclos Hoje", "0.4", "view-refresh-symbolic"),
            ("Consumo Medio", "12.5 W", "utilities-system-monitor-symbolic"),
        ];

        for (title, value, icon) in card_data {
            let card = create_stat_card(title, value, icon);
            cards_box.append(&card);
        }

        summary_group.add(&cards_box);
        page.add(&summary_group);

        // Battery History Graph
        let history_group = adw::PreferencesGroup::builder()
            .title("Historico de Bateria")
            .description("Nivel de bateria nas ultimas 24 horas")
            .build();

        let graph = HistoryGraph::new();
        let graph_box = Box::new(Orientation::Vertical, 0);
        graph_box.set_margin_top(12);
        graph_box.set_margin_bottom(12);
        graph_box.append(graph.widget());

        // Time range selector
        let range_box = Box::new(Orientation::Horizontal, 6);
        range_box.set_halign(gtk4::Align::Center);
        range_box.set_margin_top(12);

        let ranges = ["6h", "12h", "24h", "7d", "30d"];
        for (i, range) in ranges.iter().enumerate() {
            let btn = gtk4::ToggleButton::with_label(range);
            if i == 2 {
                btn.set_active(true);
            }
            btn.add_css_class("flat");
            range_box.append(&btn);
        }

        graph_box.append(&range_box);
        history_group.add(&graph_box);
        page.add(&history_group);

        // Power Profile Usage Group
        let profile_group = adw::PreferencesGroup::builder()
            .title("Uso de Perfis")
            .description("Tempo em cada perfil de energia hoje")
            .build();

        let profiles = [
            ("Alto Desempenho", "45min", 0.15, "power-profile-performance-symbolic"),
            ("Balanceado", "6h 30min", 0.72, "power-profile-balanced-symbolic"),
            ("Economia", "1h 42min", 0.13, "power-profile-power-saver-symbolic"),
        ];

        for (name, time, fraction, icon) in profiles {
            let row = adw::ActionRow::builder()
                .title(name)
                .subtitle(time)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let progress = gtk4::ProgressBar::new();
            progress.set_fraction(fraction);
            progress.set_valign(gtk4::Align::Center);
            progress.set_size_request(100, -1);
            row.add_suffix(&progress);

            profile_group.add(&row);
        }

        page.add(&profile_group);

        // Top Power Consuming Apps
        let apps_group = adw::PreferencesGroup::builder()
            .title("Consumo por Aplicativo")
            .description("Aplicativos que mais consomem energia")
            .build();

        let apps = [
            ("Firefox", "Alto", "2h 15min ativos", "firefox-symbolic", 0.35),
            ("VS Code", "Alto", "4h 30min ativos", "code-symbolic", 0.30),
            ("Spotify", "Medio", "1h 45min ativos", "spotify-symbolic", 0.15),
            ("Terminal", "Baixo", "3h 20min ativos", "utilities-terminal-symbolic", 0.10),
            ("Files", "Baixo", "45min ativos", "system-file-manager-symbolic", 0.05),
        ];

        for (name, usage, time, icon, fraction) in apps {
            let row = adw::ExpanderRow::builder()
                .title(name)
                .subtitle(&format!("Uso: {}", usage))
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let progress = gtk4::ProgressBar::new();
            progress.set_fraction(fraction);
            progress.set_valign(gtk4::Align::Center);
            progress.set_size_request(80, -1);
            row.add_suffix(&progress);

            // Details
            let time_row = adw::ActionRow::builder()
                .title("Tempo Ativo")
                .subtitle(time)
                .build();
            row.add_row(&time_row);

            let wakeups_row = adw::ActionRow::builder()
                .title("Wakeups/s")
                .subtitle("Interrupcoes por segundo")
                .build();
            let wakeups_label = Label::new(Some("12.5"));
            wakeups_row.add_suffix(&wakeups_label);
            row.add_row(&wakeups_row);

            apps_group.add(&row);
        }

        page.add(&apps_group);

        // Hardware Power Stats
        let hardware_group = adw::PreferencesGroup::builder()
            .title("Consumo de Hardware")
            .description("Estimativa de consumo por componente")
            .build();

        let components = [
            ("CPU", "8.5 W", 0.45, "cpu-symbolic"),
            ("GPU", "3.2 W", 0.17, "video-display-symbolic"),
            ("Display", "4.8 W", 0.25, "preferences-desktop-display-symbolic"),
            ("Storage", "1.2 W", 0.06, "drive-harddisk-symbolic"),
            ("Wireless", "0.8 W", 0.04, "network-wireless-symbolic"),
            ("Outros", "0.5 W", 0.03, "applications-system-symbolic"),
        ];

        for (name, power, fraction, icon) in components {
            let row = adw::ActionRow::builder()
                .title(name)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            let power_label = Label::new(Some(power));
            power_label.add_css_class("dim-label");
            row.add_suffix(&power_label);

            let progress = gtk4::ProgressBar::new();
            progress.set_fraction(fraction);
            progress.set_valign(gtk4::Align::Center);
            progress.set_size_request(80, -1);
            row.add_suffix(&progress);

            hardware_group.add(&row);
        }

        // Total power row
        let total_row = adw::ActionRow::builder()
            .title("Total Estimado")
            .build();
        let total_label = Label::new(Some("19.0 W"));
        total_label.add_css_class("title-4");
        total_row.add_suffix(&total_label);
        hardware_group.add(&total_row);

        page.add(&hardware_group);

        // Weekly Summary Group
        let weekly_group = adw::PreferencesGroup::builder()
            .title("Resumo Semanal")
            .build();

        let weekly_stats = [
            ("Ciclos de Carga", "2.8 ciclos"),
            ("Tempo Total na Bateria", "18h 32min"),
            ("Tempo Total na Tomada", "42h 15min"),
            ("Consumo Medio", "14.2 W"),
            ("Eficiencia", "92%"),
        ];

        for (label, value) in weekly_stats {
            let row = adw::ActionRow::builder()
                .title(label)
                .build();
            let value_label = Label::new(Some(value));
            row.add_suffix(&value_label);
            weekly_group.add(&row);
        }

        page.add(&weekly_group);

        // Battery Capacity Over Time
        let capacity_group = adw::PreferencesGroup::builder()
            .title("Capacidade ao Longo do Tempo")
            .description("Historico de saude da bateria")
            .build();

        let capacity_history = [
            ("Primeira Medicao (Jan 2025)", "100%", "72.0 Wh"),
            ("Ha 6 meses", "96%", "69.1 Wh"),
            ("Ha 3 meses", "94%", "67.7 Wh"),
            ("Atual", "92%", "66.2 Wh"),
        ];

        for (date, percent, capacity) in capacity_history {
            let row = adw::ActionRow::builder()
                .title(date)
                .subtitle(capacity)
                .build();
            let percent_label = Label::new(Some(percent));
            row.add_suffix(&percent_label);
            capacity_group.add(&row);
        }

        // Estimated lifespan
        let lifespan_row = adw::ActionRow::builder()
            .title("Vida Util Estimada")
            .subtitle("Baseado no padrao de uso atual")
            .build();
        let lifespan_label = Label::new(Some("~3 anos"));
        lifespan_label.add_css_class("accent");
        lifespan_row.add_suffix(&lifespan_label);
        capacity_group.add(&lifespan_row);

        page.add(&capacity_group);

        // Export Options
        let export_group = adw::PreferencesGroup::builder()
            .title("Exportar Dados")
            .build();

        let export_row = adw::ActionRow::builder()
            .title("Exportar Historico")
            .subtitle("Salvar dados em CSV ou JSON")
            .activatable(true)
            .build();
        export_row.add_suffix(&gtk4::Image::from_icon_name("document-save-symbolic"));
        export_group.add(&export_row);

        let clear_row = adw::ActionRow::builder()
            .title("Limpar Historico")
            .subtitle("Apagar todos os dados coletados")
            .activatable(true)
            .build();
        clear_row.add_suffix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        export_group.add(&clear_row);

        page.add(&export_group);

        let container = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { container }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.container
    }
}

fn create_stat_card(title: &str, value: &str, icon: &str) -> gtk4::Frame {
    let frame = gtk4::Frame::new(None);
    frame.add_css_class("card");

    let content = Box::new(Orientation::Vertical, 8);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_halign(gtk4::Align::Center);

    let icon_widget = gtk4::Image::from_icon_name(icon);
    icon_widget.set_pixel_size(32);
    icon_widget.add_css_class("dim-label");

    let value_label = Label::new(Some(value));
    value_label.add_css_class("title-2");

    let title_label = Label::new(Some(title));
    title_label.add_css_class("dim-label");
    title_label.add_css_class("caption");

    content.append(&icon_widget);
    content.append(&value_label);
    content.append(&title_label);

    frame.set_child(Some(&content));
    frame
}
