//! Tab management for Winux Edit
//!
//! Handles multiple open documents with tabbed interface using libadwaita.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use std::cell::RefCell;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::editor::EditorView;

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct TabManager {
        pub tab_view: OnceCell<adw::TabView>,
        pub editors: RefCell<Vec<EditorView>>,
        pub tab_changed_callbacks: RefCell<Vec<Box<dyn Fn(&super::TabManager)>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TabManager {
        const NAME: &'static str = "WinuxEditTabManager";
        type Type = super::TabManager;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for TabManager {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for TabManager {}
    impl BoxImpl for TabManager {}
}

glib::wrapper! {
    pub struct TabManager(ObjectSubclass<imp::TabManager>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl TabManager {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(0);
        self.set_vexpand(true);
        self.set_hexpand(true);

        // Create tab view
        let tab_view = adw::TabView::new();
        tab_view.set_vexpand(true);

        // Connect to page changes
        let manager_weak = self.downgrade();
        tab_view.connect_selected_page_notify(move |_| {
            if let Some(manager) = manager_weak.upgrade() {
                manager.notify_tab_changed();
            }
        });

        // Connect to close page requests
        let manager_weak = self.downgrade();
        tab_view.connect_close_page(move |tab_view, page| {
            if let Some(manager) = manager_weak.upgrade() {
                manager.handle_close_page(tab_view, page)
            } else {
                false
            }
        });

        imp.tab_view.set(tab_view.clone()).unwrap();

        self.append(&tab_view);
    }

    pub fn tab_view(&self) -> &adw::TabView {
        self.imp().tab_view.get().unwrap()
    }

    pub fn new_tab(&self, path: Option<&PathBuf>) {
        let imp = self.imp();
        let tab_view = imp.tab_view.get().unwrap();

        // Create new editor
        let editor = if let Some(p) = path {
            EditorView::new_with_file(p)
        } else {
            EditorView::new()
        };

        // Add to tab view
        let page = tab_view.append(&editor);
        page.set_title(&editor.get_title());

        // Set icon based on file type
        if let Some(p) = path {
            if let Some(ext) = p.extension() {
                let icon = get_icon_for_extension(&ext.to_string_lossy());
                page.set_icon(Some(&gio::ThemedIcon::new(icon)));
            }
        }

        // Track editor
        imp.editors.borrow_mut().push(editor);

        // Select the new tab
        tab_view.set_selected_page(&page);

        self.notify_tab_changed();
    }

    fn handle_close_page(&self, tab_view: &adw::TabView, page: &adw::TabPage) -> bool {
        let editor = page.child().downcast::<EditorView>().unwrap();

        if editor.is_modified() {
            // Show confirmation dialog
            let dialog = adw::AlertDialog::builder()
                .heading("Save Changes?")
                .body("The document has unsaved changes. Do you want to save them?")
                .build();

            dialog.add_response("discard", "Discard");
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("save", "Save");

            dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
            dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
            dialog.set_default_response(Some("save"));

            let tab_view_clone = tab_view.clone();
            let page_clone = page.clone();
            let manager_weak = self.downgrade();

            dialog.connect_response(None, move |_, response| {
                match response {
                    "save" => {
                        let editor = page_clone.child().downcast::<EditorView>().unwrap();
                        if editor.save() {
                            tab_view_clone.close_page_finish(&page_clone, true);
                            if let Some(manager) = manager_weak.upgrade() {
                                manager.remove_editor(&editor);
                            }
                        }
                    }
                    "discard" => {
                        tab_view_clone.close_page_finish(&page_clone, true);
                        if let Some(manager) = manager_weak.upgrade() {
                            let editor = page_clone.child().downcast::<EditorView>().unwrap();
                            manager.remove_editor(&editor);
                        }
                    }
                    _ => {
                        tab_view_clone.close_page_finish(&page_clone, false);
                    }
                }
            });

            if let Some(root) = self.root() {
                if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                    dialog.present(Some(window));
                }
            }

            // Prevent immediate close, will be handled by dialog
            true
        } else {
            self.remove_editor(&editor);
            false // Allow close
        }
    }

    fn remove_editor(&self, editor: &EditorView) {
        let mut editors = self.imp().editors.borrow_mut();
        editors.retain(|e| e != editor);
    }

    pub fn close_current(&self) {
        let tab_view = self.imp().tab_view.get().unwrap();
        if let Some(page) = tab_view.selected_page() {
            tab_view.close_page(&page);
        }
    }

    pub fn save_current(&self) -> bool {
        if let Some(editor) = self.current_editor() {
            editor.save()
        } else {
            false
        }
    }

    pub fn save_current_as(&self, path: &PathBuf) -> bool {
        if let Some(editor) = self.current_editor() {
            let result = editor.save_to(path);
            if result {
                self.update_current_tab_title();
            }
            result
        } else {
            false
        }
    }

    pub fn save_all(&self) {
        let editors = self.imp().editors.borrow();
        for editor in editors.iter() {
            editor.save();
        }
    }

    fn current_editor(&self) -> Option<EditorView> {
        let tab_view = self.imp().tab_view.get().unwrap();
        tab_view
            .selected_page()
            .map(|page| page.child().downcast::<EditorView>().unwrap())
    }

    fn update_current_tab_title(&self) {
        let tab_view = self.imp().tab_view.get().unwrap();
        if let Some(page) = tab_view.selected_page() {
            let editor = page.child().downcast::<EditorView>().unwrap();
            page.set_title(&editor.get_title());
        }
    }

    pub fn current_title(&self) -> String {
        if let Some(editor) = self.current_editor() {
            editor.get_title()
        } else {
            "Untitled".to_string()
        }
    }

    pub fn show_find_bar(&self) {
        if let Some(editor) = self.current_editor() {
            editor.show_find_bar();
        }
    }

    pub fn goto_line(&self, line: i32) {
        if let Some(editor) = self.current_editor() {
            editor.goto_line(line);
        }
    }

    pub fn set_word_wrap(&self, enabled: bool) {
        let editors = self.imp().editors.borrow();
        for editor in editors.iter() {
            editor.set_word_wrap(enabled);
        }
    }

    pub fn set_line_numbers(&self, enabled: bool) {
        let editors = self.imp().editors.borrow();
        for editor in editors.iter() {
            editor.set_line_numbers(enabled);
        }
    }

    pub fn connect_tab_changed<F: Fn(&Self) + 'static>(&self, callback: F) {
        self.imp()
            .tab_changed_callbacks
            .borrow_mut()
            .push(Box::new(callback));
    }

    fn notify_tab_changed(&self) {
        let callbacks = self.imp().tab_changed_callbacks.borrow();
        for callback in callbacks.iter() {
            callback(self);
        }
    }

    pub fn n_pages(&self) -> u32 {
        self.imp().tab_view.get().unwrap().n_pages() as u32
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

use gtk4::gio;

/// Get icon name for file extension
fn get_icon_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "rs" => "text-x-rust",
        "py" => "text-x-python",
        "js" | "mjs" => "text-x-javascript",
        "ts" | "tsx" => "text-x-typescript",
        "c" | "h" => "text-x-c",
        "cpp" | "hpp" | "cc" | "cxx" => "text-x-c++",
        "java" => "text-x-java",
        "go" => "text-x-go",
        "rb" => "text-x-ruby",
        "php" => "text-x-php",
        "html" | "htm" => "text-html",
        "css" => "text-css",
        "json" => "application-json",
        "xml" => "text-xml",
        "yaml" | "yml" => "text-x-yaml",
        "toml" => "text-x-toml",
        "md" | "markdown" => "text-x-markdown",
        "sh" | "bash" | "zsh" => "text-x-script",
        "sql" => "text-x-sql",
        "dockerfile" => "application-x-docker",
        _ => "text-x-generic",
    }
}
