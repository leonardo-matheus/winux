// Main application window

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::pages;

pub struct FirewallWindow {
    window: ApplicationWindow,
}

impl FirewallWindow {
    pub fn new(app: &Application) -> Self {
        let header = HeaderBar::new();

        let stack = ViewStack::new();
        stack.set_vexpand(true);

        // Overview Page - Status and toggle
        let overview_page = pages::OverviewPage::new();
        stack.add_titled(overview_page.widget(), Some("overview"), "Visao Geral")
            .set_icon_name(Some("security-high-symbolic"));

        // Rules Page - Firewall rules management
        let rules_page = pages::RulesPage::new();
        stack.add_titled(rules_page.widget(), Some("rules"), "Regras")
            .set_icon_name(Some("view-list-symbolic"));

        // Apps Page - Application permissions
        let apps_page = pages::AppsPage::new();
        stack.add_titled(apps_page.widget(), Some("apps"), "Aplicativos")
            .set_icon_name(Some("application-x-executable-symbolic"));

        // Logs Page - Firewall logs
        let logs_page = pages::LogsPage::new();
        stack.add_titled(logs_page.widget(), Some("logs"), "Logs")
            .set_icon_name(Some("document-open-recent-symbolic"));

        let switcher = ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Firewall")
            .default_width(950)
            .default_height(750)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
