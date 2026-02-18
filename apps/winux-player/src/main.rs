//! Winux Player - A simple media player for the Winux desktop environment
//!
//! This is a GTK4/libadwaita-based audio and video player with a clean,
//! modern interface supporting dark theme.

use adw::prelude::*;
use adw::Application;
use gtk4 as gtk;
use gtk::{gio, glib};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "org.winux.player";

/// Represents the current playback state
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Player state that persists across the application
struct PlayerState {
    playback_state: PlaybackState,
    current_position: f64,
    volume: f64,
    current_file: Option<String>,
    duration_secs: u64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            playback_state: PlaybackState::Stopped,
            current_position: 0.0,
            volume: 0.75,
            current_file: None,
            duration_secs: 0,
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

    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .build();

    let video_area = gtk::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .content_height(300)
        .content_width(500)
        .build();

    video_area.set_draw_func(|_, cr, width, height| {
        cr.set_source_rgb(0.1, 0.1, 0.12);
        cr.paint().unwrap();

        cr.set_source_rgb(0.3, 0.3, 0.35);
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let size = 60.0;

        cr.arc(center_x, center_y, size, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap();

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
        .label("Open a file to start playing")
        .css_classes(["dim-label"])
        .halign(gtk::Align::Start)
        .build();

    now_playing_box.append(&track_label);
    now_playing_box.append(&artist_label);

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

    let controls_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let stop_button = gtk::Button::builder()
        .icon_name("media-playback-stop-symbolic")
        .tooltip_text("Stop")
        .css_classes(["circular"])
        .build();

    let play_button = gtk::Button::builder()
        .icon_name("media-playback-start-symbolic")
        .tooltip_text("Play")
        .css_classes(["circular", "suggested-action"])
        .width_request(48)
        .height_request(48)
        .build();

    let prev_button = gtk::Button::builder()
        .icon_name("media-skip-backward-symbolic")
        .tooltip_text("Previous")
        .css_classes(["circular"])
        .build();

    let next_button = gtk::Button::builder()
        .icon_name("media-skip-forward-symbolic")
        .tooltip_text("Next")
        .css_classes(["circular"])
        .build();

    controls_box.append(&stop_button);
    controls_box.append(&prev_button);
    controls_box.append(&play_button);
    controls_box.append(&next_button);

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
        .build();

    let volume_scale = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .draw_value(false)
        .width_request(120)
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

    let state_clone = state.clone();
    let play_button_clone = play_button.clone();
    let track_label_clone = track_label.clone();
    let artist_label_clone = artist_label.clone();
    let video_area_clone = video_area.clone();
    let duration_label_clone = duration_label.clone();

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

        let audio_filter = gtk::FileFilter::new();
        audio_filter.set_name(Some("Audio Files"));
        audio_filter.add_mime_type("audio/*");
        filters.append(&audio_filter);

        let video_filter = gtk::FileFilter::new();
        video_filter.set_name(Some("Video Files"));
        video_filter.add_mime_type("video/*");
        filters.append(&video_filter);

        let all_filter = gtk::FileFilter::new();
        all_filter.set_name(Some("All Files"));
        all_filter.add_pattern("*");
        filters.append(&all_filter);

        file_dialog.set_filters(&filters);

        let state = state_clone.clone();
        let track_label = track_label_clone.clone();
        let artist_label = artist_label_clone.clone();
        let play_button = play_button_clone.clone();
        let video_area = video_area_clone.clone();
        let duration_label = duration_label_clone.clone();

        file_dialog.open(Some(&window_clone), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    {
                        let mut s = state.borrow_mut();
                        s.current_file = Some(path.to_string_lossy().to_string());
                        s.playback_state = PlaybackState::Stopped;
                        s.current_position = 0.0;
                        s.duration_secs = 180;
                    }

                    track_label.set_label(&filename);
                    artist_label.set_label("Ready to play");
                    duration_label.set_label("3:00");
                    play_button.set_icon_name("media-playback-start-symbolic");
                    video_area.queue_draw();
                }
            }
        });
    });

    let state_clone = state.clone();
    let artist_label_clone = artist_label.clone();
    play_button.connect_clicked(move |button| {
        let mut s = state_clone.borrow_mut();

        if s.current_file.is_none() {
            return;
        }

        match s.playback_state {
            PlaybackState::Stopped | PlaybackState::Paused => {
                s.playback_state = PlaybackState::Playing;
                button.set_icon_name("media-playback-pause-symbolic");
                button.set_tooltip_text(Some("Pause"));
                artist_label_clone.set_label("Playing...");
            }
            PlaybackState::Playing => {
                s.playback_state = PlaybackState::Paused;
                button.set_icon_name("media-playback-start-symbolic");
                button.set_tooltip_text(Some("Play"));
                artist_label_clone.set_label("Paused");
            }
        }
    });

    let state_clone = state.clone();
    let play_button_clone = play_button.clone();
    let progress_scale_clone = progress_scale.clone();
    let current_time_label_clone = current_time_label.clone();
    let artist_label_clone = artist_label.clone();
    stop_button.connect_clicked(move |_| {
        let mut s = state_clone.borrow_mut();
        s.playback_state = PlaybackState::Stopped;
        s.current_position = 0.0;

        play_button_clone.set_icon_name("media-playback-start-symbolic");
        play_button_clone.set_tooltip_text(Some("Play"));
        progress_scale_clone.set_value(0.0);
        current_time_label_clone.set_label("0:00");

        if s.current_file.is_some() {
            artist_label_clone.set_label("Stopped");
        }
    });

    let volume_button_clone = volume_button.clone();
    volume_scale.connect_value_changed(move |scale| {
        let value = scale.value();
        let icon_name = if value == 0.0 {
            "audio-volume-muted-symbolic"
        } else if value < 33.0 {
            "audio-volume-low-symbolic"
        } else if value < 66.0 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        };
        volume_button_clone.set_icon_name(icon_name);
    });

    let current_time_label_clone = current_time_label.clone();
    let state_clone = state.clone();
    progress_scale.connect_value_changed(move |scale| {
        let value = scale.value();
        let s = state_clone.borrow();

        let total_secs = (s.duration_secs as f64 * value / 100.0) as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        current_time_label_clone.set_label(&format!("{}:{:02}", mins, secs));
    });

    let state_clone = state.clone();
    let progress_scale_clone = progress_scale.clone();
    let current_time_label_clone = current_time_label.clone();
    let play_button_clone = play_button.clone();
    let artist_label_clone = artist_label.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        let mut s = state_clone.borrow_mut();

        if s.playback_state == PlaybackState::Playing {
            s.current_position += 0.5;

            if s.duration_secs > 0 {
                let progress = (s.current_position / s.duration_secs as f64) * 100.0;
                if progress >= 100.0 {
                    s.current_position = 0.0;
                    s.playback_state = PlaybackState::Stopped;
                    progress_scale_clone.set_value(0.0);
                    current_time_label_clone.set_label("0:00");
                    play_button_clone.set_icon_name("media-playback-start-symbolic");
                    artist_label_clone.set_label("Finished");
                } else {
                    progress_scale_clone.set_value(progress);

                    let mins = s.current_position as u64 / 60;
                    let secs = s.current_position as u64 % 60;
                    current_time_label_clone.set_label(&format!("{}:{:02}", mins, secs));
                }
            }
        }

        glib::ControlFlow::Continue
    });

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
