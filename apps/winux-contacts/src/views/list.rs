// Contact List View - Displays list of contacts with search and filtering

use crate::data::{storage::ContactStorage, Contact};
use crate::ui::{avatar::AvatarHelper, contact_row::ContactRow};
use gtk4::prelude::*;
use gtk4::{
    Box, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SelectionMode, Widget,
};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct ContactListView {
    container: Box,
    list_box: ListBox,
    storage: Rc<RefCell<ContactStorage>>,
    contacts: Rc<RefCell<Vec<Contact>>>,
    filtered_indices: Rc<RefCell<Vec<usize>>>,
    on_contact_selected: Rc<RefCell<Option<Box<dyn Fn(Option<String>)>>>>,
}

impl ContactListView {
    pub fn new(storage: Rc<RefCell<ContactStorage>>) -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let list_box = ListBox::new();
        list_box.set_selection_mode(SelectionMode::Single);
        list_box.add_css_class("navigation-sidebar");

        container.append(&list_box);

        let view = Self {
            container,
            list_box,
            storage,
            contacts: Rc::new(RefCell::new(Vec::new())),
            filtered_indices: Rc::new(RefCell::new(Vec::new())),
            on_contact_selected: Rc::new(RefCell::new(None)),
        };

        view.setup_signals();
        view.refresh();

        view
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn refresh(&self) {
        // Load contacts from storage
        if let Ok(storage) = self.storage.try_borrow() {
            if let Ok(contacts) = storage.get_all_contacts() {
                *self.contacts.borrow_mut() = contacts;
            }
        }

        self.rebuild_list();
    }

    pub fn filter(&self, query: &str) {
        let contacts = self.contacts.borrow();
        let mut filtered = Vec::new();

        if query.is_empty() {
            filtered = (0..contacts.len()).collect();
        } else {
            for (idx, contact) in contacts.iter().enumerate() {
                if contact.matches_search(query) {
                    filtered.push(idx);
                }
            }
        }

        *self.filtered_indices.borrow_mut() = filtered;
        self.rebuild_list();
    }

    fn rebuild_list(&self) {
        // Clear existing rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let contacts = self.contacts.borrow();
        let filtered = self.filtered_indices.borrow();

        // If no filter, show all
        let indices: Vec<usize> = if filtered.is_empty() && !contacts.is_empty() {
            (0..contacts.len()).collect()
        } else {
            filtered.clone()
        };

        // Group contacts by first letter
        let mut current_letter: Option<char> = None;

        for &idx in &indices {
            if let Some(contact) = contacts.get(idx) {
                let first_letter = contact
                    .display_name()
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().next().unwrap_or(c))
                    .unwrap_or('#');

                // Add section header if letter changed
                if current_letter != Some(first_letter) {
                    current_letter = Some(first_letter);
                    let header = self.create_section_header(first_letter);
                    self.list_box.append(&header);
                }

                let row = self.create_contact_row(contact, idx);
                self.list_box.append(&row);
            }
        }

        // Show empty state if no contacts
        if indices.is_empty() {
            let empty_row = self.create_empty_row();
            self.list_box.append(&empty_row);
        }
    }

    fn create_section_header(&self, letter: char) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_selectable(false);
        row.set_activatable(false);

        let label = Label::new(Some(&letter.to_string()));
        label.set_halign(gtk4::Align::Start);
        label.set_margin_start(16);
        label.set_margin_top(12);
        label.set_margin_bottom(4);
        label.add_css_class("heading");
        label.add_css_class("dim-label");

        row.set_child(Some(&label));
        row
    }

    fn create_contact_row(&self, contact: &Contact, index: usize) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_name(&contact.id);

        let hbox = Box::new(Orientation::Horizontal, 12);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);

        // Avatar
        let avatar = AvatarHelper::create_avatar(contact, 40);
        hbox.append(&avatar);

        // Name and info
        let vbox = Box::new(Orientation::Vertical, 2);
        vbox.set_valign(gtk4::Align::Center);

        let name_label = Label::new(Some(&contact.display_name()));
        name_label.set_halign(gtk4::Align::Start);
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name_label.add_css_class("heading");
        vbox.append(&name_label);

        // Subtitle (company or primary phone/email)
        let subtitle = if let Some(company) = &contact.company {
            company.clone()
        } else if let Some(phone) = contact.primary_phone() {
            phone.number.clone()
        } else if let Some(email) = contact.primary_email() {
            email.email.clone()
        } else {
            String::new()
        };

        if !subtitle.is_empty() {
            let subtitle_label = Label::new(Some(&subtitle));
            subtitle_label.set_halign(gtk4::Align::Start);
            subtitle_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            subtitle_label.add_css_class("dim-label");
            subtitle_label.add_css_class("caption");
            vbox.append(&subtitle_label);
        }

        hbox.append(&vbox);

        // Favorite indicator
        if contact.is_favorite {
            let star = gtk4::Image::from_icon_name("starred-symbolic");
            star.add_css_class("accent");
            star.set_halign(gtk4::Align::End);
            star.set_hexpand(true);
            hbox.append(&star);
        }

        row.set_child(Some(&hbox));
        row
    }

    fn create_empty_row(&self) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_selectable(false);

        let vbox = Box::new(Orientation::Vertical, 8);
        vbox.set_margin_top(48);
        vbox.set_margin_bottom(48);
        vbox.set_halign(gtk4::Align::Center);

        let icon = gtk4::Image::from_icon_name("system-search-symbolic");
        icon.set_pixel_size(48);
        icon.add_css_class("dim-label");
        vbox.append(&icon);

        let label = Label::new(Some("No contacts found"));
        label.add_css_class("dim-label");
        vbox.append(&label);

        row.set_child(Some(&vbox));
        row
    }

    fn setup_signals(&self) {
        let on_selected = self.on_contact_selected.clone();

        self.list_box.connect_row_selected(move |_, row| {
            if let Some(callback) = on_selected.borrow().as_ref() {
                let id = row.map(|r| r.widget_name().to_string());
                callback(id);
            }
        });
    }

    pub fn connect_contact_selected<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        *self.on_contact_selected.borrow_mut() = Some(Box::new(callback));
    }

    pub fn select_contact(&self, id: &str) {
        let mut row_idx = 0;
        while let Some(row) = self.list_box.row_at_index(row_idx) {
            if row.widget_name() == id {
                self.list_box.select_row(Some(&row));
                break;
            }
            row_idx += 1;
        }
    }
}
