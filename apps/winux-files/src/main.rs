//! Winux Files - A simple file manager for Winux OS
//! Uses GTK4 and libadwaita for a modern GNOME-like interface

use gtk4::prelude::*;
use gtk4::{
    glib, Application, Box as GtkBox, Image, Label, ListBox,
    ListBoxRow, Orientation, PolicyType, ScrolledWindow, SelectionMode, Separator,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "org.winux.Files";

fn main() -> glib::ExitCode {
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

    // Create main content area with sidebar and file list
    let content_box = GtkBox::new(Orientation::Horizontal, 0);
    content_box.append(&sidebar_scroll);
    content_box.append(&Separator::new(Orientation::Vertical));
    content_box.append(&file_scroll);

    // Create main vertical box
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&content_box);

    // Wrap in AdwApplicationWindow for proper libadwaita styling
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Files")
        .default_width(900)
        .default_height(600)
        .content(&main_box)
        .build();

    // Share file_list between closures
    let file_list = Rc::new(RefCell::new(file_list));
    let current_path: Rc<RefCell<PathBuf>> = Rc::new(RefCell::new(
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
    ));

    // Load initial directory (home)
    load_directory(&file_list.borrow(), &current_path.borrow());

    // Connect sidebar selection
    let file_list_clone = Rc::clone(&file_list);
    let current_path_clone = Rc::clone(&current_path);
    sidebar.connect_row_activated(move |_, row| {
        if let Some(path) = get_place_path(row.index()) {
            *current_path_clone.borrow_mut() = path.clone();
            load_directory(&file_list_clone.borrow(), &path);
        }
    });

    // Connect file list activation (double-click or enter)
    let file_list_for_activation = Rc::clone(&file_list);
    let current_path_for_activation = Rc::clone(&current_path);
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
                    *current_path_for_activation.borrow_mut() = path.clone();
                    load_directory(&file_list_for_activation.borrow(), &path);
                } else {
                    // Try to open file with default application
                    let _ = open::that(&path);
                }
            }
        }
    });

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
            let error_row = create_file_row("dialog-error-symbolic", &format!("Error: {}", e));
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
        let empty_row = create_file_row("folder-symbolic", "(Empty folder)");
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

        let icon_name = if is_dir {
            "folder-symbolic"
        } else {
            get_file_icon(&file_name)
        };

        let row = create_file_row(icon_name, &file_name);
        file_list.append(&row);
    }
}

fn create_file_row(icon_name: &str, file_name: &str) -> ListBoxRow {
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

    let row = ListBoxRow::new();
    row.set_child(Some(&hbox));
    row
}

fn get_file_icon(file_name: &str) -> &'static str {
    let extension = file_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        // Documents
        "pdf" => "application-pdf-symbolic",
        "doc" | "docx" | "odt" => "x-office-document-symbolic",
        "xls" | "xlsx" | "ods" => "x-office-spreadsheet-symbolic",
        "ppt" | "pptx" | "odp" => "x-office-presentation-symbolic",
        "txt" | "md" | "rst" => "text-x-generic-symbolic",

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => "image-x-generic-symbolic",

        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "audio-x-generic-symbolic",

        // Video
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" => "video-x-generic-symbolic",

        // Archives
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "package-x-generic-symbolic",

        // Code
        "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go" | "rb" => {
            "text-x-script-symbolic"
        }

        // Executables
        "exe" | "msi" | "sh" | "bin" | "appimage" => "application-x-executable-symbolic",

        // Default
        _ => "text-x-generic-symbolic",
    }
}
