// Winux Users - User and Group Management
// Copyright (c) 2026 Winux OS Project
//
// Complete user management with:
// - User listing and creation
// - User editing (avatar, name, password, groups)
// - Group management
// - Login options configuration
//
// Integrates with AccountsService via D-Bus
// Requires polkit for privileged operations

mod window;
mod pages;
mod backend;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Users";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::UsersWindow::new(app);
        window.present();
    });

    app.run()
}
