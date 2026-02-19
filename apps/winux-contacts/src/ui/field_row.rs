// Field Row - Displays a contact field with optional action button

use gtk4::prelude::*;
use gtk4::{Box, Button, GestureClick, Label, Orientation, Widget};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct FieldRow {
    container: Box,
    on_action: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_copy: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl FieldRow {
    pub fn new(value: &str, label: &str, action_icon: Option<&str>) -> Self {
        let container = Box::new(Orientation::Horizontal, 12);
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.set_margin_top(8);
        container.set_margin_bottom(8);

        // Value and label
        let text_box = Box::new(Orientation::Vertical, 2);
        text_box.set_hexpand(true);
        text_box.set_valign(gtk4::Align::Center);

        let value_label = Label::new(Some(value));
        value_label.set_halign(gtk4::Align::Start);
        value_label.set_wrap(true);
        value_label.set_selectable(true);
        text_box.append(&value_label);

        let type_label = Label::new(Some(label));
        type_label.set_halign(gtk4::Align::Start);
        type_label.add_css_class("dim-label");
        type_label.add_css_class("caption");
        text_box.append(&type_label);

        container.append(&text_box);

        let on_action: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
        let on_copy: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        // Copy button
        let copy_btn = Button::from_icon_name("edit-copy-symbolic");
        copy_btn.add_css_class("flat");
        copy_btn.add_css_class("circular");
        copy_btn.set_tooltip_text(Some("Copy"));
        copy_btn.set_valign(gtk4::Align::Center);

        let on_copy_clone = on_copy.clone();
        copy_btn.connect_clicked(move |_| {
            if let Some(callback) = on_copy_clone.borrow().as_ref() {
                callback();
            }
        });

        container.append(&copy_btn);

        // Action button (optional)
        if let Some(icon) = action_icon {
            let action_btn = Button::from_icon_name(icon);
            action_btn.add_css_class("flat");
            action_btn.add_css_class("circular");
            action_btn.set_valign(gtk4::Align::Center);

            let on_action_clone = on_action.clone();
            action_btn.connect_clicked(move |_| {
                if let Some(callback) = on_action_clone.borrow().as_ref() {
                    callback();
                }
            });

            container.append(&action_btn);
        }

        Self {
            container,
            on_action,
            on_copy,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn connect_action<F: Fn() + 'static>(&self, callback: F) {
        *self.on_action.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_copy<F: Fn() + 'static>(&self, callback: F) {
        *self.on_copy.borrow_mut() = Some(Box::new(callback));
    }
}

/// A row for displaying a simple label-value pair
pub struct InfoRow {
    container: Box,
}

impl InfoRow {
    pub fn new(label: &str, value: &str) -> Self {
        let container = Box::new(Orientation::Horizontal, 12);
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.set_margin_top(8);
        container.set_margin_bottom(8);

        let label_widget = Label::new(Some(label));
        label_widget.add_css_class("dim-label");
        label_widget.set_width_chars(15);
        label_widget.set_halign(gtk4::Align::Start);
        container.append(&label_widget);

        let value_widget = Label::new(Some(value));
        value_widget.set_halign(gtk4::Align::Start);
        value_widget.set_hexpand(true);
        value_widget.set_selectable(true);
        value_widget.set_wrap(true);
        container.append(&value_widget);

        Self { container }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}

/// An editable field row for the editor view
pub struct EditableFieldRow {
    container: Box,
    entry: gtk4::Entry,
    type_dropdown: gtk4::DropDown,
}

impl EditableFieldRow {
    pub fn new(value: &str, types: &[&str], selected_type: usize) -> Self {
        let container = Box::new(Orientation::Horizontal, 8);

        let entry = gtk4::Entry::new();
        entry.set_text(value);
        entry.set_hexpand(true);
        container.append(&entry);

        let type_strings: Vec<String> = types.iter().map(|s| s.to_string()).collect();
        let type_dropdown = gtk4::DropDown::from_strings(types);
        type_dropdown.set_selected(selected_type as u32);
        container.append(&type_dropdown);

        let remove_btn = Button::from_icon_name("list-remove-symbolic");
        remove_btn.add_css_class("flat");
        remove_btn.add_css_class("circular");
        container.append(&remove_btn);

        Self {
            container,
            entry,
            type_dropdown,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn value(&self) -> String {
        self.entry.text().to_string()
    }

    pub fn selected_type(&self) -> u32 {
        self.type_dropdown.selected()
    }
}
