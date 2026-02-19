// Main window for Winux Printers

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::cups::CupsManager;
use crate::pages::{PrintersPage, AddPrinterPage, JobsPage, SettingsPage};

pub struct PrinterWindow {
    window: adw::ApplicationWindow,
}

impl PrinterWindow {
    pub fn new(app: &Application) -> Self {
        let manager = Rc::new(RefCell::new(CupsManager::new()));

        let header = adw::HeaderBar::new();

        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        // Printers page (configured printers)
        let printers_page = PrintersPage::new(manager.clone());
        stack.add_titled(printers_page.widget(), Some("printers"), "Impressoras")
            .set_icon_name(Some("printer-symbolic"));

        // Add printer page (discovery and setup)
        let add_page = AddPrinterPage::new(manager.clone());
        stack.add_titled(add_page.widget(), Some("add"), "Adicionar")
            .set_icon_name(Some("list-add-symbolic"));

        // Jobs page (print queue)
        let jobs_page = JobsPage::new(manager.clone());
        stack.add_titled(jobs_page.widget(), Some("jobs"), "Fila")
            .set_icon_name(Some("view-list-symbolic"));

        // Settings page
        let settings_page = SettingsPage::new(manager.clone());
        stack.add_titled(settings_page.widget(), Some("settings"), "Configuracoes")
            .set_icon_name(Some("emblem-system-symbolic"));

        let switcher = adw::ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        // Refresh button
        let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Atualizar lista de impressoras"));
        refresh_btn.connect_clicked(move |_| {
            tracing::info!("Refreshing printer list");
        });
        header.pack_end(&refresh_btn);

        // Test print button
        let test_btn = gtk4::Button::from_icon_name("document-print-symbolic");
        test_btn.set_tooltip_text(Some("Imprimir pagina de teste"));
        test_btn.connect_clicked(move |_| {
            tracing::info!("Printing test page");
        });
        header.pack_end(&test_btn);

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Impressoras")
            .default_width(900)
            .default_height(650)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
