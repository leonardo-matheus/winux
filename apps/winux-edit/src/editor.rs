//! Editor widget using GtkSourceView for syntax highlighting
//!
//! Provides the main text editing component with syntax highlighting,
//! line numbers, and various editor features.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use sourceview5::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::syntax::SyntaxManager;

/// Editor state
#[derive(Default)]
pub struct EditorState {
    /// File path if opened from disk
    pub file_path: Option<PathBuf>,
    /// Whether the buffer has unsaved changes
    pub modified: bool,
    /// Current language ID
    pub language: Option<String>,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct EditorView {
        pub state: RefCell<EditorState>,
        pub source_view: OnceCell<sourceview5::View>,
        pub buffer: OnceCell<sourceview5::Buffer>,
        pub search_context: OnceCell<sourceview5::SearchContext>,
        pub find_bar: OnceCell<gtk4::SearchBar>,
        pub find_entry: OnceCell<gtk4::SearchEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditorView {
        const NAME: &'static str = "WinuxEditEditorView";
        type Type = super::EditorView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for EditorView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for EditorView {}
    impl BoxImpl for EditorView {}
}

glib::wrapper! {
    pub struct EditorView(ObjectSubclass<imp::EditorView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl EditorView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn new_with_file(path: &PathBuf) -> Self {
        let editor: Self = glib::Object::builder().build();
        editor.load_file(path);
        editor
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(0);

        // Create find bar
        let find_bar = gtk4::SearchBar::new();
        find_bar.set_show_close_button(true);

        let find_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);

        let find_entry = gtk4::SearchEntry::new();
        find_entry.set_placeholder_text(Some("Find..."));
        find_entry.set_width_request(300);
        find_box.append(&find_entry);

        // Find buttons
        let prev_button = gtk4::Button::from_icon_name("go-up-symbolic");
        prev_button.set_tooltip_text(Some("Previous Match"));
        find_box.append(&prev_button);

        let next_button = gtk4::Button::from_icon_name("go-down-symbolic");
        next_button.set_tooltip_text(Some("Next Match"));
        find_box.append(&next_button);

        // Match count label
        let match_label = gtk4::Label::new(None);
        match_label.add_css_class("dim-label");
        find_box.append(&match_label);

        find_bar.set_child(Some(&find_box));
        find_bar.connect_entry(&find_entry);

        imp.find_bar.set(find_bar.clone()).unwrap();
        imp.find_entry.set(find_entry.clone()).unwrap();

        self.append(&find_bar);

        // Create source buffer with syntax highlighting
        let buffer = sourceview5::Buffer::new(None);

        // Get language manager and set a default scheme
        let scheme_manager = sourceview5::StyleSchemeManager::default();
        if let Some(scheme) = scheme_manager.scheme("Adwaita-dark") {
            buffer.set_style_scheme(Some(&scheme));
        }

        imp.buffer.set(buffer.clone()).unwrap();

        // Create source view
        let source_view = sourceview5::View::with_buffer(&buffer);
        source_view.set_show_line_numbers(true);
        source_view.set_show_line_marks(true);
        source_view.set_auto_indent(true);
        source_view.set_indent_on_tab(true);
        source_view.set_tab_width(4);
        source_view.set_insert_spaces_instead_of_tabs(true);
        source_view.set_smart_backspace(true);
        source_view.set_highlight_current_line(true);
        source_view.set_monospace(true);
        source_view.set_wrap_mode(gtk4::WrapMode::None);

        // Enable bracket matching
        buffer.set_highlight_matching_brackets(true);

        // Set up space drawer for whitespace visualization
        let space_drawer = source_view.space_drawer();
        space_drawer.set_enable_matrix(false);

        imp.source_view.set(source_view.clone()).unwrap();

        // Scrolled window for the editor
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        scrolled.set_child(Some(&source_view));

        self.append(&scrolled);

        // Setup search context
        let search_settings = sourceview5::SearchSettings::new();
        search_settings.set_case_sensitive(false);
        search_settings.set_wrap_around(true);

        let search_context = sourceview5::SearchContext::new(&buffer, Some(&search_settings));
        imp.search_context.set(search_context.clone()).unwrap();

        // Connect find entry
        let ctx = search_context.clone();
        find_entry.connect_search_changed(move |entry| {
            let text = entry.text();
            ctx.settings().unwrap().set_search_text(if text.is_empty() {
                None
            } else {
                Some(&text)
            });
        });

        // Connect next/prev buttons
        let view_weak = self.downgrade();
        next_button.connect_clicked(move |_| {
            if let Some(view) = view_weak.upgrade() {
                view.find_next();
            }
        });

        let view_weak = self.downgrade();
        prev_button.connect_clicked(move |_| {
            if let Some(view) = view_weak.upgrade() {
                view.find_previous();
            }
        });

        // Track modifications
        let view_weak = self.downgrade();
        buffer.connect_changed(move |_| {
            if let Some(view) = view_weak.upgrade() {
                view.imp().state.borrow_mut().modified = true;
            }
        });
    }

    pub fn load_file(&self, path: &PathBuf) {
        let imp = self.imp();

        match std::fs::read_to_string(path) {
            Ok(contents) => {
                if let Some(buffer) = imp.buffer.get() {
                    buffer.set_text(&contents);

                    // Detect and set language
                    let lang_manager = sourceview5::LanguageManager::default();
                    if let Some(language) = lang_manager.guess_language(
                        Some(&path.to_string_lossy()),
                        None,
                    ) {
                        buffer.set_language(Some(&language));
                        imp.state.borrow_mut().language = Some(language.id().to_string());
                    }
                }

                let mut state = imp.state.borrow_mut();
                state.file_path = Some(path.clone());
                state.modified = false;

                info!("Loaded file: {:?}", path);
            }
            Err(e) => {
                warn!("Failed to load file {:?}: {}", path, e);
            }
        }
    }

    pub fn save(&self) -> bool {
        let imp = self.imp();
        let state = imp.state.borrow();

        if let Some(path) = &state.file_path {
            self.save_to(path)
        } else {
            false
        }
    }

    pub fn save_to(&self, path: &PathBuf) -> bool {
        let imp = self.imp();

        if let Some(buffer) = imp.buffer.get() {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, true);

            match std::fs::write(path, text.as_str()) {
                Ok(_) => {
                    let mut state = imp.state.borrow_mut();
                    state.file_path = Some(path.clone());
                    state.modified = false;

                    // Update language if needed
                    let lang_manager = sourceview5::LanguageManager::default();
                    if let Some(language) = lang_manager.guess_language(
                        Some(&path.to_string_lossy()),
                        None,
                    ) {
                        buffer.set_language(Some(&language));
                        state.language = Some(language.id().to_string());
                    }

                    info!("Saved file: {:?}", path);
                    true
                }
                Err(e) => {
                    warn!("Failed to save file {:?}: {}", path, e);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn get_title(&self) -> String {
        let state = self.imp().state.borrow();
        let prefix = if state.modified { "* " } else { "" };

        if let Some(path) = &state.file_path {
            format!(
                "{}{}",
                prefix,
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled".to_string())
            )
        } else {
            format!("{}Untitled", prefix)
        }
    }

    pub fn get_file_path(&self) -> Option<PathBuf> {
        self.imp().state.borrow().file_path.clone()
    }

    pub fn is_modified(&self) -> bool {
        self.imp().state.borrow().modified
    }

    pub fn show_find_bar(&self) {
        if let Some(find_bar) = self.imp().find_bar.get() {
            find_bar.set_search_mode(true);
            if let Some(entry) = self.imp().find_entry.get() {
                entry.grab_focus();
            }
        }
    }

    pub fn hide_find_bar(&self) {
        if let Some(find_bar) = self.imp().find_bar.get() {
            find_bar.set_search_mode(false);
        }
    }

    pub fn find_next(&self) {
        let imp = self.imp();

        if let (Some(buffer), Some(view), Some(context)) =
            (imp.buffer.get(), imp.source_view.get(), imp.search_context.get())
        {
            let cursor = buffer.iter_at_mark(&buffer.get_insert());
            let (found, start, end, _) = context.forward(&cursor);

            if found {
                buffer.select_range(&start, &end);
                view.scroll_to_iter(&start, 0.25, false, 0.0, 0.5);
            }
        }
    }

    pub fn find_previous(&self) {
        let imp = self.imp();

        if let (Some(buffer), Some(view), Some(context)) =
            (imp.buffer.get(), imp.source_view.get(), imp.search_context.get())
        {
            let cursor = buffer.iter_at_mark(&buffer.get_insert());
            let (found, start, end, _) = context.backward(&cursor);

            if found {
                buffer.select_range(&start, &end);
                view.scroll_to_iter(&start, 0.25, false, 0.0, 0.5);
            }
        }
    }

    pub fn goto_line(&self, line: i32) {
        let imp = self.imp();

        if let (Some(buffer), Some(view)) = (imp.buffer.get(), imp.source_view.get()) {
            let line_count = buffer.line_count();
            let target_line = (line - 1).clamp(0, line_count - 1);

            let iter = buffer.iter_at_line(target_line);
            if let Some(iter) = iter {
                buffer.place_cursor(&iter);
                view.scroll_to_iter(&iter, 0.25, false, 0.0, 0.5);
            }
        }
    }

    pub fn set_word_wrap(&self, enabled: bool) {
        if let Some(view) = self.imp().source_view.get() {
            view.set_wrap_mode(if enabled {
                gtk4::WrapMode::Word
            } else {
                gtk4::WrapMode::None
            });
        }
    }

    pub fn set_line_numbers(&self, enabled: bool) {
        if let Some(view) = self.imp().source_view.get() {
            view.set_show_line_numbers(enabled);
        }
    }

    pub fn set_language(&self, lang_id: &str) {
        let imp = self.imp();

        if let Some(buffer) = imp.buffer.get() {
            let lang_manager = sourceview5::LanguageManager::default();
            if let Some(language) = lang_manager.language(lang_id) {
                buffer.set_language(Some(&language));
                imp.state.borrow_mut().language = Some(lang_id.to_string());
            }
        }
    }

    pub fn set_style_scheme(&self, scheme_id: &str) {
        if let Some(buffer) = self.imp().buffer.get() {
            let scheme_manager = sourceview5::StyleSchemeManager::default();
            if let Some(scheme) = scheme_manager.scheme(scheme_id) {
                buffer.set_style_scheme(Some(&scheme));
            }
        }
    }

    pub fn get_text(&self) -> String {
        if let Some(buffer) = self.imp().buffer.get() {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            buffer.text(&start, &end, true).to_string()
        } else {
            String::new()
        }
    }

    pub fn set_text(&self, text: &str) {
        if let Some(buffer) = self.imp().buffer.get() {
            buffer.set_text(text);
        }
    }

    pub fn get_cursor_position(&self) -> (i32, i32) {
        if let Some(buffer) = self.imp().buffer.get() {
            let cursor = buffer.iter_at_mark(&buffer.get_insert());
            let line = cursor.line() + 1;
            let column = cursor.line_offset() + 1;
            (line, column)
        } else {
            (1, 1)
        }
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new()
    }
}
