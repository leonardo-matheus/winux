//! Overview page - Shows all disks and their status

use gtk4::prelude::*;
use gtk4::{Box, Orientation, ScrolledWindow, Label, ProgressBar, Grid, Frame};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, StatusPage};

use crate::backend::{DiskManager, BlockDevice};
use crate::ui::DiskGraph;

/// Overview page showing all disks
pub struct OverviewPage {
    widget: ScrolledWindow,
}

impl OverviewPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Visao Geral");

        // Load disk information
        let disk_manager = DiskManager::new();
        let devices = disk_manager.get_block_devices();

        // Filter to only show actual disks (not partitions)
        let disks: Vec<_> = devices.iter()
            .filter(|d| d.device_type == "disk" && !d.name.starts_with("loop"))
            .collect();

        // Summary section
        let summary_group = PreferencesGroup::builder()
            .title("Resumo")
            .description("Status geral dos dispositivos de armazenamento")
            .build();

        let total_disks = disks.len();
        let total_size: u64 = disks.iter().map(|d| d.size).sum();
        let healthy_count = disks.iter()
            .filter(|d| d.smart_healthy.unwrap_or(true))
            .count();

        let summary_row = ActionRow::builder()
            .title(&format!("{} Dispositivo(s) Detectado(s)", total_disks))
            .subtitle(&format!(
                "Capacidade total: {} | {} saudavel(is)",
                bytesize::ByteSize::b(total_size).to_string_as(true),
                healthy_count
            ))
            .build();
        summary_row.add_prefix(&gtk4::Image::from_icon_name("drive-multidisk-symbolic"));
        summary_group.add(&summary_row);

        page.add(&summary_group);

        // Disks section
        for disk in &disks {
            let group = Self::create_disk_group(disk, &disk_manager);
            page.add(&group);
        }

        // Optical drives
        let optical_drives: Vec<_> = devices.iter()
            .filter(|d| d.device_type == "rom")
            .collect();

        if !optical_drives.is_empty() {
            let optical_group = PreferencesGroup::builder()
                .title("Unidades Opticas")
                .build();

            for drive in optical_drives {
                let row = ActionRow::builder()
                    .title(&drive.name)
                    .subtitle(if drive.model.is_empty() {
                        "Unidade CD/DVD"
                    } else {
                        &drive.model
                    })
                    .build();
                row.add_prefix(&gtk4::Image::from_icon_name("drive-optical-symbolic"));
                optical_group.add(&row);
            }

            page.add(&optical_group);
        }

        // Loop devices (if any mounted)
        let loop_devices: Vec<_> = devices.iter()
            .filter(|d| d.device_type == "loop" && d.size > 0)
            .collect();

        if !loop_devices.is_empty() {
            let loop_group = PreferencesGroup::builder()
                .title("Dispositivos Loop")
                .description("Imagens de disco montadas")
                .build();

            for device in loop_devices {
                let size_str = bytesize::ByteSize::b(device.size).to_string_as(true);
                let row = ActionRow::builder()
                    .title(&device.name)
                    .subtitle(&size_str)
                    .build();
                row.add_prefix(&gtk4::Image::from_icon_name("media-optical-symbolic"));
                loop_group.add(&row);
            }

            page.add(&loop_group);
        }

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    fn create_disk_group(disk: &BlockDevice, manager: &DiskManager) -> PreferencesGroup {
        let icon_name = if disk.is_removable {
            "drive-removable-media-symbolic"
        } else if disk.name.starts_with("nvme") {
            "drive-harddisk-solidstate-symbolic"
        } else if disk.is_rotational {
            "drive-harddisk-symbolic"
        } else {
            "drive-harddisk-solidstate-symbolic"
        };

        let title = if disk.model.is_empty() {
            format!("{} ({})", disk.name, bytesize::ByteSize::b(disk.size).to_string_as(true))
        } else {
            format!("{} ({})", disk.model, bytesize::ByteSize::b(disk.size).to_string_as(true))
        };

        let group = PreferencesGroup::builder()
            .title(&title)
            .build();

        // Disk info row
        let info_row = ActionRow::builder()
            .title(&format!("/dev/{}", disk.name))
            .subtitle(&format!(
                "Serial: {} | Tipo: {}",
                if disk.serial.is_empty() { "N/A" } else { &disk.serial },
                if disk.is_rotational { "HDD" } else { "SSD/NVMe" }
            ))
            .build();
        info_row.add_prefix(&gtk4::Image::from_icon_name(icon_name));

        // SMART status indicator
        if let Some(healthy) = disk.smart_healthy {
            if healthy {
                let health_box = Box::new(Orientation::Horizontal, 4);
                let health_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
                health_icon.add_css_class("success");
                let health_label = Label::new(Some("SMART OK"));
                health_label.add_css_class("success");
                health_box.append(&health_icon);
                health_box.append(&health_label);
                info_row.add_suffix(&health_box);
            } else {
                let health_box = Box::new(Orientation::Horizontal, 4);
                let health_icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
                health_icon.add_css_class("warning");
                let health_label = Label::new(Some("SMART Warning"));
                health_label.add_css_class("warning");
                health_box.append(&health_icon);
                health_box.append(&health_label);
                info_row.add_suffix(&health_box);
            }
        }

        group.add(&info_row);

        // Partition visualization
        let partitions = manager.get_partitions(&disk.name);
        if !partitions.is_empty() {
            let graph = DiskGraph::new(&partitions, disk.size);
            group.add(&graph.widget());

            // Partition list
            for part in &partitions {
                let mount_info = if let Some(ref mp) = part.mount_point {
                    format!("Montado em {}", mp)
                } else {
                    "Nao montado".to_string()
                };

                let fs_info = if let Some(ref fs) = part.filesystem {
                    format!("{} - {}", fs, mount_info)
                } else {
                    mount_info
                };

                let part_row = ActionRow::builder()
                    .title(&format!("/dev/{}", part.name))
                    .subtitle(&format!(
                        "{} | {}",
                        bytesize::ByteSize::b(part.size).to_string_as(true),
                        fs_info
                    ))
                    .activatable(true)
                    .build();

                let fs_icon = match part.filesystem.as_deref() {
                    Some("ext4") | Some("ext3") | Some("ext2") => "drive-harddisk-symbolic",
                    Some("btrfs") => "drive-harddisk-symbolic",
                    Some("ntfs") | Some("ntfs3") => "drive-harddisk-symbolic",
                    Some("vfat") | Some("fat32") | Some("exfat") => "drive-removable-media-symbolic",
                    Some("swap") => "applications-system-symbolic",
                    Some("xfs") => "drive-harddisk-symbolic",
                    _ => "drive-harddisk-symbolic",
                };
                part_row.add_prefix(&gtk4::Image::from_icon_name(fs_icon));

                // Mount/unmount button
                if part.mount_point.is_some() {
                    let unmount_btn = gtk4::Button::from_icon_name("media-eject-symbolic");
                    unmount_btn.set_tooltip_text(Some("Desmontar"));
                    unmount_btn.set_valign(gtk4::Align::Center);
                    unmount_btn.add_css_class("flat");
                    part_row.add_suffix(&unmount_btn);
                } else if part.filesystem.is_some() {
                    let mount_btn = gtk4::Button::from_icon_name("media-mount-symbolic");
                    mount_btn.set_tooltip_text(Some("Montar"));
                    mount_btn.set_valign(gtk4::Align::Center);
                    mount_btn.add_css_class("flat");
                    part_row.add_suffix(&mount_btn);
                }

                part_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
                group.add(&part_row);
            }
        } else {
            let empty_row = ActionRow::builder()
                .title("Nenhuma particao encontrada")
                .subtitle("Clique para criar uma nova tabela de particoes")
                .activatable(true)
                .build();
            empty_row.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
            group.add(&empty_row);
        }

        group
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}

impl Default for OverviewPage {
    fn default() -> Self {
        Self::new()
    }
}
