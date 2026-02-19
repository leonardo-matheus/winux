//! Sidebar widget with thumbnails, TOC, and bookmarks

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Orientation, ScrolledWindow, ListBox, ListBoxRow,
    Label, DrawingArea, Stack, StackSwitcher, Frame, Image,
};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::AppState;

/// Sidebar widget
#[derive(Clone)]
pub struct Sidebar {
    container: GtkBox,
    stack: Stack,
    thumbnails_list: ListBox,
    toc_list: ListBox,
    bookmarks_list: ListBox,
    state: Rc<RefCell<AppState>>,
}

impl Sidebar {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        // Create stack for different sidebar views
        let stack = Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);

        // Thumbnails tab
        let thumbnails_list = ListBox::new();
        thumbnails_list.set_selection_mode(gtk::SelectionMode::Single);
        thumbnails_list.add_css_class("navigation-sidebar");

        let thumbnails_scroll = ScrolledWindow::builder()
            .child(&thumbnails_list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();

        stack.add_titled(&thumbnails_scroll, Some("thumbnails"), "Pages");

        // TOC tab
        let toc_list = ListBox::new();
        toc_list.set_selection_mode(gtk::SelectionMode::Single);
        toc_list.add_css_class("navigation-sidebar");

        let toc_scroll = ScrolledWindow::builder()
            .child(&toc_list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();

        stack.add_titled(&toc_scroll, Some("toc"), "Contents");

        // Bookmarks tab
        let bookmarks_list = ListBox::new();
        bookmarks_list.set_selection_mode(gtk::SelectionMode::Single);
        bookmarks_list.add_css_class("navigation-sidebar");

        let bookmarks_scroll = ScrolledWindow::builder()
            .child(&bookmarks_list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();

        stack.add_titled(&bookmarks_scroll, Some("bookmarks"), "Bookmarks");

        // Stack switcher
        let stack_switcher = StackSwitcher::new();
        stack_switcher.set_stack(Some(&stack));
        stack_switcher.set_halign(gtk::Align::Center);

        // Container
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_width_request(250);
        container.append(&stack_switcher);
        container.append(&stack);

        let sidebar = Self {
            container,
            stack,
            thumbnails_list,
            toc_list,
            bookmarks_list,
            state,
        };

        sidebar.setup_callbacks();
        sidebar
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn set_visible(&self, visible: bool) {
        self.container.set_visible(visible);
    }

    fn setup_callbacks(&self) {
        // Thumbnail selection
        {
            let state = self.state.clone();
            self.thumbnails_list.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    let index = row.index() as usize;
                    let mut app_state = state.borrow_mut();
                    app_state.current_page = index;
                    // Note: actual page rendering is handled by the main window
                }
            });
        }

        // TOC selection
        {
            let state = self.state.clone();
            self.toc_list.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    // Get page from row data
                    if let Some(page) = row.data::<usize>("page") {
                        let mut app_state = state.borrow_mut();
                        app_state.current_page = *page;
                    }
                }
            });
        }

        // Bookmark selection
        {
            let state = self.state.clone();
            self.bookmarks_list.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    if let Some(page) = row.data::<usize>("page") {
                        let mut app_state = state.borrow_mut();
                        app_state.current_page = *page;
                    }
                }
            });
        }
    }

    pub fn populate_thumbnails(&self) {
        // Clear existing thumbnails
        while let Some(child) = self.thumbnails_list.first_child() {
            self.thumbnails_list.remove(&child);
        }

        let state = self.state.borrow();

        if let Some(ref document) = state.document {
            let total_pages = document.page_count();

            for page in 0..total_pages {
                let row = self.create_thumbnail_row(page, document);
                self.thumbnails_list.append(&row);
            }
        }
    }

    fn create_thumbnail_row(&self, page: usize, document: &crate::viewer::Document) -> ListBoxRow {
        let row = ListBoxRow::new();

        let hbox = GtkBox::new(Orientation::Horizontal, 8);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);
        hbox.set_margin_top(4);
        hbox.set_margin_bottom(4);

        // Thumbnail preview
        let thumbnail = DrawingArea::new();
        thumbnail.set_size_request(80, 100);

        // Render small thumbnail
        let page_surface = document.render_page(page, 0.1);
        thumbnail.set_draw_func(move |_area, context, width, height| {
            // Gray background
            context.set_source_rgb(0.9, 0.9, 0.9);
            context.paint().ok();

            if let Some(ref surface) = page_surface {
                // Scale to fit
                let src_width = surface.width() as f64;
                let src_height = surface.height() as f64;

                let scale_x = width as f64 / src_width;
                let scale_y = height as f64 / src_height;
                let scale = scale_x.min(scale_y);

                let x = (width as f64 - src_width * scale) / 2.0;
                let y = (height as f64 - src_height * scale) / 2.0;

                context.save().ok();
                context.translate(x, y);
                context.scale(scale, scale);
                context.set_source_surface(surface, 0.0, 0.0).ok();
                context.paint().ok();
                context.restore().ok();
            }

            // Border
            context.set_source_rgb(0.7, 0.7, 0.7);
            context.set_line_width(1.0);
            context.rectangle(0.5, 0.5, width as f64 - 1.0, height as f64 - 1.0);
            context.stroke().ok();
        });

        // Page number label
        let label = Label::new(Some(&format!("{}", page + 1)));
        label.set_halign(gtk::Align::Center);

        let vbox = GtkBox::new(Orientation::Vertical, 4);
        vbox.append(&thumbnail);
        vbox.append(&label);

        hbox.append(&vbox);
        row.set_child(Some(&hbox));

        row
    }

    pub fn populate_toc(&self) {
        // Clear existing TOC
        while let Some(child) = self.toc_list.first_child() {
            self.toc_list.remove(&child);
        }

        let state = self.state.borrow();

        if let Some(ref document) = state.document {
            let toc = document.toc();
            self.add_toc_entries(&toc, 0);
        }
    }

    fn add_toc_entries(&self, entries: &[crate::viewer::TocEntry], depth: usize) {
        for entry in entries {
            let row = ListBoxRow::new();

            let hbox = GtkBox::new(Orientation::Horizontal, 8);
            hbox.set_margin_start((8 + depth * 16) as i32);
            hbox.set_margin_end(8);
            hbox.set_margin_top(4);
            hbox.set_margin_bottom(4);

            // Entry icon
            let icon = Image::from_icon_name("go-next-symbolic");
            icon.set_pixel_size(16);

            // Title label
            let label = Label::new(Some(&entry.title));
            label.set_halign(gtk::Align::Start);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_hexpand(true);

            // Page number
            let page_label = Label::new(Some(&format!("{}", entry.page + 1)));
            page_label.add_css_class("dim-label");

            hbox.append(&icon);
            hbox.append(&label);
            hbox.append(&page_label);

            row.set_child(Some(&hbox));

            // Store page number in row data
            unsafe {
                row.set_data("page", entry.page);
            }

            self.toc_list.append(&row);

            // Add children recursively
            if !entry.children.is_empty() {
                self.add_toc_entries(&entry.children, depth + 1);
            }
        }
    }

    pub fn update_bookmarks(&self) {
        // Clear existing bookmarks
        while let Some(child) = self.bookmarks_list.first_child() {
            self.bookmarks_list.remove(&child);
        }

        let state = self.state.borrow();

        for &page in state.bookmarks.all_pages() {
            let row = ListBoxRow::new();

            let hbox = GtkBox::new(Orientation::Horizontal, 8);
            hbox.set_margin_start(8);
            hbox.set_margin_end(8);
            hbox.set_margin_top(4);
            hbox.set_margin_bottom(4);

            // Bookmark icon
            let icon = Image::from_icon_name("user-bookmarks-symbolic");
            icon.set_pixel_size(16);

            // Label
            let label_text = state.bookmarks.get_label(page)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Page {}", page + 1));

            let label = Label::new(Some(&label_text));
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);

            // Page number
            let page_label = Label::new(Some(&format!("{}", page + 1)));
            page_label.add_css_class("dim-label");

            hbox.append(&icon);
            hbox.append(&label);
            hbox.append(&page_label);

            row.set_child(Some(&hbox));

            // Store page number
            unsafe {
                row.set_data("page", page);
            }

            self.bookmarks_list.append(&row);
        }

        // Show placeholder if no bookmarks
        if state.bookmarks.count() == 0 {
            let row = ListBoxRow::new();
            row.set_selectable(false);

            let label = Label::new(Some("No bookmarks"));
            label.add_css_class("dim-label");
            label.set_margin_top(20);
            label.set_margin_bottom(20);

            row.set_child(Some(&label));
            self.bookmarks_list.append(&row);
        }
    }

    pub fn update_selection(&self) {
        let state = self.state.borrow();
        let current_page = state.current_page;

        // Select the corresponding thumbnail row
        if let Some(row) = self.thumbnails_list.row_at_index(current_page as i32) {
            self.thumbnails_list.select_row(Some(&row));

            // Scroll to make it visible
            // Note: In a real implementation, we'd use scroll_to for smooth scrolling
        }
    }

    pub fn show_search_results(&self, results: &[crate::viewer::SearchResult]) {
        // This could be enhanced to show search results in the sidebar
        // For now, results are highlighted in the page view
    }
}
