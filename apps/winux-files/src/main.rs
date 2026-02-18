//! Winux Files - Modern File Manager for Winux OS
//!
//! A GTK4/Relm4-based file manager with:
//! - Grid and list view modes
//! - Sidebar with favorites and devices
//! - Full file operations (copy, move, delete, rename)
//! - Thumbnail previews
//! - Search functionality

mod app;
mod config;
mod file_ops;
mod file_view;
mod sidebar;

use anyhow::Result;
use gtk4::prelude::*;
use relm4::prelude::*;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use app::AppModel;

/// Application ID for Winux Files
const APP_ID: &str = "org.winux.files";

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Winux Files");

    // Initialize GTK
    let app = RelmApp::new(APP_ID);

    // Load CSS
    relm4::set_global_css(include_str!("../resources/style.css"));

    // Run the application
    app.run::<AppModel>(());

    Ok(())
}
