// Winux Notes - List View
// Copyright (c) 2026 Winux OS Project

use crate::data::Note;
use gtk4::prelude::*;
use gtk4::{Box, Image, Label, ListBox, ListBoxRow, Orientation, Widget};
use std::cell::RefCell;
use std::rc::Rc;

/// List view for displaying notes in a vertical list
pub struct ListView {
    container: Box,
    list_box: ListBox,
    notes: Rc<RefCell<Vec<Note>>>,
    on_note_selected: Rc<RefCell<Option<Box<dyn Fn(&Note)>>>>,
}

impl ListView {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(vec!["boxed-list"])
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        container.append(&list_box);

        Self {
            container,
            list_box,
            notes: Rc::new(RefCell::new(Vec::new())),
            on_note_selected: Rc::new(RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }

    pub fn set_notes(&self, notes: &[Note]) {
        // Clear existing rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        // Store notes
        *self.notes.borrow_mut() = notes.to_vec();

        // Create rows for each note
        for note in notes {
            let row = self.create_note_row(note);
            self.list_box.append(&row);
        }

        // Show empty state if no notes
        if notes.is_empty() {
            let empty_label = Label::builder()
                .label("No notes yet")
                .css_classes(vec!["dim-label"])
                .margin_top(50)
                .margin_bottom(50)
                .build();
            self.list_box.append(&ListBoxRow::builder().child(&empty_label).selectable(false).build());
        }
    }

    fn create_note_row(&self, note: &Note) -> ListBoxRow {
        let row_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // Color indicator
        let color_box = Box::builder()
            .width_request(4)
            .height_request(40)
            .css_classes(vec![note.color.to_css_class()])
            .build();
        row_box.append(&color_box);

        // Note content
        let content_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .build();

        // Title row with pin icon
        let title_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();

        if note.pinned {
            let pin_icon = Image::from_icon_name("view-pin-symbolic");
            pin_icon.add_css_class("dim-label");
            title_row.append(&pin_icon);
        }

        let title_label = Label::builder()
            .label(&note.title)
            .css_classes(vec!["heading"])
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .hexpand(true)
            .build();
        title_row.append(&title_label);

        if note.favorite {
            let star_icon = Image::from_icon_name("starred-symbolic");
            star_icon.add_css_class("accent");
            title_row.append(&star_icon);
        }

        content_box.append(&title_row);

        // Preview
        let preview = note.content_preview(100);
        if !preview.is_empty() {
            let preview_label = Label::builder()
                .label(&preview)
                .css_classes(vec!["dim-label", "caption"])
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .max_width_chars(60)
                .build();
            content_box.append(&preview_label);
        }

        // Tags
        if !note.tags.is_empty() {
            let tags_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(4)
                .margin_top(4)
                .build();

            for tag in note.tags.iter().take(3) {
                let tag_label = Label::builder()
                    .label(tag)
                    .css_classes(vec!["tag-badge"])
                    .build();
                tags_box.append(&tag_label);
            }

            if note.tags.len() > 3 {
                let more_label = Label::builder()
                    .label(&format!("+{}", note.tags.len() - 3))
                    .css_classes(vec!["dim-label", "caption"])
                    .build();
                tags_box.append(&more_label);
            }

            content_box.append(&tags_box);
        }

        row_box.append(&content_box);

        // Date
        let date_label = Label::builder()
            .label(&note.relative_time())
            .css_classes(vec!["dim-label", "caption"])
            .valign(gtk4::Align::Start)
            .build();
        row_box.append(&date_label);

        let row = ListBoxRow::builder()
            .child(&row_box)
            .build();

        row.set_widget_name(&note.id);

        row
    }

    pub fn on_note_selected<F: Fn(&Note) + 'static>(&self, callback: F) {
        *self.on_note_selected.borrow_mut() = Some(Box::new(callback));

        let notes = self.notes.clone();
        let callback_ref = self.on_note_selected.clone();

        self.list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let note_id = row.widget_name();
                if let Some(note) = notes.borrow().iter().find(|n| n.id == note_id.as_str()) {
                    if let Some(cb) = callback_ref.borrow().as_ref() {
                        cb(note);
                    }
                }
            }
        });
    }

    pub fn select_note(&self, note_id: &str) {
        let mut index = 0;
        while let Some(row) = self.list_box.row_at_index(index) {
            if row.widget_name() == note_id {
                self.list_box.select_row(Some(&row));
                break;
            }
            index += 1;
        }
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}
