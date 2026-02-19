//! Preview window for recordings
//!
//! Displays a preview of completed recordings with options to
//! play, trim, save, and share.

use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

use crate::AppState;
use crate::recording::{format_duration, format_file_size, OutputManager};

/// Preview window for a completed recording
pub struct PreviewWindow {
    window: adw::Window,
    video_path: PathBuf,
}

impl PreviewWindow {
    /// Create and show a preview window for a recording
    pub fn show(
        app: &gtk::Application,
        state: &Rc<RefCell<AppState>>,
        video_path: PathBuf,
    ) {
        let preview = Self::new(app, video_path.clone());
        preview.window.present();

        // Update state with recording path
        state.borrow_mut().recording_path = Some(video_path);
    }

    fn new(app: &gtk::Application, video_path: PathBuf) -> Self {
        let window = adw::Window::builder()
            .application(app)
            .title("Recording Preview")
            .default_width(800)
            .default_height(600)
            .modal(true)
            .build();

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();

        // Close button
        let close_button = gtk::Button::builder()
            .icon_name("window-close-symbolic")
            .build();

        let window_clone = window.clone();
        close_button.connect_clicked(move |_| {
            window_clone.close();
        });

        header.pack_start(&close_button);

        // Delete button
        let delete_button = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Delete recording")
            .css_classes(["flat"])
            .build();

        header.pack_end(&delete_button);

        content.append(&header);

        // Video preview area
        let video_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .css_classes(["view"])
            .build();

        // Placeholder for video player
        // In a full implementation, this would use GStreamer to play the video
        let video_placeholder = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .spacing(16)
            .vexpand(true)
            .build();

        let video_icon = gtk::Image::builder()
            .icon_name("video-x-generic-symbolic")
            .pixel_size(64)
            .css_classes(["dim-label"])
            .build();

        let play_button = gtk::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .label("Play in Video Player")
            .css_classes(["pill", "suggested-action"])
            .build();

        let path_clone = video_path.clone();
        play_button.connect_clicked(move |_| {
            let _ = open::that(&path_clone);
        });

        video_placeholder.append(&video_icon);
        video_placeholder.append(&play_button);

        video_box.append(&video_placeholder);
        content.append(&video_box);

        // Info and actions bar
        let info_bar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(16)
            .margin_start(16)
            .margin_end(16)
            .margin_top(16)
            .margin_bottom(16)
            .build();

        // File info
        let info_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .build();

        let filename = video_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Recording");

        let name_label = gtk::Label::builder()
            .label(filename)
            .css_classes(["title-4"])
            .halign(gtk::Align::Start)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .build();

        // Get file size
        let size_str = std::fs::metadata(&video_path)
            .map(|m| format_file_size(m.len()))
            .unwrap_or_else(|_| "Unknown size".to_string());

        let details_label = gtk::Label::builder()
            .label(&size_str)
            .css_classes(["dim-label"])
            .halign(gtk::Align::Start)
            .build();

        info_box.append(&name_label);
        info_box.append(&details_label);

        info_bar.append(&info_box);

        // Action buttons
        let action_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .build();

        // Open folder button
        let folder_button = gtk::Button::builder()
            .icon_name("folder-open-symbolic")
            .tooltip_text("Show in folder")
            .css_classes(["flat"])
            .build();

        let path_clone = video_path.clone();
        folder_button.connect_clicked(move |_| {
            if let Some(parent) = path_clone.parent() {
                let _ = open::that(parent);
            }
        });

        // Share button
        let share_button = gtk::Button::builder()
            .icon_name("emblem-shared-symbolic")
            .tooltip_text("Share")
            .css_classes(["flat"])
            .build();

        // Save as button
        let save_as_button = gtk::Button::builder()
            .label("Save As...")
            .css_classes(["flat"])
            .build();

        let path_clone = video_path.clone();
        let window_clone = window.clone();
        save_as_button.connect_clicked(move |_| {
            show_save_dialog(&window_clone, &path_clone);
        });

        action_box.append(&folder_button);
        action_box.append(&share_button);
        action_box.append(&save_as_button);

        info_bar.append(&action_box);

        content.append(&info_bar);

        // Trim section (collapsed by default)
        let trim_section = create_trim_section(&video_path);
        content.append(&trim_section);

        window.set_content(Some(&content));

        // Handle delete
        let window_clone = window.clone();
        let path_clone = video_path.clone();
        delete_button.connect_clicked(move |_| {
            show_delete_confirmation(&window_clone, &path_clone);
        });

        Self { window, video_path }
    }
}

fn create_trim_section(video_path: &PathBuf) -> adw::ExpanderRow {
    let expander = adw::ExpanderRow::builder()
        .title("Trim Video")
        .subtitle("Remove unwanted parts from beginning or end")
        .show_enable_switch(false)
        .build();

    // Trim controls
    let trim_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .margin_start(16)
        .margin_end(16)
        .margin_top(16)
        .margin_bottom(16)
        .build();

    // Timeline scrubber (placeholder)
    let timeline = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .adjustment(&gtk::Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0))
        .draw_value(false)
        .hexpand(true)
        .build();

    // Time range inputs
    let time_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(16)
        .halign(gtk::Align::Center)
        .build();

    let start_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let start_label = gtk::Label::new(Some("Start:"));
    let start_entry = gtk::Entry::builder()
        .placeholder_text("00:00")
        .width_chars(8)
        .build();

    start_box.append(&start_label);
    start_box.append(&start_entry);

    let end_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let end_label = gtk::Label::new(Some("End:"));
    let end_entry = gtk::Entry::builder()
        .placeholder_text("00:00")
        .width_chars(8)
        .build();

    end_box.append(&end_label);
    end_box.append(&end_entry);

    time_box.append(&start_box);
    time_box.append(&end_box);

    // Trim button
    let trim_button = gtk::Button::builder()
        .label("Trim and Save")
        .css_classes(["suggested-action"])
        .halign(gtk::Align::Center)
        .build();

    trim_box.append(&timeline);
    trim_box.append(&time_box);
    trim_box.append(&trim_button);

    // Wrap in a ListBoxRow
    let row = adw::ActionRow::builder()
        .activatable(false)
        .build();
    row.set_child(Some(&trim_box));

    expander.add_row(&row);

    expander
}

fn show_save_dialog(window: &adw::Window, source_path: &PathBuf) {
    let dialog = gtk::FileDialog::builder()
        .title("Save Recording As")
        .accept_label("Save")
        .build();

    // Set initial filename
    if let Some(filename) = source_path.file_name() {
        dialog.set_initial_name(Some(filename.to_string_lossy().as_ref()));
    }

    // Set initial folder to Videos
    if let Some(videos_dir) = dirs::video_dir() {
        if let Ok(folder) = gio::File::for_path(&videos_dir).query_info(
            "standard::*",
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        ) {
            // dialog.set_initial_folder(Some(&gio::File::for_path(&videos_dir)));
        }
    }

    let source_clone = source_path.clone();
    let window_clone = window.clone();

    dialog.save(
        Some(window),
        gio::Cancellable::NONE,
        move |result| {
            if let Ok(file) = result {
                if let Some(dest_path) = file.path() {
                    // Copy file
                    if let Err(e) = std::fs::copy(&source_clone, &dest_path) {
                        eprintln!("Failed to save file: {}", e);
                        // Show error toast
                    } else {
                        // Show success toast
                    }
                }
            }
        },
    );
}

fn show_delete_confirmation(window: &adw::Window, video_path: &PathBuf) {
    let dialog = adw::AlertDialog::builder()
        .heading("Delete Recording?")
        .body("This recording will be permanently deleted.")
        .build();

    dialog.add_responses(&[
        ("cancel", "Cancel"),
        ("delete", "Delete"),
    ]);

    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let path_clone = video_path.clone();
    let window_clone = window.clone();

    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            // Delete the file
            if let Err(e) = std::fs::remove_file(&path_clone) {
                eprintln!("Failed to delete file: {}", e);
            } else {
                window_clone.close();
            }
        }
    });

    dialog.present(Some(window));
}

/// Convert to GIF dialog
pub fn show_gif_conversion_dialog(window: &impl IsA<gtk::Window>, video_path: &PathBuf) {
    let dialog = adw::Dialog::builder()
        .title("Convert to GIF")
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(24)
        .build();

    // FPS selection
    let fps_row = adw::ComboRow::builder()
        .title("Frame Rate")
        .subtitle("Lower = smaller file size")
        .model(&gtk::StringList::new(&["10 FPS", "15 FPS", "20 FPS"]))
        .selected(1)
        .build();

    // Width selection
    let width_row = adw::ComboRow::builder()
        .title("Width")
        .subtitle("Height will scale proportionally")
        .model(&gtk::StringList::new(&["320px", "480px", "640px", "Original"]))
        .selected(2)
        .build();

    // Convert button
    let convert_button = gtk::Button::builder()
        .label("Convert")
        .css_classes(["suggested-action", "pill"])
        .halign(gtk::Align::Center)
        .margin_top(16)
        .build();

    let group = adw::PreferencesGroup::new();
    group.add(&fps_row);
    group.add(&width_row);

    content.append(&group);
    content.append(&convert_button);

    dialog.set_child(Some(&content));

    let dialog_clone = dialog.clone();
    let video_path_clone = video_path.clone();

    convert_button.connect_clicked(move |_| {
        let fps = match fps_row.selected() {
            0 => 10,
            1 => 15,
            2 => 20,
            _ => 15,
        };

        let width = match width_row.selected() {
            0 => Some(320),
            1 => Some(480),
            2 => Some(640),
            _ => None,
        };

        // Generate output path
        let mut output_path = video_path_clone.clone();
        output_path.set_extension("gif");

        // Start conversion
        let video_path = video_path_clone.clone();
        glib::spawn_future_local(async move {
            match crate::recording::output::convert_to_gif(
                &video_path,
                &output_path,
                fps,
                width,
            ).await {
                Ok(()) => {
                    // Show success
                    println!("GIF saved to: {}", output_path.display());
                }
                Err(e) => {
                    eprintln!("GIF conversion failed: {}", e);
                }
            }
        });

        dialog_clone.close();
    });

    dialog.present(Some(window));
}
