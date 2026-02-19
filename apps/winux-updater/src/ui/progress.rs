//! Progress widget for update installation

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ProgressBar, ScrolledWindow, TextView};
use libadwaita as adw;
use adw::prelude::*;
use adw::Clamp;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{UpdateProgress, UpdateStatus};

/// Widget for displaying update progress
pub struct ProgressWidget {
    widget: Box,
    title_label: Label,
    subtitle_label: Label,
    overall_label: Label,
    overall_progress: ProgressBar,
    current_label: Label,
    current_progress: ProgressBar,
    log_view: TextView,
    cancel_btn: Button,
    is_cancelled: Rc<RefCell<bool>>,
}

impl ProgressWidget {
    pub fn new() -> Self {
        let content = Box::new(Orientation::Vertical, 24);
        content.set_margin_top(48);
        content.set_margin_bottom(48);
        content.set_margin_start(48);
        content.set_margin_end(48);

        // Title
        let title_label = Label::new(Some("Instalando Atualizacoes"));
        title_label.add_css_class("title-1");
        content.append(&title_label);

        // Subtitle
        let subtitle_label = Label::new(Some("Nao desligue o computador durante a atualizacao"));
        subtitle_label.add_css_class("dim-label");
        content.append(&subtitle_label);

        // Overall progress section
        let overall_box = Box::new(Orientation::Vertical, 8);
        overall_box.set_margin_top(24);

        let overall_label = Label::new(Some("Progresso geral: 0 de 0 pacotes"));
        overall_label.set_halign(gtk4::Align::Start);
        overall_box.append(&overall_label);

        let overall_progress = ProgressBar::new();
        overall_progress.set_fraction(0.0);
        overall_progress.set_show_text(true);
        overall_progress.set_text(Some("0%"));
        overall_box.append(&overall_progress);

        content.append(&overall_box);

        // Current package progress section
        let current_box = Box::new(Orientation::Vertical, 8);
        current_box.set_margin_top(24);

        let current_label = Label::new(Some("Aguardando..."));
        current_label.set_halign(gtk4::Align::Start);
        current_box.append(&current_label);

        let current_progress = ProgressBar::new();
        current_progress.set_fraction(0.0);
        current_progress.set_show_text(true);
        current_box.append(&current_progress);

        content.append(&current_box);

        // Log output
        let log_frame = gtk4::Frame::new(Some("Log de instalacao"));
        log_frame.set_margin_top(24);

        let log_scroll = ScrolledWindow::builder()
            .min_content_height(150)
            .max_content_height(300)
            .build();

        let log_view = TextView::new();
        log_view.set_editable(false);
        log_view.set_monospace(true);
        log_view.set_wrap_mode(gtk4::WrapMode::Word);
        log_view.set_cursor_visible(false);

        log_scroll.set_child(Some(&log_view));
        log_frame.set_child(Some(&log_scroll));
        content.append(&log_frame);

        // Cancel button
        let cancel_btn = Button::with_label("Cancelar");
        cancel_btn.add_css_class("destructive-action");
        cancel_btn.set_halign(gtk4::Align::Center);
        cancel_btn.set_margin_top(24);
        content.append(&cancel_btn);

        // Wrap in clamp for nice centering
        let clamp = Clamp::new();
        clamp.set_maximum_size(800);
        clamp.set_child(Some(&content));

        let widget = Box::new(Orientation::Vertical, 0);
        widget.append(&clamp);

        let is_cancelled = Rc::new(RefCell::new(false));
        let is_cancelled_clone = is_cancelled.clone();

        cancel_btn.connect_clicked(move |_| {
            *is_cancelled_clone.borrow_mut() = true;
        });

        Self {
            widget,
            title_label,
            subtitle_label,
            overall_label,
            overall_progress,
            current_label,
            current_progress,
            log_view,
            cancel_btn,
            is_cancelled,
        }
    }

    /// Update the progress display
    pub fn update(&self, progress: &UpdateProgress) {
        // Update labels
        self.overall_label.set_text(&format!(
            "Progresso geral: {} de {} pacotes",
            progress.current_index,
            progress.total_packages
        ));

        self.current_label.set_text(&format!(
            "{}: {}",
            Self::status_text(progress.status),
            progress.current_package
        ));

        // Update progress bars
        self.overall_progress.set_fraction(progress.overall_progress);
        self.overall_progress.set_text(Some(&format!("{:.0}%", progress.overall_progress * 100.0)));

        self.current_progress.set_fraction(progress.package_progress);
        self.current_progress.set_text(Some(&format!(
            "{:.0}% - {}",
            progress.package_progress * 100.0,
            Self::status_text(progress.status)
        )));

        // Append to log
        let buffer = self.log_view.buffer();
        let mut end_iter = buffer.end_iter();
        buffer.insert(&mut end_iter, &progress.log_output);

        // Auto-scroll to bottom
        if let Some(mark) = buffer.mark("insert") {
            self.log_view.scroll_to_mark(&mark, 0.0, false, 0.0, 1.0);
        }

        // Update UI based on status
        match progress.status {
            UpdateStatus::Finished => {
                self.title_label.set_text("Atualizacao Concluida");
                self.subtitle_label.set_text("Todas as atualizacoes foram instaladas com sucesso");
                self.cancel_btn.set_label("Fechar");
                self.cancel_btn.remove_css_class("destructive-action");
                self.cancel_btn.add_css_class("suggested-action");
            }
            UpdateStatus::Failed => {
                self.title_label.set_text("Erro na Atualizacao");
                self.subtitle_label.set_text("Ocorreu um erro durante a instalacao");
                self.subtitle_label.add_css_class("error");
                self.cancel_btn.set_label("Fechar");
                self.cancel_btn.remove_css_class("destructive-action");
            }
            UpdateStatus::Cancelled => {
                self.title_label.set_text("Atualizacao Cancelada");
                self.subtitle_label.set_text("A atualizacao foi interrompida pelo usuario");
                self.cancel_btn.set_label("Fechar");
                self.cancel_btn.remove_css_class("destructive-action");
            }
            _ => {}
        }
    }

    /// Get status text in Portuguese
    fn status_text(status: UpdateStatus) -> &'static str {
        match status {
            UpdateStatus::Checking => "Verificando",
            UpdateStatus::Downloading => "Baixando",
            UpdateStatus::Installing => "Instalando",
            UpdateStatus::Configuring => "Configurando",
            UpdateStatus::Finished => "Concluido",
            UpdateStatus::Failed => "Erro",
            UpdateStatus::Cancelled => "Cancelado",
        }
    }

    /// Set title
    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }

    /// Set subtitle
    pub fn set_subtitle(&self, subtitle: &str) {
        self.subtitle_label.set_text(subtitle);
    }

    /// Append text to log
    pub fn append_log(&self, text: &str) {
        let buffer = self.log_view.buffer();
        let mut end_iter = buffer.end_iter();
        buffer.insert(&mut end_iter, text);
        buffer.insert(&mut end_iter, "\n");
    }

    /// Clear log
    pub fn clear_log(&self) {
        let buffer = self.log_view.buffer();
        buffer.set_text("");
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        *self.is_cancelled.borrow()
    }

    /// Reset cancelled state
    pub fn reset(&self) {
        *self.is_cancelled.borrow_mut() = false;
        self.overall_progress.set_fraction(0.0);
        self.current_progress.set_fraction(0.0);
        self.clear_log();
        self.title_label.set_text("Instalando Atualizacoes");
        self.subtitle_label.set_text("Nao desligue o computador durante a atualizacao");
        self.subtitle_label.remove_css_class("error");
        self.cancel_btn.set_label("Cancelar");
        self.cancel_btn.add_css_class("destructive-action");
        self.cancel_btn.remove_css_class("suggested-action");
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.widget
    }

    /// Set cancel button sensitivity
    pub fn set_cancelable(&self, cancelable: bool) {
        self.cancel_btn.set_sensitive(cancelable);
    }

    /// Connect cancel handler
    pub fn connect_cancel<F: Fn() + 'static>(&self, f: F) {
        self.cancel_btn.connect_clicked(move |_| {
            f();
        });
    }
}

impl Default for ProgressWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple progress indicator for checking updates
pub struct CheckingIndicator {
    widget: Box,
    spinner: gtk4::Spinner,
    label: Label,
}

impl CheckingIndicator {
    pub fn new() -> Self {
        let widget = Box::new(Orientation::Horizontal, 12);
        widget.set_halign(gtk4::Align::Center);
        widget.set_valign(gtk4::Align::Center);

        let spinner = gtk4::Spinner::new();
        widget.append(&spinner);

        let label = Label::new(Some("Verificando atualizacoes..."));
        label.add_css_class("dim-label");
        widget.append(&label);

        Self {
            widget,
            spinner,
            label,
        }
    }

    pub fn start(&self) {
        self.spinner.start();
        self.widget.set_visible(true);
    }

    pub fn stop(&self) {
        self.spinner.stop();
        self.widget.set_visible(false);
    }

    pub fn set_text(&self, text: &str) {
        self.label.set_text(text);
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }
}

impl Default for CheckingIndicator {
    fn default() -> Self {
        Self::new()
    }
}
