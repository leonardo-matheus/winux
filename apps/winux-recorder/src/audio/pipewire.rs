//! PipeWire audio recording support
//!
//! This module provides recording functionality via PipeWire on Linux.
//! Falls back to ALSA/cpal on systems without PipeWire.

use std::sync::Arc;
use parking_lot::RwLock;
use std::path::PathBuf;
use tokio::sync::mpsc;

use super::{AudioFormat, AudioSamples, RecordingQuality};
use crate::AppState;

/// Messages for the recording thread
#[derive(Debug)]
pub enum RecorderMessage {
    /// Start recording
    Start,
    /// Pause recording
    Pause,
    /// Resume recording
    Resume,
    /// Stop recording and save
    Stop,
    /// Add a marker at current position
    AddMarker,
    /// Cancel recording (discard)
    Cancel,
}

/// Recording events sent back to UI
#[derive(Debug, Clone)]
pub enum RecorderEvent {
    /// Recording started
    Started,
    /// Recording paused
    Paused,
    /// Recording resumed
    Resumed,
    /// Recording stopped with file path
    Stopped(PathBuf),
    /// Recording cancelled
    Cancelled,
    /// Error occurred
    Error(String),
    /// Audio level update (0.0 - 1.0)
    Level(f32),
    /// Waveform data update
    Waveform(Vec<f32>),
    /// Duration update in seconds
    Duration(f64),
    /// Marker added at timestamp
    MarkerAdded(f64),
}

/// PipeWire-based audio recorder
pub struct PipeWireRecorder {
    /// Application state
    state: Arc<RwLock<AppState>>,
    /// Command sender
    cmd_tx: Option<mpsc::Sender<RecorderMessage>>,
    /// Event receiver
    event_rx: Option<mpsc::Receiver<RecorderEvent>>,
    /// Recording quality settings
    quality: RecordingQuality,
    /// Output format
    format: AudioFormat,
    /// Output path
    output_path: PathBuf,
    /// Collected samples
    samples: Arc<RwLock<AudioSamples>>,
    /// Recording active flag
    is_recording: Arc<RwLock<bool>>,
    /// Recording paused flag
    is_paused: Arc<RwLock<bool>>,
    /// Start time
    start_time: Option<std::time::Instant>,
    /// Accumulated duration (for pause/resume)
    accumulated_duration: std::time::Duration,
}

impl PipeWireRecorder {
    /// Create a new recorder
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        let config = state.read().config.clone();

        let quality = RecordingQuality {
            sample_rate: config.sample_rate,
            channels: config.channels,
            bit_depth: 16,
        };

        Self {
            state,
            cmd_tx: None,
            event_rx: None,
            quality,
            format: config.default_format,
            output_path: config.output_dir,
            samples: Arc::new(RwLock::new(AudioSamples::new(
                config.sample_rate,
                config.channels,
            ))),
            is_recording: Arc::new(RwLock::new(false)),
            is_paused: Arc::new(RwLock::new(false)),
            start_time: None,
            accumulated_duration: std::time::Duration::ZERO,
        }
    }

    /// Initialize the recorder and return event channel
    pub fn init(&mut self) -> mpsc::Receiver<RecorderEvent> {
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (event_tx, event_rx) = mpsc::channel(32);

        self.cmd_tx = Some(cmd_tx);

        // Clone state for the recording thread
        let state = self.state.clone();
        let samples = self.samples.clone();
        let is_recording = self.is_recording.clone();
        let is_paused = self.is_paused.clone();

        // Spawn recording thread
        std::thread::spawn(move || {
            run_recorder(state, samples, is_recording, is_paused, cmd_rx, event_tx);
        });

        event_rx
    }

    /// Send a command to the recorder
    pub fn send(&self, msg: RecorderMessage) {
        if let Some(ref tx) = self.cmd_tx {
            let _ = tx.blocking_send(msg);
        }
    }

    /// Start recording
    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
        self.accumulated_duration = std::time::Duration::ZERO;
        *self.is_recording.write() = true;
        *self.is_paused.write() = false;
        self.send(RecorderMessage::Start);
    }

    /// Pause recording
    pub fn pause(&mut self) {
        if let Some(start) = self.start_time {
            self.accumulated_duration += start.elapsed();
        }
        self.start_time = None;
        *self.is_paused.write() = true;
        self.send(RecorderMessage::Pause);
    }

    /// Resume recording
    pub fn resume(&mut self) {
        self.start_time = Some(std::time::Instant::now());
        *self.is_paused.write() = false;
        self.send(RecorderMessage::Resume);
    }

    /// Stop recording and save
    pub fn stop(&mut self) {
        *self.is_recording.write() = false;
        self.send(RecorderMessage::Stop);
    }

    /// Cancel recording
    pub fn cancel(&mut self) {
        *self.is_recording.write() = false;
        self.send(RecorderMessage::Cancel);
    }

    /// Add marker at current position
    pub fn add_marker(&self) {
        self.send(RecorderMessage::AddMarker);
    }

    /// Get current duration
    pub fn duration(&self) -> std::time::Duration {
        let current = self.start_time
            .map(|s| s.elapsed())
            .unwrap_or(std::time::Duration::ZERO);
        self.accumulated_duration + current
    }

    /// Check if recording
    pub fn is_recording(&self) -> bool {
        *self.is_recording.read()
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        *self.is_paused.read()
    }
}

/// Run the recording loop in a separate thread
fn run_recorder(
    state: Arc<RwLock<AppState>>,
    samples: Arc<RwLock<AudioSamples>>,
    is_recording: Arc<RwLock<bool>>,
    is_paused: Arc<RwLock<bool>>,
    mut cmd_rx: mpsc::Receiver<RecorderMessage>,
    event_tx: mpsc::Sender<RecorderEvent>,
) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();

    // Get device
    let config = state.read().config.clone();
    let device = if let Some(ref name) = config.input_device {
        match host.input_devices() {
            Ok(mut devices) => devices.find(|d| d.name().ok().as_deref() == Some(name)),
            Err(_) => None,
        }
    } else {
        None
    }.or_else(|| host.default_input_device());

    let device = match device {
        Some(d) => d,
        None => {
            let _ = event_tx.blocking_send(RecorderEvent::Error("No input device found".into()));
            return;
        }
    };

    let stream_config = cpal::StreamConfig {
        channels: config.channels,
        sample_rate: cpal::SampleRate(config.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let samples_for_stream = samples.clone();
    let is_recording_for_stream = is_recording.clone();
    let is_paused_for_stream = is_paused.clone();
    let event_tx_for_stream = event_tx.clone();

    // Create input stream
    let stream = match device.build_input_stream(
        &stream_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if !*is_recording_for_stream.read() || *is_paused_for_stream.read() {
                return;
            }

            // Append samples
            samples_for_stream.write().data.extend_from_slice(data);

            // Calculate peak level
            let peak: f32 = data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            let _ = event_tx_for_stream.blocking_send(RecorderEvent::Level(peak));

            // Send waveform data periodically (every ~100ms worth of samples)
            let samples_per_update = (config.sample_rate as usize * config.channels as usize) / 10;
            let current_len = samples_for_stream.read().data.len();
            if current_len % samples_per_update < data.len() {
                let samples = samples_for_stream.read();
                let waveform = samples.downsample(200);
                let _ = event_tx_for_stream.blocking_send(RecorderEvent::Waveform(waveform));
                let _ = event_tx_for_stream.blocking_send(RecorderEvent::Duration(samples.duration()));
            }
        },
        move |err| {
            tracing::error!("Audio stream error: {}", err);
        },
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            let _ = event_tx.blocking_send(RecorderEvent::Error(format!("Failed to create stream: {}", e)));
            return;
        }
    };

    let mut stream_playing = false;
    let mut markers: Vec<f64> = Vec::new();

    // Process commands
    while let Some(cmd) = cmd_rx.blocking_recv() {
        match cmd {
            RecorderMessage::Start => {
                samples.write().data.clear();
                markers.clear();
                if let Err(e) = stream.play() {
                    let _ = event_tx.blocking_send(RecorderEvent::Error(format!("Failed to start: {}", e)));
                } else {
                    stream_playing = true;
                    let _ = event_tx.blocking_send(RecorderEvent::Started);
                }
            }
            RecorderMessage::Pause => {
                if stream_playing {
                    let _ = stream.pause();
                    let _ = event_tx.blocking_send(RecorderEvent::Paused);
                }
            }
            RecorderMessage::Resume => {
                if stream_playing {
                    let _ = stream.play();
                    let _ = event_tx.blocking_send(RecorderEvent::Resumed);
                }
            }
            RecorderMessage::Stop => {
                let _ = stream.pause();
                stream_playing = false;

                // Save the recording
                let samples_data = samples.read().clone();
                let output_path = save_recording(&state, &samples_data, &markers);

                match output_path {
                    Ok(path) => {
                        let _ = event_tx.blocking_send(RecorderEvent::Stopped(path));
                    }
                    Err(e) => {
                        let _ = event_tx.blocking_send(RecorderEvent::Error(format!("Failed to save: {}", e)));
                    }
                }
                break;
            }
            RecorderMessage::Cancel => {
                let _ = stream.pause();
                stream_playing = false;
                let _ = event_tx.blocking_send(RecorderEvent::Cancelled);
                break;
            }
            RecorderMessage::AddMarker => {
                let duration = samples.read().duration();
                markers.push(duration);
                let _ = event_tx.blocking_send(RecorderEvent::MarkerAdded(duration));
            }
        }
    }
}

/// Save recording to file
fn save_recording(
    state: &Arc<RwLock<AppState>>,
    samples: &AudioSamples,
    _markers: &[f64],
) -> Result<PathBuf, anyhow::Error> {
    use super::encoder::AudioEncoder;

    let config = state.read().config.clone();

    // Generate filename
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("Recording_{}.{}", timestamp, config.default_format.extension());
    let output_path = config.output_dir.join(&filename);

    // Ensure output directory exists
    std::fs::create_dir_all(&config.output_dir)?;

    // Encode and save
    let encoder = AudioEncoder::new(config.default_format);
    encoder.encode_to_file(samples, &output_path)?;

    tracing::info!("Recording saved to: {:?}", output_path);

    Ok(output_path)
}
