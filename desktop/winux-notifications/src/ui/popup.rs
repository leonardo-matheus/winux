//! Notification popup window
//!
//! Individual notification popup with animations and actions.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use glib::clone;
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, Orientation, ProgressBar, Window};
use libadwaita as adw;
use tracing::debug;

use crate::config::{NotificationConfig, NotificationPosition};
use crate::notification::{CloseReason, Notification, Urgency};
use crate::ui::NotificationManager;

/// A notification popup window
pub struct NotificationPopup {
    /// The popup window
    window: Window,
    /// Notification data
    notification: Notification,
    /// Close callback
    on_close: RefCell<Option<Box<dyn Fn(CloseReason)>>>,
    /// Whether popup has been closed
    closed: Cell<bool>,
}

impl NotificationPopup {
    pub fn new(
        notification: Notification,
        config: NotificationConfig,
        manager: Rc<NotificationManager>,
    ) -> Self {
        // Create window
        let window = Window::builder()
            .decorated(false)
            .resizable(false)
            .deletable(false)
            .default_width(config.display.popup_width as i32)
            .css_classes(vec!["notification-popup", notification.urgency_class()])
            .build();

        // Make window a popup type
        window.set_transient_for(None::<&Window>);

        // Create content
        let content = Self::build_content(&notification, &config, &manager);
        window.set_child(Some(&content));

        Self {
            window,
            notification,
            on_close: RefCell::new(None),
            closed: Cell::new(false),
        }
    }

    fn build_content(
        notification: &Notification,
        config: &NotificationConfig,
        manager: &Rc<NotificationManager>,
    ) -> Box {
        let content = Box::new(Orientation::Vertical, 0);
        content.add_css_class("notification-content");

        // Header with icon, title, and close button
        let header = Box::new(Orientation::Horizontal, 8);
        header.add_css_class("notification-header");

        // App icon
        if config.display.show_icons {
            let icon = Self::create_icon(notification, config.display.icon_size as i32);
            icon.add_css_class("notification-icon");
            header.append(&icon);
        }

        // Title and app name
        let title_box = Box::new(Orientation::Vertical, 2);
        title_box.set_hexpand(true);

        let summary_label = Label::new(Some(&notification.summary));
        summary_label.add_css_class("notification-title");
        summary_label.set_halign(gtk::Align::Start);
        summary_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        summary_label.set_max_width_chars(40);
        title_box.append(&summary_label);

        let app_label = Label::new(Some(&notification.app_name));
        app_label.add_css_class("notification-app-name");
        app_label.set_halign(gtk::Align::Start);
        title_box.append(&app_label);

        header.append(&title_box);

        // Time label
        let time_str = notification.timestamp.format("%H:%M").to_string();
        let time_label = Label::new(Some(&time_str));
        time_label.add_css_class("notification-time");
        time_label.set_valign(gtk::Align::Start);
        header.append(&time_label);

        // Close button
        let close_button = Button::from_icon_name("window-close-symbolic");
        close_button.add_css_class("notification-close");
        close_button.set_valign(gtk::Align::Start);

        let notification_id = notification.id;
        let manager_clone = Rc::clone(manager);
        close_button.connect_clicked(move |_| {
            manager_clone.close_notification(notification_id, CloseReason::Dismissed);
        });
        header.append(&close_button);

        content.append(&header);

        // Body
        if !notification.body.is_empty() {
            let body_label = Label::new(Some(&notification.body));
            body_label.add_css_class("notification-body");
            body_label.set_halign(gtk::Align::Start);
            body_label.set_valign(gtk::Align::Start);
            body_label.set_wrap(true);
            body_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            body_label.set_max_width_chars(50);
            body_label.set_lines(config.display.max_body_lines as i32);
            body_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            body_label.set_use_markup(true);
            body_label.set_xalign(0.0);
            content.append(&body_label);
        }

        // Progress bar
        if config.display.show_progress {
            if let Some(progress) = notification.hints.progress_value {
                let progress_bar = ProgressBar::new();
                progress_bar.add_css_class("notification-progress");

                if progress >= 0 {
                    progress_bar.set_fraction(progress as f64 / 100.0);
                } else {
                    progress_bar.pulse();
                    // Setup pulse animation
                    glib::timeout_add_local(
                        std::time::Duration::from_millis(100),
                        clone!(
                            #[weak]
                            progress_bar,
                            #[upgrade_or]
                            glib::ControlFlow::Break,
                            move || {
                                progress_bar.pulse();
                                glib::ControlFlow::Continue
                            }
                        ),
                    );
                }
                content.append(&progress_bar);
            }
        }

        // Actions
        if config.display.show_actions && !notification.actions.is_empty() {
            let actions_box = Box::new(Orientation::Horizontal, 8);
            actions_box.add_css_class("notification-actions");
            actions_box.set_halign(gtk::Align::End);

            for (i, action) in notification.actions.iter().enumerate() {
                let button = Button::with_label(&action.label);
                button.add_css_class("notification-action-button");

                // First action is the default
                if i == 0 {
                    button.add_css_class("default");
                }

                let action_id = action.id.clone();
                let notification_id = notification.id;
                let manager_clone = Rc::clone(manager);
                button.connect_clicked(move |_| {
                    manager_clone.on_action_invoked(notification_id, action_id.clone());
                });

                actions_box.append(&button);
            }

            content.append(&actions_box);
        }

        content
    }

    fn create_icon(notification: &Notification, size: i32) -> Image {
        // Try to load icon from different sources
        if !notification.app_icon.is_empty() {
            if notification.app_icon.starts_with('/') {
                // File path
                return Image::from_file(&notification.app_icon);
            } else {
                // Icon name
                let image = Image::from_icon_name(&notification.app_icon);
                image.set_pixel_size(size);
                return image;
            }
        }

        // Try image path from hints
        if let Some(ref path) = notification.hints.image_path {
            return Image::from_file(path);
        }

        // Default icon based on urgency
        let icon_name = match notification.hints.urgency {
            Urgency::Low => "dialog-information-symbolic",
            Urgency::Normal => "dialog-information-symbolic",
            Urgency::Critical => "dialog-warning-symbolic",
        };

        let image = Image::from_icon_name(icon_name);
        image.set_pixel_size(size);
        image
    }

    /// Get the notification ID
    pub fn notification_id(&self) -> u32 {
        self.notification.id
    }

    /// Connect close callback
    pub fn connect_closed<F: Fn(CloseReason) + 'static>(&self, callback: F) {
        *self.on_close.borrow_mut() = Some(Box::new(callback));
    }

    /// Show the popup at the specified position index
    pub fn show(&self, position_index: i32) {
        // Add slide-in animation class
        self.window.add_css_class("notification-slide-in");

        self.window.present();
        self.reposition(position_index);

        debug!("Showing notification popup: {}", self.notification.summary);
    }

    /// Reposition the popup
    pub fn reposition(&self, position_index: i32) {
        // Note: Actual positioning requires layer-shell or compositor support
        // This is a basic implementation that works with regular windows
        // In a real Wayland environment, gtk4-layer-shell would be used

        // For now, we'll just set a hint for the position
        // The actual positioning would be handled by the compositor or layer-shell
        let _ = position_index; // Position would affect Y offset
    }

    /// Close the popup
    pub fn close(&self) {
        if self.closed.get() {
            return;
        }
        self.closed.set(true);

        // Add slide-out animation
        self.window.remove_css_class("notification-slide-in");
        self.window.add_css_class("notification-slide-out");

        // Close after animation
        let window = self.window.clone();
        let on_close = self.on_close.borrow().clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
            window.close();
            if let Some(callback) = on_close {
                callback(CloseReason::Dismissed);
            }
        });
    }
}

impl Drop for NotificationPopup {
    fn drop(&mut self) {
        if !self.closed.get() {
            self.window.close();
        }
    }
}
