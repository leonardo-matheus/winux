//! Search view component for Winux Store
//!
//! Provides search functionality with live results and filters.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use std::cell::RefCell;
use std::time::Duration;
use tracing::debug;

use crate::backend::{AppPackage, PackageSource};
use crate::ui::AppCard;

/// Search state
#[derive(Default)]
pub struct SearchState {
    /// Current search query
    pub query: String,
    /// Selected source filter
    pub source_filter: Option<PackageSource>,
    /// Selected category filter
    pub category_filter: Option<String>,
    /// Search results
    pub results: Vec<AppPackage>,
    /// Whether search is in progress
    pub is_searching: bool,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct SearchView {
        pub state: RefCell<SearchState>,
        pub search_entry: OnceCell<gtk4::SearchEntry>,
        pub results_box: OnceCell<gtk4::FlowBox>,
        pub spinner: OnceCell<gtk4::Spinner>,
        pub status_page: OnceCell<adw::StatusPage>,
        pub results_count: OnceCell<gtk4::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchView {
        const NAME: &'static str = "WinuxStoreSearchView";
        type Type = super::SearchView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for SearchView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for SearchView {}
    impl BoxImpl for SearchView {}
}

glib::wrapper! {
    pub struct SearchView(ObjectSubclass<imp::SearchView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl SearchView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(0);

        // Search bar section
        let search_bar = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        search_bar.set_margin_start(24);
        search_bar.set_margin_end(24);
        search_bar.set_margin_top(24);
        search_bar.set_margin_bottom(12);

        // Search entry
        let search_entry = gtk4::SearchEntry::builder()
            .placeholder_text("Search applications...")
            .hexpand(true)
            .build();
        search_entry.add_css_class("search-bar");

        // Connect search
        let view_weak = self.downgrade();
        search_entry.connect_search_changed(move |entry| {
            if let Some(view) = view_weak.upgrade() {
                view.on_search_changed(entry.text().as_str());
            }
        });

        imp.search_entry.set(search_entry.clone()).unwrap();
        search_bar.append(&search_entry);

        // Filters row
        let filters = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

        // Source filter
        let source_label = gtk4::Label::new(Some("Source:"));
        source_label.add_css_class("dim-label");
        filters.append(&source_label);

        let source_dropdown = gtk4::DropDown::from_strings(&[
            "All Sources",
            "Flatpak",
            "APT",
        ]);
        source_dropdown.set_selected(0);

        let view_weak = self.downgrade();
        source_dropdown.connect_selected_notify(move |dropdown| {
            if let Some(view) = view_weak.upgrade() {
                let filter = match dropdown.selected() {
                    1 => Some(PackageSource::Flatpak),
                    2 => Some(PackageSource::Apt),
                    _ => None,
                };
                view.set_source_filter(filter);
            }
        });
        filters.append(&source_dropdown);

        // Category filter
        let category_label = gtk4::Label::new(Some("Category:"));
        category_label.add_css_class("dim-label");
        category_label.set_margin_start(24);
        filters.append(&category_label);

        let category_dropdown = gtk4::DropDown::from_strings(&[
            "All Categories",
            "Audio & Video",
            "Development",
            "Education",
            "Games",
            "Graphics",
            "Network",
            "Office",
            "Science",
            "System",
            "Utilities",
        ]);
        category_dropdown.set_selected(0);

        let view_weak = self.downgrade();
        category_dropdown.connect_selected_notify(move |dropdown| {
            if let Some(view) = view_weak.upgrade() {
                let filter = if dropdown.selected() > 0 {
                    dropdown.selected_item()
                        .and_then(|item| item.downcast::<gtk4::StringObject>().ok())
                        .map(|s| s.string().to_string())
                } else {
                    None
                };
                view.set_category_filter(filter);
            }
        });
        filters.append(&category_dropdown);

        // Spacer
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        filters.append(&spacer);

        // Results count
        let results_count = gtk4::Label::new(None);
        results_count.add_css_class("dim-label");
        imp.results_count.set(results_count.clone()).unwrap();
        filters.append(&results_count);

        search_bar.append(&filters);
        self.append(&search_bar);

        // Separator
        self.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

        // Results area with stack for different states
        let stack = gtk4::Stack::new();
        stack.set_vexpand(true);

        // Empty state
        let empty_status = adw::StatusPage::builder()
            .icon_name("system-search-symbolic")
            .title("Search for Applications")
            .description("Enter a search term to find applications")
            .build();
        stack.add_named(&empty_status, Some("empty"));
        imp.status_page.set(empty_status).unwrap();

        // Loading state
        let loading_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        loading_box.set_valign(gtk4::Align::Center);
        loading_box.set_halign(gtk4::Align::Center);

        let spinner = gtk4::Spinner::new();
        spinner.set_width_request(32);
        spinner.set_height_request(32);
        imp.spinner.set(spinner.clone()).unwrap();
        loading_box.append(&spinner);

        let loading_label = gtk4::Label::new(Some("Searching..."));
        loading_label.add_css_class("dim-label");
        loading_box.append(&loading_label);

        stack.add_named(&loading_box, Some("loading"));

        // Results view
        let scrolled = gtk4::ScrolledWindow::new();

        let results_box = gtk4::FlowBox::new();
        results_box.set_selection_mode(gtk4::SelectionMode::None);
        results_box.set_homogeneous(true);
        results_box.set_max_children_per_line(6);
        results_box.set_min_children_per_line(2);
        results_box.set_column_spacing(12);
        results_box.set_row_spacing(12);
        results_box.set_margin_start(24);
        results_box.set_margin_end(24);
        results_box.set_margin_top(24);
        results_box.set_margin_bottom(24);

        imp.results_box.set(results_box.clone()).unwrap();

        scrolled.set_child(Some(&results_box));
        stack.add_named(&scrolled, Some("results"));

        // No results state
        let no_results = adw::StatusPage::builder()
            .icon_name("face-uncertain-symbolic")
            .title("No Results Found")
            .description("Try a different search term")
            .build();
        stack.add_named(&no_results, Some("no-results"));

        stack.set_visible_child_name("empty");
        self.append(&stack);
    }

    fn on_search_changed(&self, query: &str) {
        let imp = self.imp();

        {
            let mut state = imp.state.borrow_mut();
            state.query = query.to_string();
        }

        if query.is_empty() {
            self.show_empty_state();
            return;
        }

        // Debounce search - in real implementation would use timeout
        self.perform_search();
    }

    fn perform_search(&self) {
        let imp = self.imp();
        let state = imp.state.borrow();

        if state.query.is_empty() {
            return;
        }

        debug!("Performing search for: {}", state.query);

        // Show loading state
        self.show_loading_state();

        // In real implementation, this would be async
        // For now, simulate with sample results
        let results = self.get_sample_results(&state.query);

        drop(state);
        self.display_results(results);
    }

    fn get_sample_results(&self, query: &str) -> Vec<AppPackage> {
        // Sample data - in real implementation would call backend
        let all_apps = vec![
            ("org.mozilla.firefox", "Firefox", "Web Browser", "firefox"),
            ("org.libreoffice.LibreOffice", "LibreOffice", "Office Suite", "libreoffice"),
            ("org.gimp.GIMP", "GIMP", "Image Editor", "gimp"),
            ("org.videolan.VLC", "VLC", "Media Player", "vlc"),
            ("com.visualstudio.code", "Visual Studio Code", "Code Editor", "code"),
            ("com.discordapp.Discord", "Discord", "Voice Chat", "discord"),
            ("com.spotify.Client", "Spotify", "Music Streaming", "spotify"),
            ("com.valvesoftware.Steam", "Steam", "Gaming Platform", "steam"),
            ("org.blender.Blender", "Blender", "3D Modeling", "blender"),
            ("org.inkscape.Inkscape", "Inkscape", "Vector Graphics", "inkscape"),
        ];

        let query_lower = query.to_lowercase();
        all_apps
            .into_iter()
            .filter(|(_, name, desc, _)| {
                name.to_lowercase().contains(&query_lower)
                    || desc.to_lowercase().contains(&query_lower)
            })
            .map(|(id, name, desc, icon)| {
                let mut pkg = AppPackage::new(id, name, PackageSource::Flatpak);
                pkg.summary = desc.to_string();
                pkg.icon = icon.to_string();
                pkg
            })
            .collect()
    }

    fn display_results(&self, results: Vec<AppPackage>) {
        let imp = self.imp();

        // Update state
        {
            let mut state = imp.state.borrow_mut();
            state.results = results.clone();
            state.is_searching = false;
        }

        // Update results count
        if let Some(label) = imp.results_count.get() {
            label.set_text(&format!("{} results", results.len()));
        }

        if results.is_empty() {
            self.show_no_results_state();
            return;
        }

        // Clear and populate results
        if let Some(flow_box) = imp.results_box.get() {
            // Remove all children
            while let Some(child) = flow_box.first_child() {
                flow_box.remove(&child);
            }

            for package in &results {
                let card = AppCard::new(&package.name, &package.summary, &package.icon);
                flow_box.append(&card);
            }
        }

        self.show_results_state();
    }

    fn show_empty_state(&self) {
        if let Some(parent) = self.last_child() {
            if let Some(stack) = parent.downcast_ref::<gtk4::Stack>() {
                stack.set_visible_child_name("empty");
            }
        }

        if let Some(label) = self.imp().results_count.get() {
            label.set_text("");
        }
    }

    fn show_loading_state(&self) {
        if let Some(parent) = self.last_child() {
            if let Some(stack) = parent.downcast_ref::<gtk4::Stack>() {
                stack.set_visible_child_name("loading");
            }
        }

        if let Some(spinner) = self.imp().spinner.get() {
            spinner.start();
        }
    }

    fn show_results_state(&self) {
        if let Some(spinner) = self.imp().spinner.get() {
            spinner.stop();
        }

        if let Some(parent) = self.last_child() {
            if let Some(stack) = parent.downcast_ref::<gtk4::Stack>() {
                stack.set_visible_child_name("results");
            }
        }
    }

    fn show_no_results_state(&self) {
        if let Some(spinner) = self.imp().spinner.get() {
            spinner.stop();
        }

        if let Some(parent) = self.last_child() {
            if let Some(stack) = parent.downcast_ref::<gtk4::Stack>() {
                stack.set_visible_child_name("no-results");
            }
        }
    }

    pub fn set_source_filter(&self, filter: Option<PackageSource>) {
        {
            let mut state = self.imp().state.borrow_mut();
            state.source_filter = filter;
        }
        self.perform_search();
    }

    pub fn set_category_filter(&self, filter: Option<String>) {
        {
            let mut state = self.imp().state.borrow_mut();
            state.category_filter = filter;
        }
        self.perform_search();
    }

    pub fn focus_search(&self) {
        if let Some(entry) = self.imp().search_entry.get() {
            entry.grab_focus();
        }
    }

    pub fn clear_search(&self) {
        if let Some(entry) = self.imp().search_entry.get() {
            entry.set_text("");
        }
        self.show_empty_state();
    }
}

impl Default for SearchView {
    fn default() -> Self {
        Self::new()
    }
}
