//! Path bar for navigation within archives

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Label, Orientation, Separator};
use std::cell::RefCell;
use std::rc::Rc;

/// Path bar component for archive navigation
#[derive(Clone)]
pub struct PathBar {
    container: GtkBox,
    path_box: GtkBox,
    archive_name: Rc<RefCell<String>>,
    current_path: Rc<RefCell<String>>,
    on_navigate: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl PathBar {
    /// Create a new path bar
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 0);
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.set_margin_top(4);
        container.set_margin_bottom(4);
        container.add_css_class("toolbar");

        // Home/archive button
        let home_btn = Button::builder()
            .icon_name("go-home-symbolic")
            .tooltip_text("Go to archive root")
            .build();
        home_btn.set_has_frame(false);

        // Path breadcrumb box
        let path_box = GtkBox::new(Orientation::Horizontal, 4);
        path_box.set_hexpand(true);

        // Up button
        let up_btn = Button::builder()
            .icon_name("go-up-symbolic")
            .tooltip_text("Go up one level")
            .build();
        up_btn.set_has_frame(false);

        container.append(&home_btn);
        container.append(&Separator::new(Orientation::Vertical));
        container.append(&path_box);
        container.append(&up_btn);

        let path_bar = Self {
            container,
            path_box,
            archive_name: Rc::new(RefCell::new(String::new())),
            current_path: Rc::new(RefCell::new(String::new())),
            on_navigate: Rc::new(RefCell::new(None)),
        };

        // Connect home button
        let path_bar_clone = path_bar.clone();
        home_btn.connect_clicked(move |_| {
            path_bar_clone.navigate_to("");
        });

        // Connect up button
        let path_bar_clone = path_bar.clone();
        up_btn.connect_clicked(move |_| {
            let current = path_bar_clone.current_path.borrow().clone();
            if let Some(parent) = current.rsplit_once('/') {
                path_bar_clone.navigate_to(parent.0);
            } else if !current.is_empty() {
                path_bar_clone.navigate_to("");
            }
        });

        path_bar
    }

    /// Get the widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Set the current path
    pub fn set_path(&self, archive_name: &str, path: &str) {
        *self.archive_name.borrow_mut() = archive_name.to_string();
        *self.current_path.borrow_mut() = path.to_string();

        self.update_breadcrumbs();
    }

    /// Update breadcrumb display
    fn update_breadcrumbs(&self) {
        // Clear existing breadcrumbs
        while let Some(child) = self.path_box.first_child() {
            self.path_box.remove(&child);
        }

        let archive_name = self.archive_name.borrow().clone();
        let current_path = self.current_path.borrow().clone();

        // Archive icon and name (root)
        let archive_btn = Button::new();
        let archive_box = GtkBox::new(Orientation::Horizontal, 4);

        let archive_icon = Image::from_icon_name("package-x-generic-symbolic");
        let archive_label = Label::new(Some(&archive_name));
        archive_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        archive_label.set_max_width_chars(30);

        archive_box.append(&archive_icon);
        archive_box.append(&archive_label);
        archive_btn.set_child(Some(&archive_box));
        archive_btn.set_has_frame(false);

        let self_clone = self.clone();
        archive_btn.connect_clicked(move |_| {
            self_clone.navigate_to("");
        });

        self.path_box.append(&archive_btn);

        // Path parts
        if !current_path.is_empty() {
            let parts: Vec<&str> = current_path.split('/').collect();
            let mut accumulated_path = String::new();

            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }

                // Add separator
                let sep = Label::new(Some("/"));
                sep.add_css_class("dim-label");
                self.path_box.append(&sep);

                // Accumulate path
                if !accumulated_path.is_empty() {
                    accumulated_path.push('/');
                }
                accumulated_path.push_str(part);

                // Create button for this path part
                let btn = Button::with_label(part);
                btn.set_has_frame(false);

                // Only make clickable if not the last part
                if i < parts.len() - 1 {
                    let path = accumulated_path.clone();
                    let self_clone = self.clone();
                    btn.connect_clicked(move |_| {
                        self_clone.navigate_to(&path);
                    });
                } else {
                    btn.set_sensitive(false);
                }

                self.path_box.append(&btn);
            }
        }
    }

    /// Navigate to a path
    fn navigate_to(&self, path: &str) {
        *self.current_path.borrow_mut() = path.to_string();
        self.update_breadcrumbs();

        if let Some(ref callback) = *self.on_navigate.borrow() {
            callback(path);
        }
    }

    /// Set navigation callback
    pub fn connect_navigate<F: Fn(&str) + 'static>(&self, callback: F) {
        *self.on_navigate.borrow_mut() = Some(Box::new(callback));
    }

    /// Get current path
    pub fn current_path(&self) -> String {
        self.current_path.borrow().clone()
    }

    /// Go up one level
    pub fn go_up(&self) {
        let current = self.current_path.borrow().clone();
        if let Some(parent) = current.rsplit_once('/') {
            self.navigate_to(parent.0);
        } else if !current.is_empty() {
            self.navigate_to("");
        }
    }

    /// Check if we can go up
    pub fn can_go_up(&self) -> bool {
        !self.current_path.borrow().is_empty()
    }
}

impl Default for PathBar {
    fn default() -> Self {
        Self::new()
    }
}
