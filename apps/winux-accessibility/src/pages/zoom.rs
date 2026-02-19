//! Zoom/Magnifier accessibility settings page

use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::settings::DconfSettings;

/// Zoom settings page
pub struct ZoomPage {
    widget: gtk4::ScrolledWindow,
}

impl ZoomPage {
    /// Create a new zoom settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Zoom");
        page.set_icon_name(Some("find-location-symbolic"));

        // Magnifier Group
        let magnifier_group = adw::PreferencesGroup::builder()
            .title("Lupa (Magnifier)")
            .description("Amplia uma regiao da tela")
            .build();

        // Enable Magnifier
        let magnifier = adw::SwitchRow::builder()
            .title("Lupa")
            .subtitle("Ativar ampliacao da tela")
            .build();

        magnifier.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_magnifier_enabled(active);
        });

        if let Ok(value) = DconfSettings::get_magnifier_enabled() {
            magnifier.set_active(value);
        }

        magnifier_group.add(&magnifier);

        // Zoom Level
        let zoom_level_row = adw::ActionRow::builder()
            .title("Nivel de Zoom")
            .subtitle("Fator de ampliacao")
            .build();

        let zoom_level = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 1.0, 20.0, 0.5);
        zoom_level.set_value(2.0);
        zoom_level.set_draw_value(true);
        zoom_level.set_width_request(200);
        zoom_level.add_mark(1.0, gtk4::PositionType::Bottom, Some("1x"));
        zoom_level.add_mark(2.0, gtk4::PositionType::Bottom, Some("2x"));
        zoom_level.add_mark(5.0, gtk4::PositionType::Bottom, Some("5x"));
        zoom_level.add_mark(10.0, gtk4::PositionType::Bottom, Some("10x"));

        if let Ok(value) = DconfSettings::get_magnifier_factor() {
            zoom_level.set_value(value);
        }

        zoom_level.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_magnifier_factor(value);
        });

        zoom_level_row.add_suffix(&zoom_level);
        magnifier_group.add(&zoom_level_row);

        // Quick Zoom Buttons
        let quick_zoom_row = adw::ActionRow::builder()
            .title("Zoom Rapido")
            .build();

        let zoom_btns = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        zoom_btns.set_valign(gtk4::Align::Center);

        for (label, factor) in [("2x", 2.0), ("4x", 4.0), ("8x", 8.0)] {
            let btn = gtk4::Button::with_label(label);
            btn.add_css_class("flat");

            let factor_val = factor;
            btn.connect_clicked(move |_| {
                DconfSettings::set_magnifier_factor(factor_val);
            });

            zoom_btns.append(&btn);
        }

        quick_zoom_row.add_suffix(&zoom_btns);
        magnifier_group.add(&quick_zoom_row);

        page.add(&magnifier_group);

        // Magnifier View Group
        let view_group = adw::PreferencesGroup::builder()
            .title("Modo de Visualizacao")
            .description("Como a lupa e exibida")
            .build();

        // Magnifier Mode
        let magnifier_mode = adw::ComboRow::builder()
            .title("Modo da Lupa")
            .build();

        let modes = gtk4::StringList::new(&[
            "Tela Inteira",
            "Lente (Janela)",
            "Tela Dividida Superior",
            "Tela Dividida Inferior",
            "Tela Dividida Esquerda",
            "Tela Dividida Direita",
        ]);
        magnifier_mode.set_model(Some(&modes));

        if let Ok(mode) = DconfSettings::get_magnifier_screen_position() {
            let index = match mode.as_str() {
                "full-screen" => 0,
                "lens" => 1,
                "top-half" => 2,
                "bottom-half" => 3,
                "left-half" => 4,
                "right-half" => 5,
                _ => 0,
            };
            magnifier_mode.set_selected(index);
        }

        magnifier_mode.connect_selected_notify(|row| {
            let mode = match row.selected() {
                0 => "full-screen",
                1 => "lens",
                2 => "top-half",
                3 => "bottom-half",
                4 => "left-half",
                5 => "right-half",
                _ => "full-screen",
            };
            DconfSettings::set_magnifier_screen_position(mode);
        });

        view_group.add(&magnifier_mode);

        // Lens Mode Settings
        let lens_size_row = adw::ActionRow::builder()
            .title("Tamanho da Lente")
            .subtitle("Diametro da lente em modo janela")
            .build();

        let lens_size = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 100.0, 600.0, 50.0);
        lens_size.set_value(300.0);
        lens_size.set_draw_value(true);
        lens_size.set_width_request(200);

        if let Ok(value) = DconfSettings::get_magnifier_lens_size() {
            lens_size.set_value(value as f64);
        }

        lens_size.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_magnifier_lens_size(value);
        });

        let lens_label = gtk4::Label::new(Some("px"));
        lens_label.add_css_class("dim-label");

        let lens_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        lens_box.append(&lens_size);
        lens_box.append(&lens_label);
        lens_box.set_valign(gtk4::Align::Center);

        lens_size_row.add_suffix(&lens_box);
        view_group.add(&lens_size_row);

        // Lens Shape
        let lens_shape = adw::ComboRow::builder()
            .title("Forma da Lente")
            .build();

        let shapes = gtk4::StringList::new(&[
            "Circular",
            "Quadrada",
            "Horizontal",
            "Vertical",
        ]);
        lens_shape.set_model(Some(&shapes));

        if let Ok(shape) = DconfSettings::get_magnifier_lens_shape() {
            let index = match shape.as_str() {
                "square" => 1,
                "horizontal" => 2,
                "vertical" => 3,
                _ => 0,
            };
            lens_shape.set_selected(index);
        }

        lens_shape.connect_selected_notify(|row| {
            let shape = match row.selected() {
                1 => "square",
                2 => "horizontal",
                3 => "vertical",
                _ => "round",
            };
            DconfSettings::set_magnifier_lens_shape(shape);
        });

        view_group.add(&lens_shape);

        page.add(&view_group);

        // Follow Behavior Group
        let follow_group = adw::PreferencesGroup::builder()
            .title("Comportamento de Seguir")
            .description("O que a lupa deve seguir")
            .build();

        // Follow Cursor
        let follow_cursor = adw::SwitchRow::builder()
            .title("Seguir Cursor")
            .subtitle("A lupa segue o cursor do mouse")
            .active(true)
            .build();

        follow_cursor.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_magnifier_mouse_tracking(if active { "centered" } else { "none" });
        });

        if let Ok(value) = DconfSettings::get_magnifier_mouse_tracking() {
            follow_cursor.set_active(value != "none");
        }

        follow_group.add(&follow_cursor);

        // Cursor Tracking Mode
        let cursor_tracking = adw::ComboRow::builder()
            .title("Modo de Rastreamento do Cursor")
            .build();

        let tracking_modes = gtk4::StringList::new(&[
            "Centralizado",
            "Proporcional",
            "Empurrar",
            "Nenhum",
        ]);
        cursor_tracking.set_model(Some(&tracking_modes));

        if let Ok(mode) = DconfSettings::get_magnifier_mouse_tracking() {
            let index = match mode.as_str() {
                "centered" => 0,
                "proportional" => 1,
                "push" => 2,
                "none" => 3,
                _ => 0,
            };
            cursor_tracking.set_selected(index);
        }

        cursor_tracking.connect_selected_notify(|row| {
            let mode = match row.selected() {
                0 => "centered",
                1 => "proportional",
                2 => "push",
                3 => "none",
                _ => "centered",
            };
            DconfSettings::set_magnifier_mouse_tracking(mode);
        });

        follow_group.add(&cursor_tracking);

        // Follow Focus
        let follow_focus = adw::SwitchRow::builder()
            .title("Seguir Foco")
            .subtitle("A lupa segue o foco do teclado")
            .active(true)
            .build();

        follow_focus.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_magnifier_focus_tracking(if active { "centered" } else { "none" });
        });

        if let Ok(value) = DconfSettings::get_magnifier_focus_tracking() {
            follow_focus.set_active(value != "none");
        }

        follow_group.add(&follow_focus);

        // Follow Caret
        let follow_caret = adw::SwitchRow::builder()
            .title("Seguir Cursor de Texto")
            .subtitle("A lupa segue o cursor de digitacao")
            .active(true)
            .build();

        follow_caret.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_magnifier_caret_tracking(if active { "centered" } else { "none" });
        });

        if let Ok(value) = DconfSettings::get_magnifier_caret_tracking() {
            follow_caret.set_active(value != "none");
        }

        follow_group.add(&follow_caret);

        page.add(&follow_group);

        // Visual Effects Group
        let effects_group = adw::PreferencesGroup::builder()
            .title("Efeitos Visuais")
            .description("Aparencia da lupa")
            .build();

        // Crosshairs
        let crosshairs = adw::SwitchRow::builder()
            .title("Mostrar Mira")
            .subtitle("Exibe linhas cruzadas no centro da lupa")
            .build();

        crosshairs.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_magnifier_cross_hairs(active);
        });

        if let Ok(value) = DconfSettings::get_magnifier_cross_hairs() {
            crosshairs.set_active(value);
        }

        effects_group.add(&crosshairs);

        // Crosshairs Length
        let cross_length_row = adw::ActionRow::builder()
            .title("Comprimento da Mira")
            .build();

        let cross_length = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 50.0, 1000.0, 50.0);
        cross_length.set_value(400.0);
        cross_length.set_draw_value(true);
        cross_length.set_width_request(200);

        if let Ok(value) = DconfSettings::get_magnifier_cross_hairs_length() {
            cross_length.set_value(value as f64);
        }

        cross_length.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_magnifier_cross_hairs_length(value);
        });

        cross_length_row.add_suffix(&cross_length);
        effects_group.add(&cross_length_row);

        // Crosshairs Thickness
        let cross_thick_row = adw::ActionRow::builder()
            .title("Espessura da Mira")
            .build();

        let cross_thick = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 1.0, 20.0, 1.0);
        cross_thick.set_value(4.0);
        cross_thick.set_draw_value(true);
        cross_thick.set_width_request(200);

        if let Ok(value) = DconfSettings::get_magnifier_cross_hairs_thickness() {
            cross_thick.set_value(value as f64);
        }

        cross_thick.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_magnifier_cross_hairs_thickness(value);
        });

        cross_thick_row.add_suffix(&cross_thick);
        effects_group.add(&cross_thick_row);

        // Invert Colors
        let invert_lightness = adw::SwitchRow::builder()
            .title("Inverter Brilho")
            .subtitle("Inverte cores claras e escuras na lupa")
            .build();

        invert_lightness.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_magnifier_invert_lightness(active);
        });

        if let Ok(value) = DconfSettings::get_magnifier_invert_lightness() {
            invert_lightness.set_active(value);
        }

        effects_group.add(&invert_lightness);

        // Brightness
        let brightness_row = adw::ActionRow::builder()
            .title("Brilho")
            .build();

        let brightness = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, -1.0, 1.0, 0.1);
        brightness.set_value(0.0);
        brightness.set_draw_value(true);
        brightness.set_width_request(200);
        brightness.add_mark(0.0, gtk4::PositionType::Bottom, Some("Normal"));

        if let Ok(value) = DconfSettings::get_magnifier_brightness() {
            brightness.set_value(value);
        }

        brightness.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_magnifier_brightness(value);
        });

        brightness_row.add_suffix(&brightness);
        effects_group.add(&brightness_row);

        // Contrast
        let contrast_row = adw::ActionRow::builder()
            .title("Contraste")
            .build();

        let contrast = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, -1.0, 1.0, 0.1);
        contrast.set_value(0.0);
        contrast.set_draw_value(true);
        contrast.set_width_request(200);
        contrast.add_mark(0.0, gtk4::PositionType::Bottom, Some("Normal"));

        if let Ok(value) = DconfSettings::get_magnifier_contrast() {
            contrast.set_value(value);
        }

        contrast.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_magnifier_contrast(value);
        });

        contrast_row.add_suffix(&contrast);
        effects_group.add(&contrast_row);

        page.add(&effects_group);

        // Keyboard Shortcuts Group
        let shortcuts_group = adw::PreferencesGroup::builder()
            .title("Atalhos de Teclado")
            .build();

        let shortcuts = [
            ("Aumentar Zoom", "Super + ="),
            ("Diminuir Zoom", "Super + -"),
            ("Ativar/Desativar Lupa", "Super + Alt + 8"),
            ("Rolar Lupa", "Super + Alt + Setas"),
        ];

        for (action, shortcut) in shortcuts {
            let row = adw::ActionRow::builder()
                .title(action)
                .build();

            let shortcut_label = gtk4::Label::new(Some(shortcut));
            shortcut_label.add_css_class("dim-label");
            row.add_suffix(&shortcut_label);
            shortcuts_group.add(&row);
        }

        page.add(&shortcuts_group);

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

impl Default for ZoomPage {
    fn default() -> Self {
        Self::new()
    }
}
