//! Main application window

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ScrolledWindow, Box as GtkBox, Orientation, Paned};
use libadwaita as adw;
use adw::prelude::*;
use adw::ApplicationWindow as AdwApplicationWindow;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::viewer::{Document, DocumentType};
use crate::ui::{Toolbar, Sidebar, PageView};
use crate::features::{Bookmarks, Annotations, SearchState};

/// Application state
pub struct AppState {
    pub document: Option<Document>,
    pub current_page: usize,
    pub total_pages: usize,
    pub zoom_level: f64,
    pub fit_mode: FitMode,
    pub night_mode: bool,
    pub bookmarks: Bookmarks,
    pub annotations: Annotations,
    pub search_state: SearchState,
    pub sidebar_visible: bool,
    pub presentation_mode: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FitMode {
    None,
    Page,
    Width,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            document: None,
            current_page: 0,
            total_pages: 0,
            zoom_level: 1.0,
            fit_mode: FitMode::Page,
            night_mode: false,
            bookmarks: Bookmarks::new(),
            annotations: Annotations::new(),
            search_state: SearchState::new(),
            sidebar_visible: true,
            presentation_mode: false,
        }
    }
}

pub fn build_window(app: &Application, initial_file: Option<PathBuf>) {
    let state = Rc::new(RefCell::new(AppState::default()));

    // Apply dark theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::PreferDark);

    // Create main layout components
    let page_view = PageView::new(state.clone());
    let sidebar = Sidebar::new(state.clone());
    let toolbar = Toolbar::new(state.clone());

    // Main content area with scrolled window
    let scrolled_window = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&page_view.widget())
        .build();

    // Paned for sidebar and main content
    let paned = Paned::builder()
        .orientation(Orientation::Horizontal)
        .start_child(&sidebar.widget())
        .end_child(&scrolled_window)
        .shrink_start_child(false)
        .shrink_end_child(false)
        .resize_start_child(false)
        .resize_end_child(true)
        .position(250)
        .build();

    // Main vertical box
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.append(&toolbar.widget());
    main_box.append(&paned);

    // Create window
    let window = AdwApplicationWindow::builder()
        .application(app)
        .title("Winux Documents")
        .default_width(1200)
        .default_height(800)
        .content(&main_box)
        .build();

    // Store window reference for callbacks
    let window_weak = window.downgrade();

    // Setup toolbar callbacks
    setup_toolbar_callbacks(&toolbar, &page_view, &sidebar, &window, state.clone());

    // Setup keyboard shortcuts
    setup_keyboard_shortcuts(&window, state.clone(), &page_view, &sidebar);

    // Load initial file if provided
    if let Some(path) = initial_file {
        load_document(&path, state.clone(), &page_view, &sidebar, &window);
    }

    window.present();
}

fn setup_toolbar_callbacks(
    toolbar: &Toolbar,
    page_view: &PageView,
    sidebar: &Sidebar,
    window: &AdwApplicationWindow,
    state: Rc<RefCell<AppState>>,
) {
    // Open file callback
    {
        let window = window.clone();
        let page_view = page_view.clone();
        let sidebar = sidebar.clone();
        let state = state.clone();

        toolbar.on_open(move || {
            let file_dialog = gtk::FileDialog::builder()
                .title("Open Document")
                .modal(true)
                .build();

            let filter = gtk::FileFilter::new();
            filter.add_mime_type("application/pdf");
            filter.add_mime_type("application/epub+zip");
            filter.add_mime_type("image/vnd.djvu");
            filter.add_mime_type("application/oxps");
            filter.add_mime_type("application/vnd.ms-xpsdocument");
            filter.add_pattern("*.pdf");
            filter.add_pattern("*.epub");
            filter.add_pattern("*.djvu");
            filter.add_pattern("*.xps");
            filter.add_pattern("*.cbz");
            filter.add_pattern("*.cbr");
            filter.set_name(Some("Documents"));

            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            file_dialog.set_filters(Some(&filters));

            let window_clone = window.clone();
            let page_view = page_view.clone();
            let sidebar = sidebar.clone();
            let state = state.clone();

            file_dialog.open(Some(&window), gtk::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        load_document(&path, state.clone(), &page_view, &sidebar, &window_clone);
                    }
                }
            });
        });
    }

    // Zoom callbacks
    {
        let page_view = page_view.clone();
        let state = state.clone();

        toolbar.on_zoom_in(move || {
            let mut s = state.borrow_mut();
            s.zoom_level = (s.zoom_level * 1.25).min(5.0);
            s.fit_mode = FitMode::None;
            drop(s);
            page_view.update_zoom();
        });
    }

    {
        let page_view = page_view.clone();
        let state = state.clone();

        toolbar.on_zoom_out(move || {
            let mut s = state.borrow_mut();
            s.zoom_level = (s.zoom_level / 1.25).max(0.1);
            s.fit_mode = FitMode::None;
            drop(s);
            page_view.update_zoom();
        });
    }

    {
        let page_view = page_view.clone();
        let state = state.clone();

        toolbar.on_fit_page(move || {
            let mut s = state.borrow_mut();
            s.fit_mode = FitMode::Page;
            drop(s);
            page_view.update_zoom();
        });
    }

    {
        let page_view = page_view.clone();
        let state = state.clone();

        toolbar.on_fit_width(move || {
            let mut s = state.borrow_mut();
            s.fit_mode = FitMode::Width;
            drop(s);
            page_view.update_zoom();
        });
    }

    // Navigation callbacks
    {
        let page_view = page_view.clone();
        let sidebar = sidebar.clone();
        let state = state.clone();

        toolbar.on_prev_page(move || {
            let mut s = state.borrow_mut();
            if s.current_page > 0 {
                s.current_page -= 1;
                drop(s);
                page_view.render_current_page();
                sidebar.update_selection();
            }
        });
    }

    {
        let page_view = page_view.clone();
        let sidebar = sidebar.clone();
        let state = state.clone();

        toolbar.on_next_page(move || {
            let mut s = state.borrow_mut();
            if s.current_page < s.total_pages.saturating_sub(1) {
                s.current_page += 1;
                drop(s);
                page_view.render_current_page();
                sidebar.update_selection();
            }
        });
    }

    // Night mode callback
    {
        let page_view = page_view.clone();
        let state = state.clone();

        toolbar.on_night_mode(move |enabled| {
            state.borrow_mut().night_mode = enabled;
            page_view.render_current_page();
        });
    }

    // Presentation mode callback
    {
        let window = window.clone();
        let state = state.clone();

        toolbar.on_presentation(move || {
            let mut s = state.borrow_mut();
            s.presentation_mode = !s.presentation_mode;
            drop(s);

            if state.borrow().presentation_mode {
                window.set_fullscreened(true);
            } else {
                window.set_fullscreened(false);
            }
        });
    }

    // Sidebar toggle callback
    {
        let sidebar = sidebar.clone();
        let state = state.clone();

        toolbar.on_toggle_sidebar(move || {
            let mut s = state.borrow_mut();
            s.sidebar_visible = !s.sidebar_visible;
            sidebar.set_visible(s.sidebar_visible);
        });
    }

    // Print callback
    {
        let window = window.clone();
        let state = state.clone();

        toolbar.on_print(move || {
            crate::features::print::print_document(&window, &state.borrow());
        });
    }

    // Search callback
    {
        let page_view = page_view.clone();
        let state = state.clone();

        toolbar.on_search(move |query| {
            crate::features::search::search_document(&query, state.clone());
            page_view.render_current_page();
        });
    }

    // Bookmark callback
    {
        let sidebar = sidebar.clone();
        let state = state.clone();

        toolbar.on_bookmark(move || {
            let mut s = state.borrow_mut();
            let page = s.current_page;
            s.bookmarks.toggle(page);
            drop(s);
            sidebar.update_bookmarks();
        });
    }
}

fn setup_keyboard_shortcuts(
    window: &AdwApplicationWindow,
    state: Rc<RefCell<AppState>>,
    page_view: &PageView,
    sidebar: &Sidebar,
) {
    let controller = gtk::EventControllerKey::new();

    let state_clone = state.clone();
    let page_view = page_view.clone();
    let sidebar = sidebar.clone();

    controller.connect_key_pressed(move |_, key, _, modifier| {
        let ctrl = modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK);

        match key {
            gtk::gdk::Key::Page_Down | gtk::gdk::Key::space | gtk::gdk::Key::Right => {
                let mut s = state_clone.borrow_mut();
                if s.current_page < s.total_pages.saturating_sub(1) {
                    s.current_page += 1;
                    drop(s);
                    page_view.render_current_page();
                    sidebar.update_selection();
                }
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Page_Up | gtk::gdk::Key::Left => {
                let mut s = state_clone.borrow_mut();
                if s.current_page > 0 {
                    s.current_page -= 1;
                    drop(s);
                    page_view.render_current_page();
                    sidebar.update_selection();
                }
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Home => {
                let mut s = state_clone.borrow_mut();
                s.current_page = 0;
                drop(s);
                page_view.render_current_page();
                sidebar.update_selection();
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::End => {
                let mut s = state_clone.borrow_mut();
                s.current_page = s.total_pages.saturating_sub(1);
                drop(s);
                page_view.render_current_page();
                sidebar.update_selection();
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::plus | gtk::gdk::Key::equal if ctrl => {
                let mut s = state_clone.borrow_mut();
                s.zoom_level = (s.zoom_level * 1.25).min(5.0);
                s.fit_mode = FitMode::None;
                drop(s);
                page_view.update_zoom();
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::minus if ctrl => {
                let mut s = state_clone.borrow_mut();
                s.zoom_level = (s.zoom_level / 1.25).max(0.1);
                s.fit_mode = FitMode::None;
                drop(s);
                page_view.update_zoom();
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::_0 if ctrl => {
                let mut s = state_clone.borrow_mut();
                s.zoom_level = 1.0;
                s.fit_mode = FitMode::None;
                drop(s);
                page_view.update_zoom();
                gtk::glib::Propagation::Stop
            }
            _ => gtk::glib::Propagation::Proceed,
        }
    });

    window.add_controller(controller);
}

pub fn load_document(
    path: &PathBuf,
    state: Rc<RefCell<AppState>>,
    page_view: &PageView,
    sidebar: &Sidebar,
    window: &AdwApplicationWindow,
) {
    match Document::open(path) {
        Ok(doc) => {
            let total_pages = doc.page_count();
            let title = doc.title().unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Document")
                    .to_string()
            });

            {
                let mut s = state.borrow_mut();
                s.document = Some(doc);
                s.current_page = 0;
                s.total_pages = total_pages;
                s.zoom_level = 1.0;
                s.fit_mode = FitMode::Page;

                // Load saved bookmarks and annotations
                s.bookmarks = Bookmarks::load_for_document(path);
                s.annotations = Annotations::load_for_document(path);
            }

            window.set_title(Some(&format!("{} - Winux Documents", title)));

            page_view.render_current_page();
            sidebar.populate_thumbnails();
            sidebar.populate_toc();
            sidebar.update_bookmarks();
        }
        Err(e) => {
            let dialog = adw::MessageDialog::new(
                Some(window),
                Some("Error Opening Document"),
                Some(&format!("Failed to open document: {}", e)),
            );
            dialog.add_response("ok", "OK");
            dialog.present();
        }
    }
}
