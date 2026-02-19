// Main window for Winux Fonts

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation, SearchEntry};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};
use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::FontManager;
use crate::pages::{browse, preview, compare, install};

pub fn build_ui(app: &Application) {
    // Initialize font manager
    let font_manager = Rc::new(RefCell::new(FontManager::new()));
    font_manager.borrow_mut().scan_fonts();

    // Header bar
    let header = HeaderBar::new();

    // Search entry
    let search_entry = SearchEntry::builder()
        .placeholder_text("Buscar fontes...")
        .hexpand(false)
        .width_request(250)
        .build();
    header.pack_start(&search_entry);

    // View stack
    let stack = ViewStack::new();
    stack.set_vexpand(true);

    // Browse page
    let browse_page = browse::create_page(font_manager.clone(), search_entry.clone());
    stack.add_titled(&browse_page, Some("browse"), "Navegar")
        .set_icon_name(Some("font-x-generic-symbolic"));

    // Preview page
    let preview_page = preview::create_page(font_manager.clone());
    stack.add_titled(&preview_page, Some("preview"), "Visualizar")
        .set_icon_name(Some("document-page-setup-symbolic"));

    // Compare page
    let compare_page = compare::create_page(font_manager.clone());
    stack.add_titled(&compare_page, Some("compare"), "Comparar")
        .set_icon_name(Some("view-dual-symbolic"));

    // Install page
    let install_page = install::create_page(font_manager.clone());
    stack.add_titled(&install_page, Some("install"), "Instalar")
        .set_icon_name(Some("document-save-symbolic"));

    // View switcher
    let switcher = ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();

    header.set_title_widget(Some(&switcher));

    // Main layout
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.append(&stack);

    // Window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Fontes")
        .default_width(1100)
        .default_height(750)
        .content(&main_box)
        .build();

    window.set_titlebar(Some(&header));
    window.present();
}
