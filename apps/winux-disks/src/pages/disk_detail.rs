//! Disk detail page - Shows detailed information about a specific disk

use gtk4::prelude::*;
use gtk4::{Box, Orientation, ScrolledWindow, Label, Button, Grid};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, ExpanderRow};

use crate::backend::{DiskManager, BlockDevice, SmartInfo};
use crate::ui::DiskGraph;

/// Disk detail page showing comprehensive disk information
pub struct DiskDetailPage {
    widget: ScrolledWindow,
}

impl DiskDetailPage {
    pub fn new(disk_name: &str) -> Self {
        let page = PreferencesPage::new();
        page.set_title("Detalhes do Disco");

        let disk_manager = DiskManager::new();

        if let Some(disk) = disk_manager.get_device(disk_name) {
            // Device info section
            let info_group = PreferencesGroup::builder()
                .title("Informacoes do Dispositivo")
                .build();

            let device_row = ActionRow::builder()
                .title("Dispositivo")
                .subtitle(&format!("/dev/{}", disk.name))
                .build();
            info_group.add(&device_row);

            let model_row = ActionRow::builder()
                .title("Modelo")
                .subtitle(if disk.model.is_empty() { "Desconhecido" } else { &disk.model })
                .build();
            info_group.add(&model_row);

            let serial_row = ActionRow::builder()
                .title("Numero de Serie")
                .subtitle(if disk.serial.is_empty() { "N/A" } else { &disk.serial })
                .build();
            info_group.add(&serial_row);

            let size_row = ActionRow::builder()
                .title("Capacidade")
                .subtitle(&format!(
                    "{} ({} bytes)",
                    bytesize::ByteSize::b(disk.size).to_string_as(true),
                    disk.size
                ))
                .build();
            info_group.add(&size_row);

            let type_row = ActionRow::builder()
                .title("Tipo")
                .subtitle(if disk.is_rotational { "HDD (Disco Rigido)" } else { "SSD/NVMe (Estado Solido)" })
                .build();
            info_group.add(&type_row);

            let removable_row = ActionRow::builder()
                .title("Removivel")
                .subtitle(if disk.is_removable { "Sim" } else { "Nao" })
                .build();
            info_group.add(&removable_row);

            page.add(&info_group);

            // SMART section
            let smart_group = PreferencesGroup::builder()
                .title("SMART (Auto-Monitoramento)")
                .description("Status de saude do disco")
                .build();

            if let Some(smart) = disk_manager.get_smart_info(&disk.name) {
                let health_row = ActionRow::builder()
                    .title("Status de Saude")
                    .subtitle(if smart.healthy { "Saudavel" } else { "Atencao - Verifique o disco" })
                    .build();

                let health_icon = if smart.healthy {
                    let icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
                    icon.add_css_class("success");
                    icon
                } else {
                    let icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
                    icon.add_css_class("error");
                    icon
                };
                health_row.add_prefix(&health_icon);
                smart_group.add(&health_row);

                if let Some(temp) = smart.temperature {
                    let temp_row = ActionRow::builder()
                        .title("Temperatura")
                        .subtitle(&format!("{}C", temp))
                        .build();

                    let temp_icon = if temp < 45 {
                        let icon = gtk4::Image::from_icon_name("temperature-symbolic");
                        icon.add_css_class("success");
                        icon
                    } else if temp < 55 {
                        let icon = gtk4::Image::from_icon_name("temperature-symbolic");
                        icon.add_css_class("warning");
                        icon
                    } else {
                        let icon = gtk4::Image::from_icon_name("temperature-symbolic");
                        icon.add_css_class("error");
                        icon
                    };
                    temp_row.add_prefix(&temp_icon);
                    smart_group.add(&temp_row);
                }

                if let Some(hours) = smart.power_on_hours {
                    let hours_row = ActionRow::builder()
                        .title("Horas Ligado")
                        .subtitle(&format!("{} horas ({:.1} dias)", hours, hours as f64 / 24.0))
                        .build();
                    hours_row.add_prefix(&gtk4::Image::from_icon_name("preferences-system-time-symbolic"));
                    smart_group.add(&hours_row);
                }

                if let Some(cycles) = smart.power_cycle_count {
                    let cycles_row = ActionRow::builder()
                        .title("Ciclos de Energia")
                        .subtitle(&format!("{}", cycles))
                        .build();
                    cycles_row.add_prefix(&gtk4::Image::from_icon_name("system-reboot-symbolic"));
                    smart_group.add(&cycles_row);
                }

                if let Some(errors) = smart.reallocated_sectors {
                    let errors_row = ActionRow::builder()
                        .title("Setores Realocados")
                        .subtitle(&format!("{}", errors))
                        .build();

                    let error_icon = if errors == 0 {
                        let icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
                        icon.add_css_class("success");
                        icon
                    } else {
                        let icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
                        icon.add_css_class("warning");
                        icon
                    };
                    errors_row.add_prefix(&error_icon);
                    smart_group.add(&errors_row);
                }

                // Self-test button
                let test_row = ActionRow::builder()
                    .title("Executar Auto-Teste")
                    .subtitle("Inicia um teste SMART curto ou longo")
                    .activatable(true)
                    .build();
                test_row.add_prefix(&gtk4::Image::from_icon_name("system-run-symbolic"));

                let test_btn = Button::with_label("Iniciar");
                test_btn.set_valign(gtk4::Align::Center);
                test_btn.add_css_class("suggested-action");
                test_row.add_suffix(&test_btn);
                smart_group.add(&test_row);

            } else {
                let no_smart = ActionRow::builder()
                    .title("SMART nao disponivel")
                    .subtitle("Este dispositivo nao suporta SMART ou nao esta acessivel")
                    .build();
                no_smart.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
                smart_group.add(&no_smart);
            }

            page.add(&smart_group);

            // Partitions section
            let partitions = disk_manager.get_partitions(&disk.name);
            let part_group = PreferencesGroup::builder()
                .title("Particoes")
                .description(&format!("{} particao(oes) encontrada(s)", partitions.len()))
                .build();

            // Visual representation
            if !partitions.is_empty() {
                let graph = DiskGraph::new(&partitions, disk.size);
                part_group.add(&graph.widget());
            }

            // Partition list
            for part in &partitions {
                let expander = ExpanderRow::builder()
                    .title(&format!("/dev/{}", part.name))
                    .subtitle(&format!(
                        "{} - {}",
                        bytesize::ByteSize::b(part.size).to_string_as(true),
                        part.filesystem.as_deref().unwrap_or("Desconhecido")
                    ))
                    .build();

                // Label
                if let Some(ref label) = part.label {
                    let label_row = ActionRow::builder()
                        .title("Rotulo")
                        .subtitle(label)
                        .build();
                    expander.add_row(&label_row);
                }

                // UUID
                if let Some(ref uuid) = part.uuid {
                    let uuid_row = ActionRow::builder()
                        .title("UUID")
                        .subtitle(uuid)
                        .build();
                    expander.add_row(&uuid_row);
                }

                // Mount point
                if let Some(ref mount) = part.mount_point {
                    let mount_row = ActionRow::builder()
                        .title("Ponto de Montagem")
                        .subtitle(mount)
                        .build();
                    expander.add_row(&mount_row);
                }

                // Actions
                let actions_row = ActionRow::builder()
                    .title("Acoes")
                    .build();

                let actions_box = Box::new(Orientation::Horizontal, 8);
                actions_box.set_halign(gtk4::Align::End);

                if part.mount_point.is_some() {
                    let unmount_btn = Button::with_label("Desmontar");
                    unmount_btn.add_css_class("flat");
                    actions_box.append(&unmount_btn);
                } else if part.filesystem.is_some() {
                    let mount_btn = Button::with_label("Montar");
                    mount_btn.add_css_class("flat");
                    actions_box.append(&mount_btn);
                }

                let format_btn = Button::with_label("Formatar");
                format_btn.add_css_class("flat");
                format_btn.add_css_class("destructive-action");
                actions_box.append(&format_btn);

                actions_row.add_suffix(&actions_box);
                expander.add_row(&actions_row);

                part_group.add(&expander);
            }

            // Create partition button
            let create_row = ActionRow::builder()
                .title("Criar Nova Particao")
                .subtitle("Adicionar uma nova particao ao disco")
                .activatable(true)
                .build();
            create_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
            create_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            part_group.add(&create_row);

            page.add(&part_group);

            // Actions section
            let actions_group = PreferencesGroup::builder()
                .title("Acoes do Disco")
                .build();

            let benchmark_row = ActionRow::builder()
                .title("Benchmark")
                .subtitle("Testar velocidade de leitura/escrita")
                .activatable(true)
                .build();
            benchmark_row.add_prefix(&gtk4::Image::from_icon_name("utilities-system-monitor-symbolic"));
            benchmark_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            actions_group.add(&benchmark_row);

            let image_row = ActionRow::builder()
                .title("Criar Imagem")
                .subtitle("Criar uma copia completa do disco")
                .activatable(true)
                .build();
            image_row.add_prefix(&gtk4::Image::from_icon_name("drive-optical-symbolic"));
            image_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            actions_group.add(&image_row);

            let format_row = ActionRow::builder()
                .title("Formatar Disco Inteiro")
                .subtitle("CUIDADO: Apaga todos os dados!")
                .activatable(true)
                .build();
            format_row.add_prefix(&gtk4::Image::from_icon_name("edit-clear-all-symbolic"));
            let warning_icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
            warning_icon.add_css_class("warning");
            format_row.add_suffix(&warning_icon);
            actions_group.add(&format_row);

            page.add(&actions_group);
        } else {
            // Disk not found
            let error_group = PreferencesGroup::new();
            let error_row = ActionRow::builder()
                .title("Disco nao encontrado")
                .subtitle(&format!("O dispositivo {} nao foi encontrado", disk_name))
                .build();
            error_row.add_prefix(&gtk4::Image::from_icon_name("dialog-error-symbolic"));
            error_group.add(&error_row);
            page.add(&error_group);
        }

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

impl Default for DiskDetailPage {
    fn default() -> Self {
        Self::new("sda")
    }
}
