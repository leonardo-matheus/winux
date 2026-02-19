// Winux Calculator - History Component
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBox, Orientation, ScrolledWindow, SelectionMode};
use libadwaita as adw;
use adw::prelude::*;
use adw::ActionRow;
use crate::engine::{Calculator, HistoryEntry};
use std::collections::VecDeque;

/// Calculator history panel
pub struct History {
    widget: Box,
    list_box: ListBox,
    entries: VecDeque<HistoryEntry>,
}

impl History {
    pub fn new() -> Self {
        let widget = Box::new(Orientation::Vertical, 0);
        widget.set_width_request(250);
        widget.add_css_class("card");
        widget.set_margin_start(6);
        widget.set_margin_end(6);
        widget.set_margin_top(6);
        widget.set_margin_bottom(6);

        // Header
        let header = Box::new(Orientation::Horizontal, 6);
        header.set_margin_top(12);
        header.set_margin_bottom(6);
        header.set_margin_start(12);
        header.set_margin_end(12);

        let title = Label::new(Some("Historico"));
        title.add_css_class("title-4");
        title.set_hexpand(true);
        title.set_halign(gtk4::Align::Start);
        header.append(&title);

        let clear_btn = Button::from_icon_name("edit-clear-all-symbolic");
        clear_btn.set_tooltip_text(Some("Limpar Historico"));
        clear_btn.add_css_class("flat");
        header.append(&clear_btn);

        widget.append(&header);

        // Separator
        widget.append(&gtk4::Separator::new(Orientation::Horizontal));

        // List of history entries
        let list_box = ListBox::new();
        list_box.set_selection_mode(SelectionMode::None);
        list_box.add_css_class("boxed-list");
        list_box.set_margin_top(6);
        list_box.set_margin_bottom(6);
        list_box.set_margin_start(6);
        list_box.set_margin_end(6);

        // Placeholder when empty
        list_box.set_placeholder(Some(&Self::create_placeholder()));

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&list_box)
            .vexpand(true)
            .build();

        widget.append(&scrolled);

        // Clear button action
        let list_clone = list_box.clone();
        clear_btn.connect_clicked(move |_| {
            // Remove all children
            while let Some(child) = list_clone.first_child() {
                list_clone.remove(&child);
            }
        });

        Self {
            widget,
            list_box,
            entries: VecDeque::new(),
        }
    }

    pub fn widget(&self) -> Box {
        self.widget.clone()
    }

    fn create_placeholder() -> Label {
        let label = Label::new(Some("Nenhum calculo ainda"));
        label.add_css_class("dim-label");
        label.set_margin_top(24);
        label.set_margin_bottom(24);
        label
    }

    /// Update history from calculator
    pub fn update(&mut self, calc: &Calculator) {
        // Sync entries from calculator
        for entry in calc.history.iter().take(1) {
            self.add_entry(entry);
        }
    }

    /// Add a history entry
    pub fn add_entry(&mut self, entry: &HistoryEntry) {
        // Check if this entry already exists
        if self.entries.front() == Some(entry) {
            return;
        }

        self.entries.push_front(entry.clone());

        // Create row for the entry
        let row = ActionRow::builder()
            .title(&entry.result)
            .subtitle(&entry.expression)
            .activatable(true)
            .build();

        // Copy button
        let copy_btn = Button::from_icon_name("edit-copy-symbolic");
        copy_btn.set_tooltip_text(Some("Copiar resultado"));
        copy_btn.add_css_class("flat");
        copy_btn.set_valign(gtk4::Align::Center);

        let result = entry.result.clone();
        copy_btn.connect_clicked(move |btn| {
            if let Some(display) = btn.display() {
                let clipboard = display.clipboard();
                clipboard.set_text(&result);
            }
        });

        row.add_suffix(&copy_btn);

        // Insert at the beginning
        self.list_box.prepend(&row);

        // Limit history display
        let max_visible = 50;
        let mut count = 0;
        let mut child = self.list_box.first_child();
        while let Some(widget) = child {
            count += 1;
            let next = widget.next_sibling();
            if count > max_visible {
                self.list_box.remove(&widget);
            }
            child = next;
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
    }

    /// Get all entries
    pub fn get_entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for HistoryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression && self.result == other.result
    }
}
