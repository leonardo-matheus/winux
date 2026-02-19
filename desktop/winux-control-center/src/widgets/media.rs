//! Media Player widget
//!
//! Provides playback controls for the currently playing media.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Adjustment, Box, Button, Image, Label, Orientation, Scale};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tracing::info;

/// Media playback state
#[derive(Clone, Debug, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

/// Currently playing media information
#[derive(Clone, Debug)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_art_url: Option<String>,
    pub duration_secs: u64,
    pub position_secs: u64,
    pub state: PlaybackState,
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            title: "Not Playing".to_string(),
            artist: String::new(),
            album: String::new(),
            album_art_url: None,
            duration_secs: 0,
            position_secs: 0,
            state: PlaybackState::Stopped,
        }
    }
}

/// Media Player control widget
pub struct MediaPlayerWidget {
    container: Box,
    album_art: Image,
    title_label: Label,
    artist_label: Label,
    play_pause_btn: Button,
    progress_slider: Scale,
    media_info: Rc<RefCell<MediaInfo>>,
}

impl MediaPlayerWidget {
    /// Create a new Media Player widget
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 12);
        container.add_css_class("media-player");

        // Main content area
        let content = Box::new(Orientation::Horizontal, 12);

        // Album art
        let album_art = Image::from_icon_name("media-optical-symbolic");
        album_art.add_css_class("album-art");
        album_art.set_pixel_size(80);
        content.append(&album_art);

        // Track info
        let info_box = Box::new(Orientation::Vertical, 4);
        info_box.set_hexpand(true);
        info_box.set_valign(gtk::Align::Center);

        let title_label = Label::new(Some("Not Playing"));
        title_label.add_css_class("media-title");
        title_label.set_halign(gtk::Align::Start);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title_label.set_max_width_chars(20);
        info_box.append(&title_label);

        let artist_label = Label::new(Some(""));
        artist_label.add_css_class("media-artist");
        artist_label.set_halign(gtk::Align::Start);
        artist_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        artist_label.set_max_width_chars(25);
        info_box.append(&artist_label);

        // Playback controls
        let controls = Box::new(Orientation::Horizontal, 8);
        controls.add_css_class("media-controls");
        controls.set_halign(gtk::Align::Start);

        // Previous button
        let prev_btn = Button::builder()
            .icon_name("media-skip-backward-symbolic")
            .build();
        prev_btn.add_css_class("media-button");
        prev_btn.add_css_class("flat");
        prev_btn.connect_clicked(|_| {
            Self::media_command("Previous");
        });
        controls.append(&prev_btn);

        // Play/Pause button
        let play_pause_btn = Button::builder()
            .icon_name("media-playback-start-symbolic")
            .build();
        play_pause_btn.add_css_class("media-button");
        play_pause_btn.add_css_class("play-pause");
        controls.append(&play_pause_btn);

        // Next button
        let next_btn = Button::builder()
            .icon_name("media-skip-forward-symbolic")
            .build();
        next_btn.add_css_class("media-button");
        next_btn.add_css_class("flat");
        next_btn.connect_clicked(|_| {
            Self::media_command("Next");
        });
        controls.append(&next_btn);

        info_box.append(&controls);
        content.append(&info_box);

        container.append(&content);

        // Progress slider
        let progress_adjustment = Adjustment::new(
            0.0,   // value
            0.0,   // lower
            100.0, // upper (will be updated with duration)
            1.0,   // step increment
            10.0,  // page increment
            0.0,   // page size
        );

        let progress_slider = Scale::new(Orientation::Horizontal, Some(&progress_adjustment));
        progress_slider.add_css_class("media-progress");
        progress_slider.set_draw_value(false);
        progress_slider.set_hexpand(true);

        // Time labels
        let time_box = Box::new(Orientation::Horizontal, 4);

        let current_time = Label::new(Some("0:00"));
        current_time.add_css_class("caption");
        current_time.add_css_class("dim-label");
        time_box.append(&current_time);

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        time_box.append(&spacer);

        let total_time = Label::new(Some("0:00"));
        total_time.add_css_class("caption");
        total_time.add_css_class("dim-label");
        time_box.append(&total_time);

        container.append(&progress_slider);
        container.append(&time_box);

        let media_info = Rc::new(RefCell::new(MediaInfo::default()));

        // Connect play/pause handler
        let media_info_clone = media_info.clone();
        let play_pause_btn_clone = play_pause_btn.clone();

        play_pause_btn.connect_clicked(move |_| {
            let info = media_info_clone.borrow();
            match info.state {
                PlaybackState::Playing => {
                    Self::media_command("Pause");
                    play_pause_btn_clone.set_icon_name("media-playback-start-symbolic");
                }
                PlaybackState::Paused | PlaybackState::Stopped => {
                    Self::media_command("Play");
                    play_pause_btn_clone.set_icon_name("media-playback-pause-symbolic");
                }
            }
        });

        // Connect progress slider
        let current_time_clone = current_time.clone();
        progress_slider.connect_value_changed(move |scale| {
            let value = scale.value() as u64;
            current_time_clone.set_text(&Self::format_time(value));
        });

        // Load mock data for demonstration
        let widget = Self {
            container,
            album_art,
            title_label,
            artist_label,
            play_pause_btn,
            progress_slider,
            media_info,
        };

        widget.load_mock_data();
        widget
    }

    /// Load mock media data for demonstration
    fn load_mock_data(&self) {
        let mock_info = MediaInfo {
            title: "Starlight".to_string(),
            artist: "Taylor Swift".to_string(),
            album: "Red".to_string(),
            album_art_url: None,
            duration_secs: 245,
            position_secs: 67,
            state: PlaybackState::Playing,
        };

        self.update_media_info(mock_info);
    }

    /// Update the displayed media information
    pub fn update_media_info(&self, info: MediaInfo) {
        self.title_label.set_text(&info.title);
        self.artist_label.set_text(&info.artist);

        // Update play/pause button icon
        let icon = match info.state {
            PlaybackState::Playing => "media-playback-pause-symbolic",
            PlaybackState::Paused | PlaybackState::Stopped => "media-playback-start-symbolic",
        };
        self.play_pause_btn.set_icon_name(icon);

        // Update progress
        if info.duration_secs > 0 {
            self.progress_slider
                .adjustment()
                .set_upper(info.duration_secs as f64);
            self.progress_slider.set_value(info.position_secs as f64);
        }

        // Update album art if available
        if let Some(_url) = &info.album_art_url {
            // In a real implementation, we would download and display the image
            // For now, use a placeholder
        }

        *self.media_info.borrow_mut() = info;
    }

    /// Format seconds as MM:SS
    fn format_time(seconds: u64) -> String {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{}:{:02}", mins, secs)
    }

    /// Send a command to the media player via MPRIS
    fn media_command(command: &str) {
        info!("Media command: {}", command);

        // Use playerctl for MPRIS control
        let _ = std::process::Command::new("playerctl")
            .arg(command.to_lowercase())
            .spawn();
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Check if media is currently playing
    pub fn is_playing(&self) -> bool {
        self.media_info.borrow().state == PlaybackState::Playing
    }

    /// Start periodic updates to sync with the media player
    pub fn start_updates(&self) {
        // In a real implementation, we would:
        // 1. Connect to MPRIS D-Bus signals
        // 2. Poll playerctl for current state
        // 3. Update the UI accordingly

        let media_info = self.media_info.clone();
        let progress_slider = self.progress_slider.clone();

        gtk::glib::timeout_add_seconds_local(1, move || {
            let mut info = media_info.borrow_mut();
            if info.state == PlaybackState::Playing && info.position_secs < info.duration_secs {
                info.position_secs += 1;
                progress_slider.set_value(info.position_secs as f64);
            }
            gtk::glib::ControlFlow::Continue
        });
    }
}

impl Default for MediaPlayerWidget {
    fn default() -> Self {
        Self::new()
    }
}
