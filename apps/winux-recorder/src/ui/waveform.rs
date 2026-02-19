//! Waveform visualization widget

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use std::sync::Arc;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::recording::RecordingState;

/// Waveform visualization widget
pub struct WaveformView {
    drawing_area: gtk::DrawingArea,
    state: Arc<RwLock<AppState>>,
    /// Local samples for drawing
    samples: Rc<RefCell<Vec<f32>>>,
    /// Playback position (0.0 - 1.0)
    playback_position: Rc<RefCell<f64>>,
    /// Markers to display
    markers: Rc<RefCell<Vec<f64>>>,
}

impl WaveformView {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        let drawing_area = gtk::DrawingArea::builder()
            .content_width(400)
            .content_height(100)
            .hexpand(true)
            .build();

        let samples = Rc::new(RefCell::new(Vec::new()));
        let playback_position = Rc::new(RefCell::new(0.0));
        let markers = Rc::new(RefCell::new(Vec::new()));

        // Set up drawing function
        let samples_for_draw = samples.clone();
        let playback_pos_for_draw = playback_position.clone();
        let markers_for_draw = markers.clone();
        let state_for_draw = state.clone();

        drawing_area.set_draw_func(move |_area, cr, width, height| {
            draw_waveform(
                cr,
                width,
                height,
                &samples_for_draw.borrow(),
                *playback_pos_for_draw.borrow(),
                &markers_for_draw.borrow(),
                &state_for_draw,
            );
        });

        // Set up update timer
        let drawing_area_weak = drawing_area.downgrade();
        let state_for_timer = state.clone();
        let samples_for_timer = samples.clone();
        let markers_for_timer = markers.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            if let Some(area) = drawing_area_weak.upgrade() {
                // Update samples from state
                let state = state_for_timer.read();
                *samples_for_timer.borrow_mut() = state.waveform_samples.clone();
                *markers_for_timer.borrow_mut() = state.markers.clone();

                area.queue_draw();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        Self {
            drawing_area,
            state,
            samples,
            playback_position,
            markers,
        }
    }

    pub fn widget(&self) -> &gtk::DrawingArea {
        &self.drawing_area
    }

    /// Set samples for display
    pub fn set_samples(&self, new_samples: Vec<f32>) {
        *self.samples.borrow_mut() = new_samples;
        self.drawing_area.queue_draw();
    }

    /// Set playback position (0.0 - 1.0)
    pub fn set_playback_position(&self, position: f64) {
        *self.playback_position.borrow_mut() = position.clamp(0.0, 1.0);
        self.drawing_area.queue_draw();
    }

    /// Add a marker
    pub fn add_marker(&self, position: f64) {
        self.markers.borrow_mut().push(position);
        self.drawing_area.queue_draw();
    }

    /// Clear waveform
    pub fn clear(&self) {
        self.samples.borrow_mut().clear();
        self.markers.borrow_mut().clear();
        *self.playback_position.borrow_mut() = 0.0;
        self.drawing_area.queue_draw();
    }
}

/// Draw the waveform
fn draw_waveform(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    samples: &[f32],
    playback_position: f64,
    markers: &[f64],
    state: &Arc<RwLock<AppState>>,
) {
    let width = width as f64;
    let height = height as f64;
    let center_y = height / 2.0;

    // Background
    cr.set_source_rgb(0.12, 0.12, 0.14);
    let _ = cr.paint();

    // Draw center line
    cr.set_source_rgb(0.25, 0.25, 0.28);
    cr.set_line_width(1.0);
    cr.move_to(0.0, center_y);
    cr.line_to(width, center_y);
    let _ = cr.stroke();

    // Get recording state for color
    let recording_state = state.read().recording_state;
    let is_recording = recording_state == RecordingState::Recording;
    let is_paused = recording_state == RecordingState::Paused;

    // Draw waveform
    if !samples.is_empty() {
        let bar_width = (width / samples.len() as f64).max(2.0);
        let gap = 1.0;

        for (i, &sample) in samples.iter().enumerate() {
            let x = i as f64 * (width / samples.len() as f64);
            let bar_height = (sample * center_y * 0.9).max(2.0);

            // Color based on state
            if is_recording {
                // Recording: red/orange gradient
                let intensity = 0.6 + sample * 0.4;
                cr.set_source_rgb(0.9 * intensity, 0.3 * intensity, 0.2 * intensity);
            } else if is_paused {
                // Paused: yellow/orange
                cr.set_source_rgb(0.9, 0.7, 0.2);
            } else {
                // Idle/playback: blue/purple gradient
                let intensity = 0.6 + sample * 0.4;
                cr.set_source_rgb(0.3 * intensity, 0.5 * intensity, 0.9 * intensity);
            }

            // Draw bar (both directions from center)
            cr.rectangle(x, center_y - bar_height, bar_width - gap, bar_height * 2.0);
            let _ = cr.fill();
        }

        // Draw playback position indicator
        if playback_position > 0.0 && playback_position < 1.0 {
            let pos_x = playback_position * width;
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_line_width(2.0);
            cr.move_to(pos_x, 0.0);
            cr.line_to(pos_x, height);
            let _ = cr.stroke();

            // Playhead triangle
            cr.move_to(pos_x - 6.0, 0.0);
            cr.line_to(pos_x + 6.0, 0.0);
            cr.line_to(pos_x, 10.0);
            cr.close_path();
            let _ = cr.fill();
        }

        // Draw markers
        cr.set_source_rgb(0.2, 0.8, 0.4);
        cr.set_line_width(2.0);

        let duration = state.read().duration;
        for &marker in markers {
            if duration > 0.0 {
                let marker_x = (marker / duration) * width;
                cr.move_to(marker_x, 0.0);
                cr.line_to(marker_x, height);
                let _ = cr.stroke();

                // Marker diamond
                cr.move_to(marker_x, 5.0);
                cr.line_to(marker_x + 5.0, 10.0);
                cr.line_to(marker_x, 15.0);
                cr.line_to(marker_x - 5.0, 10.0);
                cr.close_path();
                let _ = cr.fill();
            }
        }
    } else {
        // No samples - draw placeholder
        cr.set_source_rgb(0.3, 0.3, 0.35);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(14.0);

        let text = if is_recording {
            "Recording..."
        } else {
            "Press record to start"
        };

        let extents = cr.text_extents(text).unwrap();
        cr.move_to(
            (width - extents.width()) / 2.0,
            center_y + extents.height() / 2.0,
        );
        let _ = cr.show_text(text);
    }
}

/// Create a seekable waveform for playback
pub struct SeekableWaveform {
    drawing_area: gtk::DrawingArea,
    samples: Rc<RefCell<Vec<f32>>>,
    position: Rc<RefCell<f64>>,
    duration: Rc<RefCell<f64>>,
    on_seek: Rc<RefCell<Option<Box<dyn Fn(f64)>>>>,
}

impl SeekableWaveform {
    pub fn new() -> Self {
        let drawing_area = gtk::DrawingArea::builder()
            .content_width(400)
            .content_height(60)
            .hexpand(true)
            .build();

        let samples = Rc::new(RefCell::new(Vec::new()));
        let position = Rc::new(RefCell::new(0.0));
        let duration = Rc::new(RefCell::new(1.0));
        let on_seek: Rc<RefCell<Option<Box<dyn Fn(f64)>>>> = Rc::new(RefCell::new(None));

        // Drawing
        let samples_for_draw = samples.clone();
        let position_for_draw = position.clone();

        drawing_area.set_draw_func(move |_area, cr, width, height| {
            draw_seekable_waveform(
                cr,
                width,
                height,
                &samples_for_draw.borrow(),
                *position_for_draw.borrow(),
            );
        });

        // Click handling for seeking
        let gesture = gtk::GestureClick::new();
        let position_for_click = position.clone();
        let duration_for_click = duration.clone();
        let on_seek_for_click = on_seek.clone();
        let drawing_area_for_click = drawing_area.clone();

        gesture.connect_pressed(move |_, _, x, _| {
            let width = drawing_area_for_click.width() as f64;
            let seek_position = (x / width).clamp(0.0, 1.0);
            let seek_time = seek_position * *duration_for_click.borrow();

            *position_for_click.borrow_mut() = seek_position;
            drawing_area_for_click.queue_draw();

            if let Some(ref callback) = *on_seek_for_click.borrow() {
                callback(seek_time);
            }
        });

        drawing_area.add_controller(gesture);

        Self {
            drawing_area,
            samples,
            position,
            duration,
            on_seek,
        }
    }

    pub fn widget(&self) -> &gtk::DrawingArea {
        &self.drawing_area
    }

    pub fn set_samples(&self, new_samples: Vec<f32>) {
        *self.samples.borrow_mut() = new_samples;
        self.drawing_area.queue_draw();
    }

    pub fn set_position(&self, pos: f64) {
        let dur = *self.duration.borrow();
        *self.position.borrow_mut() = if dur > 0.0 { pos / dur } else { 0.0 };
        self.drawing_area.queue_draw();
    }

    pub fn set_duration(&self, dur: f64) {
        *self.duration.borrow_mut() = dur;
    }

    pub fn on_seek<F: Fn(f64) + 'static>(&self, callback: F) {
        *self.on_seek.borrow_mut() = Some(Box::new(callback));
    }
}

fn draw_seekable_waveform(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    samples: &[f32],
    position: f64,
) {
    let width = width as f64;
    let height = height as f64;
    let center_y = height / 2.0;

    // Background
    cr.set_source_rgb(0.15, 0.15, 0.18);
    let _ = cr.paint();

    if !samples.is_empty() {
        let bar_width = (width / samples.len() as f64).max(1.0);

        for (i, &sample) in samples.iter().enumerate() {
            let x = i as f64 * (width / samples.len() as f64);
            let bar_height = (sample * center_y * 0.85).max(1.0);

            // Color: played portion vs unplayed
            let progress = i as f64 / samples.len() as f64;
            if progress < position {
                cr.set_source_rgb(0.4, 0.6, 0.95); // Played
            } else {
                cr.set_source_rgb(0.35, 0.35, 0.4); // Unplayed
            }

            cr.rectangle(x, center_y - bar_height, bar_width - 0.5, bar_height * 2.0);
            let _ = cr.fill();
        }

        // Position indicator
        let pos_x = position * width;
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(2.0);
        cr.move_to(pos_x, 0.0);
        cr.line_to(pos_x, height);
        let _ = cr.stroke();
    }
}
