// Winux Logs - Log View Component
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    ListBox, ScrolledWindow, PolicyType, SelectionMode, Label, Box, Orientation,
    Align, Separator, Frame,
};
use libadwaita as adw;
use adw::prelude::*;

use crate::sources::LogEntry;
use crate::ui::log_row;

/// Create the main log list view
pub fn create_log_view() -> ListBox {
    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::Single);
    list.add_css_class("boxed-list");

    // Add placeholder
    let placeholder = create_placeholder();
    list.set_placeholder(Some(&placeholder));

    list
}

/// Create a scrollable log view with header
pub fn create_scrollable_log_view() -> (ScrolledWindow, ListBox) {
    let list = create_log_view();

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&list)
        .vexpand(true)
        .hexpand(true)
        .build();

    (scroll, list)
}

/// Create header row for the log list
pub fn create_list_header() -> Box {
    let header_box = Box::new(Orientation::Horizontal, 8);
    header_box.set_margin_start(12);
    header_box.set_margin_end(12);
    header_box.set_margin_top(8);
    header_box.set_margin_bottom(8);
    header_box.add_css_class("dim-label");

    let time_label = Label::new(Some("Hora"));
    time_label.set_width_request(150);
    time_label.set_xalign(0.0);

    let level_label = Label::new(Some("Nivel"));
    level_label.set_width_request(80);
    level_label.set_xalign(0.0);

    let unit_label = Label::new(Some("Unit"));
    unit_label.set_width_request(180);
    unit_label.set_xalign(0.0);

    let message_label = Label::new(Some("Mensagem"));
    message_label.set_hexpand(true);
    message_label.set_xalign(0.0);

    header_box.append(&time_label);
    header_box.append(&Separator::new(Orientation::Vertical));
    header_box.append(&level_label);
    header_box.append(&Separator::new(Orientation::Vertical));
    header_box.append(&unit_label);
    header_box.append(&Separator::new(Orientation::Vertical));
    header_box.append(&message_label);

    header_box
}

/// Create placeholder for empty list
fn create_placeholder() -> Box {
    let placeholder = Box::new(Orientation::Vertical, 12);
    placeholder.set_valign(Align::Center);
    placeholder.set_halign(Align::Center);
    placeholder.set_margin_top(48);
    placeholder.set_margin_bottom(48);

    let icon = gtk4::Image::from_icon_name("text-x-log-symbolic");
    icon.set_pixel_size(64);
    icon.add_css_class("dim-label");

    let label = Label::new(Some("Nenhum log encontrado"));
    label.add_css_class("title-2");
    label.add_css_class("dim-label");

    let description = Label::new(Some("Ajuste os filtros ou selecione outra fonte"));
    description.add_css_class("dim-label");

    placeholder.append(&icon);
    placeholder.append(&label);
    placeholder.append(&description);

    placeholder
}

/// Populate the log list with entries
pub fn populate_log_list(list: &ListBox, entries: &[LogEntry]) {
    // Clear existing entries
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    // Add entries
    for entry in entries {
        let row = log_row::create_log_row(entry);
        list.append(&row);
    }
}

/// Add a single entry to the list (for live tail)
pub fn append_log_entry(list: &ListBox, entry: &LogEntry) {
    let row = log_row::create_log_row(entry);
    list.prepend(&row); // Prepend to show newest first
}

/// Get selected entry index
pub fn get_selected_index(list: &ListBox) -> Option<i32> {
    list.selected_row().map(|row| row.index())
}

/// Select row by index
pub fn select_row_at_index(list: &ListBox, index: i32) {
    if let Some(row) = list.row_at_index(index) {
        list.select_row(Some(&row));
    }
}

/// Scroll to show the selected row
pub fn scroll_to_selected(list: &ListBox, scroll: &ScrolledWindow) {
    if let Some(row) = list.selected_row() {
        let allocation = row.allocation();
        if let Some(adjustment) = scroll.vadjustment() {
            adjustment.set_value(allocation.y() as f64);
        }
    }
}

/// Get count of visible rows
pub fn get_row_count(list: &ListBox) -> i32 {
    let mut count = 0;
    let mut idx = 0;
    while list.row_at_index(idx).is_some() {
        count += 1;
        idx += 1;
    }
    count
}

/// Create a statistics bar showing log counts
pub fn create_stats_bar(entries: &[LogEntry]) -> Box {
    use crate::filters::level::count_by_level;

    let counts = count_by_level(entries);

    let stats_box = Box::new(Orientation::Horizontal, 16);
    stats_box.set_margin_start(12);
    stats_box.set_margin_end(12);
    stats_box.set_margin_top(8);
    stats_box.set_margin_bottom(8);

    let total_label = create_stat_pill("Total", counts.total(), "dim-label");
    let error_label = create_stat_pill("Erros", counts.errors_total(), "error");
    let warning_label = create_stat_pill("Avisos", counts.warnings_total(), "warning");

    stats_box.append(&total_label);
    stats_box.append(&error_label);
    stats_box.append(&warning_label);

    stats_box
}

fn create_stat_pill(label: &str, count: usize, css_class: &str) -> Box {
    let pill = Box::new(Orientation::Horizontal, 4);
    pill.add_css_class("pill");

    let name = Label::new(Some(label));
    name.add_css_class("dim-label");

    let value = Label::new(Some(&count.to_string()));
    value.add_css_class(css_class);
    value.add_css_class("numeric");

    pill.append(&name);
    pill.append(&value);

    pill
}

/// Framed log view with title
pub fn create_framed_log_view(title: &str) -> (Frame, ListBox) {
    let (scroll, list) = create_scrollable_log_view();

    let frame = Frame::builder()
        .label(title)
        .child(&scroll)
        .build();

    (frame, list)
}
