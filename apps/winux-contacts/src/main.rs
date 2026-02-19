// Winux Contacts - Contact Management Application for Winux OS
// Copyright (c) 2026 Winux OS Project
//
// Features:
// - Contact management (name, phones, emails, addresses)
// - Groups and labels organization
// - vCard import/export (.vcf)
// - CSV import/export
// - CardDAV synchronization (Google Contacts, Nextcloud)
// - Avatar support with fallback initials
// - Search and filtering
// - Integration with GNOME Calls and email clients

mod data;
mod sync;
mod ui;
mod views;
mod window;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Contacts";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        window::build_ui(app);
    });

    app.run()
}
