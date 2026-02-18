//! Subtitles Manager - Subtitle loading and parsing
//!
//! Supports multiple subtitle formats: SRT, ASS/SSA, VTT (WebVTT)

use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

/// Subtitle entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleEntry {
    /// Start time in seconds
    pub start: f64,
    /// End time in seconds
    pub end: f64,
    /// Subtitle text (may contain formatting)
    pub text: String,
    /// Optional style name (for ASS)
    pub style: Option<String>,
}

/// Subtitle track
#[derive(Debug, Clone)]
pub struct SubtitleTrack {
    /// Track name/language
    pub name: String,
    /// Format (srt, ass, vtt)
    pub format: SubtitleFormat,
    /// Subtitle entries
    pub entries: Vec<SubtitleEntry>,
    /// Global offset in seconds (for synchronization)
    pub offset: f64,
}

/// Supported subtitle formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleFormat {
    Srt,
    Ass,
    Vtt,
}

impl SubtitleFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &str) -> Option<Self> {
        let lower = path.to_lowercase();
        if lower.ends_with(".srt") {
            Some(Self::Srt)
        } else if lower.ends_with(".ass") || lower.ends_with(".ssa") {
            Some(Self::Ass)
        } else if lower.ends_with(".vtt") {
            Some(Self::Vtt)
        } else {
            None
        }
    }
}

/// Subtitle manager
pub struct SubtitleManager {
    /// Loaded subtitle tracks
    tracks: Vec<SubtitleTrack>,
    /// Currently active track index
    active_track: Option<usize>,
    /// Current entries being displayed
    current_entries: Vec<SubtitleEntry>,
    /// Visibility
    visible: bool,
    /// Font size
    font_size: u32,
    /// Font color (ARGB)
    font_color: u32,
    /// Background color (ARGB)
    background_color: u32,
    /// Position (0.0 = bottom, 1.0 = top)
    position: f64,
}

impl SubtitleManager {
    /// Create a new subtitle manager
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            active_track: None,
            current_entries: Vec::new(),
            visible: true,
            font_size: 24,
            font_color: 0xFFFFFFFF, // White
            background_color: 0x80000000, // Semi-transparent black
            position: 0.1, // Near bottom
        }
    }

    /// Load subtitles from file
    pub fn load_file(&mut self, path: &str) -> Result<usize> {
        let format = SubtitleFormat::from_extension(path)
            .ok_or_else(|| anyhow!("Unknown subtitle format for: {}", path))?;

        let content = fs::read_to_string(path)?;
        let track = self.parse(&content, format)?;

        let index = self.tracks.len();

        // Extract name from filename
        let name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        self.tracks.push(SubtitleTrack {
            name,
            format,
            entries: track,
            offset: 0.0,
        });

        // Auto-select if first track
        if self.active_track.is_none() {
            self.active_track = Some(index);
        }

        info!("Loaded subtitle track: {} ({} entries)", path, self.tracks[index].entries.len());
        Ok(index)
    }

    /// Parse subtitle content
    fn parse(&self, content: &str, format: SubtitleFormat) -> Result<Vec<SubtitleEntry>> {
        match format {
            SubtitleFormat::Srt => self.parse_srt(content),
            SubtitleFormat::Ass => self.parse_ass(content),
            SubtitleFormat::Vtt => self.parse_vtt(content),
        }
    }

    /// Parse SRT format
    fn parse_srt(&self, content: &str) -> Result<Vec<SubtitleEntry>> {
        let mut entries = Vec::new();
        let time_regex = Regex::new(r"(\d{2}):(\d{2}):(\d{2}),(\d{3})\s*-->\s*(\d{2}):(\d{2}):(\d{2}),(\d{3})")?;

        let blocks: Vec<&str> = content.split("\n\n").collect();

        for block in blocks {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 3 {
                continue;
            }

            // Find time line
            let time_line = lines.iter().find(|l| time_regex.is_match(l));

            if let Some(time_line) = time_line {
                if let Some(caps) = time_regex.captures(time_line) {
                    let start = parse_srt_time(&caps, 1);
                    let end = parse_srt_time(&caps, 5);

                    // Get text lines (everything after time line)
                    let time_idx = lines.iter().position(|l| *l == *time_line).unwrap_or(0);
                    let text: String = lines[time_idx + 1..]
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| strip_html_tags(l))
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !text.is_empty() {
                        entries.push(SubtitleEntry {
                            start,
                            end,
                            text,
                            style: None,
                        });
                    }
                }
            }
        }

        Ok(entries)
    }

    /// Parse ASS/SSA format
    fn parse_ass(&self, content: &str) -> Result<Vec<SubtitleEntry>> {
        let mut entries = Vec::new();
        let mut in_events = false;
        let mut format_indices: Option<(usize, usize, usize, usize)> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.eq_ignore_ascii_case("[Events]") {
                in_events = true;
                continue;
            }

            if line.starts_with('[') {
                in_events = false;
                continue;
            }

            if in_events {
                if line.to_lowercase().starts_with("format:") {
                    // Parse format line to get column indices
                    let parts: Vec<&str> = line[7..].split(',').map(|s| s.trim()).collect();
                    let start_idx = parts.iter().position(|&s| s.eq_ignore_ascii_case("Start"));
                    let end_idx = parts.iter().position(|&s| s.eq_ignore_ascii_case("End"));
                    let style_idx = parts.iter().position(|&s| s.eq_ignore_ascii_case("Style"));
                    let text_idx = parts.iter().position(|&s| s.eq_ignore_ascii_case("Text"));

                    if let (Some(s), Some(e), Some(st), Some(t)) = (start_idx, end_idx, style_idx, text_idx) {
                        format_indices = Some((s, e, st, t));
                    }
                } else if line.to_lowercase().starts_with("dialogue:") {
                    if let Some((start_idx, end_idx, style_idx, text_idx)) = format_indices {
                        let parts: Vec<&str> = line[9..].splitn(10, ',').collect();

                        if parts.len() > text_idx.max(start_idx).max(end_idx) {
                            let start = parse_ass_time(parts.get(start_idx).unwrap_or(&"0:00:00.00"));
                            let end = parse_ass_time(parts.get(end_idx).unwrap_or(&"0:00:00.00"));
                            let style = parts.get(style_idx).map(|s| s.to_string());
                            let text = strip_ass_tags(parts.get(text_idx).unwrap_or(&""));

                            if !text.is_empty() {
                                entries.push(SubtitleEntry {
                                    start,
                                    end,
                                    text,
                                    style,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort by start time
        entries.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));

        Ok(entries)
    }

    /// Parse WebVTT format
    fn parse_vtt(&self, content: &str) -> Result<Vec<SubtitleEntry>> {
        let mut entries = Vec::new();
        let time_regex = Regex::new(r"(\d{2}):(\d{2}):(\d{2})\.(\d{3})\s*-->\s*(\d{2}):(\d{2}):(\d{2})\.(\d{3})")?;
        let time_regex_short = Regex::new(r"(\d{2}):(\d{2})\.(\d{3})\s*-->\s*(\d{2}):(\d{2})\.(\d{3})")?;

        let blocks: Vec<&str> = content.split("\n\n").collect();

        for block in blocks {
            if block.starts_with("WEBVTT") || block.starts_with("NOTE") {
                continue;
            }

            let lines: Vec<&str> = block.lines().collect();
            if lines.is_empty() {
                continue;
            }

            // Find time line
            let mut start = 0.0;
            let mut end = 0.0;
            let mut time_found = false;

            for (i, line) in lines.iter().enumerate() {
                if let Some(caps) = time_regex.captures(line) {
                    start = parse_vtt_time(&caps, 1);
                    end = parse_vtt_time(&caps, 5);
                    time_found = true;

                    // Get text
                    let text: String = lines[i + 1..]
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| strip_vtt_tags(l))
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !text.is_empty() {
                        entries.push(SubtitleEntry {
                            start,
                            end,
                            text,
                            style: None,
                        });
                    }
                    break;
                } else if let Some(caps) = time_regex_short.captures(line) {
                    // Short format (no hours)
                    start = parse_vtt_time_short(&caps, 1);
                    end = parse_vtt_time_short(&caps, 4);
                    time_found = true;

                    let text: String = lines[i + 1..]
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| strip_vtt_tags(l))
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !text.is_empty() {
                        entries.push(SubtitleEntry {
                            start,
                            end,
                            text,
                            style: None,
                        });
                    }
                    break;
                }
            }
        }

        Ok(entries)
    }

    /// Get active entries for a given time position
    pub fn get_entries_at(&mut self, time: f64) -> &[SubtitleEntry] {
        self.current_entries.clear();

        if let Some(track_idx) = self.active_track {
            if let Some(track) = self.tracks.get(track_idx) {
                let adjusted_time = time - track.offset;

                for entry in &track.entries {
                    if adjusted_time >= entry.start && adjusted_time <= entry.end {
                        self.current_entries.push(entry.clone());
                    }
                }
            }
        }

        &self.current_entries
    }

    /// Get formatted text at time
    pub fn get_text_at(&mut self, time: f64) -> Option<String> {
        let entries = self.get_entries_at(time);
        if entries.is_empty() {
            None
        } else {
            Some(entries.iter().map(|e| e.text.as_str()).collect::<Vec<_>>().join("\n"))
        }
    }

    /// Set active track by index
    pub fn set_active_track(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.tracks.len() {
                self.active_track = Some(idx);
                info!("Active subtitle track: {}", self.tracks[idx].name);
            }
        } else {
            self.active_track = None;
            info!("Subtitles disabled");
        }
    }

    /// Get available tracks
    pub fn tracks(&self) -> &[SubtitleTrack] {
        &self.tracks
    }

    /// Get track names
    pub fn track_names(&self) -> Vec<String> {
        self.tracks.iter().map(|t| t.name.clone()).collect()
    }

    /// Set time offset for synchronization
    pub fn set_offset(&mut self, offset: f64) {
        if let Some(idx) = self.active_track {
            if let Some(track) = self.tracks.get_mut(idx) {
                track.offset = offset;
                info!("Subtitle offset set to: {:.2}s", offset);
            }
        }
    }

    /// Adjust offset by delta
    pub fn adjust_offset(&mut self, delta: f64) {
        if let Some(idx) = self.active_track {
            if let Some(track) = self.tracks.get_mut(idx) {
                track.offset += delta;
                info!("Subtitle offset adjusted to: {:.2}s", track.offset);
            }
        }
    }

    /// Get current offset
    pub fn offset(&self) -> f64 {
        self.active_track
            .and_then(|idx| self.tracks.get(idx))
            .map(|t| t.offset)
            .unwrap_or(0.0)
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check visibility
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set font size
    pub fn set_font_size(&mut self, size: u32) {
        self.font_size = size.clamp(8, 72);
    }

    /// Get font size
    pub fn font_size(&self) -> u32 {
        self.font_size
    }

    /// Set font color
    pub fn set_font_color(&mut self, color: u32) {
        self.font_color = color;
    }

    /// Set background color
    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    /// Set position (0.0 = bottom, 1.0 = top)
    pub fn set_position(&mut self, position: f64) {
        self.position = position.clamp(0.0, 1.0);
    }

    /// Clear all tracks
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.active_track = None;
        self.current_entries.clear();
    }

    /// Remove track by index
    pub fn remove_track(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.tracks.remove(index);
            if let Some(active) = self.active_track {
                if active == index {
                    self.active_track = if self.tracks.is_empty() {
                        None
                    } else {
                        Some(0)
                    };
                } else if active > index {
                    self.active_track = Some(active - 1);
                }
            }
        }
    }
}

impl Default for SubtitleManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn parse_srt_time(caps: &regex::Captures, start: usize) -> f64 {
    let hours: f64 = caps.get(start).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let minutes: f64 = caps.get(start + 1).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let seconds: f64 = caps.get(start + 2).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let millis: f64 = caps.get(start + 3).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);

    hours * 3600.0 + minutes * 60.0 + seconds + millis / 1000.0
}

fn parse_vtt_time(caps: &regex::Captures, start: usize) -> f64 {
    let hours: f64 = caps.get(start).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let minutes: f64 = caps.get(start + 1).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let seconds: f64 = caps.get(start + 2).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let millis: f64 = caps.get(start + 3).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);

    hours * 3600.0 + minutes * 60.0 + seconds + millis / 1000.0
}

fn parse_vtt_time_short(caps: &regex::Captures, start: usize) -> f64 {
    let minutes: f64 = caps.get(start).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let seconds: f64 = caps.get(start + 1).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);
    let millis: f64 = caps.get(start + 2).map(|m| m.as_str().parse().unwrap_or(0.0)).unwrap_or(0.0);

    minutes * 60.0 + seconds + millis / 1000.0
}

fn parse_ass_time(time_str: &str) -> f64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let hours: f64 = parts[0].parse().unwrap_or(0.0);
        let minutes: f64 = parts[1].parse().unwrap_or(0.0);
        let seconds: f64 = parts[2].parse().unwrap_or(0.0);
        hours * 3600.0 + minutes * 60.0 + seconds
    } else {
        0.0
    }
}

fn strip_html_tags(text: &str) -> String {
    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    tag_regex.replace_all(text, "").to_string()
}

fn strip_ass_tags(text: &str) -> String {
    // Remove ASS override tags like {\i1}, {\b1}, {\pos(x,y)}, etc.
    let tag_regex = Regex::new(r"\{[^}]*\}").unwrap();
    let result = tag_regex.replace_all(text, "");
    // Replace \N with newline
    result.replace("\\N", "\n").replace("\\n", "\n")
}

fn strip_vtt_tags(text: &str) -> String {
    // Remove VTT tags like <v.name>, <c.classname>, etc.
    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    tag_regex.replace_all(text, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_srt() {
        let srt_content = r#"1
00:00:01,000 --> 00:00:04,000
Hello, world!

2
00:00:05,000 --> 00:00:08,000
This is a test.
"#;
        let manager = SubtitleManager::new();
        let entries = manager.parse_srt(srt_content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hello, world!");
        assert!((entries[0].start - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_strip_ass_tags() {
        let text = r"{\i1}Italic{\i0} and {\b1}bold{\b0}\NNew line";
        let stripped = strip_ass_tags(text);
        assert_eq!(stripped, "Italic and bold\nNew line");
    }
}
