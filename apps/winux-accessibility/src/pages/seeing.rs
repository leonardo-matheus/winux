//! Vision/Seeing accessibility settings page

use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::settings::{DconfSettings, AtSpiSettings};
use crate::ui::ToggleRow;

/// Seeing (Vision) settings page
pub struct SeeingPage {
    widget: gtk4::ScrolledWindow,
}

impl SeeingPage {
    /// Create a new seeing settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Visao");
        page.set_icon_name(Some("eye-open-negative-filled-symbolic"));

        // Visual Aids Group
        let visual_group = adw::PreferencesGroup::builder()
            .title("Auxilios Visuais")
            .description("Ajustes para melhorar a visibilidade")
            .build();

        // High Contrast
        let high_contrast = adw::SwitchRow::builder()
            .title("Alto Contraste")
            .subtitle("Aumenta o contraste das cores da interface")
            .build();

        high_contrast.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_high_contrast(active);
        });

        // Load current value
        if let Ok(value) = DconfSettings::get_high_contrast() {
            high_contrast.set_active(value);
        }

        visual_group.add(&high_contrast);

        // Large Text
        let large_text = adw::SwitchRow::builder()
            .title("Texto Grande")
            .subtitle("Aumenta o tamanho do texto da interface")
            .build();

        large_text.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_large_text(active);
        });

        if let Ok(value) = DconfSettings::get_large_text() {
            large_text.set_active(value);
        }

        visual_group.add(&large_text);

        // Text Scaling Factor
        let text_scale_row = adw::ActionRow::builder()
            .title("Fator de Escala do Texto")
            .subtitle("Ajusta o tamanho do texto (1.0 = 100%)")
            .build();

        let text_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.5, 3.0, 0.1);
        text_scale.set_value(1.0);
        text_scale.set_draw_value(true);
        text_scale.set_width_request(200);
        text_scale.add_mark(1.0, gtk4::PositionType::Bottom, Some("100%"));
        text_scale.add_mark(1.5, gtk4::PositionType::Bottom, Some("150%"));
        text_scale.add_mark(2.0, gtk4::PositionType::Bottom, Some("200%"));

        if let Ok(value) = DconfSettings::get_text_scaling_factor() {
            text_scale.set_value(value);
        }

        text_scale.connect_value_changed(|scale| {
            let value = scale.value();
            DconfSettings::set_text_scaling_factor(value);
        });

        text_scale_row.add_suffix(&text_scale);
        visual_group.add(&text_scale_row);

        // Large Cursor
        let large_cursor = adw::SwitchRow::builder()
            .title("Cursor Grande")
            .subtitle("Aumenta o tamanho do cursor do mouse")
            .build();

        large_cursor.connect_active_notify(|switch| {
            let active = switch.is_active();
            let size = if active { 48 } else { 24 };
            DconfSettings::set_cursor_size(size);
        });

        if let Ok(size) = DconfSettings::get_cursor_size() {
            large_cursor.set_active(size > 32);
        }

        visual_group.add(&large_cursor);

        // Cursor Size
        let cursor_size_row = adw::ComboRow::builder()
            .title("Tamanho do Cursor")
            .subtitle("Escolha o tamanho do cursor")
            .build();

        let cursor_sizes = gtk4::StringList::new(&[
            "Pequeno (24px)",
            "Medio (32px)",
            "Grande (48px)",
            "Extra Grande (64px)",
            "Enorme (96px)",
        ]);
        cursor_size_row.set_model(Some(&cursor_sizes));

        if let Ok(size) = DconfSettings::get_cursor_size() {
            let index = match size {
                24 => 0,
                32 => 1,
                48 => 2,
                64 => 3,
                96 => 4,
                _ => 1,
            };
            cursor_size_row.set_selected(index);
        }

        cursor_size_row.connect_selected_notify(|row| {
            let size = match row.selected() {
                0 => 24,
                1 => 32,
                2 => 48,
                3 => 64,
                4 => 96,
                _ => 32,
            };
            DconfSettings::set_cursor_size(size);
        });

        visual_group.add(&cursor_size_row);

        // Reduce Animations
        let reduce_animations = adw::SwitchRow::builder()
            .title("Reduzir Animacoes")
            .subtitle("Minimiza animacoes e transicoes")
            .build();

        reduce_animations.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_reduce_animations(active);
        });

        if let Ok(value) = DconfSettings::get_reduce_animations() {
            reduce_animations.set_active(value);
        }

        visual_group.add(&reduce_animations);

        page.add(&visual_group);

        // Screen Reader Group
        let reader_group = adw::PreferencesGroup::builder()
            .title("Leitor de Tela")
            .description("Orca - leitor de tela para usuarios cegos ou com baixa visao")
            .build();

        // Screen Reader Enable
        let screen_reader = adw::SwitchRow::builder()
            .title("Leitor de Tela (Orca)")
            .subtitle("Leia o conteudo da tela em voz alta")
            .build();

        screen_reader.connect_active_notify(|switch| {
            let active = switch.is_active();
            AtSpiSettings::set_screen_reader_enabled(active);
            if active {
                AtSpiSettings::start_orca();
            } else {
                AtSpiSettings::stop_orca();
            }
        });

        if let Ok(value) = AtSpiSettings::is_screen_reader_enabled() {
            screen_reader.set_active(value);
        }

        reader_group.add(&screen_reader);

        // Speech Rate
        let speech_rate_row = adw::ActionRow::builder()
            .title("Velocidade da Fala")
            .subtitle("Ajusta a velocidade do leitor de tela")
            .build();

        let speech_rate = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 50.0, 400.0, 10.0);
        speech_rate.set_value(175.0);
        speech_rate.set_draw_value(true);
        speech_rate.set_width_request(200);
        speech_rate.add_mark(100.0, gtk4::PositionType::Bottom, Some("Lento"));
        speech_rate.add_mark(175.0, gtk4::PositionType::Bottom, Some("Normal"));
        speech_rate.add_mark(300.0, gtk4::PositionType::Bottom, Some("Rapido"));

        speech_rate_row.add_suffix(&speech_rate);
        reader_group.add(&speech_rate_row);

        // Configure Orca Button
        let configure_orca = adw::ActionRow::builder()
            .title("Configurar Orca")
            .subtitle("Abrir configuracoes avancadas do leitor de tela")
            .activatable(true)
            .build();

        configure_orca.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

        configure_orca.connect_activated(|_| {
            AtSpiSettings::open_orca_settings();
        });

        reader_group.add(&configure_orca);

        page.add(&reader_group);

        // Color Filters Group
        let color_group = adw::PreferencesGroup::builder()
            .title("Filtros de Cor")
            .description("Para daltonismo e sensibilidade a cores")
            .build();

        // Color Filter Type
        let color_filter = adw::ComboRow::builder()
            .title("Filtro de Cor")
            .subtitle("Ajuda pessoas com daltonismo")
            .build();

        let filters = gtk4::StringList::new(&[
            "Nenhum",
            "Protanopia (Vermelho-Verde)",
            "Deuteranopia (Verde-Vermelho)",
            "Tritanopia (Azul-Amarelo)",
            "Escala de Cinza",
            "Escala de Cinza Invertida",
            "Cores Invertidas",
        ]);
        color_filter.set_model(Some(&filters));
        color_group.add(&color_filter);

        // Color Filter Intensity
        let filter_intensity_row = adw::ActionRow::builder()
            .title("Intensidade do Filtro")
            .build();

        let filter_intensity = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 5.0);
        filter_intensity.set_value(100.0);
        filter_intensity.set_draw_value(true);
        filter_intensity.set_width_request(200);
        filter_intensity_row.add_suffix(&filter_intensity);
        color_group.add(&filter_intensity_row);

        page.add(&color_group);

        // Reading Aids Group
        let reading_group = adw::PreferencesGroup::builder()
            .title("Auxilios de Leitura")
            .build();

        // Show Scrollbars
        let always_scrollbars = adw::SwitchRow::builder()
            .title("Sempre Mostrar Barras de Rolagem")
            .subtitle("Mostra barras de rolagem permanentemente")
            .build();

        always_scrollbars.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_always_show_scrollbars(active);
        });

        reading_group.add(&always_scrollbars);

        // Cursor Blink
        let cursor_blink = adw::SwitchRow::builder()
            .title("Cursor Piscante")
            .subtitle("Faz o cursor de texto piscar")
            .active(true)
            .build();

        cursor_blink.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_cursor_blink(active);
        });

        reading_group.add(&cursor_blink);

        // Locate Pointer
        let locate_pointer = adw::SwitchRow::builder()
            .title("Localizar Ponteiro")
            .subtitle("Pressione Ctrl para destacar o cursor")
            .build();

        locate_pointer.connect_active_notify(|switch| {
            let active = switch.is_active();
            DconfSettings::set_locate_pointer(active);
        });

        if let Ok(value) = DconfSettings::get_locate_pointer() {
            locate_pointer.set_active(value);
        }

        reading_group.add(&locate_pointer);

        page.add(&reading_group);

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

impl Default for SeeingPage {
    fn default() -> Self {
        Self::new()
    }
}
