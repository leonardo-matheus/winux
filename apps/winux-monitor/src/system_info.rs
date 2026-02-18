//! System information view for Winux Monitor
//!
//! Displays detailed hardware and software information about the system.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use std::cell::RefCell;
use sysinfo::System;
use tracing::debug;

/// System information data
#[derive(Debug, Clone, Default)]
pub struct SystemData {
    // OS Info
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,

    // Hardware
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub total_memory: u64,
    pub total_swap: u64,

    // Graphics
    pub gpu_name: String,

    // Storage
    pub disk_info: Vec<DiskInfo>,

    // Network
    pub network_interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub available: u64,
    pub fs_type: String,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct SystemInfoView {
        pub system: RefCell<System>,
        pub data: RefCell<SystemData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemInfoView {
        const NAME: &'static str = "WinuxMonitorSystemInfoView";
        type Type = super::SystemInfoView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for SystemInfoView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for SystemInfoView {}
    impl BoxImpl for SystemInfoView {}
}

glib::wrapper! {
    pub struct SystemInfoView(ObjectSubclass<imp::SystemInfoView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl SystemInfoView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        self.set_orientation(gtk4::Orientation::Vertical);
        self.set_spacing(0);

        // Initialize system
        {
            let mut system = imp.system.borrow_mut();
            system.refresh_all();
        }

        // Gather system data
        let data = self.gather_system_info();
        imp.data.replace(data.clone());

        // Create scrolled content
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(24);

        // OS section
        let os_section = self.create_section("Operating System", "computer-symbolic", &[
            ("Name", &data.os_name),
            ("Version", &data.os_version),
            ("Kernel", &data.kernel_version),
            ("Hostname", &data.hostname),
        ]);
        content.append(&os_section);

        // CPU section
        let cpu_section = self.create_section("Processor", "processor-symbolic", &[
            ("Model", &data.cpu_name),
            ("Cores", &data.cpu_cores.to_string()),
            ("Threads", &data.cpu_threads.to_string()),
        ]);
        content.append(&cpu_section);

        // Memory section
        let mem_total = format_bytes(data.total_memory);
        let swap_total = format_bytes(data.total_swap);
        let memory_section = self.create_section("Memory", "drive-harddisk-symbolic", &[
            ("Total RAM", &mem_total),
            ("Swap", &swap_total),
        ]);
        content.append(&memory_section);

        // Graphics section
        let graphics_section = self.create_section("Graphics", "video-display-symbolic", &[
            ("GPU", &data.gpu_name),
        ]);
        content.append(&graphics_section);

        // Storage section
        let storage_section = self.create_storage_section(&data.disk_info);
        content.append(&storage_section);

        // Copy button at the bottom
        let copy_button = gtk4::Button::with_label("Copy System Info");
        copy_button.add_css_class("pill");
        copy_button.set_halign(gtk4::Align::Center);
        copy_button.set_margin_top(12);

        let data_clone = data.clone();
        copy_button.connect_clicked(move |button| {
            let info = format_system_info(&data_clone);
            if let Some(display) = button.display().as_ref() {
                let clipboard = display.clipboard();
                clipboard.set_text(&info);
            }
        });
        content.append(&copy_button);

        scrolled.set_child(Some(&content));
        self.append(&scrolled);
    }

    fn gather_system_info(&self) -> SystemData {
        let imp = self.imp();
        let system = imp.system.borrow();

        let mut data = SystemData::default();

        // OS Info
        data.os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        data.os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        data.kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        data.hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

        // CPU Info
        if let Some(cpu) = system.cpus().first() {
            data.cpu_name = cpu.brand().to_string();
        }
        data.cpu_cores = system.physical_core_count().unwrap_or(0) as u32;
        data.cpu_threads = system.cpus().len() as u32;

        // Memory
        data.total_memory = system.total_memory();
        data.total_swap = system.total_swap();

        // GPU - would need additional library in production
        data.gpu_name = detect_gpu();

        // Disk info
        let disks = sysinfo::Disks::new_with_refreshed_list();
        for disk in disks.iter() {
            data.disk_info.push(DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total: disk.total_space(),
                available: disk.available_space(),
                fs_type: disk.file_system().to_string_lossy().to_string(),
            });
        }

        // Network interfaces
        let networks = sysinfo::Networks::new_with_refreshed_list();
        for (name, _network) in networks.iter() {
            data.network_interfaces.push(NetworkInterface {
                name: name.clone(),
                mac_address: String::new(), // Would need additional library
                ip_addresses: Vec::new(),
            });
        }

        data
    }

    fn create_section(&self, title: &str, icon: &str, rows: &[(&str, &str)]) -> gtk4::Box {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 8);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let icon_widget = gtk4::Image::from_icon_name(icon);
        header.append(&icon_widget);

        let title_label = gtk4::Label::new(Some(title));
        title_label.add_css_class("title-3");
        title_label.set_halign(gtk4::Align::Start);
        header.append(&title_label);

        section.append(&header);

        // Content
        let list = gtk4::ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk4::SelectionMode::None);

        for (label, value) in rows {
            let row = adw::ActionRow::builder()
                .title(*label)
                .build();

            let value_label = gtk4::Label::new(Some(value));
            value_label.add_css_class("dim-label");
            value_label.set_selectable(true);
            row.add_suffix(&value_label);

            list.append(&row);
        }

        section.append(&list);
        section
    }

    fn create_storage_section(&self, disks: &[DiskInfo]) -> gtk4::Box {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 8);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let icon = gtk4::Image::from_icon_name("drive-harddisk-symbolic");
        header.append(&icon);

        let title = gtk4::Label::new(Some("Storage"));
        title.add_css_class("title-3");
        title.set_halign(gtk4::Align::Start);
        header.append(&title);

        section.append(&header);

        // Disk list
        let list = gtk4::ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk4::SelectionMode::None);

        for disk in disks {
            let used = disk.total - disk.available;
            let percent = if disk.total > 0 {
                (used as f64 / disk.total as f64) * 100.0
            } else {
                0.0
            };

            let row = adw::ActionRow::builder()
                .title(&disk.mount_point)
                .subtitle(&format!(
                    "{} - {} of {} ({:.1}%)",
                    disk.fs_type,
                    format_bytes(used),
                    format_bytes(disk.total),
                    percent
                ))
                .build();

            // Progress bar
            let progress = gtk4::ProgressBar::new();
            progress.set_fraction(percent / 100.0);
            progress.set_width_request(100);
            progress.set_valign(gtk4::Align::Center);
            row.add_suffix(&progress);

            list.append(&row);
        }

        if disks.is_empty() {
            let row = adw::ActionRow::builder()
                .title("No disks detected")
                .build();
            list.append(&row);
        }

        section.append(&list);
        section
    }
}

impl Default for SystemInfoView {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Detect GPU (simplified - would need proper detection in production)
fn detect_gpu() -> String {
    // Try to read from /proc/driver/nvidia/gpus or lspci
    if let Ok(output) = std::process::Command::new("lspci")
        .args(["-v", "-s"])
        .arg("$(lspci | grep -i vga | cut -d' ' -f1)")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("VGA") || line.contains("3D") {
                return line.trim().to_string();
            }
        }
    }

    // Fallback to generic detection
    if std::path::Path::new("/proc/driver/nvidia").exists() {
        return "NVIDIA Graphics Card".to_string();
    }

    if let Ok(contents) = std::fs::read_to_string("/sys/class/drm/card0/device/vendor") {
        let vendor = contents.trim();
        return match vendor {
            "0x1002" => "AMD Graphics Card".to_string(),
            "0x10de" => "NVIDIA Graphics Card".to_string(),
            "0x8086" => "Intel Integrated Graphics".to_string(),
            _ => "Unknown GPU".to_string(),
        };
    }

    "Unknown".to_string()
}

/// Format all system info as text for copying
fn format_system_info(data: &SystemData) -> String {
    let mut info = String::new();

    info.push_str("=== System Information ===\n\n");

    info.push_str("Operating System:\n");
    info.push_str(&format!("  Name: {}\n", data.os_name));
    info.push_str(&format!("  Version: {}\n", data.os_version));
    info.push_str(&format!("  Kernel: {}\n", data.kernel_version));
    info.push_str(&format!("  Hostname: {}\n", data.hostname));
    info.push('\n');

    info.push_str("Processor:\n");
    info.push_str(&format!("  Model: {}\n", data.cpu_name));
    info.push_str(&format!("  Cores: {}\n", data.cpu_cores));
    info.push_str(&format!("  Threads: {}\n", data.cpu_threads));
    info.push('\n');

    info.push_str("Memory:\n");
    info.push_str(&format!("  Total RAM: {}\n", format_bytes(data.total_memory)));
    info.push_str(&format!("  Swap: {}\n", format_bytes(data.total_swap)));
    info.push('\n');

    info.push_str("Graphics:\n");
    info.push_str(&format!("  GPU: {}\n", data.gpu_name));
    info.push('\n');

    info.push_str("Storage:\n");
    for disk in &data.disk_info {
        info.push_str(&format!(
            "  {} ({}): {} / {}\n",
            disk.mount_point,
            disk.fs_type,
            format_bytes(disk.total - disk.available),
            format_bytes(disk.total)
        ));
    }

    info
}
