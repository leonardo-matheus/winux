//! Search bar component

use gtk4::prelude::*;
use gtk4::{glib, Entry, Image, Box as GtkBox, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Search bar widget
#[derive(Clone)]
pub struct SearchBar {
    container: GtkBox,
    entry: Entry,
    callbacks: Rc<RefCell<Vec<Box<dyn Fn(&str)>>>>,
}

impl SearchBar {
    /// Create new search bar
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 0);
        container.add_css_class("search-container");

        // Search icon
        let icon = Image::from_icon_name("system-search-symbolic");
        icon.add_css_class("search-icon");
        icon.set_pixel_size(24);
        container.append(&icon);

        // Search entry
        let entry = Entry::builder()
            .placeholder_text("Search apps, files, or type a command...")
            .hexpand(true)
            .build();
        entry.add_css_class("search-entry");
        container.append(&entry);

        let callbacks: Rc<RefCell<Vec<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(Vec::new()));

        // Connect text changed signal
        let callbacks_clone = callbacks.clone();
        entry.connect_changed(move |entry| {
            let text = entry.text();
            for callback in callbacks_clone.borrow().iter() {
                callback(&text);
            }
        });

        Self {
            container,
            entry,
            callbacks,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Focus the search entry
    pub fn focus(&self) {
        self.entry.grab_focus();
    }

    /// Clear the search entry
    pub fn clear(&self) {
        self.entry.set_text("");
    }

    /// Get current text
    pub fn text(&self) -> String {
        self.entry.text().to_string()
    }

    /// Set text
    pub fn set_text(&self, text: &str) {
        self.entry.set_text(text);
    }

    /// Connect to text changed signal
    pub fn connect_changed<F: Fn(&str) + 'static>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Box::new(callback));
    }

    /// Select all text
    pub fn select_all(&self) {
        self.entry.select_region(0, -1);
    }
}

impl Default for SearchBar {
    fn default() -> Self {
        Self::new()
    }
}
