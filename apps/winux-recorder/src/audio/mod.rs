//! Audio module - handles audio recording and playback

pub mod device;
pub mod encoder;
pub mod pipewire;

pub use device::{AudioDevice, list_input_devices};
pub use encoder::{AudioEncoder, EncoderConfig};
pub use pipewire::PipeWireRecorder;

/// Audio format for recording output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AudioFormat {
    #[default]
    Wav,
    Mp3,
    Ogg,
    Flac,
}

impl AudioFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Flac => "flac",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Flac => "audio/flac",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "WAV",
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Ogg => "OGG/Opus",
            AudioFormat::Flac => "FLAC",
        }
    }

    pub fn is_lossless(&self) -> bool {
        matches!(self, AudioFormat::Wav | AudioFormat::Flac)
    }
}

/// Audio sample data
#[derive(Debug, Clone)]
pub struct AudioSamples {
    /// Raw audio samples (normalized -1.0 to 1.0)
    pub data: Vec<f32>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
}

impl AudioSamples {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            data: Vec::new(),
            sample_rate,
            channels,
        }
    }

    /// Get duration in seconds
    pub fn duration(&self) -> f64 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0.0;
        }
        self.data.len() as f64 / (self.sample_rate as f64 * self.channels as f64)
    }

    /// Get peak amplitude
    pub fn peak(&self) -> f32 {
        self.data.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    /// Downsample for visualization (returns N samples)
    pub fn downsample(&self, target_samples: usize) -> Vec<f32> {
        if self.data.is_empty() || target_samples == 0 {
            return vec![0.0; target_samples];
        }

        let chunk_size = self.data.len() / target_samples;
        if chunk_size == 0 {
            return self.data.clone();
        }

        self.data
            .chunks(chunk_size)
            .map(|chunk| {
                chunk.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
            })
            .take(target_samples)
            .collect()
    }
}

/// Recording quality settings
#[derive(Debug, Clone)]
pub struct RecordingQuality {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
}

impl Default for RecordingQuality {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 1,
            bit_depth: 16,
        }
    }
}

impl RecordingQuality {
    pub fn low() -> Self {
        Self {
            sample_rate: 22050,
            channels: 1,
            bit_depth: 16,
        }
    }

    pub fn medium() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1,
            bit_depth: 16,
        }
    }

    pub fn high() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bit_depth: 24,
        }
    }
}
