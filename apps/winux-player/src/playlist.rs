//! Playlist Manager - Media playlist management
//!
//! Handles playlist operations including adding, removing, reordering items,
//! shuffle, repeat modes, and persistence.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Repeat mode for playlist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    /// No repeat
    None,
    /// Repeat single track
    One,
    /// Repeat entire playlist
    All,
}

/// Single playlist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    /// URI of the media file
    pub uri: String,
    /// Display title
    pub title: String,
    /// Duration in seconds (if known)
    pub duration: Option<f64>,
    /// Artist/Author (if available)
    pub artist: Option<String>,
    /// Album (if available)
    pub album: Option<String>,
}

impl PlaylistItem {
    /// Create a new playlist item from URI
    pub fn from_uri(uri: &str) -> Self {
        // Extract filename from URI for display
        let title = uri
            .rsplit('/')
            .next()
            .and_then(|s| s.rsplit('\\').next())
            .map(|s| urlencoding::decode(s).unwrap_or_else(|_| s.into()).to_string())
            .unwrap_or_else(|| uri.to_string());

        Self {
            uri: uri.to_string(),
            title,
            duration: None,
            artist: None,
            album: None,
        }
    }

    /// Set metadata
    pub fn set_metadata(&mut self, title: Option<String>, artist: Option<String>, album: Option<String>, duration: Option<f64>) {
        if let Some(t) = title {
            self.title = t;
        }
        self.artist = artist;
        self.album = album;
        self.duration = duration;
    }
}

/// Playlist manager
#[derive(Debug, Clone)]
pub struct PlaylistManager {
    /// Playlist items
    items: Vec<PlaylistItem>,
    /// Current item index
    current_index: Option<usize>,
    /// Repeat mode
    repeat_mode: RepeatMode,
    /// Shuffle enabled
    shuffle: bool,
    /// Shuffle order (indices)
    shuffle_order: Vec<usize>,
    /// Current shuffle position
    shuffle_position: usize,
    /// History for back navigation
    history: VecDeque<usize>,
    /// Maximum history size
    max_history: usize,
}

impl PlaylistManager {
    /// Create a new empty playlist manager
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: None,
            repeat_mode: RepeatMode::None,
            shuffle: false,
            shuffle_order: Vec::new(),
            shuffle_position: 0,
            history: VecDeque::new(),
            max_history: 50,
        }
    }

    /// Add an item to the playlist
    pub fn add_item(&mut self, uri: &str) {
        let item = PlaylistItem::from_uri(uri);
        self.items.push(item);
        info!("Added to playlist: {}", uri);

        // If this is the first item, set it as current
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }

        // Update shuffle order
        if self.shuffle {
            self.regenerate_shuffle_order();
        }
    }

    /// Add multiple items to the playlist
    pub fn add_items(&mut self, uris: &[String]) {
        for uri in uris {
            self.add_item(uri);
        }
    }

    /// Remove item at index
    pub fn remove_item(&mut self, index: usize) -> Option<PlaylistItem> {
        if index >= self.items.len() {
            return None;
        }

        let item = self.items.remove(index);

        // Adjust current index if necessary
        if let Some(current) = self.current_index {
            if index < current {
                self.current_index = Some(current - 1);
            } else if index == current {
                // Current item was removed
                if self.items.is_empty() {
                    self.current_index = None;
                } else if index >= self.items.len() {
                    self.current_index = Some(self.items.len() - 1);
                }
            }
        }

        if self.shuffle {
            self.regenerate_shuffle_order();
        }

        Some(item)
    }

    /// Clear the playlist
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_index = None;
        self.shuffle_order.clear();
        self.shuffle_position = 0;
        self.history.clear();
        info!("Playlist cleared");
    }

    /// Get current item URI
    pub fn current_uri(&self) -> Option<String> {
        self.current_index
            .and_then(|i| self.items.get(i))
            .map(|item| item.uri.clone())
    }

    /// Get current item
    pub fn current_item(&self) -> Option<&PlaylistItem> {
        self.current_index.and_then(|i| self.items.get(i))
    }

    /// Get next item URI
    pub fn next(&mut self) -> Option<String> {
        if self.items.is_empty() {
            return None;
        }

        // Save current to history
        if let Some(current) = self.current_index {
            self.history.push_back(current);
            if self.history.len() > self.max_history {
                self.history.pop_front();
            }
        }

        let next_index = if self.shuffle {
            self.next_shuffle_index()
        } else {
            self.next_sequential_index()
        };

        self.current_index = next_index;
        self.current_uri()
    }

    fn next_sequential_index(&self) -> Option<usize> {
        match self.current_index {
            Some(current) => {
                let next = current + 1;
                if next >= self.items.len() {
                    match self.repeat_mode {
                        RepeatMode::All => Some(0),
                        RepeatMode::One => Some(current),
                        RepeatMode::None => None,
                    }
                } else {
                    Some(next)
                }
            }
            None => {
                if !self.items.is_empty() {
                    Some(0)
                } else {
                    None
                }
            }
        }
    }

    fn next_shuffle_index(&mut self) -> Option<usize> {
        if self.shuffle_order.is_empty() {
            self.regenerate_shuffle_order();
        }

        self.shuffle_position += 1;

        if self.shuffle_position >= self.shuffle_order.len() {
            match self.repeat_mode {
                RepeatMode::All => {
                    self.regenerate_shuffle_order();
                    self.shuffle_position = 0;
                }
                RepeatMode::One => {
                    self.shuffle_position = self.shuffle_position.saturating_sub(1);
                }
                RepeatMode::None => {
                    return None;
                }
            }
        }

        self.shuffle_order.get(self.shuffle_position).copied()
    }

    /// Get previous item URI
    pub fn previous(&mut self) -> Option<String> {
        if self.items.is_empty() {
            return None;
        }

        // Try to get from history first
        if let Some(prev_index) = self.history.pop_back() {
            self.current_index = Some(prev_index);
            return self.current_uri();
        }

        // Otherwise, go to previous sequential item
        let prev_index = match self.current_index {
            Some(current) => {
                if current == 0 {
                    match self.repeat_mode {
                        RepeatMode::All => Some(self.items.len() - 1),
                        RepeatMode::One => Some(current),
                        RepeatMode::None => Some(0),
                    }
                } else {
                    Some(current - 1)
                }
            }
            None => {
                if !self.items.is_empty() {
                    Some(self.items.len() - 1)
                } else {
                    None
                }
            }
        };

        self.current_index = prev_index;
        self.current_uri()
    }

    /// Jump to specific index
    pub fn jump_to(&mut self, index: usize) -> Option<String> {
        if index >= self.items.len() {
            return None;
        }

        // Save current to history
        if let Some(current) = self.current_index {
            self.history.push_back(current);
            if self.history.len() > self.max_history {
                self.history.pop_front();
            }
        }

        self.current_index = Some(index);
        self.current_uri()
    }

    /// Set repeat mode
    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
        info!("Repeat mode set to: {:?}", mode);
    }

    /// Get repeat mode
    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    /// Cycle through repeat modes
    pub fn cycle_repeat_mode(&mut self) -> RepeatMode {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::None => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::None,
        };
        self.repeat_mode
    }

    /// Set shuffle enabled
    pub fn set_shuffle(&mut self, enabled: bool) {
        self.shuffle = enabled;
        if enabled {
            self.regenerate_shuffle_order();
        }
        info!("Shuffle: {}", enabled);
    }

    /// Toggle shuffle
    pub fn toggle_shuffle(&mut self) -> bool {
        self.set_shuffle(!self.shuffle);
        self.shuffle
    }

    /// Check if shuffle is enabled
    pub fn is_shuffle(&self) -> bool {
        self.shuffle
    }

    /// Regenerate shuffle order
    fn regenerate_shuffle_order(&mut self) {
        use std::collections::HashSet;

        self.shuffle_order = (0..self.items.len()).collect();

        // Fisher-Yates shuffle
        let mut rng_state: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        for i in (1..self.shuffle_order.len()).rev() {
            // Simple LCG random number generator
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (rng_state as usize) % (i + 1);
            self.shuffle_order.swap(i, j);
        }

        self.shuffle_position = 0;
    }

    /// Move item from one position to another
    pub fn move_item(&mut self, from: usize, to: usize) {
        if from >= self.items.len() || to >= self.items.len() {
            return;
        }

        let item = self.items.remove(from);
        self.items.insert(to, item);

        // Adjust current index
        if let Some(current) = self.current_index {
            if current == from {
                self.current_index = Some(to);
            } else if from < current && to >= current {
                self.current_index = Some(current - 1);
            } else if from > current && to <= current {
                self.current_index = Some(current + 1);
            }
        }
    }

    /// Get all items
    pub fn items(&self) -> &[PlaylistItem] {
        &self.items
    }

    /// Get number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if playlist is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get current index
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Save playlist to file
    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.items)?;
        fs::write(path, json)?;
        info!("Playlist saved to: {:?}", path);
        Ok(())
    }

    /// Load playlist from file
    pub fn load(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        let content = fs::read_to_string(path)?;
        let items: Vec<PlaylistItem> = serde_json::from_str(&content)?;
        self.items = items;
        self.current_index = if self.items.is_empty() {
            None
        } else {
            Some(0)
        };
        if self.shuffle {
            self.regenerate_shuffle_order();
        }
        info!("Playlist loaded from: {:?}", path);
        Ok(())
    }

    /// Get total duration of playlist (if all durations are known)
    pub fn total_duration(&self) -> Option<f64> {
        let mut total = 0.0;
        for item in &self.items {
            match item.duration {
                Some(d) => total += d,
                None => return None,
            }
        }
        Some(total)
    }

    /// Update item metadata at index
    pub fn update_item_metadata(
        &mut self,
        index: usize,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        duration: Option<f64>,
    ) {
        if let Some(item) = self.items.get_mut(index) {
            item.set_metadata(title, artist, album, duration);
        }
    }
}

impl Default for PlaylistManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Playlist file format support
pub struct PlaylistFormat;

impl PlaylistFormat {
    /// Parse M3U playlist
    pub fn parse_m3u(content: &str) -> Vec<String> {
        content
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect()
    }

    /// Parse PLS playlist
    pub fn parse_pls(content: &str) -> Vec<String> {
        content
            .lines()
            .filter_map(|line| {
                if line.to_lowercase().starts_with("file") {
                    line.split('=').nth(1).map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Export to M3U format
    pub fn export_m3u(items: &[PlaylistItem]) -> String {
        let mut output = String::from("#EXTM3U\n");
        for item in items {
            if let Some(duration) = item.duration {
                output.push_str(&format!(
                    "#EXTINF:{},{}\n",
                    duration as i64,
                    item.title
                ));
            }
            output.push_str(&item.uri);
            output.push('\n');
        }
        output
    }

    /// Detect playlist format from extension
    pub fn detect_format(path: &str) -> Option<&'static str> {
        let lower = path.to_lowercase();
        if lower.ends_with(".m3u") || lower.ends_with(".m3u8") {
            Some("m3u")
        } else if lower.ends_with(".pls") {
            Some("pls")
        } else if lower.ends_with(".json") {
            Some("json")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_add_remove() {
        let mut playlist = PlaylistManager::new();
        playlist.add_item("file:///test1.mp4");
        playlist.add_item("file:///test2.mp4");

        assert_eq!(playlist.len(), 2);
        assert_eq!(playlist.current_index(), Some(0));

        playlist.remove_item(0);
        assert_eq!(playlist.len(), 1);
        assert_eq!(playlist.current_index(), Some(0));
    }

    #[test]
    fn test_playlist_navigation() {
        let mut playlist = PlaylistManager::new();
        playlist.add_item("file:///test1.mp4");
        playlist.add_item("file:///test2.mp4");
        playlist.add_item("file:///test3.mp4");

        assert!(playlist.next().is_some());
        assert_eq!(playlist.current_index(), Some(1));

        assert!(playlist.next().is_some());
        assert_eq!(playlist.current_index(), Some(2));

        assert!(playlist.previous().is_some());
        assert_eq!(playlist.current_index(), Some(1));
    }

    #[test]
    fn test_repeat_mode() {
        let mut playlist = PlaylistManager::new();
        playlist.add_item("file:///test1.mp4");
        playlist.add_item("file:///test2.mp4");

        // Go to last item
        playlist.jump_to(1);

        // Without repeat, next should return None
        playlist.set_repeat_mode(RepeatMode::None);
        assert!(playlist.next().is_none());

        // With repeat all, next should return first item
        playlist.jump_to(1);
        playlist.set_repeat_mode(RepeatMode::All);
        assert!(playlist.next().is_some());
        assert_eq!(playlist.current_index(), Some(0));
    }

    #[test]
    fn test_m3u_parsing() {
        let content = "#EXTM3U\n#EXTINF:120,Song Title\nfile:///music/song.mp3\nfile:///music/song2.mp3";
        let items = PlaylistFormat::parse_m3u(content);
        assert_eq!(items.len(), 2);
    }
}
