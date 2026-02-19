//! Sync status widget

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;

use crate::sync::SyncStatus;

/// Sync status widget
pub struct SyncStatusWidget {
    widget: Box,
    status_icon: Image,
    status_label: Label,
    progress_bar: ProgressBar,
    file_label: Label,
    speed_label: Label,
    pause_button: Button,
}

impl SyncStatusWidget {
    /// Create a new sync status widget
    pub fn new() -> Self {
        let widget = Box::new(Orientation::Vertical, 8);
        widget.set_margin_start(16);
        widget.set_margin_end(16);
        widget.set_margin_top(8);
        widget.set_margin_bottom(8);

        // Header with icon and status
        let header = Box::new(Orientation::Horizontal, 8);

        let status_icon = Image::from_icon_name("emblem-ok-symbolic");
        status_icon.set_pixel_size(24);
        header.append(&status_icon);

        let status_label = Label::new(Some("Sincronizado"));
        status_label.add_css_class("heading");
        status_label.set_hexpand(true);
        status_label.set_halign(gtk4::Align::Start);
        header.append(&status_label);

        let pause_button = Button::from_icon_name("media-playback-pause-symbolic");
        pause_button.add_css_class("flat");
        pause_button.set_tooltip_text(Some("Pausar sincronizacao"));
        pause_button.set_visible(false);
        header.append(&pause_button);

        widget.append(&header);

        // Progress bar
        let progress_bar = ProgressBar::new();
        progress_bar.set_visible(false);
        widget.append(&progress_bar);

        // Current file label
        let file_label = Label::new(None);
        file_label.set_halign(gtk4::Align::Start);
        file_label.add_css_class("dim-label");
        file_label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
        file_label.set_visible(false);
        widget.append(&file_label);

        // Speed label
        let speed_label = Label::new(None);
        speed_label.set_halign(gtk4::Align::Start);
        speed_label.add_css_class("dim-label");
        speed_label.set_visible(false);
        widget.append(&speed_label);

        Self {
            widget,
            status_icon,
            status_label,
            progress_bar,
            file_label,
            speed_label,
            pause_button,
        }
    }

    /// Update the widget with new status
    pub fn update(&self, status: &SyncStatus) {
        if status.is_syncing {
            self.status_icon.set_icon_name(Some("emblem-synchronizing-symbolic"));
            self.status_label.set_text("Sincronizando...");
            self.progress_bar.set_visible(true);
            self.pause_button.set_visible(true);

            // Update progress
            if status.bytes_total > 0 {
                let fraction = status.bytes_transferred as f64 / status.bytes_total as f64;
                self.progress_bar.set_fraction(fraction);
            }

            // Update file name
            if let Some(file) = &status.current_file {
                self.file_label.set_text(file);
                self.file_label.set_visible(true);
            }

            // Update speed
            if status.speed > 0 {
                self.speed_label.set_text(&format!("{}/s", format_bytes(status.speed)));
                self.speed_label.set_visible(true);
            }
        } else if status.is_paused {
            self.status_icon.set_icon_name(Some("media-playback-pause-symbolic"));
            self.status_label.set_text("Sincronizacao pausada");
            self.progress_bar.set_visible(false);
            self.file_label.set_visible(false);
            self.speed_label.set_visible(false);
            self.pause_button.set_visible(true);
            self.pause_button.set_icon_name("media-playback-start-symbolic");
            self.pause_button.set_tooltip_text(Some("Retomar sincronizacao"));
        } else if status.error_count > 0 {
            self.status_icon.set_icon_name(Some("dialog-error-symbolic"));
            self.status_label.set_text(&format!("{} erros de sincronizacao", status.error_count));
            self.progress_bar.set_visible(false);
            self.file_label.set_visible(false);
            self.speed_label.set_visible(false);
            self.pause_button.set_visible(false);
        } else if status.conflict_count > 0 {
            self.status_icon.set_icon_name(Some("dialog-warning-symbolic"));
            self.status_label.set_text(&format!("{} conflitos pendentes", status.conflict_count));
            self.progress_bar.set_visible(false);
            self.file_label.set_visible(false);
            self.speed_label.set_visible(false);
            self.pause_button.set_visible(false);
        } else if status.pending_count > 0 {
            self.status_icon.set_icon_name(Some("emblem-synchronizing-symbolic"));
            self.status_label.set_text(&format!("{} arquivos pendentes", status.pending_count));
            self.progress_bar.set_visible(false);
            self.file_label.set_visible(false);
            self.speed_label.set_visible(false);
            self.pause_button.set_visible(false);
        } else {
            self.status_icon.set_icon_name(Some("emblem-ok-symbolic"));
            self.status_label.set_text("Tudo sincronizado");
            self.progress_bar.set_visible(false);
            self.file_label.set_visible(false);
            self.speed_label.set_visible(false);
            self.pause_button.set_visible(false);

            // Show last sync time
            if let Some(last_sync) = status.last_sync {
                self.file_label.set_text(&format!("Ultima sincronizacao: {}", format_time(last_sync)));
                self.file_label.set_visible(true);
            }
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.widget
    }

    /// Connect pause button clicked
    pub fn connect_pause_clicked<F: Fn() + 'static>(&self, f: F) {
        self.pause_button.connect_clicked(move |_| f());
    }
}

impl Default for SyncStatusWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format time
fn format_time(time: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - time;

    if diff.num_minutes() < 1 {
        "agora mesmo".to_string()
    } else if diff.num_hours() < 1 {
        format!("ha {} minutos", diff.num_minutes())
    } else if diff.num_days() < 1 {
        format!("ha {} horas", diff.num_hours())
    } else {
        format!("ha {} dias", diff.num_days())
    }
}
