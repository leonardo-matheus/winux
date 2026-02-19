//! Recording list item widget

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use libadwaita as adw;
use adw::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;

use crate::AppState;
use crate::recording::Recording;

/// A row in the recordings list
pub struct RecordingRow {
    widget: adw::ActionRow,
    recording: Recording,
    index: usize,
    state: Arc<RwLock<AppState>>,
}

impl RecordingRow {
    pub fn new(
        recording: Recording,
        index: usize,
        state: Arc<RwLock<AppState>>,
        window: adw::ApplicationWindow,
    ) -> Self {
        let widget = adw::ActionRow::builder()
            .title(&recording.metadata.title)
            .subtitle(&format!(
                "{} - {} - {}",
                recording.metadata.duration_string(),
                recording.metadata.format.label(),
                recording.metadata.size_string()
            ))
            .activatable(true)
            .build();

        // Play button
        let play_button = gtk::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .valign(gtk::Align::Center)
            .css_classes(["flat", "circular"])
            .tooltip_text("Play")
            .build();

        let recording_path = recording.metadata.path.clone();
        let state_for_play = state.clone();
        let window_for_play = window.clone();
        let recording_for_play = recording.clone();

        play_button.connect_clicked(move |_| {
            show_player_dialog(&window_for_play, &recording_for_play, &state_for_play);
        });

        // Menu button
        let menu_button = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .valign(gtk::Align::Center)
            .css_classes(["flat", "circular"])
            .tooltip_text("More Options")
            .build();

        let menu = gio::Menu::new();
        menu.append(Some("Rename"), Some("recording.rename"));
        menu.append(Some("Export"), Some("recording.export"));
        menu.append(Some("Open Folder"), Some("recording.open-folder"));
        menu.append(Some("Delete"), Some("recording.delete"));
        menu_button.set_menu_model(Some(&menu));

        // Create action group
        let actions = gio::SimpleActionGroup::new();

        // Rename action
        let rename_action = gio::SimpleAction::new("rename", None);
        let state_for_rename = state.clone();
        let window_for_rename = window.clone();
        let index_for_rename = index;

        rename_action.connect_activate(move |_, _| {
            show_rename_dialog(&window_for_rename, &state_for_rename, index_for_rename);
        });
        actions.add_action(&rename_action);

        // Export action
        let export_action = gio::SimpleAction::new("export", None);
        let state_for_export = state.clone();
        let window_for_export = window.clone();
        let index_for_export = index;

        export_action.connect_activate(move |_, _| {
            show_export_dialog(&window_for_export, &state_for_export, index_for_export);
        });
        actions.add_action(&export_action);

        // Open folder action
        let open_folder_action = gio::SimpleAction::new("open-folder", None);
        let path_for_folder = recording.metadata.path.clone();

        open_folder_action.connect_activate(move |_, _| {
            if let Some(parent) = path_for_folder.parent() {
                let _ = open::that(parent);
            }
        });
        actions.add_action(&open_folder_action);

        // Delete action
        let delete_action = gio::SimpleAction::new("delete", None);
        let state_for_delete = state.clone();
        let window_for_delete = window.clone();
        let index_for_delete = index;

        delete_action.connect_activate(move |_, _| {
            show_delete_dialog(&window_for_delete, &state_for_delete, index_for_delete);
        });
        actions.add_action(&delete_action);

        widget.insert_action_group("recording", Some(&actions));

        // Format icon based on type
        let format_icon = match recording.metadata.format {
            crate::audio::AudioFormat::Wav => "audio-x-generic-symbolic",
            crate::audio::AudioFormat::Mp3 => "audio-x-generic-symbolic",
            crate::audio::AudioFormat::Ogg => "audio-x-generic-symbolic",
            crate::audio::AudioFormat::Flac => "audio-x-generic-symbolic",
        };

        widget.add_prefix(&gtk::Image::from_icon_name(format_icon));
        widget.add_suffix(&play_button);
        widget.add_suffix(&menu_button);

        // Click to play
        let recording_for_activate = recording.clone();
        let state_for_activate = state.clone();
        let window_for_activate = window.clone();

        widget.connect_activated(move |_| {
            show_player_dialog(&window_for_activate, &recording_for_activate, &state_for_activate);
        });

        Self {
            widget,
            recording,
            index,
            state,
        }
    }

    pub fn widget(&self) -> &adw::ActionRow {
        &self.widget
    }
}

/// Show player dialog for a recording
fn show_player_dialog(
    window: &adw::ApplicationWindow,
    recording: &Recording,
    state: &Arc<RwLock<AppState>>,
) {
    let dialog = adw::Dialog::builder()
        .title(&recording.metadata.title)
        .content_width(400)
        .content_height(300)
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(24)
        .build();

    // Header
    let header = adw::HeaderBar::builder()
        .show_end_title_buttons(true)
        .build();

    // Waveform
    let waveform = super::waveform::SeekableWaveform::new();

    // Load waveform data
    let mut rec_clone = recording.clone();
    if rec_clone.load_waveform().is_ok() {
        if let Some(ref samples) = rec_clone.waveform {
            waveform.set_samples(samples.clone());
        }
    }
    waveform.set_duration(recording.metadata.duration);

    // Time labels
    let time_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let position_label = gtk::Label::builder()
        .label("00:00")
        .halign(gtk::Align::Start)
        .hexpand(true)
        .css_classes(["caption"])
        .build();

    let duration_label = gtk::Label::builder()
        .label(&recording.metadata.duration_string())
        .halign(gtk::Align::End)
        .css_classes(["caption"])
        .build();

    time_box.append(&position_label);
    time_box.append(&duration_label);

    // Controls
    let controls_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Center)
        .build();

    // Rewind 10s button
    let rewind_button = gtk::Button::builder()
        .icon_name("media-seek-backward-symbolic")
        .css_classes(["circular", "flat"])
        .tooltip_text("Rewind 10 seconds")
        .build();

    // Play/Pause button
    let play_button = gtk::Button::builder()
        .icon_name("media-playback-start-symbolic")
        .css_classes(["circular", "suggested-action"])
        .width_request(48)
        .height_request(48)
        .build();

    // Forward 10s button
    let forward_button = gtk::Button::builder()
        .icon_name("media-seek-forward-symbolic")
        .css_classes(["circular", "flat"])
        .tooltip_text("Forward 10 seconds")
        .build();

    controls_box.append(&rewind_button);
    controls_box.append(&play_button);
    controls_box.append(&forward_button);

    // Speed control
    let speed_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Center)
        .margin_top(8)
        .build();

    let speed_label = gtk::Label::builder()
        .label("Speed:")
        .css_classes(["dim-label"])
        .build();

    let speed_scale = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .width_request(150)
        .build();
    speed_scale.set_range(0.5, 2.0);
    speed_scale.set_value(1.0);
    speed_scale.set_increments(0.25, 0.5);

    let speed_value_label = gtk::Label::builder()
        .label("1.0x")
        .width_chars(4)
        .build();

    speed_scale.connect_value_changed({
        let label = speed_value_label.clone();
        move |scale| {
            label.set_label(&format!("{:.1}x", scale.value()));
        }
    });

    speed_box.append(&speed_label);
    speed_box.append(&speed_scale);
    speed_box.append(&speed_value_label);

    // Info
    let info_label = gtk::Label::builder()
        .label(&format!(
            "{} | {} Hz | {} channel(s) | {}",
            recording.metadata.format.label(),
            recording.metadata.sample_rate,
            recording.metadata.channels,
            recording.metadata.size_string()
        ))
        .css_classes(["dim-label", "caption"])
        .margin_top(16)
        .build();

    content.append(&header);
    content.append(waveform.widget());
    content.append(&time_box);
    content.append(&controls_box);
    content.append(&speed_box);
    content.append(&info_label);

    dialog.set_child(Some(&content));

    // Play button handler
    let is_playing = Rc::new(RefCell::new(false));
    let path = recording.metadata.path.clone();
    let duration = recording.metadata.duration;

    let is_playing_for_click = is_playing.clone();
    let play_btn = play_button.clone();

    play_button.connect_clicked(move |_| {
        let playing = *is_playing_for_click.borrow();

        if playing {
            // Stop playback
            play_btn.set_icon_name("media-playback-start-symbolic");
            *is_playing_for_click.borrow_mut() = false;
            // In a real implementation, we would stop the audio playback here
        } else {
            // Start playback
            play_btn.set_icon_name("media-playback-pause-symbolic");
            *is_playing_for_click.borrow_mut() = true;

            // Play using system player (simple approach)
            // In a real implementation, we'd use gstreamer or similar
            let path_clone = path.clone();
            std::thread::spawn(move || {
                let _ = std::process::Command::new("pw-play")
                    .arg(&path_clone)
                    .spawn();
            });
        }
    });

    dialog.present(Some(window));
}

/// Show rename dialog
fn show_rename_dialog(
    window: &adw::ApplicationWindow,
    state: &Arc<RwLock<AppState>>,
    index: usize,
) {
    let recording = match state.read().recordings.get(index) {
        Some(r) => r.clone(),
        None => return,
    };

    let dialog = adw::AlertDialog::builder()
        .heading("Rename Recording")
        .close_response("cancel")
        .default_response("rename")
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("rename", "Rename");
    dialog.set_response_appearance("rename", adw::ResponseAppearance::Suggested);

    let entry = gtk::Entry::builder()
        .text(&recording.metadata.title)
        .margin_start(12)
        .margin_end(12)
        .build();

    dialog.set_extra_child(Some(&entry));

    let state_clone = state.clone();
    let entry_clone = entry.clone();

    dialog.connect_response(None, move |dialog, response| {
        if response == "rename" {
            let new_title = entry_clone.text().to_string();
            if !new_title.is_empty() && new_title != recording.metadata.title {
                let mut state = state_clone.write();
                if let Some(rec) = state.recordings.get_mut(index) {
                    let _ = rec.rename(&new_title);
                }
            }
        }
        dialog.close();
    });

    dialog.present(Some(window));
}

/// Show export dialog
fn show_export_dialog(
    window: &adw::ApplicationWindow,
    state: &Arc<RwLock<AppState>>,
    index: usize,
) {
    let recording = match state.read().recordings.get(index) {
        Some(r) => r.clone(),
        None => return,
    };

    let file_dialog = gtk::FileDialog::builder()
        .title("Export Recording")
        .modal(true)
        .build();

    let state_clone = state.clone();
    let window_clone = window.clone();

    file_dialog.select_folder(Some(window), None::<&gio::Cancellable>, move |result| {
        if let Ok(folder) = result {
            if let Some(path) = folder.path() {
                if let Ok(exported_path) = recording.export(&path) {
                    tracing::info!("Exported to: {:?}", exported_path);
                    // Show toast
                    let toast = adw::Toast::new("Recording exported successfully");
                    if let Some(content) = window_clone.content() {
                        if let Some(overlay) = content.first_child() {
                            if let Ok(toast_overlay) = overlay.downcast::<adw::ToastOverlay>() {
                                toast_overlay.add_toast(toast);
                            }
                        }
                    }
                }
            }
        }
    });
}

/// Show delete confirmation dialog
fn show_delete_dialog(
    window: &adw::ApplicationWindow,
    state: &Arc<RwLock<AppState>>,
    index: usize,
) {
    let recording = match state.read().recordings.get(index) {
        Some(r) => r.clone(),
        None => return,
    };

    let dialog = adw::AlertDialog::builder()
        .heading("Delete Recording?")
        .body(&format!(
            "\"{}\" will be moved to the trash.",
            recording.metadata.title
        ))
        .close_response("cancel")
        .default_response("cancel")
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);

    let state_clone = state.clone();

    dialog.connect_response(None, move |dialog, response| {
        if response == "delete" {
            let mut state = state_clone.write();
            if let Some(rec) = state.recordings.get(index) {
                let _ = rec.delete();
            }
            state.recordings.remove(index);
        }
        dialog.close();
    });

    dialog.present(Some(window));
}
