// Main window for Winux Updater

use gtk4::prelude::*;
use gtk4::{Application, Box, Button, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher, Toast, ToastOverlay};
use std::cell::RefCell;
use std::rc::Rc;

use crate::pages::{UpdatesPage, HistoryPage, SettingsPage, DriversPage};
use crate::backend::UpdateManager;

pub struct UpdaterWindow {
    window: ApplicationWindow,
    toast_overlay: ToastOverlay,
    update_manager: Rc<RefCell<UpdateManager>>,
}

impl UpdaterWindow {
    pub fn new(app: &Application) -> Self {
        let update_manager = Rc::new(RefCell::new(UpdateManager::new()));

        // Header bar with refresh button
        let header = HeaderBar::new();

        let refresh_btn = Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Verificar Atualizacoes")
            .build();
        refresh_btn.add_css_class("flat");
        header.pack_start(&refresh_btn);

        // View stack for pages
        let stack = ViewStack::new();
        stack.set_vexpand(true);

        // Create pages
        let updates_page = UpdatesPage::new(update_manager.clone());
        stack.add_titled(updates_page.widget(), Some("updates"), "Atualizacoes")
            .set_icon_name(Some("software-update-available-symbolic"));

        let drivers_page = DriversPage::new(update_manager.clone());
        stack.add_titled(drivers_page.widget(), Some("drivers"), "Drivers")
            .set_icon_name(Some("drive-harddisk-symbolic"));

        let history_page = HistoryPage::new(update_manager.clone());
        stack.add_titled(history_page.widget(), Some("history"), "Historico")
            .set_icon_name(Some("document-open-recent-symbolic"));

        let settings_page = SettingsPage::new(update_manager.clone());
        stack.add_titled(settings_page.widget(), Some("settings"), "Configuracoes")
            .set_icon_name(Some("emblem-system-symbolic"));

        // View switcher
        let switcher = ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();
        header.set_title_widget(Some(&switcher));

        // Main layout
        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        // Toast overlay for notifications
        let toast_overlay = ToastOverlay::new();
        toast_overlay.set_child(Some(&main_box));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Atualizacoes do Sistema")
            .default_width(900)
            .default_height(650)
            .content(&toast_overlay)
            .build();

        window.set_titlebar(Some(&header));

        // Connect refresh button
        let stack_clone = stack.clone();
        let toast_overlay_clone = toast_overlay.clone();
        let update_manager_clone = update_manager.clone();
        refresh_btn.connect_clicked(move |_| {
            let toast = Toast::new("Verificando atualizacoes...");
            toast_overlay_clone.add_toast(toast);

            // Trigger refresh on current page
            if let Some(visible_name) = stack_clone.visible_child_name() {
                match visible_name.as_str() {
                    "updates" => {
                        // Refresh updates
                        tracing::info!("Refreshing updates list");
                    }
                    "drivers" => {
                        // Refresh drivers
                        tracing::info!("Refreshing drivers list");
                    }
                    _ => {}
                }
            }
        });

        Self {
            window,
            toast_overlay,
            update_manager,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn show_toast(&self, message: &str) {
        let toast = Toast::new(message);
        self.toast_overlay.add_toast(toast);
    }
}
