//! Job row widget for displaying print job information

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::cups::{PrintJob, JobStatus};

/// A row widget displaying print job information with actions
#[derive(Debug, Clone)]
pub struct JobRow {
    widget: adw::ExpanderRow,
    progress_bar: Option<gtk4::ProgressBar>,
    status_icon: gtk4::Image,
}

impl JobRow {
    /// Create a new job row
    pub fn new(job: &PrintJob) -> Self {
        let status_text = job.status.display_string();
        let subtitle = format!(
            "{} - {} paginas - {}",
            job.printer, job.pages, status_text
        );

        let widget = adw::ExpanderRow::builder()
            .title(&job.document_name)
            .subtitle(&subtitle)
            .build();

        // Document icon based on file extension
        let icon_name = Self::icon_for_document(&job.document_name);
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(24);
        widget.add_prefix(&icon);

        // Progress bar for printing jobs
        let progress_bar = if let JobStatus::Printing(progress) = &job.status {
            let bar = gtk4::ProgressBar::new();
            bar.set_fraction(*progress as f64 / 100.0);
            bar.set_valign(gtk4::Align::Center);
            bar.set_hexpand(false);
            bar.set_width_request(100);
            widget.add_suffix(&bar.clone());
            Some(bar)
        } else {
            None
        };

        // Status icon
        let status_icon = gtk4::Image::from_icon_name(job.status.icon_name());
        status_icon.set_valign(gtk4::Align::Center);

        match &job.status {
            JobStatus::Pending => status_icon.add_css_class("dim-label"),
            JobStatus::Held => status_icon.add_css_class("warning"),
            JobStatus::Processing | JobStatus::Printing(_) => status_icon.add_css_class("accent"),
            JobStatus::Completed => status_icon.add_css_class("success"),
            JobStatus::Cancelled => status_icon.add_css_class("dim-label"),
            JobStatus::Aborted(_) => status_icon.add_css_class("error"),
        }

        widget.add_suffix(&status_icon.clone());

        // Add action rows
        Self::add_action_rows(&widget, job);

        Self {
            widget,
            progress_bar,
            status_icon,
        }
    }

    /// Create a simple (non-expandable) job row
    pub fn new_simple(job: &PrintJob) -> adw::ActionRow {
        let status_text = job.status.display_string();
        let subtitle = format!(
            "{} - {} paginas - {}",
            job.printer, job.pages, status_text
        );

        let row = adw::ActionRow::builder()
            .title(&job.document_name)
            .subtitle(&subtitle)
            .activatable(true)
            .build();

        // Document icon
        let icon_name = Self::icon_for_document(&job.document_name);
        let icon = gtk4::Image::from_icon_name(icon_name);
        row.add_prefix(&icon);

        // Status icon
        let status_icon = gtk4::Image::from_icon_name(job.status.icon_name());

        match &job.status {
            JobStatus::Pending => status_icon.add_css_class("dim-label"),
            JobStatus::Held => status_icon.add_css_class("warning"),
            JobStatus::Processing | JobStatus::Printing(_) => status_icon.add_css_class("accent"),
            JobStatus::Completed => status_icon.add_css_class("success"),
            JobStatus::Cancelled => status_icon.add_css_class("dim-label"),
            JobStatus::Aborted(_) => status_icon.add_css_class("error"),
        }

        row.add_suffix(&status_icon);

        row
    }

    /// Create a completed job row (simpler, with reprint option)
    pub fn new_completed(job: &PrintJob) -> adw::ActionRow {
        let status_text = match &job.status {
            JobStatus::Completed => "Concluido",
            JobStatus::Cancelled => "Cancelado",
            JobStatus::Aborted(_) => "Abortado",
            _ => "Desconhecido",
        };

        let subtitle = format!("{} - {} paginas - {}", job.printer, job.pages, status_text);

        let row = adw::ActionRow::builder()
            .title(&job.document_name)
            .subtitle(&subtitle)
            .build();

        // Icon based on status
        let icon_name = match &job.status {
            JobStatus::Completed => "emblem-ok-symbolic",
            JobStatus::Cancelled => "window-close-symbolic",
            JobStatus::Aborted(_) => "dialog-error-symbolic",
            _ => "printer-symbolic",
        };
        let icon = gtk4::Image::from_icon_name(icon_name);

        match &job.status {
            JobStatus::Completed => icon.add_css_class("success"),
            JobStatus::Cancelled => icon.add_css_class("dim-label"),
            JobStatus::Aborted(_) => icon.add_css_class("error"),
            _ => {}
        }

        row.add_prefix(&icon);

        // Time indicator
        let time_label = gtk4::Label::new(Some("Agora")); // Would calculate actual time
        time_label.add_css_class("dim-label");
        time_label.add_css_class("caption");
        row.add_suffix(&time_label);

        // Reprint button for completed jobs
        if matches!(job.status, JobStatus::Completed) {
            let reprint_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
            reprint_btn.add_css_class("flat");
            reprint_btn.set_valign(gtk4::Align::Center);
            reprint_btn.set_tooltip_text(Some("Reimprimir"));
            row.add_suffix(&reprint_btn);
        }

        row
    }

    /// Add action rows to an expander row
    fn add_action_rows(expander: &adw::ExpanderRow, job: &PrintJob) {
        // Cancel job
        if job.can_cancel() {
            let cancel_row = adw::ActionRow::builder()
                .title("Cancelar Trabalho")
                .activatable(true)
                .build();
            cancel_row.add_prefix(&gtk4::Image::from_icon_name("process-stop-symbolic"));
            cancel_row.add_css_class("error");
            expander.add_row(&cancel_row);
        }

        // Hold/Release
        if job.can_hold() {
            let hold_row = adw::ActionRow::builder()
                .title("Reter Trabalho")
                .subtitle("Pausar na fila")
                .activatable(true)
                .build();
            hold_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-pause-symbolic"));
            expander.add_row(&hold_row);
        }

        if job.can_release() {
            let release_row = adw::ActionRow::builder()
                .title("Liberar Trabalho")
                .subtitle("Continuar impressao")
                .activatable(true)
                .build();
            release_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-start-symbolic"));
            expander.add_row(&release_row);
        }

        // Move in queue (for pending jobs)
        if matches!(job.status, JobStatus::Pending) {
            let up_row = adw::ActionRow::builder()
                .title("Mover para Cima")
                .subtitle("Aumentar prioridade")
                .activatable(true)
                .build();
            up_row.add_prefix(&gtk4::Image::from_icon_name("go-up-symbolic"));
            expander.add_row(&up_row);

            let down_row = adw::ActionRow::builder()
                .title("Mover para Baixo")
                .subtitle("Diminuir prioridade")
                .activatable(true)
                .build();
            down_row.add_prefix(&gtk4::Image::from_icon_name("go-down-symbolic"));
            expander.add_row(&down_row);
        }

        // Move to another printer
        if job.can_move() {
            let move_row = adw::ActionRow::builder()
                .title("Mover para Outra Impressora")
                .activatable(true)
                .build();
            move_row.add_prefix(&gtk4::Image::from_icon_name("printer-symbolic"));
            move_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            expander.add_row(&move_row);
        }

        // Job details
        let size_str = job.formatted_size();
        let details_row = adw::ActionRow::builder()
            .title("Detalhes")
            .subtitle(&format!(
                "ID: {} | Usuario: {} | Tamanho: {}",
                job.id, job.user, size_str
            ))
            .build();
        details_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        expander.add_row(&details_row);
    }

    /// Get icon name based on document file extension
    fn icon_for_document(filename: &str) -> &'static str {
        let lower = filename.to_lowercase();

        if lower.ends_with(".pdf") {
            "x-office-document-symbolic"
        } else if lower.ends_with(".doc") || lower.ends_with(".docx") || lower.ends_with(".odt") {
            "x-office-document-symbolic"
        } else if lower.ends_with(".xls") || lower.ends_with(".xlsx") || lower.ends_with(".ods") {
            "x-office-spreadsheet-symbolic"
        } else if lower.ends_with(".ppt") || lower.ends_with(".pptx") || lower.ends_with(".odp") {
            "x-office-presentation-symbolic"
        } else if lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".png")
            || lower.ends_with(".gif")
            || lower.ends_with(".bmp")
        {
            "image-x-generic-symbolic"
        } else if lower.ends_with(".txt") || lower.ends_with(".md") || lower.ends_with(".rst") {
            "text-x-generic-symbolic"
        } else if lower.ends_with(".html") || lower.ends_with(".htm") {
            "text-html-symbolic"
        } else {
            "text-x-generic-symbolic"
        }
    }

    /// Get the underlying widget
    pub fn widget(&self) -> &adw::ExpanderRow {
        &self.widget
    }

    /// Update the job progress
    pub fn update_progress(&self, progress: u8) {
        if let Some(bar) = &self.progress_bar {
            bar.set_fraction(progress as f64 / 100.0);
        }

        let new_status = JobStatus::Printing(progress);
        self.widget.set_subtitle(Some(&format!(
            "Imprimindo... {}%",
            progress
        )));
    }

    /// Update the job status
    pub fn update_status(&self, status: &JobStatus) {
        self.widget.set_subtitle(Some(&status.display_string()));
        self.status_icon.set_icon_name(Some(status.icon_name()));

        // Clear existing CSS classes
        self.status_icon.remove_css_class("dim-label");
        self.status_icon.remove_css_class("warning");
        self.status_icon.remove_css_class("accent");
        self.status_icon.remove_css_class("success");
        self.status_icon.remove_css_class("error");

        // Apply new CSS class
        match status {
            JobStatus::Pending => self.status_icon.add_css_class("dim-label"),
            JobStatus::Held => self.status_icon.add_css_class("warning"),
            JobStatus::Processing | JobStatus::Printing(_) => {
                self.status_icon.add_css_class("accent")
            }
            JobStatus::Completed => self.status_icon.add_css_class("success"),
            JobStatus::Cancelled => self.status_icon.add_css_class("dim-label"),
            JobStatus::Aborted(_) => self.status_icon.add_css_class("error"),
        }
    }
}
