//! Winux Files - A modern file manager for Winux OS
//!
//! Features:
//! - GTK4 and libadwaita for a modern GNOME-like interface
//! - Native support for Windows, macOS, and Linux file formats
//! - Cross-platform archive handling
//! - Integrated file information and actions

mod file_handlers;

use gtk4::prelude::*;
use gtk4::{
    glib, Application, Box as GtkBox, Button, Image, Label, ListBox,
    ListBoxRow, Orientation, PolicyType, ScrolledWindow, SelectionMode,
    Separator, PopoverMenu, GestureClick,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use file_handlers::{FileHandler, FileType, FileAction};

const APP_ID: &str = "org.winux.Files";

fn main() -> glib::ExitCode {
    // Initialize logging
    env_logger::init();

    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // Create header bar
    let header = adw::HeaderBar::new();
    let title = adw::WindowTitle::new("Files", "");
    header.set_title_widget(Some(&title));

    // Add navigation buttons
    let back_button = Button::from_icon_name("go-previous-symbolic");
    back_button.set_tooltip_text(Some("Go Back"));
    header.pack_start(&back_button);

    let forward_button = Button::from_icon_name("go-next-symbolic");
    forward_button.set_tooltip_text(Some("Go Forward"));
    header.pack_start(&forward_button);

    // Add view options
    let view_button = Button::from_icon_name("view-grid-symbolic");
    view_button.set_tooltip_text(Some("Change View"));
    header.pack_end(&view_button);

    // Create sidebar
    let sidebar = create_sidebar();
    let sidebar_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&sidebar)
        .build();
    sidebar_scroll.set_width_request(200);

    // Create file list
    let file_list = ListBox::new();
    file_list.set_selection_mode(SelectionMode::Single);
    file_list.add_css_class("boxed-list");

    let file_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .child(&file_list)
        .build();

    // Create info panel (hidden by default)
    let info_panel = create_info_panel();
    info_panel.set_visible(false);

    // Create main content area with sidebar, file list, and info panel
    let content_box = GtkBox::new(Orientation::Horizontal, 0);
    content_box.append(&sidebar_scroll);
    content_box.append(&Separator::new(Orientation::Vertical));
    content_box.append(&file_scroll);
    content_box.append(&Separator::new(Orientation::Vertical));
    content_box.append(&info_panel);

    // Create main vertical box
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&content_box);

    // Wrap in AdwApplicationWindow for proper libadwaita styling
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Files")
        .default_width(1000)
        .default_height(650)
        .content(&main_box)
        .build();

    // Share state between closures
    let file_list = Rc::new(RefCell::new(file_list));
    let info_panel = Rc::new(RefCell::new(info_panel));
    let current_path: Rc<RefCell<PathBuf>> = Rc::new(RefCell::new(
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
    ));
    let history: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));
    let history_index: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

    // Load initial directory (home)
    load_directory(&file_list.borrow(), &current_path.borrow());

    // Connect back button
    let current_path_back = Rc::clone(&current_path);
    let file_list_back = Rc::clone(&file_list);
    let history_back = Rc::clone(&history);
    let history_index_back = Rc::clone(&history_index);
    back_button.connect_clicked(move |_| {
        let idx = *history_index_back.borrow();
        if idx > 0 {
            *history_index_back.borrow_mut() = idx - 1;
            if let Some(path) = history_back.borrow().get(idx - 1) {
                *current_path_back.borrow_mut() = path.clone();
                load_directory(&file_list_back.borrow(), path);
            }
        } else {
            // Go to parent
            let current = current_path_back.borrow().clone();
            if let Some(parent) = current.parent() {
                let parent = parent.to_path_buf();
                *current_path_back.borrow_mut() = parent.clone();
                load_directory(&file_list_back.borrow(), &parent);
            }
        }
    });

    // Connect sidebar selection
    let file_list_clone = Rc::clone(&file_list);
    let current_path_clone = Rc::clone(&current_path);
    let history_clone = Rc::clone(&history);
    let history_index_clone = Rc::clone(&history_index);
    sidebar.connect_row_activated(move |_, row| {
        if let Some(path) = get_place_path(row.index()) {
            // Add to history
            history_clone.borrow_mut().push(path.clone());
            *history_index_clone.borrow_mut() = history_clone.borrow().len();

            *current_path_clone.borrow_mut() = path.clone();
            load_directory(&file_list_clone.borrow(), &path);
        }
    });

    // Connect file list activation (double-click or enter)
    let file_list_for_activation = Rc::clone(&file_list);
    let current_path_for_activation = Rc::clone(&current_path);
    let info_panel_for_activation = Rc::clone(&info_panel);
    let history_for_activation = Rc::clone(&history);
    let history_index_for_activation = Rc::clone(&history_index);

    file_list.borrow().connect_row_activated(move |_, row| {
        let index = row.index();
        let current = current_path_for_activation.borrow().clone();

        // Get entries again to find what was clicked
        if let Ok(entries) = fs::read_dir(&current) {
            let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            items.sort_by(|a, b| {
                let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.file_name().cmp(&b.file_name()),
                }
            });

            // Filter hidden files to match what's displayed
            let visible_items: Vec<_> = items
                .into_iter()
                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                .collect();

            if let Some(entry) = visible_items.get(index as usize) {
                let path = entry.path();
                if path.is_dir() {
                    // Add to history
                    history_for_activation.borrow_mut().push(path.clone());
                    *history_index_for_activation.borrow_mut() = history_for_activation.borrow().len();

                    *current_path_for_activation.borrow_mut() = path.clone();
                    load_directory(&file_list_for_activation.borrow(), &path);
                } else {
                    // Check if it's a special file type
                    let file_type = FileType::from_path(&path);
                    if file_type != FileType::Unknown {
                        // Show file info in info panel
                        show_file_info(&info_panel_for_activation.borrow(), &path);
                    } else {
                        // Try to open file with default application
                        let _ = open::that(&path);
                    }
                }
            }
        }
    });

    // Right-click context menu
    let gesture = GestureClick::new();
    gesture.set_button(3); // Right click

    let file_list_for_context = Rc::clone(&file_list);
    let current_path_for_context = Rc::clone(&current_path);
    let window_for_context = window.clone();

    gesture.connect_pressed(move |_, _, x, y| {
        // Get the row at this position
        if let Some(row) = file_list_for_context.borrow().row_at_y(y as i32) {
            let index = row.index();
            let current = current_path_for_context.borrow().clone();

            if let Ok(entries) = fs::read_dir(&current) {
                let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                items.sort_by(|a, b| {
                    let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.file_name().cmp(&b.file_name()),
                    }
                });

                let visible_items: Vec<_> = items
                    .into_iter()
                    .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .collect();

                if let Some(entry) = visible_items.get(index as usize) {
                    let path = entry.path();
                    show_context_menu(&window_for_context, &path, x, y);
                }
            }
        }
    });

    file_list.borrow().add_controller(gesture);

    window.present();
}

fn create_sidebar() -> ListBox {
    let sidebar = ListBox::new();
    sidebar.set_selection_mode(SelectionMode::Single);
    sidebar.add_css_class("navigation-sidebar");

    // Add places
    let places = [
        ("user-home-symbolic", "Home"),
        ("folder-documents-symbolic", "Documents"),
        ("folder-download-symbolic", "Downloads"),
        ("folder-music-symbolic", "Music"),
        ("folder-pictures-symbolic", "Pictures"),
        ("folder-videos-symbolic", "Videos"),
        ("drive-harddisk-symbolic", "File System"),
    ];

    for (icon_name, label_text) in places {
        let row = create_sidebar_row(icon_name, label_text);
        sidebar.append(&row);
    }

    // Select "Home" by default
    if let Some(first_row) = sidebar.row_at_index(0) {
        sidebar.select_row(Some(&first_row));
    }

    sidebar
}

fn create_sidebar_row(icon_name: &str, label_text: &str) -> ListBoxRow {
    let hbox = GtkBox::new(Orientation::Horizontal, 12);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);

    let icon = Image::from_icon_name(icon_name);
    let label = Label::new(Some(label_text));
    label.set_xalign(0.0);
    label.set_hexpand(true);

    hbox.append(&icon);
    hbox.append(&label);

    let row = ListBoxRow::new();
    row.set_child(Some(&hbox));
    row
}

fn create_info_panel() -> GtkBox {
    let panel = GtkBox::new(Orientation::Vertical, 8);
    panel.set_width_request(250);
    panel.set_margin_start(12);
    panel.set_margin_end(12);
    panel.set_margin_top(12);
    panel.set_margin_bottom(12);

    // Title
    let title = Label::new(Some("File Information"));
    title.add_css_class("title-3");
    panel.append(&title);

    // Info content will be added dynamically
    let info_content = GtkBox::new(Orientation::Vertical, 4);
    info_content.set_name("info-content");
    panel.append(&info_content);

    // Actions section
    let actions_label = Label::new(Some("Actions"));
    actions_label.add_css_class("title-4");
    actions_label.set_margin_top(16);
    panel.append(&actions_label);

    let actions_box = GtkBox::new(Orientation::Vertical, 4);
    actions_box.set_name("actions-box");
    panel.append(&actions_box);

    panel
}

fn show_file_info(panel: &GtkBox, path: &PathBuf) {
    panel.set_visible(true);

    // Find and clear the info content box
    let mut child = panel.first_child();
    while let Some(widget) = child {
        if widget.widget_name() == "info-content" {
            if let Ok(content_box) = widget.downcast::<GtkBox>() {
                // Clear existing content
                while let Some(c) = content_box.first_child() {
                    content_box.remove(&c);
                }

                // Get file info
                if let Ok(info) = FileHandler::get_info(path) {
                    // Add properties
                    add_info_row(&content_box, "Name", &info.name);
                    add_info_row(&content_box, "Type", &info.file_type);
                    add_info_row(&content_box, "Size", &format_size(info.size));

                    for (key, value) in &info.properties {
                        add_info_row(&content_box, key, value);
                    }
                }
            }
        }
        if widget.widget_name() == "actions-box" {
            if let Ok(actions_box) = widget.downcast::<GtkBox>() {
                // Clear existing actions
                while let Some(c) = actions_box.first_child() {
                    actions_box.remove(&c);
                }

                // Get available actions
                let file_type = FileType::from_path(path);
                let actions = file_type.get_actions();

                for action in actions {
                    let path_clone = path.clone();
                    let button = Button::with_label(&action.to_string());
                    button.connect_clicked(move |_| {
                        let _ = FileHandler::execute_action(&path_clone, action);
                    });
                    actions_box.append(&button);
                }
            }
        }
        child = widget.next_sibling();
    }
}

fn add_info_row(container: &GtkBox, label: &str, value: &str) {
    let row = GtkBox::new(Orientation::Horizontal, 8);

    let label_widget = Label::new(Some(label));
    label_widget.add_css_class("dim-label");
    label_widget.set_xalign(0.0);
    label_widget.set_width_chars(12);

    let value_widget = Label::new(Some(value));
    value_widget.set_xalign(0.0);
    value_widget.set_hexpand(true);
    value_widget.set_wrap(true);
    value_widget.set_selectable(true);

    row.append(&label_widget);
    row.append(&value_widget);

    container.append(&row);
}

fn show_context_menu(window: &adw::ApplicationWindow, path: &PathBuf, _x: f64, _y: f64) {
    let file_type = FileType::from_path(path);
    let actions = file_type.get_actions();

    if actions.is_empty() {
        return;
    }

    // Create a simple dialog for now (proper popover menu requires more setup)
    let dialog = gtk4::MessageDialog::new(
        Some(window),
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Info,
        gtk4::ButtonsType::Close,
        &format!("Available actions for {}:", path.file_name().unwrap_or_default().to_string_lossy()),
    );

    let content = dialog.content_area();
    for action in actions {
        let path_clone = path.clone();
        let button = Button::with_label(&action.to_string());
        let dialog_clone = dialog.clone();
        button.connect_clicked(move |_| {
            let result = FileHandler::execute_action(&path_clone, action);
            if let Ok(msg) = result {
                log::info!("Action result: {}", msg);
            }
            dialog_clone.close();
        });
        content.append(&button);
    }

    dialog.present();
}

fn get_place_path(index: i32) -> Option<PathBuf> {
    match index {
        0 => dirs::home_dir(),
        1 => dirs::document_dir(),
        2 => dirs::download_dir(),
        3 => dirs::audio_dir(),
        4 => dirs::picture_dir(),
        5 => dirs::video_dir(),
        6 => Some(PathBuf::from("/")),
        _ => None,
    }
}

fn load_directory(file_list: &ListBox, path: &PathBuf) {
    // Clear existing items
    while let Some(child) = file_list.first_child() {
        file_list.remove(&child);
    }

    // Read directory contents
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            let error_row = create_file_row("dialog-error-symbolic", &format!("Error: {}", e), None);
            file_list.append(&error_row);
            return;
        }
    };

    // Collect and sort entries (directories first, then alphabetically)
    let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    items.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    if items.is_empty() {
        let empty_row = create_file_row("folder-symbolic", "(Empty folder)", None);
        file_list.append(&empty_row);
        return;
    }

    for entry in items {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let is_hidden = file_name.starts_with('.');

        // Skip hidden files for simplicity
        if is_hidden {
            continue;
        }

        let path = entry.path();
        let (icon_name, type_hint) = if is_dir {
            ("folder-symbolic", None)
        } else {
            let file_type = FileType::from_path(&path);
            (file_type.get_icon(), Some(format!("{:?}", file_type)))
        };

        let row = create_file_row(icon_name, &file_name, type_hint.as_deref());
        file_list.append(&row);
    }
}

fn create_file_row(icon_name: &str, file_name: &str, type_hint: Option<&str>) -> ListBoxRow {
    let hbox = GtkBox::new(Orientation::Horizontal, 12);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);

    let icon = Image::from_icon_name(icon_name);
    icon.set_pixel_size(24);

    let label = Label::new(Some(file_name));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    hbox.append(&icon);
    hbox.append(&label);

    // Add type hint label if available
    if let Some(hint) = type_hint {
        if hint != "Unknown" {
            let type_label = Label::new(Some(hint));
            type_label.add_css_class("dim-label");
            type_label.set_xalign(1.0);
            hbox.append(&type_label);
        }
    }

    let row = ListBoxRow::new();
    row.set_child(Some(&hbox));
    row
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
