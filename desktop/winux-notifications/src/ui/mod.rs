//! UI module for notification daemon
//!
//! Contains the popup notifications, notification center, and styling.

mod center;
mod popup;
mod style;

pub use center::NotificationCenter;
pub use popup::NotificationPopup;
pub use style::StyleManager;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use glib::clone;
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, gdk};
use libadwaita as adw;
use tracing::{debug, error, info};

use crate::config::NotificationConfig;
use crate::daemon::{create_channels, DaemonEvent, DaemonState, NotificationDaemon, SignalEmitter, UiEvent};
use crate::notification::{CloseReason, Notification};

/// Main notification application
pub struct NotificationApp {
    app: Application,
    config: NotificationConfig,
}

impl NotificationApp {
    pub fn new(app_id: &str, config: NotificationConfig) -> Self {
        let app = Application::builder()
            .application_id(app_id)
            .flags(gtk::gio::ApplicationFlags::IS_SERVICE)
            .build();

        Self { app, config }
    }

    pub fn run(&self) -> gtk::glib::ExitCode {
        let config = self.config.clone();

        self.app.connect_activate(move |app| {
            let config = config.clone();
            build_ui(app, config);
        });

        self.app.run()
    }
}

/// Notification manager that handles popup lifecycle
pub struct NotificationManager {
    /// Active popup windows
    popups: RefCell<Vec<NotificationPopup>>,
    /// Configuration
    config: RefCell<NotificationConfig>,
    /// Event sender to daemon
    ui_sender: Sender<UiEvent>,
    /// Style manager
    style_manager: StyleManager,
}

impl NotificationManager {
    pub fn new(config: NotificationConfig, ui_sender: Sender<UiEvent>) -> Rc<Self> {
        Rc::new(Self {
            popups: RefCell::new(Vec::new()),
            config: RefCell::new(config),
            ui_sender,
            style_manager: StyleManager::new(),
        })
    }

    /// Show a notification popup
    pub fn show_notification(self: &Rc<Self>, notification: Notification) {
        let config = self.config.borrow();

        // Check max visible
        let current_count = self.popups.borrow().len();
        if current_count >= config.display.max_visible as usize {
            // Queue for later or dismiss oldest
            debug!("Max visible notifications reached, dismissing oldest");
            if let Some(oldest) = self.popups.borrow_mut().first_mut() {
                oldest.close();
            }
        }

        // Calculate position
        let position_index = current_count;
        let y_offset = (position_index as u32)
            * (config.display.popup_width / 3 + config.display.popup_gap);

        drop(config);

        // Create popup
        let popup = NotificationPopup::new(
            notification.clone(),
            self.config.borrow().clone(),
            Rc::clone(self),
        );

        // Connect close handler
        let manager = Rc::clone(self);
        let notification_id = notification.id;
        popup.connect_closed(move |reason| {
            manager.on_popup_closed(notification_id, reason);
        });

        // Show popup
        popup.show(position_index as i32);

        // Store popup
        self.popups.borrow_mut().push(popup);

        // Setup timeout if needed
        let timeout = notification.effective_timeout(self.config.borrow().display.default_timeout);
        if timeout > 0 {
            let manager = Rc::clone(self);
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(timeout as u64),
                move || {
                    manager.close_notification(notification_id, CloseReason::Expired);
                },
            );
        }
    }

    /// Close a notification
    pub fn close_notification(&self, id: u32, reason: CloseReason) {
        let mut popups = self.popups.borrow_mut();
        if let Some(pos) = popups.iter().position(|p| p.notification_id() == id) {
            let popup = popups.remove(pos);
            popup.close();

            // Reposition remaining popups
            for (i, popup) in popups.iter().enumerate() {
                popup.reposition(i as i32);
            }
        }

        // Notify daemon
        let sender = self.ui_sender.clone();
        glib::spawn_future_local(async move {
            let _ = sender.send(UiEvent::NotificationClosed(id, reason)).await;
        });
    }

    /// Handle popup closed
    fn on_popup_closed(&self, id: u32, reason: CloseReason) {
        let mut popups = self.popups.borrow_mut();
        popups.retain(|p| p.notification_id() != id);

        // Reposition remaining popups
        for (i, popup) in popups.iter().enumerate() {
            popup.reposition(i as i32);
        }

        // Notify daemon
        let sender = self.ui_sender.clone();
        glib::spawn_future_local(async move {
            let _ = sender.send(UiEvent::NotificationClosed(id, reason)).await;
        });
    }

    /// Handle action invoked
    pub fn on_action_invoked(&self, id: u32, action: String) {
        let sender = self.ui_sender.clone();
        glib::spawn_future_local(async move {
            let _ = sender.send(UiEvent::ActionInvoked(id, action)).await;
        });

        // Close notification after action
        self.close_notification(id, CloseReason::Dismissed);
    }

    /// Close all notifications
    pub fn close_all(&self) {
        let popups: Vec<_> = self.popups.borrow_mut().drain(..).collect();
        for popup in popups {
            popup.close();
        }
    }
}

fn build_ui(app: &Application, config: NotificationConfig) {
    info!("Building notification UI");

    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    // Load CSS
    let style_manager = StyleManager::new();
    style_manager.load_css();

    // Create daemon state
    let state = Arc::new(DaemonState::new(config.clone()));

    // Create communication channels
    let ((daemon_sender, daemon_receiver), (ui_sender, ui_receiver)) = create_channels();

    // Create notification manager
    let manager = NotificationManager::new(config, ui_sender.clone());

    // Start D-Bus daemon in background
    let daemon = NotificationDaemon::new(Arc::clone(&state), daemon_sender, ui_receiver);

    glib::spawn_future_local(clone!(
        #[strong]
        manager,
        async move {
            match daemon.start().await {
                Ok(connection) => {
                    info!("D-Bus daemon started successfully");

                    // Create signal emitter for later use
                    let _emitter = SignalEmitter::new(connection);
                }
                Err(e) => {
                    error!("Failed to start D-Bus daemon: {}", e);
                }
            }
        }
    ));

    // Handle incoming daemon events
    glib::spawn_future_local(clone!(
        #[strong]
        manager,
        async move {
            while let Ok(event) = daemon_receiver.recv().await {
                match event {
                    DaemonEvent::NewNotification(notification) => {
                        debug!("Received new notification: {}", notification.summary);
                        manager.show_notification(notification);
                    }
                    DaemonEvent::CloseNotification(id, reason) => {
                        debug!("Closing notification: {}", id);
                        manager.close_notification(id, reason);
                    }
                    DaemonEvent::DndChanged(enabled) => {
                        debug!("DND changed: {}", enabled);
                        // Update UI state
                    }
                }
            }
        }
    ));

    // Keep app running as a service
    app.hold();

    info!("Notification daemon ready");
}
