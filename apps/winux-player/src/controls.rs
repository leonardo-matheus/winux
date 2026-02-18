//! Video Controls - Playback control UI components
//!
//! Provides the control overlay with progress bar, play/pause buttons,
//! volume controls, and other playback settings.

use crate::player::{format_duration, PlayerState, PlayerWidget};
use crate::playlist::PlaylistManager;
use glib::clone;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tracing::info;

/// Video controls widget
#[derive(Clone)]
pub struct VideoControls {
    inner: Rc<VideoControlsInner>,
}

struct VideoControlsInner {
    /// Main container widget
    container: gtk4::Box,
    /// Progress bar/slider
    progress_bar: gtk4::Scale,
    /// Current time label
    current_time_label: gtk4::Label,
    /// Duration label
    duration_label: gtk4::Label,
    /// Play/Pause button
    play_button: gtk4::Button,
    /// Volume button
    volume_button: gtk4::VolumeButton,
    /// Speed button
    speed_button: gtk4::MenuButton,
    /// Speed label (shows current speed)
    speed_label: gtk4::Label,
    /// Fullscreen button
    fullscreen_button: gtk4::Button,
    /// Player reference
    player: PlayerWidget,
    /// Playlist manager
    playlist_manager: Rc<RefCell<PlaylistManager>>,
    /// A point for A-B repeat (in seconds)
    a_point: Cell<Option<f64>>,
    /// B point for A-B repeat (in seconds)
    b_point: Cell<Option<f64>>,
    /// A-B repeat indicator button
    ab_button: gtk4::ToggleButton,
    /// Seeking flag (to prevent feedback loop)
    seeking: Cell<bool>,
}

impl VideoControls {
    /// Create new video controls
    pub fn new(
        player: PlayerWidget,
        playlist_manager: Rc<RefCell<PlaylistManager>>,
    ) -> Self {
        // Main container
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        container.add_css_class("controls-overlay");
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_bottom(12);

        // Progress bar row
        let progress_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        progress_row.set_valign(gtk4::Align::Center);

        // Current time label
        let current_time_label = gtk4::Label::new(Some("00:00"));
        current_time_label.add_css_class("time-label");
        current_time_label.set_width_chars(8);
        progress_row.append(&current_time_label);

        // Progress bar/slider
        let progress_bar = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        progress_bar.set_hexpand(true);
        progress_bar.set_draw_value(false);
        progress_bar.add_css_class("progress-bar");
        progress_row.append(&progress_bar);

        // Duration label
        let duration_label = gtk4::Label::new(Some("00:00"));
        duration_label.add_css_class("time-label");
        duration_label.set_width_chars(8);
        progress_row.append(&duration_label);

        container.append(&progress_row);

        // Controls row
        let controls_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        controls_row.set_halign(gtk4::Align::Center);

        // Previous button
        let prev_button = gtk4::Button::from_icon_name("media-skip-backward-symbolic");
        prev_button.add_css_class("control-button");
        prev_button.set_tooltip_text(Some("Previous"));
        controls_row.append(&prev_button);

        // Seek backward button
        let seek_back_button = gtk4::Button::from_icon_name("media-seek-backward-symbolic");
        seek_back_button.add_css_class("control-button");
        seek_back_button.set_tooltip_text(Some("Seek Backward (-10s)"));
        controls_row.append(&seek_back_button);

        // Play/Pause button
        let play_button = gtk4::Button::from_icon_name("media-playback-start-symbolic");
        play_button.add_css_class("control-button");
        play_button.add_css_class("play-button");
        play_button.set_tooltip_text(Some("Play/Pause (Space)"));
        controls_row.append(&play_button);

        // Seek forward button
        let seek_fwd_button = gtk4::Button::from_icon_name("media-seek-forward-symbolic");
        seek_fwd_button.add_css_class("control-button");
        seek_fwd_button.set_tooltip_text(Some("Seek Forward (+10s)"));
        controls_row.append(&seek_fwd_button);

        // Next button
        let next_button = gtk4::Button::from_icon_name("media-skip-forward-symbolic");
        next_button.add_css_class("control-button");
        next_button.set_tooltip_text(Some("Next"));
        controls_row.append(&next_button);

        // Separator
        controls_row.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));

        // A-B repeat button
        let ab_button = gtk4::ToggleButton::new();
        ab_button.set_icon_name("media-playlist-repeat-symbolic");
        ab_button.add_css_class("control-button");
        ab_button.set_tooltip_text(Some("A-B Repeat ([ to set A, ] to set B, \\ to clear)"));
        controls_row.append(&ab_button);

        // Speed button with popover
        let speed_button = gtk4::MenuButton::new();
        let speed_label = gtk4::Label::new(Some("1.0x"));
        speed_label.add_css_class("speed-indicator");
        speed_button.set_child(Some(&speed_label));
        speed_button.add_css_class("control-button");
        speed_button.set_tooltip_text(Some("Playback Speed"));
        controls_row.append(&speed_button);

        // Spacer
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        controls_row.append(&spacer);

        // Volume button
        let volume_button = gtk4::VolumeButton::new();
        volume_button.set_value(1.0);
        volume_button.add_css_class("control-button");
        volume_button.set_tooltip_text(Some("Volume (M to mute)"));
        controls_row.append(&volume_button);

        // Fullscreen button
        let fullscreen_button = gtk4::Button::from_icon_name("view-fullscreen-symbolic");
        fullscreen_button.add_css_class("control-button");
        fullscreen_button.set_tooltip_text(Some("Fullscreen (F11)"));
        controls_row.append(&fullscreen_button);

        container.append(&controls_row);

        // Create speed menu popover
        let speed_popover = create_speed_popover(&player, &speed_label);
        speed_button.set_popover(Some(&speed_popover));

        let inner = Rc::new(VideoControlsInner {
            container,
            progress_bar,
            current_time_label,
            duration_label,
            play_button,
            volume_button,
            speed_button,
            speed_label,
            fullscreen_button,
            player: player.clone(),
            playlist_manager,
            a_point: Cell::new(None),
            b_point: Cell::new(None),
            ab_button,
            seeking: Cell::new(false),
        });

        let controls = Self { inner };

        // Setup connections
        controls.setup_connections(
            &prev_button,
            &seek_back_button,
            &seek_fwd_button,
            &next_button,
        );

        controls
    }

    fn setup_connections(
        &self,
        prev_button: &gtk4::Button,
        seek_back_button: &gtk4::Button,
        seek_fwd_button: &gtk4::Button,
        next_button: &gtk4::Button,
    ) {
        let inner = self.inner.clone();

        // Play/Pause button
        let player = self.inner.player.clone();
        let play_btn = self.inner.play_button.clone();
        self.inner.play_button.connect_clicked(move |_| {
            player.toggle_play();
        });

        // Update play button icon based on state
        let play_btn_clone = self.inner.play_button.clone();
        self.inner.player.connect_state_changed(move |state| {
            let icon_name = match state {
                PlayerState::Playing => "media-playback-pause-symbolic",
                _ => "media-playback-start-symbolic",
            };
            play_btn_clone.set_icon_name(icon_name);
        });

        // Seek buttons
        let player = self.inner.player.clone();
        seek_back_button.connect_clicked(move |_| {
            player.seek_relative(-10.0);
        });

        let player = self.inner.player.clone();
        seek_fwd_button.connect_clicked(move |_| {
            player.seek_relative(10.0);
        });

        // Previous/Next buttons
        let playlist = self.inner.playlist_manager.clone();
        let player = self.inner.player.clone();
        prev_button.connect_clicked(move |_| {
            if let Some(uri) = playlist.borrow_mut().previous() {
                player.load_uri(&uri);
                player.play();
            }
        });

        let playlist = self.inner.playlist_manager.clone();
        let player = self.inner.player.clone();
        next_button.connect_clicked(move |_| {
            if let Some(uri) = playlist.borrow_mut().next() {
                player.load_uri(&uri);
                player.play();
            }
        });

        // Progress bar seeking
        let inner_clone = inner.clone();
        self.inner.progress_bar.connect_value_changed(move |scale| {
            if !inner_clone.seeking.get() {
                return;
            }
            let value = scale.value();
            let duration = inner_clone.player.duration();
            if duration > 0.0 {
                let position = (value / 100.0) * duration;
                inner_clone.player.seek_absolute(position);
            }
        });

        // Track when user is dragging the progress bar
        let inner_clone = inner.clone();
        self.inner.progress_bar.connect_change_value(move |_, _, _| {
            inner_clone.seeking.set(true);
            glib::Propagation::Proceed
        });

        // Progress bar button release - user finished seeking
        let progress_bar_gesture = gtk4::GestureClick::new();
        let inner_clone = inner.clone();
        progress_bar_gesture.connect_released(move |_, _, _, _| {
            inner_clone.seeking.set(false);
        });
        self.inner.progress_bar.add_controller(progress_bar_gesture);

        // Volume control
        let player = self.inner.player.clone();
        self.inner.volume_button.connect_value_changed(move |_, value| {
            player.set_volume(value);
        });

        // Position update callback
        let current_time = self.inner.current_time_label.clone();
        let duration_label = self.inner.duration_label.clone();
        let progress_bar = self.inner.progress_bar.clone();
        let inner_clone = inner.clone();
        self.inner.player.connect_position_changed(move |position, duration| {
            current_time.set_text(&format_duration(position));
            duration_label.set_text(&format_duration(duration));

            // Update progress bar only if user is not seeking
            if !inner_clone.seeking.get() && duration > 0.0 {
                let percentage = (position / duration) * 100.0;
                progress_bar.set_value(percentage);
            }
        });

        // A-B repeat button
        let inner_clone = inner.clone();
        self.inner.ab_button.connect_toggled(move |btn| {
            if btn.is_active() {
                // Set A point at current position
                let position = inner_clone.player.position();
                inner_clone.a_point.set(Some(position));
                inner_clone.b_point.set(None);
                btn.set_tooltip_text(Some(&format!("A: {:.1}s - Press ] to set B point", position)));
            } else {
                // Clear A-B repeat
                inner_clone.player.clear_ab_repeat();
                inner_clone.a_point.set(None);
                inner_clone.b_point.set(None);
            }
        });
    }

    /// Get the main widget
    pub fn widget(&self) -> &gtk4::Box {
        &self.inner.container
    }

    /// Set A point at current position
    pub fn set_a_point(&self) {
        let position = self.inner.player.position();
        self.inner.a_point.set(Some(position));
        self.inner.player.set_a_point(position);
        self.inner.ab_button.set_active(true);
        info!("A point set at {:.2}s", position);
    }

    /// Set B point at current position
    pub fn set_b_point(&self) {
        let position = self.inner.player.position();
        if self.inner.a_point.get().is_some() {
            self.inner.b_point.set(Some(position));
            self.inner.player.set_b_point(position);
            info!("B point set at {:.2}s", position);
        }
    }

    /// Clear A-B repeat
    pub fn clear_ab_repeat(&self) {
        self.inner.a_point.set(None);
        self.inner.b_point.set(None);
        self.inner.player.clear_ab_repeat();
        self.inner.ab_button.set_active(false);
        info!("A-B repeat cleared");
    }

    /// Get fullscreen button for external binding
    pub fn fullscreen_button(&self) -> &gtk4::Button {
        &self.inner.fullscreen_button
    }

    /// Update volume display
    pub fn set_volume_display(&self, volume: f64) {
        self.inner.volume_button.set_value(volume);
    }

    /// Update speed display
    pub fn set_speed_display(&self, speed: f64) {
        self.inner.speed_label.set_text(&format!("{:.2}x", speed));
    }
}

/// Create speed selection popover
fn create_speed_popover(player: &PlayerWidget, speed_label: &gtk4::Label) -> gtk4::Popover {
    let popover = gtk4::Popover::new();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);

    let speeds = [
        ("0.25x", 0.25),
        ("0.5x", 0.5),
        ("0.75x", 0.75),
        ("Normal", 1.0),
        ("1.25x", 1.25),
        ("1.5x", 1.5),
        ("1.75x", 1.75),
        ("2x", 2.0),
    ];

    for (label_text, speed) in speeds {
        let btn = gtk4::Button::with_label(label_text);
        btn.add_css_class("flat");

        let player_clone = player.clone();
        let label_clone = speed_label.clone();
        let popover_clone = popover.clone();
        btn.connect_clicked(move |_| {
            player_clone.set_speed(speed);
            label_clone.set_text(&format!("{:.2}x", speed));
            popover_clone.popdown();
        });

        vbox.append(&btn);
    }

    // Custom speed slider
    vbox.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    let custom_label = gtk4::Label::new(Some("Custom Speed"));
    custom_label.add_css_class("dim-label");
    vbox.append(&custom_label);

    let speed_slider = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.25, 2.0, 0.05);
    speed_slider.set_value(1.0);
    speed_slider.set_width_request(150);

    let player_clone = player.clone();
    let label_clone = speed_label.clone();
    speed_slider.connect_value_changed(move |scale| {
        let speed = scale.value();
        player_clone.set_speed(speed);
        label_clone.set_text(&format!("{:.2}x", speed));
    });

    vbox.append(&speed_slider);

    popover.set_child(Some(&vbox));
    popover
}

/// Thumbnail preview for progress bar hover
pub struct ProgressPreview {
    popover: gtk4::Popover,
    image: gtk4::Picture,
    time_label: gtk4::Label,
}

impl ProgressPreview {
    /// Create new progress preview
    pub fn new() -> Self {
        let popover = gtk4::Popover::new();
        popover.set_position(gtk4::PositionType::Top);

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        vbox.set_margin_start(4);
        vbox.set_margin_end(4);
        vbox.set_margin_top(4);
        vbox.set_margin_bottom(4);

        let image = gtk4::Picture::new();
        image.set_size_request(160, 90);
        vbox.append(&image);

        let time_label = gtk4::Label::new(Some("00:00"));
        time_label.add_css_class("time-label");
        vbox.append(&time_label);

        popover.set_child(Some(&vbox));

        Self {
            popover,
            image,
            time_label,
        }
    }

    /// Show preview at position
    pub fn show(&self, time: f64) {
        self.time_label.set_text(&format_duration(time));
        self.popover.popup();
    }

    /// Hide preview
    pub fn hide(&self) {
        self.popover.popdown();
    }

    /// Get popover widget
    pub fn popover(&self) -> &gtk4::Popover {
        &self.popover
    }
}

impl Default for ProgressPreview {
    fn default() -> Self {
        Self::new()
    }
}
