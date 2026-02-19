//! File row widget

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;

use crate::providers::CloudFile;
use crate::sync::FileSyncStatus;

/// File row widget for displaying cloud files
pub struct FileRow {
    widget: adw::ActionRow,
}

impl FileRow {
    /// Create a new file row
    pub fn new(file: &CloudFile, sync_status: Option<FileSyncStatus>) -> Self {
        let row = adw::ActionRow::builder()
            .title(&file.name)
            .activatable(true)
            .build();

        // File icon based on type/mime
        let icon_name = Self::get_icon_for_file(file);
        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(32);
        row.add_prefix(&icon);

        // Subtitle with size and modification date
        let subtitle = if file.is_folder {
            "Pasta".to_string()
        } else {
            format!(
                "{} - Modificado: {}",
                format_bytes(file.size),
                format_date(file.modified_at)
            )
        };
        row.set_subtitle(&subtitle);

        // Sync status indicator
        if let Some(status) = sync_status {
            let status_icon = Self::get_status_icon(status);
            row.add_suffix(&status_icon);
        }

        // Share button
        let share_btn = Button::from_icon_name("emblem-shared-symbolic");
        share_btn.add_css_class("flat");
        share_btn.set_valign(gtk4::Align::Center);
        share_btn.set_tooltip_text(Some("Compartilhar"));
        row.add_suffix(&share_btn);

        // More options button
        let more_btn = Button::from_icon_name("view-more-symbolic");
        more_btn.add_css_class("flat");
        more_btn.set_valign(gtk4::Align::Center);
        more_btn.set_tooltip_text(Some("Mais opcoes"));
        row.add_suffix(&more_btn);

        // Navigation arrow for folders
        if file.is_folder {
            row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
        }

        Self { widget: row }
    }

    /// Create a file row with sync progress
    pub fn new_with_progress(file: &CloudFile, progress: f64) -> Self {
        let row = adw::ActionRow::builder()
            .title(&file.name)
            .build();

        let icon_name = Self::get_icon_for_file(file);
        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(32);
        row.add_prefix(&icon);

        // Subtitle with progress info
        row.set_subtitle(&format!(
            "{:.0}% - {} de {}",
            progress * 100.0,
            format_bytes((file.size as f64 * progress) as u64),
            format_bytes(file.size)
        ));

        // Progress bar
        let progress_bar = ProgressBar::new();
        progress_bar.set_fraction(progress);
        progress_bar.set_valign(gtk4::Align::Center);
        progress_bar.set_size_request(100, -1);
        row.add_suffix(&progress_bar);

        // Cancel button
        let cancel_btn = Button::from_icon_name("window-close-symbolic");
        cancel_btn.add_css_class("flat");
        cancel_btn.set_valign(gtk4::Align::Center);
        cancel_btn.set_tooltip_text(Some("Cancelar"));
        row.add_suffix(&cancel_btn);

        Self { widget: row }
    }

    /// Get icon name for file based on type
    fn get_icon_for_file(file: &CloudFile) -> &'static str {
        if file.is_folder {
            return "folder-symbolic";
        }

        // Check by extension
        let ext = std::path::Path::new(&file.name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            // Documents
            "pdf" => "application-pdf-symbolic",
            "doc" | "docx" | "odt" => "x-office-document-symbolic",
            "xls" | "xlsx" | "ods" => "x-office-spreadsheet-symbolic",
            "ppt" | "pptx" | "odp" => "x-office-presentation-symbolic",
            "txt" | "md" | "rtf" => "text-x-generic-symbolic",

            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" => "image-x-generic-symbolic",

            // Audio
            "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "audio-x-generic-symbolic",

            // Video
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" => "video-x-generic-symbolic",

            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "package-x-generic-symbolic",

            // Code
            "rs" | "py" | "js" | "ts" | "html" | "css" | "json" | "xml" | "yaml" | "toml" => "text-x-script-symbolic",

            // Default
            _ => "text-x-generic-symbolic",
        }
    }

    /// Get status icon for sync status
    fn get_status_icon(status: FileSyncStatus) -> Image {
        let (icon_name, tooltip) = match status {
            FileSyncStatus::Synced => ("emblem-ok-symbolic", "Sincronizado"),
            FileSyncStatus::PendingUpload => ("go-up-symbolic", "Aguardando upload"),
            FileSyncStatus::PendingDownload => ("go-down-symbolic", "Aguardando download"),
            FileSyncStatus::Syncing => ("emblem-synchronizing-symbolic", "Sincronizando..."),
            FileSyncStatus::Conflict => ("dialog-warning-symbolic", "Conflito"),
            FileSyncStatus::Error => ("dialog-error-symbolic", "Erro"),
            FileSyncStatus::Ignored => ("action-unavailable-symbolic", "Ignorado"),
        };

        let icon = Image::from_icon_name(icon_name);
        icon.set_tooltip_text(Some(tooltip));
        icon
    }

    /// Get the widget
    pub fn widget(&self) -> &adw::ActionRow {
        &self.widget
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

/// Format date
fn format_date(date: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - date;

    if diff.num_hours() < 24 {
        if diff.num_hours() < 1 {
            "agora".to_string()
        } else {
            format!("ha {} horas", diff.num_hours())
        }
    } else if diff.num_days() < 7 {
        format!("ha {} dias", diff.num_days())
    } else {
        date.format("%d/%m/%Y").to_string()
    }
}
