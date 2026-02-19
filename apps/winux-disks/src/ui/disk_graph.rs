//! Disk partition visualization graph
//!
//! Displays partitions as a horizontal bar graph with colors for different filesystems.

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label, DrawingArea, Frame, Grid};
use libadwaita as adw;
use adw::prelude::*;
use adw::ActionRow;

use crate::backend::Partition;

/// Visual representation of disk partitions
pub struct DiskGraph {
    widget: Box,
}

impl DiskGraph {
    /// Create a new disk graph visualization
    pub fn new(partitions: &[Partition], total_size: u64) -> Self {
        let main_box = Box::new(Orientation::Vertical, 8);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);

        // Create the partition bar
        let bar_box = Box::new(Orientation::Horizontal, 0);
        bar_box.set_hexpand(true);
        bar_box.add_css_class("card");

        // Calculate used space
        let used_space: u64 = partitions.iter().map(|p| p.size).sum();
        let free_space = if total_size > used_space { total_size - used_space } else { 0 };

        // Add partition segments
        for partition in partitions {
            let width_fraction = partition.size as f64 / total_size as f64;
            let segment = Self::create_segment(partition, width_fraction);
            bar_box.append(&segment);
        }

        // Add free space segment if any
        if free_space > 0 {
            let free_fraction = free_space as f64 / total_size as f64;
            let free_segment = Self::create_free_segment(free_fraction);
            bar_box.append(&free_segment);
        }

        main_box.append(&bar_box);

        // Create legend
        let legend_box = Box::new(Orientation::Horizontal, 16);
        legend_box.set_halign(gtk4::Align::Center);
        legend_box.set_margin_top(8);

        // Collect unique filesystems
        let mut fs_types: Vec<&str> = partitions
            .iter()
            .filter_map(|p| p.filesystem.as_deref())
            .collect();
        fs_types.sort();
        fs_types.dedup();

        for fs in fs_types {
            let legend_item = Self::create_legend_item(fs);
            legend_box.append(&legend_item);
        }

        // Add free space to legend if present
        if free_space > 0 {
            let free_item = Self::create_legend_item("Livre");
            legend_box.append(&free_item);
        }

        main_box.append(&legend_box);

        Self { widget: main_box }
    }

    /// Create a segment for a partition
    fn create_segment(partition: &Partition, width_fraction: f64) -> Box {
        let segment = Box::new(Orientation::Vertical, 0);
        segment.set_hexpand(true);
        segment.set_size_request((width_fraction * 400.0) as i32, 40);

        // Color based on filesystem
        let color_class = Self::get_fs_color_class(partition.filesystem.as_deref());
        segment.add_css_class(color_class);
        segment.add_css_class("partition-segment");

        // Add tooltip with partition info
        let tooltip = format!(
            "{}\n{}\n{}",
            partition.name,
            partition.filesystem.as_deref().unwrap_or("Desconhecido"),
            bytesize::ByteSize::b(partition.size).to_string_as(true)
        );
        segment.set_tooltip_text(Some(&tooltip));

        // Label inside segment (if large enough)
        if width_fraction > 0.1 {
            let label = Label::new(Some(&partition.name));
            label.add_css_class("caption");
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            segment.set_valign(gtk4::Align::Center);
            segment.append(&label);
        }

        segment
    }

    /// Create a segment for free space
    fn create_free_segment(width_fraction: f64) -> Box {
        let segment = Box::new(Orientation::Vertical, 0);
        segment.set_hexpand(true);
        segment.set_size_request((width_fraction * 400.0) as i32, 40);
        segment.add_css_class("partition-free");
        segment.add_css_class("partition-segment");
        segment.set_tooltip_text(Some("Espaco livre"));

        if width_fraction > 0.1 {
            let label = Label::new(Some("Livre"));
            label.add_css_class("caption");
            label.add_css_class("dim-label");
            segment.set_valign(gtk4::Align::Center);
            segment.append(&label);
        }

        segment
    }

    /// Create a legend item
    fn create_legend_item(fs_type: &str) -> Box {
        let item = Box::new(Orientation::Horizontal, 4);

        let color_box = Box::new(Orientation::Horizontal, 0);
        color_box.set_size_request(16, 16);
        let color_class = if fs_type == "Livre" {
            "partition-free"
        } else {
            Self::get_fs_color_class(Some(fs_type))
        };
        color_box.add_css_class(color_class);
        color_box.add_css_class("legend-color");

        let label = Label::new(Some(fs_type));
        label.add_css_class("caption");

        item.append(&color_box);
        item.append(&label);

        item
    }

    /// Get CSS class for filesystem color
    fn get_fs_color_class(fs_type: Option<&str>) -> &'static str {
        match fs_type {
            Some("ext4") | Some("ext3") | Some("ext2") => "fs-ext4",
            Some("btrfs") => "fs-btrfs",
            Some("xfs") => "fs-xfs",
            Some("ntfs") | Some("ntfs3") => "fs-ntfs",
            Some("vfat") | Some("fat32") | Some("fat16") | Some("fat") => "fs-fat",
            Some("exfat") => "fs-exfat",
            Some("swap") => "fs-swap",
            Some("f2fs") => "fs-f2fs",
            Some("zfs") => "fs-zfs",
            Some("crypto_LUKS") | Some("luks") => "fs-luks",
            Some("lvm2") | Some("LVM2_member") => "fs-lvm",
            _ => "fs-unknown",
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.widget
    }

    /// Create a detailed partition list view
    pub fn create_detailed_list(partitions: &[Partition], total_size: u64) -> Grid {
        let grid = Grid::new();
        grid.set_column_spacing(12);
        grid.set_row_spacing(8);
        grid.set_margin_start(12);
        grid.set_margin_end(12);
        grid.set_margin_top(12);
        grid.set_margin_bottom(12);

        // Header
        let headers = ["Particao", "Sistema", "Tamanho", "Usado", "Montagem"];
        for (col, header) in headers.iter().enumerate() {
            let label = Label::new(Some(header));
            label.add_css_class("heading");
            label.set_xalign(0.0);
            grid.attach(&label, col as i32, 0, 1, 1);
        }

        // Partition rows
        for (row, partition) in partitions.iter().enumerate() {
            let row = row as i32 + 1;

            // Name
            let name_label = Label::new(Some(&format!("/dev/{}", partition.name)));
            name_label.set_xalign(0.0);
            grid.attach(&name_label, 0, row, 1, 1);

            // Filesystem
            let fs_label = Label::new(Some(partition.filesystem.as_deref().unwrap_or("-")));
            fs_label.set_xalign(0.0);
            grid.attach(&fs_label, 1, row, 1, 1);

            // Size
            let size_label = Label::new(Some(&bytesize::ByteSize::b(partition.size).to_string_as(true)));
            size_label.set_xalign(0.0);
            grid.attach(&size_label, 2, row, 1, 1);

            // Usage percentage
            let percent = (partition.size as f64 / total_size as f64) * 100.0;
            let percent_label = Label::new(Some(&format!("{:.1}%", percent)));
            percent_label.set_xalign(0.0);
            grid.attach(&percent_label, 3, row, 1, 1);

            // Mount point
            let mount_label = Label::new(Some(partition.mount_point.as_deref().unwrap_or("-")));
            mount_label.set_xalign(0.0);
            grid.attach(&mount_label, 4, row, 1, 1);
        }

        grid
    }

    /// Get CSS for partition colors (to be added to app CSS)
    pub fn get_css() -> &'static str {
        r#"
.partition-segment {
    min-height: 40px;
    border-radius: 0;
}

.partition-segment:first-child {
    border-radius: 8px 0 0 8px;
}

.partition-segment:last-child {
    border-radius: 0 8px 8px 0;
}

.partition-segment:only-child {
    border-radius: 8px;
}

.partition-free {
    background-color: alpha(@window_bg_color, 0.3);
    border: 1px dashed @borders;
}

.fs-ext4 {
    background-color: #3584e4;
}

.fs-btrfs {
    background-color: #ff7800;
}

.fs-xfs {
    background-color: #e01b24;
}

.fs-ntfs {
    background-color: #0078d4;
}

.fs-fat {
    background-color: #33d17a;
}

.fs-exfat {
    background-color: #57e389;
}

.fs-swap {
    background-color: #9141ac;
}

.fs-f2fs {
    background-color: #f66151;
}

.fs-zfs {
    background-color: #1c71d8;
}

.fs-luks {
    background-color: #613583;
}

.fs-lvm {
    background-color: #865e3c;
}

.fs-unknown {
    background-color: @card_bg_color;
    border: 1px solid @borders;
}

.legend-color {
    border-radius: 4px;
    border: 1px solid rgba(0, 0, 0, 0.1);
}
"#
    }
}
