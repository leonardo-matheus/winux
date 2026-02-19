//! Preview window with editor for captured screenshots

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Box as GtkBox, Orientation, Label, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{HeaderBar, ApplicationWindow as AdwApplicationWindow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::capture::{CaptureResult, get_screenshots_dir, generate_filename};
use crate::editor::EditorCanvas;
use crate::output::{clipboard, save};
use super::toolbar::{EditorToolbar, ActionBar};

/// Preview window showing the captured screenshot with editing capabilities
pub struct PreviewWindow;

impl PreviewWindow {
    pub fn show(app: &Application, state: &Rc<RefCell<AppState>>, capture: CaptureResult) {
        let window = AdwApplicationWindow::builder()
            .application(app)
            .title("Edit Screenshot")
            .default_width(900)
            .default_height(700)
            .build();

        // Editor state
        let editor_state = Rc::new(RefCell::new(state.borrow().editor.clone()));

        // Create canvas
        let canvas = Rc::new(EditorCanvas::new(editor_state.clone()));

        // Load the captured image
        if let Err(e) = canvas.load_image(&capture.path) {
            eprintln!("Failed to load image: {}", e);
        }

        // Create toolbar
        let canvas_clone = canvas.clone();
        let editor_state_clone = editor_state.clone();

        let toolbar = EditorToolbar::new(
            editor_state.clone(),
            move |_tool| {
                canvas_clone.refresh();
            },
            {
                let editor_state = editor_state_clone.clone();
                let canvas = canvas.clone();
                move || {
                    editor_state.borrow_mut().undo();
                    canvas.refresh();
                }
            },
            {
                let editor_state = editor_state.clone();
                let canvas = canvas.clone();
                move || {
                    editor_state.borrow_mut().redo();
                    canvas.refresh();
                }
            },
            {
                let editor_state = editor_state.clone();
                let canvas = canvas.clone();
                move || {
                    editor_state.borrow_mut().clear();
                    canvas.refresh();
                }
            },
        );

        // Create action bar
        let window_clone = window.clone();
        let canvas_clone = canvas.clone();
        let capture_path = capture.path.clone();

        let action_bar = ActionBar::new(
            // Save
            {
                let window = window.clone();
                let canvas = canvas.clone();
                move || {
                    let screenshots_dir = get_screenshots_dir();
                    let filename = generate_filename();
                    let path = screenshots_dir.join(&filename);

                    match canvas.export(&path) {
                        Ok(()) => {
                            show_notification(&window, &format!("Saved to {}", path.display()));
                        }
                        Err(e) => {
                            show_error(&window, &format!("Failed to save: {}", e));
                        }
                    }
                }
            },
            // Copy to clipboard
            {
                let window = window.clone();
                let canvas = canvas.clone();
                move || {
                    // Export to temp file first, then copy to clipboard
                    let temp_path = std::env::temp_dir().join("winux-screenshot-temp.png");
                    match canvas.export(&temp_path) {
                        Ok(()) => {
                            match clipboard::copy_to_clipboard(&temp_path) {
                                Ok(()) => {
                                    show_notification(&window, "Copied to clipboard");
                                }
                                Err(e) => {
                                    show_error(&window, &format!("Failed to copy: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            show_error(&window, &format!("Failed to export: {}", e));
                        }
                    }
                }
            },
            // Share
            {
                let window = window.clone();
                move || {
                    show_notification(&window, "Share feature coming soon!");
                }
            },
            // Discard
            {
                let window = window.clone();
                let capture_path = capture_path.clone();
                move || {
                    // Delete temp file if exists
                    std::fs::remove_file(&capture_path).ok();
                    window.close();
                }
            },
        );

        // Header bar
        let header_bar = HeaderBar::new();

        // Info label
        let info_label = Label::new(Some(&format!(
            "{}x{} pixels",
            capture.width, capture.height
        )));
        info_label.add_css_class("dim-label");
        header_bar.pack_end(&info_label);

        // Save As button
        let save_as_btn = gtk::Button::builder()
            .icon_name("document-save-as-symbolic")
            .tooltip_text("Save As...")
            .build();

        {
            let window = window.clone();
            let canvas = canvas.clone();
            save_as_btn.connect_clicked(move |_| {
                show_save_dialog(&window, &canvas);
            });
        }
        header_bar.pack_end(&save_as_btn);

        // Canvas in scrolled window
        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(canvas.widget())
            .build();

        // Layout
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header_bar);
        main_box.append(&toolbar.container);
        main_box.append(&scrolled);
        main_box.append(&action_bar.container);

        window.set_content(Some(&main_box));

        // Keyboard shortcuts
        let controller = gtk::EventControllerKey::new();
        {
            let editor_state = editor_state.clone();
            let canvas = canvas.clone();
            let window = window.clone();

            controller.connect_key_pressed(move |_, key, _, modifier| {
                use gtk::gdk::Key;

                let ctrl = modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK);
                let shift = modifier.contains(gtk::gdk::ModifierType::SHIFT_MASK);

                if ctrl && key == Key::z && !shift {
                    // Undo
                    editor_state.borrow_mut().undo();
                    canvas.refresh();
                    return glib::Propagation::Stop;
                }

                if ctrl && key == Key::z && shift {
                    // Redo
                    editor_state.borrow_mut().redo();
                    canvas.refresh();
                    return glib::Propagation::Stop;
                }

                if ctrl && key == Key::y {
                    // Redo (alternative)
                    editor_state.borrow_mut().redo();
                    canvas.refresh();
                    return glib::Propagation::Stop;
                }

                if ctrl && key == Key::s && !shift {
                    // Save
                    let screenshots_dir = get_screenshots_dir();
                    let filename = generate_filename();
                    let path = screenshots_dir.join(&filename);

                    match canvas.export(&path) {
                        Ok(()) => {
                            show_notification(&window, &format!("Saved to {}", path.display()));
                        }
                        Err(e) => {
                            show_error(&window, &format!("Failed to save: {}", e));
                        }
                    }
                    return glib::Propagation::Stop;
                }

                if ctrl && key == Key::s && shift {
                    // Save As
                    show_save_dialog(&window, &canvas);
                    return glib::Propagation::Stop;
                }

                if ctrl && key == Key::c {
                    // Copy to clipboard
                    let temp_path = std::env::temp_dir().join("winux-screenshot-temp.png");
                    match canvas.export(&temp_path) {
                        Ok(()) => {
                            match clipboard::copy_to_clipboard(&temp_path) {
                                Ok(()) => {
                                    show_notification(&window, "Copied to clipboard");
                                }
                                Err(e) => {
                                    show_error(&window, &format!("Failed to copy: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            show_error(&window, &format!("Failed to export: {}", e));
                        }
                    }
                    return glib::Propagation::Stop;
                }

                if key == Key::Escape {
                    window.close();
                    return glib::Propagation::Stop;
                }

                glib::Propagation::Proceed
            });
        }
        window.add_controller(controller);

        window.present();
    }
}

fn show_save_dialog(window: &AdwApplicationWindow, canvas: &Rc<EditorCanvas>) {
    let dialog = gtk::FileDialog::builder()
        .title("Save Screenshot")
        .modal(true)
        .build();

    // Set default filename
    let filename = generate_filename();
    dialog.set_initial_name(Some(&filename));

    // Set default folder
    let screenshots_dir = get_screenshots_dir();
    let folder = gtk::gio::File::for_path(&screenshots_dir);
    dialog.set_initial_folder(Some(&folder));

    // File filter for PNG
    let filter = gtk::FileFilter::new();
    filter.add_mime_type("image/png");
    filter.set_name(Some("PNG Images"));

    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));

    let window_clone = window.clone();
    let canvas = canvas.clone();

    dialog.save(Some(window), gtk::gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                match canvas.export(&path) {
                    Ok(()) => {
                        show_notification(&window_clone, &format!("Saved to {}", path.display()));
                    }
                    Err(e) => {
                        show_error(&window_clone, &format!("Failed to save: {}", e));
                    }
                }
            }
        }
    });
}

fn show_notification(window: &AdwApplicationWindow, message: &str) {
    let toast = adw::Toast::new(message);
    toast.set_timeout(3);

    // Find or create toast overlay
    if let Some(content) = window.content() {
        if let Some(toast_overlay) = content.downcast_ref::<adw::ToastOverlay>() {
            toast_overlay.add_toast(toast);
        } else {
            // Wrap content in toast overlay
            let overlay = adw::ToastOverlay::new();
            overlay.set_child(Some(&content));
            window.set_content(Some(&overlay));
            overlay.add_toast(toast);
        }
    }
}

fn show_error(window: &AdwApplicationWindow, message: &str) {
    let dialog = gtk::AlertDialog::builder()
        .message("Error")
        .detail(message)
        .modal(true)
        .build();

    dialog.show(Some(window));
}
