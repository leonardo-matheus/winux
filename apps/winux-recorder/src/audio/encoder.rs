//! Audio encoders for different output formats

use std::path::Path;
use super::{AudioFormat, AudioSamples};

/// Encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Target bitrate for lossy formats (bits per second)
    pub bitrate: u32,
    /// Quality for VBR encoding (0.0 - 1.0)
    pub quality: f32,
    /// Use variable bitrate
    pub vbr: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            bitrate: 192_000,
            quality: 0.5,
            vbr: true,
        }
    }
}

impl EncoderConfig {
    pub fn low() -> Self {
        Self {
            bitrate: 64_000,
            quality: 0.3,
            vbr: true,
        }
    }

    pub fn medium() -> Self {
        Self {
            bitrate: 128_000,
            quality: 0.5,
            vbr: true,
        }
    }

    pub fn high() -> Self {
        Self {
            bitrate: 320_000,
            quality: 0.8,
            vbr: true,
        }
    }
}

/// Audio encoder
pub struct AudioEncoder {
    format: AudioFormat,
    config: EncoderConfig,
}

impl AudioEncoder {
    /// Create a new encoder for the given format
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            config: EncoderConfig::default(),
        }
    }

    /// Create encoder with specific config
    pub fn with_config(format: AudioFormat, config: EncoderConfig) -> Self {
        Self { format, config }
    }

    /// Encode audio samples to file
    pub fn encode_to_file(&self, samples: &AudioSamples, path: &Path) -> Result<(), anyhow::Error> {
        match self.format {
            AudioFormat::Wav => self.encode_wav(samples, path),
            AudioFormat::Mp3 => self.encode_mp3(samples, path),
            AudioFormat::Ogg => self.encode_ogg(samples, path),
            AudioFormat::Flac => self.encode_flac(samples, path),
        }
    }

    /// Encode to WAV format using hound
    fn encode_wav(&self, samples: &AudioSamples, path: &Path) -> Result<(), anyhow::Error> {
        let spec = hound::WavSpec {
            channels: samples.channels,
            sample_rate: samples.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;

        // Convert f32 samples to i16
        for sample in &samples.data {
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        writer.finalize()?;
        Ok(())
    }

    /// Encode to MP3 format using ffmpeg (fallback to wav if not available)
    fn encode_mp3(&self, samples: &AudioSamples, path: &Path) -> Result<(), anyhow::Error> {
        // First, write to a temporary WAV file
        let temp_wav = path.with_extension("wav.tmp");
        self.encode_wav(samples, &temp_wav)?;

        // Try to use ffmpeg for MP3 encoding
        let result = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-i", temp_wav.to_str().unwrap(),
                "-codec:a", "libmp3lame",
                "-b:a", &format!("{}k", self.config.bitrate / 1000),
                "-ar", &samples.sample_rate.to_string(),
                "-ac", &samples.channels.to_string(),
                path.to_str().unwrap(),
            ])
            .output();

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_wav);

        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("FFmpeg encoding failed: {}", stderr))
            }
            Err(e) => {
                // ffmpeg not available, save as WAV with .mp3 extension
                // (not ideal, but better than losing the recording)
                tracing::warn!("FFmpeg not available, saving as WAV: {}", e);
                let wav_path = path.with_extension("wav");
                self.encode_wav(samples, &wav_path)?;
                std::fs::rename(&wav_path, path)?;
                Ok(())
            }
        }
    }

    /// Encode to OGG/Opus format using ffmpeg
    fn encode_ogg(&self, samples: &AudioSamples, path: &Path) -> Result<(), anyhow::Error> {
        // First, write to a temporary WAV file
        let temp_wav = path.with_extension("wav.tmp");
        self.encode_wav(samples, &temp_wav)?;

        // Try to use ffmpeg for OGG/Opus encoding
        let result = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-i", temp_wav.to_str().unwrap(),
                "-codec:a", "libopus",
                "-b:a", &format!("{}k", self.config.bitrate / 1000),
                "-ar", "48000", // Opus requires 48kHz
                "-ac", &samples.channels.to_string(),
                path.to_str().unwrap(),
            ])
            .output();

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_wav);

        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("FFmpeg encoding failed: {}", stderr))
            }
            Err(e) => {
                tracing::warn!("FFmpeg not available, saving as WAV: {}", e);
                let wav_path = path.with_extension("wav");
                self.encode_wav(samples, &wav_path)?;
                std::fs::rename(&wav_path, path)?;
                Ok(())
            }
        }
    }

    /// Encode to FLAC format using ffmpeg
    fn encode_flac(&self, samples: &AudioSamples, path: &Path) -> Result<(), anyhow::Error> {
        // First, write to a temporary WAV file
        let temp_wav = path.with_extension("wav.tmp");
        self.encode_wav(samples, &temp_wav)?;

        // Try to use ffmpeg for FLAC encoding
        let result = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-i", temp_wav.to_str().unwrap(),
                "-codec:a", "flac",
                "-compression_level", "8",
                "-ar", &samples.sample_rate.to_string(),
                "-ac", &samples.channels.to_string(),
                path.to_str().unwrap(),
            ])
            .output();

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_wav);

        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("FFmpeg encoding failed: {}", stderr))
            }
            Err(e) => {
                tracing::warn!("FFmpeg not available, saving as WAV: {}", e);
                let wav_path = path.with_extension("wav");
                self.encode_wav(samples, &wav_path)?;
                std::fs::rename(&wav_path, path)?;
                Ok(())
            }
        }
    }
}

/// Decode audio file to samples
pub fn decode_file(path: &Path) -> Result<AudioSamples, anyhow::Error> {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "wav" => decode_wav(path),
        "mp3" | "ogg" | "opus" | "flac" => decode_with_ffmpeg(path),
        _ => Err(anyhow::anyhow!("Unsupported audio format: {}", extension)),
    }
}

/// Decode WAV file using hound
fn decode_wav(path: &Path) -> Result<AudioSamples, anyhow::Error> {
    let reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max_value = (1 << (spec.bits_per_sample - 1)) as f32;
            reader
                .into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_value)
                .collect()
        }
        hound::SampleFormat::Float => {
            reader
                .into_samples::<f32>()
                .filter_map(|s| s.ok())
                .collect()
        }
    };

    Ok(AudioSamples {
        data: samples,
        sample_rate: spec.sample_rate,
        channels: spec.channels,
    })
}

/// Decode non-WAV files using ffmpeg
fn decode_with_ffmpeg(path: &Path) -> Result<AudioSamples, anyhow::Error> {
    // Convert to WAV using ffmpeg, then decode
    let temp_wav = std::env::temp_dir().join(format!(
        "winux_recorder_{}.wav",
        uuid::Uuid::new_v4()
    ));

    let output = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", path.to_str().unwrap(),
            "-codec:a", "pcm_s16le",
            "-f", "wav",
            temp_wav.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("FFmpeg decoding failed: {}", stderr));
    }

    let samples = decode_wav(&temp_wav)?;

    // Clean up
    let _ = std::fs::remove_file(&temp_wav);

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_roundtrip() {
        let samples = AudioSamples {
            data: vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5],
            sample_rate: 44100,
            channels: 1,
        };

        let temp_path = std::env::temp_dir().join("test_recording.wav");
        let encoder = AudioEncoder::new(AudioFormat::Wav);
        encoder.encode_to_file(&samples, &temp_path).unwrap();

        let decoded = decode_wav(&temp_path).unwrap();
        assert_eq!(decoded.sample_rate, samples.sample_rate);
        assert_eq!(decoded.channels, samples.channels);

        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }
}
