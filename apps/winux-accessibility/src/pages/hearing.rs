//! Hearing accessibility settings page

use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::settings::DconfSettings;

/// Hearing settings page
pub struct HearingPage {
    widget: gtk4::ScrolledWindow,
}

impl HearingPage {
    /// Create a new hearing settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Audicao");
        page.set_icon_name(Some("audio-speakers-symbolic"));

        // Visual Alerts Group
        let alerts_group = adw::PreferencesGroup::builder()
            .title("Alertas Visuais")
            .description("Substitua sons por sinais visuais")
            .build();

        // Visual Alerts Enable
        let visual_alerts = adw::SwitchRow::builder()
            .title("Alertas Visuais")
            .subtitle("Pisca a tela quando um alerta sonoro ocorre")
            .build();

        visual_alerts.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_visual_alerts(active);
        });

        if let Ok(value) = DconfSettings::get_visual_alerts() {
            visual_alerts.set_active(value);
        }

        alerts_group.add(&visual_alerts);

        // Flash Type
        let flash_type = adw::ComboRow::builder()
            .title("Tipo de Flash")
            .subtitle("Escolha o que pisca durante alertas")
            .build();

        let flash_types = gtk4::StringList::new(&[
            "Piscar a Janela Inteira",
            "Piscar a Barra de Titulo",
            "Piscar a Tela Inteira",
        ]);
        flash_type.set_model(Some(&flash_types));

        if let Ok(flash) = DconfSettings::get_visual_alerts_type() {
            let index = match flash.as_str() {
                "frame" => 1,
                "fullscreen" => 2,
                _ => 0,
            };
            flash_type.set_selected(index);
        }

        flash_type.connect_selected_notify(|row| {
            let flash_type = match row.selected() {
                1 => "frame",
                2 => "fullscreen",
                _ => "window",
            };
            DconfSettings::set_visual_alerts_type(flash_type);
        });

        alerts_group.add(&flash_type);

        // Test Alert Button
        let test_alert = adw::ActionRow::builder()
            .title("Testar Alerta Visual")
            .activatable(true)
            .build();

        let test_btn = gtk4::Button::with_label("Testar");
        test_btn.add_css_class("suggested-action");
        test_btn.set_valign(gtk4::Align::Center);

        test_btn.connect_clicked(|_| {
            DconfSettings::trigger_visual_alert();
        });

        test_alert.add_suffix(&test_btn);
        alerts_group.add(&test_alert);

        page.add(&alerts_group);

        // Captions Group
        let captions_group = adw::PreferencesGroup::builder()
            .title("Legendas")
            .description("Configuracoes de legendas automaticas")
            .build();

        // Enable Captions
        let captions_enabled = adw::SwitchRow::builder()
            .title("Legendas Automaticas")
            .subtitle("Mostra legendas para audio quando disponivel")
            .build();

        captions_enabled.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_captions_enabled(active);
        });

        captions_group.add(&captions_enabled);

        // Caption Style
        let caption_style = adw::ComboRow::builder()
            .title("Estilo das Legendas")
            .build();

        let caption_styles = gtk4::StringList::new(&[
            "Padrao",
            "Fundo Branco, Texto Preto",
            "Fundo Preto, Texto Branco",
            "Fundo Amarelo, Texto Preto",
            "Fundo Transparente",
        ]);
        caption_style.set_model(Some(&caption_styles));
        captions_group.add(&caption_style);

        // Caption Font Size
        let caption_size = adw::ComboRow::builder()
            .title("Tamanho da Fonte")
            .build();

        let caption_sizes = gtk4::StringList::new(&[
            "Pequeno",
            "Medio",
            "Grande",
            "Extra Grande",
        ]);
        caption_size.set_model(Some(&caption_sizes));
        caption_size.set_selected(1);
        captions_group.add(&caption_size);

        // Caption Position
        let caption_position = adw::ComboRow::builder()
            .title("Posicao das Legendas")
            .build();

        let positions = gtk4::StringList::new(&[
            "Inferior",
            "Superior",
        ]);
        caption_position.set_model(Some(&positions));
        captions_group.add(&caption_position);

        page.add(&captions_group);

        // Audio Balance Group
        let audio_group = adw::PreferencesGroup::builder()
            .title("Balanco de Audio")
            .description("Ajuste o balanco estereo")
            .build();

        // Mono Audio
        let mono_audio = adw::SwitchRow::builder()
            .title("Audio Mono")
            .subtitle("Combina canais de audio esquerdo e direito")
            .build();

        mono_audio.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_mono_audio(active);
        });

        if let Ok(value) = DconfSettings::get_mono_audio() {
            mono_audio.set_active(value);
        }

        audio_group.add(&mono_audio);

        // Audio Balance
        let balance_row = adw::ActionRow::builder()
            .title("Balanco Estereo")
            .subtitle("Ajuste o balanco esquerda-direita")
            .build();

        let balance_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        balance_box.set_valign(gtk4::Align::Center);

        let left_label = gtk4::Label::new(Some("E"));
        let right_label = gtk4::Label::new(Some("D"));

        let balance_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, -1.0, 1.0, 0.1);
        balance_scale.set_value(0.0);
        balance_scale.set_draw_value(false);
        balance_scale.set_width_request(200);
        balance_scale.add_mark(0.0, gtk4::PositionType::Bottom, Some("Centro"));

        balance_scale.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_audio_balance(value);
        });

        balance_box.append(&left_label);
        balance_box.append(&balance_scale);
        balance_box.append(&right_label);

        balance_row.add_suffix(&balance_box);
        audio_group.add(&balance_row);

        page.add(&audio_group);

        // Sound Notifications Group
        let sound_group = adw::PreferencesGroup::builder()
            .title("Notificacoes Sonoras")
            .build();

        // Event Sounds
        let event_sounds = adw::SwitchRow::builder()
            .title("Sons de Eventos")
            .subtitle("Tocar sons para eventos do sistema")
            .active(true)
            .build();

        event_sounds.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_event_sounds(active);
        });

        sound_group.add(&event_sounds);

        // Input Feedback Sounds
        let input_sounds = adw::SwitchRow::builder()
            .title("Sons de Entrada")
            .subtitle("Tocar sons ao clicar em botoes")
            .build();

        input_sounds.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_input_feedback_sounds(active);
        });

        sound_group.add(&input_sounds);

        page.add(&sound_group);

        // Hearing Aids Group
        let aids_group = adw::PreferencesGroup::builder()
            .title("Aparelhos Auditivos")
            .description("Configuracoes para aparelhos auditivos Bluetooth")
            .build();

        // Hearing Aid Mode
        let hearing_aid_mode = adw::SwitchRow::builder()
            .title("Modo de Aparelho Auditivo")
            .subtitle("Otimiza audio para aparelhos auditivos")
            .build();
        aids_group.add(&hearing_aid_mode);

        // Open Bluetooth Settings
        let bt_settings = adw::ActionRow::builder()
            .title("Configuracoes de Bluetooth")
            .subtitle("Gerenciar dispositivos Bluetooth")
            .activatable(true)
            .build();

        bt_settings.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

        bt_settings.connect_activated(|_| {
            let _ = std::process::Command::new("winux-bluetooth")
                .spawn();
        });

        aids_group.add(&bt_settings);

        page.add(&aids_group);

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

impl Default for HearingPage {
    fn default() -> Self {
        Self::new()
    }
}
