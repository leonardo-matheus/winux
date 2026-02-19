//! Recording session management
//!
//! Manages the lifecycle of a single recording session, coordinating
//! between the capture source, encoder, and output handling.

use anyhow::{Result, anyhow};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc;

use crate::AppState;
use crate::capture::{SourceType, CursorMode, CaptureSource, ScreencastPortal};
use crate::capture::encoder::{VideoEncoder, EncoderConfig, EncoderState};
use crate::recording::{RecordingConfig, RecordingState};

/// Events that can occur during a recording session
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// Recording started successfully
    Started,
    /// Recording paused
    Paused,
    /// Recording resumed
    Resumed,
    /// Recording stopped
    Stopped,
    /// Recording progress update
    Progress {
        duration: Duration,
        bytes_written: u64,
    },
    /// An error occurred
    Error(String),
}

/// Recording session state
#[derive(Debug)]
struct SessionState {
    /// Recording start time
    start_time: Option<Instant>,
    /// Total paused duration
    paused_duration: Duration,
    /// When the current pause started
    pause_start: Option<Instant>,
    /// Current state
    state: RecordingState,
    /// Output file path
    output_path: PathBuf,
    /// Portal session path
    portal_session: Option<String>,
}

/// A recording session
pub struct RecordingSession {
    /// Session state
    state: Arc<Mutex<SessionState>>,
    /// Recording configuration
    config: RecordingConfig,
    /// Video encoder
    encoder: Arc<Mutex<Option<VideoEncoder>>>,
    /// Capture source information
    capture_source: Arc<Mutex<Option<CaptureSource>>>,
    /// Event sender
    event_tx: mpsc::UnboundedSender<SessionEvent>,
    /// Event receiver (for the UI to listen)
    event_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<SessionEvent>>>>,
}

impl RecordingSession {
    /// Create a new recording session
    pub fn new(config: RecordingConfig, output_path: PathBuf) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            state: Arc::new(Mutex::new(SessionState {
                start_time: None,
                paused_duration: Duration::ZERO,
                pause_start: None,
                state: RecordingState::Idle,
                output_path,
                portal_session: None,
            })),
            config,
            encoder: Arc::new(Mutex::new(None)),
            capture_source: Arc::new(Mutex::new(None)),
            event_tx,
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
        }
    }

    /// Start the recording session from app state
    pub async fn start(app_state: &Rc<RefCell<AppState>>) -> Result<Self> {
        let state = app_state.borrow();

        let config = state.config.clone();
        let output_path = state.get_output_path();
        let source_type = state.source_type;

        drop(state);

        let session = Self::new(config.clone(), output_path.clone());
        session.start_capture(source_type).await?;

        Ok(session)
    }

    /// Start the capture process
    async fn start_capture(&self, source_type: SourceType) -> Result<()> {
        // Update state
        {
            let mut state = self.state.lock();
            state.state = RecordingState::Starting;
        }

        // Initialize portal and request capture
        let portal = ScreencastPortal::new().await?;

        // Create portal session
        let session_path = portal.create_session().await?;

        // Store portal session for cleanup
        {
            let mut state = self.state.lock();
            state.portal_session = Some(session_path.clone());
        }

        // Select sources
        portal.select_sources(
            &session_path,
            source_type.portal_source_type(),
            self.config.cursor_mode.portal_value(),
            false, // Single source for now
        ).await?;

        // Start the capture stream
        let streams = portal.start(&session_path).await?;

        if streams.is_empty() {
            return Err(anyhow!("No capture source selected"));
        }

        let stream = &streams[0];

        // Store capture source info
        let capture_source = CaptureSource {
            node_id: stream.node_id,
            source_type,
            width: stream.width,
            height: stream.height,
            x: stream.x,
            y: stream.y,
            session_path: session_path.clone(),
        };

        *self.capture_source.lock() = Some(capture_source.clone());

        // Build and start the encoder
        self.start_encoder(capture_source.node_id).await?;

        // Update state and record start time
        {
            let mut state = self.state.lock();
            state.state = RecordingState::Recording;
            state.start_time = Some(Instant::now());
        }

        // Send event
        let _ = self.event_tx.send(SessionEvent::Started);

        // Start progress updates
        self.start_progress_updates();

        Ok(())
    }

    /// Start the video encoder
    async fn start_encoder(&self, node_id: u32) -> Result<()> {
        let state = self.state.lock();
        let output_path = state.output_path.clone();
        drop(state);

        let encoder_config = EncoderConfig {
            codec: self.config.codec,
            format: self.config.format,
            resolution: self.config.resolution,
            framerate: self.config.framerate,
            bitrate: self.config.video_bitrate,
            hardware_accel: self.config.hardware_acceleration,
            include_audio: self.config.include_system_audio || self.config.include_microphone,
            audio_bitrate: self.config.audio_bitrate,
            output_path,
        };

        let mut encoder = VideoEncoder::new(encoder_config);
        encoder.build_pipeline(node_id)?;
        encoder.start()?;

        *self.encoder.lock() = Some(encoder);

        Ok(())
    }

    /// Start periodic progress updates
    fn start_progress_updates(&self) {
        let state = self.state.clone();
        let encoder = self.encoder.clone();
        let event_tx = self.event_tx.clone();

        // Spawn a task for progress updates
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));

            loop {
                interval.tick().await;

                let current_state = {
                    let state = state.lock();
                    state.state
                };

                if current_state != RecordingState::Recording && current_state != RecordingState::Paused {
                    break;
                }

                // Get duration and bytes
                let duration = {
                    let state = state.lock();
                    if let Some(start) = state.start_time {
                        let total = start.elapsed();
                        total.saturating_sub(state.paused_duration)
                    } else {
                        Duration::ZERO
                    }
                };

                let bytes_written = {
                    let encoder = encoder.lock();
                    encoder.as_ref().map(|e| e.bytes_written()).unwrap_or(0)
                };

                // Send progress event
                let _ = event_tx.send(SessionEvent::Progress {
                    duration,
                    bytes_written,
                });
            }
        });
    }

    /// Pause the recording
    pub fn pause(&self) -> Result<()> {
        let mut state = self.state.lock();

        if state.state != RecordingState::Recording {
            return Err(anyhow!("Recording is not active"));
        }

        state.state = RecordingState::Paused;
        state.pause_start = Some(Instant::now());

        // Pause encoder
        if let Some(encoder) = self.encoder.lock().as_ref() {
            encoder.pause()?;
        }

        let _ = self.event_tx.send(SessionEvent::Paused);

        Ok(())
    }

    /// Resume the recording
    pub fn resume(&self) -> Result<()> {
        let mut state = self.state.lock();

        if state.state != RecordingState::Paused {
            return Err(anyhow!("Recording is not paused"));
        }

        // Add paused duration
        if let Some(pause_start) = state.pause_start.take() {
            state.paused_duration += pause_start.elapsed();
        }

        state.state = RecordingState::Recording;

        // Resume encoder
        if let Some(encoder) = self.encoder.lock().as_ref() {
            encoder.resume()?;
        }

        let _ = self.event_tx.send(SessionEvent::Resumed);

        Ok(())
    }

    /// Stop and finalize the recording
    pub async fn stop(&self) -> Result<PathBuf> {
        {
            let mut state = self.state.lock();
            state.state = RecordingState::Stopping;
        }

        // Stop encoder
        if let Some(encoder) = self.encoder.lock().as_ref() {
            encoder.stop()?;
        }

        // Close portal session
        let portal_session = {
            let state = self.state.lock();
            state.portal_session.clone()
        };

        if let Some(session_path) = portal_session {
            let portal = ScreencastPortal::new().await?;
            let _ = portal.close_session(&session_path).await;
        }

        // Get output path
        let output_path = {
            let mut state = self.state.lock();
            state.state = RecordingState::Completed;
            state.output_path.clone()
        };

        let _ = self.event_tx.send(SessionEvent::Stopped);

        Ok(output_path)
    }

    /// Cancel the recording without saving
    pub async fn cancel(&self) -> Result<()> {
        // Stop encoder without finalizing
        if let Some(encoder) = self.encoder.lock().take() {
            drop(encoder);
        }

        // Close portal session
        let portal_session = {
            let state = self.state.lock();
            state.portal_session.clone()
        };

        if let Some(session_path) = portal_session {
            let portal = ScreencastPortal::new().await?;
            let _ = portal.close_session(&session_path).await;
        }

        // Delete output file if it exists
        let output_path = {
            let mut state = self.state.lock();
            state.state = RecordingState::Idle;
            state.output_path.clone()
        };

        if output_path.exists() {
            let _ = std::fs::remove_file(&output_path);
        }

        Ok(())
    }

    /// Get the current recording state
    pub fn state(&self) -> RecordingState {
        self.state.lock().state
    }

    /// Get the current recording duration
    pub fn duration(&self) -> Duration {
        let state = self.state.lock();

        if let Some(start) = state.start_time {
            let total = start.elapsed();
            total.saturating_sub(state.paused_duration)
        } else {
            Duration::ZERO
        }
    }

    /// Get the output file path
    pub fn output_path(&self) -> PathBuf {
        self.state.lock().output_path.clone()
    }

    /// Take the event receiver (can only be called once)
    pub fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<SessionEvent>> {
        self.event_rx.lock().take()
    }

    /// Get capture source information
    pub fn capture_source(&self) -> Option<CaptureSource> {
        self.capture_source.lock().clone()
    }
}

impl Drop for RecordingSession {
    fn drop(&mut self) {
        // Ensure resources are cleaned up
        if let Some(encoder) = self.encoder.lock().take() {
            drop(encoder);
        }
    }
}
