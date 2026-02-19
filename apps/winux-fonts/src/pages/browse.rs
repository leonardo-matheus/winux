// Browse fonts page - list all installed fonts with filtering

use gtk4::prelude::*;
use gtk4::{
    Box, Label, ListBox, Orientation, ScrolledWindow, SearchEntry,
    SelectionMode, Separator, DropDown, StringList,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, Clamp};
use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::{FontManager, FontCategory};
use crate::ui::font_card::FontCard;

pub fn create_page(
    font_manager: Rc<RefCell<FontManager>>,
    search_entry: SearchEntry,
) -> gtk4::ScrolledWindow {
    let main_box = Box::new(Orientation::Horizontal, 0);

    // Left sidebar - filters
    let sidebar = create_sidebar(font_manager.clone());
    main_box.append(&sidebar);

    // Separator
    let sep = Separator::new(Orientation::Vertical);
    main_box.append(&sep);

    // Right content - font list
    let content = create_font_list(font_manager.clone());
    content.set_hexpand(true);
    main_box.append(&content);

    // Connect search
    {
        let fm = font_manager.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            let mut manager = fm.borrow_mut();
            manager.set_search_filter(&query);
        });
    }

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&main_box)
        .build();

    scrolled
}

fn create_sidebar(font_manager: Rc<RefCell<FontManager>>) -> Box {
    let sidebar = Box::new(Orientation::Vertical, 12);
    sidebar.set_width_request(250);
    sidebar.set_margin_top(12);
    sidebar.set_margin_bottom(12);
    sidebar.set_margin_start(12);
    sidebar.set_margin_end(12);

    // Categories header
    let cat_label = Label::builder()
        .label("Categorias")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-4"])
        .build();
    sidebar.append(&cat_label);

    // Category list
    let cat_list = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .css_classes(vec!["boxed-list"])
        .build();

    let categories = [
        ("Todas", "font-x-generic-symbolic", FontCategory::All),
        ("Serif", "format-text-italic-symbolic", FontCategory::Serif),
        ("Sans-Serif", "format-text-bold-symbolic", FontCategory::SansSerif),
        ("Monoespaco", "utilities-terminal-symbolic", FontCategory::Monospace),
        ("Display", "format-text-larger-symbolic", FontCategory::Display),
        ("Cursiva", "format-text-underline-symbolic", FontCategory::Handwriting),
    ];

    for (name, icon, category) in categories {
        let row = ActionRow::builder()
            .title(name)
            .activatable(true)
            .build();

        let icon_widget = gtk4::Image::from_icon_name(icon);
        row.add_prefix(&icon_widget);

        // Count label
        let count = font_manager.borrow().count_by_category(&category);
        let count_label = Label::new(Some(&count.to_string()));
        count_label.add_css_class("dim-label");
        row.add_suffix(&count_label);

        let fm = font_manager.clone();
        let cat = category.clone();
        row.connect_activated(move |_| {
            fm.borrow_mut().set_category_filter(cat.clone());
        });

        cat_list.append(&row);
    }

    // Select first row
    if let Some(first) = cat_list.row_at_index(0) {
        cat_list.select_row(Some(&first));
    }

    sidebar.append(&cat_list);

    // Stats section
    let stats_label = Label::builder()
        .label("Estatisticas")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-4"])
        .margin_top(24)
        .build();
    sidebar.append(&stats_label);

    let stats_group = PreferencesGroup::new();

    let total_row = ActionRow::builder()
        .title("Total de Fontes")
        .subtitle(&font_manager.borrow().font_count().to_string())
        .build();
    stats_group.add(&total_row);

    let families_row = ActionRow::builder()
        .title("Familias")
        .subtitle(&font_manager.borrow().family_count().to_string())
        .build();
    stats_group.add(&families_row);

    sidebar.append(&stats_group);

    sidebar
}

fn create_font_list(font_manager: Rc<RefCell<FontManager>>) -> Box {
    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    // Toolbar
    let toolbar = Box::new(Orientation::Horizontal, 12);

    // Sort dropdown
    let sort_label = Label::new(Some("Ordenar por:"));
    toolbar.append(&sort_label);

    let sort_options = StringList::new(&["Nome", "Familia", "Estilo", "Data"]);
    let sort_dropdown = DropDown::builder()
        .model(&sort_options)
        .build();
    toolbar.append(&sort_dropdown);

    // Spacer
    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);

    // View mode buttons
    let grid_btn = gtk4::ToggleButton::builder()
        .icon_name("view-grid-symbolic")
        .tooltip_text("Visualizacao em grade")
        .active(true)
        .build();

    let list_btn = gtk4::ToggleButton::builder()
        .icon_name("view-list-symbolic")
        .tooltip_text("Visualizacao em lista")
        .build();

    list_btn.set_group(Some(&grid_btn));

    let view_box = Box::new(Orientation::Horizontal, 0);
    view_box.add_css_class("linked");
    view_box.append(&grid_btn);
    view_box.append(&list_btn);
    toolbar.append(&view_box);

    content.append(&toolbar);

    // Font grid
    let font_grid = gtk4::FlowBox::builder()
        .valign(gtk4::Align::Start)
        .max_children_per_line(4)
        .min_children_per_line(2)
        .selection_mode(SelectionMode::Single)
        .homogeneous(true)
        .row_spacing(12)
        .column_spacing(12)
        .build();

    // Add font cards
    let fonts = font_manager.borrow().get_fonts();
    for font_info in fonts.iter().take(50) {
        let card = FontCard::new(font_info);
        font_grid.append(&card.widget());
    }

    let grid_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .child(&font_grid)
        .build();

    content.append(&grid_scroll);

    content
}
