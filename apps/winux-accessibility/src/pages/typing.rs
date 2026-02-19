//! Typing accessibility settings page

use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::settings::DconfSettings;

/// Typing settings page
pub struct TypingPage {
    widget: gtk4::ScrolledWindow,
}

impl TypingPage {
    /// Create a new typing settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Digitacao");
        page.set_icon_name(Some("input-keyboard-symbolic"));

        // On-Screen Keyboard Group
        let osk_group = adw::PreferencesGroup::builder()
            .title("Teclado na Tela")
            .description("Use um teclado virtual na tela")
            .build();

        // Enable On-Screen Keyboard
        let osk_enabled = adw::SwitchRow::builder()
            .title("Teclado na Tela")
            .subtitle("Mostra um teclado virtual quando necessario")
            .build();

        osk_enabled.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_screen_keyboard_enabled(active);
        });

        if let Ok(value) = DconfSettings::get_screen_keyboard_enabled() {
            osk_enabled.set_active(value);
        }

        osk_group.add(&osk_enabled);

        // OSK Layout
        let osk_layout = adw::ComboRow::builder()
            .title("Layout do Teclado")
            .build();

        let layouts = gtk4::StringList::new(&[
            "Padrao",
            "Compacto",
            "Estendido",
            "Apenas Numeros",
        ]);
        osk_layout.set_model(Some(&layouts));
        osk_group.add(&osk_layout);

        page.add(&osk_group);

        // Sticky Keys Group
        let sticky_group = adw::PreferencesGroup::builder()
            .title("Teclas de Aderencia (Sticky Keys)")
            .description("Pressione teclas modificadoras uma de cada vez")
            .build();

        // Enable Sticky Keys
        let sticky_keys = adw::SwitchRow::builder()
            .title("Teclas de Aderencia")
            .subtitle("Mantenha Shift, Ctrl, Alt pressionados sem segurar")
            .build();

        sticky_keys.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_sticky_keys(active);
        });

        if let Ok(value) = DconfSettings::get_sticky_keys() {
            sticky_keys.set_active(value);
        }

        sticky_group.add(&sticky_keys);

        // Two Key Activation
        let sticky_two_key = adw::SwitchRow::builder()
            .title("Ativar com Duas Teclas")
            .subtitle("Ativa Sticky Keys pressionando Shift 5 vezes")
            .active(true)
            .build();

        sticky_two_key.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_sticky_keys_two_key_off(active);
        });

        sticky_group.add(&sticky_two_key);

        // Sticky Keys Beep
        let sticky_beep = adw::SwitchRow::builder()
            .title("Beep ao Pressionar Modificador")
            .subtitle("Emite som quando uma tecla modificadora e pressionada")
            .build();

        sticky_beep.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_sticky_keys_beep(active);
        });

        sticky_group.add(&sticky_beep);

        page.add(&sticky_group);

        // Slow Keys Group
        let slow_group = adw::PreferencesGroup::builder()
            .title("Teclas Lentas (Slow Keys)")
            .description("Ignore pressionamentos acidentais rapidos")
            .build();

        // Enable Slow Keys
        let slow_keys = adw::SwitchRow::builder()
            .title("Teclas Lentas")
            .subtitle("Teclas precisam ser pressionadas por mais tempo")
            .build();

        slow_keys.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_slow_keys(active);
        });

        if let Ok(value) = DconfSettings::get_slow_keys() {
            slow_keys.set_active(value);
        }

        slow_group.add(&slow_keys);

        // Slow Keys Delay
        let slow_delay_row = adw::ActionRow::builder()
            .title("Atraso de Aceitacao")
            .subtitle("Tempo para manter a tecla pressionada")
            .build();

        let slow_delay = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 100.0, 2000.0, 100.0);
        slow_delay.set_value(300.0);
        slow_delay.set_draw_value(true);
        slow_delay.set_width_request(200);

        if let Ok(value) = DconfSettings::get_slow_keys_delay() {
            slow_delay.set_value(value as f64);
        }

        slow_delay.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_slow_keys_delay(value);
        });

        let delay_label = gtk4::Label::new(Some("ms"));
        delay_label.add_css_class("dim-label");

        let delay_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        delay_box.append(&slow_delay);
        delay_box.append(&delay_label);
        delay_box.set_valign(gtk4::Align::Center);

        slow_delay_row.add_suffix(&delay_box);
        slow_group.add(&slow_delay_row);

        // Slow Keys Beep
        let slow_beep = adw::SwitchRow::builder()
            .title("Beep ao Aceitar/Rejeitar")
            .subtitle("Emite som quando tecla e aceita ou rejeitada")
            .build();

        slow_beep.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_slow_keys_beep_accept(active);
        });

        slow_group.add(&slow_beep);

        page.add(&slow_group);

        // Bounce Keys Group
        let bounce_group = adw::PreferencesGroup::builder()
            .title("Teclas de Rejeicao (Bounce Keys)")
            .description("Ignore pressionamentos repetidos acidentais")
            .build();

        // Enable Bounce Keys
        let bounce_keys = adw::SwitchRow::builder()
            .title("Teclas de Rejeicao")
            .subtitle("Ignora pressionamentos rapidos e repetidos da mesma tecla")
            .build();

        bounce_keys.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_bounce_keys(active);
        });

        if let Ok(value) = DconfSettings::get_bounce_keys() {
            bounce_keys.set_active(value);
        }

        bounce_group.add(&bounce_keys);

        // Bounce Keys Delay
        let bounce_delay_row = adw::ActionRow::builder()
            .title("Intervalo de Rejeicao")
            .subtitle("Tempo minimo entre pressionamentos")
            .build();

        let bounce_delay = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 100.0, 2000.0, 100.0);
        bounce_delay.set_value(300.0);
        bounce_delay.set_draw_value(true);
        bounce_delay.set_width_request(200);

        if let Ok(value) = DconfSettings::get_bounce_keys_delay() {
            bounce_delay.set_value(value as f64);
        }

        bounce_delay.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_bounce_keys_delay(value);
        });

        let bounce_label = gtk4::Label::new(Some("ms"));
        bounce_label.add_css_class("dim-label");

        let bounce_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        bounce_box.append(&bounce_delay);
        bounce_box.append(&bounce_label);
        bounce_box.set_valign(gtk4::Align::Center);

        bounce_delay_row.add_suffix(&bounce_box);
        bounce_group.add(&bounce_delay_row);

        // Bounce Keys Beep
        let bounce_beep = adw::SwitchRow::builder()
            .title("Beep ao Rejeitar")
            .subtitle("Emite som quando tecla repetida e rejeitada")
            .build();

        bounce_beep.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_bounce_keys_beep(active);
        });

        bounce_group.add(&bounce_beep);

        page.add(&bounce_group);

        // Repeat Keys Group
        let repeat_group = adw::PreferencesGroup::builder()
            .title("Repeticao de Teclas")
            .description("Controle o comportamento de repeticao ao manter teclas pressionadas")
            .build();

        // Enable Key Repeat
        let repeat_keys = adw::SwitchRow::builder()
            .title("Repeticao de Teclas")
            .subtitle("Repetir caractere quando tecla e mantida pressionada")
            .active(true)
            .build();

        repeat_keys.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_repeat_keys(active);
        });

        repeat_group.add(&repeat_keys);

        // Repeat Delay
        let repeat_delay_row = adw::ActionRow::builder()
            .title("Atraso Inicial")
            .subtitle("Tempo antes de comecar a repetir")
            .build();

        let repeat_delay = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 100.0, 2000.0, 100.0);
        repeat_delay.set_value(500.0);
        repeat_delay.set_draw_value(true);
        repeat_delay.set_width_request(200);

        if let Ok(value) = DconfSettings::get_repeat_keys_delay() {
            repeat_delay.set_value(value as f64);
        }

        repeat_delay.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_repeat_keys_delay(value);
        });

        repeat_delay_row.add_suffix(&repeat_delay);
        repeat_group.add(&repeat_delay_row);

        // Repeat Interval
        let repeat_interval_row = adw::ActionRow::builder()
            .title("Intervalo de Repeticao")
            .subtitle("Velocidade de repeticao")
            .build();

        let repeat_interval = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 10.0, 500.0, 10.0);
        repeat_interval.set_value(30.0);
        repeat_interval.set_draw_value(true);
        repeat_interval.set_width_request(200);

        if let Ok(value) = DconfSettings::get_repeat_keys_interval() {
            repeat_interval.set_value(value as f64);
        }

        repeat_interval.connect_value_changed(|scale| {
            let value = scale.value() as u32;
            DconfSettings::set_repeat_keys_interval(value);
        });

        repeat_interval_row.add_suffix(&repeat_interval);
        repeat_group.add(&repeat_interval_row);

        page.add(&repeat_group);

        // Typing Assists Group
        let assists_group = adw::PreferencesGroup::builder()
            .title("Auxilios de Digitacao")
            .build();

        // Word Completion
        let word_completion = adw::SwitchRow::builder()
            .title("Completar Palavras")
            .subtitle("Sugere palavras enquanto voce digita")
            .build();
        assists_group.add(&word_completion);

        // Auto Capitalization
        let auto_caps = adw::SwitchRow::builder()
            .title("Capitalizacao Automatica")
            .subtitle("Coloca maiuscula no inicio de frases")
            .active(true)
            .build();
        assists_group.add(&auto_caps);

        // Spell Check
        let spell_check = adw::SwitchRow::builder()
            .title("Verificacao Ortografica")
            .subtitle("Destaca erros ortograficos")
            .active(true)
            .build();
        assists_group.add(&spell_check);

        page.add(&assists_group);

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

impl Default for TypingPage {
    fn default() -> Self {
        Self::new()
    }
}
