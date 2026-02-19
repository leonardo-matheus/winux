// Winux Logs - Log Sources Module
// Copyright (c) 2026 Winux OS Project

pub mod journald;
pub mod kernel;
pub mod syslog;
pub mod app_logs;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Log severity levels following syslog priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Emergency = 0,  // System is unusable
    Alert = 1,      // Action must be taken immediately
    Critical = 2,   // Critical conditions
    Error = 3,      // Error conditions
    Warning = 4,    // Warning conditions
    Notice = 5,     // Normal but significant condition
    Info = 6,       // Informational messages
    Debug = 7,      // Debug-level messages
}

impl LogLevel {
    pub fn from_priority(priority: u8) -> Self {
        match priority {
            0 => LogLevel::Emergency,
            1 => LogLevel::Alert,
            2 => LogLevel::Critical,
            3 => LogLevel::Error,
            4 => LogLevel::Warning,
            5 => LogLevel::Notice,
            6 => LogLevel::Info,
            7 => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Emergency => "emerg",
            LogLevel::Alert => "alert",
            LogLevel::Critical => "crit",
            LogLevel::Error => "err",
            LogLevel::Warning => "warning",
            LogLevel::Notice => "notice",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LogLevel::Emergency => "Emergency",
            LogLevel::Alert => "Alert",
            LogLevel::Critical => "Critical",
            LogLevel::Error => "Error",
            LogLevel::Warning => "Warning",
            LogLevel::Notice => "Notice",
            LogLevel::Info => "Info",
            LogLevel::Debug => "Debug",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            LogLevel::Emergency | LogLevel::Alert | LogLevel::Critical | LogLevel::Error => "error",
            LogLevel::Warning => "warning",
            LogLevel::Notice | LogLevel::Info => "accent",
            LogLevel::Debug => "dim-label",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            LogLevel::Emergency | LogLevel::Alert | LogLevel::Critical => "dialog-error-symbolic",
            LogLevel::Error => "process-stop-symbolic",
            LogLevel::Warning => "dialog-warning-symbolic",
            LogLevel::Notice | LogLevel::Info => "dialog-information-symbolic",
            LogLevel::Debug => "preferences-system-symbolic",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Available log sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    Journal,    // systemd journal
    Kernel,     // dmesg / kernel ring buffer
    Syslog,     // Traditional /var/log files
    AppLogs,    // Application-specific logs
}

impl LogSource {
    pub fn display_name(&self) -> &'static str {
        match self {
            LogSource::Journal => "Journal",
            LogSource::Kernel => "Kernel",
            LogSource::Syslog => "Syslog",
            LogSource::AppLogs => "Aplicativos",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            LogSource::Journal => "text-x-log-symbolic",
            LogSource::Kernel => "computer-symbolic",
            LogSource::Syslog => "folder-documents-symbolic",
            LogSource::AppLogs => "application-x-executable-symbolic",
        }
    }
}

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Local>,
    /// Log level/priority
    pub level: LogLevel,
    /// Source of the log (journal, kernel, syslog, app)
    pub source: String,
    /// systemd unit name (if applicable)
    pub unit: Option<String>,
    /// Process ID
    pub pid: Option<u32>,
    /// User ID
    pub uid: Option<u32>,
    /// The log message
    pub message: String,
    /// Full raw message (for detail view)
    pub raw_message: Option<String>,
    /// Additional fields from journald
    #[serde(default)]
    pub extra_fields: HashMap<String, String>,
    /// Stack trace if present
    pub stack_trace: Option<String>,
    /// Boot ID
    pub boot_id: Option<String>,
    /// Machine ID
    pub machine_id: Option<String>,
    /// Hostname
    pub hostname: Option<String>,
}

impl LogEntry {
    pub fn new(timestamp: DateTime<Local>, level: LogLevel, message: String) -> Self {
        Self {
            timestamp,
            level,
            source: String::new(),
            unit: None,
            pid: None,
            uid: None,
            message,
            raw_message: None,
            extra_fields: HashMap::new(),
            stack_trace: None,
            boot_id: None,
            machine_id: None,
            hostname: None,
        }
    }

    /// Format timestamp for display
    pub fn formatted_time(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Format timestamp with microseconds
    pub fn formatted_time_precise(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
    }

    /// Get a short display string for the unit
    pub fn unit_display(&self) -> String {
        self.unit.clone().unwrap_or_else(|| "-".to_string())
    }

    /// Check if this entry contains a stack trace
    pub fn has_stack_trace(&self) -> bool {
        self.stack_trace.is_some() ||
        self.message.contains("Traceback") ||
        self.message.contains("at 0x") ||
        self.message.contains("panic")
    }

    /// Export as plain text
    pub fn to_text(&self) -> String {
        format!(
            "{} [{}] {}: {}",
            self.formatted_time(),
            self.level.as_str().to_uppercase(),
            self.unit_display(),
            self.message
        )
    }

    /// Export as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Boot information
#[derive(Debug, Clone)]
pub struct BootInfo {
    pub boot_id: String,
    pub timestamp: DateTime<Local>,
    pub offset: i32,  // 0 = current, -1 = previous, etc.
}

/// Get list of available boots
pub fn get_boot_list() -> Vec<BootInfo> {
    let output = std::process::Command::new("journalctl")
        .args(["--list-boots", "--no-pager", "-o", "json"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                // Parse boot list
                // Format: offset boot_id timestamp
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut boots = Vec::new();

                for line in stdout.lines() {
                    if let Ok(boot) = serde_json::from_str::<serde_json::Value>(line) {
                        if let (Some(offset), Some(boot_id)) = (
                            boot.get("boot_offset").and_then(|v| v.as_i64()),
                            boot.get("boot_id").and_then(|v| v.as_str()),
                        ) {
                            boots.push(BootInfo {
                                boot_id: boot_id.to_string(),
                                timestamp: Local::now(), // Would parse from JSON
                                offset: offset as i32,
                            });
                        }
                    }
                }

                if boots.is_empty() {
                    // Fallback: at least current boot
                    boots.push(BootInfo {
                        boot_id: String::new(),
                        timestamp: Local::now(),
                        offset: 0,
                    });
                }

                boots
            } else {
                vec![BootInfo {
                    boot_id: String::new(),
                    timestamp: Local::now(),
                    offset: 0,
                }]
            }
        }
        Err(_) => vec![BootInfo {
            boot_id: String::new(),
            timestamp: Local::now(),
            offset: 0,
        }],
    }
}
