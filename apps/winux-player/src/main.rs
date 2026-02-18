//! Winux Player - A media player launcher for the Winux desktop environment
//!
//! This is a GTK4/libadwaita-based media player UI that opens files
//! with the system default player (VLC, Totem, etc.)

use adw::prelude::*;
use adw::Application;
use gtk4 as gtk;
use gtk::gio;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "org.winux.player";

/// Player state
struct PlayerState {
    current_file: Option<String>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current_file: None,
        }
    }
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        let settings = adw::StyleManager::default();
        settings.set_color_scheme(adw::ColorScheme::ForceDark);
    });

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(PlayerState::default()));

    // Header bar
    let header = adw::HeaderBar::new();

    let open_button = gtk::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open media file")
        .build();

    header.pack_start(&open_button);

    let title_label = gtk::Label::builder()
        .label("Winux Player")
        .css_classes(["title"])
        .build();
    header.set_title_widget(Some(&title_label));

    // Main content box
    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .build();

    // Video area placeholder (mockup display)
    let video_area = gtk::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .content_height(300)
        .content_width(500)
        .build();

    video_area.set_draw_func(|_, cr, width, height| {
        // Dark background
        cr.set_source_rgb(0.1, 0.1, 0.12);
        cr.paint().unwrap();

        // Play button circle
        cr.set_source_rgb(0.3, 0.3, 0.35);
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let size = 60.0;

        cr.arc(center_x, center_y, size, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap();

        // Play triangle
        cr.set_source_rgb(0.5, 0.5, 0.55);
        cr.move_to(center_x - size * 0.3, center_y - size * 0.4);
        cr.line_to(center_x - size * 0.3, center_y + size * 0.4);
        cr.line_to(center_x + size * 0.5, center_y);
        cr.close_path();
        cr.fill().unwrap();
    });

    let video_frame = gtk::Frame::builder()
        .child(&video_area)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .build();

    // Now playing info
    let now_playing_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .build();

    let track_label = gtk::Label::builder()
        .label("No media loaded")
        .css_classes(["title-3"])
        .halign(gtk::Align::Start)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let artist_label = gtk::Label::builder()
        .label("Open a file to launch in system player")
        .css_classes(["dim-label"])
        .halign(gtk::Align::Start)
        .build();

    now_playing_box.append(&track_label);
    now_playing_box.append(&artist_label);

    // Progress bar (mockup - non-functional)
    let progress_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .build();

    let current_time_label = gtk::Label::builder()
        .label("0:00")
        .css_classes(["caption", "dim-label"])
        .width_chars(5)
        .build();

    let progress_scale = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .draw_value(false)
        .sensitive(false)
        .build();
    progress_scale.set_range(0.0, 100.0);
    progress_scale.set_value(0.0);

    let duration_label = gtk::Label::builder()
        .label("0:00")
        .css_classes(["caption", "dim-label"])
        .width_chars(5)
        .build();

    progress_box.append(&current_time_label);
    progress_box.append(&progress_scale);
    progress_box.append(&duration_label);

    // Controls
    let controls_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let play_button = gtk::Button::builder()
        .icon_name("media-playback-start-symbolic")
        .tooltip_text("Open in system player")
        .css_classes(["circular", "suggested-action"])
        .width_request(48)
        .height_request(48)
        .build();

    controls_box.append(&play_button);

    // Volume controls (mockup)
    let volume_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(12)
        .halign(gtk::Align::End)
        .build();

    let volume_button = gtk::Button::builder()
        .icon_name("audio-volume-high-symbolic")
        .tooltip_text("Volume")
        .css_classes(["flat"])
        .sensitive(false)
        .build();

    let volume_scale = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .draw_value(false)
        .width_request(120)
        .sensitive(false)
        .build();
    volume_scale.set_range(0.0, 100.0);
    volume_scale.set_value(75.0);

    volume_box.append(&volume_button);
    volume_box.append(&volume_scale);

    let bottom_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .build();

    let spacer = gtk::Box::builder()
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

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Winux Player")
        .default_width(600)
        .default_height(500)
        .content(&toolbar_view)
        .build();

    // Open button click handler
    let state_clone = state.clone();
    let track_label_clone = track_label.clone();
    let artist_label_clone = artist_label.clone();
    let window_clone = window.clone();

    open_button.connect_clicked(move |_| {
        let file_dialog = gtk::FileDialog::builder()
            .title("Open Media File")
            .modal(true)
            .build();

        let filters = gio::ListStore::new::<gtk::FileFilter>();

        let media_filter = gtk::FileFilter::new();
        media_filter.set_name(Some("Media Files"));
        media_filter.add_mime_type("audio/*");
        media_filter.add_mime_type("video/*");
        filters.append(&media_filter);

        let all_filter = gtk::FileFilter::new();
        all_filter.set_name(Some("All Files"));
        all_filter.add_pattern("*");
        filters.append(&all_filter);

        file_dialog.set_filters(&filters);

        let state = state_clone.clone();
        let track_label = track_label_clone.clone();
        let artist_label = artist_label_clone.clone();

        file_dialog.open(Some(&window_clone), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let path_str = path.to_string_lossy().to_string();

                    {
                        let mut s = state.borrow_mut();
                        s.current_file = Some(path_str.clone());
                    }

                    track_label.set_label(&filename);
                    artist_label.set_label("Click play to open in system player");

                    // Open file with system default player
                    if let Err(e) = open::that(&path_str) {
                        eprintln!("Failed to open file with system player: {}", e);
                        artist_label.set_label(&format!("Error: {}", e));
                    } else {
                        artist_label.set_label("Opened in system player");
                    }
                }
            }
        });
    });

    // Play button click handler - opens current file in system player
    let state_clone = state.clone();
    let artist_label_clone = artist_label.clone();

    play_button.connect_clicked(move |_| {
        let s = state_clone.borrow();
        if let Some(ref path) = s.current_file {
            if let Err(e) = open::that(path) {
                eprintln!("Failed to open file with system player: {}", e);
                artist_label_clone.set_label(&format!("Error: {}", e));
            } else {
                artist_label_clone.set_label("Opened in system player");
            }
        }
    });

    // Custom CSS
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string(
        r#"
        .title-3 {
            font-weight: bold;
            font-size: 1.2em;
        }

        .circular {
            border-radius: 9999px;
            min-width: 36px;
            min-height: 36px;
        }

        scale {
            padding: 8px 0;
        }

        scale trough {
            border-radius: 4px;
            min-height: 6px;
        }

        scale highlight {
            border-radius: 4px;
        }
        "#
    );

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not get default display"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}
