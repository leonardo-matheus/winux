// Winux Terminal - Simplified terminal launcher
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Label, Orientation, HeaderBar, Align};
use libadwaita as adw;
use std::process::Command;

const APP_ID: &str = "org.winux.terminal";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(48)
        .margin_end(48)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let icon = Label::builder()
        .label("\u{1F4BB}")
        .build();
    icon.add_css_class("title-1");

    let title = Label::builder()
        .label("Winux Terminal")
        .build();
    title.add_css_class("title-1");

    let subtitle = Label::builder()
        .label("Full terminal emulator coming soon!\nFor now, click below to open GNOME Terminal.")
        .justify(gtk4::Justification::Center)
        .build();
    subtitle.add_css_class("dim-label");

    let open_terminal_btn = Button::builder()
        .label("Open GNOME Terminal")
        .halign(Align::Center)
        .build();
    open_terminal_btn.add_css_class("suggested-action");
    open_terminal_btn.add_css_class("pill");

    open_terminal_btn.connect_clicked(|_| {
        launch_terminal();
    });

    let status_label = Label::builder()
        .label("VTE library not available - using system terminal")
        .build();
    status_label.add_css_class("dim-label");
    status_label.add_css_class("caption");

    main_box.append(&icon);
    main_box.append(&title);
    main_box.append(&subtitle);
    main_box.append(&open_terminal_btn);
    main_box.append(&status_label);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Terminal")
        .default_width(500)
        .default_height(350)
        .build();

    window.set_titlebar(Some(&header));
    window.set_child(Some(&main_box));

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    window.present();
}

fn launch_terminal() {
    // Try various terminal emulators in order of preference
    let terminals = [
        ("gnome-terminal", vec!["--"]),
        ("konsole", vec!["-e"]),
        ("xfce4-terminal", vec!["-e"]),
        ("mate-terminal", vec!["-e"]),
        ("xterm", vec!["-e"]),
        ("terminator", vec!["-e"]),
        ("tilix", vec!["-e"]),
    ];

    for (terminal, _args) in terminals.iter() {
        match Command::new(terminal).spawn() {
            Ok(_) => {
                tracing::info!("Launched {} successfully", terminal);
                return;
            }
            Err(e) => {
                tracing::debug!("Failed to launch {}: {}", terminal, e);
                continue;
            }
        }
    }

    tracing::error!("No terminal emulator found on system");
}
