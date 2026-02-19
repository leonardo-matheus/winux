//! Quick gallery for viewing recent photos/videos

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Orientation, Label, Picture, ScrolledWindow, FlowBox, Button, Frame};
use libadwaita as adw;
use adw::prelude::*;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::window::AppState;

/// Build the gallery view
pub fn build_gallery(state: Rc<RefCell<AppState>>) -> GtkBox {
    let gallery_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Gallery header
    let header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_start(16)
        .margin_end(16)
        .margin_top(16)
        .margin_bottom(8)
        .build();

    let title = Label::builder()
        .label("Recent Captures")
        .css_classes(["title-2"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .build();

    let open_folder_button = Button::builder()
        .icon_name("folder-open-symbolic")
        .tooltip_text("Open in Files")
        .css_classes(["flat", "circular"])
        .build();

    let state_clone = state.clone();
    open_folder_button.connect_clicked(move |_| {
        let state = state_clone.borrow();
        let output_dir = state.capture_settings.output_directory.clone();
        // Open folder in file manager
        if let Err(e) = open::that(&output_dir) {
            log::error!("Failed to open folder: {}", e);
        }
    });

    header.append(&title);
    header.append(&open_folder_button);

    // Gallery content
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    let flow_box = FlowBox::builder()
        .valign(gtk::Align::Start)
        .max_children_per_line(4)
        .min_children_per_line(2)
        .selection_mode(gtk::SelectionMode::Single)
        .homogeneous(true)
        .column_spacing(8)
        .row_spacing(8)
        .margin_start(16)
        .margin_end(16)
        .margin_top(8)
        .margin_bottom(16)
        .build();

    // Load recent images
    let state_clone = state.clone();
    let flow_box_clone = flow_box.clone();

    // Initial load
    load_gallery_items(&flow_box_clone, &state_clone.borrow().capture_settings.output_directory);

    // Refresh button
    let refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Refresh")
        .css_classes(["flat", "circular"])
        .build();

    let state_clone = state.clone();
    let flow_box_clone = flow_box.clone();
    refresh_button.connect_clicked(move |_| {
        let state = state_clone.borrow();
        load_gallery_items(&flow_box_clone, &state.capture_settings.output_directory);
    });

    header.append(&refresh_button);

    scrolled.set_child(Some(&flow_box));

    gallery_box.append(&header);
    gallery_box.append(&scrolled);

    // Empty state if no photos
    let empty_state = build_empty_state();
    gallery_box.append(&empty_state);

    gallery_box
}

/// Load gallery items from the output directory
fn load_gallery_items(flow_box: &FlowBox, output_dir: &PathBuf) {
    // Clear existing items
    while let Some(child) = flow_box.first_child() {
        flow_box.remove(&child);
    }

    // Create directory if it doesn't exist
    if !output_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(output_dir) {
            log::error!("Failed to create output directory: {}", e);
            return;
        }
    }

    // Read image files from directory
    let image_extensions = ["jpg", "jpeg", "png", "webp", "mp4", "webm", "mkv"];

    let mut files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if image_extensions.contains(&ext_str.as_str()) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            files.push((path, modified));
                        }
                    }
                }
            }
        }
    }

    // Sort by modification time (newest first)
    files.sort_by(|a, b| b.1.cmp(&a.1));

    // Limit to recent files
    let max_items = 20;
    for (path, _) in files.into_iter().take(max_items) {
        let item = build_gallery_item(&path);
        flow_box.append(&item);
    }
}

/// Build a single gallery item widget
fn build_gallery_item(path: &PathBuf) -> GtkBox {
    let item_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["gallery-item"])
        .build();

    let thumbnail_frame = Frame::builder()
        .build();

    // Check if it's a video or image
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_video = ["mp4", "webm", "mkv"].contains(&ext.as_str());

    if is_video {
        // Video thumbnail placeholder
        let video_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .width_request(120)
            .height_request(90)
            .css_classes(["video-thumbnail"])
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        let play_icon = gtk::Image::builder()
            .icon_name("media-playback-start-symbolic")
            .pixel_size(32)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        video_box.append(&play_icon);
        thumbnail_frame.set_child(Some(&video_box));
    } else {
        // Image thumbnail
        let picture = Picture::builder()
            .content_fit(gtk::ContentFit::Cover)
            .width_request(120)
            .height_request(90)
            .build();

        let file = gtk::gio::File::for_path(path);
        picture.set_file(Some(&file));

        thumbnail_frame.set_child(Some(&picture));
    }

    // File name label
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");

    let label = Label::builder()
        .label(filename)
        .css_classes(["caption"])
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .max_width_chars(15)
        .build();

    // Make item clickable
    let gesture = gtk::GestureClick::new();
    let path_clone = path.clone();
    gesture.connect_released(move |_, _, _, _| {
        // Open with default application
        open_with_viewer(&path_clone);
    });
    thumbnail_frame.add_controller(gesture);

    item_box.append(&thumbnail_frame);
    item_box.append(&label);

    item_box
}

/// Build empty state widget
fn build_empty_state() -> GtkBox {
    let empty_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .vexpand(true)
        .visible(false)
        .build();

    let icon = gtk::Image::builder()
        .icon_name("camera-photo-symbolic")
        .pixel_size(64)
        .css_classes(["dim-label"])
        .build();

    let title = Label::builder()
        .label("No Photos Yet")
        .css_classes(["title-2", "dim-label"])
        .build();

    let subtitle = Label::builder()
        .label("Take some photos to see them here")
        .css_classes(["dim-label"])
        .build();

    empty_box.append(&icon);
    empty_box.append(&title);
    empty_box.append(&subtitle);

    empty_box
}

/// Open file with appropriate viewer
fn open_with_viewer(path: &PathBuf) {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_video = ["mp4", "webm", "mkv"].contains(&ext.as_str());

    if is_video {
        // Open video with winux-player or default
        if let Err(_) = std::process::Command::new("winux-player")
            .arg(path)
            .spawn()
        {
            // Fallback to system default
            let _ = open::that(path);
        }
    } else {
        // Open image with winux-image or default
        if let Err(_) = std::process::Command::new("winux-image")
            .arg(path)
            .spawn()
        {
            // Fallback to system default
            let _ = open::that(path);
        }
    }
}

/// Get formatted file size
#[allow(dead_code)]
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get formatted date
#[allow(dead_code)]
fn format_date(time: std::time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M").to_string()
}
