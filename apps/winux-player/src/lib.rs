//! Winux Player Library
//!
//! Native video player for Winux OS built with GTK4/Adwaita and GStreamer.
//!
//! # Features
//!
//! - Hardware-accelerated video playback with GStreamer
//! - Support for multiple video formats (MP4, MKV, AVI, WebM, MOV, FLV, WMV, OGG)
//! - Support for audio formats (MP3, FLAC, WAV, AAC, OGG)
//! - Playlist management with shuffle and repeat modes
//! - Subtitle support (SRT, ASS/SSA, VTT)
//! - Speed control (0.25x - 2x)
//! - A-B repeat functionality
//! - Picture-in-Picture mode
//! - Keyboard shortcuts
//! - Configurable settings
//!
//! # Modules
//!
//! - [`config`] - Application configuration and settings
//! - [`controls`] - Video playback controls UI
//! - [`player`] - GStreamer player widget
//! - [`playlist`] - Playlist management
//! - [`subtitles`] - Subtitle loading and rendering
//!
//! # Example
//!
//! ```rust,no_run
//! use winux_player::{PlayerWidget, Config};
//!
//! // Create player
//! let player = PlayerWidget::new();
//!
//! // Load a video
//! player.load_uri("file:///path/to/video.mp4");
//!
//! // Play
//! player.play();
//!
//! // Seek to 30 seconds
//! player.seek_absolute(30.0);
//!
//! // Set speed to 1.5x
//! player.set_speed(1.5);
//! ```

pub mod config;
pub mod controls;
pub mod player;
pub mod playlist;
pub mod subtitles;

// Re-exports for convenience
pub use config::{Config, PlaybackConfig, SubtitleConfig, VideoConfig, WindowConfig};
pub use controls::{ProgressPreview, VideoControls};
pub use player::{format_duration, parse_duration, PlayerState, PlayerWidget};
pub use playlist::{PlaylistFormat, PlaylistItem, PlaylistManager, RepeatMode};
pub use subtitles::{SubtitleEntry, SubtitleFormat, SubtitleManager, SubtitleTrack};

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application ID
pub const APP_ID: &str = "org.winux.player";

/// Application name
pub const APP_NAME: &str = "Winux Player";

/// Supported video file extensions
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "webm", "mov", "flv", "wmv", "ogv", "m4v", "3gp", "mpeg", "mpg", "ts",
    "m2ts", "vob",
];

/// Supported audio file extensions
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "aac", "ogg", "m4a", "wma", "opus", "ape", "alac",
];

/// Supported subtitle file extensions
pub const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "ass", "ssa", "vtt", "sub", "idx"];

/// Supported playlist file extensions
pub const PLAYLIST_EXTENSIONS: &[&str] = &["m3u", "m3u8", "pls"];

/// Check if a file extension is a supported media file
pub fn is_supported_media(extension: &str) -> bool {
    let ext_lower = extension.to_lowercase();
    VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) || AUDIO_EXTENSIONS.contains(&ext_lower.as_str())
}

/// Check if a file extension is a supported video file
pub fn is_supported_video(extension: &str) -> bool {
    VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

/// Check if a file extension is a supported audio file
pub fn is_supported_audio(extension: &str) -> bool {
    AUDIO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

/// Check if a file extension is a supported subtitle file
pub fn is_supported_subtitle(extension: &str) -> bool {
    SUBTITLE_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

/// Check if a file extension is a supported playlist file
pub fn is_supported_playlist(extension: &str) -> bool {
    PLAYLIST_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

/// Get file type description
pub fn get_file_type_description(extension: &str) -> &'static str {
    let ext_lower = extension.to_lowercase();

    match ext_lower.as_str() {
        // Video formats
        "mp4" => "MPEG-4 Video",
        "mkv" => "Matroska Video",
        "avi" => "AVI Video",
        "webm" => "WebM Video",
        "mov" => "QuickTime Video",
        "flv" => "Flash Video",
        "wmv" => "Windows Media Video",
        "ogv" => "Ogg Video",
        "m4v" => "MPEG-4 Video",
        "3gp" => "3GP Video",
        "mpeg" | "mpg" => "MPEG Video",
        "ts" | "m2ts" => "MPEG Transport Stream",
        "vob" => "DVD Video Object",

        // Audio formats
        "mp3" => "MP3 Audio",
        "flac" => "FLAC Audio",
        "wav" => "WAV Audio",
        "aac" => "AAC Audio",
        "ogg" => "Ogg Vorbis Audio",
        "m4a" => "MPEG-4 Audio",
        "wma" => "Windows Media Audio",
        "opus" => "Opus Audio",
        "ape" => "Monkey's Audio",
        "alac" => "Apple Lossless Audio",

        // Subtitle formats
        "srt" => "SubRip Subtitles",
        "ass" | "ssa" => "Advanced SubStation Alpha",
        "vtt" => "WebVTT Subtitles",
        "sub" => "MicroDVD Subtitles",

        // Playlist formats
        "m3u" | "m3u8" => "M3U Playlist",
        "pls" => "PLS Playlist",

        _ => "Unknown",
    }
}

/// MIME type for media files
pub fn get_mime_type(extension: &str) -> &'static str {
    let ext_lower = extension.to_lowercase();

    match ext_lower.as_str() {
        // Video MIME types
        "mp4" | "m4v" => "video/mp4",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "flv" => "video/x-flv",
        "wmv" => "video/x-ms-wmv",
        "ogv" => "video/ogg",
        "3gp" => "video/3gpp",
        "mpeg" | "mpg" => "video/mpeg",
        "ts" | "m2ts" => "video/mp2t",

        // Audio MIME types
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "wma" => "audio/x-ms-wma",
        "opus" => "audio/opus",

        // Subtitle MIME types
        "srt" => "application/x-subrip",
        "ass" | "ssa" => "text/x-ssa",
        "vtt" => "text/vtt",

        // Playlist MIME types
        "m3u" | "m3u8" => "audio/x-mpegurl",
        "pls" => "audio/x-scpls",

        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_media() {
        assert!(is_supported_media("mp4"));
        assert!(is_supported_media("MP4"));
        assert!(is_supported_media("mkv"));
        assert!(is_supported_media("mp3"));
        assert!(!is_supported_media("txt"));
    }

    #[test]
    fn test_is_supported_video() {
        assert!(is_supported_video("mp4"));
        assert!(is_supported_video("mkv"));
        assert!(!is_supported_video("mp3"));
    }

    #[test]
    fn test_is_supported_audio() {
        assert!(is_supported_audio("mp3"));
        assert!(is_supported_audio("flac"));
        assert!(!is_supported_audio("mp4"));
    }

    #[test]
    fn test_is_supported_subtitle() {
        assert!(is_supported_subtitle("srt"));
        assert!(is_supported_subtitle("ass"));
        assert!(is_supported_subtitle("vtt"));
        assert!(!is_supported_subtitle("txt"));
    }

    #[test]
    fn test_get_file_type_description() {
        assert_eq!(get_file_type_description("mp4"), "MPEG-4 Video");
        assert_eq!(get_file_type_description("mkv"), "Matroska Video");
        assert_eq!(get_file_type_description("mp3"), "MP3 Audio");
    }

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type("mp4"), "video/mp4");
        assert_eq!(get_mime_type("mp3"), "audio/mpeg");
        assert_eq!(get_mime_type("srt"), "application/x-subrip");
    }
}
