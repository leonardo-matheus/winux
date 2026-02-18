//! Application model and state management for Winux Store

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::{gio, glib};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, info, warn};

use crate::backend::{AppPackage, PackageBackend, PackageManager};
use crate::ui::{AppCard, AppDetailsPage, SearchView};

/// Application state
#[derive(Default)]
pub struct AppState {
    /// Currently selected category
    pub current_category: Option<String>,
    /// Search query
    pub search_query: Option<String>,
    /// Cached packages
    pub packages: Vec<AppPackage>,
    /// Installation queue
    pub install_queue: Vec<String>,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct StoreApplication {
        pub window: OnceCell<adw::ApplicationWindow>,
        pub state: RefCell<AppState>,
        pub backend: OnceCell<PackageManager>,
        pub navigation_view: OnceCell<adw::NavigationView>,
        pub toast_overlay: OnceCell<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StoreApplication {
        const NAME: &'static str = "WinuxStoreApplication";
        type Type = super::StoreApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for StoreApplication {}

    impl ApplicationImpl for StoreApplication {
        fn activate(&self) {
            let app = self.obj();
            app.setup_window();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.setup_actions();
            app.setup_shortcuts();
        }
    }

    impl GtkApplicationImpl for StoreApplication {}
    impl AdwApplicationImpl for StoreApplication {}
}

glib::wrapper! {
    pub struct StoreApplication(ObjectSubclass<imp::StoreApplication>)
        @extends adw::Application, gtk4::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl StoreApplication {
    pub fn new(app_id: &str) -> Self {
        glib::Object::builder()
            .property("application-id", app_id)
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .build()
    }

    fn setup_window(&self) {
        let imp = self.imp();

        // Initialize backend
        let backend = PackageManager::new();
        imp.backend.set(backend).expect("Backend already set");

        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(self)
            .title("Winux Store")
            .default_width(1200)
            .default_height(800)
            .build();

        // Create navigation view for page transitions
        let navigation_view = adw::NavigationView::new();

        // Create toast overlay for notifications
        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&navigation_view));

        // Create header bar
        let header = adw::HeaderBar::new();

        // Search button
        let search_button = gtk4::ToggleButton::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search applications")
            .build();

        // Menu button
        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&self.create_app_menu())
            .build();

        header.pack_end(&menu_button);
        header.pack_end(&search_button);

        // Create main content
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.append(&header);

        // Create search bar
        let search_bar = gtk4::SearchBar::new();
        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_hexpand(true);
        search_bar.set_child(Some(&search_entry));
        search_bar.connect_entry(&search_entry);
        search_button.bind_property("active", &search_bar, "search-mode-enabled")
            .bidirectional()
            .build();
        main_box.append(&search_bar);

        // Create content with sidebar
        let content_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        // Sidebar with categories
        let sidebar = self.create_sidebar();
        content_box.append(&sidebar);

        // Main content area
        let content_stack = adw::ViewStack::new();
        content_stack.set_hexpand(true);
        content_stack.set_vexpand(true);

        // Featured page
        let featured_page = self.create_featured_page();
        content_stack.add_titled(&featured_page, Some("featured"), "Featured");

        // Categories page
        let categories_page = self.create_categories_page();
        content_stack.add_titled(&categories_page, Some("categories"), "Categories");

        // Installed page
        let installed_page = self.create_installed_page();
        content_stack.add_titled(&installed_page, Some("installed"), "Installed");

        // Updates page
        let updates_page = self.create_updates_page();
        content_stack.add_titled(&updates_page, Some("updates"), "Updates");

        content_box.append(&content_stack);
        main_box.append(&content_box);

        // Create navigation page
        let nav_page = adw::NavigationPage::builder()
            .title("Winux Store")
            .child(&main_box)
            .build();

        navigation_view.push(&nav_page);

        // Store references
        imp.navigation_view.set(navigation_view).expect("NavigationView already set");
        imp.toast_overlay.set(toast_overlay.clone()).expect("ToastOverlay already set");

        // Set window content
        window.set_content(Some(&toast_overlay));

        // Connect search
        let app_weak = self.downgrade();
        search_entry.connect_search_changed(move |entry| {
            if let Some(app) = app_weak.upgrade() {
                app.perform_search(entry.text().as_str());
            }
        });

        imp.window.set(window.clone()).expect("Window already set");
        window.present();

        info!("Winux Store window initialized");
    }

    fn create_sidebar(&self) -> gtk4::Box {
        let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        sidebar.set_width_request(250);
        sidebar.add_css_class("sidebar");
        sidebar.set_margin_top(12);
        sidebar.set_margin_bottom(12);
        sidebar.set_margin_start(12);
        sidebar.set_margin_end(12);

        let categories = [
            ("star-filled-symbolic", "Featured"),
            ("applications-other-symbolic", "Categories"),
            ("folder-download-symbolic", "Installed"),
            ("software-update-available-symbolic", "Updates"),
        ];

        let list_box = gtk4::ListBox::new();
        list_box.add_css_class("navigation-sidebar");
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        for (icon, label) in categories {
            let row = adw::ActionRow::builder()
                .title(label)
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));
            list_box.append(&row);
        }

        // Select first row
        if let Some(row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&row));
        }

        sidebar.append(&list_box);

        // Separator
        sidebar.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

        // Category filters
        let category_label = gtk4::Label::new(Some("Categories"));
        category_label.add_css_class("heading");
        category_label.set_halign(gtk4::Align::Start);
        category_label.set_margin_top(12);
        sidebar.append(&category_label);

        let app_categories = [
            "All",
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
        ];

        let category_list = gtk4::ListBox::new();
        category_list.add_css_class("navigation-sidebar");

        for category in app_categories {
            let row = gtk4::ListBoxRow::new();
            let label = gtk4::Label::new(Some(category));
            label.set_halign(gtk4::Align::Start);
            label.set_margin_start(12);
            label.set_margin_end(12);
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            row.set_child(Some(&label));
            category_list.append(&row);
        }

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&category_list));
        sidebar.append(&scrolled);

        sidebar
    }

    fn create_featured_page(&self) -> gtk4::ScrolledWindow {
        let scrolled = gtk4::ScrolledWindow::new();
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);

        // Banner carousel
        let carousel = adw::Carousel::new();
        carousel.set_height_request(300);
        carousel.add_css_class("card");

        for i in 0..3 {
            let banner = self.create_banner_item(i);
            carousel.append(&banner);
        }

        let carousel_indicator = adw::CarouselIndicatorDots::new();
        carousel_indicator.set_carousel(Some(&carousel));

        content.append(&carousel);
        content.append(&carousel_indicator);

        // Editor's Choice section
        let editors_choice = self.create_app_section("Editor's Choice", &[
            ("Firefox", "Web Browser", "firefox"),
            ("LibreOffice", "Office Suite", "libreoffice"),
            ("GIMP", "Image Editor", "gimp"),
            ("VLC", "Media Player", "vlc"),
        ]);
        content.append(&editors_choice);

        // Popular Apps section
        let popular = self.create_app_section("Popular Apps", &[
            ("Visual Studio Code", "Code Editor", "code"),
            ("Discord", "Voice Chat", "discord"),
            ("Spotify", "Music Streaming", "spotify"),
            ("Steam", "Gaming Platform", "steam"),
        ]);
        content.append(&popular);

        // Recent Updates section
        let updates = self.create_app_section("Recently Updated", &[
            ("Blender", "3D Modeling", "blender"),
            ("Inkscape", "Vector Graphics", "inkscape"),
            ("Kdenlive", "Video Editor", "kdenlive"),
            ("Audacity", "Audio Editor", "audacity"),
        ]);
        content.append(&updates);

        scrolled.set_child(Some(&content));
        scrolled
    }

    fn create_banner_item(&self, index: u32) -> gtk4::Box {
        let banner = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        banner.set_valign(gtk4::Align::Center);
        banner.set_halign(gtk4::Align::Center);

        let titles = ["Discover Amazing Apps", "Gaming on Linux", "Creative Tools"];
        let subtitles = [
            "Find the best open source software",
            "Play thousands of games with Proton",
            "Professional tools for creators",
        ];

        let title = gtk4::Label::new(Some(titles[index as usize]));
        title.add_css_class("title-1");

        let subtitle = gtk4::Label::new(Some(subtitles[index as usize]));
        subtitle.add_css_class("dim-label");

        let button = gtk4::Button::with_label("Explore");
        button.add_css_class("suggested-action");
        button.add_css_class("pill");

        banner.append(&title);
        banner.append(&subtitle);
        banner.append(&button);

        banner
    }

    fn create_app_section(&self, title: &str, apps: &[(&str, &str, &str)]) -> gtk4::Box {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 12);

        // Section header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let label = gtk4::Label::new(Some(title));
        label.add_css_class("title-2");
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);

        let see_all = gtk4::Button::with_label("See All");
        see_all.add_css_class("flat");

        header.append(&label);
        header.append(&see_all);
        section.append(&header);

        // App grid
        let flow_box = gtk4::FlowBox::new();
        flow_box.set_selection_mode(gtk4::SelectionMode::None);
        flow_box.set_homogeneous(true);
        flow_box.set_max_children_per_line(4);
        flow_box.set_min_children_per_line(2);
        flow_box.set_column_spacing(12);
        flow_box.set_row_spacing(12);

        for (name, description, icon) in apps {
            let card = AppCard::new(name, description, icon);
            flow_box.append(&card);
        }

        section.append(&flow_box);
        section
    }

    fn create_categories_page(&self) -> gtk4::ScrolledWindow {
        let scrolled = gtk4::ScrolledWindow::new();
        let flow_box = gtk4::FlowBox::new();
        flow_box.set_selection_mode(gtk4::SelectionMode::None);
        flow_box.set_homogeneous(true);
        flow_box.set_max_children_per_line(4);
        flow_box.set_column_spacing(24);
        flow_box.set_row_spacing(24);
        flow_box.set_margin_start(24);
        flow_box.set_margin_end(24);
        flow_box.set_margin_top(24);
        flow_box.set_margin_bottom(24);

        let categories = [
            ("audio-x-generic-symbolic", "Audio & Video", "Media players, editors"),
            ("applications-engineering-symbolic", "Development", "IDEs, tools, SDKs"),
            ("accessories-dictionary-symbolic", "Education", "Learning tools"),
            ("applications-games-symbolic", "Games", "Play on Linux"),
            ("applications-graphics-symbolic", "Graphics", "Image and vector editors"),
            ("network-workgroup-symbolic", "Network", "Browsers, email, chat"),
            ("x-office-document-symbolic", "Office", "Documents, spreadsheets"),
            ("applications-science-symbolic", "Science", "Research tools"),
            ("preferences-system-symbolic", "System", "System utilities"),
            ("applications-utilities-symbolic", "Utilities", "Everyday tools"),
        ];

        for (icon, name, desc) in categories {
            let card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            card.add_css_class("card");
            card.set_width_request(200);
            card.set_height_request(150);
            card.set_valign(gtk4::Align::Center);
            card.set_halign(gtk4::Align::Center);

            let image = gtk4::Image::from_icon_name(icon);
            image.set_pixel_size(64);

            let label = gtk4::Label::new(Some(name));
            label.add_css_class("title-4");

            let description = gtk4::Label::new(Some(desc));
            description.add_css_class("dim-label");
            description.add_css_class("caption");

            card.append(&image);
            card.append(&label);
            card.append(&description);

            flow_box.append(&card);
        }

        scrolled.set_child(Some(&flow_box));
        scrolled
    }

    fn create_installed_page(&self) -> gtk4::Box {
        let page = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        page.set_margin_start(24);
        page.set_margin_end(24);
        page.set_margin_top(24);

        let status = adw::StatusPage::builder()
            .icon_name("folder-download-symbolic")
            .title("Installed Applications")
            .description("Loading installed applications...")
            .build();

        page.append(&status);
        page
    }

    fn create_updates_page(&self) -> gtk4::Box {
        let page = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        page.set_margin_start(24);
        page.set_margin_end(24);
        page.set_margin_top(24);

        let status = adw::StatusPage::builder()
            .icon_name("software-update-available-symbolic")
            .title("Software Updates")
            .description("All applications are up to date")
            .build();

        let check_button = gtk4::Button::with_label("Check for Updates");
        check_button.add_css_class("suggested-action");
        check_button.add_css_class("pill");
        check_button.set_halign(gtk4::Align::Center);
        status.set_child(Some(&check_button));

        page.append(&status);
        page
    }

    fn create_app_menu(&self) -> gio::Menu {
        let menu = gio::Menu::new();

        let section1 = gio::Menu::new();
        section1.append(Some("Refresh"), Some("app.refresh"));
        section1.append(Some("Repositories"), Some("app.repositories"));
        menu.append_section(None, &section1);

        let section2 = gio::Menu::new();
        section2.append(Some("Preferences"), Some("app.preferences"));
        section2.append(Some("Keyboard Shortcuts"), Some("win.show-help-overlay"));
        section2.append(Some("About Winux Store"), Some("app.about"));
        menu.append_section(None, &section2);

        menu
    }

    fn setup_actions(&self) {
        // Refresh action
        let refresh_action = gio::SimpleAction::new("refresh", None);
        let app_weak = self.downgrade();
        refresh_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.refresh_packages();
            }
        });
        self.add_action(&refresh_action);

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
        prefs_action.connect_activate(|_, _| {
            info!("Opening preferences");
        });
        self.add_action(&prefs_action);

        // Repositories action
        let repos_action = gio::SimpleAction::new("repositories", None);
        repos_action.connect_activate(|_, _| {
            info!("Opening repositories");
        });
        self.add_action(&repos_action);

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
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("app.refresh", &["<Control>r", "F5"]);
        self.set_accels_for_action("app.preferences", &["<Control>comma"]);
    }

    fn perform_search(&self, query: &str) {
        let mut state = self.imp().state.borrow_mut();
        state.search_query = if query.is_empty() {
            None
        } else {
            Some(query.to_string())
        };
        debug!("Search query: {:?}", state.search_query);
    }

    fn refresh_packages(&self) {
        info!("Refreshing package list");
        self.show_toast("Refreshing package list...");
    }

    pub fn show_toast(&self, message: &str) {
        if let Some(toast_overlay) = self.imp().toast_overlay.get() {
            let toast = adw::Toast::new(message);
            toast_overlay.add_toast(toast);
        }
    }

    pub fn show_app_details(&self, package: &AppPackage) {
        if let Some(navigation_view) = self.imp().navigation_view.get() {
            let details_page = AppDetailsPage::new(package);
            let nav_page = adw::NavigationPage::builder()
                .title(&package.name)
                .child(&details_page)
                .build();
            navigation_view.push(&nav_page);
        }
    }

    fn show_about_dialog(&self) {
        let window = self.imp().window.get();

        let about = adw::AboutDialog::builder()
            .application_name("Winux Store")
            .application_icon("system-software-install")
            .version("1.0.0")
            .developer_name("Winux Team")
            .copyright("© 2024 Winux Project")
            .license_type(gtk4::License::Gpl30)
            .website("https://winux.org")
            .issue_url("https://github.com/winux/winux-store/issues")
            .build();

        about.add_credit_section(Some("Created by"), &["Winux Development Team"]);

        if let Some(win) = window {
            about.present(Some(win));
        }
    }
}
