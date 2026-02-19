// Winux Welcome - Desktop Mode Selection
// Choose between Windows-like, Mac-like, or Linux-like experience

use gtk4::prelude::*;
use gtk4::{Box, Button, CheckButton, Grid, Label, Orientation, Frame};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::WelcomeState;
use crate::ui::option_card;

pub fn create_page(state: Rc<RefCell<WelcomeState>>) -> Box {
    let page = Box::new(Orientation::Vertical, 24);
    page.set_valign(gtk4::Align::Start);
    page.set_margin_top(32);
    page.set_margin_bottom(32);
    page.set_margin_start(48);
    page.set_margin_end(48);

    // Title
    let title = Label::new(Some("Escolha seu Estilo"));
    title.add_css_class("title-1");
    page.append(&title);

    // Subtitle
    let subtitle = Label::new(Some("Selecione a experiencia de desktop que mais combina com voce"));
    subtitle.add_css_class("dim-label");
    page.append(&subtitle);

    // Mode selection cards
    let modes_box = Box::new(Orientation::Horizontal, 24);
    modes_box.set_halign(gtk4::Align::Center);
    modes_box.set_margin_top(32);

    // Create radio button group
    let windows_radio = CheckButton::new();
    let linux_radio = CheckButton::new();
    let mac_radio = CheckButton::new();

    linux_radio.set_group(Some(&windows_radio));
    mac_radio.set_group(Some(&windows_radio));

    // Windows-like card
    let windows_card = create_mode_card(
        "Windows Like",
        "Barra de tarefas na parte inferior\nMenu Iniciar familiar\nIdeal para quem vem do Windows",
        create_windows_preview(),
        windows_radio.clone(),
    );
    modes_box.append(&windows_card);

    // Linux-like card
    let linux_card = create_mode_card(
        "Linux Like",
        "Barra superior estilo GNOME\nWorkspaces dinamicos\nExperiencia Linux tradicional",
        create_linux_preview(),
        linux_radio.clone(),
    );
    // Set Linux as default
    linux_radio.set_active(true);
    modes_box.append(&linux_card);

    // Mac-like card
    let mac_card = create_mode_card(
        "Mac Like",
        "Dock centralizado na parte inferior\nBotoes coloridos nas janelas\nDesign elegante e minimalista",
        create_mac_preview(),
        mac_radio.clone(),
    );
    modes_box.append(&mac_card);

    page.append(&modes_box);

    // Connect signals to update state
    let state_clone = state.clone();
    windows_radio.connect_toggled(move |btn| {
        if btn.is_active() {
            state_clone.borrow_mut().desktop_mode = Some("Windows Like".to_string());
        }
    });

    let state_clone = state.clone();
    linux_radio.connect_toggled(move |btn| {
        if btn.is_active() {
            state_clone.borrow_mut().desktop_mode = Some("Linux Like".to_string());
        }
    });

    let state_clone = state.clone();
    mac_radio.connect_toggled(move |btn| {
        if btn.is_active() {
            state_clone.borrow_mut().desktop_mode = Some("Mac Like".to_string());
        }
    });

    // Set default
    state.borrow_mut().desktop_mode = Some("Linux Like".to_string());

    // Info text
    let info = Label::new(Some("Voce pode mudar o estilo a qualquer momento em Configuracoes > Personalizar"));
    info.add_css_class("dim-label");
    info.add_css_class("caption");
    info.set_margin_top(24);
    page.append(&info);

    page
}

fn create_mode_card(title: &str, description: &str, preview: Box, radio: CheckButton) -> Box {
    let card = Box::new(Orientation::Vertical, 12);
    card.add_css_class("card");
    card.set_size_request(260, 340);

    let inner = Box::new(Orientation::Vertical, 12);
    inner.set_margin_top(16);
    inner.set_margin_bottom(16);
    inner.set_margin_start(16);
    inner.set_margin_end(16);

    // Preview frame
    let preview_frame = Frame::new(None);
    preview_frame.set_size_request(228, 150);
    preview_frame.add_css_class("view");
    preview_frame.set_child(Some(&preview));
    inner.append(&preview_frame);

    // Title
    let title_label = Label::new(Some(title));
    title_label.add_css_class("title-3");
    inner.append(&title_label);

    // Description
    let desc_label = Label::new(Some(description));
    desc_label.add_css_class("dim-label");
    desc_label.add_css_class("caption");
    desc_label.set_wrap(true);
    desc_label.set_justify(gtk4::Justification::Center);
    inner.append(&desc_label);

    // Radio button
    let radio_box = Box::new(Orientation::Horizontal, 8);
    radio_box.set_halign(gtk4::Align::Center);
    radio_box.append(&radio);

    let select_label = Label::new(Some("Selecionar"));
    radio_box.append(&select_label);
    inner.append(&radio_box);

    card.append(&inner);

    // Make entire card clickable
    let gesture = gtk4::GestureClick::new();
    let radio_clone = radio.clone();
    gesture.connect_released(move |_, _, _, _| {
        radio_clone.set_active(true);
    });
    card.add_controller(gesture);

    card
}

fn create_windows_preview() -> Box {
    let preview = Box::new(Orientation::Vertical, 0);
    preview.set_vexpand(true);

    // Desktop area
    let desktop = Box::new(Orientation::Vertical, 0);
    desktop.set_vexpand(true);
    desktop.add_css_class("view");

    // Desktop icons
    let icons_grid = Grid::new();
    icons_grid.set_margin_top(8);
    icons_grid.set_margin_start(8);
    icons_grid.set_column_spacing(8);
    icons_grid.set_row_spacing(8);

    for i in 0..2 {
        let icon = gtk4::Image::from_icon_name("folder-symbolic");
        icon.set_pixel_size(24);
        icons_grid.attach(&icon, 0, i, 1, 1);
    }

    desktop.append(&icons_grid);
    preview.append(&desktop);

    // Taskbar at bottom
    let taskbar = Box::new(Orientation::Horizontal, 4);
    taskbar.add_css_class("toolbar");
    taskbar.set_size_request(-1, 32);
    taskbar.set_margin_start(4);
    taskbar.set_margin_end(4);

    let start_btn = Button::new();
    start_btn.set_icon_name("view-grid-symbolic");
    start_btn.add_css_class("flat");
    taskbar.append(&start_btn);

    let sep = gtk4::Separator::new(Orientation::Vertical);
    taskbar.append(&sep);

    // Pinned apps
    for icon_name in ["folder-symbolic", "terminal-symbolic", "web-browser-symbolic"] {
        let app_btn = Button::new();
        app_btn.set_icon_name(icon_name);
        app_btn.add_css_class("flat");
        taskbar.append(&app_btn);
    }

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    taskbar.append(&spacer);

    // System tray
    let time = Label::new(Some("12:00"));
    time.add_css_class("caption");
    taskbar.append(&time);

    preview.append(&taskbar);
    preview
}

fn create_linux_preview() -> Box {
    let preview = Box::new(Orientation::Vertical, 0);

    // Top bar (GNOME style)
    let topbar = Box::new(Orientation::Horizontal, 4);
    topbar.add_css_class("toolbar");
    topbar.set_size_request(-1, 24);

    let activities = Label::new(Some("Activities"));
    activities.add_css_class("caption");
    activities.set_margin_start(8);
    topbar.append(&activities);

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    topbar.append(&spacer);

    let clock = Label::new(Some("12:00"));
    clock.add_css_class("caption");
    topbar.append(&clock);

    let spacer2 = Box::new(Orientation::Horizontal, 0);
    spacer2.set_hexpand(true);
    topbar.append(&spacer2);

    let indicators = Box::new(Orientation::Horizontal, 4);
    indicators.set_margin_end(8);
    let wifi = gtk4::Image::from_icon_name("network-wireless-symbolic");
    wifi.set_pixel_size(12);
    indicators.append(&wifi);
    let battery = gtk4::Image::from_icon_name("battery-full-symbolic");
    battery.set_pixel_size(12);
    indicators.append(&battery);
    topbar.append(&indicators);

    preview.append(&topbar);

    // Desktop area
    let desktop = Box::new(Orientation::Vertical, 0);
    desktop.set_vexpand(true);
    desktop.add_css_class("view");
    preview.append(&desktop);

    // Dash/dock at bottom (optional in GNOME)
    let dock_container = Box::new(Orientation::Horizontal, 0);
    dock_container.set_halign(gtk4::Align::Center);
    dock_container.set_margin_bottom(8);

    let dock = Box::new(Orientation::Horizontal, 4);
    dock.add_css_class("card");
    dock.set_margin_top(4);
    dock.set_margin_bottom(4);
    dock.set_margin_start(8);
    dock.set_margin_end(8);

    for icon_name in ["view-grid-symbolic", "folder-symbolic", "terminal-symbolic"] {
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(20);
        icon.set_margin_start(4);
        icon.set_margin_end(4);
        icon.set_margin_top(4);
        icon.set_margin_bottom(4);
        dock.append(&icon);
    }

    dock_container.append(&dock);
    preview.append(&dock_container);

    preview
}

fn create_mac_preview() -> Box {
    let preview = Box::new(Orientation::Vertical, 0);

    // Menu bar (macOS style)
    let menubar = Box::new(Orientation::Horizontal, 12);
    menubar.add_css_class("toolbar");
    menubar.set_size_request(-1, 20);

    let logo = Label::new(Some("W"));
    logo.add_css_class("caption");
    logo.add_css_class("heading");
    logo.set_margin_start(8);
    menubar.append(&logo);

    let menus = ["File", "Edit", "View"];
    for menu in menus {
        let label = Label::new(Some(menu));
        label.add_css_class("caption");
        menubar.append(&label);
    }

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    menubar.append(&spacer);

    let time = Label::new(Some("12:00"));
    time.add_css_class("caption");
    time.set_margin_end(8);
    menubar.append(&time);

    preview.append(&menubar);

    // Desktop
    let desktop = Box::new(Orientation::Vertical, 0);
    desktop.set_vexpand(true);
    desktop.add_css_class("view");
    preview.append(&desktop);

    // Dock at bottom (centered, macOS style)
    let dock_container = Box::new(Orientation::Horizontal, 0);
    dock_container.set_halign(gtk4::Align::Center);
    dock_container.set_margin_bottom(6);

    let dock = Box::new(Orientation::Horizontal, 6);
    dock.add_css_class("card");
    dock.set_margin_top(4);
    dock.set_margin_bottom(4);
    dock.set_margin_start(12);
    dock.set_margin_end(12);

    for icon_name in ["view-grid-symbolic", "folder-symbolic", "terminal-symbolic", "web-browser-symbolic", "preferences-system-symbolic"] {
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(22);
        icon.set_margin_start(4);
        icon.set_margin_end(4);
        icon.set_margin_top(6);
        icon.set_margin_bottom(6);
        dock.append(&icon);
    }

    dock_container.append(&dock);
    preview.append(&dock_container);

    preview
}
