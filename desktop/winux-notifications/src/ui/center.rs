//! Notification Center
//!
//! A panel showing notification history with grouping and management features.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{
    Box, Button, Image, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Separator,
    Switch, Window,
};
use libadwaita as adw;
use adw::prelude::*;
use tracing::debug;

use crate::config::NotificationConfig;
use crate::history::NotificationHistory;
use crate::notification::Notification;

/// Callback types for notification center events
pub type OnClearAll = Box<dyn Fn()>;
pub type OnClearApp = Box<dyn Fn(&str)>;
pub type OnDndToggle = Box<dyn Fn(bool)>;
pub type OnNotificationClick = Box<dyn Fn(u32)>;
pub type OnNotificationDismiss = Box<dyn Fn(u32)>;

/// Notification center window
pub struct NotificationCenter {
    /// Main window
    window: Window,
    /// List box for notifications
    list_box: ListBox,
    /// Empty state widget
    empty_state: Box,
    /// DND switch
    dnd_switch: Switch,
    /// Clear all button
    clear_all_button: Button,
    /// Current history reference
    history: RefCell<NotificationHistory>,
    /// Configuration
    config: RefCell<NotificationConfig>,
    /// Callbacks
    on_clear_all: RefCell<Option<OnClearAll>>,
    on_clear_app: RefCell<Option<OnClearApp>>,
    on_dnd_toggle: RefCell<Option<OnDndToggle>>,
    on_notification_click: RefCell<Option<OnNotificationClick>>,
    on_notification_dismiss: RefCell<Option<OnNotificationDismiss>>,
}

impl NotificationCenter {
    pub fn new(config: NotificationConfig, history: NotificationHistory) -> Rc<Self> {
        let window = Window::builder()
            .title("Notifications")
            .default_width(400)
            .default_height(600)
            .decorated(true)
            .resizable(true)
            .css_classes(vec!["notification-center"])
            .build();

        let main_box = Box::new(Orientation::Vertical, 0);

        // Header
        let header = Self::build_header();
        main_box.append(&header.0);

        let dnd_switch = header.1;
        let clear_all_button = header.2;

        // Scrolled list
        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(vec!["notification-center-list"])
            .build();

        scrolled.set_child(Some(&list_box));
        main_box.append(&scrolled);

        // Empty state
        let empty_state = Self::build_empty_state();
        main_box.append(&empty_state);

        window.set_child(Some(&main_box));

        let center = Rc::new(Self {
            window,
            list_box,
            empty_state,
            dnd_switch,
            clear_all_button,
            history: RefCell::new(history),
            config: RefCell::new(config),
            on_clear_all: RefCell::new(None),
            on_clear_app: RefCell::new(None),
            on_dnd_toggle: RefCell::new(None),
            on_notification_click: RefCell::new(None),
            on_notification_dismiss: RefCell::new(None),
        });

        // Connect signals
        center.connect_signals();

        // Initial update
        center.update_list();

        center
    }

    fn build_header() -> (Box, Switch, Button) {
        let header = Box::new(Orientation::Vertical, 8);
        header.add_css_class("notification-center-header");

        // Title row
        let title_row = Box::new(Orientation::Horizontal, 12);

        let title = Label::new(Some("Notifications"));
        title.add_css_class("notification-center-title");
        title.set_hexpand(true);
        title.set_halign(gtk::Align::Start);
        title_row.append(&title);

        let clear_all_button = Button::with_label("Clear All");
        clear_all_button.add_css_class("clear-all-button");
        clear_all_button.add_css_class("flat");
        title_row.append(&clear_all_button);

        header.append(&title_row);

        // DND row
        let dnd_row = Box::new(Orientation::Horizontal, 12);
        dnd_row.add_css_class("dnd-toggle");

        let dnd_icon = Image::from_icon_name("notifications-disabled-symbolic");
        dnd_icon.add_css_class("dnd-toggle-icon");
        dnd_row.append(&dnd_icon);

        let dnd_label_box = Box::new(Orientation::Vertical, 2);
        dnd_label_box.set_hexpand(true);

        let dnd_label = Label::new(Some("Do Not Disturb"));
        dnd_label.set_halign(gtk::Align::Start);
        dnd_label_box.append(&dnd_label);

        let dnd_sublabel = Label::new(Some("Silence all notifications"));
        dnd_sublabel.add_css_class("notification-center-subtitle");
        dnd_sublabel.set_halign(gtk::Align::Start);
        dnd_label_box.append(&dnd_sublabel);

        dnd_row.append(&dnd_label_box);

        let dnd_switch = Switch::new();
        dnd_switch.set_valign(gtk::Align::Center);
        dnd_row.append(&dnd_switch);

        header.append(&dnd_row);

        (header, dnd_switch, clear_all_button)
    }

    fn build_empty_state() -> Box {
        let empty = Box::new(Orientation::Vertical, 16);
        empty.add_css_class("notification-center-empty");
        empty.set_valign(gtk::Align::Center);
        empty.set_halign(gtk::Align::Center);
        empty.set_vexpand(true);

        let icon = Image::from_icon_name("notifications-symbolic");
        icon.add_css_class("notification-center-empty-icon");
        icon.set_pixel_size(64);
        icon.set_opacity(0.3);
        empty.append(&icon);

        let text = Label::new(Some("No notifications"));
        text.add_css_class("notification-center-empty-text");
        empty.append(&text);

        let subtext = Label::new(Some("You're all caught up!"));
        subtext.add_css_class("notification-center-subtitle");
        empty.append(&subtext);

        empty
    }

    fn connect_signals(self: &Rc<Self>) {
        // Clear all button
        let center = Rc::clone(self);
        self.clear_all_button.connect_clicked(move |_| {
            center.clear_all();
        });

        // DND switch
        let center = Rc::clone(self);
        self.dnd_switch.connect_state_set(move |_, state| {
            if let Some(ref callback) = *center.on_dnd_toggle.borrow() {
                callback(state);
            }
            glib::Propagation::Proceed
        });
    }

    /// Update the notification list
    pub fn update_list(&self) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let history = self.history.borrow();

        if history.is_empty() {
            self.list_box.set_visible(false);
            self.empty_state.set_visible(true);
            self.clear_all_button.set_sensitive(false);
            return;
        }

        self.list_box.set_visible(true);
        self.empty_state.set_visible(false);
        self.clear_all_button.set_sensitive(true);

        // Group by app
        let grouped = history.grouped_by_app();

        for (app_name, notifications) in grouped {
            // Group header
            let header_row = ListBoxRow::new();
            header_row.set_selectable(false);
            header_row.set_activatable(false);

            let header_box = Box::new(Orientation::Horizontal, 8);
            header_box.add_css_class("notification-group-header");

            let app_label = Label::new(Some(app_name));
            app_label.set_hexpand(true);
            app_label.set_halign(gtk::Align::Start);
            header_box.append(&app_label);

            let count_label = Label::new(Some(&format!("{}", notifications.len())));
            count_label.add_css_class("notification-center-subtitle");
            header_box.append(&count_label);

            // Clear app button
            let clear_app_button = Button::from_icon_name("edit-clear-symbolic");
            clear_app_button.add_css_class("flat");
            clear_app_button.set_tooltip_text(Some("Clear notifications from this app"));

            let app_name_clone = app_name.clone();
            let on_clear_app = self.on_clear_app.borrow().clone();
            clear_app_button.connect_clicked(move |_| {
                if let Some(ref callback) = on_clear_app {
                    callback(&app_name_clone);
                }
            });
            header_box.append(&clear_app_button);

            header_row.set_child(Some(&header_box));
            self.list_box.append(&header_row);

            // Notification items
            for notification in notifications {
                let row = self.create_notification_row(notification);
                self.list_box.append(&row);
            }
        }
    }

    fn create_notification_row(&self, notification: &Notification) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_selectable(false);

        let mut classes = vec!["notification-history-item"];
        if !notification.read {
            classes.push("unread");
        }

        let content = Box::new(Orientation::Horizontal, 12);
        for class in &classes {
            content.add_css_class(class);
        }

        // Icon
        let icon = if !notification.app_icon.is_empty() {
            if notification.app_icon.starts_with('/') {
                Image::from_file(&notification.app_icon)
            } else {
                let img = Image::from_icon_name(&notification.app_icon);
                img.set_pixel_size(32);
                img
            }
        } else {
            let img = Image::from_icon_name("dialog-information-symbolic");
            img.set_pixel_size(32);
            img
        };
        content.append(&icon);

        // Text
        let text_box = Box::new(Orientation::Vertical, 4);
        text_box.set_hexpand(true);

        let summary = Label::new(Some(&notification.summary));
        summary.set_halign(gtk::Align::Start);
        summary.set_ellipsize(gtk::pango::EllipsizeMode::End);
        summary.add_css_class("notification-title");
        text_box.append(&summary);

        if !notification.body.is_empty() {
            let body = Label::new(Some(&notification.body));
            body.set_halign(gtk::Align::Start);
            body.set_ellipsize(gtk::pango::EllipsizeMode::End);
            body.set_max_width_chars(40);
            body.add_css_class("notification-body");
            text_box.append(&body);
        }

        let time_str = notification.timestamp.format("%H:%M").to_string();
        let time = Label::new(Some(&time_str));
        time.set_halign(gtk::Align::Start);
        time.add_css_class("notification-time");
        text_box.append(&time);

        content.append(&text_box);

        // Dismiss button
        let dismiss_button = Button::from_icon_name("window-close-symbolic");
        dismiss_button.add_css_class("flat");
        dismiss_button.add_css_class("notification-close");
        dismiss_button.set_valign(gtk::Align::Center);

        let notification_id = notification.id;
        let on_dismiss = self.on_notification_dismiss.borrow().clone();
        dismiss_button.connect_clicked(move |_| {
            if let Some(ref callback) = on_dismiss {
                callback(notification_id);
            }
        });
        content.append(&dismiss_button);

        // Click handler
        let notification_id = notification.id;
        let on_click = self.on_notification_click.borrow().clone();
        row.connect_activate(move |_| {
            if let Some(ref callback) = on_click {
                callback(notification_id);
            }
        });

        row.set_child(Some(&content));
        row
    }

    /// Clear all notifications
    pub fn clear_all(&self) {
        self.history.borrow_mut().clear();
        self.update_list();

        if let Some(ref callback) = *self.on_clear_all.borrow() {
            callback();
        }
    }

    /// Clear notifications for a specific app
    pub fn clear_app(&self, app_name: &str) {
        self.history.borrow_mut().clear_app(app_name);
        self.update_list();
    }

    /// Set DND state
    pub fn set_dnd(&self, enabled: bool) {
        self.dnd_switch.set_active(enabled);

        // Update visual state
        if enabled {
            self.dnd_switch.parent().map(|p| p.add_css_class("active"));
        } else {
            self.dnd_switch.parent().map(|p| p.remove_css_class("active"));
        }
    }

    /// Get DND state
    pub fn is_dnd(&self) -> bool {
        self.dnd_switch.is_active()
    }

    /// Update history
    pub fn set_history(&self, history: NotificationHistory) {
        *self.history.borrow_mut() = history;
        self.update_list();
    }

    /// Show the notification center
    pub fn show(&self) {
        self.window.present();
    }

    /// Hide the notification center
    pub fn hide(&self) {
        self.window.close();
    }

    /// Toggle visibility
    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    // Callback setters
    pub fn connect_clear_all<F: Fn() + 'static>(&self, callback: F) {
        *self.on_clear_all.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_clear_app<F: Fn(&str) + 'static>(&self, callback: F) {
        *self.on_clear_app.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_dnd_toggle<F: Fn(bool) + 'static>(&self, callback: F) {
        *self.on_dnd_toggle.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_notification_click<F: Fn(u32) + 'static>(&self, callback: F) {
        *self.on_notification_click.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_notification_dismiss<F: Fn(u32) + 'static>(&self, callback: F) {
        *self.on_notification_dismiss.borrow_mut() = Some(Box::new(callback));
    }
}
