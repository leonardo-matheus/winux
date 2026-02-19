//! Main window for Winux Archive

use gtk4::prelude::*;
use gtk4::{
    glib, Application, Box as GtkBox, Button, FileChooserAction, FileChooserNative,
    FileFilter, Label, MenuButton, Orientation, Popover, ResponseType,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::archive::{Archive, ArchiveFormat, ArchiveEntry};
use crate::operations::{ExtractOptions, CreateOptions};
use crate::ui::{FileListView, PathBar, ProgressDialog};

/// Application state
pub struct AppState {
    pub archive: Option<Archive>,
    pub current_path: String,
    pub selected_entries: Vec<ArchiveEntry>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            archive: None,
            current_path: String::new(),
            selected_entries: Vec::new(),
        }
    }
}

pub fn build_window(app: &Application, file_path: Option<PathBuf>) {
    let state = Rc::new(RefCell::new(AppState::default()));

    // Create header bar
    let header = adw::HeaderBar::new();
    let title = adw::WindowTitle::new("Archive Manager", "");
    header.set_title_widget(Some(&title));

    // Open button
    let open_btn = Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open Archive")
        .build();

    // Create new archive button
    let new_btn = Button::builder()
        .icon_name("archive-insert-symbolic")
        .tooltip_text("Create New Archive")
        .build();

    // Extract button
    let extract_btn = Button::builder()
        .icon_name("extract-archive-symbolic")
        .tooltip_text("Extract All")
        .sensitive(false)
        .build();

    // Add files button
    let add_btn = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add Files")
        .sensitive(false)
        .build();

    // Menu button
    let menu_btn = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Menu")
        .build();

    // Create menu popover
    let menu_popover = create_menu_popover();
    menu_btn.set_popover(Some(&menu_popover));

    header.pack_start(&open_btn);
    header.pack_start(&new_btn);
    header.pack_end(&menu_btn);
    header.pack_end(&extract_btn);
    header.pack_end(&add_btn);

    // Create path bar
    let path_bar = PathBar::new();

    // Create file list view
    let file_list = FileListView::new();

    // Create main content
    let content_box = GtkBox::new(Orientation::Vertical, 0);
    content_box.append(path_bar.widget());
    content_box.append(file_list.widget());

    // Create status bar
    let status_bar = create_status_bar();

    // Create main vertical box
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&content_box);
    main_box.append(&status_bar);

    // Create window
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Archive Manager")
        .default_width(900)
        .default_height(600)
        .content(&main_box)
        .build();

    // Connect signals
    let state_clone = Rc::clone(&state);
    let window_clone = window.clone();
    let file_list_clone = file_list.clone();
    let path_bar_clone = path_bar.clone();
    let title_clone = title.clone();
    let extract_btn_clone = extract_btn.clone();
    let add_btn_clone = add_btn.clone();
    let status_bar_clone = status_bar.clone();

    open_btn.connect_clicked(move |_| {
        let dialog = FileChooserNative::builder()
            .title("Open Archive")
            .action(FileChooserAction::Open)
            .modal(true)
            .transient_for(&window_clone)
            .build();

        // Add filters
        let filter = FileFilter::new();
        filter.set_name(Some("All Archives"));
        filter.add_pattern("*.zip");
        filter.add_pattern("*.rar");
        filter.add_pattern("*.tar");
        filter.add_pattern("*.tar.gz");
        filter.add_pattern("*.tgz");
        filter.add_pattern("*.tar.bz2");
        filter.add_pattern("*.tbz2");
        filter.add_pattern("*.tar.xz");
        filter.add_pattern("*.txz");
        filter.add_pattern("*.7z");
        filter.add_pattern("*.iso");
        filter.add_pattern("*.zst");
        filter.add_pattern("*.zstd");
        dialog.add_filter(&filter);

        let state = Rc::clone(&state_clone);
        let file_list = file_list_clone.clone();
        let path_bar = path_bar_clone.clone();
        let title = title_clone.clone();
        let extract_btn = extract_btn_clone.clone();
        let add_btn = add_btn_clone.clone();
        let status_bar = status_bar_clone.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        open_archive(
                            &state,
                            &path,
                            &file_list,
                            &path_bar,
                            &title,
                            &extract_btn,
                            &add_btn,
                            &status_bar,
                        );
                    }
                }
            }
        });

        dialog.show();
    });

    // Create new archive
    let window_clone = window.clone();
    new_btn.connect_clicked(move |_| {
        show_create_dialog(&window_clone);
    });

    // Extract all
    let state_clone = Rc::clone(&state);
    let window_clone = window.clone();
    extract_btn.connect_clicked(move |_| {
        let state = state_clone.borrow();
        if state.archive.is_some() {
            show_extract_dialog(&window_clone, &state_clone);
        }
    });

    // Add files
    let state_clone = Rc::clone(&state);
    let window_clone = window.clone();
    add_btn.connect_clicked(move |_| {
        show_add_files_dialog(&window_clone, &state_clone);
    });

    // Enable drag and drop
    setup_drag_drop(&window, Rc::clone(&state), &file_list, &path_bar, &title, &extract_btn, &add_btn, &status_bar);

    // Open file if provided
    if let Some(path) = file_path {
        open_archive(
            &state,
            &path,
            &file_list,
            &path_bar,
            &title,
            &extract_btn,
            &add_btn,
            &status_bar,
        );
    }

    window.present();
}

fn create_menu_popover() -> Popover {
    let menu_box = GtkBox::new(Orientation::Vertical, 4);
    menu_box.set_margin_start(8);
    menu_box.set_margin_end(8);
    menu_box.set_margin_top(8);
    menu_box.set_margin_bottom(8);

    let items = [
        ("Test Integrity", "document-properties-symbolic"),
        ("Properties", "dialog-information-symbolic"),
        ("Preferences", "preferences-system-symbolic"),
        ("About", "help-about-symbolic"),
    ];

    for (label, icon) in items {
        let btn = Button::builder()
            .label(label)
            .build();
        btn.set_has_frame(false);
        menu_box.append(&btn);
    }

    let popover = Popover::new();
    popover.set_child(Some(&menu_box));
    popover
}

fn create_status_bar() -> GtkBox {
    let status_bar = GtkBox::new(Orientation::Horizontal, 12);
    status_bar.set_margin_start(12);
    status_bar.set_margin_end(12);
    status_bar.set_margin_top(6);
    status_bar.set_margin_bottom(6);
    status_bar.add_css_class("toolbar");

    let items_label = Label::new(Some("No archive loaded"));
    items_label.set_hexpand(true);
    items_label.set_xalign(0.0);

    let size_label = Label::new(Some(""));
    let compression_label = Label::new(Some(""));

    status_bar.append(&items_label);
    status_bar.append(&size_label);
    status_bar.append(&compression_label);

    status_bar
}

fn open_archive(
    state: &Rc<RefCell<AppState>>,
    path: &PathBuf,
    file_list: &FileListView,
    path_bar: &PathBar,
    title: &adw::WindowTitle,
    extract_btn: &Button,
    add_btn: &Button,
    status_bar: &GtkBox,
) {
    match Archive::open(path) {
        Ok(archive) => {
            let file_name = path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            title.set_title(&file_name);
            title.set_subtitle(&path.to_string_lossy());

            // Get entries
            let entries = archive.list_entries("").unwrap_or_default();

            // Update file list
            file_list.set_entries(&entries);

            // Update path bar
            path_bar.set_path(&file_name, "");

            // Update status bar
            update_status_bar(status_bar, &archive, &entries);

            // Enable buttons
            extract_btn.set_sensitive(true);
            add_btn.set_sensitive(archive.format().supports_add());

            // Update state
            let mut state = state.borrow_mut();
            state.archive = Some(archive);
            state.current_path = String::new();
        }
        Err(e) => {
            eprintln!("Failed to open archive: {}", e);
            // Show error dialog
        }
    }
}

fn update_status_bar(status_bar: &GtkBox, archive: &Archive, entries: &[ArchiveEntry]) {
    let items_label = status_bar.first_child().and_then(|w| w.downcast::<Label>().ok());
    let size_label = items_label.as_ref().and_then(|w| w.next_sibling()).and_then(|w| w.downcast::<Label>().ok());
    let compression_label = size_label.as_ref().and_then(|w| w.next_sibling()).and_then(|w| w.downcast::<Label>().ok());

    if let Some(label) = items_label {
        let file_count = entries.iter().filter(|e| !e.is_directory).count();
        let dir_count = entries.iter().filter(|e| e.is_directory).count();
        label.set_text(&format!("{} files, {} folders", file_count, dir_count));
    }

    if let Some(label) = size_label {
        let total_size: u64 = entries.iter().map(|e| e.uncompressed_size).sum();
        label.set_text(&format_size(total_size));
    }

    if let Some(label) = compression_label {
        label.set_text(&format!("Format: {}", archive.format().name()));
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn show_create_dialog(window: &adw::ApplicationWindow) {
    let dialog = adw::MessageDialog::new(
        Some(window),
        Some("Create New Archive"),
        Some("Select files and choose archive format"),
    );

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("create", "Create");
    dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);

    // Add format selection and options
    let content = create_archive_options_widget();
    dialog.set_extra_child(Some(&content));

    let window_clone = window.clone();
    dialog.connect_response(None, move |dialog, response| {
        if response == "create" {
            // Show file chooser to select output location
            let save_dialog = FileChooserNative::builder()
                .title("Save Archive As")
                .action(FileChooserAction::Save)
                .modal(true)
                .transient_for(&window_clone)
                .build();

            save_dialog.connect_response(|dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            // Create the archive
                            println!("Creating archive at: {:?}", path);
                        }
                    }
                }
            });

            save_dialog.show();
        }
        dialog.close();
    });

    dialog.present();
}

fn create_archive_options_widget() -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);

    // Format selection
    let format_row = adw::ActionRow::builder()
        .title("Format")
        .subtitle("Archive format to create")
        .build();

    let format_combo = gtk4::DropDown::from_strings(&[
        "ZIP", "TAR.GZ", "TAR.BZ2", "TAR.XZ", "7z", "ZSTD"
    ]);
    format_combo.set_valign(gtk4::Align::Center);
    format_row.add_suffix(&format_combo);

    // Compression level
    let compression_row = adw::ActionRow::builder()
        .title("Compression Level")
        .subtitle("Higher = smaller file, slower")
        .build();

    let compression_scale = gtk4::Scale::with_range(Orientation::Horizontal, 1.0, 9.0, 1.0);
    compression_scale.set_value(6.0);
    compression_scale.set_width_request(150);
    compression_scale.set_valign(gtk4::Align::Center);
    compression_row.add_suffix(&compression_scale);

    // Password protection
    let password_row = adw::ActionRow::builder()
        .title("Password Protection")
        .subtitle("Encrypt the archive")
        .build();

    let password_switch = gtk4::Switch::new();
    password_switch.set_valign(gtk4::Align::Center);
    password_row.add_suffix(&password_switch);

    // Split volumes
    let split_row = adw::ActionRow::builder()
        .title("Split into Volumes")
        .subtitle("Split archive into multiple parts")
        .build();

    let split_switch = gtk4::Switch::new();
    split_switch.set_valign(gtk4::Align::Center);
    split_row.add_suffix(&split_switch);

    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);
    list_box.add_css_class("boxed-list");
    list_box.append(&format_row);
    list_box.append(&compression_row);
    list_box.append(&password_row);
    list_box.append(&split_row);

    vbox.append(&list_box);
    vbox
}

fn show_extract_dialog(window: &adw::ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let dialog = FileChooserNative::builder()
        .title("Extract to...")
        .action(FileChooserAction::SelectFolder)
        .modal(true)
        .transient_for(window)
        .build();

    let state = Rc::clone(state);
    let window = window.clone();

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(dest_path) = file.path() {
                    let state = state.borrow();
                    if let Some(ref archive) = state.archive {
                        // Show progress dialog
                        let progress = ProgressDialog::new(&window, "Extracting...");
                        progress.show();

                        // Extract in background
                        let options = ExtractOptions {
                            destination: dest_path,
                            overwrite: true,
                            preserve_permissions: true,
                            password: None,
                        };

                        match crate::operations::extract::extract_all(archive, &options) {
                            Ok(_) => {
                                progress.set_complete("Extraction complete!");
                            }
                            Err(e) => {
                                progress.set_error(&format!("Extraction failed: {}", e));
                            }
                        }
                    }
                }
            }
        }
    });

    dialog.show();
}

fn show_add_files_dialog(window: &adw::ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let dialog = FileChooserNative::builder()
        .title("Add Files to Archive")
        .action(FileChooserAction::Open)
        .modal(true)
        .transient_for(window)
        .select_multiple(true)
        .build();

    let state = Rc::clone(state);
    let window = window.clone();

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let files: Vec<PathBuf> = dialog.files()
                .iter::<gtk4::gio::File>()
                .filter_map(|f| f.ok())
                .filter_map(|f| f.path())
                .collect();

            if !files.is_empty() {
                let mut state = state.borrow_mut();
                if let Some(ref mut archive) = state.archive {
                    // Show progress dialog
                    let progress = ProgressDialog::new(&window, "Adding files...");
                    progress.show();

                    match crate::operations::add::add_files(archive, &files, "") {
                        Ok(_) => {
                            progress.set_complete("Files added successfully!");
                        }
                        Err(e) => {
                            progress.set_error(&format!("Failed to add files: {}", e));
                        }
                    }
                }
            }
        }
    });

    dialog.show();
}

fn setup_drag_drop(
    window: &adw::ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    file_list: &FileListView,
    path_bar: &PathBar,
    title: &adw::WindowTitle,
    extract_btn: &Button,
    add_btn: &Button,
    status_bar: &GtkBox,
) {
    let drop_target = gtk4::DropTarget::new(gtk4::gio::File::static_type(), gtk4::gdk::DragAction::COPY);

    let state_clone = state;
    let file_list = file_list.clone();
    let path_bar = path_bar.clone();
    let title = title.clone();
    let extract_btn = extract_btn.clone();
    let add_btn = add_btn.clone();
    let status_bar = status_bar.clone();

    drop_target.connect_drop(move |_, value, _, _| {
        if let Ok(file) = value.get::<gtk4::gio::File>() {
            if let Some(path) = file.path() {
                // Check if it's an archive
                if ArchiveFormat::from_path(&path).is_some() {
                    open_archive(
                        &state_clone,
                        &path,
                        &file_list,
                        &path_bar,
                        &title,
                        &extract_btn,
                        &add_btn,
                        &status_bar,
                    );
                } else {
                    // Add file to current archive
                    let state = state_clone.borrow();
                    if state.archive.is_some() {
                        // Add file logic here
                    }
                }
                return true;
            }
        }
        false
    });

    window.add_controller(drop_target);
}
