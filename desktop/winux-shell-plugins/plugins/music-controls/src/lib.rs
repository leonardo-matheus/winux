//! Music Controls Plugin
//!
//! MPRIS media player controls in the panel.

use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use winux_shell_plugins::prelude::*;

/// Playback status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl Default for PlaybackStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Media metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MediaMetadata {
    /// Track title
    title: String,
    /// Artist name(s)
    artist: String,
    /// Album name
    album: String,
    /// Album art URL/path
    art_url: Option<String>,
    /// Track length in microseconds
    length: u64,
    /// Current position in microseconds
    position: u64,
}

/// Player state
#[derive(Debug, Clone, Default)]
struct PlayerState {
    /// Playback status
    status: PlaybackStatus,
    /// Current metadata
    metadata: MediaMetadata,
    /// Player name (e.g., "Spotify", "Firefox")
    player_name: String,
    /// Player identity
    player_identity: String,
    /// Can go next
    can_next: bool,
    /// Can go previous
    can_previous: bool,
    /// Can play
    can_play: bool,
    /// Can pause
    can_pause: bool,
    /// Can seek
    can_seek: bool,
    /// Volume (0-100)
    volume: u32,
}

/// Music controls configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MusicConfig {
    /// Show album art
    show_album_art: bool,
    /// Show in panel when no media
    show_when_stopped: bool,
    /// Scroll wheel controls volume
    scroll_volume: bool,
    /// Preferred players (shown first)
    preferred_players: Vec<String>,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            show_album_art: true,
            show_when_stopped: false,
            scroll_volume: true,
            preferred_players: vec!["spotify".into(), "rhythmbox".into()],
        }
    }
}

/// Music controls plugin
pub struct MusicControlsPlugin {
    config: MusicConfig,
    state: Arc<RwLock<PlayerState>>,
    available_players: Arc<RwLock<Vec<String>>>,
}

impl Default for MusicControlsPlugin {
    fn default() -> Self {
        Self {
            config: MusicConfig::default(),
            state: Arc::new(RwLock::new(PlayerState::default())),
            available_players: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl MusicControlsPlugin {
    /// Format duration from microseconds
    fn format_duration(microseconds: u64) -> String {
        let total_secs = microseconds / 1_000_000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}:{:02}", mins, secs)
    }

    /// Get player icon based on name
    fn get_player_icon(player_name: &str) -> &'static str {
        let name = player_name.to_lowercase();
        if name.contains("spotify") {
            "spotify-symbolic"
        } else if name.contains("firefox") || name.contains("chromium") || name.contains("chrome") {
            "web-browser-symbolic"
        } else if name.contains("vlc") {
            "vlc-symbolic"
        } else if name.contains("rhythmbox") {
            "rhythmbox-symbolic"
        } else {
            "audio-x-generic-symbolic"
        }
    }

    /// Update mock player state
    fn update_state(&mut self) {
        // In a real implementation, this would query MPRIS via D-Bus
        // For now, set some mock data
        let mut state = self.state.write().unwrap();

        // Mock: simulate a playing track
        state.status = PlaybackStatus::Playing;
        state.player_name = "Spotify".to_string();
        state.player_identity = "org.mpris.MediaPlayer2.spotify".to_string();
        state.metadata = MediaMetadata {
            title: "Never Gonna Give You Up".to_string(),
            artist: "Rick Astley".to_string(),
            album: "Whenever You Need Somebody".to_string(),
            art_url: None,
            length: 212_000_000, // 3:32
            position: 45_000_000, // 0:45
        };
        state.can_next = true;
        state.can_previous = true;
        state.can_play = true;
        state.can_pause = true;
        state.can_seek = true;
        state.volume = 75;
    }

    /// Send play/pause command
    fn play_pause(&self) {
        log::info!("Music: Play/Pause");
        // In real impl, would call MPRIS PlayPause
    }

    /// Send next command
    fn next(&self) {
        log::info!("Music: Next");
        // In real impl, would call MPRIS Next
    }

    /// Send previous command
    fn previous(&self) {
        log::info!("Music: Previous");
        // In real impl, would call MPRIS Previous
    }

    /// Set volume
    fn set_volume(&self, volume: u32) {
        log::info!("Music: Set volume to {}%", volume);
        // In real impl, would call MPRIS SetVolume
    }

    /// Seek to position
    fn seek(&self, position: u64) {
        log::info!("Music: Seek to {}us", position);
        // In real impl, would call MPRIS Seek
    }
}

impl Plugin for MusicControlsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "org.winux.music-controls".into(),
            name: "Music Controls".into(),
            version: Version::new(1, 0, 0),
            description: "MPRIS media player controls in the panel".into(),
            authors: vec!["Winux Team".into()],
            homepage: Some("https://winux.org/plugins/music".into()),
            license: Some("MIT".into()),
            min_api_version: Version::new(1, 0, 0),
            capabilities: vec![PluginCapability::PanelWidget],
            permissions: {
                let mut perms = PermissionSet::new();
                perms.add(Permission::DBusSession);
                perms.add(Permission::PanelWidgets);
                perms.add(Permission::AudioControl);
                perms.add(Permission::OwnData);
                perms
            },
            icon: Some("audio-x-generic-symbolic".into()),
            category: Some("Multimedia".into()),
            keywords: vec!["music".into(), "media".into(), "player".into(), "mpris".into(), "spotify".into()],
            ..Default::default()
        }
    }

    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        // Load config
        let config_path = ctx.config_file("config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.config = config;
                }
            }
        }

        // Initial state update
        self.update_state();

        log::info!("Music controls plugin initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        log::info!("Music controls plugin shutting down");
        Ok(())
    }

    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        Some(Box::new(MusicPanelWidget {
            state: self.state.clone(),
            config: self.config.clone(),
        }))
    }

    fn command_provider(&self) -> Option<Box<dyn CommandProvider>> {
        Some(Box::new(MusicCommandProvider {
            state: self.state.clone(),
        }))
    }

    fn wants_updates(&self) -> bool {
        true
    }

    fn update_interval(&self) -> u32 {
        1000 // Update position every second
    }

    fn update(&mut self) -> PluginResult<()> {
        // In real impl, would refresh MPRIS state
        // For mock, just update position
        let mut state = self.state.write().unwrap();
        if state.status == PlaybackStatus::Playing {
            state.metadata.position += 1_000_000; // +1 second
            if state.metadata.position >= state.metadata.length {
                state.metadata.position = 0;
            }
        }
        Ok(())
    }
}

/// Panel widget for music controls
struct MusicPanelWidget {
    state: Arc<RwLock<PlayerState>>,
    config: MusicConfig,
}

impl PanelWidget for MusicPanelWidget {
    fn id(&self) -> &str {
        "music-controls"
    }

    fn name(&self) -> &str {
        "Music Controls"
    }

    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    fn size(&self) -> WidgetSize {
        WidgetSize::Medium
    }

    fn priority(&self) -> i32 {
        8
    }

    fn state(&self) -> WidgetState {
        let state = self.state.read().unwrap();

        if state.status == PlaybackStatus::Stopped && !self.config.show_when_stopped {
            return WidgetState::with_icon("audio-x-generic-symbolic").active(false);
        }

        let icon = match state.status {
            PlaybackStatus::Playing => "media-playback-pause-symbolic",
            PlaybackStatus::Paused => "media-playback-start-symbolic",
            PlaybackStatus::Stopped => "audio-x-generic-symbolic",
        };

        let label = if state.status != PlaybackStatus::Stopped {
            if state.metadata.title.len() > 20 {
                format!("{}...", &state.metadata.title[..17])
            } else {
                state.metadata.title.clone()
            }
        } else {
            String::new()
        };

        let tooltip = if state.status != PlaybackStatus::Stopped {
            format!(
                "{}\n{}\n{}",
                state.metadata.title,
                state.metadata.artist,
                state.player_name
            )
        } else {
            "No media playing".to_string()
        };

        WidgetState::with_icon(icon)
            .label(&label)
            .tooltip(&tooltip)
            .active(state.status == PlaybackStatus::Playing)
    }

    fn build_widget(&self) -> gtk::Widget {
        let state = self.state.read().unwrap();

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        hbox.set_valign(gtk::Align::Center);
        hbox.add_css_class("music-controls-widget");

        if state.status == PlaybackStatus::Stopped && !self.config.show_when_stopped {
            // Hidden when no media
            hbox.set_visible(false);
            return hbox.upcast();
        }

        // Player icon
        let player_icon = MusicControlsPlugin::get_player_icon(&state.player_name);
        let icon = gtk::Image::from_icon_name(player_icon);
        icon.set_pixel_size(16);
        hbox.append(&icon);

        if state.status != PlaybackStatus::Stopped {
            // Track info
            let info_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

            let title = gtk::Label::new(Some(&state.metadata.title));
            title.set_ellipsize(gtk::pango::EllipsizeMode::End);
            title.set_max_width_chars(20);
            title.set_halign(gtk::Align::Start);
            title.add_css_class("music-title");
            info_box.append(&title);

            let artist = gtk::Label::new(Some(&state.metadata.artist));
            artist.set_ellipsize(gtk::pango::EllipsizeMode::End);
            artist.set_max_width_chars(20);
            artist.set_halign(gtk::Align::Start);
            artist.add_css_class("dim-label");
            artist.add_css_class("caption");
            info_box.append(&artist);

            hbox.append(&info_box);

            // Play/Pause button
            let play_icon = if state.status == PlaybackStatus::Playing {
                "media-playback-pause-symbolic"
            } else {
                "media-playback-start-symbolic"
            };
            let play_btn = gtk::Button::from_icon_name(play_icon);
            play_btn.add_css_class("flat");
            play_btn.set_valign(gtk::Align::Center);
            hbox.append(&play_btn);
        }

        // Tooltip
        let tooltip = if state.status != PlaybackStatus::Stopped {
            format!(
                "{} - {}\n{}\n{} / {}",
                state.metadata.artist,
                state.metadata.title,
                state.player_name,
                MusicControlsPlugin::format_duration(state.metadata.position),
                MusicControlsPlugin::format_duration(state.metadata.length)
            )
        } else {
            "No media playing".to_string()
        };
        hbox.set_tooltip_text(Some(&tooltip));

        hbox.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        WidgetAction::ShowPopup
    }

    fn on_scroll(&mut self, delta: f64, direction: ScrollDirection) {
        if !self.config.scroll_volume {
            return;
        }

        let mut state = self.state.write().unwrap();
        let change = (delta.abs() * 5.0) as i32;

        match direction {
            ScrollDirection::Up => {
                state.volume = (state.volume as i32 + change).min(100) as u32;
            }
            ScrollDirection::Down => {
                state.volume = (state.volume as i32 - change).max(0) as u32;
            }
            _ => {}
        }
    }

    fn popup_config(&self) -> Option<PopupConfig> {
        Some(PopupConfig {
            width: 320,
            height: 280,
            ..Default::default()
        })
    }

    fn build_popup(&self) -> Option<gtk::Widget> {
        let state = self.state.read().unwrap();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
        vbox.set_margin_top(16);
        vbox.set_margin_bottom(16);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);
        vbox.add_css_class("music-popup");

        if state.status == PlaybackStatus::Stopped {
            let no_media = gtk::Label::new(Some("No media playing"));
            no_media.add_css_class("dim-label");
            no_media.set_vexpand(true);
            no_media.set_valign(gtk::Align::Center);
            vbox.append(&no_media);
            return Some(vbox.upcast());
        }

        // Album art and info
        let top_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        // Album art placeholder
        let art_frame = gtk::Frame::new(None);
        art_frame.set_size_request(80, 80);
        art_frame.add_css_class("album-art");

        if let Some(_art_url) = &state.metadata.art_url {
            // Would load actual album art here
            let art_icon = gtk::Image::from_icon_name("audio-x-generic-symbolic");
            art_icon.set_pixel_size(48);
            art_frame.set_child(Some(&art_icon));
        } else {
            let art_icon = gtk::Image::from_icon_name("audio-x-generic-symbolic");
            art_icon.set_pixel_size(48);
            art_frame.set_child(Some(&art_icon));
        }
        top_box.append(&art_frame);

        // Track info
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info_box.set_hexpand(true);
        info_box.set_valign(gtk::Align::Center);

        let title = gtk::Label::new(Some(&state.metadata.title));
        title.set_halign(gtk::Align::Start);
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title.add_css_class("title-4");
        info_box.append(&title);

        let artist = gtk::Label::new(Some(&state.metadata.artist));
        artist.set_halign(gtk::Align::Start);
        artist.set_ellipsize(gtk::pango::EllipsizeMode::End);
        info_box.append(&artist);

        let album = gtk::Label::new(Some(&state.metadata.album));
        album.set_halign(gtk::Align::Start);
        album.set_ellipsize(gtk::pango::EllipsizeMode::End);
        album.add_css_class("dim-label");
        info_box.append(&album);

        top_box.append(&info_box);
        vbox.append(&top_box);

        // Progress
        let progress_box = gtk::Box::new(gtk::Orientation::Vertical, 4);

        let progress = if state.metadata.length > 0 {
            state.metadata.position as f64 / state.metadata.length as f64
        } else {
            0.0
        };
        let progress_bar = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        progress_bar.set_value(progress);
        progress_bar.set_draw_value(false);
        progress_box.append(&progress_bar);

        let time_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let current_time = gtk::Label::new(Some(&MusicControlsPlugin::format_duration(state.metadata.position)));
        current_time.add_css_class("dim-label");
        current_time.add_css_class("caption");
        time_box.append(&current_time);

        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        time_box.append(&spacer);

        let total_time = gtk::Label::new(Some(&MusicControlsPlugin::format_duration(state.metadata.length)));
        total_time.add_css_class("dim-label");
        total_time.add_css_class("caption");
        time_box.append(&total_time);

        progress_box.append(&time_box);
        vbox.append(&progress_box);

        // Controls
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        controls.set_halign(gtk::Align::Center);
        controls.set_margin_top(8);

        let prev_btn = gtk::Button::from_icon_name("media-skip-backward-symbolic");
        prev_btn.add_css_class("circular");
        prev_btn.set_sensitive(state.can_previous);
        controls.append(&prev_btn);

        let play_icon = if state.status == PlaybackStatus::Playing {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        };
        let play_btn = gtk::Button::from_icon_name(play_icon);
        play_btn.add_css_class("circular");
        play_btn.add_css_class("suggested-action");
        play_btn.set_size_request(48, 48);
        controls.append(&play_btn);

        let next_btn = gtk::Button::from_icon_name("media-skip-forward-symbolic");
        next_btn.add_css_class("circular");
        next_btn.set_sensitive(state.can_next);
        controls.append(&next_btn);

        vbox.append(&controls);

        // Volume
        let volume_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        volume_box.set_margin_top(8);

        let vol_icon = gtk::Image::from_icon_name("audio-volume-high-symbolic");
        vol_icon.set_pixel_size(16);
        volume_box.append(&vol_icon);

        let volume_slider = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_slider.set_value(state.volume as f64);
        volume_slider.set_hexpand(true);
        volume_slider.set_draw_value(false);
        volume_box.append(&volume_slider);

        let vol_label = gtk::Label::new(Some(&format!("{}%", state.volume)));
        vol_label.add_css_class("dim-label");
        vol_label.set_width_chars(4);
        volume_box.append(&vol_label);

        vbox.append(&volume_box);

        // Player name
        let player_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        player_box.set_halign(gtk::Align::Center);
        player_box.set_margin_top(8);

        let player_icon = MusicControlsPlugin::get_player_icon(&state.player_name);
        let p_icon = gtk::Image::from_icon_name(player_icon);
        p_icon.set_pixel_size(12);
        player_box.append(&p_icon);

        let player_label = gtk::Label::new(Some(&state.player_name));
        player_label.add_css_class("dim-label");
        player_label.add_css_class("caption");
        player_box.append(&player_label);

        vbox.append(&player_box);

        Some(vbox.upcast())
    }
}

/// Command provider for music
struct MusicCommandProvider {
    state: Arc<RwLock<PlayerState>>,
}

impl CommandProvider for MusicCommandProvider {
    fn id(&self) -> &str {
        "music-commands"
    }

    fn commands(&self) -> Vec<Command> {
        let state = self.state.read().unwrap();
        let playing = state.status == PlaybackStatus::Playing;

        vec![
            Command::new(
                "music.play_pause",
                if playing { "Pause Music" } else { "Play Music" },
            )
            .with_description(if playing { "Pause playback" } else { "Resume playback" })
            .with_icon(if playing {
                "media-playback-pause-symbolic"
            } else {
                "media-playback-start-symbolic"
            })
            .with_shortcut("Media Play")
            .with_category("Media"),
            Command::new("music.next", "Next Track")
                .with_description("Skip to next track")
                .with_icon("media-skip-forward-symbolic")
                .with_shortcut("Media Next")
                .with_category("Media"),
            Command::new("music.previous", "Previous Track")
                .with_description("Go to previous track")
                .with_icon("media-skip-backward-symbolic")
                .with_shortcut("Media Previous")
                .with_category("Media"),
            Command::new("music.volume_up", "Volume Up")
                .with_description("Increase volume by 10%")
                .with_icon("audio-volume-high-symbolic")
                .with_category("Media")
                .hidden(),
            Command::new("music.volume_down", "Volume Down")
                .with_description("Decrease volume by 10%")
                .with_icon("audio-volume-low-symbolic")
                .with_category("Media")
                .hidden(),
        ]
    }

    fn execute(&mut self, command_id: &str, _context: &CommandContext) -> CommandResult {
        match command_id {
            "music.play_pause" => {
                let mut state = self.state.write().unwrap();
                state.status = match state.status {
                    PlaybackStatus::Playing => PlaybackStatus::Paused,
                    PlaybackStatus::Paused => PlaybackStatus::Playing,
                    PlaybackStatus::Stopped => PlaybackStatus::Stopped,
                };
                CommandResult::Success
            }
            "music.next" => {
                log::info!("Next track");
                CommandResult::Success
            }
            "music.previous" => {
                log::info!("Previous track");
                CommandResult::Success
            }
            "music.volume_up" => {
                let mut state = self.state.write().unwrap();
                state.volume = (state.volume + 10).min(100);
                CommandResult::Success
            }
            "music.volume_down" => {
                let mut state = self.state.write().unwrap();
                state.volume = state.volume.saturating_sub(10);
                CommandResult::Success
            }
            _ => CommandResult::Error(format!("Unknown command: {}", command_id)),
        }
    }
}

// Plugin entry point
winux_shell_plugins::declare_plugin!(MusicControlsPlugin, MusicControlsPlugin::default);
