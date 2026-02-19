//! UI components module

pub mod controls;
pub mod recording_row;
pub mod waveform;

pub use controls::RecordingControls;
pub use recording_row::RecordingRow;
pub use waveform::WaveformView;

use gtk4 as gtk;
use gtk::prelude::*;
use libadwaita as adw;

/// Recording list widget
pub struct RecordingList {
    widget: gtk::ListBox,
    state: std::sync::Arc<parking_lot::RwLock<crate::AppState>>,
    window: adw::ApplicationWindow,
}

impl RecordingList {
    pub fn new(
        state: std::sync::Arc<parking_lot::RwLock<crate::AppState>>,
        window: adw::ApplicationWindow,
    ) -> Self {
        let widget = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        let list = Self { widget, state, window };
        list.refresh();
        list
    }

    pub fn widget(&self) -> &gtk::ListBox {
        &self.widget
    }

    /// Refresh the list from state
    pub fn refresh(&self) {
        // Clear existing rows
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }

        // Add rows for each recording
        let recordings = self.state.read().recordings.clone();

        if recordings.is_empty() {
            let empty_label = gtk::Label::builder()
                .label("No recordings yet.\nPress the record button to start.")
                .justify(gtk::Justification::Center)
                .margin_top(24)
                .margin_bottom(24)
                .css_classes(["dim-label"])
                .build();
            self.widget.append(&empty_label);
        } else {
            for (index, recording) in recordings.iter().enumerate() {
                let row = RecordingRow::new(
                    recording.clone(),
                    index,
                    self.state.clone(),
                    self.window.clone(),
                );
                self.widget.append(row.widget());
            }
        }
    }
}

/// Show a toast notification
pub fn show_toast(window: &adw::ApplicationWindow, message: &str) {
    // Find or create an overlay for toasts
    if let Some(content) = window.content() {
        if let Some(overlay) = content.first_child() {
            if let Ok(toast_overlay) = overlay.downcast::<adw::ToastOverlay>() {
                let toast = adw::Toast::new(message);
                toast_overlay.add_toast(toast);
            }
        }
    }
}

/// Format seconds to MM:SS or HH:MM:SS string
pub fn format_time(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}
