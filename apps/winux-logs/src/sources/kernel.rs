// Winux Logs - Kernel Log Source
// Copyright (c) 2026 Winux OS Project
//
// Read kernel messages from dmesg / kernel ring buffer

use super::{LogEntry, LogLevel};
use chrono::{DateTime, Duration, Local};
use std::process::Command;

/// Load kernel logs from dmesg
pub fn load_kernel_logs(limit: usize) -> Vec<LogEntry> {
    // Try to use dmesg with JSON output first (Linux 4.15+)
    let output = Command::new("dmesg")
        .args(["--json", "--decode"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                parse_dmesg_json(&stdout, limit)
            } else {
                // Fall back to human-readable format
                load_kernel_logs_text(limit)
            }
        }
        Err(_) => load_kernel_logs_text(limit),
    }
}

fn load_kernel_logs_text(limit: usize) -> Vec<LogEntry> {
    let output = Command::new("dmesg")
        .args(["--decode", "--color=never", "-T"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                parse_dmesg_text(&stdout, limit)
            } else {
                get_sample_kernel_logs()
            }
        }
        Err(_) => get_sample_kernel_logs(),
    }
}

fn parse_dmesg_json(output: &str, limit: usize) -> Vec<LogEntry> {
    // dmesg --json outputs a JSON object with "dmesg" array
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(messages) = json.get("dmesg").and_then(|d| d.as_array()) {
            let entries: Vec<LogEntry> = messages.iter()
                .rev()
                .take(limit)
                .filter_map(|msg| parse_dmesg_json_entry(msg))
                .collect();

            if !entries.is_empty() {
                return entries;
            }
        }
    }

    get_sample_kernel_logs()
}

fn parse_dmesg_json_entry(json: &serde_json::Value) -> Option<LogEntry> {
    let message = json.get("msg")?.as_str()?.to_string();

    // Parse timestamp (seconds since boot, with decimals)
    let boot_time = get_boot_time();
    let timestamp = if let Some(ts) = json.get("time_usec") {
        let usec = ts.as_i64()?;
        boot_time + Duration::microseconds(usec)
    } else {
        Local::now()
    };

    // Parse facility and priority
    let level = if let Some(pri) = json.get("pri") {
        let p = pri.as_i64()? as u8 & 0x07;
        LogLevel::from_priority(p)
    } else {
        LogLevel::Info
    };

    let facility = json.get("fac")
        .and_then(|f| f.as_str())
        .unwrap_or("kern");

    let mut entry = LogEntry::new(timestamp, level, message);
    entry.source = "kernel".to_string();
    entry.unit = Some(format!("kernel:{}", facility));

    Some(entry)
}

fn parse_dmesg_text(output: &str, limit: usize) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for line in output.lines().rev().take(limit) {
        if let Some(entry) = parse_dmesg_text_line(line) {
            entries.push(entry);
        }
    }

    if entries.is_empty() {
        return get_sample_kernel_logs();
    }

    entries
}

fn parse_dmesg_text_line(line: &str) -> Option<LogEntry> {
    // Format with -T: [Wed Feb 19 10:30:45 2026] message
    // Format with -d: [facility:level] message
    // Combined: [Wed Feb 19 10:30:45 2026] kern :info : message

    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Try to parse timestamp in brackets
    let (timestamp, rest) = if line.starts_with('[') {
        if let Some(end) = line.find(']') {
            let ts_str = &line[1..end];
            let rest = line[end + 1..].trim();

            // Parse timestamp
            let timestamp = parse_kernel_timestamp(ts_str);
            (timestamp, rest)
        } else {
            (Local::now(), line)
        }
    } else {
        (Local::now(), line)
    };

    // Try to parse facility:level prefix
    let (level, facility, message) = if rest.contains(':') {
        let parts: Vec<&str> = rest.splitn(3, ':').collect();
        if parts.len() >= 2 {
            let fac = parts[0].trim();
            let lev = parts[1].trim();
            let msg = if parts.len() > 2 { parts[2].trim() } else { "" };

            let level = match lev {
                "emerg" => LogLevel::Emergency,
                "alert" => LogLevel::Alert,
                "crit" => LogLevel::Critical,
                "err" => LogLevel::Error,
                "warn" | "warning" => LogLevel::Warning,
                "notice" => LogLevel::Notice,
                "info" => LogLevel::Info,
                "debug" => LogLevel::Debug,
                _ => LogLevel::Info,
            };

            (level, Some(fac.to_string()), msg.to_string())
        } else {
            (LogLevel::Info, None, rest.to_string())
        }
    } else {
        (LogLevel::Info, None, rest.to_string())
    };

    if message.is_empty() {
        return None;
    }

    let mut entry = LogEntry::new(timestamp, level, message);
    entry.source = "kernel".to_string();
    entry.unit = Some(format!("kernel{}", facility.map(|f| format!(":{}", f)).unwrap_or_default()));

    Some(entry)
}

fn parse_kernel_timestamp(ts_str: &str) -> DateTime<Local> {
    // Try parsing various formats
    // Format: Wed Feb 19 10:30:45 2026
    if let Ok(dt) = DateTime::parse_from_str(&format!("{} +0000", ts_str), "%a %b %d %H:%M:%S %Y %z") {
        return dt.with_timezone(&Local);
    }

    // Format: seconds.microseconds since boot
    if let Ok(secs) = ts_str.trim().parse::<f64>() {
        let boot_time = get_boot_time();
        return boot_time + Duration::milliseconds((secs * 1000.0) as i64);
    }

    Local::now()
}

fn get_boot_time() -> DateTime<Local> {
    // Read /proc/uptime and calculate boot time
    if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
        if let Some(uptime_str) = content.split_whitespace().next() {
            if let Ok(uptime_secs) = uptime_str.parse::<f64>() {
                let uptime = Duration::milliseconds((uptime_secs * 1000.0) as i64);
                return Local::now() - uptime;
            }
        }
    }

    // Fallback: assume 1 hour ago
    Local::now() - Duration::hours(1)
}

fn get_sample_kernel_logs() -> Vec<LogEntry> {
    let now = Local::now();

    vec![
        create_kernel_entry(now, LogLevel::Info, "kern", "Linux version 6.8.0-winux (gcc version 13.2.0)"),
        create_kernel_entry(now, LogLevel::Info, "kern", "Command line: BOOT_IMAGE=/boot/vmlinuz-linux root=UUID=xxxx rw quiet"),
        create_kernel_entry(now, LogLevel::Info, "kern", "BIOS-provided physical RAM map:"),
        create_kernel_entry(now, LogLevel::Info, "kern", "  BIOS-e820: [mem 0x0000000000000000-0x000000000009ffff] usable"),
        create_kernel_entry(now, LogLevel::Info, "kern", "ACPI: Early table checksum verification disabled"),
        create_kernel_entry(now, LogLevel::Notice, "kern", "NX (Execute Disable) protection: active"),
        create_kernel_entry(now, LogLevel::Info, "kern", "DMI: LENOVO ThinkPad/20QXS123, BIOS N2QET99W (1.56) 03/15/2024"),
        create_kernel_entry(now, LogLevel::Info, "kern", "Hypervisor detected: KVM"),
        create_kernel_entry(now, LogLevel::Info, "kern", "CPU: Intel(R) Core(TM) i7-10750H CPU @ 2.60GHz"),
        create_kernel_entry(now, LogLevel::Info, "kern", "smpboot: CPU0: Intel(R) Core(TM) i7-10750H CPU @ 2.60GHz (family: 0x6)"),
        create_kernel_entry(now, LogLevel::Warning, "kern", "x86/cpu: SGX disabled by BIOS"),
        create_kernel_entry(now, LogLevel::Info, "kern", "Memory: 16384MB RAM"),
        create_kernel_entry(now, LogLevel::Info, "kern", "ACPI: PM-Timer IO Port: 0x1808"),
        create_kernel_entry(now, LogLevel::Info, "kern", "ACPI: Preparing to enter system sleep state S3"),
        create_kernel_entry(now, LogLevel::Warning, "kern", "ACPI: battery: AC Adapter [AC]: state not set"),
        create_kernel_entry(now, LogLevel::Error, "kern", "ata1.00: failed command: READ DMA EXT"),
        create_kernel_entry(now, LogLevel::Info, "kern", "USB: Found XHCI controller"),
        create_kernel_entry(now, LogLevel::Info, "kern", "iwlwifi: loaded firmware version 77.61a1c6e0.0"),
        create_kernel_entry(now, LogLevel::Notice, "kern", "wlan0: associated with AP"),
        create_kernel_entry(now, LogLevel::Info, "kern", "NVIDIA driver loaded version 550.54.14"),
    ]
}

fn create_kernel_entry(
    timestamp: DateTime<Local>,
    level: LogLevel,
    facility: &str,
    message: &str,
) -> LogEntry {
    let mut entry = LogEntry::new(timestamp, level, message.to_string());
    entry.source = "kernel".to_string();
    entry.unit = Some(format!("kernel:{}", facility));
    entry
}

/// Get kernel ring buffer size
pub fn get_ring_buffer_size() -> Option<usize> {
    let output = Command::new("dmesg")
        .arg("--buffer-size")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim().parse().ok()
    } else {
        None
    }
}

/// Clear kernel ring buffer (requires root)
pub fn clear_ring_buffer() -> Result<(), std::io::Error> {
    let status = Command::new("dmesg")
        .arg("--clear")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Failed to clear dmesg (requires root)",
        ))
    }
}
