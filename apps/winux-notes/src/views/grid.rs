// Winux Notes - Grid View (Google Keep style)
// Copyright (c) 2026 Winux OS Project

use crate::data::Note;
use crate::ui::NoteCard;
use gtk4::prelude::*;
use gtk4::{Box, FlowBox, Label, Orientation, Widget};
use std::cell::RefCell;
use std::rc::Rc;

/// Grid view for displaying notes as cards (Google Keep style)
pub struct GridView {
    container: Box,
    pinned_flow: FlowBox,
    others_flow: FlowBox,
    pinned_label: Label,
    others_label: Label,
    notes: Rc<RefCell<Vec<Note>>>,
    on_note_selected: Rc<RefCell<Option<Box<dyn Fn(&Note)>>>>,
}

impl GridView {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .spacing(10)
            .margin_start(15)
            .margin_end(15)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // Pinned section
        let pinned_label = Label::builder()
            .label("Pinned")
            .css_classes(vec!["title-4", "dim-label"])
            .halign(gtk4::Align::Start)
            .margin_top(10)
            .margin_bottom(5)
            .visible(false)
            .build();
        container.append(&pinned_label);

        let pinned_flow = FlowBox::builder()
            .homogeneous(false)
            .column_spacing(12)
            .row_spacing(12)
            .min_children_per_line(1)
            .max_children_per_line(5)
            .selection_mode(gtk4::SelectionMode::Single)
            .visible(false)
            .build();
        container.append(&pinned_flow);

        // Others section
        let others_label = Label::builder()
            .label("Others")
            .css_classes(vec!["title-4", "dim-label"])
            .halign(gtk4::Align::Start)
            .margin_top(15)
            .margin_bottom(5)
            .visible(false)
            .build();
        container.append(&others_label);

        let others_flow = FlowBox::builder()
            .homogeneous(false)
            .column_spacing(12)
            .row_spacing(12)
            .min_children_per_line(1)
            .max_children_per_line(5)
            .selection_mode(gtk4::SelectionMode::Single)
            .vexpand(true)
            .build();
        container.append(&others_flow);

        Self {
            container,
            pinned_flow,
            others_flow,
            pinned_label,
            others_label,
            notes: Rc::new(RefCell::new(Vec::new())),
            on_note_selected: Rc::new(RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }

    pub fn set_notes(&self, notes: &[Note]) {
        // Clear existing cards
        while let Some(child) = self.pinned_flow.first_child() {
            self.pinned_flow.remove(&child);
        }
        while let Some(child) = self.others_flow.first_child() {
            self.others_flow.remove(&child);
        }

        // Store notes
        *self.notes.borrow_mut() = notes.to_vec();

        // Separate pinned and unpinned notes
        let pinned: Vec<_> = notes.iter().filter(|n| n.pinned).collect();
        let others: Vec<_> = notes.iter().filter(|n| !n.pinned).collect();

        // Show/hide pinned section
        let has_pinned = !pinned.is_empty();
        let has_others = !others.is_empty();

        self.pinned_label.set_visible(has_pinned);
        self.pinned_flow.set_visible(has_pinned);
        self.others_label.set_visible(has_pinned && has_others);

        // Create cards for pinned notes
        for note in pinned {
            let card = NoteCard::new(note);
            self.setup_card_click(&card, note);
            self.pinned_flow.insert(&card, -1);
        }

        // Create cards for other notes
        for note in others {
            let card = NoteCard::new(note);
            self.setup_card_click(&card, note);
            self.others_flow.insert(&card, -1);
        }

        // Show empty state if no notes
        if notes.is_empty() {
            let empty_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(10)
                .halign(gtk4::Align::Center)
                .valign(gtk4::Align::Center)
                .margin_top(100)
                .build();

            let icon = gtk4::Image::builder()
                .icon_name("document-new-symbolic")
                .pixel_size(64)
                .css_classes(vec!["dim-label"])
                .build();
            empty_box.append(&icon);

            let label = Label::builder()
                .label("No notes yet")
                .css_classes(vec!["title-2", "dim-label"])
                .build();
            empty_box.append(&label);

            let hint = Label::builder()
                .label("Create a new note to get started")
                .css_classes(vec!["dim-label"])
                .build();
            empty_box.append(&hint);

            self.others_flow.insert(&empty_box, -1);
        }
    }

    fn setup_card_click(&self, card: &NoteCard, note: &Note) {
        let notes = self.notes.clone();
        let note_id = note.id.clone();
        let callback_ref = self.on_note_selected.clone();

        card.connect_clicked(move |_| {
            if let Some(note) = notes.borrow().iter().find(|n| n.id == note_id) {
                if let Some(cb) = callback_ref.borrow().as_ref() {
                    cb(note);
                }
            }
        });
    }

    pub fn on_note_selected<F: Fn(&Note) + 'static>(&self, callback: F) {
        *self.on_note_selected.borrow_mut() = Some(Box::new(callback));
    }
}

impl Default for GridView {
    fn default() -> Self {
        Self::new()
    }
}
