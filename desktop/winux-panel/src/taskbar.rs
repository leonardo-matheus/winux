//! Taskbar widget module
//!
//! Provides the main taskbar component with:
//! - Start button
//! - Pinned applications
//! - Running window buttons
//! - System tray area
//! - Clock widget

use crate::config::PanelConfig;
use crate::start_menu::StartMenu;
use crate::system_tray::SystemTray;
use crate::widgets::clock::ClockWidget;
use crate::PanelState;
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Button, Image, Orientation, Separator};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Represents a window in the taskbar
#[derive(Debug, Clone)]
pub struct TaskbarWindow {
    /// Window ID
    pub id: u64,
    /// Window title
    pub title: String,
    /// Application ID
    pub app_id: String,
    /// Icon name
    pub icon_name: Option<String>,
    /// Is the window focused
    pub focused: bool,
    /// Is the window minimized
    pub minimized: bool,
}

/// Application group for grouped taskbar buttons
#[derive(Debug, Clone)]
pub struct AppGroup {
    /// Application ID
    pub app_id: String,
    /// Application name
    pub name: String,
    /// Icon name
    pub icon_name: Option<String>,
    /// Windows belonging to this app
    pub windows: Vec<TaskbarWindow>,
    /// Is this app pinned
    pub pinned: bool,
}

/// Main taskbar widget
pub struct Taskbar {
    /// The main container widget
    container: GtkBox,
    /// Start button
    start_button: Button,
    /// Start menu popup
    start_menu: Rc<RefCell<StartMenu>>,
    /// Pinned and running apps area
    apps_box: GtkBox,
    /// System tray
    system_tray: SystemTray,
    /// Clock widget
    clock: ClockWidget,
    /// Application groups
    app_groups: Rc<RefCell<HashMap<String, AppGroup>>>,
    /// Panel state
    state: Arc<RwLock<PanelState>>,
}

impl Taskbar {
    /// Create a new taskbar
    pub fn new(state: Arc<RwLock<PanelState>>) -> Self {
        // Main horizontal container
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .hexpand(true)
            .build();

        container.add_css_class("winux-taskbar");

        // Create start button
        let start_button = Self::create_start_button();

        // Create start menu
        let start_menu = Rc::new(RefCell::new(StartMenu::new()));

        // Connect start button to toggle menu
        let start_menu_clone = Rc::clone(&start_menu);
        start_button.connect_clicked(move |button| {
            debug!("Start button clicked");
            start_menu_clone.borrow_mut().toggle(button);
        });

        // Apps area (pinned + running)
        let apps_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .margin_start(8)
            .build();

        // Create system tray
        let system_tray = SystemTray::new();

        // Create clock
        let clock = ClockWidget::new();

        // Separator before system tray
        let separator = Separator::new(Orientation::Vertical);
        separator.set_margin_start(8);
        separator.set_margin_end(8);

        // Build the taskbar layout
        container.append(&start_button);
        container.append(&apps_box);

        // Right-aligned section
        let right_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .halign(gtk4::Align::End)
            .build();

        right_box.append(&separator);
        right_box.append(system_tray.widget());
        right_box.append(clock.widget());

        container.append(&right_box);

        let taskbar = Self {
            container,
            start_button,
            start_menu,
            apps_box,
            system_tray,
            clock,
            app_groups: Rc::new(RefCell::new(HashMap::new())),
            state,
        };

        // Initialize pinned apps
        taskbar.setup_pinned_apps();

        // Start watching for window changes
        taskbar.setup_window_tracking();

        taskbar
    }

    /// Create the start button
    fn create_start_button() -> Button {
        let button = Button::builder()
            .tooltip_text("Start")
            .build();

        button.add_css_class("start-button");

        // Use Winux logo icon or fallback
        let icon = Image::from_icon_name("winux-logo");
        icon.set_pixel_size(24);

        // Fallback to view-grid if winux-logo is not available
        if icon.icon_name().is_none() {
            icon.set_from_icon_name(Some("view-grid-symbolic"));
        }

        button.set_child(Some(&icon));

        button
    }

    /// Setup pinned applications in the taskbar
    fn setup_pinned_apps(&self) {
        // Get pinned apps from config (would be loaded from state)
        let pinned_apps = vec![
            ("org.winux.Files", "system-file-manager", "Files"),
            ("org.winux.Terminal", "utilities-terminal", "Terminal"),
            ("firefox", "firefox", "Firefox"),
            ("org.winux.Settings", "preferences-system", "Settings"),
        ];

        for (app_id, icon, name) in pinned_apps {
            let button = self.create_app_button(app_id, icon, name, true, false);
            self.apps_box.append(&button);

            // Add to app groups
            self.app_groups.borrow_mut().insert(
                app_id.to_string(),
                AppGroup {
                    app_id: app_id.to_string(),
                    name: name.to_string(),
                    icon_name: Some(icon.to_string()),
                    windows: vec![],
                    pinned: true,
                },
            );
        }
    }

    /// Create an application button for the taskbar
    fn create_app_button(
        &self,
        app_id: &str,
        icon_name: &str,
        tooltip: &str,
        pinned: bool,
        active: bool,
    ) -> Button {
        let button = Button::builder()
            .tooltip_text(tooltip)
            .build();

        button.add_css_class("taskbar-button");

        if pinned {
            button.add_css_class("pinned");
        }

        if active {
            button.add_css_class("active");
        }

        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(24);
        button.set_child(Some(&icon));

        // Store app_id for click handler
        let app_id_owned = app_id.to_string();

        button.connect_clicked(move |_| {
            info!("Launching or focusing: {}", app_id_owned);
            // TODO: Implement window focusing or app launching
            Self::activate_app(&app_id_owned);
        });

        // Add right-click context menu
        let gesture = gtk4::GestureClick::new();
        gesture.set_button(3); // Right click

        let app_id_context = app_id.to_string();
        gesture.connect_released(move |_, _, x, y| {
            debug!("Right-click on app: {} at ({}, {})", app_id_context, x, y);
            // TODO: Show context menu with pin/unpin, close all, etc.
        });

        button.add_controller(gesture);

        button
    }

    /// Activate (focus or launch) an application
    fn activate_app(app_id: &str) {
        // Try to find and focus existing window, or launch new instance
        debug!("Activating app: {}", app_id);

        // TODO: Integrate with window manager to focus existing windows
        // For now, try to launch via D-Bus or direct execution
        if let Err(e) = Self::launch_app(app_id) {
            warn!("Failed to launch app {}: {}", app_id, e);
        }
    }

    /// Launch an application by its desktop file ID
    fn launch_app(app_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        use gtk4::gio;

        // Try to get the app info from the desktop file
        let desktop_id = if app_id.ends_with(".desktop") {
            app_id.to_string()
        } else {
            format!("{}.desktop", app_id)
        };

        if let Some(app_info) = gio::DesktopAppInfo::new(&desktop_id) {
            app_info.launch(&[], gio::AppLaunchContext::NONE)?;
            info!("Launched application: {}", app_id);
            Ok(())
        } else {
            Err(format!("Application not found: {}", app_id).into())
        }
    }

    /// Setup window tracking to update taskbar
    fn setup_window_tracking(&self) {
        // TODO: Connect to compositor's window events
        // This would use Wayland protocols or X11 to track windows

        // For now, set up a periodic refresh
        let apps_box = self.apps_box.clone();
        let app_groups = Rc::clone(&self.app_groups);

        glib::timeout_add_seconds_local(2, move || {
            // Refresh window list
            Self::refresh_windows(&apps_box, &app_groups);
            glib::ControlFlow::Continue
        });
    }

    /// Refresh the window list from the compositor
    fn refresh_windows(
        _apps_box: &GtkBox,
        _app_groups: &Rc<RefCell<HashMap<String, AppGroup>>>,
    ) {
        // TODO: Query compositor for current windows
        // Update app_groups with running windows
        // Add/remove buttons as needed
    }

    /// Add a new window to the taskbar
    pub fn add_window(&mut self, window: TaskbarWindow) {
        let mut groups = self.app_groups.borrow_mut();

        if let Some(group) = groups.get_mut(&window.app_id) {
            // App already exists, add window to group
            group.windows.push(window);
        } else {
            // New app, create group
            let group = AppGroup {
                app_id: window.app_id.clone(),
                name: window.title.clone(),
                icon_name: window.icon_name.clone(),
                windows: vec![window.clone()],
                pinned: false,
            };

            // Create button for new app
            let button = self.create_app_button(
                &window.app_id,
                window.icon_name.as_deref().unwrap_or("application-x-executable"),
                &window.title,
                false,
                window.focused,
            );

            self.apps_box.append(&button);
            groups.insert(window.app_id.clone(), group);
        }
    }

    /// Remove a window from the taskbar
    pub fn remove_window(&mut self, window_id: u64) {
        let mut groups = self.app_groups.borrow_mut();
        let mut app_to_remove: Option<String> = None;

        for (app_id, group) in groups.iter_mut() {
            group.windows.retain(|w| w.id != window_id);

            // If no windows and not pinned, mark for removal
            if group.windows.is_empty() && !group.pinned {
                app_to_remove = Some(app_id.clone());
            }
        }

        // Remove empty unpinned apps
        if let Some(app_id) = app_to_remove {
            groups.remove(&app_id);
            // TODO: Remove the button from apps_box
        }
    }

    /// Update window focus state
    pub fn set_window_focused(&mut self, window_id: u64, focused: bool) {
        let mut groups = self.app_groups.borrow_mut();

        // First, unfocus all windows
        if focused {
            for group in groups.values_mut() {
                for window in &mut group.windows {
                    window.focused = false;
                }
            }
        }

        // Then set the focused window
        for group in groups.values_mut() {
            for window in &mut group.windows {
                if window.id == window_id {
                    window.focused = focused;
                    // TODO: Update button CSS class
                    return;
                }
            }
        }
    }

    /// Pin an application to the taskbar
    pub fn pin_app(&mut self, app_id: &str) {
        let mut groups = self.app_groups.borrow_mut();

        if let Some(group) = groups.get_mut(app_id) {
            group.pinned = true;
        } else {
            // Create new pinned app entry
            groups.insert(
                app_id.to_string(),
                AppGroup {
                    app_id: app_id.to_string(),
                    name: app_id.to_string(),
                    icon_name: None,
                    windows: vec![],
                    pinned: true,
                },
            );
        }

        // TODO: Update config and save
    }

    /// Unpin an application from the taskbar
    pub fn unpin_app(&mut self, app_id: &str) {
        let mut groups = self.app_groups.borrow_mut();

        if let Some(group) = groups.get_mut(app_id) {
            group.pinned = false;

            // Remove if no windows
            if group.windows.is_empty() {
                groups.remove(app_id);
                // TODO: Remove button from apps_box
            }
        }

        // TODO: Update config and save
    }

    /// Get the main widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }
}
