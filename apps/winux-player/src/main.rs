//! Winux Player - Media player launcher for Winux OS
//!
//! Opens media files with the system default player (VLC, Totem, etc.)

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Box, Button, Label, Orientation, Frame, DrawingArea};
use libadwaita as adw;

const APP_ID: &str = "org.winux.player";

fn main() -> gtk::glib::ExitCode {
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // Apply dark theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

    // Header bar
    let header = adw::HeaderBar::new();

    let title_label = Label::builder()
        .label("Winux Player")
        .build();
    header.set_title_widget(Some(&title_label));

    // Main content
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .build();

    // Video area placeholder
    let video_area = DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .content_height(300)
        .content_width(500)
        .build();

    video_area.set_draw_func(|_, cr, width, height| {
        // Dark background
        cr.set_source_rgb(0.1, 0.1, 0.12);
        let _ = cr.paint();

        // Play button circle
        cr.set_source_rgb(0.3, 0.3, 0.35);
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let size = 60.0;

        cr.arc(center_x, center_y, size, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.fill();

        // Play triangle
        cr.set_source_rgb(0.5, 0.5, 0.55);
        cr.move_to(center_x - size * 0.3, center_y - size * 0.4);
        cr.line_to(center_x - size * 0.3, center_y + size * 0.4);
        cr.line_to(center_x + size * 0.5, center_y);
        cr.close_path();
        let _ = cr.fill();
    });

    let video_frame = Frame::builder()
        .child(&video_area)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .build();

    // Info label
    let info_label = Label::builder()
        .label("Winux Player\n\nA full media player is coming soon!\nFor now, use VLC or Totem from the app menu.")
        .justify(gtk::Justification::Center)
        .margin_start(24)
        .margin_end(24)
        .margin_bottom(24)
        .build();
    info_label.add_css_class("dim-label");

    // Open with VLC button
    let vlc_button = Button::builder()
        .label("Open VLC Media Player")
        .halign(gtk::Align::Center)
        .margin_bottom(12)
        .build();
    vlc_button.add_css_class("suggested-action");
    vlc_button.add_css_class("pill");

    vlc_button.connect_clicked(|_| {
        let _ = std::process::Command::new("vlc").spawn();
    });

    // Open with Totem button
    let totem_button = Button::builder()
        .label("Open GNOME Videos")
        .halign(gtk::Align::Center)
        .margin_bottom(24)
        .build();
    totem_button.add_css_class("pill");

    totem_button.connect_clicked(|_| {
        let _ = std::process::Command::new("totem").spawn();
    });

    main_box.append(&video_frame);
    main_box.append(&info_label);
    main_box.append(&vlc_button);
    main_box.append(&totem_button);

    // Content box with header
    let content_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    content_box.append(&header);
    content_box.append(&main_box);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Winux Player")
        .default_width(600)
        .default_height(500)
        .content(&content_box)
        .build();

    window.present();
}
