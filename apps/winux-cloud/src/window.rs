// Main window for Winux Cloud

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::pages::{
    AccountsPage, SyncPage, FilesPage, SettingsPage, ActivityPage,
};

pub struct CloudWindow {
    window: ApplicationWindow,
}

impl CloudWindow {
    pub fn new(app: &Application) -> Self {
        let header = HeaderBar::new();

        let stack = ViewStack::new();
        stack.set_vexpand(true);

        // Accounts Page - Connected cloud accounts
        let accounts_page = AccountsPage::new();
        stack.add_titled(accounts_page.widget(), Some("accounts"), "Contas")
            .set_icon_name(Some("system-users-symbolic"));

        // Sync Page - Sync status
        let sync_page = SyncPage::new();
        stack.add_titled(sync_page.widget(), Some("sync"), "Sincronizacao")
            .set_icon_name(Some("emblem-synchronizing-symbolic"));

        // Files Page - Cloud files
        let files_page = FilesPage::new();
        stack.add_titled(files_page.widget(), Some("files"), "Arquivos")
            .set_icon_name(Some("folder-remote-symbolic"));

        // Activity Page - Activity log
        let activity_page = ActivityPage::new();
        stack.add_titled(activity_page.widget(), Some("activity"), "Atividade")
            .set_icon_name(Some("view-list-symbolic"));

        // Settings Page - Sync settings
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
            .title("Winux Cloud")
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
