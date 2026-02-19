// Winux Notes - Main Window
// Copyright (c) 2026 Winux OS Project

use crate::data::{Note, NoteColor, Notebook, Storage};
use crate::ui::{NoteCard, Toolbar};
use crate::views::{EditorView, GridView, ListView};

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, Entry, FlowBox, Label, ListBox, ListBoxRow,
    Orientation, Paned, Popover, ScrolledWindow, SearchEntry, Separator, Stack, StackSwitcher,
};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

pub struct NotesWindow {
    window: ApplicationWindow,
    storage: Rc<RefCell<Storage>>,
    notes_stack: Stack,
    list_view: ListView,
    grid_view: GridView,
    editor_view: EditorView,
    sidebar_list: ListBox,
    current_notebook: Rc<RefCell<Option<String>>>,
    current_note: Rc<RefCell<Option<String>>>,
    search_entry: SearchEntry,
}

impl NotesWindow {
    pub fn new(app: &Application) -> Self {
        // Force dark theme
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Winux Notes")
            .default_width(1200)
            .default_height(800)
            .build();

        // Initialize storage
        let storage = Rc::new(RefCell::new(Storage::new().expect("Failed to initialize storage")));

        // Create main layout
        let main_box = Box::new(Orientation::Horizontal, 0);

        // Create sidebar
        let sidebar = Self::create_sidebar_box();
        let sidebar_list = Self::create_sidebar_list(&storage.borrow());

        let sidebar_scroll = ScrolledWindow::builder()
            .child(&sidebar_list)
            .vexpand(true)
            .build();
        sidebar.append(&sidebar_scroll);

        main_box.append(&sidebar);

        // Separator
        let separator = Separator::new(Orientation::Vertical);
        main_box.append(&separator);

        // Create content area with Paned for resizable panels
        let paned = Paned::new(Orientation::Horizontal);
        paned.set_hexpand(true);

        // Notes list/grid area
        let notes_area = Box::new(Orientation::Vertical, 0);

        // Header with search and view toggle
        let header = Self::create_header();
        let search_entry = header.1.clone();
        notes_area.append(&header.0);

        // Stack for list/grid views
        let notes_stack = Stack::new();
        notes_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

        let list_view = ListView::new();
        let grid_view = GridView::new();

        notes_stack.add_titled(list_view.widget(), Some("list"), "List");
        notes_stack.add_titled(grid_view.widget(), Some("grid"), "Grid");

        let stack_scroll = ScrolledWindow::builder()
            .child(&notes_stack)
            .vexpand(true)
            .hexpand(true)
            .build();
        notes_area.append(&stack_scroll);

        paned.set_start_child(Some(&notes_area));
        paned.set_resize_start_child(true);
        paned.set_shrink_start_child(false);

        // Editor area
        let editor_view = EditorView::new();
        paned.set_end_child(Some(editor_view.widget()));
        paned.set_resize_end_child(true);
        paned.set_shrink_end_child(false);
        paned.set_position(400);

        main_box.append(&paned);

        window.set_child(Some(&main_box));

        let current_notebook = Rc::new(RefCell::new(None));
        let current_note = Rc::new(RefCell::new(None));

        let mut notes_window = Self {
            window,
            storage,
            notes_stack,
            list_view,
            grid_view,
            editor_view,
            sidebar_list,
            current_notebook,
            current_note,
            search_entry,
        };

        notes_window.setup_signals();
        notes_window.load_notes(None);

        notes_window
    }

    fn create_sidebar_box() -> Box {
        let sidebar = Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(220)
            .css_classes(vec!["sidebar"])
            .build();

        // App title
        let title = Label::builder()
            .label("Notes")
            .css_classes(vec!["title-2"])
            .margin_top(15)
            .margin_bottom(10)
            .margin_start(15)
            .halign(gtk4::Align::Start)
            .build();
        sidebar.append(&title);

        // New note button
        let new_note_btn = Button::builder()
            .label("New Note")
            .css_classes(vec!["suggested-action"])
            .margin_start(10)
            .margin_end(10)
            .margin_bottom(10)
            .build();
        new_note_btn.set_icon_name("list-add-symbolic");
        sidebar.append(&new_note_btn);

        sidebar
    }

    fn create_sidebar_list(storage: &Storage) -> ListBox {
        let list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(vec!["navigation-sidebar"])
            .margin_start(10)
            .margin_end(10)
            .build();

        // Fixed items
        let all_notes = Self::create_sidebar_row("view-list-symbolic", "All Notes", None);
        list.append(&all_notes);

        let favorites = Self::create_sidebar_row("starred-symbolic", "Favorites", None);
        list.append(&favorites);

        let pinned = Self::create_sidebar_row("view-pin-symbolic", "Pinned", None);
        list.append(&pinned);

        // Separator
        let sep = Separator::new(Orientation::Horizontal);
        let sep_row = ListBoxRow::builder()
            .child(&sep)
            .selectable(false)
            .activatable(false)
            .build();
        list.append(&sep_row);

        // Notebooks header
        let notebooks_header = Box::new(Orientation::Horizontal, 0);
        let header_label = Label::builder()
            .label("Notebooks")
            .css_classes(vec!["dim-label", "caption"])
            .margin_start(10)
            .margin_top(5)
            .margin_bottom(5)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        notebooks_header.append(&header_label);

        let add_notebook_btn = Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(vec!["flat", "circular"])
            .tooltip_text("New Notebook")
            .build();
        notebooks_header.append(&add_notebook_btn);

        let header_row = ListBoxRow::builder()
            .child(&notebooks_header)
            .selectable(false)
            .activatable(false)
            .build();
        list.append(&header_row);

        // Load notebooks from storage
        if let Ok(notebooks) = storage.get_notebooks() {
            for notebook in notebooks {
                let row = Self::create_sidebar_row(
                    "folder-symbolic",
                    &notebook.name,
                    Some(&notebook.id),
                );
                list.append(&row);
            }
        }

        // Select first row by default
        if let Some(first_row) = list.row_at_index(0) {
            list.select_row(Some(&first_row));
        }

        list
    }

    fn create_sidebar_row(icon_name: &str, label_text: &str, id: Option<&str>) -> ListBoxRow {
        let row_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(8)
            .margin_end(8)
            .build();

        let icon = gtk4::Image::from_icon_name(icon_name);
        let label = Label::builder()
            .label(label_text)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        row_box.append(&icon);
        row_box.append(&label);

        let row = ListBoxRow::builder()
            .child(&row_box)
            .build();

        if let Some(id) = id {
            row.set_widget_name(id);
        }

        row
    }

    fn create_header() -> (Box, SearchEntry) {
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_start(15)
            .margin_end(15)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // Search entry
        let search = SearchEntry::builder()
            .placeholder_text("Search notes...")
            .hexpand(true)
            .build();
        header.append(&search);

        // View toggle buttons
        let view_box = Box::new(Orientation::Horizontal, 0);
        view_box.add_css_class("linked");

        let list_btn = Button::builder()
            .icon_name("view-list-symbolic")
            .tooltip_text("List View")
            .build();
        list_btn.set_widget_name("list");
        view_box.append(&list_btn);

        let grid_btn = Button::builder()
            .icon_name("view-grid-symbolic")
            .tooltip_text("Grid View")
            .build();
        grid_btn.set_widget_name("grid");
        view_box.append(&grid_btn);

        header.append(&view_box);

        // Sort menu button
        let sort_btn = Button::builder()
            .icon_name("view-sort-descending-symbolic")
            .tooltip_text("Sort")
            .build();
        header.append(&sort_btn);

        (header, search)
    }

    fn setup_signals(&mut self) {
        // TODO: Connect signals for:
        // - New note button
        // - Sidebar selection
        // - Search
        // - View toggle
        // - Note selection
        // - Editor changes
    }

    fn load_notes(&self, notebook_id: Option<&str>) {
        let notes = if let Some(id) = notebook_id {
            self.storage.borrow().get_notes_by_notebook(id).unwrap_or_default()
        } else {
            self.storage.borrow().get_all_notes().unwrap_or_default()
        };

        self.list_view.set_notes(&notes);
        self.grid_view.set_notes(&notes);
    }

    pub fn present(&self) {
        self.window.present();
    }
}
