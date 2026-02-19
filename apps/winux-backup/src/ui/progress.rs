//! Progress widget for backup/restore operations

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;

use crate::backends::{BackupPhase, BackupProgress};

/// Widget to display backup/restore progress
pub struct BackupProgressWidget {
    widget: Box,
    title_label: Label,
    status_label: Label,
    file_label: Label,
    progress_bar: ProgressBar,
    speed_label: Label,
    eta_label: Label,
}

impl BackupProgressWidget {
    /// Create a new progress widget
    pub fn new() -> Self {
        let widget = Box::new(Orientation::Vertical, 12);
        widget.set_margin_top(24);
        widget.set_margin_bottom(24);
        widget.set_margin_start(24);
        widget.set_margin_end(24);
        widget.add_css_class("card");

        // Title
        let title_label = Label::new(Some("Backup em Progresso"));
        title_label.add_css_class("title-2");
        widget.append(&title_label);

        // Status
        let status_label = Label::new(Some("Preparando..."));
        status_label.add_css_class("heading");
        widget.append(&status_label);

        // Current file
        let file_label = Label::new(None);
        file_label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
        file_label.add_css_class("dim-label");
        file_label.add_css_class("caption");
        widget.append(&file_label);

        // Progress bar
        let progress_bar = ProgressBar::new();
        progress_bar.set_margin_top(8);
        progress_bar.set_margin_bottom(8);
        progress_bar.set_show_text(true);
        widget.append(&progress_bar);

        // Stats row
        let stats_box = Box::new(Orientation::Horizontal, 12);
        stats_box.set_halign(gtk4::Align::Center);

        let speed_label = Label::new(Some("0 MB/s"));
        speed_label.add_css_class("numeric");
        stats_box.append(&speed_label);

        let separator = Label::new(Some("|"));
        separator.add_css_class("dim-label");
        stats_box.append(&separator);

        let eta_label = Label::new(Some("Calculando..."));
        stats_box.append(&eta_label);

        widget.append(&stats_box);

        Self {
            widget,
            title_label,
            status_label,
            file_label,
            progress_bar,
            speed_label,
            eta_label,
        }
    }

    /// Update the progress display
    pub fn update(&self, progress: &BackupProgress) {
        // Update status based on phase
        let status = match progress.phase {
            BackupPhase::Scanning => "Escaneando arquivos...",
            BackupPhase::Backing => "Fazendo backup...",
            BackupPhase::Compressing => "Comprimindo...",
            BackupPhase::Encrypting => "Criptografando...",
            BackupPhase::Verifying => "Verificando...",
            BackupPhase::Cleaning => "Limpando...",
            BackupPhase::Complete => "Concluido!",
            BackupPhase::Failed => "Falhou!",
        };
        self.status_label.set_text(status);

        // Update current file
        self.file_label.set_text(&progress.current_file);

        // Update progress bar
        if progress.bytes_total > 0 {
            let fraction = progress.bytes_processed as f64 / progress.bytes_total as f64;
            self.progress_bar.set_fraction(fraction);
            self.progress_bar.set_text(Some(&format!(
                "{} / {} ({:.1}%)",
                format_size(progress.bytes_processed),
                format_size(progress.bytes_total),
                fraction * 100.0
            )));
        } else if progress.files_total > 0 {
            let fraction = progress.files_processed as f64 / progress.files_total as f64;
            self.progress_bar.set_fraction(fraction);
            self.progress_bar.set_text(Some(&format!(
                "{} / {} arquivos ({:.1}%)",
                progress.files_processed,
                progress.files_total,
                fraction * 100.0
            )));
        } else {
            self.progress_bar.pulse();
        }

        // Update speed
        let speed_str = format!("{}/s", format_size(progress.speed_bytes_per_sec));
        self.speed_label.set_text(&speed_str);

        // Update ETA
        let eta_str = match progress.eta_seconds {
            Some(0) => "Quase pronto...".to_string(),
            Some(secs) => format_duration(secs),
            None => "Calculando...".to_string(),
        };
        self.eta_label.set_text(&eta_str);
    }

    /// Set the title (e.g., "Backup" or "Restauracao")
    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.widget
    }

    /// Reset to initial state
    pub fn reset(&self) {
        self.status_label.set_text("Preparando...");
        self.file_label.set_text("");
        self.progress_bar.set_fraction(0.0);
        self.progress_bar.set_text(None);
        self.speed_label.set_text("0 MB/s");
        self.eta_label.set_text("Calculando...");
    }

    /// Mark as complete
    pub fn mark_complete(&self) {
        self.status_label.set_text("Concluido!");
        self.progress_bar.set_fraction(1.0);
        self.eta_label.set_text("Pronto");
    }

    /// Mark as failed
    pub fn mark_failed(&self, error: &str) {
        self.status_label.set_text("Falhou!");
        self.status_label.add_css_class("error");
        self.file_label.set_text(error);
    }
}

impl Default for BackupProgressWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes to human readable string
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format seconds to human readable duration
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{} segundos", seconds)
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        if secs > 0 {
            format!("{} min {} seg", mins, secs)
        } else {
            format!("{} minutos", mins)
        }
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins > 0 {
            format!("{} h {} min", hours, mins)
        } else {
            format!("{} horas", hours)
        }
    }
}

/// Create a compact progress indicator for lists
pub fn create_compact_progress(label: &str, fraction: f64) -> Box {
    let container = Box::new(Orientation::Horizontal, 8);

    let label_widget = Label::new(Some(label));
    label_widget.set_hexpand(true);
    label_widget.set_halign(gtk4::Align::Start);
    container.append(&label_widget);

    let progress = ProgressBar::new();
    progress.set_fraction(fraction);
    progress.set_size_request(100, -1);
    container.append(&progress);

    let percent_label = Label::new(Some(&format!("{:.0}%", fraction * 100.0)));
    percent_label.add_css_class("numeric");
    percent_label.set_width_chars(4);
    container.append(&percent_label);

    container
}
