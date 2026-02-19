//! Recording controls (record, pause, stop, etc.)

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use libadwaita as adw;
use std::sync::Arc;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::recording::RecordingState;
use crate::audio::pipewire::{PipeWireRecorder, RecorderEvent};

/// Recording controls widget
pub struct RecordingControls {
    widget: gtk::Box,
    state: Arc<RwLock<AppState>>,
    recorder: Rc<RefCell<Option<PipeWireRecorder>>>,
    record_button: gtk::Button,
    pause_button: gtk::Button,
    stop_button: gtk::Button,
    marker_button: gtk::Button,
    duration_label: gtk::Label,
    level_bar: gtk::LevelBar,
}

impl RecordingControls {
    pub fn new(
        state: Arc<RwLock<AppState>>,
        duration_label: gtk::Label,
        level_bar: gtk::LevelBar,
    ) -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::Center)
            .margin_top(16)
            .margin_bottom(8)
            .build();

        // Marker button (add bookmark during recording)
        let marker_button = gtk::Button::builder()
            .icon_name("bookmark-new-symbolic")
            .tooltip_text("Add Marker")
            .css_classes(["circular"])
            .sensitive(false)
            .build();

        // Pause button
        let pause_button = gtk::Button::builder()
            .icon_name("media-playback-pause-symbolic")
            .tooltip_text("Pause")
            .css_classes(["circular"])
            .sensitive(false)
            .build();

        // Main record button (large, circular, red)
        let record_button = gtk::Button::builder()
            .icon_name("media-record-symbolic")
            .tooltip_text("Start Recording")
            .width_request(72)
            .height_request(72)
            .css_classes(["circular", "destructive-action", "record-button"])
            .build();

        // Stop button
        let stop_button = gtk::Button::builder()
            .icon_name("media-playback-stop-symbolic")
            .tooltip_text("Stop")
            .css_classes(["circular"])
            .sensitive(false)
            .build();

        // Discard button
        let discard_button = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Discard Recording")
            .css_classes(["circular"])
            .sensitive(false)
            .build();

        widget.append(&marker_button);
        widget.append(&pause_button);
        widget.append(&record_button);
        widget.append(&stop_button);
        widget.append(&discard_button);

        let recorder: Rc<RefCell<Option<PipeWireRecorder>>> = Rc::new(RefCell::new(None));

        let controls = Self {
            widget,
            state: state.clone(),
            recorder: recorder.clone(),
            record_button: record_button.clone(),
            pause_button: pause_button.clone(),
            stop_button: stop_button.clone(),
            marker_button: marker_button.clone(),
            duration_label: duration_label.clone(),
            level_bar: level_bar.clone(),
        };

        // Connect record button
        let state_for_record = state.clone();
        let recorder_for_record = recorder.clone();
        let record_btn = record_button.clone();
        let pause_btn = pause_button.clone();
        let stop_btn = stop_button.clone();
        let marker_btn = marker_button.clone();
        let discard_btn = discard_button.clone();
        let duration_lbl = duration_label.clone();
        let level_br = level_bar.clone();

        record_button.connect_clicked(move |_| {
            let current_state = state_for_record.read().recording_state;

            match current_state {
                RecordingState::Idle => {
                    // Start recording
                    let mut rec = PipeWireRecorder::new(state_for_record.clone());
                    let event_rx = rec.init();
                    rec.start();
                    *recorder_for_record.borrow_mut() = Some(rec);

                    state_for_record.write().recording_state = RecordingState::Recording;
                    state_for_record.write().duration = 0.0;
                    state_for_record.write().waveform_samples.clear();
                    state_for_record.write().markers.clear();

                    // Update button states
                    record_btn.set_sensitive(false);
                    pause_btn.set_sensitive(true);
                    stop_btn.set_sensitive(true);
                    marker_btn.set_sensitive(true);
                    discard_btn.set_sensitive(true);

                    // Handle events from recorder
                    let state_for_events = state_for_record.clone();
                    let record_btn_for_events = record_btn.clone();
                    let pause_btn_for_events = pause_btn.clone();
                    let stop_btn_for_events = stop_btn.clone();
                    let marker_btn_for_events = marker_btn.clone();
                    let discard_btn_for_events = discard_btn.clone();
                    let duration_lbl_for_events = duration_lbl.clone();
                    let level_br_for_events = level_br.clone();
                    let recorder_for_events = recorder_for_record.clone();

                    glib::spawn_future_local(async move {
                        let mut rx = event_rx;
                        while let Some(event) = rx.recv().await {
                            match event {
                                RecorderEvent::Level(level) => {
                                    level_br_for_events.set_value(level as f64);
                                }
                                RecorderEvent::Duration(duration) => {
                                    let hours = (duration / 3600.0) as u64;
                                    let mins = ((duration % 3600.0) / 60.0) as u64;
                                    let secs = (duration % 60.0) as u64;
                                    let time_str = if hours > 0 {
                                        format!("{:02}:{:02}:{:02}", hours, mins, secs)
                                    } else {
                                        format!("{:02}:{:02}", mins, secs)
                                    };
                                    duration_lbl_for_events.set_label(&time_str);
                                    state_for_events.write().duration = duration;
                                }
                                RecorderEvent::Waveform(samples) => {
                                    state_for_events.write().waveform_samples = samples;
                                }
                                RecorderEvent::MarkerAdded(timestamp) => {
                                    state_for_events.write().markers.push(timestamp);
                                }
                                RecorderEvent::Stopped(path) => {
                                    // Recording saved successfully
                                    tracing::info!("Recording saved: {:?}", path);

                                    // Add to recordings list
                                    if let Some(recording) = crate::recording::Recording::from_file(&path) {
                                        state_for_events.write().recordings.insert(0, recording);
                                    }

                                    // Reset UI
                                    state_for_events.write().recording_state = RecordingState::Idle;
                                    record_btn_for_events.set_sensitive(true);
                                    pause_btn_for_events.set_sensitive(false);
                                    stop_btn_for_events.set_sensitive(false);
                                    marker_btn_for_events.set_sensitive(false);
                                    discard_btn_for_events.set_sensitive(false);
                                    duration_lbl_for_events.set_label("00:00:00");
                                    level_br_for_events.set_value(0.0);

                                    *recorder_for_events.borrow_mut() = None;
                                    break;
                                }
                                RecorderEvent::Cancelled => {
                                    state_for_events.write().recording_state = RecordingState::Idle;
                                    record_btn_for_events.set_sensitive(true);
                                    pause_btn_for_events.set_sensitive(false);
                                    stop_btn_for_events.set_sensitive(false);
                                    marker_btn_for_events.set_sensitive(false);
                                    discard_btn_for_events.set_sensitive(false);
                                    duration_lbl_for_events.set_label("00:00:00");
                                    level_br_for_events.set_value(0.0);

                                    *recorder_for_events.borrow_mut() = None;
                                    break;
                                }
                                RecorderEvent::Error(err) => {
                                    tracing::error!("Recording error: {}", err);
                                    // Reset UI on error
                                    state_for_events.write().recording_state = RecordingState::Idle;
                                    record_btn_for_events.set_sensitive(true);
                                    pause_btn_for_events.set_sensitive(false);
                                    stop_btn_for_events.set_sensitive(false);
                                    marker_btn_for_events.set_sensitive(false);
                                    discard_btn_for_events.set_sensitive(false);

                                    *recorder_for_events.borrow_mut() = None;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });
                }
                _ => {}
            }
        });

        // Connect pause button
        let state_for_pause = state.clone();
        let recorder_for_pause = recorder.clone();
        let pause_btn_ref = pause_button.clone();

        pause_button.connect_clicked(move |btn| {
            if let Some(ref mut rec) = *recorder_for_pause.borrow_mut() {
                let current_state = state_for_pause.read().recording_state;

                if current_state == RecordingState::Recording {
                    rec.pause();
                    state_for_pause.write().recording_state = RecordingState::Paused;
                    btn.set_icon_name("media-playback-start-symbolic");
                    btn.set_tooltip_text(Some("Resume"));
                } else if current_state == RecordingState::Paused {
                    rec.resume();
                    state_for_pause.write().recording_state = RecordingState::Recording;
                    btn.set_icon_name("media-playback-pause-symbolic");
                    btn.set_tooltip_text(Some("Pause"));
                }
            }
        });

        // Connect stop button
        let recorder_for_stop = recorder.clone();

        stop_button.connect_clicked(move |_| {
            if let Some(ref mut rec) = *recorder_for_stop.borrow_mut() {
                rec.stop();
            }
        });

        // Connect marker button
        let recorder_for_marker = recorder.clone();

        marker_button.connect_clicked(move |_| {
            if let Some(ref rec) = *recorder_for_marker.borrow() {
                rec.add_marker();
            }
        });

        // Connect discard button
        let recorder_for_discard = recorder.clone();

        discard_button.connect_clicked(move |_| {
            if let Some(ref mut rec) = *recorder_for_discard.borrow_mut() {
                rec.cancel();
            }
        });

        controls
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }
}

/// Playback controls for recorded audio
pub struct PlaybackControls {
    widget: gtk::Box,
    play_button: gtk::Button,
    stop_button: gtk::Button,
    speed_button: gtk::MenuButton,
    position_label: gtk::Label,
    duration_label: gtk::Label,
    is_playing: Rc<RefCell<bool>>,
    playback_speed: Rc<RefCell<f64>>,
}

impl PlaybackControls {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Center)
            .build();

        let position_label = gtk::Label::builder()
            .label("00:00")
            .css_classes(["caption"])
            .build();

        let play_button = gtk::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .css_classes(["circular"])
            .build();

        let stop_button = gtk::Button::builder()
            .icon_name("media-playback-stop-symbolic")
            .css_classes(["circular"])
            .sensitive(false)
            .build();

        // Speed control
        let speed_button = gtk::MenuButton::builder()
            .label("1.0x")
            .tooltip_text("Playback Speed")
            .build();

        let speed_menu = gio::Menu::new();
        speed_menu.append(Some("0.5x"), Some("app.speed-0.5"));
        speed_menu.append(Some("0.75x"), Some("app.speed-0.75"));
        speed_menu.append(Some("1.0x"), Some("app.speed-1.0"));
        speed_menu.append(Some("1.25x"), Some("app.speed-1.25"));
        speed_menu.append(Some("1.5x"), Some("app.speed-1.5"));
        speed_menu.append(Some("2.0x"), Some("app.speed-2.0"));
        speed_button.set_menu_model(Some(&speed_menu));

        let duration_label = gtk::Label::builder()
            .label("00:00")
            .css_classes(["caption"])
            .build();

        widget.append(&position_label);
        widget.append(&play_button);
        widget.append(&stop_button);
        widget.append(&speed_button);
        widget.append(&duration_label);

        Self {
            widget,
            play_button,
            stop_button,
            speed_button,
            position_label,
            duration_label,
            is_playing: Rc::new(RefCell::new(false)),
            playback_speed: Rc::new(RefCell::new(1.0)),
        }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn set_duration(&self, duration: f64) {
        self.duration_label.set_label(&format_time(duration));
    }

    pub fn set_position(&self, position: f64) {
        self.position_label.set_label(&format_time(position));
    }

    pub fn on_play<F: Fn() + 'static>(&self, callback: F) {
        let cb = Rc::new(callback);
        let is_playing = self.is_playing.clone();
        let play_btn = self.play_button.clone();
        let stop_btn = self.stop_button.clone();

        self.play_button.connect_clicked(move |_| {
            let playing = *is_playing.borrow();
            if playing {
                // Pause
                play_btn.set_icon_name("media-playback-start-symbolic");
                *is_playing.borrow_mut() = false;
            } else {
                // Play
                play_btn.set_icon_name("media-playback-pause-symbolic");
                stop_btn.set_sensitive(true);
                *is_playing.borrow_mut() = true;
                cb();
            }
        });
    }

    pub fn on_stop<F: Fn() + 'static>(&self, callback: F) {
        let cb = Rc::new(callback);
        let is_playing = self.is_playing.clone();
        let play_btn = self.play_button.clone();
        let stop_btn = self.stop_button.clone();

        self.stop_button.connect_clicked(move |_| {
            play_btn.set_icon_name("media-playback-start-symbolic");
            stop_btn.set_sensitive(false);
            *is_playing.borrow_mut() = false;
            cb();
        });
    }

    pub fn set_speed(&self, speed: f64) {
        *self.playback_speed.borrow_mut() = speed;
        self.speed_button.set_label(&format!("{:.1}x", speed));
    }

    pub fn stop(&self) {
        self.play_button.set_icon_name("media-playback-start-symbolic");
        self.stop_button.set_sensitive(false);
        *self.is_playing.borrow_mut() = false;
    }
}

fn format_time(seconds: f64) -> String {
    let mins = (seconds / 60.0) as u64;
    let secs = (seconds % 60.0) as u64;
    format!("{:02}:{:02}", mins, secs)
}
