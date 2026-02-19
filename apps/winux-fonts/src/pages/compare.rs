// Compare fonts page - compare 2+ fonts side by side

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, Label, ListBox, Orientation, ScrolledWindow,
    SelectionMode, Separator, DropDown, StringList, Frame,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, Clamp};
use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::FontManager;

pub fn create_page(font_manager: Rc<RefCell<FontManager>>) -> gtk4::ScrolledWindow {
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Header
    let header = Box::new(Orientation::Horizontal, 12);

    let title = Label::builder()
        .label("Comparar Fontes")
        .css_classes(vec!["title-2"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);

    let add_font_btn = Button::builder()
        .label("Adicionar Fonte")
        .icon_name("list-add-symbolic")
        .css_classes(vec!["suggested-action"])
        .build();
    header.append(&add_font_btn);

    main_box.append(&header);

    // Sample text input
    let text_group = PreferencesGroup::builder()
        .title("Texto de Amostra")
        .build();

    let text_entry = Entry::builder()
        .text("The quick brown fox jumps over the lazy dog")
        .build();
    text_group.add(&text_entry);

    main_box.append(&text_group);

    // Size control
    let size_box = Box::new(Orientation::Horizontal, 12);
    let size_label = Label::new(Some("Tamanho:"));
    size_box.append(&size_label);

    let sizes = StringList::new(&["12pt", "16pt", "24pt", "32pt", "48pt", "64pt", "72pt"]);
    let size_dropdown = DropDown::builder()
        .model(&sizes)
        .selected(2) // 24pt
        .build();
    size_box.append(&size_dropdown);

    main_box.append(&size_box);

    // Comparison area
    let compare_area = Box::new(Orientation::Vertical, 16);
    compare_area.set_margin_top(16);

    // Selected fonts container (will be populated dynamically)
    let selected_fonts: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![
        "Noto Sans".to_string(),
        "Noto Serif".to_string(),
    ]));

    // Create comparison rows for initial fonts
    for font_name in selected_fonts.borrow().iter() {
        let row = create_comparison_row(font_name, &text_entry.text(), 24);
        compare_area.append(&row);
    }

    // Add font dialog handler
    {
        let fm = font_manager.clone();
        let area = compare_area.clone();
        let entry = text_entry.clone();
        let fonts = selected_fonts.clone();

        add_font_btn.connect_clicked(move |_| {
            // In a real app, this would show a font picker dialog
            // For now, add a sample font
            let sample_fonts = ["Ubuntu", "DejaVu Sans", "Liberation Mono", "Cantarell"];
            let current_count = fonts.borrow().len();
            if current_count < sample_fonts.len() {
                let new_font = sample_fonts[current_count].to_string();
                fonts.borrow_mut().push(new_font.clone());

                let row = create_comparison_row(&new_font, &entry.text(), 24);
                area.append(&row);
            }
        });
    }

    // Update text on change
    {
        let area = compare_area.clone();
        let fonts = selected_fonts.clone();

        text_entry.connect_changed(move |entry| {
            let text = entry.text().to_string();
            // Remove existing rows
            while let Some(child) = area.first_child() {
                area.remove(&child);
            }
            // Recreate with new text
            for font_name in fonts.borrow().iter() {
                let row = create_comparison_row(font_name, &text, 24);
                area.append(&row);
            }
        });
    }

    let compare_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .child(&compare_area)
        .build();

    main_box.append(&compare_scroll);

    // Tips section
    let tips_group = PreferencesGroup::builder()
        .title("Dicas")
        .build();

    let tips = ActionRow::builder()
        .title("Compare fontes para escolher a melhor")
        .subtitle("Adicione ate 6 fontes para comparacao lado a lado. Use o mesmo texto em todas para avaliar legibilidade e estilo.")
        .build();
    tips.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
    tips_group.add(&tips);

    main_box.append(&tips_group);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_comparison_row(font_name: &str, sample_text: &str, size: i32) -> Box {
    let row = Box::new(Orientation::Vertical, 8);
    row.add_css_class("card");
    row.set_margin_bottom(8);

    // Header with font name and remove button
    let header = Box::new(Orientation::Horizontal, 12);
    header.set_margin_top(12);
    header.set_margin_start(12);
    header.set_margin_end(12);

    let name_label = Label::builder()
        .label(font_name)
        .css_classes(vec!["title-4"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&name_label);

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);

    let remove_btn = Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(vec!["flat", "circular"])
        .tooltip_text("Remover da comparacao")
        .build();
    header.append(&remove_btn);

    row.append(&header);

    // Sample text with font applied
    let sample_label = Label::builder()
        .label(sample_text)
        .halign(gtk4::Align::Start)
        .wrap(true)
        .margin_top(8)
        .margin_bottom(16)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Apply font family and size
    let attr_list = pango::AttrList::new();
    let font_desc = pango::FontDescription::from_string(&format!("{} {}", font_name, size));
    let font_attr = pango::AttrFontDesc::new(&font_desc);
    attr_list.insert(font_attr);
    sample_label.set_attributes(Some(&attr_list));

    row.append(&sample_label);

    // Additional info line
    let info_box = Box::new(Orientation::Horizontal, 12);
    info_box.set_margin_start(12);
    info_box.set_margin_end(12);
    info_box.set_margin_bottom(12);

    let style_label = Label::builder()
        .label("Regular")
        .css_classes(vec!["dim-label"])
        .build();
    info_box.append(&style_label);

    let size_label = Label::builder()
        .label(&format!("{}pt", size))
        .css_classes(vec!["dim-label"])
        .build();
    info_box.append(&size_label);

    row.append(&info_box);

    // Remove button handler
    {
        let row_clone = row.clone();
        remove_btn.connect_clicked(move |_| {
            if let Some(parent) = row_clone.parent() {
                if let Some(parent_box) = parent.downcast_ref::<Box>() {
                    parent_box.remove(&row_clone);
                }
            }
        });
    }

    row
}
