// Winux Cloud - Native Cloud Sync Application
// Copyright (c) 2026 Winux OS Project
//
// Complete cloud synchronization solution with:
// - Multiple providers (Google Drive, OneDrive, Dropbox, Nextcloud, WebDAV, S3)
// - OAuth2 authentication with browser redirect
// - Bidirectional sync with delta sync
// - Conflict resolution (keep both, local wins, remote wins)
// - Selective sync (choose folders)
// - File system watcher (inotify)
// - Background sync (daemon mode)
// - Nautilus/Files integration (emblems)
// - Bandwidth limiting
// - Version history
// - Link sharing
// - Optional client-side encryption

mod window;
mod pages;
mod providers;
mod sync;
mod database;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "org.winux.Cloud";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = window::CloudWindow::new(app);
        window.present();
    });

    app.run()
}
