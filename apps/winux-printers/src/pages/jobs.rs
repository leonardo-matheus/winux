//! Print queue / jobs page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::cups::{CupsManager, PrintJob, JobStatus};
use crate::ui::JobRow;

/// Jobs page - shows print queue and allows job management
pub struct JobsPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<CupsManager>>,
}

impl JobsPage {
    pub fn new(manager: Rc<RefCell<CupsManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Fila de Impressao");
        page.set_icon_name(Some("view-list-symbolic"));

        // Active jobs group
        let active_group = adw::PreferencesGroup::builder()
            .title("Trabalhos Ativos")
            .description("Trabalhos atualmente sendo impressos ou na fila")
            .build();

        // Sample active jobs
        let active_jobs = vec![
            PrintJob::new(
                1001,
                "Relatorio-Financeiro-2026.pdf",
                "HP-LaserJet-Pro",
                "leonardo",
                JobStatus::Printing(45),
                15,
                1024 * 1024 * 2,  // 2 MB
                chrono::Utc::now(),
            ),
            PrintJob::new(
                1002,
                "Apresentacao-Projeto.pdf",
                "HP-LaserJet-Pro",
                "leonardo",
                JobStatus::Pending,
                8,
                1024 * 512,  // 512 KB
                chrono::Utc::now(),
            ),
            PrintJob::new(
                1003,
                "Foto-Familia.jpg",
                "Canon-PIXMA",
                "leonardo",
                JobStatus::Held,
                1,
                1024 * 1024 * 5,  // 5 MB
                chrono::Utc::now(),
            ),
        ];

        for job in &active_jobs {
            let row = Self::create_job_row(job);
            active_group.add(&row);
        }

        if active_jobs.is_empty() {
            let empty_row = adw::ActionRow::builder()
                .title("Nenhum trabalho na fila")
                .subtitle("A fila de impressao esta vazia")
                .sensitive(false)
                .build();
            empty_row.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
            active_group.add(&empty_row);
        }

        page.add(&active_group);

        // Completed jobs group
        let completed_group = adw::PreferencesGroup::builder()
            .title("Trabalhos Concluidos")
            .description("Ultimos trabalhos impressos")
            .build();

        let completed_jobs = vec![
            PrintJob::new(
                999,
                "Contrato-Servicos.pdf",
                "HP-LaserJet-Pro",
                "leonardo",
                JobStatus::Completed,
                4,
                1024 * 256,  // 256 KB
                chrono::Utc::now() - chrono::Duration::minutes(30),
            ),
            PrintJob::new(
                998,
                "Email-Cliente.html",
                "PDF-Printer",
                "leonardo",
                JobStatus::Completed,
                2,
                1024 * 64,  // 64 KB
                chrono::Utc::now() - chrono::Duration::hours(1),
            ),
            PrintJob::new(
                997,
                "Teste.txt",
                "Brother-MFC",
                "leonardo",
                JobStatus::Cancelled,
                1,
                1024,  // 1 KB
                chrono::Utc::now() - chrono::Duration::hours(2),
            ),
        ];

        for job in &completed_jobs {
            let row = Self::create_completed_job_row(job);
            completed_group.add(&row);
        }

        // Clear history button
        let clear_row = adw::ActionRow::builder()
            .title("Limpar Historico")
            .subtitle("Remover todos os trabalhos concluidos")
            .activatable(true)
            .build();
        clear_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        completed_group.add(&clear_row);

        page.add(&completed_group);

        // Queue actions group
        let actions_group = adw::PreferencesGroup::builder()
            .title("Acoes da Fila")
            .build();

        let cancel_all_row = adw::ActionRow::builder()
            .title("Cancelar Todos os Trabalhos")
            .subtitle("Cancelar todos os trabalhos pendentes")
            .activatable(true)
            .build();
        cancel_all_row.add_prefix(&gtk4::Image::from_icon_name("process-stop-symbolic"));
        cancel_all_row.add_css_class("error");
        actions_group.add(&cancel_all_row);

        let pause_all_row = adw::ActionRow::builder()
            .title("Pausar Fila")
            .subtitle("Pausar temporariamente toda a impressao")
            .activatable(true)
            .build();
        pause_all_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-pause-symbolic"));
        actions_group.add(&pause_all_row);

        page.add(&actions_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    fn create_job_row(job: &PrintJob) -> adw::ExpanderRow {
        let status_text = match &job.status {
            JobStatus::Pending => "Aguardando".to_string(),
            JobStatus::Held => "Retido".to_string(),
            JobStatus::Processing => "Processando...".to_string(),
            JobStatus::Printing(progress) => format!("Imprimindo... {}%", progress),
            JobStatus::Completed => "Concluido".to_string(),
            JobStatus::Cancelled => "Cancelado".to_string(),
            JobStatus::Aborted(reason) => format!("Abortado: {}", reason),
        };

        let subtitle = format!("{} - {} paginas - {}", job.printer, job.pages, status_text);

        let row = adw::ExpanderRow::builder()
            .title(&job.document_name)
            .subtitle(&subtitle)
            .build();

        // Document icon
        row.add_prefix(&gtk4::Image::from_icon_name("x-office-document-symbolic"));

        // Progress bar for printing jobs
        if let JobStatus::Printing(progress) = &job.status {
            let progress_bar = gtk4::ProgressBar::new();
            progress_bar.set_fraction(*progress as f64 / 100.0);
            progress_bar.set_valign(gtk4::Align::Center);
            progress_bar.set_hexpand(false);
            progress_bar.set_width_request(100);
            row.add_suffix(&progress_bar);
        }

        // Status indicator
        let status_icon = match &job.status {
            JobStatus::Pending => {
                let icon = gtk4::Image::from_icon_name("content-loading-symbolic");
                icon.add_css_class("dim-label");
                icon
            }
            JobStatus::Held => {
                let icon = gtk4::Image::from_icon_name("media-playback-pause-symbolic");
                icon.add_css_class("warning");
                icon
            }
            JobStatus::Processing | JobStatus::Printing(_) => {
                let icon = gtk4::Image::from_icon_name("printer-printing-symbolic");
                icon.add_css_class("accent");
                icon
            }
            _ => gtk4::Image::from_icon_name("printer-symbolic"),
        };
        row.add_suffix(&status_icon);

        // Expandable actions
        // Cancel job
        let cancel_row = adw::ActionRow::builder()
            .title("Cancelar Trabalho")
            .activatable(true)
            .build();
        cancel_row.add_prefix(&gtk4::Image::from_icon_name("process-stop-symbolic"));
        cancel_row.add_css_class("error");
        row.add_row(&cancel_row);

        // Hold/Release
        let hold_title = if matches!(job.status, JobStatus::Held) {
            "Liberar Trabalho"
        } else {
            "Reter Trabalho"
        };
        let hold_row = adw::ActionRow::builder()
            .title(hold_title)
            .activatable(true)
            .build();
        hold_row.add_prefix(&gtk4::Image::from_icon_name("media-playback-pause-symbolic"));
        row.add_row(&hold_row);

        // Move up in queue
        let up_row = adw::ActionRow::builder()
            .title("Mover para Cima")
            .subtitle("Aumentar prioridade na fila")
            .activatable(true)
            .build();
        up_row.add_prefix(&gtk4::Image::from_icon_name("go-up-symbolic"));
        row.add_row(&up_row);

        // Move down in queue
        let down_row = adw::ActionRow::builder()
            .title("Mover para Baixo")
            .subtitle("Diminuir prioridade na fila")
            .activatable(true)
            .build();
        down_row.add_prefix(&gtk4::Image::from_icon_name("go-down-symbolic"));
        row.add_row(&down_row);

        // Move to another printer
        let move_row = adw::ActionRow::builder()
            .title("Mover para Outra Impressora")
            .activatable(true)
            .build();
        move_row.add_prefix(&gtk4::Image::from_icon_name("printer-symbolic"));
        move_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        row.add_row(&move_row);

        // Job details
        let size_str = bytesize::ByteSize::b(job.size).to_string();
        let details_row = adw::ActionRow::builder()
            .title("Detalhes")
            .subtitle(&format!(
                "ID: {} | Usuario: {} | Tamanho: {}",
                job.id, job.user, size_str
            ))
            .build();
        details_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        row.add_row(&details_row);

        row
    }

    fn create_completed_job_row(job: &PrintJob) -> adw::ActionRow {
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

        // Time ago indicator
        let time_label = gtk4::Label::new(Some("Agora"));
        time_label.add_css_class("dim-label");
        time_label.add_css_class("caption");
        row.add_suffix(&time_label);

        // Reprint button
        if matches!(job.status, JobStatus::Completed) {
            let reprint_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
            reprint_btn.add_css_class("flat");
            reprint_btn.set_valign(gtk4::Align::Center);
            reprint_btn.set_tooltip_text(Some("Reimprimir"));
            row.add_suffix(&reprint_btn);
        }

        row
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
