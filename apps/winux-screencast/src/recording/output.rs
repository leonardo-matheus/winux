//! Output file handling
//!
//! Manages output file operations including saving, naming,
//! and post-processing of recorded videos.

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;

use crate::OutputFormat;

/// Output file metadata
#[derive(Debug, Clone)]
pub struct OutputMetadata {
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Duration in seconds
    pub duration: f64,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Framerate
    pub framerate: f64,
    /// Video codec
    pub video_codec: String,
    /// Audio codec (if any)
    pub audio_codec: Option<String>,
    /// Container format
    pub format: OutputFormat,
    /// Creation timestamp
    pub created: chrono::DateTime<chrono::Local>,
}

/// Output manager handles file operations
pub struct OutputManager {
    /// Default output directory
    output_dir: PathBuf,
}

impl OutputManager {
    /// Create a new output manager
    pub fn new(output_dir: PathBuf) -> Self {
        // Ensure output directory exists
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir).ok();
        }

        Self { output_dir }
    }

    /// Get the output directory
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Set the output directory
    pub fn set_output_dir(&mut self, dir: PathBuf) -> Result<()> {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        self.output_dir = dir;
        Ok(())
    }

    /// Generate a unique filename for a new recording
    pub fn generate_filename(&self, format: OutputFormat) -> PathBuf {
        let now = chrono::Local::now();
        let filename = format!(
            "Screencast_{}.{}",
            now.format("%Y-%m-%d_%H-%M-%S"),
            format.extension()
        );

        let mut path = self.output_dir.join(&filename);

        // Ensure unique filename
        let mut counter = 1;
        while path.exists() {
            let filename = format!(
                "Screencast_{}_{}.{}",
                now.format("%Y-%m-%d_%H-%M-%S"),
                counter,
                format.extension()
            );
            path = self.output_dir.join(&filename);
            counter += 1;
        }

        path
    }

    /// Get metadata for an existing recording
    pub fn get_metadata(&self, path: &Path) -> Result<OutputMetadata> {
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", path.display()));
        }

        let metadata = fs::metadata(path)?;
        let size = metadata.len();

        // Determine format from extension
        let format = match path.extension().and_then(|e| e.to_str()) {
            Some("mp4") => OutputFormat::MP4,
            Some("webm") => OutputFormat::WebM,
            Some("mkv") => OutputFormat::MKV,
            Some("gif") => OutputFormat::GIF,
            _ => OutputFormat::MP4,
        };

        // For full metadata, we would use GStreamer to probe the file
        // For now, return basic info
        Ok(OutputMetadata {
            path: path.to_path_buf(),
            size,
            duration: 0.0, // Would need to probe file
            width: 0,
            height: 0,
            framerate: 0.0,
            video_codec: String::new(),
            audio_codec: None,
            format,
            created: chrono::Local::now(),
        })
    }

    /// Probe a video file for detailed metadata using GStreamer
    pub async fn probe_file(&self, path: &Path) -> Result<OutputMetadata> {
        use gstreamer as gst;
        use gstreamer_pbutils as gst_pbutils;
        use gst_pbutils::prelude::*;

        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", path.display()));
        }

        let uri = format!("file://{}", path.display());
        let discoverer = gst_pbutils::Discoverer::new(gst::ClockTime::from_seconds(10))?;
        let info = discoverer.discover_uri(&uri)?;

        let duration = info.duration()
            .map(|d| d.nseconds() as f64 / 1_000_000_000.0)
            .unwrap_or(0.0);

        // Get video stream info
        let mut width = 0u32;
        let mut height = 0u32;
        let mut framerate = 0.0f64;
        let mut video_codec = String::new();
        let mut audio_codec = None;

        for stream in info.video_streams() {
            width = stream.width();
            height = stream.height();
            framerate = stream.framerate().numer() as f64 / stream.framerate().denom() as f64;

            if let Some(caps) = stream.caps() {
                if let Some(structure) = caps.structure(0) {
                    video_codec = structure.name().to_string();
                }
            }
        }

        for stream in info.audio_streams() {
            if let Some(caps) = stream.caps() {
                if let Some(structure) = caps.structure(0) {
                    audio_codec = Some(structure.name().to_string());
                }
            }
        }

        let metadata = fs::metadata(path)?;
        let format = match path.extension().and_then(|e| e.to_str()) {
            Some("mp4") => OutputFormat::MP4,
            Some("webm") => OutputFormat::WebM,
            Some("mkv") => OutputFormat::MKV,
            Some("gif") => OutputFormat::GIF,
            _ => OutputFormat::MP4,
        };

        Ok(OutputMetadata {
            path: path.to_path_buf(),
            size: metadata.len(),
            duration,
            width,
            height,
            framerate,
            video_codec,
            audio_codec,
            format,
            created: chrono::Local::now(),
        })
    }

    /// List all recordings in the output directory
    pub fn list_recordings(&self) -> Result<Vec<PathBuf>> {
        let mut recordings = Vec::new();

        for entry in fs::read_dir(&self.output_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "mp4" | "webm" | "mkv" | "gif") {
                        recordings.push(path);
                    }
                }
            }
        }

        // Sort by modification time, newest first
        recordings.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        Ok(recordings)
    }

    /// Delete a recording
    pub fn delete_recording(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        // Verify it's in our output directory for safety
        if !path.starts_with(&self.output_dir) {
            return Err(anyhow!("Cannot delete file outside output directory"));
        }

        fs::remove_file(path)?;
        Ok(())
    }

    /// Move/rename a recording
    pub fn rename_recording(&self, old_path: &Path, new_name: &str) -> Result<PathBuf> {
        if !old_path.exists() {
            return Err(anyhow!("Source file does not exist"));
        }

        let extension = old_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");

        let new_filename = if new_name.contains('.') {
            new_name.to_string()
        } else {
            format!("{}.{}", new_name, extension)
        };

        let new_path = self.output_dir.join(&new_filename);

        if new_path.exists() {
            return Err(anyhow!("A file with that name already exists"));
        }

        fs::rename(old_path, &new_path)?;
        Ok(new_path)
    }

    /// Copy a recording to a new location
    pub fn copy_recording(&self, source: &Path, destination: &Path) -> Result<()> {
        if !source.exists() {
            return Err(anyhow!("Source file does not exist"));
        }

        fs::copy(source, destination)?;
        Ok(())
    }

    /// Get total size of all recordings
    pub fn total_size(&self) -> Result<u64> {
        let mut total = 0u64;

        for path in self.list_recordings()? {
            if let Ok(metadata) = fs::metadata(&path) {
                total += metadata.len();
            }
        }

        Ok(total)
    }

    /// Open a recording with the default video player
    pub fn open_recording(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("File does not exist"));
        }

        open::that(path)?;
        Ok(())
    }

    /// Open the containing folder for a recording
    pub fn show_in_folder(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            open::that(parent)?;
        }
        Ok(())
    }
}

/// Trim a video file using GStreamer
pub async fn trim_video(
    input: &Path,
    output: &Path,
    start_time: f64,
    end_time: f64,
) -> Result<()> {
    use gstreamer as gst;
    use gst::prelude::*;

    let start_ns = (start_time * 1_000_000_000.0) as u64;
    let end_ns = (end_time * 1_000_000_000.0) as u64;

    let pipeline_str = format!(
        "filesrc location={} ! decodebin name=dec \
         dec. ! videoconvert ! x264enc ! queue ! mux. \
         dec. ! audioconvert ! audioresample ! fdkaacenc ! queue ! mux. \
         mp4mux name=mux ! filesink location={}",
        input.display(),
        output.display()
    );

    let pipeline = gst::parse::launch(&pipeline_str)?;
    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>()
        .map_err(|_| anyhow!("Not a pipeline"))?;

    // Set up seeking
    pipeline.set_state(gst::State::Paused)?;

    // Wait for state change
    let _ = pipeline.state(gst::ClockTime::from_seconds(5));

    // Seek to start position with stop at end
    pipeline.seek(
        1.0,
        gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
        gst::SeekType::Set,
        gst::ClockTime::from_nseconds(start_ns),
        gst::SeekType::Set,
        gst::ClockTime::from_nseconds(end_ns),
    )?;

    // Start playback
    pipeline.set_state(gst::State::Playing)?;

    // Wait for EOS
    let bus = pipeline.bus().ok_or_else(|| anyhow!("No bus"))?;
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                return Err(anyhow!("Trim error: {}", err.error()));
            }
            _ => {}
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

/// Convert a video to GIF
pub async fn convert_to_gif(
    input: &Path,
    output: &Path,
    fps: u32,
    width: Option<u32>,
) -> Result<()> {
    use gstreamer as gst;
    use gst::prelude::*;

    let width_filter = width
        .map(|w| format!("videoscale ! video/x-raw,width={} ! ", w))
        .unwrap_or_default();

    let pipeline_str = format!(
        "filesrc location={} ! decodebin ! videoconvert ! \
         videorate ! video/x-raw,framerate={}/1 ! \
         {}gifenc ! filesink location={}",
        input.display(),
        fps.min(15), // GIFs work better with lower FPS
        width_filter,
        output.display()
    );

    let pipeline = gst::parse::launch(&pipeline_str)?;
    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>()
        .map_err(|_| anyhow!("Not a pipeline"))?;

    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline.bus().ok_or_else(|| anyhow!("No bus"))?;
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                return Err(anyhow!("Conversion error: {}", err.error()));
            }
            _ => {}
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
