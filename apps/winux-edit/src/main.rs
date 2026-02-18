//! Winux Edit - Text editor with syntax highlighting for Winux OS
//!
//! A modern, lightweight text editor with syntax highlighting support
//! for various programming languages, multi-tab interface, and customizable settings.

mod config;
mod editor;
mod syntax;
mod tabs;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::{gio, glib};
use std::cell::RefCell;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use config::EditorConfig;
use editor::EditorView;
use tabs::TabManager;

/// Application ID for D-Bus and desktop integration
const APP_ID: &str = "com.winux.Edit";

fn main() -> glib::ExitCode {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!("Starting Winux Edit");

    // Create and run the application
    let app = EditApplication::new(APP_ID);
    app.run()
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct EditApplication {
        pub window: OnceCell<adw::ApplicationWindow>,
        pub tab_manager: OnceCell<TabManager>,
        pub config: RefCell<EditorConfig>,
        pub toast_overlay: OnceCell<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditApplication {
        const NAME: &'static str = "WinuxEditApplication";
        type Type = super::EditApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for EditApplication {}

    impl ApplicationImpl for EditApplication {
        fn activate(&self) {
            let app = self.obj();
            app.setup_window();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.setup_actions();
            app.setup_shortcuts();
            app.load_config();
        }

        fn open(&self, files: &[gio::File], _hint: &str) {
            let app = self.obj();
            app.activate();

            for file in files {
                if let Some(path) = file.path() {
                    app.open_file(&path);
                }
            }
        }
    }

    impl GtkApplicationImpl for EditApplication {}
    impl AdwApplicationImpl for EditApplication {}
}

glib::wrapper! {
    pub struct EditApplication(ObjectSubclass<imp::EditApplication>)
        @extends adw::Application, gtk4::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl EditApplication {
    pub fn new(app_id: &str) -> Self {
        glib::Object::builder()
            .property("application-id", app_id)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .build()
    }

    fn load_config(&self) {
        let config = EditorConfig::load();
        self.imp().config.replace(config);
    }

    fn setup_window(&self) {
        let imp = self.imp();

        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(self)
            .title("Winux Edit")
            .default_width(1000)
            .default_height(700)
            .build();

        // Create main layout
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Create header bar
        let header = adw::HeaderBar::new();

        // Tab bar area
        let tab_bar = adw::TabBar::new();
        tab_bar.set_autohide(false);
        tab_bar.set_expand_tabs(false);
        header.set_title_widget(Some(&tab_bar));

        // New tab button
        let new_button = gtk4::Button::builder()
            .icon_name("tab-new-symbolic")
            .tooltip_text("New Tab (Ctrl+T)")
            .build();
        let app_weak = self.downgrade();
        new_button.connect_clicked(move |_| {
            if let Some(app) = app_weak.upgrade() {
                app.new_tab();
            }
        });
        header.pack_start(&new_button);

        // Open button
        let open_button = gtk4::Button::builder()
            .icon_name("document-open-symbolic")
            .tooltip_text("Open File (Ctrl+O)")
            .build();
        let app_weak = self.downgrade();
        open_button.connect_clicked(move |_| {
            if let Some(app) = app_weak.upgrade() {
                app.show_open_dialog();
            }
        });
        header.pack_start(&open_button);

        // Save button
        let save_button = gtk4::Button::builder()
            .icon_name("document-save-symbolic")
            .tooltip_text("Save (Ctrl+S)")
            .build();
        let app_weak = self.downgrade();
        save_button.connect_clicked(move |_| {
            if let Some(app) = app_weak.upgrade() {
                app.save_current_tab();
            }
        });
        header.pack_start(&save_button);

        // Menu button
        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&self.create_app_menu())
            .build();
        header.pack_end(&menu_button);

        // Search button
        let search_button = gtk4::ToggleButton::builder()
            .icon_name("edit-find-symbolic")
            .tooltip_text("Find (Ctrl+F)")
            .build();
        header.pack_end(&search_button);

        main_box.append(&header);

        // Create toast overlay
        let toast_overlay = adw::ToastOverlay::new();

        // Create tab manager with tab view
        let tab_manager = TabManager::new();
        tab_bar.set_view(Some(tab_manager.tab_view()));

        // Connect tab manager signals
        let app_weak = self.downgrade();
        tab_manager.connect_tab_changed(move |manager| {
            if let Some(app) = app_weak.upgrade() {
                app.update_title();
            }
        });

        toast_overlay.set_child(Some(&tab_manager));
        main_box.append(&toast_overlay);

        // Store references
        imp.tab_manager.set(tab_manager.clone()).unwrap();
        imp.toast_overlay.set(toast_overlay).unwrap();

        window.set_content(Some(&main_box));
        imp.window.set(window.clone()).unwrap();

        // Create initial tab
        self.new_tab();

        window.present();
        info!("Winux Edit window initialized");
    }

    fn create_app_menu(&self) -> gio::Menu {
        let menu = gio::Menu::new();

        let file_section = gio::Menu::new();
        file_section.append(Some("New Window"), Some("app.new-window"));
        file_section.append(Some("Save As..."), Some("app.save-as"));
        file_section.append(Some("Save All"), Some("app.save-all"));
        menu.append_section(None, &file_section);

        let edit_section = gio::Menu::new();
        edit_section.append(Some("Find and Replace"), Some("app.find-replace"));
        edit_section.append(Some("Go to Line..."), Some("app.goto-line"));
        menu.append_section(None, &edit_section);

        let view_section = gio::Menu::new();
        view_section.append(Some("Toggle Word Wrap"), Some("app.toggle-wrap"));
        view_section.append(Some("Toggle Line Numbers"), Some("app.toggle-line-numbers"));
        view_section.append(Some("Toggle Minimap"), Some("app.toggle-minimap"));
        menu.append_section(None, &view_section);

        let settings_section = gio::Menu::new();
        settings_section.append(Some("Preferences"), Some("app.preferences"));
        settings_section.append(Some("Keyboard Shortcuts"), Some("win.show-help-overlay"));
        settings_section.append(Some("About Winux Edit"), Some("app.about"));
        menu.append_section(None, &settings_section);

        menu
    }

    fn setup_actions(&self) {
        // New tab action
        let new_tab_action = gio::SimpleAction::new("new-tab", None);
        let app_weak = self.downgrade();
        new_tab_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.new_tab();
            }
        });
        self.add_action(&new_tab_action);

        // Open action
        let open_action = gio::SimpleAction::new("open", None);
        let app_weak = self.downgrade();
        open_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_open_dialog();
            }
        });
        self.add_action(&open_action);

        // Save action
        let save_action = gio::SimpleAction::new("save", None);
        let app_weak = self.downgrade();
        save_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.save_current_tab();
            }
        });
        self.add_action(&save_action);

        // Save As action
        let save_as_action = gio::SimpleAction::new("save-as", None);
        let app_weak = self.downgrade();
        save_as_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_save_as_dialog();
            }
        });
        self.add_action(&save_as_action);

        // Save All action
        let save_all_action = gio::SimpleAction::new("save-all", None);
        let app_weak = self.downgrade();
        save_all_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.save_all_tabs();
            }
        });
        self.add_action(&save_all_action);

        // Close tab action
        let close_tab_action = gio::SimpleAction::new("close-tab", None);
        let app_weak = self.downgrade();
        close_tab_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.close_current_tab();
            }
        });
        self.add_action(&close_tab_action);

        // Find action
        let find_action = gio::SimpleAction::new("find", None);
        let app_weak = self.downgrade();
        find_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_find_bar();
            }
        });
        self.add_action(&find_action);

        // Go to line action
        let goto_action = gio::SimpleAction::new("goto-line", None);
        let app_weak = self.downgrade();
        goto_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_goto_dialog();
            }
        });
        self.add_action(&goto_action);

        // Toggle word wrap
        let wrap_action = gio::SimpleAction::new_stateful(
            "toggle-wrap",
            None,
            &false.to_variant(),
        );
        let app_weak = self.downgrade();
        wrap_action.connect_activate(move |action, _| {
            let state = action.state().unwrap();
            let current: bool = state.get().unwrap();
            action.set_state(&(!current).to_variant());
            if let Some(app) = app_weak.upgrade() {
                app.toggle_word_wrap(!current);
            }
        });
        self.add_action(&wrap_action);

        // Toggle line numbers
        let line_nums_action = gio::SimpleAction::new_stateful(
            "toggle-line-numbers",
            None,
            &true.to_variant(),
        );
        let app_weak = self.downgrade();
        line_nums_action.connect_activate(move |action, _| {
            let state = action.state().unwrap();
            let current: bool = state.get().unwrap();
            action.set_state(&(!current).to_variant());
            if let Some(app) = app_weak.upgrade() {
                app.toggle_line_numbers(!current);
            }
        });
        self.add_action(&line_nums_action);

        // About action
        let about_action = gio::SimpleAction::new("about", None);
        let app_weak = self.downgrade();
        about_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_about_dialog();
            }
        });
        self.add_action(&about_action);

        // Preferences action
        let prefs_action = gio::SimpleAction::new("preferences", None);
        let app_weak = self.downgrade();
        prefs_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.show_preferences();
            }
        });
        self.add_action(&prefs_action);

        // Quit action
        let quit_action = gio::SimpleAction::new("quit", None);
        let app_weak = self.downgrade();
        quit_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.quit();
            }
        });
        self.add_action(&quit_action);
    }

    fn setup_shortcuts(&self) {
        self.set_accels_for_action("app.new-tab", &["<Control>t"]);
        self.set_accels_for_action("app.open", &["<Control>o"]);
        self.set_accels_for_action("app.save", &["<Control>s"]);
        self.set_accels_for_action("app.save-as", &["<Control><Shift>s"]);
        self.set_accels_for_action("app.close-tab", &["<Control>w"]);
        self.set_accels_for_action("app.find", &["<Control>f"]);
        self.set_accels_for_action("app.goto-line", &["<Control>g"]);
        self.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    pub fn new_tab(&self) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            tab_manager.new_tab(None);
        }
    }

    pub fn open_file(&self, path: &PathBuf) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            tab_manager.new_tab(Some(path));
        }
    }

    fn show_open_dialog(&self) {
        let window = self.imp().window.get();

        let dialog = gtk4::FileDialog::builder()
            .title("Open File")
            .modal(true)
            .build();

        let app_weak = self.downgrade();
        dialog.open(window.as_ref(), None::<&gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Some(app) = app_weak.upgrade() {
                        app.open_file(&path);
                    }
                }
            }
        });
    }

    fn show_save_as_dialog(&self) {
        let imp = self.imp();
        let window = imp.window.get();

        let dialog = gtk4::FileDialog::builder()
            .title("Save As")
            .modal(true)
            .build();

        let app_weak = self.downgrade();
        dialog.save(window.as_ref(), None::<&gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Some(app) = app_weak.upgrade() {
                        if let Some(tab_manager) = app.imp().tab_manager.get() {
                            tab_manager.save_current_as(&path);
                        }
                    }
                }
            }
        });
    }

    fn save_current_tab(&self) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            if !tab_manager.save_current() {
                // No file path, show save as dialog
                self.show_save_as_dialog();
            }
        }
    }

    fn save_all_tabs(&self) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            tab_manager.save_all();
        }
        self.show_toast("All files saved");
    }

    fn close_current_tab(&self) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            tab_manager.close_current();
        }
    }

    fn show_find_bar(&self) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            tab_manager.show_find_bar();
        }
    }

    fn show_goto_dialog(&self) {
        let window = self.imp().window.get();

        let dialog = adw::AlertDialog::builder()
            .heading("Go to Line")
            .build();

        let entry = gtk4::Entry::builder()
            .placeholder_text("Line number")
            .input_purpose(gtk4::InputPurpose::Digits)
            .build();

        dialog.set_extra_child(Some(&entry));
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("go", "Go");
        dialog.set_default_response(Some("go"));
        dialog.set_response_appearance("go", adw::ResponseAppearance::Suggested);

        let app_weak = self.downgrade();
        let entry_clone = entry.clone();
        dialog.connect_response(None, move |_, response| {
            if response == "go" {
                if let Ok(line) = entry_clone.text().parse::<i32>() {
                    if let Some(app) = app_weak.upgrade() {
                        if let Some(tab_manager) = app.imp().tab_manager.get() {
                            tab_manager.goto_line(line);
                        }
                    }
                }
            }
        });

        if let Some(win) = window {
            dialog.present(Some(win));
        }
    }

    fn toggle_word_wrap(&self, enabled: bool) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            tab_manager.set_word_wrap(enabled);
        }
    }

    fn toggle_line_numbers(&self, enabled: bool) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            tab_manager.set_line_numbers(enabled);
        }
    }

    fn update_title(&self) {
        if let Some(tab_manager) = self.imp().tab_manager.get() {
            if let Some(window) = self.imp().window.get() {
                let title = tab_manager.current_title();
                window.set_title(Some(&format!("{} - Winux Edit", title)));
            }
        }
    }

    pub fn show_toast(&self, message: &str) {
        if let Some(toast_overlay) = self.imp().toast_overlay.get() {
            let toast = adw::Toast::new(message);
            toast_overlay.add_toast(toast);
        }
    }

    fn show_preferences(&self) {
        info!("Opening preferences");
        // Would show preferences dialog
    }

    fn show_about_dialog(&self) {
        let window = self.imp().window.get();

        let about = adw::AboutDialog::builder()
            .application_name("Winux Edit")
            .application_icon("text-editor")
            .version("1.0.0")
            .developer_name("Winux Team")
            .copyright("© 2024 Winux Project")
            .license_type(gtk4::License::Gpl30)
            .website("https://winux.org")
            .issue_url("https://github.com/winux/winux-edit/issues")
            .build();

        about.add_credit_section(Some("Created by"), &["Winux Development Team"]);

        if let Some(win) = window {
            about.present(Some(win));
        }
    }
}
