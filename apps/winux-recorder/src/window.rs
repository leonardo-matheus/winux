//! Main application window

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Application, Box, Button, Label, Orientation, ComboBoxText};
use libadwaita as adw;
use adw::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock;
use std::cell::RefCell;

use crate::AppState;
use crate::audio::AudioFormat;
use crate::recording::RecordingState;
use crate::ui::{WaveformView, RecordingControls, RecordingList};

/// Main recorder window
pub struct RecorderWindow {
    window: adw::ApplicationWindow,
    state: Arc<RwLock<AppState>>,
    waveform: RefCell<Option<WaveformView>>,
    controls: RefCell<Option<RecordingControls>>,
    recording_list: RefCell<Option<RecordingList>>,
    duration_label: RefCell<Option<Label>>,
    level_bar: RefCell<Option<gtk::LevelBar>>,
}

impl RecorderWindow {
    pub fn new(app: &Application, state: Arc<RwLock<AppState>>) -> adw::ApplicationWindow {
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Winux Recorder")
            .default_width(500)
            .default_height(700)
            .build();

        // Header bar
        let header = adw::HeaderBar::new();

        // Settings button
        let settings_button = Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Settings")
            .build();

        let state_clone = state.clone();
        let window_clone = window.clone();
        settings_button.connect_clicked(move |_| {
            show_settings_dialog(&window_clone, &state_clone);
        });
        header.pack_end(&settings_button);

        // Main content
        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        // Recording view (top half)
        let recording_view = create_recording_view(&state);
        recording_view.set_vexpand(true);

        // Recordings list (bottom half)
        let recordings_view = create_recordings_view(&state, &window);
        recordings_view.set_vexpand(true);

        main_box.append(&recording_view);
        main_box.append(&gtk::Separator::new(Orientation::Horizontal));
        main_box.append(&recordings_view);

        // Content with header
        let content_box = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        content_box.append(&header);
        content_box.append(&main_box);

        window.set_content(Some(&content_box));

        // Start update timer for UI
        let state_for_timer = state.clone();
        let window_weak = window.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            if let Some(_window) = window_weak.upgrade() {
                // Update UI elements based on state
                let _state = state_for_timer.read();
                // Updates would happen here via signals
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        window
    }
}

/// Create the recording view with waveform and controls
fn create_recording_view(state: &Arc<RwLock<AppState>>) -> Box {
    let view_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(12)
        .build();

    // Duration display
    let duration_label = Label::builder()
        .label("00:00:00")
        .css_classes(["title-1"])
        .build();

    // Waveform visualization
    let waveform = WaveformView::new(state.clone());
    waveform.widget().set_height_request(120);
    waveform.widget().set_hexpand(true);

    // Level meter
    let level_bar = gtk::LevelBar::builder()
        .min_value(0.0)
        .max_value(1.0)
        .value(0.0)
        .height_request(8)
        .margin_start(12)
        .margin_end(12)
        .build();
    level_bar.add_css_class("recording-level");

    // Recording controls
    let controls = RecordingControls::new(state.clone(), duration_label.clone(), level_bar.clone());

    // Format selector
    let format_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Center)
        .margin_top(8)
        .build();

    let format_label = Label::builder()
        .label("Format:")
        .css_classes(["dim-label"])
        .build();

    let format_combo = ComboBoxText::new();
    format_combo.append(Some("wav"), "WAV (Lossless)");
    format_combo.append(Some("mp3"), "MP3");
    format_combo.append(Some("ogg"), "OGG/Opus");
    format_combo.append(Some("flac"), "FLAC (Lossless)");

    let current_format = state.read().config.default_format;
    format_combo.set_active_id(Some(match current_format {
        AudioFormat::Wav => "wav",
        AudioFormat::Mp3 => "mp3",
        AudioFormat::Ogg => "ogg",
        AudioFormat::Flac => "flac",
    }));

    let state_clone = state.clone();
    format_combo.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            let format = match id.as_str() {
                "wav" => AudioFormat::Wav,
                "mp3" => AudioFormat::Mp3,
                "ogg" => AudioFormat::Ogg,
                "flac" => AudioFormat::Flac,
                _ => AudioFormat::Wav,
            };
            state_clone.write().config.default_format = format;
        }
    });

    format_box.append(&format_label);
    format_box.append(&format_combo);

    view_box.append(&duration_label);
    view_box.append(waveform.widget());
    view_box.append(&level_bar);
    view_box.append(controls.widget());
    view_box.append(&format_box);

    view_box
}

/// Create the recordings list view
fn create_recordings_view(state: &Arc<RwLock<AppState>>, window: &adw::ApplicationWindow) -> Box {
    let view_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    // Header
    let header_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let title = Label::builder()
        .label("Recordings")
        .css_classes(["title-3"])
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();

    let count_label = Label::builder()
        .label(&format!("{} recordings", state.read().recordings.len()))
        .css_classes(["dim-label"])
        .build();

    header_box.append(&title);
    header_box.append(&count_label);

    // Scrolled list
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list = RecordingList::new(state.clone(), window.clone());
    scrolled.set_child(Some(list.widget()));

    view_box.append(&header_box);
    view_box.append(&scrolled);

    view_box
}

/// Show settings dialog
fn show_settings_dialog(window: &adw::ApplicationWindow, state: &Arc<RwLock<AppState>>) {
    let dialog = adw::PreferencesWindow::builder()
        .title("Recorder Settings")
        .transient_for(window)
        .modal(true)
        .build();

    // Audio settings page
    let audio_page = adw::PreferencesPage::builder()
        .title("Audio")
        .icon_name("audio-input-microphone-symbolic")
        .build();

    // Input device group
    let input_group = adw::PreferencesGroup::builder()
        .title("Input Device")
        .description("Select the microphone to use for recording")
        .build();

    let device_row = adw::ComboRow::builder()
        .title("Microphone")
        .build();

    // Populate with available devices
    let devices = crate::audio::device::list_input_devices();
    let device_model = gtk::StringList::new(&[]);
    device_model.append("Default");
    for device in &devices {
        device_model.append(device);
    }
    device_row.set_model(Some(&device_model));

    // Set current device
    if let Some(ref current_device) = state.read().config.input_device {
        for (i, device) in devices.iter().enumerate() {
            if device == current_device {
                device_row.set_selected((i + 1) as u32);
                break;
            }
        }
    }

    let state_clone = state.clone();
    device_row.connect_selected_notify(move |row| {
        let idx = row.selected() as usize;
        if idx == 0 {
            state_clone.write().config.input_device = None;
        } else if let Some(device) = devices.get(idx - 1) {
            state_clone.write().config.input_device = Some(device.clone());
        }
    });

    input_group.add(&device_row);

    // Quality group
    let quality_group = adw::PreferencesGroup::builder()
        .title("Recording Quality")
        .build();

    // Quality preset
    let quality_row = adw::ComboRow::builder()
        .title("Quality Preset")
        .subtitle("Higher quality means larger file size")
        .build();

    let quality_model = gtk::StringList::new(&["Low", "Medium", "High"]);
    quality_row.set_model(Some(&quality_model));

    let current_quality = state.read().config.quality;
    quality_row.set_selected(match current_quality {
        crate::QualityPreset::Low => 0,
        crate::QualityPreset::Medium => 1,
        crate::QualityPreset::High => 2,
    });

    let state_clone = state.clone();
    quality_row.connect_selected_notify(move |row| {
        let quality = match row.selected() {
            0 => crate::QualityPreset::Low,
            1 => crate::QualityPreset::Medium,
            _ => crate::QualityPreset::High,
        };
        state_clone.write().config.quality = quality;
    });

    // Sample rate
    let sample_rate_row = adw::ComboRow::builder()
        .title("Sample Rate")
        .subtitle("Higher sample rate captures more detail")
        .build();

    let sample_rates = gtk::StringList::new(&["22050 Hz", "44100 Hz", "48000 Hz", "96000 Hz"]);
    sample_rate_row.set_model(Some(&sample_rates));

    let current_rate = state.read().config.sample_rate;
    sample_rate_row.set_selected(match current_rate {
        22050 => 0,
        44100 => 1,
        48000 => 2,
        96000 => 3,
        _ => 2,
    });

    let state_clone = state.clone();
    sample_rate_row.connect_selected_notify(move |row| {
        let rate = match row.selected() {
            0 => 22050,
            1 => 44100,
            2 => 48000,
            3 => 96000,
            _ => 48000,
        };
        state_clone.write().config.sample_rate = rate;
    });

    // Channels
    let channels_row = adw::ComboRow::builder()
        .title("Channels")
        .subtitle("Mono uses less storage, stereo captures spatial audio")
        .build();

    let channels_model = gtk::StringList::new(&["Mono", "Stereo"]);
    channels_row.set_model(Some(&channels_model));
    channels_row.set_selected(if state.read().config.channels == 1 { 0 } else { 1 });

    let state_clone = state.clone();
    channels_row.connect_selected_notify(move |row| {
        let channels = if row.selected() == 0 { 1 } else { 2 };
        state_clone.write().config.channels = channels;
    });

    quality_group.add(&quality_row);
    quality_group.add(&sample_rate_row);
    quality_group.add(&channels_row);

    // Storage group
    let storage_group = adw::PreferencesGroup::builder()
        .title("Storage")
        .build();

    let output_dir = state.read().config.output_dir.to_string_lossy().to_string();
    let folder_row = adw::ActionRow::builder()
        .title("Save Location")
        .subtitle(&output_dir)
        .build();

    let folder_button = Button::builder()
        .icon_name("folder-open-symbolic")
        .valign(gtk::Align::Center)
        .build();

    let state_clone = state.clone();
    let dialog_clone = dialog.clone();
    folder_button.connect_clicked(move |_| {
        let file_dialog = gtk::FileDialog::builder()
            .title("Select Save Location")
            .modal(true)
            .build();

        let state_clone2 = state_clone.clone();
        let dialog_clone2 = dialog_clone.clone();
        file_dialog.select_folder(Some(&dialog_clone), None::<&gio::Cancellable>, move |result| {
            if let Ok(folder) = result {
                if let Some(path) = folder.path() {
                    state_clone2.write().config.output_dir = path;
                    // Would need to update the subtitle here
                }
            }
        });
    });

    folder_row.add_suffix(&folder_button);
    storage_group.add(&folder_row);

    // Default format
    let format_row = adw::ComboRow::builder()
        .title("Default Format")
        .build();

    let format_model = gtk::StringList::new(&["WAV (Lossless)", "MP3", "OGG/Opus", "FLAC (Lossless)"]);
    format_row.set_model(Some(&format_model));

    let current_format = state.read().config.default_format;
    format_row.set_selected(match current_format {
        AudioFormat::Wav => 0,
        AudioFormat::Mp3 => 1,
        AudioFormat::Ogg => 2,
        AudioFormat::Flac => 3,
    });

    let state_clone = state.clone();
    format_row.connect_selected_notify(move |row| {
        let format = match row.selected() {
            0 => AudioFormat::Wav,
            1 => AudioFormat::Mp3,
            2 => AudioFormat::Ogg,
            3 => AudioFormat::Flac,
            _ => AudioFormat::Wav,
        };
        state_clone.write().config.default_format = format;
    });

    storage_group.add(&format_row);

    audio_page.add(&input_group);
    audio_page.add(&quality_group);
    audio_page.add(&storage_group);

    dialog.add(&audio_page);

    // Save config on close
    let state_clone = state.clone();
    dialog.connect_close_request(move |_| {
        state_clone.read().save_config();
        glib::Propagation::Proceed
    });

    dialog.present();
}
