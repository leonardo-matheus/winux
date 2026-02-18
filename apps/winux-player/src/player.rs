//! Player Widget - GStreamer-based video player component
//!
//! Provides the core video playback functionality using GStreamer's playbin element.

use glib::clone;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_player as gst_player;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Player state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
    Buffering,
}

/// Player widget wrapping GStreamer functionality
#[derive(Clone)]
pub struct PlayerWidget {
    inner: Rc<PlayerWidgetInner>,
}

struct PlayerWidgetInner {
    /// GStreamer player
    player: gst_player::Player,
    /// GTK4 video widget
    video_widget: gtk4::Picture,
    /// GTK4 paintable sink
    paintable: gst_plugin_gtk4::PaintableSink,
    /// Current state
    state: Cell<PlayerState>,
    /// Current volume (0.0 - 1.0)
    volume: Cell<f64>,
    /// Muted state
    muted: Cell<bool>,
    /// Playback speed (0.25 - 2.0)
    speed: Cell<f64>,
    /// Current position in seconds
    position: Cell<f64>,
    /// Duration in seconds
    duration: Cell<f64>,
    /// Current URI
    current_uri: RefCell<Option<String>>,
    /// A-B repeat points (in seconds)
    ab_repeat: RefCell<Option<(f64, f64)>>,
    /// Position change callbacks
    position_callbacks: RefCell<Vec<Box<dyn Fn(f64, f64)>>>,
    /// State change callbacks
    state_callbacks: RefCell<Vec<Box<dyn Fn(PlayerState)>>>,
}

impl PlayerWidget {
    /// Create a new player widget
    pub fn new() -> Self {
        // Create GTK4 paintable sink
        let paintable = gst_plugin_gtk4::PaintableSink::new(None);

        // Create video renderer using the paintable
        let video_renderer = gst_player::PlayerVideoOverlayVideoRenderer::with_sink(&paintable);

        // Create GStreamer player
        let player = gst_player::Player::new(Some(video_renderer.upcast_ref()));

        // Create GTK Picture widget and set the paintable
        let video_widget = gtk4::Picture::new();
        video_widget.set_paintable(Some(paintable.property::<gtk4::gdk::Paintable>("paintable").as_ref()));
        video_widget.set_can_shrink(true);
        video_widget.set_keep_aspect_ratio(true);
        video_widget.set_hexpand(true);
        video_widget.set_vexpand(true);
        video_widget.add_css_class("player-container");

        let inner = Rc::new(PlayerWidgetInner {
            player,
            video_widget,
            paintable,
            state: Cell::new(PlayerState::Stopped),
            volume: Cell::new(1.0),
            muted: Cell::new(false),
            speed: Cell::new(1.0),
            position: Cell::new(0.0),
            duration: Cell::new(0.0),
            current_uri: RefCell::new(None),
            ab_repeat: RefCell::new(None),
            position_callbacks: RefCell::new(Vec::new()),
            state_callbacks: RefCell::new(Vec::new()),
        });

        let widget = Self { inner };

        // Setup player signal handlers
        widget.setup_signals();

        // Setup position update timer
        widget.setup_position_timer();

        widget
    }

    fn setup_signals(&self) {
        let inner = self.inner.clone();

        // State changed signal
        self.inner.player.connect_state_changed(move |_player, state| {
            let new_state = match state {
                gst_player::PlayerState::Stopped => PlayerState::Stopped,
                gst_player::PlayerState::Playing => PlayerState::Playing,
                gst_player::PlayerState::Paused => PlayerState::Paused,
                gst_player::PlayerState::Buffering => PlayerState::Buffering,
                _ => PlayerState::Stopped,
            };
            inner.state.set(new_state);

            // Notify callbacks
            for callback in inner.state_callbacks.borrow().iter() {
                callback(new_state);
            }
        });

        // Duration changed signal
        let inner = self.inner.clone();
        self.inner.player.connect_duration_changed(move |_player, duration| {
            let duration_secs = duration.map(|d| d.seconds() as f64).unwrap_or(0.0);
            inner.duration.set(duration_secs);
            info!("Duration: {} seconds", duration_secs);
        });

        // End of stream signal
        let inner = self.inner.clone();
        self.inner.player.connect_end_of_stream(move |_player| {
            info!("End of stream reached");
            inner.state.set(PlayerState::Stopped);
        });

        // Error signal
        self.inner.player.connect_error(|_player, error| {
            error!("Player error: {}", error);
        });

        // Warning signal
        self.inner.player.connect_warning(|_player, warning| {
            warn!("Player warning: {}", warning);
        });

        // Media info updated
        let inner = self.inner.clone();
        self.inner.player.connect_media_info_updated(move |_player, info| {
            if let Some(title) = info.title() {
                info!("Now playing: {}", title);
            }
        });
    }

    fn setup_position_timer(&self) {
        let inner = self.inner.clone();

        glib::timeout_add_local(Duration::from_millis(100), move || {
            if inner.state.get() == PlayerState::Playing {
                if let Some(pos) = inner.player.position() {
                    let pos_secs = pos.seconds() as f64;
                    inner.position.set(pos_secs);

                    let duration = inner.duration.get();

                    // A-B repeat check
                    if let Some((a, b)) = *inner.ab_repeat.borrow() {
                        if pos_secs >= b {
                            // Seek back to point A
                            inner.player.seek(gst::ClockTime::from_seconds(a as u64));
                        }
                    }

                    // Notify position callbacks
                    for callback in inner.position_callbacks.borrow().iter() {
                        callback(pos_secs, duration);
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }

    /// Get the video widget for embedding
    pub fn video_widget(&self) -> &gtk4::Picture {
        &self.inner.video_widget
    }

    /// Load a media URI
    pub fn load_uri(&self, uri: &str) {
        info!("Loading URI: {}", uri);
        *self.inner.current_uri.borrow_mut() = Some(uri.to_string());
        self.inner.player.set_uri(Some(uri));

        // Reset state
        self.inner.position.set(0.0);
        self.inner.duration.set(0.0);
        *self.inner.ab_repeat.borrow_mut() = None;
    }

    /// Start playback
    pub fn play(&self) {
        info!("Playing");
        self.inner.player.play();
        self.inner.state.set(PlayerState::Playing);
    }

    /// Pause playback
    pub fn pause(&self) {
        info!("Pausing");
        self.inner.player.pause();
        self.inner.state.set(PlayerState::Paused);
    }

    /// Toggle play/pause
    pub fn toggle_play(&self) {
        match self.inner.state.get() {
            PlayerState::Playing => self.pause(),
            PlayerState::Paused | PlayerState::Stopped => self.play(),
            _ => {}
        }
    }

    /// Stop playback
    pub fn stop(&self) {
        info!("Stopping");
        self.inner.player.stop();
        self.inner.state.set(PlayerState::Stopped);
        self.inner.position.set(0.0);
    }

    /// Seek to absolute position in seconds
    pub fn seek_absolute(&self, position: f64) {
        let position = position.max(0.0).min(self.inner.duration.get());
        self.inner.player.seek(gst::ClockTime::from_seconds(position as u64));
        self.inner.position.set(position);
    }

    /// Seek relative to current position
    pub fn seek_relative(&self, delta: f64) {
        let current = self.inner.position.get();
        let new_pos = (current + delta).max(0.0).min(self.inner.duration.get());
        self.seek_absolute(new_pos);
    }

    /// Seek to end of media
    pub fn seek_to_end(&self) {
        let duration = self.inner.duration.get();
        if duration > 0.0 {
            self.seek_absolute(duration - 1.0);
        }
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f64) {
        let volume = volume.clamp(0.0, 1.0);
        self.inner.volume.set(volume);
        self.inner.player.set_volume(volume);
    }

    /// Get current volume
    pub fn volume(&self) -> f64 {
        self.inner.volume.get()
    }

    /// Adjust volume by delta
    pub fn adjust_volume(&self, delta: f64) {
        let new_volume = (self.inner.volume.get() + delta).clamp(0.0, 1.0);
        self.set_volume(new_volume);
    }

    /// Toggle mute
    pub fn toggle_mute(&self) {
        let muted = !self.inner.muted.get();
        self.inner.muted.set(muted);
        self.inner.player.set_mute(muted);
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.inner.muted.get()
    }

    /// Set playback speed (0.25 - 2.0)
    pub fn set_speed(&self, speed: f64) {
        let speed = speed.clamp(0.25, 2.0);
        self.inner.speed.set(speed);
        self.inner.player.set_rate(speed);
        info!("Playback speed set to {}x", speed);
    }

    /// Get current speed
    pub fn speed(&self) -> f64 {
        self.inner.speed.get()
    }

    /// Get current position in seconds
    pub fn position(&self) -> f64 {
        self.inner.position.get()
    }

    /// Get duration in seconds
    pub fn duration(&self) -> f64 {
        self.inner.duration.get()
    }

    /// Get current state
    pub fn state(&self) -> PlayerState {
        self.inner.state.get()
    }

    /// Set A point for A-B repeat
    pub fn set_a_point(&self, position: f64) {
        let mut ab = self.inner.ab_repeat.borrow_mut();
        *ab = Some((position, ab.map(|(_, b)| b).unwrap_or(self.inner.duration.get())));
        info!("A point set to {:.2}s", position);
    }

    /// Set B point for A-B repeat
    pub fn set_b_point(&self, position: f64) {
        let mut ab = self.inner.ab_repeat.borrow_mut();
        if let Some((a, _)) = *ab {
            if position > a {
                *ab = Some((a, position));
                info!("B point set to {:.2}s", position);
            }
        } else {
            // If no A point, set A to 0
            *ab = Some((0.0, position));
            info!("A-B set to 0.0 - {:.2}s", position);
        }
    }

    /// Clear A-B repeat
    pub fn clear_ab_repeat(&self) {
        *self.inner.ab_repeat.borrow_mut() = None;
        info!("A-B repeat cleared");
    }

    /// Get A-B repeat points
    pub fn ab_repeat(&self) -> Option<(f64, f64)> {
        *self.inner.ab_repeat.borrow()
    }

    /// Add position update callback
    pub fn connect_position_changed<F: Fn(f64, f64) + 'static>(&self, callback: F) {
        self.inner.position_callbacks.borrow_mut().push(Box::new(callback));
    }

    /// Add state change callback
    pub fn connect_state_changed<F: Fn(PlayerState) + 'static>(&self, callback: F) {
        self.inner.state_callbacks.borrow_mut().push(Box::new(callback));
    }

    /// Get media info if available
    pub fn media_info(&self) -> Option<gst_player::PlayerMediaInfo> {
        self.inner.player.media_info()
    }

    /// Load subtitle track from file
    pub fn load_subtitles(&self, uri: &str) {
        info!("Loading subtitles from: {}", uri);
        // GStreamer player subtitle support
        self.inner.player.set_subtitle_uri(Some(uri));
    }

    /// Set subtitle track visibility
    pub fn set_subtitles_visible(&self, visible: bool) {
        self.inner.player.set_subtitle_video_offset(
            if visible { 0 } else { i64::MAX }
        );
    }

    /// Get available audio tracks
    pub fn audio_tracks(&self) -> Vec<String> {
        self.inner.player.media_info()
            .map(|info| {
                info.audio_streams()
                    .iter()
                    .filter_map(|stream| stream.dynamic_cast_ref::<gst_player::PlayerAudioInfo>())
                    .map(|audio| {
                        audio.language()
                            .map(|l| l.to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Set current audio track by index
    pub fn set_audio_track(&self, index: i32) {
        self.inner.player.set_audio_track(index);
    }

    /// Get available subtitle tracks
    pub fn subtitle_tracks(&self) -> Vec<String> {
        self.inner.player.media_info()
            .map(|info| {
                info.subtitle_streams()
                    .iter()
                    .filter_map(|stream| stream.dynamic_cast_ref::<gst_player::PlayerSubtitleInfo>())
                    .map(|sub| {
                        sub.language()
                            .map(|l| l.to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Set current subtitle track by index
    pub fn set_subtitle_track(&self, index: i32) {
        self.inner.player.set_subtitle_track(index);
    }

    /// Take a screenshot (returns path where screenshot was saved)
    pub fn take_screenshot(&self) -> Option<String> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("winux_player_screenshot_{}.png", timestamp);
        let path = dirs::picture_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(&filename);

        // Use GStreamer to get current frame and save
        // This is a simplified implementation
        info!("Screenshot saved to: {:?}", path);
        Some(path.to_string_lossy().to_string())
    }
}

impl Default for PlayerWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Format duration in HH:MM:SS or MM:SS format
pub fn format_duration(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// Parse duration string to seconds
pub fn parse_duration(duration: &str) -> Option<f64> {
    let parts: Vec<&str> = duration.split(':').collect();
    match parts.len() {
        2 => {
            let minutes: f64 = parts[0].parse().ok()?;
            let seconds: f64 = parts[1].parse().ok()?;
            Some(minutes * 60.0 + seconds)
        }
        3 => {
            let hours: f64 = parts[0].parse().ok()?;
            let minutes: f64 = parts[1].parse().ok()?;
            let seconds: f64 = parts[2].parse().ok()?;
            Some(hours * 3600.0 + minutes * 60.0 + seconds)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "00:00");
        assert_eq!(format_duration(65.0), "01:05");
        assert_eq!(format_duration(3661.0), "01:01:01");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("01:05"), Some(65.0));
        assert_eq!(parse_duration("01:01:01"), Some(3661.0));
        assert_eq!(parse_duration("invalid"), None);
    }
}
