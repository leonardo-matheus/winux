// Winux Notes - Checklist Editor
// Copyright (c) 2026 Winux OS Project

use crate::data::ChecklistItem;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, Entry, Image, ListBox, ListBoxRow, Orientation, Widget,
};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

/// Checklist editor for todo items
pub struct ChecklistEditor {
    container: Box,
    list_box: ListBox,
    items: Rc<RefCell<Vec<ChecklistItem>>>,
    on_changed: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl ChecklistEditor {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_start(15)
            .margin_end(15)
            .margin_top(10)
            .margin_bottom(10)
            .hexpand(true)
            .vexpand(true)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(vec!["boxed-list"])
            .build();
        container.append(&list_box);

        // Add new item button
        let add_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_top(10)
            .build();

        let add_btn = Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(vec!["flat"])
            .build();
        add_box.append(&add_btn);

        let add_label = Button::builder()
            .label("Add item")
            .css_classes(vec!["flat"])
            .build();
        add_box.append(&add_label);

        container.append(&add_box);

        let items: Rc<RefCell<Vec<ChecklistItem>>> = Rc::new(RefCell::new(Vec::new()));
        let on_changed: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        let editor = Self {
            container,
            list_box,
            items,
            on_changed,
        };

        // Setup add button handlers
        let editor_clone = editor.clone_inner();
        add_btn.connect_clicked(move |_| {
            editor_clone.add_item("");
        });

        let editor_clone = editor.clone_inner();
        add_label.connect_clicked(move |_| {
            editor_clone.add_item("");
        });

        editor
    }

    fn clone_inner(&self) -> ChecklistEditorInner {
        ChecklistEditorInner {
            list_box: self.list_box.clone(),
            items: self.items.clone(),
            on_changed: self.on_changed.clone(),
        }
    }

    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }

    pub fn set_items(&self, items: &[ChecklistItem]) {
        // Clear existing
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        *self.items.borrow_mut() = items.to_vec();

        // Add rows
        for item in items {
            self.add_item_row(item);
        }
    }

    pub fn get_items(&self) -> Vec<ChecklistItem> {
        self.items.borrow().clone()
    }

    pub fn add_item(&self, text: &str) {
        let item = ChecklistItem {
            id: Uuid::new_v4().to_string(),
            text: text.to_string(),
            checked: false,
        };

        self.items.borrow_mut().push(item.clone());
        self.add_item_row(&item);
        self.notify_changed();
    }

    fn add_item_row(&self, item: &ChecklistItem) {
        let row = self.create_item_row(item);
        self.list_box.append(&row);
    }

    fn create_item_row(&self, item: &ChecklistItem) -> ListBoxRow {
        let row_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        // Checkbox
        let check = CheckButton::builder()
            .active(item.checked)
            .build();
        row_box.append(&check);

        // Text entry
        let entry = Entry::builder()
            .text(&item.text)
            .hexpand(true)
            .css_classes(vec!["flat"])
            .placeholder_text("New item...")
            .build();

        // Strike through if checked
        if item.checked {
            entry.add_css_class("dim-label");
        }

        row_box.append(&entry);

        // Delete button
        let delete_btn = Button::builder()
            .icon_name("edit-delete-symbolic")
            .css_classes(vec!["flat", "circular", "destructive-action"])
            .tooltip_text("Delete item")
            .build();
        delete_btn.set_visible(false);
        row_box.append(&delete_btn);

        let row = ListBoxRow::builder()
            .child(&row_box)
            .build();
        row.set_widget_name(&item.id);

        // Setup handlers
        let items = self.items.clone();
        let item_id = item.id.clone();
        let entry_clone = entry.clone();
        let on_changed = self.on_changed.clone();

        check.connect_toggled(move |check| {
            let mut items = items.borrow_mut();
            if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.checked = check.is_active();
                if item.checked {
                    entry_clone.add_css_class("dim-label");
                } else {
                    entry_clone.remove_css_class("dim-label");
                }
            }
            if let Some(cb) = on_changed.borrow().as_ref() {
                cb();
            }
        });

        let items = self.items.clone();
        let item_id = item.id.clone();
        let on_changed = self.on_changed.clone();

        entry.connect_changed(move |entry| {
            let mut items = items.borrow_mut();
            if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.text = entry.text().to_string();
            }
            if let Some(cb) = on_changed.borrow().as_ref() {
                cb();
            }
        });

        // Show delete button on focus
        let delete_btn_clone = delete_btn.clone();
        entry.connect_has_focus_notify(move |entry| {
            delete_btn_clone.set_visible(entry.has_focus());
        });

        let items = self.items.clone();
        let item_id = item.id.clone();
        let list_box = self.list_box.clone();
        let row_clone = row.clone();
        let on_changed = self.on_changed.clone();

        delete_btn.connect_clicked(move |_| {
            items.borrow_mut().retain(|i| i.id != item_id);
            list_box.remove(&row_clone);
            if let Some(cb) = on_changed.borrow().as_ref() {
                cb();
            }
        });

        row
    }

    pub fn clear(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        self.items.borrow_mut().clear();
    }

    pub fn on_changed<F: Fn() + 'static>(&self, callback: F) {
        *self.on_changed.borrow_mut() = Some(Box::new(callback));
    }

    fn notify_changed(&self) {
        if let Some(cb) = self.on_changed.borrow().as_ref() {
            cb();
        }
    }

    /// Check if all items are completed
    pub fn all_completed(&self) -> bool {
        let items = self.items.borrow();
        !items.is_empty() && items.iter().all(|i| i.checked)
    }

    /// Get completion stats (completed, total)
    pub fn completion_stats(&self) -> (usize, usize) {
        let items = self.items.borrow();
        let total = items.len();
        let completed = items.iter().filter(|i| i.checked).count();
        (completed, total)
    }
}

/// Inner helper for callbacks
struct ChecklistEditorInner {
    list_box: ListBox,
    items: Rc<RefCell<Vec<ChecklistItem>>>,
    on_changed: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl ChecklistEditorInner {
    fn add_item(&self, text: &str) {
        let item = ChecklistItem {
            id: Uuid::new_v4().to_string(),
            text: text.to_string(),
            checked: false,
        };

        self.items.borrow_mut().push(item.clone());

        // Create and add row
        let row_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        let check = CheckButton::builder()
            .active(false)
            .build();
        row_box.append(&check);

        let entry = Entry::builder()
            .hexpand(true)
            .css_classes(vec!["flat"])
            .placeholder_text("New item...")
            .build();
        row_box.append(&entry);

        let delete_btn = Button::builder()
            .icon_name("edit-delete-symbolic")
            .css_classes(vec!["flat", "circular", "destructive-action"])
            .tooltip_text("Delete item")
            .visible(false)
            .build();
        row_box.append(&delete_btn);

        let row = ListBoxRow::builder()
            .child(&row_box)
            .build();
        row.set_widget_name(&item.id);

        // Setup handlers
        let items = self.items.clone();
        let item_id = item.id.clone();
        let entry_clone = entry.clone();
        let on_changed = self.on_changed.clone();

        check.connect_toggled(move |check| {
            let mut items = items.borrow_mut();
            if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.checked = check.is_active();
                if item.checked {
                    entry_clone.add_css_class("dim-label");
                } else {
                    entry_clone.remove_css_class("dim-label");
                }
            }
            if let Some(cb) = on_changed.borrow().as_ref() {
                cb();
            }
        });

        let items = self.items.clone();
        let item_id = item.id.clone();
        let on_changed = self.on_changed.clone();

        entry.connect_changed(move |entry| {
            let mut items = items.borrow_mut();
            if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.text = entry.text().to_string();
            }
            if let Some(cb) = on_changed.borrow().as_ref() {
                cb();
            }
        });

        let delete_btn_clone = delete_btn.clone();
        entry.connect_has_focus_notify(move |entry| {
            delete_btn_clone.set_visible(entry.has_focus());
        });

        let items = self.items.clone();
        let item_id = item.id.clone();
        let list_box = self.list_box.clone();
        let row_clone = row.clone();
        let on_changed = self.on_changed.clone();

        delete_btn.connect_clicked(move |_| {
            items.borrow_mut().retain(|i| i.id != item_id);
            list_box.remove(&row_clone);
            if let Some(cb) = on_changed.borrow().as_ref() {
                cb();
            }
        });

        self.list_box.append(&row);
        entry.grab_focus();

        // Notify change
        if let Some(cb) = self.on_changed.borrow().as_ref() {
            cb();
        }
    }
}

impl Default for ChecklistEditor {
    fn default() -> Self {
        Self::new()
    }
}
