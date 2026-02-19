//! Video encoding module using GStreamer
//!
//! Handles video encoding with support for multiple codecs
//! (H.264, H.265, VP9, AV1) and containers (MP4, WebM, MKV, GIF).

use anyhow::{Result, anyhow};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::{VideoCodec, OutputFormat, Resolution, Framerate};

/// Encoder state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderState {
    Idle,
    Encoding,
    Paused,
    Finished,
    Error,
}

/// Encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Video codec
    pub codec: VideoCodec,
    /// Output format/container
    pub format: OutputFormat,
    /// Target resolution
    pub resolution: Resolution,
    /// Target framerate
    pub framerate: Framerate,
    /// Bitrate in kbps (0 = auto)
    pub bitrate: u32,
    /// Enable hardware acceleration
    pub hardware_accel: bool,
    /// Include audio
    pub include_audio: bool,
    /// Audio bitrate in kbps
    pub audio_bitrate: u32,
    /// Output file path
    pub output_path: PathBuf,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            format: OutputFormat::MP4,
            resolution: Resolution::Original,
            framerate: Framerate::Fps30,
            bitrate: 8000, // 8 Mbps
            hardware_accel: true,
            include_audio: true,
            audio_bitrate: 192,
            output_path: PathBuf::new(),
        }
    }
}

/// Video encoder using GStreamer
pub struct VideoEncoder {
    config: EncoderConfig,
    pipeline: Option<gst::Pipeline>,
    state: Arc<Mutex<EncoderState>>,
    /// Encoded bytes so far
    bytes_written: Arc<Mutex<u64>>,
    /// Duration encoded in nanoseconds
    duration_ns: Arc<Mutex<u64>>,
}

impl VideoEncoder {
    /// Create a new video encoder
    pub fn new(config: EncoderConfig) -> Self {
        Self {
            config,
            pipeline: None,
            state: Arc::new(Mutex::new(EncoderState::Idle)),
            bytes_written: Arc::new(Mutex::new(0)),
            duration_ns: Arc::new(Mutex::new(0)),
        }
    }

    /// Get current encoder state
    pub fn state(&self) -> EncoderState {
        *self.state.lock()
    }

    /// Get bytes written so far
    pub fn bytes_written(&self) -> u64 {
        *self.bytes_written.lock()
    }

    /// Get duration encoded in seconds
    pub fn duration_seconds(&self) -> f64 {
        let ns = *self.duration_ns.lock();
        ns as f64 / 1_000_000_000.0
    }

    /// Build the GStreamer pipeline for encoding
    pub fn build_pipeline(&mut self, pipewire_node_id: u32) -> Result<()> {
        let pipeline_str = self.build_pipeline_string(pipewire_node_id);

        let pipeline = gst::parse::launch(&pipeline_str)
            .map_err(|e| anyhow!("Failed to create pipeline: {}", e))?;

        let pipeline = pipeline.dynamic_cast::<gst::Pipeline>()
            .map_err(|_| anyhow!("Element is not a pipeline"))?;

        self.pipeline = Some(pipeline);
        Ok(())
    }

    /// Build the pipeline string based on configuration
    fn build_pipeline_string(&self, pipewire_node_id: u32) -> String {
        let mut parts = Vec::new();

        // Video source from PipeWire
        parts.push(format!(
            "pipewiresrc path={} do-timestamp=true",
            pipewire_node_id
        ));

        // Video conversion and scaling
        parts.push("videoconvert".to_string());

        // Apply resolution scaling if needed
        if let Some((width, height)) = self.config.resolution.dimensions() {
            parts.push(format!(
                "videoscale ! video/x-raw,width={},height={}",
                width, height
            ));
        }

        // Set framerate
        parts.push(format!(
            "videorate ! video/x-raw,framerate={}/1",
            self.config.framerate.value()
        ));

        // Add video encoder
        let encoder = self.get_encoder_element();
        parts.push(encoder);

        // Handle GIF separately (different pipeline)
        if self.config.format == OutputFormat::GIF {
            return self.build_gif_pipeline_string(pipewire_node_id);
        }

        // Queue before muxer
        parts.push("queue".to_string());

        // Create muxer
        let muxer = format!("{} name=mux", self.config.format.gstreamer_muxer());
        parts.push(muxer);

        // Audio pipeline (if enabled)
        if self.config.include_audio && self.config.format != OutputFormat::GIF {
            let audio_pipeline = self.get_audio_pipeline();
            // Audio will be linked to mux.audio_%u
            parts.push(format!("mux. {} ! mux.", audio_pipeline));
        }

        // File sink
        parts.push(format!(
            "mux. ! filesink location={}",
            self.config.output_path.display()
        ));

        parts.join(" ! ")
    }

    /// Build a GIF-specific pipeline
    fn build_gif_pipeline_string(&self, pipewire_node_id: u32) -> String {
        let fps = self.config.framerate.value().min(15); // GIFs work better with lower FPS

        format!(
            "pipewiresrc path={} do-timestamp=true ! \
             videoconvert ! \
             videorate ! video/x-raw,framerate={}/1 ! \
             videoscale ! video/x-raw,width=640 ! \
             gifenc ! \
             filesink location={}",
            pipewire_node_id,
            fps,
            self.config.output_path.display()
        )
    }

    /// Get the appropriate encoder element
    fn get_encoder_element(&self) -> String {
        let bitrate_kbps = self.config.bitrate;

        // Try hardware encoders first if enabled
        if self.config.hardware_accel {
            if let Some(hw_encoder) = self.get_hardware_encoder() {
                return hw_encoder;
            }
        }

        // Fall back to software encoders
        match self.config.codec {
            VideoCodec::H264 => format!(
                "x264enc bitrate={} speed-preset=veryfast tune=zerolatency",
                bitrate_kbps
            ),
            VideoCodec::H265 => format!(
                "x265enc bitrate={} speed-preset=veryfast tune=zerolatency",
                bitrate_kbps
            ),
            VideoCodec::VP9 => format!(
                "vp9enc target-bitrate={} cpu-used=4 deadline=1",
                bitrate_kbps * 1000 // VP9 uses bits, not kbits
            ),
            VideoCodec::AV1 => format!(
                "av1enc target-bitrate={} cpu-used=4",
                bitrate_kbps * 1000
            ),
        }
    }

    /// Try to get a hardware encoder if available
    fn get_hardware_encoder(&self) -> Option<String> {
        let bitrate_kbps = self.config.bitrate;

        // Check for VAAPI (Intel/AMD)
        if gst::ElementFactory::find("vaapih264enc").is_some() {
            match self.config.codec {
                VideoCodec::H264 => return Some(format!(
                    "vaapih264enc bitrate={} rate-control=cbr",
                    bitrate_kbps
                )),
                VideoCodec::H265 => return Some(format!(
                    "vaapih265enc bitrate={} rate-control=cbr",
                    bitrate_kbps
                )),
                VideoCodec::VP9 => return Some(format!(
                    "vaapivp9enc bitrate={} rate-control=cbr",
                    bitrate_kbps
                )),
                _ => {}
            }
        }

        // Check for NVENC (NVIDIA)
        if gst::ElementFactory::find("nvh264enc").is_some() {
            match self.config.codec {
                VideoCodec::H264 => return Some(format!(
                    "nvh264enc bitrate={} preset=low-latency-hq",
                    bitrate_kbps
                )),
                VideoCodec::H265 => return Some(format!(
                    "nvh265enc bitrate={} preset=low-latency-hq",
                    bitrate_kbps
                )),
                _ => {}
            }
        }

        None
    }

    /// Get the audio pipeline string
    fn get_audio_pipeline(&self) -> String {
        format!(
            "pulsesrc ! audioconvert ! audioresample ! \
             audio/x-raw,rate=48000,channels=2 ! \
             {} bitrate={} ! queue",
            match self.config.format {
                OutputFormat::MP4 | OutputFormat::MKV => "fdkaacenc",
                OutputFormat::WebM => "opusenc",
                OutputFormat::GIF => return String::new(), // No audio for GIF
            },
            self.config.audio_bitrate * 1000
        )
    }

    /// Start encoding
    pub fn start(&self) -> Result<()> {
        let pipeline = self.pipeline.as_ref()
            .ok_or_else(|| anyhow!("Pipeline not built"))?;

        pipeline.set_state(gst::State::Playing)
            .map_err(|e| anyhow!("Failed to start pipeline: {:?}", e))?;

        *self.state.lock() = EncoderState::Encoding;
        Ok(())
    }

    /// Pause encoding
    pub fn pause(&self) -> Result<()> {
        let pipeline = self.pipeline.as_ref()
            .ok_or_else(|| anyhow!("Pipeline not built"))?;

        pipeline.set_state(gst::State::Paused)
            .map_err(|e| anyhow!("Failed to pause pipeline: {:?}", e))?;

        *self.state.lock() = EncoderState::Paused;
        Ok(())
    }

    /// Resume encoding
    pub fn resume(&self) -> Result<()> {
        let pipeline = self.pipeline.as_ref()
            .ok_or_else(|| anyhow!("Pipeline not built"))?;

        pipeline.set_state(gst::State::Playing)
            .map_err(|e| anyhow!("Failed to resume pipeline: {:?}", e))?;

        *self.state.lock() = EncoderState::Encoding;
        Ok(())
    }

    /// Stop encoding and finalize the file
    pub fn stop(&self) -> Result<()> {
        let pipeline = self.pipeline.as_ref()
            .ok_or_else(|| anyhow!("Pipeline not built"))?;

        // Send EOS to properly finalize the file
        pipeline.send_event(gst::event::Eos::new());

        // Wait for EOS to be processed
        let bus = pipeline.bus().ok_or_else(|| anyhow!("No bus on pipeline"))?;
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
            use gst::MessageView;
            match msg.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(err) => {
                    return Err(anyhow!("Encoding error: {}", err.error()));
                }
                _ => {}
            }
        }

        pipeline.set_state(gst::State::Null)
            .map_err(|e| anyhow!("Failed to stop pipeline: {:?}", e))?;

        *self.state.lock() = EncoderState::Finished;
        Ok(())
    }

    /// Get current pipeline position in nanoseconds
    pub fn position_ns(&self) -> Option<u64> {
        self.pipeline.as_ref().and_then(|p| {
            p.query_position::<gst::ClockTime>().map(|t| t.nseconds())
        })
    }

    /// Check if a codec is available
    pub fn is_codec_available(codec: VideoCodec) -> bool {
        let encoder_name = match codec {
            VideoCodec::H264 => "x264enc",
            VideoCodec::H265 => "x265enc",
            VideoCodec::VP9 => "vp9enc",
            VideoCodec::AV1 => "av1enc",
        };

        gst::ElementFactory::find(encoder_name).is_some()
    }

    /// Get list of available codecs
    pub fn available_codecs() -> Vec<VideoCodec> {
        let mut codecs = Vec::new();

        if Self::is_codec_available(VideoCodec::H264) {
            codecs.push(VideoCodec::H264);
        }
        if Self::is_codec_available(VideoCodec::H265) {
            codecs.push(VideoCodec::H265);
        }
        if Self::is_codec_available(VideoCodec::VP9) {
            codecs.push(VideoCodec::VP9);
        }
        if Self::is_codec_available(VideoCodec::AV1) {
            codecs.push(VideoCodec::AV1);
        }

        codecs
    }

    /// Check if hardware acceleration is available
    pub fn has_hardware_acceleration() -> bool {
        gst::ElementFactory::find("vaapih264enc").is_some() ||
        gst::ElementFactory::find("nvh264enc").is_some()
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}

/// Calculate recommended bitrate based on resolution and framerate
pub fn recommended_bitrate(resolution: Resolution, framerate: Framerate) -> u32 {
    let base_bitrate = match resolution {
        Resolution::Original => 12000, // Assume 1080p
        Resolution::R1080p => 12000,
        Resolution::R720p => 6000,
        Resolution::R480p => 2500,
    };

    // Increase bitrate for higher framerates
    let fps_multiplier = match framerate {
        Framerate::Fps24 => 0.8,
        Framerate::Fps30 => 1.0,
        Framerate::Fps60 => 1.5,
    };

    (base_bitrate as f64 * fps_multiplier) as u32
}
