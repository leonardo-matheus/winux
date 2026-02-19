// Battery status page for Winux Power

use gdk4;
use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::PowerManager;
use crate::ui::{BatteryGauge, HistoryGraph};

pub struct BatteryPage {
    container: gtk4::ScrolledWindow,
    gauge: Rc<RefCell<BatteryGauge>>,
    history_graph: Rc<RefCell<HistoryGraph>>,
}

impl BatteryPage {
    pub fn new(manager: Rc<RefCell<PowerManager>>) -> Self {
        let page = adw::PreferencesPage::new();

        // Battery Status Group (with circular gauge)
        let status_group = adw::PreferencesGroup::builder()
            .title("Status da Bateria")
            .description("Informacoes em tempo real")
            .build();

        // Custom battery gauge widget
        let gauge = BatteryGauge::new();
        let gauge_rc = Rc::new(RefCell::new(gauge));

        let gauge_box = Box::new(Orientation::Vertical, 12);
        gauge_box.set_halign(gtk4::Align::Center);
        gauge_box.set_margin_top(24);
        gauge_box.set_margin_bottom(24);
        gauge_box.append(gauge_rc.borrow().widget());

        // Time remaining label
        let time_label = Label::new(Some("Tempo restante: 4h 32min"));
        time_label.add_css_class("title-3");
        gauge_box.append(&time_label);

        // Charging status label
        let charging_label = Label::new(Some("Carregando - 65W USB-C"));
        charging_label.add_css_class("dim-label");
        gauge_box.append(&charging_label);

        status_group.add(&gauge_box);
        page.add(&status_group);

        // Battery Info Group
        let info_group = adw::PreferencesGroup::builder()
            .title("Informacoes da Bateria")
            .build();

        // Percentage row
        let percentage_row = adw::ActionRow::builder()
            .title("Nivel Atual")
            .build();
        percentage_row.add_prefix(&gtk4::Image::from_icon_name("battery-level-80-charging-symbolic"));
        let percentage_label = Label::new(Some("78%"));
        percentage_label.add_css_class("title-4");
        percentage_row.add_suffix(&percentage_label);
        info_group.add(&percentage_row);

        // Status row
        let status_row = adw::ActionRow::builder()
            .title("Status")
            .subtitle("Estado atual da bateria")
            .build();
        let status_badge = Label::new(Some("Carregando"));
        status_badge.add_css_class("success");
        status_row.add_suffix(&status_badge);
        info_group.add(&status_row);

        // Energy rate row
        let energy_row = adw::ActionRow::builder()
            .title("Taxa de Energia")
            .subtitle("Consumo ou carga atual")
            .build();
        let energy_label = Label::new(Some("+45.2 W"));
        energy_label.add_css_class("accent");
        energy_row.add_suffix(&energy_label);
        info_group.add(&energy_row);

        // Voltage row
        let voltage_row = adw::ActionRow::builder()
            .title("Tensao")
            .subtitle("Voltagem atual da bateria")
            .build();
        let voltage_label = Label::new(Some("12.4 V"));
        voltage_row.add_suffix(&voltage_label);
        info_group.add(&voltage_row);

        page.add(&info_group);

        // Battery Health Group
        let health_group = adw::PreferencesGroup::builder()
            .title("Saude da Bateria")
            .description("Informacoes de capacidade e desgaste")
            .build();

        // Health percentage
        let health_row = adw::ActionRow::builder()
            .title("Saude")
            .subtitle("Capacidade atual vs original")
            .build();
        let health_bar = gtk4::ProgressBar::new();
        health_bar.set_fraction(0.92);
        health_bar.set_text(Some("92%"));
        health_bar.set_show_text(true);
        health_bar.set_valign(gtk4::Align::Center);
        health_bar.set_size_request(150, -1);
        health_row.add_suffix(&health_bar);
        health_group.add(&health_row);

        // Design capacity
        let design_row = adw::ActionRow::builder()
            .title("Capacidade de Design")
            .subtitle("Capacidade original da bateria")
            .build();
        let design_label = Label::new(Some("72.0 Wh"));
        design_row.add_suffix(&design_label);
        health_group.add(&design_row);

        // Current capacity
        let current_row = adw::ActionRow::builder()
            .title("Capacidade Atual")
            .subtitle("Capacidade maxima atual")
            .build();
        let current_label = Label::new(Some("66.2 Wh"));
        current_row.add_suffix(&current_label);
        health_group.add(&current_row);

        // Charge cycles
        let cycles_row = adw::ActionRow::builder()
            .title("Ciclos de Carga")
            .subtitle("Numero de ciclos completos")
            .build();
        let cycles_label = Label::new(Some("287"));
        cycles_row.add_suffix(&cycles_label);
        health_group.add(&cycles_row);

        // Battery age
        let age_row = adw::ActionRow::builder()
            .title("Idade da Bateria")
            .subtitle("Tempo desde a fabricacao")
            .build();
        let age_label = Label::new(Some("1 ano 8 meses"));
        age_row.add_suffix(&age_label);
        health_group.add(&age_row);

        page.add(&health_group);

        // History Graph Group
        let history_group = adw::PreferencesGroup::builder()
            .title("Historico de Bateria")
            .description("Grafico das ultimas 24 horas")
            .build();

        let history_graph = HistoryGraph::new();
        let history_rc = Rc::new(RefCell::new(history_graph));

        let graph_box = Box::new(Orientation::Vertical, 0);
        graph_box.set_margin_top(12);
        graph_box.set_margin_bottom(12);
        graph_box.append(history_rc.borrow().widget());

        // Legend
        let legend_box = Box::new(Orientation::Horizontal, 24);
        legend_box.set_halign(gtk4::Align::Center);
        legend_box.set_margin_top(12);

        let charge_legend = create_legend_item("Carregando", "#57e389");
        let discharge_legend = create_legend_item("Descarregando", "#f66151");
        legend_box.append(&charge_legend);
        legend_box.append(&discharge_legend);

        graph_box.append(&legend_box);
        history_group.add(&graph_box);

        page.add(&history_group);

        // Battery Details Group
        let details_group = adw::PreferencesGroup::builder()
            .title("Detalhes Tecnicos")
            .build();

        // Technology
        let tech_row = adw::ActionRow::builder()
            .title("Tecnologia")
            .build();
        let tech_label = Label::new(Some("Li-ion"));
        tech_row.add_suffix(&tech_label);
        details_group.add(&tech_row);

        // Model
        let model_row = adw::ActionRow::builder()
            .title("Modelo")
            .build();
        let model_label = Label::new(Some("DELL 4GVMP"));
        model_row.add_suffix(&model_label);
        details_group.add(&model_row);

        // Serial
        let serial_row = adw::ActionRow::builder()
            .title("Serial")
            .build();
        let serial_label = Label::new(Some("1234-5678-ABCD"));
        serial_row.add_suffix(&serial_label);
        details_group.add(&serial_row);

        // Manufacturer
        let manufacturer_row = adw::ActionRow::builder()
            .title("Fabricante")
            .build();
        let manufacturer_label = Label::new(Some("Samsung SDI"));
        manufacturer_row.add_suffix(&manufacturer_label);
        details_group.add(&manufacturer_row);

        page.add(&details_group);

        // Update battery info periodically
        let manager_clone = manager.clone();
        let gauge_clone = gauge_rc.clone();
        let percentage_label_clone = percentage_label.clone();
        let time_label_clone = time_label.clone();
        let charging_label_clone = charging_label.clone();
        let status_badge_clone = status_badge.clone();
        let energy_label_clone = energy_label.clone();

        glib::timeout_add_seconds_local(5, move || {
            let mgr = manager_clone.borrow();
            let percentage = mgr.get_battery_percentage();
            let charging = mgr.is_charging();
            let time_remaining = mgr.get_time_remaining();
            let energy_rate = mgr.get_energy_rate();

            // Update gauge
            gauge_clone.borrow().set_percentage(percentage);

            // Update labels
            percentage_label_clone.set_text(&format!("{}%", percentage));

            if charging {
                time_label_clone.set_text(&format!("Tempo para carga completa: {}", format_time(time_remaining)));
                charging_label_clone.set_text("Carregando");
                status_badge_clone.set_text("Carregando");
                status_badge_clone.remove_css_class("warning");
                status_badge_clone.add_css_class("success");
                energy_label_clone.set_text(&format!("+{:.1} W", energy_rate));
            } else {
                time_label_clone.set_text(&format!("Tempo restante: {}", format_time(time_remaining)));
                charging_label_clone.set_text("Descarregando");
                status_badge_clone.set_text("Na bateria");
                status_badge_clone.remove_css_class("success");
                status_badge_clone.add_css_class("warning");
                energy_label_clone.set_text(&format!("-{:.1} W", energy_rate));
            }

            glib::ControlFlow::Continue
        });

        let container = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            container,
            gauge: gauge_rc,
            history_graph: history_rc,
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.container
    }
}

fn create_legend_item(text: &str, color: &str) -> Box {
    let item = Box::new(Orientation::Horizontal, 6);

    let color_box = gtk4::DrawingArea::new();
    color_box.set_size_request(16, 16);
    let color_str = color.to_string();
    color_box.set_draw_func(move |_, cr, _, _| {
        if let Ok(rgba) = color_str.parse::<gdk4::RGBA>() {
            cr.set_source_rgba(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                1.0,
            );
        } else {
            cr.set_source_rgb(0.5, 0.5, 0.5);
        }
        let _ = cr.paint();
    });

    let label = Label::new(Some(text));
    label.add_css_class("dim-label");

    item.append(&color_box);
    item.append(&label);
    item
}

fn format_time(minutes: u32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        format!("{}h {}min", hours, mins)
    } else {
        format!("{}min", mins)
    }
}
