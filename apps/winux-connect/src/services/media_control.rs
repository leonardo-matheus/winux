//! MPRIS media control service
//!
//! Provides bidirectional media control:
//! - Control phone media from PC
//! - Control PC media from phone (via MPRIS D-Bus interface)

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// MPRIS D-Bus interface
pub const MPRIS_BUS_NAME_PREFIX: &str = "org.mpris.MediaPlayer2.";
pub const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";
pub const MPRIS_PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

/// Media control service
pub struct MediaControlService {
    running: Arc<RwLock<bool>>,
    local_players: Arc<RwLock<HashMap<String, LocalMediaPlayer>>>,
    remote_players: Arc<RwLock<HashMap<String, RemoteMediaPlayer>>>,
    allow_remote_control: Arc<RwLock<bool>>,
}

impl MediaControlService {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            local_players: Arc::new(RwLock::new(HashMap::new())),
            remote_players: Arc::new(RwLock::new(HashMap::new())),
            allow_remote_control: Arc::new(RwLock::new(true)),
        }
    }

    /// Start the media control service
    pub fn start(&self) -> Result<(), String> {
        *self.running.write().unwrap() = true;

        // In production, would:
        // 1. Connect to session D-Bus
        // 2. Enumerate MPRIS players
        // 3. Listen for player changes

        self.enumerate_local_players()?;

        tracing::info!("Media control service started");
        Ok(())
    }

    /// Stop the media control service
    pub fn stop(&self) {
        *self.running.write().unwrap() = false;
        tracing::info!("Media control service stopped");
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        *self.running.read().unwrap()
    }

    /// Enable/disable remote control of local players
    pub fn set_allow_remote_control(&self, allow: bool) {
        *self.allow_remote_control.write().unwrap() = allow;
    }

    /// Check if remote control is allowed
    pub fn is_remote_control_allowed(&self) -> bool {
        *self.allow_remote_control.read().unwrap()
    }

    /// Enumerate local media players via MPRIS
    fn enumerate_local_players(&self) -> Result<(), String> {
        // In production, would query D-Bus for players
        // For now, add some sample players

        let players = vec![
            LocalMediaPlayer {
                name: "Spotify".to_string(),
                identity: "Spotify".to_string(),
                bus_name: "org.mpris.MediaPlayer2.spotify".to_string(),
                can_control: true,
                can_go_next: true,
                can_go_previous: true,
                can_pause: true,
                can_play: true,
                can_seek: true,
                playback_status: PlaybackStatus::Playing,
                current_track: Some(TrackInfo {
                    title: "Bohemian Rhapsody".to_string(),
                    artist: "Queen".to_string(),
                    album: "A Night at the Opera".to_string(),
                    duration: 354000, // ms
                    art_url: None,
                }),
                position: 125000,
                volume: 0.75,
            },
            LocalMediaPlayer {
                name: "VLC".to_string(),
                identity: "VLC media player".to_string(),
                bus_name: "org.mpris.MediaPlayer2.vlc".to_string(),
                can_control: true,
                can_go_next: true,
                can_go_previous: true,
                can_pause: true,
                can_play: true,
                can_seek: true,
                playback_status: PlaybackStatus::Paused,
                current_track: None,
                position: 0,
                volume: 1.0,
            },
        ];

        let mut local = self.local_players.write().unwrap();
        for player in players {
            local.insert(player.name.clone(), player);
        }

        Ok(())
    }

    /// Get local media players
    pub fn get_local_players(&self) -> Vec<LocalMediaPlayer> {
        self.local_players.read().unwrap().values().cloned().collect()
    }

    /// Get remote media players (from phone)
    pub fn get_remote_players(&self) -> Vec<RemoteMediaPlayer> {
        self.remote_players.read().unwrap().values().cloned().collect()
    }

    /// Control local player (from remote device)
    pub fn control_local(&self, player_name: &str, action: MediaAction) -> Result<(), String> {
        if !*self.allow_remote_control.read().unwrap() {
            return Err("Remote control not allowed".to_string());
        }

        let players = self.local_players.read().unwrap();
        if let Some(player) = players.get(player_name) {
            // In production, would send D-Bus command
            tracing::info!("Local player {} action: {:?}", player_name, action);
            Ok(())
        } else {
            Err(format!("Player {} not found", player_name))
        }
    }

    /// Control remote player (on phone)
    pub fn control_remote(&self, device_id: &str, player_name: &str, action: MediaAction) -> Result<(), String> {
        let players = self.remote_players.read().unwrap();
        let key = format!("{}:{}", device_id, player_name);

        if players.contains_key(&key) {
            // In production, would send packet via connection manager
            tracing::info!("Remote player {}:{} action: {:?}", device_id, player_name, action);
            Ok(())
        } else {
            Err(format!("Remote player {} not found on device {}", player_name, device_id))
        }
    }

    /// Update remote player info (from device packet)
    pub fn update_remote_player(&self, device_id: &str, player: RemoteMediaPlayer) {
        let key = format!("{}:{}", device_id, player.name);
        self.remote_players.write().unwrap().insert(key, player);
    }

    /// Remove remote player
    pub fn remove_remote_player(&self, device_id: &str, player_name: &str) {
        let key = format!("{}:{}", device_id, player_name);
        self.remote_players.write().unwrap().remove(&key);
    }

    /// Remove all remote players for a device
    pub fn remove_device_players(&self, device_id: &str) {
        self.remote_players.write().unwrap().retain(|k, _| !k.starts_with(device_id));
    }

    // Player control methods

    /// Play
    pub fn play(&self, player_name: &str) -> Result<(), String> {
        self.control_local(player_name, MediaAction::Play)
    }

    /// Pause
    pub fn pause(&self, player_name: &str) -> Result<(), String> {
        self.control_local(player_name, MediaAction::Pause)
    }

    /// Play/Pause toggle
    pub fn play_pause(&self, player_name: &str) -> Result<(), String> {
        self.control_local(player_name, MediaAction::PlayPause)
    }

    /// Stop
    pub fn stop(&self, player_name: &str) -> Result<(), String> {
        self.control_local(player_name, MediaAction::Stop)
    }

    /// Next track
    pub fn next(&self, player_name: &str) -> Result<(), String> {
        self.control_local(player_name, MediaAction::Next)
    }

    /// Previous track
    pub fn previous(&self, player_name: &str) -> Result<(), String> {
        self.control_local(player_name, MediaAction::Previous)
    }

    /// Seek to position (in milliseconds)
    pub fn seek(&self, player_name: &str, position: i64) -> Result<(), String> {
        self.control_local(player_name, MediaAction::Seek(position))
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, player_name: &str, volume: f64) -> Result<(), String> {
        self.control_local(player_name, MediaAction::SetVolume(volume))
    }
}

impl Default for MediaControlService {
    fn default() -> Self {
        Self::new()
    }
}

/// Local media player (on PC)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalMediaPlayer {
    pub name: String,
    pub identity: String,
    pub bus_name: String,
    pub can_control: bool,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_pause: bool,
    pub can_play: bool,
    pub can_seek: bool,
    pub playback_status: PlaybackStatus,
    pub current_track: Option<TrackInfo>,
    pub position: i64,
    pub volume: f64,
}

/// Remote media player (on phone)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteMediaPlayer {
    pub name: String,
    pub device_id: String,
    pub is_playing: bool,
    pub can_pause: bool,
    pub can_play: bool,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_seek: bool,
    pub current_track: Option<TrackInfo>,
    pub position: i64,
    pub duration: i64,
    pub volume: i32,
}

/// Track information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: i64, // milliseconds
    pub art_url: Option<String>,
}

/// Playback status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

/// Media control action
#[derive(Clone, Debug)]
pub enum MediaAction {
    Play,
    Pause,
    PlayPause,
    Stop,
    Next,
    Previous,
    Seek(i64),
    SetVolume(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_service_creation() {
        let service = MediaControlService::new();
        assert!(!service.is_running());
        assert!(service.is_remote_control_allowed());
    }

    #[test]
    fn test_start_stop() {
        let service = MediaControlService::new();
        service.start().unwrap();
        assert!(service.is_running());
        service.stop();
        assert!(!service.is_running());
    }

    #[test]
    fn test_remote_control_toggle() {
        let service = MediaControlService::new();
        assert!(service.is_remote_control_allowed());
        service.set_allow_remote_control(false);
        assert!(!service.is_remote_control_allowed());
    }

    #[test]
    fn test_local_players() {
        let service = MediaControlService::new();
        service.start().unwrap();

        let players = service.get_local_players();
        assert!(!players.is_empty());
    }
}
