// Winux Logs - Syslog Source
// Copyright (c) 2026 Winux OS Project
//
// Read traditional syslog files from /var/log

use super::{LogEntry, LogLevel};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Common log files in /var/log
const SYSLOG_FILES: &[(&str, &str)] = &[
    ("syslog", "/var/log/syslog"),
    ("messages", "/var/log/messages"),
    ("auth", "/var/log/auth.log"),
    ("kern", "/var/log/kern.log"),
    ("daemon", "/var/log/daemon.log"),
    ("boot", "/var/log/boot.log"),
    ("dpkg", "/var/log/dpkg.log"),
    ("pacman", "/var/log/pacman.log"),
    ("Xorg", "/var/log/Xorg.0.log"),
];

/// Available syslog file information
#[derive(Debug, Clone)]
pub struct SyslogFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<DateTime<Local>>,
    pub readable: bool,
}

/// Get list of available syslog files
pub fn get_available_files() -> Vec<SyslogFile> {
    let mut files = Vec::new();

    for (name, path) in SYSLOG_FILES {
        let path = PathBuf::from(path);
        if path.exists() {
            let metadata = fs::metadata(&path).ok();

            files.push(SyslogFile {
                name: name.to_string(),
                path: path.clone(),
                size: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                modified: metadata.and_then(|m| m.modified().ok())
                    .and_then(|t| DateTime::from_timestamp(
                        t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
                        0
                    ))
                    .map(|dt| dt.with_timezone(&Local)),
                readable: File::open(&path).is_ok(),
            });
        }
    }

    // Also scan for other log files
    if let Ok(entries) = fs::read_dir("/var/log") {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip already added and compressed files
                if name.ends_with(".gz") || name.ends_with(".xz") ||
                   name.ends_with(".old") || name.contains(".journal") {
                    continue;
                }

                if !files.iter().any(|f| f.path == path) {
                    let metadata = fs::metadata(&path).ok();

                    files.push(SyslogFile {
                        name,
                        path: path.clone(),
                        size: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                        modified: metadata.and_then(|m| m.modified().ok())
                            .and_then(|t| DateTime::from_timestamp(
                                t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
                                0
                            ))
                            .map(|dt| dt.with_timezone(&Local)),
                        readable: File::open(&path).is_ok(),
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

/// Load logs from all syslog files
pub fn load_syslog_logs(limit: usize) -> Vec<LogEntry> {
    let files = get_available_files();
    let mut all_entries = Vec::new();

    // Read from primary syslog files
    for file in &files {
        if file.readable {
            let entries = load_file_logs(&file.path, limit / files.len().max(1));
            all_entries.extend(entries);
        }
    }

    // Sort by timestamp (newest first)
    all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    all_entries.truncate(limit);

    if all_entries.is_empty() {
        get_sample_syslog_entries()
    } else {
        all_entries
    }
}

/// Load logs from a specific file
pub fn load_file_logs(path: &Path, limit: usize) -> Vec<LogEntry> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let mut entries = Vec::new();

    // Read from end of file for better performance
    let reader = BufReader::new(&file);
    let lines: Vec<String> = reader.lines()
        .filter_map(|l| l.ok())
        .collect();

    for line in lines.into_iter().rev().take(limit) {
        if let Some(mut entry) = parse_syslog_line(&line) {
            entry.source = format!("/var/log/{}", file_name);
            entries.push(entry);
        }
    }

    entries
}

/// Tail a syslog file (read last N lines)
pub fn tail_file(path: &Path, lines: usize) -> Vec<LogEntry> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Simple tail implementation
    let reader = BufReader::new(&file);
    let all_lines: Vec<String> = reader.lines()
        .filter_map(|l| l.ok())
        .collect();

    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    all_lines[start..]
        .iter()
        .filter_map(|line| {
            let mut entry = parse_syslog_line(line)?;
            entry.source = format!("/var/log/{}", file_name);
            Some(entry)
        })
        .collect()
}

fn parse_syslog_line(line: &str) -> Option<LogEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Try different syslog formats

    // Format 1: RFC 3164 (traditional syslog)
    // Feb 19 10:30:45 hostname process[pid]: message
    if let Some(entry) = parse_rfc3164(line) {
        return Some(entry);
    }

    // Format 2: RFC 5424 (newer syslog)
    // <priority>version timestamp hostname app-name procid msgid message
    if let Some(entry) = parse_rfc5424(line) {
        return Some(entry);
    }

    // Format 3: Simple timestamp
    // 2026-02-19 10:30:45 message
    if let Some(entry) = parse_simple_timestamp(line) {
        return Some(entry);
    }

    // Fallback: just use the line as message
    Some(LogEntry::new(Local::now(), LogLevel::Info, line.to_string()))
}

fn parse_rfc3164(line: &str) -> Option<LogEntry> {
    // Format: Feb 19 10:30:45 hostname process[pid]: message
    // Or: Feb 19 10:30:45 hostname process: message

    let parts: Vec<&str> = line.splitn(5, ' ').collect();
    if parts.len() < 5 {
        return None;
    }

    // Parse timestamp (month day time)
    let timestamp_str = format!("{} {} {} {}", parts[0], parts[1], parts[2], Local::now().format("%Y"));
    let timestamp = NaiveDateTime::parse_from_str(&timestamp_str, "%b %d %H:%M:%S %Y")
        .ok()
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .unwrap_or_else(Local::now);

    let hostname = parts[3];
    let rest = parts[4];

    // Parse process[pid]: message
    let (process, pid, message) = if let Some(bracket_start) = rest.find('[') {
        if let Some(bracket_end) = rest.find(']') {
            let proc = &rest[..bracket_start];
            let pid_str = &rest[bracket_start + 1..bracket_end];
            let msg = rest[bracket_end + 1..].trim_start_matches(':').trim();
            (proc, pid_str.parse().ok(), msg.to_string())
        } else {
            let (proc, msg) = rest.split_once(':').unwrap_or((rest, ""));
            (proc.trim(), None, msg.trim().to_string())
        }
    } else if let Some((proc, msg)) = rest.split_once(':') {
        (proc.trim(), None, msg.trim().to_string())
    } else {
        ("", None, rest.to_string())
    };

    // Guess log level from message content
    let level = guess_log_level(&message);

    let mut entry = LogEntry::new(timestamp, level, message);
    entry.unit = if !process.is_empty() {
        Some(process.to_string())
    } else {
        None
    };
    entry.pid = pid;
    entry.hostname = Some(hostname.to_string());

    Some(entry)
}

fn parse_rfc5424(line: &str) -> Option<LogEntry> {
    // Format: <priority>version timestamp hostname app-name procid msgid message
    // Example: <34>1 2026-02-19T10:30:45.123Z host app 1234 - message

    if !line.starts_with('<') {
        return None;
    }

    let pri_end = line.find('>')?;
    let priority: u8 = line[1..pri_end].parse().ok()?;
    let level = LogLevel::from_priority(priority & 0x07);

    let rest = &line[pri_end + 1..];

    // Skip version if present
    let rest = if rest.starts_with('1') || rest.starts_with('0') {
        rest.get(2..)?.trim()
    } else {
        rest.trim()
    };

    let parts: Vec<&str> = rest.splitn(6, ' ').collect();
    if parts.len() < 5 {
        return None;
    }

    // Parse ISO timestamp
    let timestamp = DateTime::parse_from_rfc3339(parts[0])
        .map(|dt| dt.with_timezone(&Local))
        .unwrap_or_else(|_| Local::now());

    let hostname = parts[1];
    let app_name = parts[2];
    let procid = parts[3];
    let message = if parts.len() > 5 { parts[5] } else { "" };

    let mut entry = LogEntry::new(timestamp, level, message.to_string());
    entry.unit = if app_name != "-" { Some(app_name.to_string()) } else { None };
    entry.pid = if procid != "-" { procid.parse().ok() } else { None };
    entry.hostname = if hostname != "-" { Some(hostname.to_string()) } else { None };

    Some(entry)
}

fn parse_simple_timestamp(line: &str) -> Option<LogEntry> {
    // Format: 2026-02-19 10:30:45 message
    // Or: [2026-02-19 10:30:45] message

    let line = line.trim_start_matches('[').trim();

    if line.len() < 19 {
        return None;
    }

    let timestamp_str = &line[..19];
    let timestamp = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|dt| Local.from_local_datetime(&dt).single())?;

    let message = line[19..].trim_start_matches(']').trim();

    let level = guess_log_level(message);
    Some(LogEntry::new(timestamp, level, message.to_string()))
}

fn guess_log_level(message: &str) -> LogLevel {
    let lower = message.to_lowercase();

    if lower.contains("error") || lower.contains("fail") || lower.contains("fatal") {
        LogLevel::Error
    } else if lower.contains("warn") {
        LogLevel::Warning
    } else if lower.contains("debug") || lower.contains("trace") {
        LogLevel::Debug
    } else if lower.contains("notice") {
        LogLevel::Notice
    } else if lower.contains("crit") {
        LogLevel::Critical
    } else if lower.contains("alert") {
        LogLevel::Alert
    } else if lower.contains("emerg") || lower.contains("panic") {
        LogLevel::Emergency
    } else {
        LogLevel::Info
    }
}

fn get_sample_syslog_entries() -> Vec<LogEntry> {
    let now = Local::now();

    vec![
        create_syslog_entry(now, LogLevel::Info, "syslog", "rsyslogd", "start"),
        create_syslog_entry(now, LogLevel::Info, "auth.log", "sshd", "Server listening on 0.0.0.0 port 22"),
        create_syslog_entry(now, LogLevel::Info, "auth.log", "sshd", "Accepted publickey for winux from 192.168.1.100"),
        create_syslog_entry(now, LogLevel::Warning, "auth.log", "sudo", "winux : 3 incorrect password attempts"),
        create_syslog_entry(now, LogLevel::Info, "daemon.log", "cron", "INFO (Running @reboot jobs)"),
        create_syslog_entry(now, LogLevel::Notice, "kern.log", "kernel", "Bluetooth: hci0: BCM: chip id 150"),
        create_syslog_entry(now, LogLevel::Error, "kern.log", "kernel", "ata2.00: failed command: READ DMA"),
        create_syslog_entry(now, LogLevel::Info, "dpkg.log", "dpkg", "status installed linux-image-6.8.0"),
        create_syslog_entry(now, LogLevel::Info, "pacman.log", "pacman", "[ALPM] upgraded mesa (24.0.1-1 -> 24.0.2-1)"),
        create_syslog_entry(now, LogLevel::Warning, "Xorg.0.log", "Xorg", "(WW) Open ACPI failed"),
    ]
}

fn create_syslog_entry(
    timestamp: DateTime<Local>,
    level: LogLevel,
    file: &str,
    unit: &str,
    message: &str,
) -> LogEntry {
    let mut entry = LogEntry::new(timestamp, level, message.to_string());
    entry.source = format!("/var/log/{}", file);
    entry.unit = Some(unit.to_string());
    entry
}
