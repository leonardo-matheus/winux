// Winux Logs - systemd Journal Source
// Copyright (c) 2026 Winux OS Project
//
// Read logs from systemd journal using journalctl

use super::{LogEntry, LogLevel};
use chrono::{DateTime, Local, TimeZone};
use std::collections::HashMap;
use std::process::Command;

/// Load journal logs with optional filtering
pub fn load_journal_logs(boot: Option<i32>, limit: usize) -> Vec<LogEntry> {
    let mut args = vec![
        "--no-pager".to_string(),
        "-o".to_string(),
        "json".to_string(),
        "-n".to_string(),
        limit.to_string(),
        "--reverse".to_string(),
    ];

    if let Some(boot_offset) = boot {
        args.push("-b".to_string());
        args.push(boot_offset.to_string());
    }

    let output = Command::new("journalctl")
        .args(&args)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                parse_journal_output(&String::from_utf8_lossy(&output.stdout))
            } else {
                // Return sample data for demo
                get_sample_journal_entries()
            }
        }
        Err(_) => get_sample_journal_entries(),
    }
}

/// Load journal logs for a specific unit
pub fn load_unit_logs(unit: &str, limit: usize) -> Vec<LogEntry> {
    let output = Command::new("journalctl")
        .args([
            "--no-pager",
            "-o", "json",
            "-u", unit,
            "-n", &limit.to_string(),
            "--reverse",
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                parse_journal_output(&String::from_utf8_lossy(&output.stdout))
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

/// Load journal logs for a specific time range
pub fn load_logs_in_range(since: &str, until: &str, limit: usize) -> Vec<LogEntry> {
    let output = Command::new("journalctl")
        .args([
            "--no-pager",
            "-o", "json",
            "--since", since,
            "--until", until,
            "-n", &limit.to_string(),
            "--reverse",
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                parse_journal_output(&String::from_utf8_lossy(&output.stdout))
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

/// Load logs with a specific priority level or higher
pub fn load_logs_by_priority(priority: LogLevel, limit: usize) -> Vec<LogEntry> {
    let output = Command::new("journalctl")
        .args([
            "--no-pager",
            "-o", "json",
            "-p", priority.as_str(),
            "-n", &limit.to_string(),
            "--reverse",
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                parse_journal_output(&String::from_utf8_lossy(&output.stdout))
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

/// Get list of systemd units that have logs
pub fn get_units_with_logs() -> Vec<String> {
    let output = Command::new("journalctl")
        .args(["--field", "_SYSTEMD_UNIT"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                get_sample_units()
            }
        }
        Err(_) => get_sample_units(),
    }
}

fn parse_journal_output(output: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(entry) = parse_journal_entry(&json) {
                entries.push(entry);
            }
        }
    }

    entries
}

fn parse_journal_entry(json: &serde_json::Value) -> Option<LogEntry> {
    let message = json.get("MESSAGE")?.as_str()?.to_string();

    // Parse timestamp (microseconds since epoch)
    let timestamp = if let Some(ts) = json.get("__REALTIME_TIMESTAMP") {
        let micros = ts.as_str()?.parse::<i64>().ok()?;
        let secs = micros / 1_000_000;
        let nsecs = ((micros % 1_000_000) * 1000) as u32;
        Local.timestamp_opt(secs, nsecs).single()?
    } else {
        Local::now()
    };

    // Parse priority
    let level = if let Some(priority) = json.get("PRIORITY") {
        let p = priority.as_str()?.parse::<u8>().unwrap_or(6);
        LogLevel::from_priority(p)
    } else {
        LogLevel::Info
    };

    let mut entry = LogEntry::new(timestamp, level, message);

    // Parse optional fields
    entry.source = "journal".to_string();

    if let Some(unit) = json.get("_SYSTEMD_UNIT") {
        entry.unit = Some(unit.as_str()?.to_string());
    } else if let Some(unit) = json.get("UNIT") {
        entry.unit = Some(unit.as_str()?.to_string());
    } else if let Some(syslog_id) = json.get("SYSLOG_IDENTIFIER") {
        entry.unit = Some(syslog_id.as_str()?.to_string());
    }

    if let Some(pid) = json.get("_PID") {
        entry.pid = pid.as_str()?.parse().ok();
    }

    if let Some(uid) = json.get("_UID") {
        entry.uid = uid.as_str()?.parse().ok();
    }

    if let Some(boot_id) = json.get("_BOOT_ID") {
        entry.boot_id = Some(boot_id.as_str()?.to_string());
    }

    if let Some(machine_id) = json.get("_MACHINE_ID") {
        entry.machine_id = Some(machine_id.as_str()?.to_string());
    }

    if let Some(hostname) = json.get("_HOSTNAME") {
        entry.hostname = Some(hostname.as_str()?.to_string());
    }

    // Collect extra fields
    let skip_fields = [
        "MESSAGE", "__REALTIME_TIMESTAMP", "__MONOTONIC_TIMESTAMP",
        "PRIORITY", "_PID", "_UID", "_GID", "_BOOT_ID", "_MACHINE_ID",
        "_HOSTNAME", "_SYSTEMD_UNIT", "UNIT", "SYSLOG_IDENTIFIER",
        "__CURSOR", "_TRANSPORT",
    ];

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            if !skip_fields.contains(&key.as_str()) {
                if let Some(v) = value.as_str() {
                    entry.extra_fields.insert(key.clone(), v.to_string());
                }
            }
        }
    }

    // Store raw JSON
    entry.raw_message = Some(json.to_string());

    Some(entry)
}

fn get_sample_journal_entries() -> Vec<LogEntry> {
    let now = Local::now();

    vec![
        create_sample_entry(now, LogLevel::Info, "systemd", "systemd[1]", "Started Session 1 of user winux"),
        create_sample_entry(now, LogLevel::Notice, "NetworkManager", "NetworkManager[892]", "device (wlan0): state change: prepare -> config"),
        create_sample_entry(now, LogLevel::Warning, "kernel", "kernel", "ACPI: AC Adapter [AC]: state not set"),
        create_sample_entry(now, LogLevel::Error, "gdm-password", "gdm-password]", "gkr-pam: unable to locate daemon control file"),
        create_sample_entry(now, LogLevel::Info, "pulseaudio", "pulseaudio[1234]", "Sink alsa_output.pci-0000:00:1f.3.analog-stereo successfully suspended"),
        create_sample_entry(now, LogLevel::Debug, "dbus-daemon", "dbus-daemon[567]", "Activating service name='org.freedesktop.secrets'"),
        create_sample_entry(now, LogLevel::Critical, "systemd-coredump", "systemd-coredump[2345]", "Process 1234 (example-app) dumped core"),
        create_sample_entry(now, LogLevel::Info, "sshd", "sshd[3456]", "Server listening on 0.0.0.0 port 22"),
        create_sample_entry(now, LogLevel::Warning, "sudo", "sudo[4567]", "winux : TTY=pts/0 ; PWD=/home/winux ; USER=root ; COMMAND=/usr/bin/pacman -Syu"),
        create_sample_entry(now, LogLevel::Info, "systemd-logind", "systemd-logind[891]", "New session 2 of user winux"),
    ]
}

fn create_sample_entry(
    timestamp: DateTime<Local>,
    level: LogLevel,
    unit: &str,
    source: &str,
    message: &str,
) -> LogEntry {
    let mut entry = LogEntry::new(timestamp, level, message.to_string());
    entry.unit = Some(unit.to_string());
    entry.source = source.to_string();
    entry.pid = Some(rand_pid());
    entry
}

fn rand_pid() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 10000) + 100
}

fn get_sample_units() -> Vec<String> {
    vec![
        "NetworkManager.service".to_string(),
        "bluetooth.service".to_string(),
        "cups.service".to_string(),
        "dbus.service".to_string(),
        "gdm.service".to_string(),
        "sshd.service".to_string(),
        "systemd-journald.service".to_string(),
        "systemd-logind.service".to_string(),
        "systemd-networkd.service".to_string(),
        "systemd-resolved.service".to_string(),
    ]
}

/// Start following journal in real-time (returns a channel receiver)
pub fn start_journal_follow() -> async_channel::Receiver<LogEntry> {
    let (sender, receiver) = async_channel::bounded(100);

    std::thread::spawn(move || {
        let mut child = match Command::new("journalctl")
            .args(["-f", "-o", "json", "--no-pager"])
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return,
        };

        if let Some(stdout) = child.stdout.take() {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(entry) = parse_journal_entry(&json) {
                            if sender.send_blocking(entry).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }

        let _ = child.kill();
    });

    receiver
}
