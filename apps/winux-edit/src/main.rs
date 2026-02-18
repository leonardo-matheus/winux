// Winux Edit - Simple text editor using GTK4 TextView
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, FileChooserAction, FileChooserDialog,
    HeaderBar, Orientation, ResponseType, ScrolledWindow, TextView, WrapMode,
};
use libadwaita as adw;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

const APP_ID: &str = "org.winux.edit";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // Enable dark theme
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    let header = HeaderBar::new();

    let new_btn = Button::builder()
        .icon_name("document-new-symbolic")
        .tooltip_text("New")
        .build();
    let open_btn = Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open")
        .build();
    let save_btn = Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Save")
        .build();

    header.pack_start(&new_btn);
    header.pack_start(&open_btn);
    header.pack_start(&save_btn);

    // Create TextView with basic styling
    let text_view = TextView::builder()
        .monospace(true)
        .wrap_mode(WrapMode::None)
        .left_margin(8)
        .right_margin(8)
        .top_margin(8)
        .bottom_margin(8)
        .build();

    // Apply dark theme CSS for the text view
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        r#"
        textview {
            background-color: #1e1e1e;
            color: #d4d4d4;
        }
        textview text {
            background-color: #1e1e1e;
            color: #d4d4d4;
        }
        "#,
    );

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not get default display"),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let scrolled = ScrolledWindow::builder()
        .child(&text_view)
        .hexpand(true)
        .vexpand(true)
        .build();

    let current_file: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Edit")
        .default_width(900)
        .default_height(600)
        .build();

    window.set_titlebar(Some(&header));
    window.set_child(Some(&scrolled));

    // New file handler
    let buffer = text_view.buffer();
    let buffer_clone = buffer.clone();
    let current_file_clone = current_file.clone();
    let window_clone = window.clone();
    new_btn.connect_clicked(move |_| {
        buffer_clone.set_text("");
        *current_file_clone.borrow_mut() = None;
        window_clone.set_title(Some("Winux Edit - New File"));
    });

    // Open file handler
    let buffer_clone = buffer.clone();
    let current_file_clone = current_file.clone();
    let window_clone = window.clone();
    open_btn.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(
            Some("Open File"),
            Some(&window_clone),
            FileChooserAction::Open,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Open", ResponseType::Accept),
            ],
        );

        let buffer = buffer_clone.clone();
        let current_file = current_file_clone.clone();
        let win = window_clone.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        if let Ok(content) = fs::read_to_string(&path) {
                            buffer.set_text(&content);
                            *current_file.borrow_mut() = Some(path.to_string_lossy().to_string());
                            win.set_title(Some(&format!(
                                "Winux Edit - {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            )));
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.show();
    });

    // Save file handler
    let buffer_clone = buffer.clone();
    let current_file_clone = current_file.clone();
    let window_clone = window.clone();
    save_btn.connect_clicked(move |_| {
        let file_path = current_file_clone.borrow().clone();
        if let Some(path) = file_path {
            // Save to existing file
            let start = buffer_clone.start_iter();
            let end = buffer_clone.end_iter();
            let text = buffer_clone.text(&start, &end, false);
            let _ = fs::write(&path, text.as_str());
        } else {
            // Show save dialog for new file
            let dialog = FileChooserDialog::new(
                Some("Save File"),
                Some(&window_clone),
                FileChooserAction::Save,
                &[
                    ("Cancel", ResponseType::Cancel),
                    ("Save", ResponseType::Accept),
                ],
            );

            let buffer = buffer_clone.clone();
            let current_file = current_file_clone.clone();
            let win = window_clone.clone();

            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let start = buffer.start_iter();
                            let end = buffer.end_iter();
                            let text = buffer.text(&start, &end, false);
                            let _ = fs::write(&path, text.as_str());
                            *current_file.borrow_mut() = Some(path.to_string_lossy().to_string());
                            win.set_title(Some(&format!(
                                "Winux Edit - {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            )));
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        }
    });

    window.present();
}
