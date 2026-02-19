//! Format page - Format disks and partitions

use gtk4::prelude::*;
use gtk4::{Box, Orientation, ScrolledWindow, Label, Button, Entry, CheckButton, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, ComboRow, SwitchRow, MessageDialog, ResponseAppearance};

use crate::backend::DiskManager;

/// Format page for formatting disks and partitions
pub struct FormatPage {
    widget: ScrolledWindow,
}

impl FormatPage {
    pub fn new(device_name: &str) -> Self {
        let page = PreferencesPage::new();
        page.set_title("Formatar");

        let disk_manager = DiskManager::new();

        // Warning banner
        let warning_group = PreferencesGroup::new();

        let warning_row = ActionRow::builder()
            .title("ATENCAO: Operacao Destrutiva")
            .subtitle("Formatar ira APAGAR TODOS OS DADOS do dispositivo selecionado!")
            .build();

        let warning_icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
        warning_icon.add_css_class("error");
        warning_row.add_prefix(&warning_icon);
        warning_group.add(&warning_row);

        page.add(&warning_group);

        // Device info
        let device_group = PreferencesGroup::builder()
            .title("Dispositivo a Formatar")
            .build();

        if let Some(device) = disk_manager.get_device(device_name)
            .or_else(|| disk_manager.get_partition_info(device_name))
        {
            let device_row = ActionRow::builder()
                .title(&format!("/dev/{}", device.name))
                .subtitle(&format!(
                    "{} - {}",
                    if device.model.is_empty() { "Particao" } else { &device.model },
                    bytesize::ByteSize::b(device.size).to_string_as(true)
                ))
                .build();
            device_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));
            device_group.add(&device_row);
        }

        page.add(&device_group);

        // Format options
        let format_group = PreferencesGroup::builder()
            .title("Opcoes de Formatacao")
            .description("Configure o formato do dispositivo")
            .build();

        // Filesystem selection
        let fs_row = ComboRow::builder()
            .title("Sistema de Arquivos")
            .subtitle("Escolha o formato mais adequado para seu uso")
            .build();
        let filesystems = gtk4::StringList::new(&[
            "ext4 - Linux (Recomendado)",
            "btrfs - Linux (Snapshots, Compressao)",
            "xfs - Linux (Alto Desempenho)",
            "ntfs - Windows (Compativel)",
            "FAT32 - Universal (Max 4GB/arquivo)",
            "exFAT - Universal (Sem limite)",
        ]);
        fs_row.set_model(Some(&filesystems));
        format_group.add(&fs_row);

        // Volume label
        let label_row = ActionRow::builder()
            .title("Rotulo do Volume")
            .subtitle("Nome que aparecera ao montar o dispositivo")
            .build();
        let label_entry = Entry::new();
        label_entry.set_placeholder_text(Some("Meu Disco"));
        label_entry.set_valign(gtk4::Align::Center);
        label_entry.set_width_request(200);
        label_row.add_suffix(&label_entry);
        format_group.add(&label_row);

        page.add(&format_group);

        // Advanced options
        let advanced_group = PreferencesGroup::builder()
            .title("Opcoes Avancadas")
            .build();

        // Quick format
        let quick_row = SwitchRow::builder()
            .title("Formatacao Rapida")
            .subtitle("Apenas limpa a tabela de arquivos (mais rapido)")
            .active(true)
            .build();
        advanced_group.add(&quick_row);

        // Overwrite with zeros
        let zero_row = SwitchRow::builder()
            .title("Sobrescrever com Zeros")
            .subtitle("Apaga dados de forma segura (muito lento)")
            .active(false)
            .build();
        advanced_group.add(&zero_row);

        // Check for bad sectors
        let check_row = SwitchRow::builder()
            .title("Verificar Setores Defeituosos")
            .subtitle("Procura por erros no disco (lento)")
            .active(false)
            .build();
        advanced_group.add(&check_row);

        // Encryption
        let encrypt_row = SwitchRow::builder()
            .title("Criptografar (LUKS)")
            .subtitle("Protege dados com senha")
            .active(false)
            .build();
        advanced_group.add(&encrypt_row);

        // Encryption password (shown when encryption is enabled)
        let pass_row = ActionRow::builder()
            .title("Senha de Criptografia")
            .subtitle("Minimo 8 caracteres")
            .build();
        let pass_entry = gtk4::PasswordEntry::new();
        pass_entry.set_show_peek_icon(true);
        pass_entry.set_valign(gtk4::Align::Center);
        pass_entry.set_width_request(200);
        pass_row.add_suffix(&pass_entry);
        advanced_group.add(&pass_row);

        let confirm_pass_row = ActionRow::builder()
            .title("Confirmar Senha")
            .build();
        let confirm_entry = gtk4::PasswordEntry::new();
        confirm_entry.set_show_peek_icon(true);
        confirm_entry.set_valign(gtk4::Align::Center);
        confirm_entry.set_width_request(200);
        confirm_pass_row.add_suffix(&confirm_entry);
        advanced_group.add(&confirm_pass_row);

        page.add(&advanced_group);

        // Filesystem-specific options
        let ext4_group = PreferencesGroup::builder()
            .title("Opcoes ext4")
            .description("Configuracoes especificas para ext4")
            .build();

        let journal_row = SwitchRow::builder()
            .title("Journaling")
            .subtitle("Protege contra corrupcao em caso de falha")
            .active(true)
            .build();
        ext4_group.add(&journal_row);

        let reserved_row = ActionRow::builder()
            .title("Espaco Reservado")
            .subtitle("Porcentagem reservada para o sistema")
            .build();

        let reserved_adj = gtk4::Adjustment::new(5.0, 0.0, 10.0, 0.5, 1.0, 0.0);
        let reserved_spin = gtk4::SpinButton::new(Some(&reserved_adj), 0.5, 1);
        reserved_spin.set_valign(gtk4::Align::Center);

        let percent_label = Label::new(Some("%"));
        percent_label.set_margin_start(4);

        let reserved_box = Box::new(Orientation::Horizontal, 4);
        reserved_box.append(&reserved_spin);
        reserved_box.append(&percent_label);
        reserved_row.add_suffix(&reserved_box);
        ext4_group.add(&reserved_row);

        page.add(&ext4_group);

        // btrfs options
        let btrfs_group = PreferencesGroup::builder()
            .title("Opcoes btrfs")
            .description("Configuracoes especificas para btrfs")
            .build();

        let compress_row = ComboRow::builder()
            .title("Compressao")
            .subtitle("Comprime dados automaticamente")
            .build();
        let compress_opts = gtk4::StringList::new(&["Desabilitado", "lzo (Rapido)", "zstd (Balanceado)", "zlib (Maximo)"]);
        compress_row.set_model(Some(&compress_opts));
        compress_row.set_selected(2);
        btrfs_group.add(&compress_row);

        let subvol_row = SwitchRow::builder()
            .title("Criar Subvolumes Padrao")
            .subtitle("Cria @ e @home para snapshots")
            .active(true)
            .build();
        btrfs_group.add(&subvol_row);

        page.add(&btrfs_group);

        // Confirmation
        let confirm_group = PreferencesGroup::builder()
            .title("Confirmacao")
            .build();

        let confirm_check = CheckButton::with_label(
            "Eu entendo que TODOS OS DADOS serao permanentemente apagados"
        );
        confirm_check.add_css_class("error");

        let confirm_box = Box::new(Orientation::Vertical, 8);
        confirm_box.set_margin_start(12);
        confirm_box.set_margin_end(12);
        confirm_box.set_margin_top(12);
        confirm_box.set_margin_bottom(12);
        confirm_box.append(&confirm_check);

        let format_btn = Button::with_label("Formatar Dispositivo");
        format_btn.add_css_class("destructive-action");
        format_btn.add_css_class("pill");
        format_btn.set_halign(gtk4::Align::Center);
        format_btn.set_sensitive(false);

        // Enable button only when checkbox is checked
        let format_btn_clone = format_btn.clone();
        confirm_check.connect_toggled(move |check| {
            format_btn_clone.set_sensitive(check.is_active());
        });

        let device_name_clone = device_name.to_string();
        format_btn.connect_clicked(move |btn| {
            if let Some(window) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                let dialog = MessageDialog::builder()
                    .heading("Confirmacao Final")
                    .body(&format!(
                        "Voce esta prestes a formatar /dev/{}.\n\n\
                        Esta e sua ULTIMA CHANCE de cancelar.\n\n\
                        Todos os dados serao PERMANENTEMENTE PERDIDOS.\n\n\
                        Deseja continuar?",
                        device_name_clone
                    ))
                    .transient_for(&window)
                    .modal(true)
                    .build();

                dialog.add_response("cancel", "Cancelar");
                dialog.add_response("format", "FORMATAR AGORA");
                dialog.set_response_appearance("format", ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                let device = device_name_clone.clone();
                dialog.connect_response(Some("format"), move |_, _| {
                    tracing::info!("User confirmed format of /dev/{}", device);
                    // Format operation would be executed here
                });

                dialog.present();
            }
        });

        confirm_box.append(&format_btn);

        let confirm_row = ActionRow::new();
        confirm_row.set_child(Some(&confirm_box));
        confirm_group.add(&confirm_row);

        page.add(&confirm_group);

        // Progress (shown during format)
        let progress_group = PreferencesGroup::builder()
            .title("Progresso")
            .build();

        let progress_bar = ProgressBar::new();
        progress_bar.set_show_text(true);
        progress_bar.set_text(Some("Aguardando..."));
        progress_bar.set_fraction(0.0);
        progress_bar.set_margin_start(12);
        progress_bar.set_margin_end(12);
        progress_bar.set_margin_top(12);
        progress_bar.set_margin_bottom(12);

        let status_label = Label::new(Some("Selecione as opcoes e clique em Formatar"));
        status_label.add_css_class("dim-label");
        status_label.set_margin_bottom(12);

        let progress_box = Box::new(Orientation::Vertical, 8);
        progress_box.append(&progress_bar);
        progress_box.append(&status_label);

        let progress_row = ActionRow::new();
        progress_row.set_child(Some(&progress_box));
        progress_group.add(&progress_row);

        page.add(&progress_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}

impl Default for FormatPage {
    fn default() -> Self {
        Self::new("sda1")
    }
}
