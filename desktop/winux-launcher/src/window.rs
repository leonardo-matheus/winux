//! Main launcher window

use crate::config::Config;
use crate::search::{SearchEngine, SearchResult, SearchResultKind};
use crate::ui::{PreviewPanel, ResultList, SearchBar};
use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tracing::{debug, info};

/// Main launcher window
pub struct LauncherWindow {
    window: adw::ApplicationWindow,
    search_bar: SearchBar,
    result_list: ResultList,
    preview_panel: PreviewPanel,
    search_engine: Rc<RefCell<SearchEngine>>,
    config: Arc<Config>,
}

impl LauncherWindow {
    /// Create new launcher window
    pub fn new(app: &adw::Application, config: Arc<Config>) -> adw::ApplicationWindow {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Winux Launcher")
            .default_width(config.ui.width)
            .default_height(100)
            .decorated(false)
            .modal(true)
            .resizable(false)
            .build();

        // Add CSS classes
        window.add_css_class("launcher-window");

        // Make window transparent for blur effect
        if let Some(surface) = window.surface() {
            // Window is ready
        }

        // Create main container
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.add_css_class("launcher-container");

        // Create horizontal box for results and preview
        let content_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        // Create search bar
        let search_bar = SearchBar::new();
        main_box.append(search_bar.widget());

        // Create result list
        let result_list = ResultList::new(config.clone());
        content_box.append(result_list.widget());

        // Create preview panel
        let preview_panel = PreviewPanel::new(config.clone());
        if config.ui.show_preview {
            content_box.append(preview_panel.widget());
        }

        // Add content box to main
        main_box.append(&content_box);

        // Create footer hints
        let footer = Self::create_footer();
        main_box.append(&footer);

        window.set_content(Some(&main_box));

        // Initialize search engine
        let search_engine = Rc::new(RefCell::new(SearchEngine::new(config.clone())));

        // Setup key event controller
        let key_controller = gtk4::EventControllerKey::new();

        let search_bar_clone = search_bar.clone();
        let result_list_clone = result_list.clone();
        let preview_panel_clone = preview_panel.clone();
        let search_engine_clone = search_engine.clone();
        let config_clone = config.clone();
        let window_weak = window.downgrade();

        key_controller.connect_key_pressed(move |_, key, _keycode, state| {
            match key {
                gdk::Key::Escape => {
                    if let Some(window) = window_weak.upgrade() {
                        window.close();
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Up => {
                    result_list_clone.select_previous();
                    if let Some(result) = result_list_clone.selected() {
                        preview_panel_clone.show_result(&result);
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Down => {
                    result_list_clone.select_next();
                    if let Some(result) = result_list_clone.selected() {
                        preview_panel_clone.show_result(&result);
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    if let Some(result) = result_list_clone.selected() {
                        Self::execute_result(&result, state.contains(gdk::ModifierType::SHIFT_MASK));
                        if config_clone.general.auto_hide {
                            if let Some(window) = window_weak.upgrade() {
                                window.close();
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Tab => {
                    // Toggle preview
                    preview_panel_clone.toggle_expanded();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });

        window.add_controller(key_controller);

        // Connect search bar text change
        let result_list_clone = result_list.clone();
        let preview_panel_clone = preview_panel.clone();
        let search_engine_clone = search_engine.clone();
        let config_clone = config.clone();

        search_bar.connect_changed(move |query| {
            let engine = search_engine_clone.borrow();
            let results = engine.search(&query);

            result_list_clone.set_results(results.clone());

            // Update preview with first result
            if let Some(first) = results.first() {
                preview_panel_clone.show_result(first);
            } else {
                preview_panel_clone.clear();
            }
        });

        // Center window on screen
        window.connect_realize(|window| {
            if let Some(display) = gdk::Display::default() {
                if let Some(monitor) = display.monitors().item(0) {
                    if let Some(monitor) = monitor.downcast_ref::<gdk::Monitor>() {
                        let geometry = monitor.geometry();
                        let width = window.width();
                        let height = window.height();

                        let x = geometry.x() + (geometry.width() - width) / 2;
                        let y = geometry.y() + (geometry.height() / 4);

                        // Position window
                        // Note: On Wayland, window positioning is restricted
                    }
                }
            }
        });

        // Focus search entry on show
        window.connect_show(glib::clone!(
            #[strong]
            search_bar,
            move |_| {
                search_bar.focus();
                search_bar.clear();
            }
        ));

        window
    }

    /// Create footer with keyboard hints
    fn create_footer() -> gtk4::Box {
        let footer = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        footer.add_css_class("footer-hints");
        footer.set_halign(gtk4::Align::Center);

        let hints = vec![
            ("Enter", "Open"),
            ("Tab", "Preview"),
            ("Ctrl+C", "Copy"),
            ("Esc", "Close"),
        ];

        for (key, action) in hints {
            let hint_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

            let key_label = gtk4::Label::new(Some(key));
            key_label.add_css_class("hint-key");

            let action_label = gtk4::Label::new(Some(action));
            action_label.add_css_class("hint-text");

            hint_box.append(&key_label);
            hint_box.append(&action_label);
            footer.append(&hint_box);
        }

        footer
    }

    /// Execute a search result
    fn execute_result(result: &SearchResult, secondary: bool) {
        info!("Executing result: {:?}", result.title);

        match &result.kind {
            SearchResultKind::Application { desktop_file, .. } => {
                if let Err(e) = open::that_in_background(desktop_file) {
                    tracing::error!("Failed to launch application: {}", e);
                }
            }
            SearchResultKind::File { path } => {
                if secondary {
                    // Open containing folder
                    if let Some(parent) = path.parent() {
                        let _ = open::that_in_background(parent);
                    }
                } else {
                    let _ = open::that_in_background(path);
                }
            }
            SearchResultKind::Calculator { result: calc_result, .. } => {
                // Copy result to clipboard
                if let Some(display) = gdk::Display::default() {
                    let clipboard = display.clipboard();
                    clipboard.set_text(calc_result);
                }
            }
            SearchResultKind::Conversion { result: conv_result, .. } => {
                // Copy result to clipboard
                if let Some(display) = gdk::Display::default() {
                    let clipboard = display.clipboard();
                    clipboard.set_text(conv_result);
                }
            }
            SearchResultKind::WebSearch { url, .. } => {
                let _ = open::that_in_background(url);
            }
            SearchResultKind::Command { command } => {
                Self::execute_command(command);
            }
            SearchResultKind::Plugin { plugin_id, action } => {
                // TODO: Execute plugin action
                debug!("Plugin action: {} - {}", plugin_id, action);
            }
        }
    }

    /// Execute system command
    fn execute_command(command: &str) {
        match command {
            "shutdown" => {
                let _ = std::process::Command::new("systemctl")
                    .arg("poweroff")
                    .spawn();
            }
            "restart" | "reboot" => {
                let _ = std::process::Command::new("systemctl")
                    .arg("reboot")
                    .spawn();
            }
            "logout" => {
                let _ = std::process::Command::new("loginctl")
                    .arg("terminate-user")
                    .arg(std::env::var("USER").unwrap_or_default())
                    .spawn();
            }
            "lock" => {
                let _ = std::process::Command::new("loginctl")
                    .arg("lock-session")
                    .spawn();
            }
            "sleep" | "suspend" => {
                let _ = std::process::Command::new("systemctl")
                    .arg("suspend")
                    .spawn();
            }
            "settings" => {
                let _ = std::process::Command::new("winux-settings").spawn();
            }
            "files" => {
                let _ = std::process::Command::new("winux-files").spawn();
            }
            "terminal" => {
                let _ = std::process::Command::new("winux-terminal").spawn();
            }
            _ => {
                // Try to run as generic command
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .spawn();
            }
        }
    }
}
