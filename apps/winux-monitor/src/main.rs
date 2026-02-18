//! Winux Monitor - System monitor and task manager for Winux OS
//!
//! A comprehensive system monitoring application that displays real-time
//! information about CPU, memory, disk, network, and running processes.

mod performance;
mod processes;
mod startup;
mod system_info;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::{gio, glib};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use performance::PerformanceView;
use processes::ProcessesView;
use startup::StartupView;
use system_info::SystemInfoView;

/// Application ID for D-Bus and desktop integration
const APP_ID: &str = "com.winux.Monitor";

/// Refresh interval in milliseconds
const REFRESH_INTERVAL_MS: u64 = 1000;

fn main() -> glib::ExitCode {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!("Starting Winux Monitor");

    // Create and run the application
    let app = MonitorApplication::new(APP_ID);
    app.run()
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct MonitorApplication {
        pub window: OnceCell<adw::ApplicationWindow>,
        pub processes_view: OnceCell<ProcessesView>,
        pub performance_view: OnceCell<PerformanceView>,
        pub startup_view: OnceCell<StartupView>,
        pub system_info_view: OnceCell<SystemInfoView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonitorApplication {
        const NAME: &'static str = "WinuxMonitorApplication";
        type Type = super::MonitorApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MonitorApplication {}

    impl ApplicationImpl for MonitorApplication {
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

    impl GtkApplicationImpl for MonitorApplication {}
    impl AdwApplicationImpl for MonitorApplication {}
}

glib::wrapper! {
    pub struct MonitorApplication(ObjectSubclass<imp::MonitorApplication>)
        @extends adw::Application, gtk4::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MonitorApplication {
    pub fn new(app_id: &str) -> Self {
        glib::Object::builder()
            .property("application-id", app_id)
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .build()
    }

    fn setup_window(&self) {
        let imp = self.imp();

        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(self)
            .title("Winux Monitor")
            .default_width(1100)
            .default_height(700)
            .build();

        // Create main layout
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Create header bar
        let header = adw::HeaderBar::new();

        // View switcher in header
        let view_switcher = adw::ViewSwitcher::new();
        view_switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
        header.set_title_widget(Some(&view_switcher));

        // Menu button
        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&self.create_app_menu())
            .build();
        header.pack_end(&menu_button);

        // Search button
        let search_button = gtk4::ToggleButton::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search processes")
            .build();
        header.pack_end(&search_button);

        main_box.append(&header);

        // Create view stack
        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        // Processes view
        let processes_view = ProcessesView::new();
        stack.add_titled(&processes_view, Some("processes"), "Processes")
            .set_icon_name(Some("view-list-symbolic"));
        imp.processes_view.set(processes_view.clone()).unwrap();

        // Performance view
        let performance_view = PerformanceView::new();
        stack.add_titled(&performance_view, Some("performance"), "Performance")
            .set_icon_name(Some("utilities-system-monitor-symbolic"));
        imp.performance_view.set(performance_view.clone()).unwrap();

        // Startup view
        let startup_view = StartupView::new();
        stack.add_titled(&startup_view, Some("startup"), "Startup Apps")
            .set_icon_name(Some("media-playback-start-symbolic"));
        imp.startup_view.set(startup_view.clone()).unwrap();

        // System info view
        let system_info_view = SystemInfoView::new();
        stack.add_titled(&system_info_view, Some("system"), "System")
            .set_icon_name(Some("computer-symbolic"));
        imp.system_info_view.set(system_info_view.clone()).unwrap();

        view_switcher.set_stack(Some(&stack));

        // View switcher bar for mobile
        let view_switcher_bar = adw::ViewSwitcherBar::new();
        view_switcher_bar.set_stack(Some(&stack));

        // Breakpoint for adaptive layout
        let breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::parse("max-width: 550sp").unwrap());
        breakpoint.add_setter(&view_switcher, "visible", &false.to_value());
        breakpoint.add_setter(&view_switcher_bar, "reveal", &true.to_value());

        main_box.append(&stack);
        main_box.append(&view_switcher_bar);

        window.add_breakpoint(breakpoint);
        window.set_content(Some(&main_box));

        imp.window.set(window.clone()).unwrap();

        // Setup refresh timer
        self.setup_refresh_timer();

        window.present();
        info!("Winux Monitor window initialized");
    }

    fn setup_refresh_timer(&self) {
        let imp = self.imp();

        // Create a refresh timer
        glib::timeout_add_local(
            Duration::from_millis(REFRESH_INTERVAL_MS),
            glib::clone!(
                #[weak(rename_to = app)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move || {
                    app.refresh_data();
                    glib::ControlFlow::Continue
                }
            ),
        );
    }

    fn refresh_data(&self) {
        let imp = self.imp();

        // Refresh processes
        if let Some(view) = imp.processes_view.get() {
            view.refresh();
        }

        // Refresh performance
        if let Some(view) = imp.performance_view.get() {
            view.refresh();
        }
    }

    fn create_app_menu(&self) -> gio::Menu {
        let menu = gio::Menu::new();

        let section1 = gio::Menu::new();
        section1.append(Some("Refresh"), Some("app.refresh"));
        menu.append_section(None, &section1);

        let section2 = gio::Menu::new();
        section2.append(Some("Preferences"), Some("app.preferences"));
        section2.append(Some("Keyboard Shortcuts"), Some("win.show-help-overlay"));
        section2.append(Some("About Winux Monitor"), Some("app.about"));
        menu.append_section(None, &section2);

        menu
    }

    fn setup_actions(&self) {
        // Refresh action
        let refresh_action = gio::SimpleAction::new("refresh", None);
        let app_weak = self.downgrade();
        refresh_action.connect_activate(move |_, _| {
            if let Some(app) = app_weak.upgrade() {
                app.refresh_data();
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

    fn show_about_dialog(&self) {
        let window = self.imp().window.get();

        let about = adw::AboutDialog::builder()
            .application_name("Winux Monitor")
            .application_icon("utilities-system-monitor")
            .version("1.0.0")
            .developer_name("Winux Team")
            .copyright("© 2024 Winux Project")
            .license_type(gtk4::License::Gpl30)
            .website("https://winux.org")
            .issue_url("https://github.com/winux/winux-monitor/issues")
            .build();

        about.add_credit_section(Some("Created by"), &["Winux Development Team"]);

        if let Some(win) = window {
            about.present(Some(win));
        }
    }
}
