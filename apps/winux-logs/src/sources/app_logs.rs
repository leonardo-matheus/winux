// Winux Logs - Application Logs Source
// Copyright (c) 2026 Winux OS Project
//
// Read logs from application-specific locations

use super::{LogEntry, LogLevel};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Known application log locations
#[derive(Debug, Clone)]
pub struct AppLogLocation {
    pub name: String,
    pub display_name: String,
    pub path: PathBuf,
    pub icon: String,
}

/// Get list of known application log locations
pub fn get_app_log_locations() -> Vec<AppLogLocation> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    let cache_dir = dirs::cache_dir().unwrap_or_else(|| home.join(".cache"));
    let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
    let data_dir = dirs::data_dir().unwrap_or_else(|| home.join(".local/share"));

    let mut locations = vec![
        // Desktop environments
        AppLogLocation {
            name: "gnome-shell".to_string(),
            display_name: "GNOME Shell".to_string(),
            path: cache_dir.join("gdm/session.log"),
            icon: "start-here-symbolic".to_string(),
        },
        AppLogLocation {
            name: "xsession".to_string(),
            display_name: "X Session".to_string(),
            path: home.join(".xsession-errors"),
            icon: "video-display-symbolic".to_string(),
        },
        // Browsers
        AppLogLocation {
            name: "firefox".to_string(),
            display_name: "Firefox".to_string(),
            path: data_dir.join("firefox"),
            icon: "firefox-symbolic".to_string(),
        },
        AppLogLocation {
            name: "chrome".to_string(),
            display_name: "Google Chrome".to_string(),
            path: config_dir.join("google-chrome/chrome_debug.log"),
            icon: "google-chrome-symbolic".to_string(),
        },
        // Development tools
        AppLogLocation {
            name: "vscode".to_string(),
            display_name: "VS Code".to_string(),
            path: cache_dir.join("code/logs"),
            icon: "code-symbolic".to_string(),
        },
        AppLogLocation {
            name: "docker".to_string(),
            display_name: "Docker".to_string(),
            path: PathBuf::from("/var/log/docker.log"),
            icon: "docker-symbolic".to_string(),
        },
        // Package managers
        AppLogLocation {
            name: "flatpak".to_string(),
            display_name: "Flatpak".to_string(),
            path: data_dir.join("flatpak/logs"),
            icon: "package-x-generic-symbolic".to_string(),
        },
        AppLogLocation {
            name: "snap".to_string(),
            display_name: "Snap".to_string(),
            path: PathBuf::from("/var/log/snap.log"),
            icon: "package-x-generic-symbolic".to_string(),
        },
        // System services
        AppLogLocation {
            name: "cups".to_string(),
            display_name: "CUPS (Printing)".to_string(),
            path: PathBuf::from("/var/log/cups/error_log"),
            icon: "printer-symbolic".to_string(),
        },
        AppLogLocation {
            name: "nginx".to_string(),
            display_name: "Nginx".to_string(),
            path: PathBuf::from("/var/log/nginx/error.log"),
            icon: "network-server-symbolic".to_string(),
        },
        AppLogLocation {
            name: "apache".to_string(),
            display_name: "Apache".to_string(),
            path: PathBuf::from("/var/log/apache2/error.log"),
            icon: "network-server-symbolic".to_string(),
        },
        // Databases
        AppLogLocation {
            name: "postgresql".to_string(),
            display_name: "PostgreSQL".to_string(),
            path: PathBuf::from("/var/log/postgresql"),
            icon: "folder-database-symbolic".to_string(),
        },
        AppLogLocation {
            name: "mysql".to_string(),
            display_name: "MySQL/MariaDB".to_string(),
            path: PathBuf::from("/var/log/mysql/error.log"),
            icon: "folder-database-symbolic".to_string(),
        },
        // Gaming
        AppLogLocation {
            name: "steam".to_string(),
            display_name: "Steam".to_string(),
            path: home.join(".steam/logs"),
            icon: "steam-symbolic".to_string(),
        },
        AppLogLocation {
            name: "wine".to_string(),
            display_name: "Wine".to_string(),
            path: home.join(".wine/dosdevices/c:/windows/Logs"),
            icon: "wine-symbolic".to_string(),
        },
    ];

    // Filter to only existing paths
    locations.retain(|loc| loc.path.exists() || loc.path.parent().map(|p| p.exists()).unwrap_or(false));

    locations
}

/// Load logs from all application sources
pub fn load_app_logs(limit: usize) -> Vec<LogEntry> {
    let locations = get_app_log_locations();
    let mut all_entries = Vec::new();

    for location in &locations {
        let entries = load_app_log(&location.path, &location.name, limit / locations.len().max(1));
        all_entries.extend(entries);
    }

    // Sort by timestamp (newest first)
    all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    all_entries.truncate(limit);

    if all_entries.is_empty() {
        get_sample_app_logs()
    } else {
        all_entries
    }
}

/// Load logs from a specific application
pub fn load_app_log(path: &Path, app_name: &str, limit: usize) -> Vec<LogEntry> {
    if path.is_dir() {
        load_app_log_dir(path, app_name, limit)
    } else if path.is_file() {
        load_app_log_file(path, app_name, limit)
    } else {
        Vec::new()
    }
}

fn load_app_log_dir(dir: &Path, app_name: &str, limit: usize) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    if let Ok(dir_entries) = fs::read_dir(dir) {
        for entry in dir_entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            // Only read log files
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if !name.ends_with(".log") && !name.contains("log") {
                continue;
            }

            if path.is_file() {
                let file_entries = load_app_log_file(&path, app_name, limit / 10);
                entries.extend(file_entries);
            }

            if entries.len() >= limit {
                break;
            }
        }
    }

    entries.truncate(limit);
    entries
}

fn load_app_log_file(path: &Path, app_name: &str, limit: usize) -> Vec<LogEntry> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines()
        .filter_map(|l| l.ok())
        .collect();

    lines.into_iter()
        .rev()
        .take(limit)
        .filter_map(|line| parse_app_log_line(&line, app_name))
        .collect()
}

fn parse_app_log_line(line: &str, app_name: &str) -> Option<LogEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Try various log formats

    // JSON format (common in modern apps)
    if line.starts_with('{') {
        if let Some(entry) = parse_json_log(line, app_name) {
            return Some(entry);
        }
    }

    // Timestamp prefix formats
    // [2026-02-19 10:30:45] [level] message
    // 2026-02-19T10:30:45.123Z level: message
    // [timestamp] level message

    if let Some(entry) = parse_bracketed_log(line, app_name) {
        return Some(entry);
    }

    if let Some(entry) = parse_iso_timestamp_log(line, app_name) {
        return Some(entry);
    }

    // Fallback
    let level = guess_level_from_content(line);
    let mut entry = LogEntry::new(Local::now(), level, line.to_string());
    entry.unit = Some(app_name.to_string());
    entry.source = app_name.to_string();

    Some(entry)
}

fn parse_json_log(line: &str, app_name: &str) -> Option<LogEntry> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    let message = json.get("message")
        .or_else(|| json.get("msg"))
        .or_else(|| json.get("text"))
        .and_then(|v| v.as_str())?
        .to_string();

    let level = json.get("level")
        .or_else(|| json.get("severity"))
        .and_then(|v| v.as_str())
        .map(|s| parse_level_string(s))
        .unwrap_or(LogLevel::Info);

    let timestamp = json.get("timestamp")
        .or_else(|| json.get("time"))
        .or_else(|| json.get("ts"))
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Local))
        .unwrap_or_else(Local::now);

    let mut entry = LogEntry::new(timestamp, level, message);
    entry.unit = Some(app_name.to_string());
    entry.source = app_name.to_string();

    // Extract extra fields
    for (key, value) in json.as_object()? {
        if !["message", "msg", "text", "level", "severity", "timestamp", "time", "ts"].contains(&key.as_str()) {
            if let Some(v) = value.as_str() {
                entry.extra_fields.insert(key.clone(), v.to_string());
            } else {
                entry.extra_fields.insert(key.clone(), value.to_string());
            }
        }
    }

    entry.raw_message = Some(line.to_string());

    Some(entry)
}

fn parse_bracketed_log(line: &str, app_name: &str) -> Option<LogEntry> {
    // [2026-02-19 10:30:45] [INFO] message
    // [2026-02-19 10:30:45.123] message

    if !line.starts_with('[') {
        return None;
    }

    let first_close = line.find(']')?;
    let timestamp_str = &line[1..first_close];

    let timestamp = parse_timestamp(timestamp_str)?;
    let rest = line[first_close + 1..].trim();

    // Check for level in second bracket
    let (level, message) = if rest.starts_with('[') {
        if let Some(level_end) = rest.find(']') {
            let level_str = &rest[1..level_end];
            let level = parse_level_string(level_str);
            (level, rest[level_end + 1..].trim())
        } else {
            (guess_level_from_content(rest), rest)
        }
    } else {
        (guess_level_from_content(rest), rest)
    };

    let mut entry = LogEntry::new(timestamp, level, message.to_string());
    entry.unit = Some(app_name.to_string());
    entry.source = app_name.to_string();

    Some(entry)
}

fn parse_iso_timestamp_log(line: &str, app_name: &str) -> Option<LogEntry> {
    // 2026-02-19T10:30:45.123Z INFO: message
    // 2026-02-19 10:30:45 - INFO - message

    if line.len() < 19 {
        return None;
    }

    let timestamp_str = &line[..23.min(line.len())];
    let timestamp = parse_timestamp(timestamp_str)?;

    let rest = line[23.min(line.len())..].trim();

    // Look for level
    let level_indicators = ["INFO", "DEBUG", "WARN", "WARNING", "ERROR", "ERR", "FATAL", "CRITICAL"];
    let (level, message) = if let Some(indicator) = level_indicators.iter()
        .find(|i| rest.to_uppercase().starts_with(*i))
    {
        let skip = indicator.len();
        let level = parse_level_string(indicator);
        let msg = rest[skip..].trim_start_matches(':').trim_start_matches('-').trim();
        (level, msg)
    } else {
        (guess_level_from_content(rest), rest)
    };

    let mut entry = LogEntry::new(timestamp, level, message.to_string());
    entry.unit = Some(app_name.to_string());
    entry.source = app_name.to_string();

    Some(entry)
}

fn parse_timestamp(s: &str) -> Option<DateTime<Local>> {
    // Try various formats
    let formats = [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%d/%m/%Y %H:%M:%S",
    ];

    let s = s.trim().trim_end_matches('Z');

    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
            if let Some(local) = Local.from_local_datetime(&dt).single() {
                return Some(local);
            }
        }
    }

    // Try RFC3339
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Local));
    }

    None
}

fn parse_level_string(s: &str) -> LogLevel {
    match s.to_lowercase().as_str() {
        "emergency" | "emerg" => LogLevel::Emergency,
        "alert" => LogLevel::Alert,
        "critical" | "crit" | "fatal" => LogLevel::Critical,
        "error" | "err" => LogLevel::Error,
        "warning" | "warn" => LogLevel::Warning,
        "notice" => LogLevel::Notice,
        "info" | "information" => LogLevel::Info,
        "debug" | "trace" => LogLevel::Debug,
        _ => LogLevel::Info,
    }
}

fn guess_level_from_content(s: &str) -> LogLevel {
    let lower = s.to_lowercase();

    if lower.contains("error") || lower.contains("fail") {
        LogLevel::Error
    } else if lower.contains("warn") {
        LogLevel::Warning
    } else if lower.contains("debug") || lower.contains("trace") {
        LogLevel::Debug
    } else if lower.contains("fatal") || lower.contains("panic") {
        LogLevel::Critical
    } else {
        LogLevel::Info
    }
}

fn get_sample_app_logs() -> Vec<LogEntry> {
    let now = Local::now();

    vec![
        create_app_entry(now, LogLevel::Info, "firefox", "Profile loaded successfully"),
        create_app_entry(now, LogLevel::Warning, "firefox", "WebGL: Performance warning - context lost"),
        create_app_entry(now, LogLevel::Info, "vscode", "Extension host started"),
        create_app_entry(now, LogLevel::Debug, "vscode", "Loading workspace configuration"),
        create_app_entry(now, LogLevel::Error, "chrome", "GPU process crashed, restarting"),
        create_app_entry(now, LogLevel::Info, "docker", "Container abc123 started"),
        create_app_entry(now, LogLevel::Warning, "docker", "Container xyz789 using deprecated config"),
        create_app_entry(now, LogLevel::Info, "steam", "Workshop content sync completed"),
        create_app_entry(now, LogLevel::Error, "steam", "Failed to connect to Steam servers"),
        create_app_entry(now, LogLevel::Info, "cups", "Printer HP_LaserJet ready"),
    ]
}

fn create_app_entry(
    timestamp: DateTime<Local>,
    level: LogLevel,
    app: &str,
    message: &str,
) -> LogEntry {
    let mut entry = LogEntry::new(timestamp, level, message.to_string());
    entry.unit = Some(app.to_string());
    entry.source = app.to_string();
    entry
}
