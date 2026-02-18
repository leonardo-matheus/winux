//! Winux Player - Media player launcher for Winux OS
//!
//! Opens media files with the system default player (VLC, Totem, etc.)

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Label, Orientation, Scale, Frame, DrawingArea};
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "org.winux.player";

fn main() -> gtk::glib::ExitCode {
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let current_file: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // Apply dark theme
    let style_manager = adw::StyleManager::default();
    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

    // Header bar
    let header = adw::HeaderBar::new();

    let open_button = Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open media file")
        .build();
    header.pack_start(&open_button);

    let title_label = Label::builder()
        .label("Winux Player")
        .css_classes(vec!["title"])
        .build();
    header.set_title_widget(Some(&title_label));

    // Main content
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // Video area placeholder
    let video_area = DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .content_height(300)
        .content_width(500)
        .build();

    video_area.set_draw_func(|_, cr, width, height| {
        // Dark background
        cr.set_source_rgb(0.1, 0.1, 0.12);
        let _ = cr.paint();

        // Play button circle
        cr.set_source_rgb(0.3, 0.3, 0.35);
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let size = 60.0;

        cr.arc(center_x, center_y, size, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.fill();

        // Play triangle
        cr.set_source_rgb(0.5, 0.5, 0.55);
        cr.move_to(center_x - size * 0.3, center_y - size * 0.4);
        cr.line_to(center_x - size * 0.3, center_y + size * 0.4);
        cr.line_to(center_x + size * 0.5, center_y);
        cr.close_path();
        let _ = cr.fill();
    });

    let video_frame = Frame::builder()
        .child(&video_area)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .build();

    // Now playing info
    let now_playing_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .build();

    let track_label = Label::builder()
        .label("No media loaded")
        .css_classes(vec!["title-3"])
        .halign(gtk::Align::Start)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let artist_label = Label::builder()
        .label("Open a file to launch in system player")
        .css_classes(vec!["dim-label"])
        .halign(gtk::Align::Start)
        .build();

    now_playing_box.append(&track_label);
    now_playing_box.append(&artist_label);

    // Progress bar (mockup)
    let progress_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .build();

    let current_time = Label::builder()
        .label("0:00")
        .css_classes(vec!["caption", "dim-label"])
        .width_chars(5)
        .build();

    let progress_scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .draw_value(false)
        .sensitive(false)
        .build();
    progress_scale.set_range(0.0, 100.0);
    progress_scale.set_value(0.0);

    let duration = Label::builder()
        .label("0:00")
        .css_classes(vec!["caption", "dim-label"])
        .width_chars(5)
        .build();

    progress_box.append(&current_time);
    progress_box.append(&progress_scale);
    progress_box.append(&duration);

    // Controls
    let controls_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let play_button = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .tooltip_text("Open in system player")
        .css_classes(vec!["circular", "suggested-action"])
        .width_request(48)
        .height_request(48)
        .build();

    controls_box.append(&play_button);

    // Volume (mockup)
    let volume_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(12)
        .halign(gtk::Align::End)
        .build();

    let volume_button = Button::builder()
        .icon_name("audio-volume-high-symbolic")
        .css_classes(vec!["flat"])
        .sensitive(false)
        .build();

    let volume_scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .draw_value(false)
        .width_request(120)
        .sensitive(false)
        .build();
    volume_scale.set_range(0.0, 100.0);
    volume_scale.set_value(75.0);

    volume_box.append(&volume_button);
    volume_box.append(&volume_scale);

    let bottom_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    let spacer = Box::builder()
        .hexpand(true)
        .build();

    bottom_box.append(&controls_box);
    bottom_box.append(&spacer);
    bottom_box.append(&volume_box);

    main_box.append(&video_frame);
    main_box.append(&now_playing_box);
    main_box.append(&progress_box);
    main_box.append(&bottom_box);

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&main_box));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Winux Player")
        .default_width(600)
        .default_height(500)
        .build();

    window.set_content(Some(&toolbar_view));

    // Open button handler
    let current_file_clone = current_file.clone();
    let track_label_clone = track_label.clone();
    let artist_label_clone = artist_label.clone();
    let window_clone = window.clone();

    open_button.connect_clicked(move |_| {
        let file_dialog = gtk::FileDialog::builder()
            .title("Open Media File")
            .modal(true)
            .build();

        let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();

        let media_filter = gtk::FileFilter::new();
        media_filter.set_name(Some("Media Files"));
        media_filter.add_mime_type("audio/*");
        media_filter.add_mime_type("video/*");
        filters.append(&media_filter);

        file_dialog.set_filters(Some(&filters));

        let current_file = current_file_clone.clone();
        let track_label = track_label_clone.clone();
        let artist_label = artist_label_clone.clone();

        file_dialog.open(Some(&window_clone), gtk::gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let path_str = path.to_string_lossy().to_string();
                    *current_file.borrow_mut() = Some(path_str.clone());

                    track_label.set_label(&filename);
                    artist_label.set_label("Click play to open in system player");

                    if let Err(e) = open::that(&path_str) {
                        eprintln!("Failed to open: {}", e);
                        artist_label.set_label(&format!("Error: {}", e));
                    } else {
                        artist_label.set_label("Opened in system player");
                    }
                }
            }
        });
    });

    // Play button handler
    let current_file_clone = current_file.clone();
    let artist_label_clone = artist_label.clone();

    play_button.connect_clicked(move |_| {
        if let Some(ref path) = *current_file_clone.borrow() {
            if let Err(e) = open::that(path) {
                eprintln!("Failed to open: {}", e);
                artist_label_clone.set_label(&format!("Error: {}", e));
            } else {
                artist_label_clone.set_label("Opened in system player");
            }
        }
    });

    window.present();
}
