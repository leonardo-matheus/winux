use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Button, FlowBox, Frame, Image,
    Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SearchEntry, Separator,
};
use libadwaita as adw;
use adw::prelude::*;

const APP_ID: &str = "org.winux.store";

fn main() -> gtk::glib::ExitCode {
    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // Force dark theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

    // Main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Store")
        .default_width(1200)
        .default_height(800)
        .build();

    // Main horizontal box (sidebar + content)
    let main_box = Box::new(Orientation::Horizontal, 0);

    // Create sidebar
    let sidebar = create_sidebar();
    main_box.append(&sidebar);

    // Separator between sidebar and content
    let separator = Separator::new(Orientation::Vertical);
    main_box.append(&separator);

    // Create main content area
    let content = create_content_area();
    main_box.append(&content);

    window.set_child(Some(&main_box));
    window.present();
}

fn create_sidebar() -> Box {
    let sidebar = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(250)
        .css_classes(vec!["sidebar"])
        .build();

    // Sidebar header
    let header_label = Label::builder()
        .label("Categories")
        .css_classes(vec!["title-2"])
        .margin_top(20)
        .margin_bottom(10)
        .margin_start(15)
        .halign(gtk::Align::Start)
        .build();
    sidebar.append(&header_label);

    // Categories list
    let categories_list = ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(vec!["navigation-sidebar"])
        .margin_start(10)
        .margin_end(10)
        .build();

    let categories = vec![
        ("view-grid-symbolic", "All Apps"),
        ("applications-development-symbolic", "Development"),
        ("applications-games-symbolic", "Games"),
        ("applications-internet-symbolic", "Internet"),
        ("applications-multimedia-symbolic", "Multimedia"),
        ("applications-graphics-symbolic", "Graphics"),
        ("applications-office-symbolic", "Office"),
        ("applications-system-symbolic", "System"),
        ("applications-utilities-symbolic", "Utilities"),
        ("applications-education-symbolic", "Education"),
        ("applications-science-symbolic", "Science"),
    ];

    for (icon_name, category_name) in categories {
        let row = create_category_row(icon_name, category_name);
        categories_list.append(&row);
    }

    // Select first row by default
    if let Some(first_row) = categories_list.row_at_index(0) {
        categories_list.select_row(Some(&first_row));
    }

    sidebar.append(&categories_list);

    // Spacer
    let spacer = Box::builder()
        .vexpand(true)
        .build();
    sidebar.append(&spacer);

    // Updates section at bottom
    let updates_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_start(15)
        .margin_end(15)
        .margin_bottom(20)
        .build();

    let updates_icon = Image::from_icon_name("software-update-available-symbolic");
    let updates_label = Label::new(Some("3 Updates Available"));
    updates_label.set_css_classes(&["dim-label"]);

    updates_box.append(&updates_icon);
    updates_box.append(&updates_label);
    sidebar.append(&updates_box);

    sidebar
}

fn create_category_row(icon_name: &str, label_text: &str) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let icon = Image::from_icon_name(icon_name);
    let label = Label::new(Some(label_text));
    label.set_halign(gtk::Align::Start);

    row_box.append(&icon);
    row_box.append(&label);

    let row = ListBoxRow::builder()
        .child(&row_box)
        .build();

    row
}

fn create_content_area() -> Box {
    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .build();

    // Header with search
    let header = create_header();
    content.append(&header);

    // Scrollable content
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    let inner_content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(10)
        .margin_bottom(20)
        .spacing(20)
        .build();

    // Featured section
    let featured_section = create_featured_section();
    inner_content.append(&featured_section);

    // Popular apps section
    let popular_section = create_apps_section("Popular Apps", get_popular_apps());
    inner_content.append(&popular_section);

    // New releases section
    let new_section = create_apps_section("New Releases", get_new_apps());
    inner_content.append(&new_section);

    // Editor's choice section
    let editors_section = create_apps_section("Editor's Choice", get_editors_choice());
    inner_content.append(&editors_section);

    scrolled.set_child(Some(&inner_content));
    content.append(&scrolled);

    content
}

fn create_header() -> Box {
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .margin_start(20)
        .margin_end(20)
        .margin_top(15)
        .margin_bottom(15)
        .build();

    // Title
    let title = Label::builder()
        .label("Winux Store")
        .css_classes(vec!["title-1"])
        .halign(gtk::Align::Start)
        .build();
    header.append(&title);

    // Spacer
    let spacer = Box::builder()
        .hexpand(true)
        .build();
    header.append(&spacer);

    // Search entry
    let search = SearchEntry::builder()
        .placeholder_text("Search apps...")
        .width_request(300)
        .build();
    header.append(&search);

    // User menu button
    let user_button = Button::builder()
        .icon_name("open-menu-symbolic")
        .css_classes(vec!["flat"])
        .build();
    header.append(&user_button);

    header
}

fn create_featured_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .build();

    // Section title
    let title = Label::builder()
        .label("Featured")
        .css_classes(vec!["title-2"])
        .halign(gtk::Align::Start)
        .build();
    section.append(&title);

    // Featured banner
    let banner = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let banner_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(20)
        .margin_start(25)
        .margin_end(25)
        .margin_top(25)
        .margin_bottom(25)
        .build();

    // Featured app icon placeholder
    let featured_icon = Box::builder()
        .width_request(128)
        .height_request(128)
        .css_classes(vec!["card"])
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let icon_label = Label::builder()
        .label("VS")
        .css_classes(vec!["title-1"])
        .build();
    featured_icon.append(&icon_label);

    banner_content.append(&featured_icon);

    // Featured app info
    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .valign(gtk::Align::Center)
        .hexpand(true)
        .build();

    let app_name = Label::builder()
        .label("Visual Studio Code")
        .css_classes(vec!["title-1"])
        .halign(gtk::Align::Start)
        .build();
    info_box.append(&app_name);

    let app_desc = Label::builder()
        .label("Code editing. Redefined. Free. Built on open source. Runs everywhere.")
        .css_classes(vec!["dim-label"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    info_box.append(&app_desc);

    let app_category = Label::builder()
        .label("Development")
        .css_classes(vec!["caption"])
        .halign(gtk::Align::Start)
        .build();
    info_box.append(&app_category);

    let buttons_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(10)
        .build();

    let install_btn = Button::builder()
        .label("Install")
        .css_classes(vec!["suggested-action", "pill"])
        .build();
    buttons_box.append(&install_btn);

    let details_btn = Button::builder()
        .label("Details")
        .css_classes(vec!["pill"])
        .build();
    buttons_box.append(&details_btn);

    info_box.append(&buttons_box);
    banner_content.append(&info_box);

    banner.set_child(Some(&banner_content));
    section.append(&banner);

    section
}

fn create_apps_section(title_text: &str, apps: Vec<AppInfo>) -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(15)
        .build();

    // Section header
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let title = Label::builder()
        .label(title_text)
        .css_classes(vec!["title-2"])
        .halign(gtk::Align::Start)
        .build();
    header.append(&title);

    let spacer = Box::builder()
        .hexpand(true)
        .build();
    header.append(&spacer);

    let see_all = Button::builder()
        .label("See All")
        .css_classes(vec!["flat"])
        .build();
    header.append(&see_all);

    section.append(&header);

    // Apps flow box
    let flow_box = FlowBox::builder()
        .homogeneous(true)
        .column_spacing(15)
        .row_spacing(15)
        .min_children_per_line(2)
        .max_children_per_line(4)
        .selection_mode(gtk::SelectionMode::None)
        .build();

    for app in apps {
        let card = create_app_card(&app);
        flow_box.append(&card);
    }

    section.append(&flow_box);

    section
}

struct AppInfo {
    name: &'static str,
    description: &'static str,
    icon_text: &'static str,
    category: &'static str,
}

fn create_app_card(app: &AppInfo) -> Frame {
    let card = Frame::builder()
        .css_classes(vec!["card"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(15)
        .margin_end(15)
        .margin_top(15)
        .margin_bottom(15)
        .width_request(200)
        .build();

    // App icon placeholder
    let icon_frame = Box::builder()
        .width_request(64)
        .height_request(64)
        .css_classes(vec!["card"])
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let icon_label = Label::builder()
        .label(app.icon_text)
        .css_classes(vec!["title-3"])
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    icon_frame.append(&icon_label);

    content.append(&icon_frame);

    // App name
    let name_label = Label::builder()
        .label(app.name)
        .css_classes(vec!["title-4"])
        .halign(gtk::Align::Center)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    content.append(&name_label);

    // App description
    let desc_label = Label::builder()
        .label(app.description)
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk::Align::Center)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .lines(2)
        .wrap(true)
        .justify(gtk::Justification::Center)
        .build();
    content.append(&desc_label);

    // Category
    let category_label = Label::builder()
        .label(app.category)
        .css_classes(vec!["caption"])
        .halign(gtk::Align::Center)
        .build();
    content.append(&category_label);

    // Install button
    let install_btn = Button::builder()
        .label("Install")
        .css_classes(vec!["suggested-action"])
        .margin_top(5)
        .build();
    content.append(&install_btn);

    card.set_child(Some(&content));
    card
}

fn get_popular_apps() -> Vec<AppInfo> {
    vec![
        AppInfo {
            name: "Firefox",
            description: "Fast, private & safe web browser",
            icon_text: "FF",
            category: "Internet",
        },
        AppInfo {
            name: "GIMP",
            description: "GNU Image Manipulation Program",
            icon_text: "GP",
            category: "Graphics",
        },
        AppInfo {
            name: "VLC",
            description: "Multimedia player and framework",
            icon_text: "VLC",
            category: "Multimedia",
        },
        AppInfo {
            name: "LibreOffice",
            description: "Powerful office suite",
            icon_text: "LO",
            category: "Office",
        },
    ]
}

fn get_new_apps() -> Vec<AppInfo> {
    vec![
        AppInfo {
            name: "Blender",
            description: "3D creation suite",
            icon_text: "BL",
            category: "Graphics",
        },
        AppInfo {
            name: "Kdenlive",
            description: "Video editor by KDE",
            icon_text: "KD",
            category: "Multimedia",
        },
        AppInfo {
            name: "Godot",
            description: "Game engine for 2D and 3D",
            icon_text: "GO",
            category: "Development",
        },
        AppInfo {
            name: "Inkscape",
            description: "Vector graphics editor",
            icon_text: "IN",
            category: "Graphics",
        },
    ]
}

fn get_editors_choice() -> Vec<AppInfo> {
    vec![
        AppInfo {
            name: "Telegram",
            description: "Cloud-based messaging app",
            icon_text: "TG",
            category: "Internet",
        },
        AppInfo {
            name: "OBS Studio",
            description: "Video recording & streaming",
            icon_text: "OBS",
            category: "Multimedia",
        },
        AppInfo {
            name: "Audacity",
            description: "Audio editor and recorder",
            icon_text: "AU",
            category: "Multimedia",
        },
        AppInfo {
            name: "Krita",
            description: "Digital painting application",
            icon_text: "KR",
            category: "Graphics",
        },
    ]
}
