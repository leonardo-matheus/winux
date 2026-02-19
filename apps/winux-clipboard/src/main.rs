//! Winux Clipboard Manager
//!
//! A modern clipboard manager for Wayland with history, search,
//! and intelligent content handling.

mod config;
mod daemon;
mod history;
mod storage;
mod ui;

use anyhow::{Context, Result};
use gtk4::prelude::*;
use gtk4::{glib, Application, gio};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use config::Config;
use daemon::{ClipboardDaemon, ClipboardEvent, DaemonCommand};
use history::ClipboardHistory;
use storage::Storage;
use ui::ClipboardPopup;

const APP_ID: &str = "org.winux.Clipboard";

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!("Starting Winux Clipboard Manager");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--daemon" | "-d" => {
                return run_daemon();
            }
            "--show" | "-s" => {
                return show_popup();
            }
            "--clear" => {
                return clear_history();
            }
            "--export" => {
                let path = args.get(2).map(|s| s.as_str()).unwrap_or("clipboard_history.json");
                return export_history(path);
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-v" => {
                println!("winux-clipboard 1.0.0");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                print_help();
                return Ok(());
            }
        }
    }

    // Default: run GUI
    run_gui()
}

fn print_help() {
    println!(
        r#"Winux Clipboard Manager

USAGE:
    winux-clipboard [OPTIONS]

OPTIONS:
    -d, --daemon     Run as background daemon
    -s, --show       Show clipboard popup
    --clear          Clear clipboard history
    --export [PATH]  Export history to JSON file
    -h, --help       Show this help message
    -v, --version    Show version information

Without options, starts the GUI application.
"#
    );
}

/// Run the clipboard daemon in background
fn run_daemon() -> Result<()> {
    info!("Running clipboard daemon");

    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let config = Config::load().unwrap_or_default();

        let mut storage = Storage::new(config.clone());
        storage.init()?;

        let history = storage.load_history().unwrap_or_else(|e| {
            warn!("Failed to load history: {}, starting fresh", e);
            ClipboardHistory::new(config.max_history)
        });

        let history = Arc::new(RwLock::new(history));
        let storage = Arc::new(RwLock::new(storage));

        let (event_tx, mut event_rx) = mpsc::channel::<ClipboardEvent>(100);
        let (command_tx, command_rx) = mpsc::channel::<DaemonCommand>(10);

        // Start daemon
        let mut daemon = ClipboardDaemon::new(
            config,
            history.clone(),
            storage.clone(),
            event_tx,
            command_rx,
        );

        // Handle events
        let history_clone = history.clone();
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    ClipboardEvent::NewItem(item) => {
                        info!("New clipboard item: {} ({})", item.preview, item.content_type.display_name());
                    }
                    ClipboardEvent::Cleared => {
                        info!("Clipboard cleared");
                    }
                    ClipboardEvent::Error(e) => {
                        error!("Clipboard error: {}", e);
                    }
                }
            }
        });

        // Handle signals for clean shutdown
        #[cfg(unix)]
        {
            use signal_hook::consts::signal::*;
            use signal_hook_tokio::Signals;
            use futures::StreamExt;

            let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
            let command_tx_clone = command_tx.clone();

            tokio::spawn(async move {
                while let Some(sig) = signals.next().await {
                    info!("Received signal {:?}, shutting down", sig);
                    let _ = command_tx_clone.send(DaemonCommand::Stop).await;
                    break;
                }
            });
        }

        daemon.run().await
    })
}

/// Show the clipboard popup
fn show_popup() -> Result<()> {
    // Send D-Bus message to show popup if daemon is running
    // For now, just launch the GUI
    run_gui()
}

/// Clear clipboard history
fn clear_history() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let storage = Storage::new(config);
    storage.clear_all()?;
    info!("Clipboard history cleared");
    Ok(())
}

/// Export history to JSON file
fn export_history(path: &str) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let mut storage = Storage::new(config.clone());
    storage.init()?;

    let history = storage.load_history()?;

    let output_path = std::path::Path::new(path);
    storage.export_history(&history, output_path)?;

    info!("History exported to {}", path);
    Ok(())
}

/// Run the GUI application
fn run_gui() -> Result<()> {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_ui);

    // Run the application
    let exit_code = app.run();

    if exit_code.value() != 0 {
        anyhow::bail!("Application exited with code {}", exit_code.value());
    }

    Ok(())
}

fn build_ui(app: &Application) {
    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}, using defaults", e);
        Config::default()
    });

    // Initialize storage
    let mut storage = Storage::new(config.clone());
    if let Err(e) = storage.init() {
        error!("Failed to initialize storage: {}", e);
    }

    // Load history
    let history = storage.load_history().unwrap_or_else(|e| {
        warn!("Failed to load history: {}, starting fresh", e);
        ClipboardHistory::new(config.max_history)
    });

    let history = Arc::new(RwLock::new(history));
    let storage = Arc::new(RwLock::new(storage));

    // Create popup window
    let window = ClipboardPopup::new(app, config.clone());
    window.set_history(history.clone());

    // Set up callbacks
    let history_paste = history.clone();
    let storage_paste = storage.clone();
    window.connect_paste(move |item| {
        // Copy to clipboard
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            let content = item.content.clone();
            let content_type = item.content_type.clone();
            let history = history_paste.clone();
            let id = item.id;

            handle.spawn(async move {
                // Mark as used
                let mut history = history.write().await;
                history.mark_used(id);

                // Set clipboard content via wl-copy
                let result = match content_type {
                    history::ContentType::Text => {
                        std::process::Command::new("wl-copy")
                            .arg(&content)
                            .status()
                    }
                    history::ContentType::Html => {
                        std::process::Command::new("wl-copy")
                            .args(["--type", "text/html"])
                            .arg(&content)
                            .status()
                    }
                    history::ContentType::Image => {
                        // Content is image path
                        if let Ok(data) = std::fs::read(&content) {
                            std::process::Command::new("wl-copy")
                                .args(["--type", "image/png"])
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                                .and_then(|mut child| {
                                    use std::io::Write;
                                    if let Some(stdin) = child.stdin.as_mut() {
                                        stdin.write_all(&data)?;
                                    }
                                    child.wait()
                                })
                        } else {
                            Ok(std::process::ExitStatus::default())
                        }
                    }
                    _ => Ok(std::process::ExitStatus::default()),
                };

                if let Err(e) = result {
                    error!("Failed to set clipboard: {}", e);
                }
            });
        }
    });

    let history_delete = history.clone();
    let storage_delete = storage.clone();
    let window_delete = window.clone();
    window.connect_delete(move |id| {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            let history = history_delete.clone();
            let storage = storage_delete.clone();

            handle.spawn(async move {
                let mut history = history.write().await;
                if let Some(item) = history.remove(id) {
                    // Delete associated image if any
                    if let Some(path) = item.image_path {
                        let storage = storage.read().await;
                        let _ = storage.delete_image(&path);
                    }

                    // Save history
                    let storage = storage.read().await;
                    if let Err(e) = storage.save_history(&history) {
                        error!("Failed to save history: {}", e);
                    }
                }
            });
        }

        // Refresh the list
        glib::idle_add_local_once({
            let window = window_delete.clone();
            move || {
                window.refresh_list();
            }
        });
    });

    let history_pin = history.clone();
    let storage_pin = storage.clone();
    let window_pin = window.clone();
    window.connect_pin(move |id| {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            let history = history_pin.clone();
            let storage = storage_pin.clone();

            handle.spawn(async move {
                let mut history = history.write().await;
                history.toggle_pin(id);

                let storage = storage.read().await;
                if let Err(e) = storage.save_history(&history) {
                    error!("Failed to save history: {}", e);
                }
            });
        }

        // Refresh the list
        glib::idle_add_local_once({
            let window = window_pin.clone();
            move || {
                window.refresh_list();
            }
        });
    });

    let history_clear = history.clone();
    let storage_clear = storage.clone();
    let window_clear = window.clone();
    window.connect_clear(move || {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            let history = history_clear.clone();
            let storage = storage_clear.clone();

            handle.spawn(async move {
                let mut history = history.write().await;
                history.clear_unpinned();

                let storage = storage.read().await;
                if let Err(e) = storage.save_history(&history) {
                    error!("Failed to save history: {}", e);
                }
            });
        }

        // Refresh the list
        glib::idle_add_local_once({
            let window = window_clear.clone();
            move || {
                window.refresh_list();
            }
        });
    });

    // Initial list refresh
    window.refresh_list();

    // Show window
    window.present();
}
