// Display power settings page for Winux Power

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label, Scale};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::PowerManager;

pub struct DisplayPage {
    container: gtk4::ScrolledWindow,
}

impl DisplayPage {
    pub fn new(manager: Rc<RefCell<PowerManager>>) -> Self {
        let page = adw::PreferencesPage::new();

        // Screen Timeout Group
        let timeout_group = adw::PreferencesGroup::builder()
            .title("Tempo para Desligar Tela")
            .description("Desligue a tela automaticamente apos inatividade")
            .build();

        // On battery timeout
        let battery_timeout_row = adw::ComboRow::builder()
            .title("Na Bateria")
            .subtitle("Tempo de inatividade para desligar")
            .build();
        battery_timeout_row.add_prefix(&gtk4::Image::from_icon_name("battery-symbolic"));
        let battery_times = gtk4::StringList::new(&[
            "30 segundos",
            "1 minuto",
            "2 minutos",
            "5 minutos",
            "10 minutos",
            "15 minutos",
            "Nunca",
        ]);
        battery_timeout_row.set_model(Some(&battery_times));
        battery_timeout_row.set_selected(2);
        timeout_group.add(&battery_timeout_row);

        // On AC timeout
        let ac_timeout_row = adw::ComboRow::builder()
            .title("Na Tomada")
            .subtitle("Tempo de inatividade para desligar")
            .build();
        ac_timeout_row.add_prefix(&gtk4::Image::from_icon_name("ac-adapter-symbolic"));
        let ac_times = gtk4::StringList::new(&[
            "1 minuto",
            "2 minutos",
            "5 minutos",
            "10 minutos",
            "15 minutos",
            "30 minutos",
            "1 hora",
            "Nunca",
        ]);
        ac_timeout_row.set_model(Some(&ac_times));
        ac_timeout_row.set_selected(3);
        timeout_group.add(&ac_timeout_row);

        page.add(&timeout_group);

        // Dim When Idle Group
        let dim_group = adw::PreferencesGroup::builder()
            .title("Escurecer Quando Inativo")
            .description("Reduza o brilho antes de desligar a tela")
            .build();

        // Enable dim
        let dim_switch_row = adw::SwitchRow::builder()
            .title("Escurecer Tela")
            .subtitle("Reduz o brilho antes de desligar")
            .active(true)
            .build();
        dim_group.add(&dim_switch_row);

        // Dim brightness level
        let dim_level_row = adw::ActionRow::builder()
            .title("Nivel de Escurecimento")
            .subtitle("Brilho quando escurecido")
            .build();

        let dim_scale = Scale::with_range(Orientation::Horizontal, 5.0, 50.0, 1.0);
        dim_scale.set_value(20.0);
        dim_scale.set_hexpand(true);
        dim_scale.set_size_request(200, -1);
        dim_scale.set_valign(gtk4::Align::Center);

        let dim_label = Label::new(Some("20%"));
        dim_label.set_width_chars(4);
        dim_scale.connect_value_changed(move |scale| {
            dim_label.set_text(&format!("{}%", scale.value() as i32));
        });

        dim_level_row.add_suffix(&dim_scale);
        dim_group.add(&dim_level_row);

        // Time before dim
        let dim_time_row = adw::ComboRow::builder()
            .title("Tempo para Escurecer")
            .subtitle("Segundos antes de escurecer")
            .build();
        let dim_times = gtk4::StringList::new(&[
            "10 segundos",
            "20 segundos",
            "30 segundos",
            "1 minuto",
        ]);
        dim_time_row.set_model(Some(&dim_times));
        dim_time_row.set_selected(1);
        dim_group.add(&dim_time_row);

        page.add(&dim_group);

        // Brightness on Battery Group
        let brightness_group = adw::PreferencesGroup::builder()
            .title("Brilho na Bateria")
            .description("Ajuste automatico de brilho")
            .build();

        // Reduce brightness on battery
        let reduce_brightness_row = adw::SwitchRow::builder()
            .title("Reduzir Brilho na Bateria")
            .subtitle("Diminui o brilho quando na bateria")
            .active(true)
            .build();
        brightness_group.add(&reduce_brightness_row);

        // Battery brightness level
        let battery_brightness_row = adw::ActionRow::builder()
            .title("Brilho na Bateria")
            .subtitle("Nivel de brilho quando na bateria")
            .build();

        let brightness_scale = Scale::with_range(Orientation::Horizontal, 10.0, 100.0, 1.0);
        brightness_scale.set_value(60.0);
        brightness_scale.set_hexpand(true);
        brightness_scale.set_size_request(200, -1);
        brightness_scale.set_valign(gtk4::Align::Center);

        let brightness_label = Label::new(Some("60%"));
        brightness_label.set_width_chars(4);
        brightness_scale.connect_value_changed(move |scale| {
            brightness_label.set_text(&format!("{}%", scale.value() as i32));
        });

        battery_brightness_row.add_suffix(&brightness_scale);
        brightness_group.add(&battery_brightness_row);

        // Adaptive brightness
        let adaptive_row = adw::SwitchRow::builder()
            .title("Brilho Adaptativo")
            .subtitle("Ajusta baseado na luz ambiente")
            .active(true)
            .build();
        brightness_group.add(&adaptive_row);

        page.add(&brightness_group);

        // Power Button Actions Group
        let actions_group = adw::PreferencesGroup::builder()
            .title("Acoes de Energia")
            .description("Configure o comportamento dos botoes")
            .build();

        // Power button action
        let power_btn_row = adw::ComboRow::builder()
            .title("Botao de Energia")
            .subtitle("Acao ao pressionar o botao de energia")
            .build();
        power_btn_row.add_prefix(&gtk4::Image::from_icon_name("system-shutdown-symbolic"));
        let power_actions = gtk4::StringList::new(&[
            "Suspender",
            "Hibernar",
            "Desligar",
            "Perguntar",
            "Nada",
        ]);
        power_btn_row.set_model(Some(&power_actions));
        power_btn_row.set_selected(0);
        actions_group.add(&power_btn_row);

        // Lid close action (on battery)
        let lid_battery_row = adw::ComboRow::builder()
            .title("Fechar Tampa (Bateria)")
            .subtitle("Acao ao fechar a tampa na bateria")
            .build();
        lid_battery_row.add_prefix(&gtk4::Image::from_icon_name("computer-laptop-symbolic"));
        let lid_actions = gtk4::StringList::new(&[
            "Suspender",
            "Hibernar",
            "Desligar Tela",
            "Nada",
        ]);
        lid_battery_row.set_model(Some(&lid_actions));
        lid_battery_row.set_selected(0);
        actions_group.add(&lid_battery_row);

        // Lid close action (on AC)
        let lid_ac_row = adw::ComboRow::builder()
            .title("Fechar Tampa (Tomada)")
            .subtitle("Acao ao fechar a tampa na tomada")
            .build();
        let lid_ac_actions = gtk4::StringList::new(&[
            "Suspender",
            "Hibernar",
            "Desligar Tela",
            "Nada",
        ]);
        lid_ac_row.set_model(Some(&lid_ac_actions));
        lid_ac_row.set_selected(2);
        actions_group.add(&lid_ac_row);

        page.add(&actions_group);

        // Critical Battery Action Group
        let critical_group = adw::PreferencesGroup::builder()
            .title("Bateria Critica")
            .description("Acoes quando a bateria esta muito baixa")
            .build();

        // Critical level threshold
        let critical_level_row = adw::SpinRow::builder()
            .title("Nivel Critico")
            .subtitle("Porcentagem considerada critica")
            .adjustment(&gtk4::Adjustment::new(5.0, 3.0, 20.0, 1.0, 1.0, 0.0))
            .build();
        critical_group.add(&critical_level_row);

        // Low battery warning
        let low_warning_row = adw::SpinRow::builder()
            .title("Aviso de Bateria Baixa")
            .subtitle("Mostrar notificacao abaixo de")
            .adjustment(&gtk4::Adjustment::new(20.0, 10.0, 40.0, 1.0, 5.0, 0.0))
            .build();
        critical_group.add(&low_warning_row);

        // Critical action
        let critical_action_row = adw::ComboRow::builder()
            .title("Acao Critica")
            .subtitle("Acao quando bateria esta critica")
            .build();
        critical_action_row.add_prefix(&gtk4::Image::from_icon_name("battery-level-0-symbolic"));
        let critical_actions = gtk4::StringList::new(&[
            "Hibernar",
            "Desligar",
            "Suspender",
        ]);
        critical_action_row.set_model(Some(&critical_actions));
        critical_action_row.set_selected(0);
        critical_group.add(&critical_action_row);

        page.add(&critical_group);

        // Suspend Settings Group
        let suspend_group = adw::PreferencesGroup::builder()
            .title("Suspensao e Hibernacao")
            .build();

        // Auto suspend on battery
        let auto_suspend_battery_row = adw::ComboRow::builder()
            .title("Suspensao Automatica (Bateria)")
            .subtitle("Suspender apos inatividade na bateria")
            .build();
        let suspend_times = gtk4::StringList::new(&[
            "5 minutos",
            "10 minutos",
            "15 minutos",
            "30 minutos",
            "1 hora",
            "Nunca",
        ]);
        auto_suspend_battery_row.set_model(Some(&suspend_times));
        auto_suspend_battery_row.set_selected(2);
        suspend_group.add(&auto_suspend_battery_row);

        // Auto suspend on AC
        let auto_suspend_ac_row = adw::ComboRow::builder()
            .title("Suspensao Automatica (Tomada)")
            .subtitle("Suspender apos inatividade na tomada")
            .build();
        let ac_suspend_times = gtk4::StringList::new(&[
            "15 minutos",
            "30 minutos",
            "1 hora",
            "2 horas",
            "Nunca",
        ]);
        auto_suspend_ac_row.set_model(Some(&ac_suspend_times));
        auto_suspend_ac_row.set_selected(4);
        suspend_group.add(&auto_suspend_ac_row);

        // Hybrid sleep
        let hybrid_row = adw::SwitchRow::builder()
            .title("Suspensao Hibrida")
            .subtitle("Combina suspensao com hibernacao para seguranca")
            .active(false)
            .build();
        suspend_group.add(&hybrid_row);

        page.add(&suspend_group);

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
