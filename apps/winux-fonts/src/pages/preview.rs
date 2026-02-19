// Font preview page - customizable preview with different sizes and styles

use gtk4::prelude::*;
use gtk4::{
    Box, Label, Orientation, ScrolledWindow, Entry, Scale,
    SpinButton, Adjustment, ColorButton, TextView, TextBuffer,
    ComboBoxText,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, Clamp, ComboRow, SwitchRow};
use gdk4::RGBA;
use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::FontManager;
use crate::ui::specimen::SpecimenView;

pub fn create_page(font_manager: Rc<RefCell<FontManager>>) -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Font selection group
    let font_group = PreferencesGroup::builder()
        .title("Fonte")
        .description("Selecione a fonte para visualizar")
        .build();

    // Font family selector
    let family_row = ComboRow::builder()
        .title("Familia")
        .subtitle("Selecione uma familia de fontes")
        .build();

    let families: Vec<&str> = font_manager.borrow()
        .get_families()
        .iter()
        .take(100)
        .map(|s| s.as_str())
        .collect();

    let family_list = gtk4::StringList::new(&families);
    family_row.set_model(Some(&family_list));
    font_group.add(&family_row);

    // Style selector
    let style_row = ComboRow::builder()
        .title("Estilo")
        .subtitle("Regular, Bold, Italic, etc.")
        .build();
    let styles = gtk4::StringList::new(&["Regular", "Bold", "Italic", "Bold Italic", "Light", "Medium"]);
    style_row.set_model(Some(&styles));
    font_group.add(&style_row);

    page.add(&font_group);

    // Preview settings group
    let settings_group = PreferencesGroup::builder()
        .title("Configuracoes de Visualizacao")
        .build();

    // Size slider
    let size_row = ActionRow::builder()
        .title("Tamanho")
        .subtitle("12 - 144pt")
        .build();

    let size_adj = Adjustment::new(24.0, 12.0, 144.0, 1.0, 10.0, 0.0);
    let size_scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&size_adj)
        .width_request(200)
        .draw_value(true)
        .value_pos(gtk4::PositionType::Left)
        .build();
    size_row.add_suffix(&size_scale);
    settings_group.add(&size_row);

    // Text color
    let text_color_row = ActionRow::builder()
        .title("Cor do Texto")
        .build();
    let text_color_btn = ColorButton::builder()
        .rgba(&RGBA::new(1.0, 1.0, 1.0, 1.0))
        .build();
    text_color_row.add_suffix(&text_color_btn);
    settings_group.add(&text_color_row);

    // Background color
    let bg_color_row = ActionRow::builder()
        .title("Cor de Fundo")
        .build();
    let bg_color_btn = ColorButton::builder()
        .rgba(&RGBA::new(0.1, 0.1, 0.1, 1.0))
        .build();
    bg_color_row.add_suffix(&bg_color_btn);
    settings_group.add(&bg_color_row);

    page.add(&settings_group);

    // Custom text group
    let text_group = PreferencesGroup::builder()
        .title("Texto de Amostra")
        .description("Digite seu proprio texto ou use um dos modelos")
        .build();

    // Preset texts
    let preset_row = ComboRow::builder()
        .title("Texto Predefinido")
        .build();
    let presets = gtk4::StringList::new(&[
        "Personalizado",
        "Alfabeto",
        "Numeros",
        "Pangrama (PT)",
        "Pangrama (EN)",
        "Lorem Ipsum",
    ]);
    preset_row.set_model(Some(&presets));
    text_group.add(&preset_row);

    page.add(&text_group);

    // Preview area
    let preview_group = PreferencesGroup::builder()
        .title("Visualizacao")
        .build();

    // Create specimen view
    let specimen = SpecimenView::new();
    specimen.set_text("ABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz\n0123456789\n!@#$%^&*()[]{}");

    let specimen_widget = specimen.widget();
    specimen_widget.set_margin_top(12);
    specimen_widget.set_margin_bottom(12);
    preview_group.add(&specimen_widget);

    // Multi-size preview
    let sizes_box = Box::new(Orientation::Vertical, 8);
    sizes_box.set_margin_top(12);
    sizes_box.set_margin_bottom(12);

    for size in [12, 18, 24, 36, 48, 72] {
        let label = Label::builder()
            .label(&format!("{}pt - The quick brown fox jumps over the lazy dog", size))
            .halign(gtk4::Align::Start)
            .build();

        let attr_list = pango::AttrList::new();
        let size_attr = pango::AttrInt::new_size(size * pango::SCALE);
        attr_list.insert(size_attr);
        label.set_attributes(Some(&attr_list));

        sizes_box.append(&label);
    }

    preview_group.add(&sizes_box);
    page.add(&preview_group);

    // Font weights preview
    let weights_group = PreferencesGroup::builder()
        .title("Pesos Disponiveis")
        .description("Visualize todos os pesos da fonte")
        .build();

    let weights = [
        ("Thin", 100),
        ("Light", 300),
        ("Regular", 400),
        ("Medium", 500),
        ("Semi-Bold", 600),
        ("Bold", 700),
        ("Extra-Bold", 800),
        ("Black", 900),
    ];

    for (name, weight) in weights {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(&format!("Weight: {}", weight))
            .build();

        let preview_label = Label::new(Some("Aa Bb Cc"));
        let attr_list = pango::AttrList::new();
        let weight_attr = pango::AttrInt::new_weight(pango::Weight::__Unknown(weight));
        attr_list.insert(weight_attr);
        preview_label.set_attributes(Some(&attr_list));
        row.add_suffix(&preview_label);

        weights_group.add(&row);
    }

    page.add(&weights_group);

    // Connect signals
    {
        let spec = specimen.clone();
        size_adj.connect_value_changed(move |adj| {
            spec.set_size(adj.value() as i32);
        });
    }

    {
        let spec = specimen.clone();
        text_color_btn.connect_rgba_notify(move |btn| {
            spec.set_text_color(btn.rgba());
        });
    }

    {
        let spec = specimen.clone();
        bg_color_btn.connect_rgba_notify(move |btn| {
            spec.set_bg_color(btn.rgba());
        });
    }

    {
        let spec = specimen.clone();
        preset_row.connect_selected_notify(move |row| {
            let texts = [
                "", // Custom - don't change
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz",
                "0123456789",
                "A raposa marrom rapida salta sobre o cachorro preguicoso",
                "The quick brown fox jumps over the lazy dog",
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            ];
            let idx = row.selected() as usize;
            if idx > 0 && idx < texts.len() {
                spec.set_text(texts[idx]);
            }
        });
    }

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}
