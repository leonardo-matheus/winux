//! Start Menu module
//!
//! Provides the start menu with:
//! - Search bar for applications
//! - Pinned apps grid
//! - All apps list
//! - User profile section
//! - Power options

use freedesktop_desktop_entry::{DesktopEntry, Iter as DesktopIter};
use gtk4::prelude::*;
use gtk4::{
    gdk, gio, glib, Align, Box as GtkBox, Button, Entry, FlowBox, FlowBoxChild,
    Image, Label, ListBox, ListBoxRow, Orientation, Popover, ScrolledWindow, SearchEntry,
    Separator, Widget,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{debug, error, info, warn};

/// Application entry for the start menu
#[derive(Debug, Clone)]
pub struct AppEntry {
    /// Desktop file ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Generic name (optional)
    pub generic_name: Option<String>,
    /// Comment/description
    pub comment: Option<String>,
    /// Icon name
    pub icon: Option<String>,
    /// Executable command
    pub exec: Option<String>,
    /// Desktop file path
    pub path: PathBuf,
    /// Categories
    pub categories: Vec<String>,
    /// Keywords for search
    pub keywords: Vec<String>,
}

/// Start menu widget
pub struct StartMenu {
    /// The popover container
    popover: Popover,
    /// Main content box
    content: GtkBox,
    /// Search entry
    search_entry: SearchEntry,
    /// Pinned apps grid
    pinned_grid: FlowBox,
    /// All apps list
    all_apps_list: ListBox,
    /// Search results list
    search_results: ListBox,
    /// Cached application entries
    apps: Rc<RefCell<Vec<AppEntry>>>,
    /// Pinned app IDs
    pinned_apps: Rc<RefCell<Vec<String>>>,
    /// Current search query
    search_query: Rc<RefCell<String>>,
    /// Is showing search results
    showing_search: Rc<RefCell<bool>>,
}

impl StartMenu {
    /// Create a new start menu
    pub fn new() -> Self {
        // Create popover
        let popover = Popover::new();
        popover.set_has_arrow(false);
        popover.set_position(gtk4::PositionType::Top);
        popover.add_css_class("start-menu");

        // Main content
        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .width_request(600)
            .height_request(700)
            .build();

        // Search entry
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search apps, settings, and files...")
            .hexpand(true)
            .margin_start(16)
            .margin_end(16)
            .margin_top(16)
            .margin_bottom(8)
            .build();

        search_entry.add_css_class("start-menu-search");

        // Pinned apps section
        let pinned_section = Self::create_pinned_section();
        let pinned_grid = pinned_section.1.clone();

        // All apps section
        let all_apps_section = Self::create_all_apps_section();
        let all_apps_list = all_apps_section.1.clone();

        // Search results (initially hidden)
        let search_results = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .visible(false)
            .build();

        search_results.add_css_class("all-apps-list");

        // User area
        let user_area = Self::create_user_area();

        // Power options
        let power_section = Self::create_power_section();

        // Build layout
        content.append(&search_entry);
        content.append(&pinned_section.0);
        content.append(&all_apps_section.0);
        content.append(&search_results);

        // Separator before user/power area
        let separator = Separator::new(Orientation::Horizontal);
        separator.set_margin_top(8);
        content.append(&separator);

        content.append(&user_area);
        content.append(&power_section);

        popover.set_child(Some(&content));

        let apps = Rc::new(RefCell::new(Vec::new()));
        let pinned_apps = Rc::new(RefCell::new(vec![
            "org.winux.Files".to_string(),
            "org.winux.Terminal".to_string(),
            "firefox".to_string(),
            "org.winux.Settings".to_string(),
            "org.winux.Store".to_string(),
            "org.winux.Edit".to_string(),
        ]));
        let search_query = Rc::new(RefCell::new(String::new()));
        let showing_search = Rc::new(RefCell::new(false));

        let menu = Self {
            popover,
            content,
            search_entry,
            pinned_grid,
            all_apps_list,
            search_results,
            apps,
            pinned_apps,
            search_query,
            showing_search,
        };

        // Setup search handling
        menu.setup_search();

        // Load applications
        menu.load_applications();

        menu
    }

    /// Create the pinned apps section
    fn create_pinned_section() -> (GtkBox, FlowBox) {
        let section = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(16)
            .margin_end(16)
            .margin_top(8)
            .build();

        // Section header
        let header = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let title = Label::builder()
            .label("Pinned")
            .halign(Align::Start)
            .hexpand(true)
            .build();

        title.add_css_class("title-4");

        let all_apps_button = Button::builder()
            .label("All apps")
            .has_frame(false)
            .build();

        header.append(&title);
        header.append(&all_apps_button);

        // Pinned apps grid
        let grid = FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .homogeneous(true)
            .max_children_per_line(6)
            .min_children_per_line(6)
            .row_spacing(8)
            .column_spacing(8)
            .build();

        grid.add_css_class("pinned-apps-grid");

        section.append(&header);
        section.append(&grid);

        (section, grid)
    }

    /// Create the all apps section
    fn create_all_apps_section() -> (GtkBox, ListBox) {
        let section = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .vexpand(true)
            .build();

        // Scrolled window for app list
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .build();

        list.add_css_class("all-apps-list");

        scrolled.set_child(Some(&list));
        section.append(&scrolled);

        (section, list)
    }

    /// Create the user area
    fn create_user_area() -> GtkBox {
        let area = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_start(16)
            .margin_end(16)
            .margin_top(12)
            .margin_bottom(8)
            .build();

        area.add_css_class("user-area");

        // User avatar
        let avatar = Image::from_icon_name("avatar-default-symbolic");
        avatar.set_pixel_size(40);

        // User info
        let info_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        // Get actual username
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "User".to_string());

        let name_label = Label::builder()
            .label(&username)
            .halign(Align::Start)
            .build();

        name_label.add_css_class("title-4");

        info_box.append(&name_label);

        area.append(&avatar);
        area.append(&info_box);

        area
    }

    /// Create the power options section
    fn create_power_section() -> GtkBox {
        let section = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(Align::End)
            .margin_start(16)
            .margin_end(16)
            .margin_bottom(12)
            .build();

        section.add_css_class("power-options");

        // Sleep button
        let sleep_btn = Button::builder()
            .icon_name("system-suspend-symbolic")
            .tooltip_text("Sleep")
            .build();

        sleep_btn.add_css_class("power-button");

        sleep_btn.connect_clicked(|_| {
            info!("Sleep requested");
            // TODO: Trigger system sleep via D-Bus
        });

        // Restart button
        let restart_btn = Button::builder()
            .icon_name("system-reboot-symbolic")
            .tooltip_text("Restart")
            .build();

        restart_btn.add_css_class("power-button");

        restart_btn.connect_clicked(|_| {
            info!("Restart requested");
            // TODO: Trigger system restart via D-Bus
        });

        // Shutdown button
        let shutdown_btn = Button::builder()
            .icon_name("system-shutdown-symbolic")
            .tooltip_text("Shut down")
            .build();

        shutdown_btn.add_css_class("power-button");

        shutdown_btn.connect_clicked(|_| {
            info!("Shutdown requested");
            // TODO: Trigger system shutdown via D-Bus
        });

        section.append(&sleep_btn);
        section.append(&restart_btn);
        section.append(&shutdown_btn);

        section
    }

    /// Setup search functionality
    fn setup_search(&self) {
        let apps = Rc::clone(&self.apps);
        let search_results = self.search_results.clone();
        let all_apps_section = self.all_apps_list.parent().unwrap().parent().unwrap(); // Get the section box
        let search_query = Rc::clone(&self.search_query);
        let showing_search = Rc::clone(&self.showing_search);

        self.search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string().to_lowercase();
            *search_query.borrow_mut() = query.clone();

            if query.is_empty() {
                // Show normal view
                *showing_search.borrow_mut() = false;
                search_results.set_visible(false);
                all_apps_section.set_visible(true);
            } else {
                // Show search results
                *showing_search.borrow_mut() = true;
                all_apps_section.set_visible(false);
                search_results.set_visible(true);

                // Clear previous results
                while let Some(child) = search_results.first_child() {
                    search_results.remove(&child);
                }

                // Filter and display matching apps
                let apps_list = apps.borrow();
                let matching: Vec<_> = apps_list
                    .iter()
                    .filter(|app| {
                        app.name.to_lowercase().contains(&query)
                            || app.generic_name.as_ref().map_or(false, |n| n.to_lowercase().contains(&query))
                            || app.keywords.iter().any(|k| k.to_lowercase().contains(&query))
                    })
                    .collect();

                for app in matching.iter().take(10) {
                    let row = Self::create_app_list_row(app);
                    search_results.append(&row);
                }
            }
        });
    }

    /// Load applications from desktop entries
    fn load_applications(&self) {
        let apps = Rc::clone(&self.apps);
        let pinned_apps = Rc::clone(&self.pinned_apps);
        let pinned_grid = self.pinned_grid.clone();
        let all_apps_list = self.all_apps_list.clone();

        // Load in idle to not block UI
        glib::idle_add_local_once(move || {
            let mut app_list = Vec::new();

            // Standard XDG application directories
            let app_dirs = [
                "/usr/share/applications",
                "/usr/local/share/applications",
            ];

            // Also check user directory
            let home = std::env::var("HOME").unwrap_or_default();
            let user_apps = format!("{}/.local/share/applications", home);

            for dir in app_dirs.iter().chain(std::iter::once(&user_apps.as_str())) {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "desktop") {
                            if let Some(app) = Self::parse_desktop_entry(&path) {
                                app_list.push(app);
                            }
                        }
                    }
                }
            }

            // Sort alphabetically
            app_list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Remove duplicates by ID
            app_list.dedup_by(|a, b| a.id == b.id);

            info!("Loaded {} applications", app_list.len());

            // Populate pinned grid
            let pinned = pinned_apps.borrow();
            for app_id in pinned.iter() {
                if let Some(app) = app_list.iter().find(|a| a.id == *app_id) {
                    let child = Self::create_pinned_app_button(app);
                    pinned_grid.append(&child);
                } else {
                    // Create placeholder for missing app
                    let child = Self::create_placeholder_button(app_id);
                    pinned_grid.append(&child);
                }
            }

            // Populate all apps list
            for app in &app_list {
                let row = Self::create_app_list_row(app);
                all_apps_list.append(&row);
            }

            *apps.borrow_mut() = app_list;
        });
    }

    /// Parse a desktop entry file
    fn parse_desktop_entry(path: &PathBuf) -> Option<AppEntry> {
        let content = std::fs::read_to_string(path).ok()?;
        let entry = DesktopEntry::from_str(path, &content, Some(&["en"])).ok()?;

        // Skip hidden or NoDisplay entries
        if entry.no_display() {
            return None;
        }

        let name = entry.name(None)?.to_string();
        let id = path
            .file_stem()?
            .to_str()?
            .to_string();

        Some(AppEntry {
            id,
            name,
            generic_name: entry.generic_name(None).map(|s| s.to_string()),
            comment: entry.comment(None).map(|s| s.to_string()),
            icon: entry.icon().map(|s| s.to_string()),
            exec: entry.exec().map(|s| s.to_string()),
            path: path.clone(),
            categories: entry
                .categories()
                .map(|c| c.split(';').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            keywords: entry
                .keywords(None)
                .map(|k| k.split(';').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
    }

    /// Create a pinned app button
    fn create_pinned_app_button(app: &AppEntry) -> FlowBoxChild {
        let child = FlowBoxChild::new();

        let button = Button::builder()
            .tooltip_text(&app.name)
            .build();

        button.add_css_class("pinned-app-button");

        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .halign(Align::Center)
            .build();

        // Icon
        let icon = if let Some(icon_name) = &app.icon {
            Image::from_icon_name(icon_name)
        } else {
            Image::from_icon_name("application-x-executable")
        };

        icon.set_pixel_size(48);

        // Label
        let label = Label::builder()
            .label(&app.name)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(12)
            .build();

        content.append(&icon);
        content.append(&label);
        button.set_child(Some(&content));

        // Connect click handler
        let app_id = app.id.clone();
        button.connect_clicked(move |_| {
            Self::launch_app(&app_id);
        });

        child.set_child(Some(&button));
        child
    }

    /// Create a placeholder button for missing apps
    fn create_placeholder_button(app_id: &str) -> FlowBoxChild {
        let child = FlowBoxChild::new();

        let button = Button::builder()
            .tooltip_text(app_id)
            .sensitive(false)
            .build();

        button.add_css_class("pinned-app-button");

        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .halign(Align::Center)
            .build();

        let icon = Image::from_icon_name("application-x-executable");
        icon.set_pixel_size(48);

        let label = Label::builder()
            .label(app_id)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(12)
            .build();

        content.append(&icon);
        content.append(&label);
        button.set_child(Some(&content));

        child.set_child(Some(&button));
        child
    }

    /// Create an app list row
    fn create_app_list_row(app: &AppEntry) -> ListBoxRow {
        let row = ListBoxRow::builder()
            .activatable(true)
            .build();

        row.add_css_class("app-list-item");

        let content = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();

        // Icon
        let icon = if let Some(icon_name) = &app.icon {
            Image::from_icon_name(icon_name)
        } else {
            Image::from_icon_name("application-x-executable")
        };

        icon.set_pixel_size(32);

        // Labels
        let labels_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();

        let name_label = Label::builder()
            .label(&app.name)
            .halign(Align::Start)
            .build();

        name_label.add_css_class("title-4");

        labels_box.append(&name_label);

        if let Some(comment) = &app.comment {
            let comment_label = Label::builder()
                .label(comment)
                .halign(Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();

            comment_label.add_css_class("dim-label");
            labels_box.append(&comment_label);
        }

        content.append(&icon);
        content.append(&labels_box);
        row.set_child(Some(&content));

        // Connect activation
        let app_id = app.id.clone();
        row.connect_activate(move |_| {
            Self::launch_app(&app_id);
        });

        row
    }

    /// Launch an application
    fn launch_app(app_id: &str) {
        info!("Launching application: {}", app_id);

        let desktop_id = if app_id.ends_with(".desktop") {
            app_id.to_string()
        } else {
            format!("{}.desktop", app_id)
        };

        if let Some(app_info) = gio::DesktopAppInfo::new(&desktop_id) {
            if let Err(e) = app_info.launch(&[], gio::AppLaunchContext::NONE) {
                error!("Failed to launch {}: {}", app_id, e);
            }
        } else {
            warn!("Could not find desktop entry for: {}", app_id);
        }
    }

    /// Toggle the start menu visibility
    pub fn toggle(&mut self, parent: &impl IsA<Widget>) {
        if self.popover.is_visible() {
            self.popover.popdown();
        } else {
            self.popover.set_parent(parent);
            self.popover.popup();
            self.search_entry.grab_focus();
        }
    }

    /// Show the start menu
    pub fn show(&self, parent: &impl IsA<Widget>) {
        self.popover.set_parent(parent);
        self.popover.popup();
        self.search_entry.grab_focus();
    }

    /// Hide the start menu
    pub fn hide(&self) {
        self.popover.popdown();
    }

    /// Check if the menu is visible
    pub fn is_visible(&self) -> bool {
        self.popover.is_visible()
    }

    /// Get the popover widget
    pub fn widget(&self) -> &Popover {
        &self.popover
    }
}

impl Default for StartMenu {
    fn default() -> Self {
        Self::new()
    }
}
