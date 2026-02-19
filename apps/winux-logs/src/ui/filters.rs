// Winux Logs - Filters UI Component
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    Box, Label, Orientation, CheckButton, Button, Entry, ComboBoxText,
    Calendar, Popover, ScrolledWindow, PolicyType, FlowBox, FlowBoxChild,
    SelectionMode, Separator, ToggleButton, MenuButton,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup};

use crate::sources::LogLevel;
use crate::filters::FilterState;

/// Create the filters panel for the sidebar
pub fn create_filters_panel() -> Box {
    let panel = Box::new(Orientation::Vertical, 8);
    panel.set_margin_start(8);
    panel.set_margin_end(8);
    panel.set_margin_top(8);
    panel.set_margin_bottom(8);

    // Level filter section
    let level_section = create_level_filter();
    panel.append(&level_section);

    // Time filter section
    let time_section = create_time_filter();
    panel.append(&time_section);

    // Quick filter buttons
    let quick_filters = create_quick_filters();
    panel.append(&quick_filters);

    // Clear filters button
    let clear_button = Button::with_label("Limpar Filtros");
    clear_button.add_css_class("flat");
    clear_button.set_margin_top(8);

    panel.append(&clear_button);

    panel
}

/// Create level filter section
fn create_level_filter() -> ExpanderRow {
    let expander = ExpanderRow::builder()
        .title("Nivel")
        .subtitle("Filtrar por severidade")
        .expanded(true)
        .build();

    let levels_box = Box::new(Orientation::Vertical, 4);
    levels_box.set_margin_start(8);
    levels_box.set_margin_end(8);
    levels_box.set_margin_top(4);
    levels_box.set_margin_bottom(4);

    let levels = [
        (LogLevel::Emergency, "emergency"),
        (LogLevel::Alert, "alert"),
        (LogLevel::Critical, "critical"),
        (LogLevel::Error, "error"),
        (LogLevel::Warning, "warning"),
        (LogLevel::Notice, "notice"),
        (LogLevel::Info, "info"),
        (LogLevel::Debug, "debug"),
    ];

    for (level, _id) in levels {
        let check = CheckButton::builder()
            .label(level.display_name())
            .active(true)
            .build();

        // Add level-specific styling
        check.add_css_class(crate::ui::level_css_class(&level));

        levels_box.append(&check);
    }

    // Add preset buttons
    let presets_box = Box::new(Orientation::Horizontal, 4);
    presets_box.set_margin_top(8);

    let errors_btn = Button::with_label("Erros");
    errors_btn.add_css_class("pill");
    errors_btn.add_css_class("error");

    let warnings_btn = Button::with_label("Avisos+");
    warnings_btn.add_css_class("pill");
    warnings_btn.add_css_class("warning");

    let all_btn = Button::with_label("Todos");
    all_btn.add_css_class("pill");

    presets_box.append(&errors_btn);
    presets_box.append(&warnings_btn);
    presets_box.append(&all_btn);

    levels_box.append(&presets_box);

    let row = ActionRow::new();
    row.set_child(Some(&levels_box));
    expander.add_row(&row);

    expander
}

/// Create time filter section
fn create_time_filter() -> ExpanderRow {
    let expander = ExpanderRow::builder()
        .title("Periodo")
        .subtitle("Filtrar por tempo")
        .expanded(false)
        .build();

    let content = Box::new(Orientation::Vertical, 8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);

    // Quick time presets
    let presets_label = Label::new(Some("Rapido:"));
    presets_label.set_xalign(0.0);
    presets_label.add_css_class("dim-label");

    let presets_box = FlowBox::new();
    presets_box.set_selection_mode(SelectionMode::Single);
    presets_box.set_max_children_per_line(3);
    presets_box.set_row_spacing(4);
    presets_box.set_column_spacing(4);

    let preset_names = [
        "15 min", "1 hora", "6 horas", "24 horas", "Hoje", "Ontem",
    ];

    for name in preset_names {
        let btn = ToggleButton::with_label(name);
        btn.add_css_class("pill");
        btn.add_css_class("small");
        presets_box.append(&btn);
    }

    content.append(&presets_label);
    content.append(&presets_box);

    // Custom range
    content.append(&Separator::new(Orientation::Horizontal));

    let custom_label = Label::new(Some("Personalizado:"));
    custom_label.set_xalign(0.0);
    custom_label.add_css_class("dim-label");
    custom_label.set_margin_top(8);
    content.append(&custom_label);

    // From
    let from_box = Box::new(Orientation::Horizontal, 8);
    let from_label = Label::new(Some("De:"));
    from_label.set_width_request(40);

    let from_entry = Entry::builder()
        .placeholder_text("AAAA-MM-DD HH:MM")
        .build();

    let from_calendar_btn = MenuButton::builder()
        .icon_name("x-office-calendar-symbolic")
        .build();

    let from_calendar = create_calendar_popover();
    from_calendar_btn.set_popover(Some(&from_calendar));

    from_box.append(&from_label);
    from_box.append(&from_entry);
    from_box.append(&from_calendar_btn);

    content.append(&from_box);

    // To
    let to_box = Box::new(Orientation::Horizontal, 8);
    let to_label = Label::new(Some("Ate:"));
    to_label.set_width_request(40);

    let to_entry = Entry::builder()
        .placeholder_text("AAAA-MM-DD HH:MM")
        .build();

    let to_calendar_btn = MenuButton::builder()
        .icon_name("x-office-calendar-symbolic")
        .build();

    let to_calendar = create_calendar_popover();
    to_calendar_btn.set_popover(Some(&to_calendar));

    to_box.append(&to_label);
    to_box.append(&to_entry);
    to_box.append(&to_calendar_btn);

    content.append(&to_box);

    // Apply button
    let apply_btn = Button::with_label("Aplicar");
    apply_btn.add_css_class("suggested-action");
    apply_btn.add_css_class("pill");
    apply_btn.set_halign(gtk4::Align::End);
    apply_btn.set_margin_top(8);
    content.append(&apply_btn);

    let row = ActionRow::new();
    row.set_child(Some(&content));
    expander.add_row(&row);

    expander
}

/// Create calendar popover for date selection
fn create_calendar_popover() -> Popover {
    let calendar = Calendar::new();
    calendar.set_margin_start(8);
    calendar.set_margin_end(8);
    calendar.set_margin_top(8);
    calendar.set_margin_bottom(8);

    let popover = Popover::new();
    popover.set_child(Some(&calendar));

    popover
}

/// Create quick filter buttons
fn create_quick_filters() -> Box {
    let quick_box = Box::new(Orientation::Vertical, 4);

    let label = Label::new(Some("Filtros Rapidos"));
    label.set_xalign(0.0);
    label.add_css_class("dim-label");
    label.add_css_class("caption");

    let buttons_box = FlowBox::new();
    buttons_box.set_selection_mode(SelectionMode::None);
    buttons_box.set_max_children_per_line(2);
    buttons_box.set_row_spacing(4);
    buttons_box.set_column_spacing(4);

    let quick_filters = [
        ("Erros de Autenticacao", "auth.*fail|denied"),
        ("Rede", "network|connection"),
        ("Disco", "disk|storage|mount"),
        ("Kernel Panic", "panic|oops|BUG"),
    ];

    for (name, _pattern) in quick_filters {
        let btn = Button::with_label(name);
        btn.add_css_class("flat");
        btn.add_css_class("small");
        buttons_box.append(&btn);
    }

    quick_box.append(&label);
    quick_box.append(&buttons_box);

    quick_box
}

/// Create a unit filter dropdown
pub fn create_unit_filter(units: &[String]) -> ComboBoxText {
    let combo = ComboBoxText::new();
    combo.append(Some("all"), "Todas as Units");

    for unit in units {
        combo.append(Some(unit), unit);
    }

    combo.set_active_id(Some("all"));
    combo
}

/// Create active filters display (chips)
pub fn create_active_filters_bar(filter_state: &FilterState) -> Box {
    let bar = Box::new(Orientation::Horizontal, 4);
    bar.add_css_class("toolbar");
    bar.set_margin_start(8);
    bar.set_margin_end(8);
    bar.set_margin_top(4);
    bar.set_margin_bottom(4);

    if !filter_state.is_active() {
        return bar;
    }

    let label = Label::new(Some("Filtros:"));
    label.add_css_class("dim-label");
    bar.append(&label);

    // Level filters
    for level in &filter_state.levels {
        let chip = create_filter_chip(level.display_name(), crate::ui::level_css_class(level));
        bar.append(&chip);
    }

    // Unit filters
    for unit in &filter_state.units {
        let chip = create_filter_chip(unit, "accent");
        bar.append(&chip);
    }

    // Time filter
    if filter_state.time_from.is_some() || filter_state.time_to.is_some() {
        let time_text = if let (Some(from), Some(to)) = (&filter_state.time_from, &filter_state.time_to) {
            format!("{} - {}", from.format("%H:%M"), to.format("%H:%M"))
        } else if let Some(from) = &filter_state.time_from {
            format!("Desde {}", from.format("%H:%M"))
        } else if let Some(to) = &filter_state.time_to {
            format!("Ate {}", to.format("%H:%M"))
        } else {
            String::new()
        };

        if !time_text.is_empty() {
            let chip = create_filter_chip(&time_text, "");
            bar.append(&chip);
        }
    }

    // Search filter
    if let Some(ref query) = filter_state.search_query {
        if !query.is_empty() {
            let chip = create_filter_chip(&format!("\"{}\"", query), "");
            bar.append(&chip);
        }
    }

    bar
}

/// Create a filter chip widget
fn create_filter_chip(text: &str, css_class: &str) -> Box {
    let chip = Box::new(Orientation::Horizontal, 4);
    chip.add_css_class("filter-chip");
    chip.add_css_class("pill");
    if !css_class.is_empty() {
        chip.add_css_class(css_class);
    }

    let label = Label::new(Some(text));
    label.add_css_class("caption");

    let remove_btn = Button::builder()
        .icon_name("window-close-symbolic")
        .build();
    remove_btn.add_css_class("flat");
    remove_btn.add_css_class("circular");

    chip.append(&label);
    chip.append(&remove_btn);

    chip
}

/// Create search options dropdown
pub fn create_search_options() -> Box {
    let options = Box::new(Orientation::Vertical, 8);
    options.set_margin_start(12);
    options.set_margin_end(12);
    options.set_margin_top(8);
    options.set_margin_bottom(8);

    let regex_check = CheckButton::builder()
        .label("Usar expressao regular")
        .build();

    let case_check = CheckButton::builder()
        .label("Diferencia maiusculas/minusculas")
        .build();

    let fields_check = CheckButton::builder()
        .label("Buscar em todos os campos")
        .build();

    options.append(&regex_check);
    options.append(&case_check);
    options.append(&fields_check);

    options
}

/// Create boot filter selector
pub fn create_boot_filter() -> Box {
    let boot_box = Box::new(Orientation::Horizontal, 8);

    let label = Label::new(Some("Boot:"));
    label.add_css_class("dim-label");

    let combo = ComboBoxText::new();
    combo.append(Some("0"), "Atual");
    combo.append(Some("-1"), "Anterior");
    combo.append(Some("-2"), "2 atras");
    combo.append(Some("-3"), "3 atras");
    combo.append(Some("all"), "Todos");
    combo.set_active_id(Some("0"));

    boot_box.append(&label);
    boot_box.append(&combo);

    boot_box
}
