// System Information - Gather system context for AI

use sysinfo::{System, Disks, Networks};
use std::env;

pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub total_memory_gb: f64,
    pub used_memory_gb: f64,
    pub available_memory_gb: f64,
    pub disk_info: Vec<DiskInfo>,
    pub uptime_hours: f64,
    pub username: String,
    pub home_dir: String,
    pub current_dir: String,
    pub shell: String,
}

pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_gb: f64,
    pub available_gb: f64,
    pub filesystem: String,
}

impl SystemInfo {
    pub fn collect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let disk_info: Vec<DiskInfo> = disks.iter().map(|disk| {
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_gb: disk.total_space() as f64 / 1_073_741_824.0,
                available_gb: disk.available_space() as f64 / 1_073_741_824.0,
                filesystem: disk.file_system().to_string_lossy().to_string(),
            }
        }).collect();

        let total_memory = sys.total_memory() as f64 / 1_073_741_824.0;
        let used_memory = sys.used_memory() as f64 / 1_073_741_824.0;

        Self {
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            cpu_brand: sys.cpus().first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            cpu_cores: sys.cpus().len(),
            total_memory_gb: total_memory,
            used_memory_gb: used_memory,
            available_memory_gb: total_memory - used_memory,
            disk_info,
            uptime_hours: System::uptime() as f64 / 3600.0,
            username: env::var("USER").or_else(|_| env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string()),
            home_dir: dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            current_dir: env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            shell: env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
        }
    }

    /// Generate a summary string for AI context
    pub fn summary(&self) -> String {
        let disk_summary: String = self.disk_info.iter()
            .map(|d| format!("  - {} ({}) at {}: {:.1}GB free of {:.1}GB",
                d.name, d.filesystem, d.mount_point, d.available_gb, d.total_gb))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"System: {} {}
Kernel: {}
Hostname: {}
User: {}

CPU: {} ({} cores)
Memory: {:.1}GB used / {:.1}GB total ({:.1}GB available)
Uptime: {:.1} hours

Disks:
{}

Shell: {}
Home: {}
Current Directory: {}"#,
            self.os_name,
            self.os_version,
            self.kernel_version,
            self.hostname,
            self.username,
            self.cpu_brand,
            self.cpu_cores,
            self.used_memory_gb,
            self.total_memory_gb,
            self.available_memory_gb,
            self.uptime_hours,
            disk_summary,
            self.shell,
            self.home_dir,
            self.current_dir
        )
    }

    /// Get a brief one-line summary
    pub fn brief(&self) -> String {
        format!(
            "{} {} | {} | {:.1}GB/{:.1}GB RAM",
            self.os_name,
            self.os_version,
            self.cpu_brand,
            self.used_memory_gb,
            self.total_memory_gb
        )
    }
}

/// Get running processes summary
pub fn get_running_processes(limit: usize) -> Vec<(String, f32, f64)> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes: Vec<_> = sys.processes()
        .iter()
        .map(|(_, process)| {
            (
                process.name().to_string_lossy().to_string(),
                process.cpu_usage(),
                process.memory() as f64 / 1_073_741_824.0,
            )
        })
        .collect();

    // Sort by CPU usage
    processes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(limit);
    processes
}

/// Get network interfaces summary
pub fn get_network_interfaces() -> Vec<(String, u64, u64)> {
    let networks = Networks::new_with_refreshed_list();

    networks.iter()
        .map(|(name, data)| {
            (
                name.clone(),
                data.received(),
                data.transmitted(),
            )
        })
        .collect()
}
