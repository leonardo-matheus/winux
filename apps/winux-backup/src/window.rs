// Main window for Winux Backup

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::pages::{
    OverviewPage, CreatePage, RestorePage, SchedulePage, SettingsPage,
};

pub struct BackupWindow {
    window: ApplicationWindow,
}

impl BackupWindow {
    pub fn new(app: &Application) -> Self {
        let header = HeaderBar::new();

        let stack = ViewStack::new();
        stack.set_vexpand(true);

        // Overview Page - Dashboard
        let overview_page = OverviewPage::new();
        stack.add_titled(overview_page.widget(), Some("overview"), "Visao Geral")
            .set_icon_name(Some("view-grid-symbolic"));

        // Create Backup Page
        let create_page = CreatePage::new();
        stack.add_titled(create_page.widget(), Some("create"), "Criar Backup")
            .set_icon_name(Some("list-add-symbolic"));

        // Restore Page
        let restore_page = RestorePage::new();
        stack.add_titled(restore_page.widget(), Some("restore"), "Restaurar")
            .set_icon_name(Some("edit-undo-symbolic"));

        // Schedule Page
        let schedule_page = SchedulePage::new();
        stack.add_titled(schedule_page.widget(), Some("schedule"), "Agendamento")
            .set_icon_name(Some("alarm-symbolic"));

        // Settings Page
        let settings_page = SettingsPage::new();
        stack.add_titled(settings_page.widget(), Some("settings"), "Configuracoes")
            .set_icon_name(Some("emblem-system-symbolic"));

        let switcher = ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Winux Backup")
            .default_width(1000)
            .default_height(700)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
