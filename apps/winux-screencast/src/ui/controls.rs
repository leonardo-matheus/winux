//! Recording controls widget
//!
//! Provides start/pause/stop controls for screen recording.

use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::recording::{self, RecordingState, format_duration, format_file_size};

/// Recording controls widget
pub struct RecordingControls {
    widget: gtk::Box,
    state: Rc<RefCell<AppState>>,
    start_button: gtk::Button,
    pause_button: gtk::Button,
    stop_button: gtk::Button,
    timer_label: gtk::Label,
    size_label: gtk::Label,
    status_label: gtk::Label,
}

impl RecordingControls {
    /// Create new recording controls
    pub fn new(state: &Rc<RefCell<AppState>>) -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(16)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        // Status display
        let status_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .halign(gtk::Align::Center)
            .build();

        let timer_label = gtk::Label::builder()
            .label("00:00")
            .css_classes(["title-1", "numeric"])
            .build();

        let status_label = gtk::Label::builder()
            .label("Ready to record")
            .css_classes(["dim-label"])
            .build();

        let size_label = gtk::Label::builder()
            .label("")
            .css_classes(["caption", "dim-label"])
            .build();

        status_box.append(&timer_label);
        status_box.append(&status_label);
        status_box.append(&size_label);

        widget.append(&status_box);

        // Control buttons
        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::Center)
            .margin_top(16)
            .build();

        // Start/Record button
        let start_button = gtk::Button::builder()
            .icon_name("media-record-symbolic")
            .css_classes(["suggested-action", "circular", "large"])
            .tooltip_text("Start Recording (Ctrl+Alt+R)")
            .width_request(64)
            .height_request(64)
            .build();

        // Pause button
        let pause_button = gtk::Button::builder()
            .icon_name("media-playback-pause-symbolic")
            .css_classes(["circular"])
            .tooltip_text("Pause Recording (Ctrl+Alt+P)")
            .sensitive(false)
            .width_request(48)
            .height_request(48)
            .build();

        // Stop button
        let stop_button = gtk::Button::builder()
            .icon_name("media-playback-stop-symbolic")
            .css_classes(["destructive-action", "circular"])
            .tooltip_text("Stop Recording")
            .sensitive(false)
            .width_request(48)
            .height_request(48)
            .build();

        button_box.append(&pause_button);
        button_box.append(&start_button);
        button_box.append(&stop_button);

        widget.append(&button_box);

        // Output location indicator
        let output_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Center)
            .margin_top(16)
            .build();

        let folder_icon = gtk::Image::builder()
            .icon_name("folder-videos-symbolic")
            .build();

        let output_label = gtk::Label::builder()
            .label(&state.borrow().output_directory.display().to_string())
            .css_classes(["dim-label", "caption"])
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .max_width_chars(40)
            .build();

        let open_folder_btn = gtk::Button::builder()
            .icon_name("folder-open-symbolic")
            .css_classes(["flat", "circular"])
            .tooltip_text("Open folder")
            .build();

        let output_dir = state.borrow().output_directory.clone();
        open_folder_btn.connect_clicked(move |_| {
            let _ = open::that(&output_dir);
        });

        output_box.append(&folder_icon);
        output_box.append(&output_label);
        output_box.append(&open_folder_btn);

        widget.append(&output_box);

        let controls = Self {
            widget,
            state: state.clone(),
            start_button,
            pause_button,
            stop_button,
            timer_label,
            size_label,
            status_label,
        };

        controls.connect_signals();
        controls
    }

    /// Get the root widget
    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }

    fn connect_signals(&self) {
        // Start button
        let state = self.state.clone();
        let pause_btn = self.pause_button.clone();
        let stop_btn = self.stop_button.clone();
        let start_btn = self.start_button.clone();
        let timer = self.timer_label.clone();
        let status = self.status_label.clone();

        self.start_button.connect_clicked(move |btn| {
            let current_state = state.borrow().recording_state;

            if current_state.can_start() {
                // Start recording
                btn.set_icon_name("media-record-symbolic");
                btn.set_sensitive(false);
                pause_btn.set_sensitive(true);
                stop_btn.set_sensitive(true);
                status.set_label("Starting...");

                // This would actually start the recording
                recording::start_recording(
                    &btn.root().and_downcast::<gtk::ApplicationWindow>()
                        .unwrap()
                        .application()
                        .unwrap(),
                    &state,
                );

                // Start timer update
                let state_clone = state.clone();
                let timer_clone = timer.clone();
                let status_clone = status.clone();
                let btn_clone = btn.clone();
                let pause_btn_clone = pause_btn.clone();
                let stop_btn_clone = stop_btn.clone();

                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    let rec_state = state_clone.borrow().recording_state;

                    match rec_state {
                        RecordingState::Recording => {
                            let duration = state_clone.borrow().recording_duration;
                            timer_clone.set_label(&format_duration(duration));
                            status_clone.set_label("Recording");
                            glib::ControlFlow::Continue
                        }
                        RecordingState::Paused => {
                            status_clone.set_label("Paused");
                            glib::ControlFlow::Continue
                        }
                        RecordingState::Completed | RecordingState::Failed | RecordingState::Idle => {
                            // Reset UI
                            btn_clone.set_sensitive(true);
                            pause_btn_clone.set_sensitive(false);
                            stop_btn_clone.set_sensitive(false);

                            if rec_state == RecordingState::Completed {
                                status_clone.set_label("Recording saved");
                            } else if rec_state == RecordingState::Failed {
                                status_clone.set_label("Recording failed");
                            } else {
                                status_clone.set_label("Ready to record");
                            }

                            glib::ControlFlow::Break
                        }
                        _ => glib::ControlFlow::Continue,
                    }
                });
            }
        });

        // Pause button
        let state = self.state.clone();
        let pause_btn = self.pause_button.clone();
        let status = self.status_label.clone();

        self.pause_button.connect_clicked(move |btn| {
            let current_state = state.borrow().recording_state;

            if current_state == RecordingState::Recording {
                // Pause
                recording::pause_recording(&state);
                btn.set_icon_name("media-playback-start-symbolic");
                btn.set_tooltip_text(Some("Resume Recording"));
                status.set_label("Paused");
            } else if current_state == RecordingState::Paused {
                // Resume
                recording::resume_recording(&state);
                btn.set_icon_name("media-playback-pause-symbolic");
                btn.set_tooltip_text(Some("Pause Recording"));
                status.set_label("Recording");
            }
        });

        // Stop button
        let state = self.state.clone();
        let start_btn = self.start_button.clone();
        let pause_btn = self.pause_button.clone();
        let stop_btn = self.stop_button.clone();
        let status = self.status_label.clone();
        let timer = self.timer_label.clone();

        self.stop_button.connect_clicked(move |_| {
            recording::stop_recording(&state);

            // Reset UI
            start_btn.set_sensitive(true);
            start_btn.set_icon_name("media-record-symbolic");
            pause_btn.set_sensitive(false);
            pause_btn.set_icon_name("media-playback-pause-symbolic");
            stop_btn.set_sensitive(false);

            status.set_label("Saving...");
        });
    }

    /// Update the display based on current state
    pub fn update(&self) {
        let state = self.state.borrow();
        let recording_state = state.recording_state;

        // Update button states
        self.start_button.set_sensitive(recording_state.can_start());
        self.pause_button.set_sensitive(recording_state.can_pause() || recording_state == RecordingState::Paused);
        self.stop_button.set_sensitive(recording_state.can_stop());

        // Update labels
        self.status_label.set_label(recording_state.label());
        self.timer_label.set_label(&format_duration(state.recording_duration));

        // Update pause button icon
        if recording_state == RecordingState::Paused {
            self.pause_button.set_icon_name("media-playback-start-symbolic");
        } else {
            self.pause_button.set_icon_name("media-playback-pause-symbolic");
        }

        // Update start button appearance during recording
        if recording_state == RecordingState::Recording {
            self.start_button.remove_css_class("suggested-action");
            self.start_button.add_css_class("recording");
        } else {
            self.start_button.remove_css_class("recording");
            self.start_button.add_css_class("suggested-action");
        }
    }
}

/// Floating recording indicator widget
/// Shown during recording when main window is minimized
pub struct FloatingIndicator {
    window: gtk::Window,
    timer_label: gtk::Label,
    pause_button: gtk::Button,
    stop_button: gtk::Button,
}

impl FloatingIndicator {
    pub fn new(app: &gtk::Application) -> Self {
        let window = gtk::Window::builder()
            .application(app)
            .title("Recording")
            .decorated(false)
            .resizable(false)
            .deletable(false)
            .default_width(200)
            .default_height(50)
            .build();

        // Make window stay on top
        window.set_keep_above(true);

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Recording indicator
        let indicator = gtk::Box::builder()
            .css_classes(["recording-indicator"])
            .width_request(12)
            .height_request(12)
            .build();

        // Timer
        let timer_label = gtk::Label::builder()
            .label("00:00")
            .css_classes(["numeric"])
            .hexpand(true)
            .build();

        // Pause button
        let pause_button = gtk::Button::builder()
            .icon_name("media-playback-pause-symbolic")
            .css_classes(["flat", "circular"])
            .build();

        // Stop button
        let stop_button = gtk::Button::builder()
            .icon_name("media-playback-stop-symbolic")
            .css_classes(["flat", "circular", "destructive-action"])
            .build();

        content.append(&indicator);
        content.append(&timer_label);
        content.append(&pause_button);
        content.append(&stop_button);

        window.set_child(Some(&content));

        Self {
            window,
            timer_label,
            pause_button,
            stop_button,
        }
    }

    pub fn show(&self) {
        self.window.present();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn update_time(&self, seconds: f64) {
        self.timer_label.set_label(&format_duration(seconds));
    }

    pub fn set_paused(&self, paused: bool) {
        if paused {
            self.pause_button.set_icon_name("media-playback-start-symbolic");
        } else {
            self.pause_button.set_icon_name("media-playback-pause-symbolic");
        }
    }
}

/// Custom CSS for recording controls
pub fn get_custom_css() -> &'static str {
    r#"
    .recording-indicator {
        background-color: @error_color;
        border-radius: 50%;
        animation: pulse 1s ease-in-out infinite;
    }

    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
    }

    .large {
        min-width: 64px;
        min-height: 64px;
    }

    button.recording {
        background-color: @error_color;
        color: white;
    }

    button.recording:hover {
        background-color: shade(@error_color, 1.1);
    }
    "#
}
