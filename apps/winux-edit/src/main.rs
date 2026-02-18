// Winux Edit - Text editor with syntax highlighting
// Copyright (c) 2026 Winux OS Project

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Label, Orientation, HeaderBar, ScrolledWindow, FileChooserAction, FileChooserDialog, ResponseType};
use libadwaita as adw;
use sourceview5::{View, Buffer, LanguageManager, StyleSchemeManager};
use sourceview5::prelude::*;
use std::fs;
use std::cell::RefCell;
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
    let header = HeaderBar::new();

    let open_btn = Button::builder().icon_name("document-open-symbolic").tooltip_text("Open").build();
    let save_btn = Button::builder().icon_name("document-save-symbolic").tooltip_text("Save").build();
    let new_btn = Button::builder().icon_name("document-new-symbolic").tooltip_text("New").build();

    header.pack_start(&new_btn);
    header.pack_start(&open_btn);
    header.pack_start(&save_btn);

    let buffer = Buffer::new(None);

    // Set up syntax highlighting
    if let Some(scheme_manager) = StyleSchemeManager::default() {
        if let Some(scheme) = scheme_manager.scheme("Adwaita-dark") {
            buffer.set_style_scheme(Some(&scheme));
        }
    }

    let view = View::with_buffer(&buffer);
    view.set_show_line_numbers(true);
    view.set_highlight_current_line(true);
    view.set_auto_indent(true);
    view.set_indent_on_tab(true);
    view.set_tab_width(4);
    view.set_insert_spaces_instead_of_tabs(true);
    view.set_monospace(true);

    let scrolled = ScrolledWindow::builder()
        .child(&view)
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

    // New file
    let buffer_clone = buffer.clone();
    let current_file_clone = current_file.clone();
    let window_clone = window.clone();
    new_btn.connect_clicked(move |_| {
        buffer_clone.set_text("");
        *current_file_clone.borrow_mut() = None;
        window_clone.set_title(Some("Winux Edit - New File"));
    });

    // Open file
    let buffer_clone = buffer.clone();
    let current_file_clone = current_file.clone();
    let window_clone = window.clone();
    open_btn.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(
            Some("Open File"),
            Some(&window_clone),
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
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
                            win.set_title(Some(&format!("Winux Edit - {}", path.file_name().unwrap_or_default().to_string_lossy())));

                            // Detect language
                            if let Some(lang_manager) = LanguageManager::default() {
                                if let Some(lang) = lang_manager.guess_language(Some(&path.to_string_lossy()), None) {
                                    buffer.set_language(Some(&lang));
                                }
                            }
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.show();
    });

    // Save file
    let buffer_clone = buffer.clone();
    let current_file_clone = current_file.clone();
    let window_clone = window.clone();
    save_btn.connect_clicked(move |_| {
        let file_path = current_file_clone.borrow().clone();
        if let Some(path) = file_path {
            let start = buffer_clone.start_iter();
            let end = buffer_clone.end_iter();
            if let Some(text) = buffer_clone.text(&start, &end, false) {
                let _ = fs::write(&path, text.as_str());
            }
        } else {
            let dialog = FileChooserDialog::new(
                Some("Save File"),
                Some(&window_clone),
                FileChooserAction::Save,
                &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
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
                            if let Some(text) = buffer.text(&start, &end, false) {
                                let _ = fs::write(&path, text.as_str());
                                *current_file.borrow_mut() = Some(path.to_string_lossy().to_string());
                                win.set_title(Some(&format!("Winux Edit - {}", path.file_name().unwrap_or_default().to_string_lossy())));
                            }
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        }
    });

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(true);
    }

    window.present();
}
