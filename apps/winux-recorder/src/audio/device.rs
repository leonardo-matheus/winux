//! Audio device management

use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::Arc;
use parking_lot::RwLock;

/// Represents an audio input device
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device name
    pub name: String,
    /// Device identifier
    pub id: String,
    /// Whether this is the default device
    pub is_default: bool,
    /// Supported sample rates
    pub supported_sample_rates: Vec<u32>,
    /// Supported channel counts
    pub supported_channels: Vec<u16>,
}

impl AudioDevice {
    /// Get the default input device
    pub fn default_input() -> Option<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()?;
        let name = device.name().ok()?;

        let supported_configs: Vec<_> = device
            .supported_input_configs()
            .ok()?
            .collect();

        let mut sample_rates: Vec<u32> = supported_configs
            .iter()
            .flat_map(|c| vec![c.min_sample_rate().0, c.max_sample_rate().0])
            .collect();
        sample_rates.sort_unstable();
        sample_rates.dedup();

        let mut channels: Vec<u16> = supported_configs
            .iter()
            .map(|c| c.channels())
            .collect();
        channels.sort_unstable();
        channels.dedup();

        Some(Self {
            name: name.clone(),
            id: name,
            is_default: true,
            supported_sample_rates: sample_rates,
            supported_channels: channels,
        })
    }

    /// Get a device by name
    pub fn by_name(name: &str) -> Option<Self> {
        let host = cpal::default_host();
        let device = host.input_devices().ok()?
            .find(|d| d.name().ok().as_deref() == Some(name))?;

        let device_name = device.name().ok()?;
        let default_name = host.default_input_device()
            .and_then(|d| d.name().ok());

        let supported_configs: Vec<_> = device
            .supported_input_configs()
            .ok()?
            .collect();

        let mut sample_rates: Vec<u32> = supported_configs
            .iter()
            .flat_map(|c| vec![c.min_sample_rate().0, c.max_sample_rate().0])
            .collect();
        sample_rates.sort_unstable();
        sample_rates.dedup();

        let mut channels: Vec<u16> = supported_configs
            .iter()
            .map(|c| c.channels())
            .collect();
        channels.sort_unstable();
        channels.dedup();

        Some(Self {
            name: device_name.clone(),
            id: device_name.clone(),
            is_default: default_name.as_deref() == Some(&device_name),
            supported_sample_rates: sample_rates,
            supported_channels: channels,
        })
    }
}

/// List all available input devices
pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();

    match host.input_devices() {
        Ok(devices) => devices
            .filter_map(|d| d.name().ok())
            .collect(),
        Err(e) => {
            tracing::warn!("Failed to enumerate input devices: {}", e);
            Vec::new()
        }
    }
}

/// Audio input stream handler
pub struct AudioInput {
    /// Currently selected device name
    device_name: Option<String>,
    /// Sample rate
    sample_rate: u32,
    /// Number of channels
    channels: u16,
    /// Buffer for incoming samples
    buffer: Arc<RwLock<Vec<f32>>>,
    /// Peak level (0.0 - 1.0)
    peak_level: Arc<RwLock<f32>>,
    /// Whether recording is active
    is_recording: Arc<RwLock<bool>>,
}

impl AudioInput {
    pub fn new(device_name: Option<String>, sample_rate: u32, channels: u16) -> Self {
        Self {
            device_name,
            sample_rate,
            channels,
            buffer: Arc::new(RwLock::new(Vec::new())),
            peak_level: Arc::new(RwLock::new(0.0)),
            is_recording: Arc::new(RwLock::new(false)),
        }
    }

    /// Start recording
    pub fn start(&self) -> Result<cpal::Stream, anyhow::Error> {
        let host = cpal::default_host();

        let device = if let Some(ref name) = self.device_name {
            host.input_devices()?
                .find(|d| d.name().ok().as_deref() == Some(name))
                .ok_or_else(|| anyhow::anyhow!("Device not found: {}", name))?
        } else {
            host.default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device"))?
        };

        let config = cpal::StreamConfig {
            channels: self.channels,
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = self.buffer.clone();
        let peak_level = self.peak_level.clone();
        let is_recording = self.is_recording.clone();

        *is_recording.write() = true;

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !*is_recording.read() {
                    return;
                }

                // Calculate peak level
                let peak: f32 = data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                *peak_level.write() = peak;

                // Append to buffer
                buffer.write().extend_from_slice(data);
            },
            move |err| {
                tracing::error!("Audio input error: {}", err);
            },
            None,
        )?;

        use cpal::traits::StreamTrait;
        stream.play()?;

        Ok(stream)
    }

    /// Stop recording and return samples
    pub fn stop(&self) -> Vec<f32> {
        *self.is_recording.write() = false;
        std::mem::take(&mut *self.buffer.write())
    }

    /// Get current peak level
    pub fn peak_level(&self) -> f32 {
        *self.peak_level.read()
    }

    /// Get current buffer length in samples
    pub fn buffer_len(&self) -> usize {
        self.buffer.read().len()
    }

    /// Clear the buffer
    pub fn clear_buffer(&self) {
        self.buffer.write().clear();
    }

    /// Get a copy of current samples for visualization
    pub fn get_samples(&self) -> Vec<f32> {
        self.buffer.read().clone()
    }

    /// Get recent samples for waveform (last N samples)
    pub fn get_recent_samples(&self, count: usize) -> Vec<f32> {
        let buffer = self.buffer.read();
        if buffer.len() <= count {
            buffer.clone()
        } else {
            buffer[buffer.len() - count..].to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let devices = list_input_devices();
        println!("Found {} input devices", devices.len());
        for device in &devices {
            println!("  - {}", device);
        }
    }
}
