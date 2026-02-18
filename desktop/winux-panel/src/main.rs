//! Winux Panel - Top panel with clock, systray, and notifications
//! 
//! This component provides the top panel featuring:
//! - System clock and date
//! - System tray / status indicators
//! - Notification center access
//! - Quick settings access

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "org.winux.Panel";
const PANEL_HEIGHT: i32 = 32;

fn main() -> gtk::glib::ExitCode {
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Panel")
        .default_width(1920)
        .default_height(PANEL_HEIGHT)
        .decorated(false)
        .resizable(false)
        .build();

    let panel = Box::new(Orientation::Horizontal, 0);
    panel.set_hexpand(true);
    panel.add_css_class("panel");

    // Left section - Activities button
    let left_section = build_left_section();
    panel.append(&left_section);

    // Center section - Clock
    let center_section = build_center_section();
    center_section.set_hexpand(true);
    panel.append(&center_section);

    // Right section - System tray and quick settings
    let right_section = build_right_section();
    panel.append(&right_section);

    window.set_child(Some(&panel));
    window.present();

    println!("[winux-panel] Top panel initialized");
    println!("[winux-panel] Panel height: {}px", PANEL_HEIGHT);
}

fn build_left_section() -> Box {
    let section = Box::new(Orientation::Horizontal, 8);
    section.set_margin_start(8);

    let activities_btn = Button::builder()
        .label("Activities")
        .build();
    activities_btn.add_css_class("flat");
    activities_btn.connect_clicked(|_| {
        println!("[winux-panel] Activities overview triggered");
    });

    section.append(&activities_btn);
    section
}

fn build_center_section() -> Box {
    let section = Box::new(Orientation::Horizontal, 0);
    section.set_halign(gtk::Align::Center);

    let clock_label = Label::new(Some("12:00"));
    clock_label.add_css_class("clock");
    
    // Update clock every second
    let clock_label_clone = clock_label.clone();
    gtk::glib::timeout_add_seconds_local(1, move || {
        let now = gtk::glib::DateTime::now_local().unwrap();
        let time_str = now.format("%H:%M").unwrap().to_string();
        clock_label_clone.set_text(&time_str);
        gtk::glib::ControlFlow::Continue
    });

    // Initial time update
    if let Some(now) = gtk::glib::DateTime::now_local() {
        if let Ok(time_str) = now.format("%H:%M") {
            clock_label.set_text(&time_str.to_string());
        }
    }

    let clock_btn = Button::builder()
        .child(&clock_label)
        .build();
    clock_btn.add_css_class("flat");
    clock_btn.connect_clicked(|_| {
        println!("[winux-panel] Calendar/notifications panel triggered");
    });

    section.append(&clock_btn);
    section
}

fn build_right_section() -> Box {
    let section = Box::new(Orientation::Horizontal, 4);
    section.set_margin_end(8);

    // Notification counter (placeholder)
    let notification_count: Rc<RefCell<u32>> = Rc::new(RefCell::new(3));
    
    let notif_btn = Button::builder()
        .icon_name("preferences-system-notifications-symbolic")
        .tooltip_text("Notifications")
        .build();
    notif_btn.add_css_class("flat");
    
    let count = notification_count.clone();
    notif_btn.connect_clicked(move |_| {
        println!("[winux-panel] Notification center opened ({} notifications)", count.borrow());
    });

    // System tray indicators
    let network_btn = Button::builder()
        .icon_name("network-wireless-symbolic")
        .tooltip_text("Network")
        .build();
    network_btn.add_css_class("flat");

    let volume_btn = Button::builder()
        .icon_name("audio-volume-high-symbolic")
        .tooltip_text("Sound")
        .build();
    volume_btn.add_css_class("flat");

    let power_btn = Button::builder()
        .icon_name("system-shutdown-symbolic")
        .tooltip_text("Power")
        .build();
    power_btn.add_css_class("flat");
    power_btn.connect_clicked(|_| {
        println!("[winux-panel] Power menu opened");
    });

    section.append(&notif_btn);
    section.append(&network_btn);
    section.append(&volume_btn);
    section.append(&power_btn);

    section
}
