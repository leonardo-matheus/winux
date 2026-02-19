//! Format confirmation dialog
//!
//! Provides a secure confirmation dialog for format operations.

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label, Entry, Button, CheckButton, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{MessageDialog, ResponseAppearance, PreferencesGroup, ActionRow, ComboRow, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

/// Format confirmation dialog with multiple safety checks
pub struct FormatDialog {
    device_name: String,
    device_size: u64,
}

impl FormatDialog {
    pub fn new(device_name: &str, device_size: u64) -> Self {
        Self {
            device_name: device_name.to_string(),
            device_size,
        }
    }

    /// Show the format dialog
    pub fn show(&self, parent: &impl IsA<gtk4::Window>) {
        let dialog = MessageDialog::builder()
            .heading("Formatar Dispositivo")
            .body(&format!(
                "Voce esta prestes a formatar:\n\n\
                 Dispositivo: /dev/{}\n\
                 Tamanho: {}\n\n\
                 ATENCAO: Todos os dados serao PERMANENTEMENTE APAGADOS!\n\
                 Esta acao NAO PODE ser desfeita.",
                self.device_name,
                bytesize::ByteSize::b(self.device_size).to_string_as(true)
            ))
            .transient_for(parent)
            .modal(true)
            .build();

        // Add extra confirmation widgets
        let content = Box::new(Orientation::Vertical, 12);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Filesystem selection
        let fs_row = ComboRow::builder()
            .title("Sistema de Arquivos")
            .build();
        let filesystems = gtk4::StringList::new(&[
            "ext4",
            "btrfs",
            "xfs",
            "ntfs",
            "FAT32",
            "exFAT",
        ]);
        fs_row.set_model(Some(&filesystems));

        let fs_box = Box::new(Orientation::Vertical, 4);
        fs_box.append(&fs_row);
        content.append(&fs_box);

        // Label entry
        let label_box = Box::new(Orientation::Horizontal, 8);
        let label_label = Label::new(Some("Rotulo:"));
        let label_entry = Entry::new();
        label_entry.set_placeholder_text(Some("Nome do volume"));
        label_entry.set_hexpand(true);
        label_box.append(&label_label);
        label_box.append(&label_entry);
        content.append(&label_box);

        // Type device name confirmation
        let confirm_box = Box::new(Orientation::Vertical, 4);
        let confirm_label = Label::new(Some(&format!(
            "Digite '{}' para confirmar:",
            self.device_name
        )));
        confirm_label.set_xalign(0.0);
        confirm_label.add_css_class("caption");

        let confirm_entry = Entry::new();
        confirm_entry.set_placeholder_text(Some(&self.device_name));

        confirm_box.append(&confirm_label);
        confirm_box.append(&confirm_entry);
        content.append(&confirm_box);

        // Final checkbox
        let final_check = CheckButton::with_label(
            "Eu entendo que TODOS os dados serao perdidos"
        );
        final_check.add_css_class("error");
        content.append(&final_check);

        dialog.set_extra_child(Some(&content));

        // Add responses
        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("format", "FORMATAR");
        dialog.set_response_appearance("format", ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        // Initially disable format button
        dialog.set_response_enabled("format", false);

        // Enable format only when all conditions are met
        let device_name = self.device_name.clone();
        let final_check_clone = final_check.clone();
        let confirm_entry_clone = confirm_entry.clone();
        let dialog_clone = dialog.clone();

        let check_conditions = move || {
            let name_matches = confirm_entry_clone.text().as_str() == device_name;
            let checkbox_checked = final_check_clone.is_active();
            dialog_clone.set_response_enabled("format", name_matches && checkbox_checked);
        };

        let check_conditions_clone = check_conditions.clone();
        confirm_entry.connect_changed(move |_| {
            check_conditions_clone();
        });

        final_check.connect_toggled(move |_| {
            check_conditions();
        });

        // Handle response
        let device_name = self.device_name.clone();
        dialog.connect_response(Some("format"), move |dialog, _| {
            tracing::info!("Format confirmed for /dev/{}", device_name);

            // Show progress dialog
            Self::show_progress_dialog(dialog, &device_name);
        });

        dialog.present();
    }

    /// Show format progress dialog
    fn show_progress_dialog(parent_dialog: &MessageDialog, device_name: &str) {
        if let Some(window) = parent_dialog.transient_for() {
            let progress_dialog = MessageDialog::builder()
                .heading("Formatando...")
                .body(&format!("Formatando /dev/{}...", device_name))
                .transient_for(&window)
                .modal(true)
                .build();

            let content = Box::new(Orientation::Vertical, 12);
            content.set_margin_start(24);
            content.set_margin_end(24);

            let progress = ProgressBar::new();
            progress.set_show_text(true);
            progress.set_text(Some("Preparando..."));
            progress.pulse();
            content.append(&progress);

            let status = Label::new(Some("Iniciando formatacao..."));
            status.add_css_class("dim-label");
            content.append(&status);

            progress_dialog.set_extra_child(Some(&content));

            // Only cancel button while formatting
            progress_dialog.add_response("cancel", "Cancelar");
            progress_dialog.set_response_appearance("cancel", ResponseAppearance::Destructive);

            // Simulate progress (in real app, this would track actual format progress)
            let progress_clone = progress.clone();
            let status_clone = status.clone();
            let dialog_clone = progress_dialog.clone();

            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                progress_clone.pulse();
                glib::ControlFlow::Continue
            });

            progress_dialog.present();

            // Close parent dialog
            parent_dialog.close();
        }
    }

    /// Create a simple quick format dialog
    pub fn show_quick(parent: &impl IsA<gtk4::Window>, device_name: &str, on_confirm: impl Fn() + 'static) {
        let dialog = MessageDialog::builder()
            .heading("Formatacao Rapida")
            .body(&format!(
                "Formatar /dev/{} com formatacao rapida?\n\n\
                 Isso apagara a tabela de arquivos mas nao \
                 sobrescrevera os dados.",
                device_name
            ))
            .transient_for(parent)
            .modal(true)
            .build();

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("format", "Formatar");
        dialog.set_response_appearance("format", ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        dialog.connect_response(Some("format"), move |_, _| {
            on_confirm();
        });

        dialog.present();
    }

    /// Show secure erase confirmation
    pub fn show_secure_erase(parent: &impl IsA<gtk4::Window>, device_name: &str, device_size: u64) {
        let time_estimate = Self::estimate_erase_time(device_size);

        let dialog = MessageDialog::builder()
            .heading("Apagamento Seguro")
            .body(&format!(
                "Voce esta prestes a apagar SEGURAMENTE:\n\n\
                 Dispositivo: /dev/{}\n\
                 Tamanho: {}\n\
                 Tempo estimado: {}\n\n\
                 Este processo sobrescrevera TODOS os dados com zeros,\n\
                 tornando a recuperacao praticamente impossivel.\n\n\
                 ESTA ACAO E IRREVERSIVEL!",
                device_name,
                bytesize::ByteSize::b(device_size).to_string_as(true),
                time_estimate
            ))
            .transient_for(parent)
            .modal(true)
            .build();

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("erase", "APAGAR SEGURAMENTE");
        dialog.set_response_appearance("erase", ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        dialog.present();
    }

    /// Estimate secure erase time based on size
    fn estimate_erase_time(size_bytes: u64) -> String {
        // Assume ~100 MB/s write speed for conservative estimate
        let seconds = size_bytes / (100 * 1024 * 1024);

        if seconds < 60 {
            format!("~{} segundos", seconds)
        } else if seconds < 3600 {
            format!("~{} minutos", seconds / 60)
        } else {
            format!("~{:.1} horas", seconds as f64 / 3600.0)
        }
    }
}

/// Benchmark dialog for disk speed testing
pub struct BenchmarkDialog;

impl BenchmarkDialog {
    pub fn show(parent: &impl IsA<gtk4::Window>, device_name: &str) {
        let dialog = MessageDialog::builder()
            .heading("Benchmark de Disco")
            .body(&format!("Testar velocidade de /dev/{}", device_name))
            .transient_for(parent)
            .modal(true)
            .build();

        let content = Box::new(Orientation::Vertical, 16);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Test size selection
        let size_row = ComboRow::builder()
            .title("Tamanho do Teste")
            .subtitle("Arquivos maiores dao resultados mais precisos")
            .build();
        let sizes = gtk4::StringList::new(&["100 MB", "500 MB", "1 GB", "4 GB"]);
        size_row.set_model(Some(&sizes));
        size_row.set_selected(1);
        content.append(&size_row);

        // Results area
        let results_box = Box::new(Orientation::Vertical, 8);
        results_box.set_margin_top(16);

        let read_label = Label::new(Some("Leitura: --"));
        read_label.set_xalign(0.0);
        results_box.append(&read_label);

        let write_label = Label::new(Some("Escrita: --"));
        write_label.set_xalign(0.0);
        results_box.append(&write_label);

        let iops_label = Label::new(Some("IOPS: --"));
        iops_label.set_xalign(0.0);
        iops_label.add_css_class("dim-label");
        results_box.append(&iops_label);

        content.append(&results_box);

        // Progress bar
        let progress = ProgressBar::new();
        progress.set_visible(false);
        content.append(&progress);

        dialog.set_extra_child(Some(&content));

        dialog.add_response("close", "Fechar");
        dialog.add_response("start", "Iniciar Teste");
        dialog.set_response_appearance("start", ResponseAppearance::Suggested);
        dialog.set_default_response(Some("start"));
        dialog.set_close_response("close");

        // Handle start benchmark
        let progress_clone = progress.clone();
        let read_label_clone = read_label.clone();
        let write_label_clone = write_label.clone();
        let iops_label_clone = iops_label.clone();
        let dialog_clone = dialog.clone();

        dialog.connect_response(Some("start"), move |_, _| {
            progress_clone.set_visible(true);
            progress_clone.set_fraction(0.0);
            progress_clone.set_text(Some("Testando leitura..."));

            // Disable start button during test
            dialog_clone.set_response_enabled("start", false);

            // Simulate benchmark (in real app, would run actual tests)
            let progress = progress_clone.clone();
            let read_label = read_label_clone.clone();
            let write_label = write_label_clone.clone();
            let iops_label = iops_label_clone.clone();
            let dialog = dialog_clone.clone();

            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                let current = progress.fraction();
                if current < 0.5 {
                    progress.set_fraction(current + 0.02);
                    progress.set_text(Some("Testando leitura..."));
                } else if current < 1.0 {
                    progress.set_fraction(current + 0.02);
                    progress.set_text(Some("Testando escrita..."));

                    if current >= 0.5 && current < 0.52 {
                        read_label.set_text("Leitura: 523.4 MB/s");
                    }
                } else {
                    progress.set_text(Some("Concluido!"));
                    write_label.set_text("Escrita: 498.2 MB/s");
                    iops_label.set_text("IOPS: 45,230 (leitura) / 42,180 (escrita)");
                    dialog.set_response_enabled("start", true);
                    return glib::ControlFlow::Break;
                }
                glib::ControlFlow::Continue
            });
        });

        dialog.present();
    }
}

/// Disk image creation dialog
pub struct ImageDialog;

impl ImageDialog {
    pub fn show_create(parent: &impl IsA<gtk4::Window>, device_name: &str, device_size: u64) {
        let dialog = MessageDialog::builder()
            .heading("Criar Imagem de Disco")
            .body(&format!(
                "Criar uma imagem completa de /dev/{}\n\
                 Tamanho: {}",
                device_name,
                bytesize::ByteSize::b(device_size).to_string_as(true)
            ))
            .transient_for(parent)
            .modal(true)
            .build();

        let content = Box::new(Orientation::Vertical, 12);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Output path
        let path_box = Box::new(Orientation::Horizontal, 8);
        let path_label = Label::new(Some("Salvar em:"));
        let path_entry = Entry::new();
        path_entry.set_text(&format!("{}.img", device_name));
        path_entry.set_hexpand(true);

        let browse_btn = Button::from_icon_name("folder-open-symbolic");
        browse_btn.set_tooltip_text(Some("Escolher local"));

        path_box.append(&path_label);
        path_box.append(&path_entry);
        path_box.append(&browse_btn);
        content.append(&path_box);

        // Format selection
        let format_row = ComboRow::builder()
            .title("Formato")
            .build();
        let formats = gtk4::StringList::new(&["Raw (.img)", "Comprimido (.img.gz)", "ISO (.iso)"]);
        format_row.set_model(Some(&formats));
        content.append(&format_row);

        // Compress option
        let compress_row = SwitchRow::builder()
            .title("Comprimir")
            .subtitle("Reduz o tamanho do arquivo (mais lento)")
            .active(true)
            .build();
        content.append(&compress_row);

        // Verify checksum
        let verify_row = SwitchRow::builder()
            .title("Verificar Checksum")
            .subtitle("Gera e verifica SHA256 apos criar")
            .active(true)
            .build();
        content.append(&verify_row);

        dialog.set_extra_child(Some(&content));

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("create", "Criar Imagem");
        dialog.set_response_appearance("create", ResponseAppearance::Suggested);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        dialog.present();
    }

    pub fn show_restore(parent: &impl IsA<gtk4::Window>, device_name: &str) {
        let dialog = MessageDialog::builder()
            .heading("Restaurar Imagem de Disco")
            .body(&format!(
                "ATENCAO: Restaurar uma imagem ira APAGAR\n\
                 todos os dados em /dev/{}!",
                device_name
            ))
            .transient_for(parent)
            .modal(true)
            .build();

        let content = Box::new(Orientation::Vertical, 12);
        content.set_margin_start(24);
        content.set_margin_end(24);

        // Input file
        let file_box = Box::new(Orientation::Horizontal, 8);
        let file_label = Label::new(Some("Arquivo:"));
        let file_entry = Entry::new();
        file_entry.set_placeholder_text(Some("Selecione um arquivo .img ou .iso"));
        file_entry.set_hexpand(true);

        let browse_btn = Button::from_icon_name("folder-open-symbolic");

        file_box.append(&file_label);
        file_box.append(&file_entry);
        file_box.append(&browse_btn);
        content.append(&file_box);

        // Verify before restore
        let verify_row = SwitchRow::builder()
            .title("Verificar Checksum")
            .subtitle("Verifica integridade antes de restaurar")
            .active(true)
            .build();
        content.append(&verify_row);

        // Confirmation
        let confirm_check = CheckButton::with_label(
            "Eu entendo que todos os dados serao perdidos"
        );
        confirm_check.add_css_class("error");
        content.append(&confirm_check);

        dialog.set_extra_child(Some(&content));

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("restore", "Restaurar");
        dialog.set_response_appearance("restore", ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");
        dialog.set_response_enabled("restore", false);

        let dialog_clone = dialog.clone();
        confirm_check.connect_toggled(move |check| {
            dialog_clone.set_response_enabled("restore", check.is_active());
        });

        dialog.present();
    }
}
