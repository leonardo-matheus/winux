//! Toolbar widget with document controls

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, Orientation, ToggleButton, MenuButton,
    Label, SpinButton, Entry, Separator,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::HeaderBar;
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::AppState;

/// Toolbar widget
#[derive(Clone)]
pub struct Toolbar {
    header_bar: HeaderBar,
    state: Rc<RefCell<AppState>>,

    // Buttons for callback setup
    open_button: Button,
    zoom_in_button: Button,
    zoom_out_button: Button,
    fit_page_button: Button,
    fit_width_button: Button,
    prev_button: Button,
    next_button: Button,
    night_mode_button: ToggleButton,
    presentation_button: Button,
    sidebar_button: ToggleButton,
    print_button: Button,
    bookmark_button: ToggleButton,
    search_entry: Entry,
    page_spin: SpinButton,
    page_label: Label,
}

impl Toolbar {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let header_bar = HeaderBar::new();

        // Open button
        let open_button = Button::builder()
            .icon_name("document-open-symbolic")
            .tooltip_text("Open Document (Ctrl+O)")
            .build();

        // Sidebar toggle
        let sidebar_button = ToggleButton::builder()
            .icon_name("sidebar-show-symbolic")
            .tooltip_text("Toggle Sidebar (F9)")
            .active(true)
            .build();

        // Navigation buttons
        let prev_button = Button::builder()
            .icon_name("go-previous-symbolic")
            .tooltip_text("Previous Page (Page Up)")
            .build();

        let next_button = Button::builder()
            .icon_name("go-next-symbolic")
            .tooltip_text("Next Page (Page Down)")
            .build();

        // Page indicator
        let page_spin = SpinButton::with_range(1.0, 1.0, 1.0);
        page_spin.set_width_chars(4);
        page_spin.set_tooltip_text(Some("Go to page"));

        let page_label = Label::new(Some("/ 1"));
        page_label.add_css_class("dim-label");

        let nav_box = GtkBox::new(Orientation::Horizontal, 4);
        nav_box.append(&prev_button);
        nav_box.append(&page_spin);
        nav_box.append(&page_label);
        nav_box.append(&next_button);

        // Zoom buttons
        let zoom_out_button = Button::builder()
            .icon_name("zoom-out-symbolic")
            .tooltip_text("Zoom Out (Ctrl+-)")
            .build();

        let zoom_in_button = Button::builder()
            .icon_name("zoom-in-symbolic")
            .tooltip_text("Zoom In (Ctrl++)")
            .build();

        let fit_page_button = Button::builder()
            .icon_name("zoom-fit-best-symbolic")
            .tooltip_text("Fit Page")
            .build();

        let fit_width_button = Button::builder()
            .icon_name("zoom-fit-width-symbolic")
            .tooltip_text("Fit Width")
            .build();

        let zoom_box = GtkBox::new(Orientation::Horizontal, 0);
        zoom_box.add_css_class("linked");
        zoom_box.append(&zoom_out_button);
        zoom_box.append(&fit_page_button);
        zoom_box.append(&fit_width_button);
        zoom_box.append(&zoom_in_button);

        // Search entry
        let search_entry = Entry::builder()
            .placeholder_text("Search...")
            .width_chars(20)
            .build();
        search_entry.set_primary_icon_name(Some("system-search-symbolic"));

        // View controls
        let night_mode_button = ToggleButton::builder()
            .icon_name("weather-clear-night-symbolic")
            .tooltip_text("Night Mode")
            .build();

        let presentation_button = Button::builder()
            .icon_name("view-fullscreen-symbolic")
            .tooltip_text("Presentation Mode (F5)")
            .build();

        let bookmark_button = ToggleButton::builder()
            .icon_name("user-bookmarks-symbolic")
            .tooltip_text("Add/Remove Bookmark (Ctrl+D)")
            .build();

        let print_button = Button::builder()
            .icon_name("printer-symbolic")
            .tooltip_text("Print (Ctrl+P)")
            .build();

        // Menu button for additional options
        let menu_button = MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .build();

        // Create menu
        let menu = gtk::gio::Menu::new();

        let view_section = gtk::gio::Menu::new();
        view_section.append(Some("Rotate Left"), Some("app.rotate-left"));
        view_section.append(Some("Rotate Right"), Some("app.rotate-right"));
        menu.append_section(Some("View"), &view_section);

        let edit_section = gtk::gio::Menu::new();
        edit_section.append(Some("Copy Text"), Some("app.copy-text"));
        edit_section.append(Some("Select All"), Some("app.select-all"));
        menu.append_section(Some("Edit"), &edit_section);

        let tools_section = gtk::gio::Menu::new();
        tools_section.append(Some("Highlight"), Some("app.highlight"));
        tools_section.append(Some("Add Note"), Some("app.add-note"));
        tools_section.append(Some("Draw"), Some("app.draw"));
        menu.append_section(Some("Annotations"), &tools_section);

        let doc_section = gtk::gio::Menu::new();
        doc_section.append(Some("Properties"), Some("app.properties"));
        doc_section.append(Some("Export as PDF"), Some("app.export-pdf"));
        menu.append_section(Some("Document"), &doc_section);

        menu_button.set_menu_model(Some(&menu));

        // Assemble header bar
        header_bar.pack_start(&open_button);
        header_bar.pack_start(&sidebar_button);
        header_bar.pack_start(&gtk::Separator::new(Orientation::Vertical));
        header_bar.pack_start(&nav_box);

        header_bar.pack_end(&menu_button);
        header_bar.pack_end(&print_button);
        header_bar.pack_end(&gtk::Separator::new(Orientation::Vertical));
        header_bar.pack_end(&presentation_button);
        header_bar.pack_end(&night_mode_button);
        header_bar.pack_end(&bookmark_button);
        header_bar.pack_end(&gtk::Separator::new(Orientation::Vertical));
        header_bar.pack_end(&zoom_box);
        header_bar.pack_end(&search_entry);

        let toolbar = Self {
            header_bar,
            state,
            open_button,
            zoom_in_button,
            zoom_out_button,
            fit_page_button,
            fit_width_button,
            prev_button,
            next_button,
            night_mode_button,
            presentation_button,
            sidebar_button,
            print_button,
            bookmark_button,
            search_entry,
            page_spin,
            page_label,
        };

        toolbar.setup_internal_callbacks();
        toolbar
    }

    pub fn widget(&self) -> &HeaderBar {
        &self.header_bar
    }

    fn setup_internal_callbacks(&self) {
        // Page spin button
        {
            let state = self.state.clone();
            self.page_spin.connect_value_changed(move |spin| {
                let page = spin.value() as usize;
                let mut app_state = state.borrow_mut();
                if page > 0 && page <= app_state.total_pages {
                    app_state.current_page = page - 1;
                }
            });
        }

        // Update page label when total pages changes
        {
            let page_label = self.page_label.clone();
            let page_spin = self.page_spin.clone();
            let state = self.state.clone();

            // This would ideally be connected to a signal when document changes
            // For now, we update in update_page_indicator
        }
    }

    pub fn update_page_indicator(&self) {
        let state = self.state.borrow();
        let total = state.total_pages;
        let current = state.current_page + 1;

        self.page_spin.set_range(1.0, total.max(1) as f64);
        self.page_spin.set_value(current as f64);
        self.page_label.set_text(&format!("/ {}", total));

        // Update bookmark button state
        self.bookmark_button.set_active(state.bookmarks.is_bookmarked(state.current_page));
    }

    // Callback setters
    pub fn on_open<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.open_button.connect_clicked(move |_| callback());
    }

    pub fn on_zoom_in<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.zoom_in_button.connect_clicked(move |_| callback());
    }

    pub fn on_zoom_out<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.zoom_out_button.connect_clicked(move |_| callback());
    }

    pub fn on_fit_page<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.fit_page_button.connect_clicked(move |_| callback());
    }

    pub fn on_fit_width<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.fit_width_button.connect_clicked(move |_| callback());
    }

    pub fn on_prev_page<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.prev_button.connect_clicked(move |_| callback());
    }

    pub fn on_next_page<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.next_button.connect_clicked(move |_| callback());
    }

    pub fn on_night_mode<F: Fn(bool) + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.night_mode_button.connect_toggled(move |btn| {
            callback(btn.is_active());
        });
    }

    pub fn on_presentation<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.presentation_button.connect_clicked(move |_| callback());
    }

    pub fn on_toggle_sidebar<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.sidebar_button.connect_toggled(move |_| callback());
    }

    pub fn on_print<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.print_button.connect_clicked(move |_| callback());
    }

    pub fn on_bookmark<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.bookmark_button.connect_toggled(move |_| callback());
    }

    pub fn on_search<F: Fn(String) + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.search_entry.connect_activate(move |entry| {
            callback(entry.text().to_string());
        });
    }

    pub fn on_page_changed<F: Fn(usize) + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        self.page_spin.connect_value_changed(move |spin| {
            let page = spin.value() as usize;
            if page > 0 {
                callback(page - 1);
            }
        });
    }

    pub fn set_search_text(&self, text: &str) {
        self.search_entry.set_text(text);
    }

    pub fn get_search_text(&self) -> String {
        self.search_entry.text().to_string()
    }

    pub fn set_sidebar_active(&self, active: bool) {
        self.sidebar_button.set_active(active);
    }

    pub fn set_bookmark_active(&self, active: bool) {
        self.bookmark_button.set_active(active);
    }

    pub fn set_night_mode_active(&self, active: bool) {
        self.night_mode_button.set_active(active);
    }
}
