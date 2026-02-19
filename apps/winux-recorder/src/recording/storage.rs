//! Recording storage and management

use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::audio::AudioFormat;

/// Recording metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// Unique ID
    pub id: uuid::Uuid,
    /// Recording title/name
    pub title: String,
    /// File path
    pub path: PathBuf,
    /// Duration in seconds
    pub duration: f64,
    /// Audio format
    pub format: AudioFormat,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// File size in bytes
    pub file_size: u64,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Markers/bookmarks
    pub markers: Vec<f64>,
    /// User notes
    pub notes: Option<String>,
    /// Whether this is a favorite
    pub is_favorite: bool,
}

impl RecordingMetadata {
    /// Create new metadata from file
    pub fn from_file(path: &Path) -> Option<Self> {
        let filename = path.file_stem()?.to_str()?;
        let extension = path.extension()?.to_str()?;

        let format = match extension.to_lowercase().as_str() {
            "wav" => AudioFormat::Wav,
            "mp3" => AudioFormat::Mp3,
            "ogg" | "opus" => AudioFormat::Ogg,
            "flac" => AudioFormat::Flac,
            _ => return None,
        };

        let metadata = std::fs::metadata(path).ok()?;
        let file_size = metadata.len();

        // Try to get audio duration
        let (duration, sample_rate, channels) = get_audio_info(path).unwrap_or((0.0, 48000, 1));

        let created_at = metadata
            .created()
            .ok()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(chrono::Utc::now);

        let modified_at = metadata
            .modified()
            .ok()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(chrono::Utc::now);

        Some(Self {
            id: uuid::Uuid::new_v4(),
            title: filename.to_string(),
            path: path.to_path_buf(),
            duration,
            format,
            sample_rate,
            channels,
            file_size,
            created_at,
            modified_at,
            markers: Vec::new(),
            notes: None,
            is_favorite: false,
        })
    }

    /// Format duration as string
    pub fn duration_string(&self) -> String {
        let total_secs = self.duration as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Format file size as string
    pub fn size_string(&self) -> String {
        if self.file_size < 1024 {
            format!("{} B", self.file_size)
        } else if self.file_size < 1024 * 1024 {
            format!("{:.1} KB", self.file_size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.file_size as f64 / (1024.0 * 1024.0))
        }
    }
}

/// A recording with metadata and optional cached waveform
#[derive(Debug, Clone)]
pub struct Recording {
    /// Metadata
    pub metadata: RecordingMetadata,
    /// Cached waveform for display (downsampled)
    pub waveform: Option<Vec<f32>>,
}

impl Recording {
    /// Create from metadata
    pub fn new(metadata: RecordingMetadata) -> Self {
        Self {
            metadata,
            waveform: None,
        }
    }

    /// Create from file path
    pub fn from_file(path: &Path) -> Option<Self> {
        let metadata = RecordingMetadata::from_file(path)?;
        Some(Self::new(metadata))
    }

    /// Load waveform for display
    pub fn load_waveform(&mut self) -> Result<(), anyhow::Error> {
        let samples = crate::audio::encoder::decode_file(&self.metadata.path)?;
        self.waveform = Some(samples.downsample(200));
        Ok(())
    }

    /// Get file path
    pub fn path(&self) -> &Path {
        &self.metadata.path
    }

    /// Get title
    pub fn title(&self) -> &str {
        &self.metadata.title
    }

    /// Rename the recording
    pub fn rename(&mut self, new_title: &str) -> Result<(), anyhow::Error> {
        let old_path = &self.metadata.path;
        let parent = old_path.parent().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        let extension = old_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");

        let new_filename = format!("{}.{}", new_title, extension);
        let new_path = parent.join(&new_filename);

        std::fs::rename(old_path, &new_path)?;

        self.metadata.title = new_title.to_string();
        self.metadata.path = new_path;
        self.metadata.modified_at = chrono::Utc::now();

        Ok(())
    }

    /// Delete the recording
    pub fn delete(&self) -> Result<(), anyhow::Error> {
        // Try to move to trash first
        if trash::delete(&self.metadata.path).is_err() {
            // Fall back to permanent deletion
            std::fs::remove_file(&self.metadata.path)?;
        }
        Ok(())
    }

    /// Export/copy to another location
    pub fn export(&self, destination: &Path) -> Result<PathBuf, anyhow::Error> {
        let filename = self.metadata.path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
        let dest_path = destination.join(filename);

        std::fs::copy(&self.metadata.path, &dest_path)?;

        Ok(dest_path)
    }
}

/// Get audio file info (duration, sample_rate, channels)
fn get_audio_info(path: &Path) -> Result<(f64, u32, u16), anyhow::Error> {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if extension == "wav" {
        let reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        let num_samples = reader.len() as f64;
        let duration = num_samples / (spec.sample_rate as f64 * spec.channels as f64);
        return Ok((duration, spec.sample_rate, spec.channels));
    }

    // For other formats, try ffprobe
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path.to_str().unwrap(),
        ])
        .output()?;

    if output.status.success() {
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        let duration = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let streams = json["streams"].as_array();
        let (sample_rate, channels) = streams
            .and_then(|s| s.first())
            .map(|stream| {
                let sr = stream["sample_rate"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(48000);
                let ch = stream["channels"]
                    .as_u64()
                    .unwrap_or(1) as u16;
                (sr, ch)
            })
            .unwrap_or((48000, 1));

        return Ok((duration, sample_rate, channels));
    }

    // Default fallback
    Ok((0.0, 48000, 1))
}

/// Load all recordings from the output directory
pub fn load_recordings(state: &Arc<RwLock<AppState>>) {
    let output_dir = state.read().config.output_dir.clone();

    if !output_dir.exists() {
        return;
    }

    let mut recordings = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&output_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(recording) = Recording::from_file(&path) {
                    recordings.push(recording);
                }
            }
        }
    }

    // Sort by creation date (newest first)
    recordings.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));

    state.write().recordings = recordings;
}

/// Save metadata database
pub fn save_metadata_db(state: &Arc<RwLock<AppState>>) -> Result<(), anyhow::Error> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("No config directory"))?
        .join("winux-recorder");

    std::fs::create_dir_all(&config_dir)?;

    let db_path = config_dir.join("recordings.db");
    let conn = rusqlite::Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS recordings (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL,
            title TEXT NOT NULL,
            duration REAL,
            format TEXT,
            sample_rate INTEGER,
            channels INTEGER,
            file_size INTEGER,
            created_at TEXT,
            markers TEXT,
            notes TEXT,
            is_favorite INTEGER DEFAULT 0
        )",
        [],
    )?;

    let recordings = state.read().recordings.clone();

    for recording in &recordings {
        let meta = &recording.metadata;
        let markers_json = serde_json::to_string(&meta.markers)?;

        conn.execute(
            "INSERT OR REPLACE INTO recordings
             (id, path, title, duration, format, sample_rate, channels, file_size, created_at, markers, notes, is_favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                meta.id.to_string(),
                meta.path.to_string_lossy(),
                meta.title,
                meta.duration,
                format!("{:?}", meta.format),
                meta.sample_rate,
                meta.channels,
                meta.file_size as i64,
                meta.created_at.to_rfc3339(),
                markers_json,
                meta.notes,
                meta.is_favorite as i32,
            ],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_size_string() {
        let meta = RecordingMetadata {
            id: uuid::Uuid::new_v4(),
            title: "Test".to_string(),
            path: PathBuf::from("test.wav"),
            duration: 60.0,
            format: AudioFormat::Wav,
            sample_rate: 48000,
            channels: 1,
            file_size: 1024 * 1024 * 5, // 5 MB
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            markers: vec![],
            notes: None,
            is_favorite: false,
        };

        assert_eq!(meta.size_string(), "5.0 MB");
        assert_eq!(meta.duration_string(), "01:00");
    }
}
