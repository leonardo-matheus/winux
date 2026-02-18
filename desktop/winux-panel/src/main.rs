//! Winux Panel - Simple GTK4 panel application
//!
//! This is a simplified panel that works without layer-shell.
//! On GNOME, the built-in GNOME Shell panel will be used instead.
//! This application serves as a fallback or development tool.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

const APP_ID: &str = "org.winux.Panel";

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
        .default_width(400)
        .default_height(200)
        .build();

    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.set_valign(gtk::Align::Center);
    content.set_halign(gtk::Align::Center);

    let title_label = Label::new(Some("Winux Panel"));
    title_label.add_css_class("title-1");
    content.append(&title_label);

    let info_label = Label::new(Some(
        "This panel uses the GNOME Shell top bar.\n\n\
        The gtk4-layer-shell dependency is not available\n\
        on Ubuntu 24.04, so this application defers\n\
        panel functionality to the desktop environment.\n\n\
        GNOME Shell provides:\n\
        • Clock and calendar\n\
        • System tray and indicators\n\
        • Notifications\n\
        • Quick settings"
    ));
    info_label.set_justify(gtk::Justification::Center);
    info_label.set_wrap(true);
    info_label.add_css_class("body");
    content.append(&info_label);

    let clock_label = Label::new(Some(""));
    clock_label.add_css_class("title-2");
    content.append(&clock_label);

    // Update clock every second
    let clock_label_clone = clock_label.clone();
    gtk::glib::timeout_add_seconds_local(1, move || {
        if let Ok(now) = gtk::glib::DateTime::now_local() {
            if let Ok(time_str) = now.format("%H:%M:%S") {
                clock_label_clone.set_text(&format!("Current time: {}", time_str));
            }
        }
        gtk::glib::ControlFlow::Continue
    });

    // Initial time update
    if let Ok(now) = gtk::glib::DateTime::now_local() {
        if let Ok(time_str) = now.format("%H:%M:%S") {
            clock_label.set_text(&format!("Current time: {}", time_str));
        }
    }

    window.set_child(Some(&content));
    window.present();

    println!("[winux-panel] Panel application started");
    println!("[winux-panel] Using GNOME Shell for panel functionality");
}
