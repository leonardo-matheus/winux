//! Pointing/Mouse accessibility settings page

use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::settings::DconfSettings;

/// Pointing (Mouse) settings page
pub struct PointingPage {
    widget: gtk4::ScrolledWindow,
}

impl PointingPage {
    /// Create a new pointing settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Mouse e Ponteiro");
        page.set_icon_name(Some("input-mouse-symbolic"));

        // Mouse Keys Group
        let mouse_keys_group = adw::PreferencesGroup::builder()
            .title("Teclas do Mouse")
            .description("Controle o ponteiro do mouse com o teclado numerico")
            .build();

        // Enable Mouse Keys
        let mouse_keys = adw::SwitchRow::builder()
            .title("Teclas do Mouse")
            .subtitle("Use o teclado numerico para mover o cursor")
            .build();

        mouse_keys.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_mouse_keys(active);
        });

        if let Ok(value) = DconfSettings::get_mouse_keys() {
            mouse_keys.set_active(value);
        }

        mouse_keys_group.add(&mouse_keys);

        // Mouse Keys Speed
        let speed_row = adw::ActionRow::builder()
            .title("Velocidade do Cursor")
            .subtitle("Velocidade de movimento com as teclas")
            .build();

        let speed_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 10.0, 500.0, 10.0);
        speed_scale.set_value(300.0);
        speed_scale.set_draw_value(true);
        speed_scale.set_width_request(200);
        speed_scale.add_mark(100.0, gtk4::PositionType::Bottom, Some("Lento"));
        speed_scale.add_mark(300.0, gtk4::PositionType::Bottom, Some("Normal"));
        speed_scale.add_mark(500.0, gtk4::PositionType::Bottom, Some("Rapido"));

        if let Ok(value) = DconfSettings::get_mouse_keys_max_speed() {
            speed_scale.set_value(value as f64);
        }

        speed_scale.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_mouse_keys_max_speed(value);
        });

        speed_row.add_suffix(&speed_scale);
        mouse_keys_group.add(&speed_row);

        // Mouse Keys Acceleration
        let accel_row = adw::ActionRow::builder()
            .title("Aceleracao")
            .subtitle("Tempo para atingir velocidade maxima")
            .build();

        let accel_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 100.0, 2000.0, 100.0);
        accel_scale.set_value(1000.0);
        accel_scale.set_draw_value(true);
        accel_scale.set_width_request(200);

        if let Ok(value) = DconfSettings::get_mouse_keys_accel_time() {
            accel_scale.set_value(value as f64);
        }

        accel_scale.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_mouse_keys_accel_time(value);
        });

        accel_row.add_suffix(&accel_scale);
        mouse_keys_group.add(&accel_row);

        // Mouse Keys Info
        let info_row = adw::ActionRow::builder()
            .title("Teclas de Controle")
            .subtitle("Num 8=Cima, 2=Baixo, 4=Esq, 6=Dir, 5=Clique")
            .build();

        info_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        mouse_keys_group.add(&info_row);

        page.add(&mouse_keys_group);

        // Click Assist Group
        let click_group = adw::PreferencesGroup::builder()
            .title("Assistencia de Clique")
            .description("Facilita cliques e arrasto")
            .build();

        // Simulated Secondary Click
        let secondary_click = adw::SwitchRow::builder()
            .title("Clique Secundario Simulado")
            .subtitle("Mantenha pressionado para clique com botao direito")
            .build();

        secondary_click.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_secondary_click_enabled(active);
        });

        if let Ok(value) = DconfSettings::get_secondary_click_enabled() {
            secondary_click.set_active(value);
        }

        click_group.add(&secondary_click);

        // Secondary Click Delay
        let secondary_delay_row = adw::ActionRow::builder()
            .title("Tempo para Clique Secundario")
            .subtitle("Tempo para manter pressionado")
            .build();

        let secondary_delay = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.5, 3.0, 0.1);
        secondary_delay.set_value(1.2);
        secondary_delay.set_draw_value(true);
        secondary_delay.set_width_request(200);

        if let Ok(value) = DconfSettings::get_secondary_click_time() {
            secondary_delay.set_value(value);
        }

        secondary_delay.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_secondary_click_time(value);
        });

        let sec_label = gtk4::Label::new(Some("seg"));
        sec_label.add_css_class("dim-label");

        let sec_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        sec_box.append(&secondary_delay);
        sec_box.append(&sec_label);
        sec_box.set_valign(gtk4::Align::Center);

        secondary_delay_row.add_suffix(&sec_box);
        click_group.add(&secondary_delay_row);

        page.add(&click_group);

        // Hover Click Group
        let hover_group = adw::PreferencesGroup::builder()
            .title("Clique por Pausa (Hover Click)")
            .description("Clique automaticamente quando o cursor para")
            .build();

        // Enable Hover Click
        let hover_click = adw::SwitchRow::builder()
            .title("Clique por Pausa")
            .subtitle("Clica automaticamente quando o cursor permanece parado")
            .build();

        hover_click.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_dwell_click_enabled(active);
        });

        if let Ok(value) = DconfSettings::get_dwell_click_enabled() {
            hover_click.set_active(value);
        }

        hover_group.add(&hover_click);

        // Hover Click Delay
        let hover_delay_row = adw::ActionRow::builder()
            .title("Atraso do Clique")
            .subtitle("Tempo parado antes de clicar")
            .build();

        let hover_delay = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.2, 3.0, 0.1);
        hover_delay.set_value(1.2);
        hover_delay.set_draw_value(true);
        hover_delay.set_width_request(200);

        if let Ok(value) = DconfSettings::get_dwell_time() {
            hover_delay.set_value(value);
        }

        hover_delay.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_dwell_time(value);
        });

        let hover_label = gtk4::Label::new(Some("seg"));
        hover_label.add_css_class("dim-label");

        let hover_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        hover_box.append(&hover_delay);
        hover_box.append(&hover_label);
        hover_box.set_valign(gtk4::Align::Center);

        hover_delay_row.add_suffix(&hover_box);
        hover_group.add(&hover_delay_row);

        // Hover Click Motion Threshold
        let threshold_row = adw::ActionRow::builder()
            .title("Limite de Movimento")
            .subtitle("Movimento maximo durante a pausa")
            .build();

        let threshold_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 1.0, 30.0, 1.0);
        threshold_scale.set_value(10.0);
        threshold_scale.set_draw_value(true);
        threshold_scale.set_width_request(200);

        if let Ok(value) = DconfSettings::get_dwell_threshold() {
            threshold_scale.set_value(value as f64);
        }

        threshold_scale.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_dwell_threshold(value);
        });

        let threshold_label = gtk4::Label::new(Some("px"));
        threshold_label.add_css_class("dim-label");

        let threshold_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        threshold_box.append(&threshold_scale);
        threshold_box.append(&threshold_label);
        threshold_box.set_valign(gtk4::Align::Center);

        threshold_row.add_suffix(&threshold_box);
        hover_group.add(&threshold_row);

        // Hover Click Mode
        let hover_mode = adw::ComboRow::builder()
            .title("Modo de Clique")
            .subtitle("Tipo de clique ao pausar")
            .build();

        let modes = gtk4::StringList::new(&[
            "Clique Simples",
            "Clique Duplo",
            "Arrastar",
            "Clique Secundario",
            "Janela de Escolha",
        ]);
        hover_mode.set_model(Some(&modes));

        if let Ok(mode) = DconfSettings::get_dwell_mode() {
            let index = match mode.as_str() {
                "double" => 1,
                "drag" => 2,
                "secondary" => 3,
                "choice" => 4,
                _ => 0,
            };
            hover_mode.set_selected(index);
        }

        hover_mode.connect_selected_notify(|row| {
            let mode = match row.selected() {
                1 => "double",
                2 => "drag",
                3 => "secondary",
                4 => "choice",
                _ => "single",
            };
            DconfSettings::set_dwell_mode(mode);
        });

        hover_group.add(&hover_mode);

        page.add(&hover_group);

        // Double Click Group
        let double_group = adw::PreferencesGroup::builder()
            .title("Clique Duplo")
            .description("Ajuste o tempo limite para clique duplo")
            .build();

        // Double Click Timeout
        let double_timeout_row = adw::ActionRow::builder()
            .title("Tempo Limite do Clique Duplo")
            .subtitle("Intervalo maximo entre cliques")
            .build();

        let double_timeout = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 100.0, 1000.0, 50.0);
        double_timeout.set_value(400.0);
        double_timeout.set_draw_value(true);
        double_timeout.set_width_request(200);
        double_timeout.add_mark(200.0, gtk4::PositionType::Bottom, Some("Rapido"));
        double_timeout.add_mark(400.0, gtk4::PositionType::Bottom, Some("Normal"));
        double_timeout.add_mark(800.0, gtk4::PositionType::Bottom, Some("Lento"));

        if let Ok(value) = DconfSettings::get_double_click_time() {
            double_timeout.set_value(value as f64);
        }

        double_timeout.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_double_click_time(value);
        });

        let double_label = gtk4::Label::new(Some("ms"));
        double_label.add_css_class("dim-label");

        let double_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        double_box.append(&double_timeout);
        double_box.append(&double_label);
        double_box.set_valign(gtk4::Align::Center);

        double_timeout_row.add_suffix(&double_box);
        double_group.add(&double_timeout_row);

        // Test Double Click
        let test_double = adw::ActionRow::builder()
            .title("Testar Clique Duplo")
            .subtitle("Clique duas vezes para testar")
            .activatable(true)
            .build();

        let test_icon = gtk4::Image::from_icon_name("input-mouse-symbolic");
        test_icon.set_pixel_size(32);

        let test_btn = gtk4::Button::new();
        test_btn.set_child(Some(&test_icon));
        test_btn.add_css_class("flat");
        test_btn.set_valign(gtk4::Align::Center);

        let click_count = std::rc::Rc::new(std::cell::RefCell::new(0));
        let last_click = std::rc::Rc::new(std::cell::RefCell::new(std::time::Instant::now()));

        test_btn.connect_clicked({
            let click_count = click_count.clone();
            let last_click = last_click.clone();
            move |btn| {
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(*last_click.borrow());

                if elapsed.as_millis() < 400 {
                    *click_count.borrow_mut() += 1;
                    if *click_count.borrow() >= 2 {
                        btn.set_tooltip_text(Some("Clique duplo detectado!"));
                        *click_count.borrow_mut() = 0;
                    }
                } else {
                    *click_count.borrow_mut() = 1;
                }

                *last_click.borrow_mut() = now;
            }
        });

        test_double.add_suffix(&test_btn);
        double_group.add(&test_double);

        page.add(&double_group);

        // Pointer Speed Group
        let pointer_group = adw::PreferencesGroup::builder()
            .title("Velocidade do Ponteiro")
            .build();

        // Pointer Speed
        let pointer_speed_row = adw::ActionRow::builder()
            .title("Velocidade")
            .subtitle("Velocidade de movimento do cursor")
            .build();

        let pointer_speed = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, -1.0, 1.0, 0.1);
        pointer_speed.set_value(0.0);
        pointer_speed.set_draw_value(false);
        pointer_speed.set_width_request(200);
        pointer_speed.add_mark(-1.0, gtk4::PositionType::Bottom, Some("Lento"));
        pointer_speed.add_mark(0.0, gtk4::PositionType::Bottom, Some("Normal"));
        pointer_speed.add_mark(1.0, gtk4::PositionType::Bottom, Some("Rapido"));

        pointer_speed.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_pointer_speed(value);
        });

        pointer_speed_row.add_suffix(&pointer_speed);
        pointer_group.add(&pointer_speed_row);

        // Natural Scrolling
        let natural_scroll = adw::SwitchRow::builder()
            .title("Rolagem Natural")
            .subtitle("O conteudo segue a direcao do dedo")
            .build();

        natural_scroll.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_natural_scroll(active);
        });

        pointer_group.add(&natural_scroll);

        // Left-handed Mouse
        let left_handed = adw::SwitchRow::builder()
            .title("Mouse para Canhotos")
            .subtitle("Troca os botoes primario e secundario")
            .build();

        left_handed.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_left_handed(active);
        });

        pointer_group.add(&left_handed);

        page.add(&pointer_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    /// Get the page widget
    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}

impl Default for PointingPage {
    fn default() -> Self {
        Self::new()
    }
}
