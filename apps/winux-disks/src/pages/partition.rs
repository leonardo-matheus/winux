//! Partition management page

use gtk4::prelude::*;
use gtk4::{Box, Orientation, ScrolledWindow, Label, Button, Entry, SpinButton, Adjustment};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, ComboRow, SwitchRow, MessageDialog, ResponseAppearance};

use crate::backend::{DiskManager, PartedManager};

/// Partition management page for creating, deleting, and resizing partitions
pub struct PartitionPage {
    widget: ScrolledWindow,
}

impl PartitionPage {
    pub fn new(disk_name: &str) -> Self {
        let page = PreferencesPage::new();
        page.set_title("Gerenciar Particoes");

        let disk_manager = DiskManager::new();

        // Disk info header
        let disk_info = PreferencesGroup::builder()
            .title("Disco Selecionado")
            .build();

        if let Some(disk) = disk_manager.get_device(disk_name) {
            let info_row = ActionRow::builder()
                .title(&format!("/dev/{}", disk.name))
                .subtitle(&format!(
                    "{} - {}",
                    if disk.model.is_empty() { "Disco" } else { &disk.model },
                    bytesize::ByteSize::b(disk.size).to_string_as(true)
                ))
                .build();
            info_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));
            disk_info.add(&info_row);
        }

        page.add(&disk_info);

        // Partition table type
        let table_group = PreferencesGroup::builder()
            .title("Tabela de Particoes")
            .description("Tipo de esquema de particao do disco")
            .build();

        let table_row = ComboRow::builder()
            .title("Tipo de Tabela")
            .subtitle("GPT e recomendado para discos modernos")
            .build();
        let tables = gtk4::StringList::new(&["GPT (GUID Partition Table)", "MBR (Master Boot Record)"]);
        table_row.set_model(Some(&tables));
        table_group.add(&table_row);

        let new_table_row = ActionRow::builder()
            .title("Criar Nova Tabela de Particoes")
            .subtitle("ATENCAO: Isso apagara todas as particoes existentes!")
            .activatable(true)
            .build();
        new_table_row.add_prefix(&gtk4::Image::from_icon_name("edit-clear-all-symbolic"));

        let new_table_btn = Button::with_label("Criar");
        new_table_btn.add_css_class("destructive-action");
        new_table_btn.set_valign(gtk4::Align::Center);
        new_table_row.add_suffix(&new_table_btn);

        // Warning dialog for new partition table
        let disk_name_clone = disk_name.to_string();
        new_table_btn.connect_clicked(move |btn| {
            if let Some(window) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                let dialog = MessageDialog::builder()
                    .heading("Criar Nova Tabela de Particoes")
                    .body(&format!(
                        "Isso ira APAGAR PERMANENTEMENTE todas as particoes e dados em /dev/{}.\n\n\
                        Esta acao NAO PODE ser desfeita. Tem certeza?",
                        disk_name_clone
                    ))
                    .transient_for(&window)
                    .modal(true)
                    .build();

                dialog.add_response("cancel", "Cancelar");
                dialog.add_response("confirm", "Criar Tabela");
                dialog.set_response_appearance("confirm", ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                dialog.present();
            }
        });

        table_group.add(&new_table_row);
        page.add(&table_group);

        // Create partition section
        let create_group = PreferencesGroup::builder()
            .title("Criar Particao")
            .description("Adicionar uma nova particao ao espaco livre")
            .build();

        // Partition name/label
        let label_row = ActionRow::builder()
            .title("Rotulo")
            .subtitle("Nome opcional para a particao")
            .build();
        let label_entry = Entry::new();
        label_entry.set_placeholder_text(Some("Minha Particao"));
        label_entry.set_valign(gtk4::Align::Center);
        label_entry.set_width_request(200);
        label_row.add_suffix(&label_entry);
        create_group.add(&label_row);

        // Partition size
        let size_row = ActionRow::builder()
            .title("Tamanho")
            .subtitle("Tamanho da particao em GB")
            .build();

        let size_adj = Adjustment::new(10.0, 0.1, 1000.0, 0.1, 1.0, 0.0);
        let size_spin = SpinButton::new(Some(&size_adj), 0.1, 1);
        size_spin.set_valign(gtk4::Align::Center);
        size_row.add_suffix(&size_spin);

        let size_unit = ComboRow::builder()
            .title("Unidade")
            .build();
        let units = gtk4::StringList::new(&["GB", "MB", "% do espaco livre"]);
        size_unit.set_model(Some(&units));

        create_group.add(&size_row);

        // Filesystem type
        let fs_row = ComboRow::builder()
            .title("Sistema de Arquivos")
            .subtitle("Formato da particao")
            .build();
        let filesystems = gtk4::StringList::new(&[
            "ext4 (Linux - Recomendado)",
            "btrfs (Linux - Snapshots)",
            "xfs (Linux - Alto desempenho)",
            "ntfs (Windows)",
            "FAT32 (Universal - max 4GB por arquivo)",
            "exFAT (Universal - sem limite de arquivo)",
            "swap (Linux - Memoria virtual)",
        ]);
        fs_row.set_model(Some(&filesystems));
        create_group.add(&fs_row);

        // Partition type (for GPT)
        let part_type_row = ComboRow::builder()
            .title("Tipo de Particao")
            .subtitle("Tipo GPT da particao")
            .build();
        let part_types = gtk4::StringList::new(&[
            "Dados Linux",
            "Sistema EFI",
            "Microsoft Basic Data",
            "Linux Swap",
            "Linux Home",
            "Linux Root (x86-64)",
        ]);
        part_type_row.set_model(Some(&part_types));
        create_group.add(&part_type_row);

        // Encrypt option
        let encrypt_row = SwitchRow::builder()
            .title("Criptografar Particao")
            .subtitle("Usar LUKS para proteger dados")
            .active(false)
            .build();
        create_group.add(&encrypt_row);

        // Create button
        let create_btn_row = ActionRow::new();
        let create_btn = Button::with_label("Criar Particao");
        create_btn.add_css_class("suggested-action");
        create_btn.set_halign(gtk4::Align::End);
        create_btn.set_margin_top(12);
        create_btn.set_margin_bottom(12);

        create_btn.connect_clicked(|btn| {
            if let Some(window) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                let dialog = MessageDialog::builder()
                    .heading("Criar Particao")
                    .body("Deseja criar a particao com as configuracoes selecionadas?\n\n\
                           Certifique-se de que os dados importantes estao em backup.")
                    .transient_for(&window)
                    .modal(true)
                    .build();

                dialog.add_response("cancel", "Cancelar");
                dialog.add_response("confirm", "Criar");
                dialog.set_response_appearance("confirm", ResponseAppearance::Suggested);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                dialog.present();
            }
        });

        create_btn_row.add_suffix(&create_btn);
        create_group.add(&create_btn_row);

        page.add(&create_group);

        // Delete partition section
        let delete_group = PreferencesGroup::builder()
            .title("Excluir Particao")
            .description("Selecione uma particao existente para excluir")
            .build();

        let partitions = disk_manager.get_partitions(disk_name);
        for part in &partitions {
            let part_row = ActionRow::builder()
                .title(&format!("/dev/{}", part.name))
                .subtitle(&format!(
                    "{} - {}",
                    bytesize::ByteSize::b(part.size).to_string_as(true),
                    part.filesystem.as_deref().unwrap_or("Sem formato")
                ))
                .build();

            let delete_btn = Button::from_icon_name("user-trash-symbolic");
            delete_btn.add_css_class("flat");
            delete_btn.add_css_class("destructive-action");
            delete_btn.set_tooltip_text(Some("Excluir particao"));
            delete_btn.set_valign(gtk4::Align::Center);

            let part_name = part.name.clone();
            delete_btn.connect_clicked(move |btn| {
                if let Some(window) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                    let dialog = MessageDialog::builder()
                        .heading("Excluir Particao")
                        .body(&format!(
                            "Isso ira APAGAR PERMANENTEMENTE a particao /dev/{} e todos os seus dados.\n\n\
                            Esta acao NAO PODE ser desfeita. Tem certeza?",
                            part_name
                        ))
                        .transient_for(&window)
                        .modal(true)
                        .build();

                    dialog.add_response("cancel", "Cancelar");
                    dialog.add_response("delete", "Excluir Permanentemente");
                    dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
                    dialog.set_default_response(Some("cancel"));
                    dialog.set_close_response("cancel");

                    dialog.present();
                }
            });

            part_row.add_suffix(&delete_btn);
            delete_group.add(&part_row);
        }

        if partitions.is_empty() {
            let empty_row = ActionRow::builder()
                .title("Nenhuma particao")
                .subtitle("Este disco nao possui particoes para excluir")
                .build();
            empty_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
            delete_group.add(&empty_row);
        }

        page.add(&delete_group);

        // Resize partition section (advanced)
        let resize_group = PreferencesGroup::builder()
            .title("Redimensionar Particao")
            .description("Aumentar ou diminuir o tamanho de uma particao")
            .build();

        let resize_info = ActionRow::builder()
            .title("Redimensionamento de Particao")
            .subtitle("Selecione uma particao e defina o novo tamanho. A particao deve estar desmontada.")
            .build();
        resize_info.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        resize_group.add(&resize_info);

        let resize_part_row = ComboRow::builder()
            .title("Particao")
            .subtitle("Selecione a particao para redimensionar")
            .build();

        let part_names: Vec<&str> = partitions.iter()
            .map(|p| p.name.as_str())
            .collect();
        let part_list = gtk4::StringList::new(&part_names);
        resize_part_row.set_model(Some(&part_list));
        resize_group.add(&resize_part_row);

        let new_size_row = ActionRow::builder()
            .title("Novo Tamanho")
            .subtitle("Tamanho desejado em GB")
            .build();

        let new_size_adj = Adjustment::new(10.0, 0.1, 1000.0, 0.1, 1.0, 0.0);
        let new_size_spin = SpinButton::new(Some(&new_size_adj), 0.1, 1);
        new_size_spin.set_valign(gtk4::Align::Center);
        new_size_row.add_suffix(&new_size_spin);
        resize_group.add(&new_size_row);

        let resize_btn_row = ActionRow::new();
        let resize_btn = Button::with_label("Redimensionar");
        resize_btn.add_css_class("suggested-action");
        resize_btn.set_halign(gtk4::Align::End);
        resize_btn.set_margin_top(12);
        resize_btn.set_margin_bottom(12);

        resize_btn.connect_clicked(|btn| {
            if let Some(window) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                let dialog = MessageDialog::builder()
                    .heading("Redimensionar Particao")
                    .body("AVISO: Redimensionar particoes pode causar perda de dados.\n\n\
                           Faca backup antes de continuar. Deseja prosseguir?")
                    .transient_for(&window)
                    .modal(true)
                    .build();

                dialog.add_response("cancel", "Cancelar");
                dialog.add_response("resize", "Redimensionar");
                dialog.set_response_appearance("resize", ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                dialog.present();
            }
        });

        resize_btn_row.add_suffix(&resize_btn);
        resize_group.add(&resize_btn_row);

        page.add(&resize_group);

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

impl Default for PartitionPage {
    fn default() -> Self {
        Self::new("sda")
    }
}
