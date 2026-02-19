//! File row widget for displaying file transfer status

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::pages::files::{FileTransfer, TransferStatus, TransferDirection};

/// File row widget showing transfer progress
pub struct FileRow {
    widget: adw::ActionRow,
}

impl FileRow {
    pub fn new(transfer: &FileTransfer) -> Self {
        let row = adw::ActionRow::builder()
            .title(&transfer.filename)
            .subtitle(&Self::format_subtitle(transfer))
            .build();

        // File icon
        let icon_name = Self::get_file_icon(&transfer.filename);
        row.add_prefix(&gtk4::Image::from_icon_name(icon_name));

        // Direction indicator
        let direction_icon = if transfer.direction == TransferDirection::Upload {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };
        row.add_prefix(&gtk4::Image::from_icon_name(direction_icon));

        // Content based on status
        match transfer.status {
            TransferStatus::InProgress => {
                // Progress bar
                let progress_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
                progress_box.set_width_request(150);
                progress_box.set_valign(gtk4::Align::Center);

                let progress_bar = gtk4::ProgressBar::new();
                progress_bar.set_fraction(transfer.progress);
                progress_box.append(&progress_bar);

                let progress_label = gtk4::Label::new(Some(&format!(
                    "{:.0}% - {}",
                    transfer.progress * 100.0,
                    Self::format_size(transfer.size)
                )));
                progress_label.add_css_class("caption");
                progress_label.add_css_class("dim-label");
                progress_box.append(&progress_label);

                row.add_suffix(&progress_box);

                // Cancel button
                let cancel_btn = gtk4::Button::from_icon_name("process-stop-symbolic");
                cancel_btn.set_valign(gtk4::Align::Center);
                cancel_btn.set_tooltip_text(Some("Cancelar"));
                row.add_suffix(&cancel_btn);
            }
            TransferStatus::Pending => {
                let spinner = gtk4::Spinner::new();
                spinner.set_spinning(true);
                row.add_suffix(&spinner);

                let waiting_label = gtk4::Label::new(Some("Aguardando..."));
                waiting_label.add_css_class("dim-label");
                row.add_suffix(&waiting_label);

                let cancel_btn = gtk4::Button::from_icon_name("process-stop-symbolic");
                cancel_btn.set_valign(gtk4::Align::Center);
                row.add_suffix(&cancel_btn);
            }
            TransferStatus::Completed => {
                let status_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
                status_icon.add_css_class("success");
                row.add_suffix(&status_icon);

                let size_label = gtk4::Label::new(Some(&Self::format_size(transfer.size)));
                size_label.add_css_class("dim-label");
                row.add_suffix(&size_label);

                // Open button
                let open_btn = gtk4::Button::from_icon_name("folder-open-symbolic");
                open_btn.set_valign(gtk4::Align::Center);
                open_btn.set_tooltip_text(Some("Abrir pasta"));
                row.add_suffix(&open_btn);
            }
            TransferStatus::Failed => {
                let status_icon = gtk4::Image::from_icon_name("dialog-error-symbolic");
                status_icon.add_css_class("error");
                row.add_suffix(&status_icon);

                let retry_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
                retry_btn.set_valign(gtk4::Align::Center);
                retry_btn.set_tooltip_text(Some("Tentar novamente"));
                row.add_suffix(&retry_btn);
            }
            TransferStatus::Cancelled => {
                let status_label = gtk4::Label::new(Some("Cancelado"));
                status_label.add_css_class("dim-label");
                row.add_suffix(&status_label);
            }
        }

        Self { widget: row }
    }

    fn format_subtitle(transfer: &FileTransfer) -> String {
        let direction = if transfer.direction == TransferDirection::Upload {
            "Enviando para"
        } else {
            "Recebendo de"
        };

        let status = match transfer.status {
            TransferStatus::Pending => "Aguardando",
            TransferStatus::InProgress => "Transferindo",
            TransferStatus::Completed => "Concluido",
            TransferStatus::Failed => "Falhou",
            TransferStatus::Cancelled => "Cancelado",
        };

        format!("{} {} - {}", direction, transfer.device_name, status)
    }

    fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        if unit_idx == 0 {
            format!("{} {}", bytes, UNITS[0])
        } else {
            format!("{:.1} {}", size, UNITS[unit_idx])
        }
    }

    fn get_file_icon(filename: &str) -> &'static str {
        let extension = filename.rsplit('.').next().unwrap_or("").to_lowercase();

        match extension.as_str() {
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => "image-x-generic-symbolic",

            // Videos
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" => "video-x-generic-symbolic",

            // Audio
            "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => "audio-x-generic-symbolic",

            // Documents
            "pdf" => "x-office-document-symbolic",
            "doc" | "docx" | "odt" => "x-office-document-symbolic",
            "xls" | "xlsx" | "ods" => "x-office-spreadsheet-symbolic",
            "ppt" | "pptx" | "odp" => "x-office-presentation-symbolic",
            "txt" | "md" | "rst" => "text-x-generic-symbolic",

            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "package-x-generic-symbolic",

            // Code
            "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go" => "text-x-script-symbolic",

            // APK
            "apk" => "application-x-executable-symbolic",

            // Default
            _ => "text-x-generic-symbolic",
        }
    }

    pub fn widget(&self) -> adw::ActionRow {
        self.widget.clone()
    }
}

/// File row with expandable details
pub struct FileRowExpanded {
    widget: adw::ExpanderRow,
}

impl FileRowExpanded {
    pub fn new(transfer: &FileTransfer) -> Self {
        let row = adw::ExpanderRow::builder()
            .title(&transfer.filename)
            .subtitle(&FileRow::format_subtitle(transfer))
            .build();

        let icon_name = FileRow::get_file_icon(&transfer.filename);
        row.add_prefix(&gtk4::Image::from_icon_name(icon_name));

        // Progress for active transfers
        if transfer.status == TransferStatus::InProgress {
            let progress_bar = gtk4::ProgressBar::new();
            progress_bar.set_fraction(transfer.progress);
            progress_bar.set_valign(gtk4::Align::Center);
            progress_bar.set_width_request(100);
            row.add_suffix(&progress_bar);
        }

        // Details rows
        let size_row = adw::ActionRow::builder()
            .title("Tamanho")
            .subtitle(&FileRow::format_size(transfer.size))
            .build();
        row.add_row(&size_row);

        let device_row = adw::ActionRow::builder()
            .title("Dispositivo")
            .subtitle(&transfer.device_name)
            .build();
        row.add_row(&device_row);

        // Actions based on status
        match transfer.status {
            TransferStatus::InProgress | TransferStatus::Pending => {
                let cancel_row = adw::ActionRow::builder()
                    .title("Cancelar Transferencia")
                    .activatable(true)
                    .build();
                cancel_row.add_prefix(&gtk4::Image::from_icon_name("process-stop-symbolic"));
                row.add_row(&cancel_row);
            }
            TransferStatus::Completed => {
                let open_row = adw::ActionRow::builder()
                    .title("Abrir Arquivo")
                    .activatable(true)
                    .build();
                open_row.add_prefix(&gtk4::Image::from_icon_name("document-open-symbolic"));
                row.add_row(&open_row);

                let folder_row = adw::ActionRow::builder()
                    .title("Abrir Pasta")
                    .activatable(true)
                    .build();
                folder_row.add_prefix(&gtk4::Image::from_icon_name("folder-open-symbolic"));
                row.add_row(&folder_row);
            }
            TransferStatus::Failed => {
                let retry_row = adw::ActionRow::builder()
                    .title("Tentar Novamente")
                    .activatable(true)
                    .build();
                retry_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));
                row.add_row(&retry_row);
            }
            _ => {}
        }

        Self { widget: row }
    }

    pub fn widget(&self) -> adw::ExpanderRow {
        self.widget.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(FileRow::format_size(0), "0 B");
        assert_eq!(FileRow::format_size(512), "512 B");
        assert_eq!(FileRow::format_size(1024), "1.0 KB");
        assert_eq!(FileRow::format_size(1048576), "1.0 MB");
        assert_eq!(FileRow::format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_file_icon() {
        assert_eq!(FileRow::get_file_icon("photo.jpg"), "image-x-generic-symbolic");
        assert_eq!(FileRow::get_file_icon("video.mp4"), "video-x-generic-symbolic");
        assert_eq!(FileRow::get_file_icon("music.mp3"), "audio-x-generic-symbolic");
        assert_eq!(FileRow::get_file_icon("document.pdf"), "x-office-document-symbolic");
        assert_eq!(FileRow::get_file_icon("archive.zip"), "package-x-generic-symbolic");
        assert_eq!(FileRow::get_file_icon("code.rs"), "text-x-script-symbolic");
        assert_eq!(FileRow::get_file_icon("unknown"), "text-x-generic-symbolic");
    }
}
