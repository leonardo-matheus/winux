// Winux Files - Modern file manager
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Label, ListView, Orientation, HeaderBar, Paned, ScrolledWindow, SingleSelection, SignalListItemFactory, ListItem};
use libadwaita as adw;
use gio::ListStore;
use glib::Object;
use std::path::PathBuf;

const APP_ID: &str = "org.winux.files";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    let back_btn = Button::builder().icon_name("go-previous-symbolic").tooltip_text("Back").build();
    let forward_btn = Button::builder().icon_name("go-next-symbolic").tooltip_text("Forward").build();
    let home_btn = Button::builder().icon_name("go-home-symbolic").tooltip_text("Home").build();

    let nav_box = Box::new(Orientation::Horizontal, 0);
    nav_box.add_css_class("linked");
    nav_box.append(&back_btn);
    nav_box.append(&forward_btn);
    nav_box.append(&home_btn);
    header.pack_start(&nav_box);

    let menu_btn = Button::builder().icon_name("open-menu-symbolic").build();
    header.pack_end(&menu_btn);

    // Sidebar
    let sidebar = Box::new(Orientation::Vertical, 6);
    sidebar.set_margin_start(12);
    sidebar.set_margin_end(12);
    sidebar.set_margin_top(12);
    sidebar.set_width_request(200);

    let places = ["Home", "Documents", "Downloads", "Pictures", "Music", "Videos", "Trash"];
    for place in places {
        let btn = Button::builder().label(place).has_frame(false).build();
        sidebar.append(&btn);
    }

    // File list
    let store = ListStore::new::<glib::BoxedAnyObject>();
    let selection = SingleSelection::new(Some(store.clone()));

    let factory = SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let label = Label::new(None);
        label.set_xalign(0.0);
        item.set_child(Some(&label));
    });
    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        if let Some(obj) = item.item() {
            if let Some(label) = item.child().and_then(|c| c.downcast::<Label>().ok()) {
                label.set_text("File");
            }
        }
    });

    let list_view = ListView::new(Some(selection), Some(factory));
    let scrolled = ScrolledWindow::builder().child(&list_view).hexpand(true).vexpand(true).build();

    let content = Box::new(Orientation::Vertical, 0);
    let path_label = Label::new(Some(&dirs::home_dir().unwrap_or_default().to_string_lossy()));
    path_label.set_xalign(0.0);
    path_label.set_margin_all(12);
    content.append(&path_label);
    content.append(&scrolled);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(&content));
    paned.set_position(200);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Files")
        .default_width(1000)
        .default_height(700)
        .build();

    window.set_titlebar(Some(&header));
    window.set_child(Some(&paned));

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    window.present();
}
