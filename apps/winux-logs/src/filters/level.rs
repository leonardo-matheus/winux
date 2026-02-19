// Winux Logs - Level Filter
// Copyright (c) 2026 Winux OS Project

use crate::sources::{LogEntry, LogLevel};

/// Filter entries by log level
pub fn filter_by_level(entries: &[LogEntry], levels: &[LogLevel]) -> Vec<&LogEntry> {
    if levels.is_empty() {
        // No filter, return all
        entries.iter().collect()
    } else {
        entries.iter()
            .filter(|e| levels.contains(&e.level))
            .collect()
    }
}

/// Filter entries by minimum severity level
/// Returns entries with level >= min_level (lower numbers are more severe)
pub fn filter_by_min_level(entries: &[LogEntry], min_level: LogLevel) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|e| e.level <= min_level)
        .collect()
}

/// Get count of entries per level
pub fn count_by_level(entries: &[LogEntry]) -> LevelCounts {
    let mut counts = LevelCounts::default();

    for entry in entries {
        match entry.level {
            LogLevel::Emergency => counts.emergency += 1,
            LogLevel::Alert => counts.alert += 1,
            LogLevel::Critical => counts.critical += 1,
            LogLevel::Error => counts.error += 1,
            LogLevel::Warning => counts.warning += 1,
            LogLevel::Notice => counts.notice += 1,
            LogLevel::Info => counts.info += 1,
            LogLevel::Debug => counts.debug += 1,
        }
    }

    counts
}

/// Count of entries per log level
#[derive(Debug, Clone, Default)]
pub struct LevelCounts {
    pub emergency: usize,
    pub alert: usize,
    pub critical: usize,
    pub error: usize,
    pub warning: usize,
    pub notice: usize,
    pub info: usize,
    pub debug: usize,
}

impl LevelCounts {
    /// Total number of entries
    pub fn total(&self) -> usize {
        self.emergency + self.alert + self.critical + self.error +
        self.warning + self.notice + self.info + self.debug
    }

    /// Count of all error-level and above entries
    pub fn errors_total(&self) -> usize {
        self.emergency + self.alert + self.critical + self.error
    }

    /// Count of warnings
    pub fn warnings_total(&self) -> usize {
        self.warning
    }

    /// Get count for a specific level
    pub fn get(&self, level: LogLevel) -> usize {
        match level {
            LogLevel::Emergency => self.emergency,
            LogLevel::Alert => self.alert,
            LogLevel::Critical => self.critical,
            LogLevel::Error => self.error,
            LogLevel::Warning => self.warning,
            LogLevel::Notice => self.notice,
            LogLevel::Info => self.info,
            LogLevel::Debug => self.debug,
        }
    }
}

/// All available log levels for filter selection
pub fn all_levels() -> Vec<LogLevel> {
    vec![
        LogLevel::Emergency,
        LogLevel::Alert,
        LogLevel::Critical,
        LogLevel::Error,
        LogLevel::Warning,
        LogLevel::Notice,
        LogLevel::Info,
        LogLevel::Debug,
    ]
}

/// Common level filter presets
pub mod presets {
    use super::*;

    /// Emergency, Alert, Critical, Error
    pub fn errors_only() -> Vec<LogLevel> {
        vec![
            LogLevel::Emergency,
            LogLevel::Alert,
            LogLevel::Critical,
            LogLevel::Error,
        ]
    }

    /// Warning and above
    pub fn warnings_and_above() -> Vec<LogLevel> {
        vec![
            LogLevel::Emergency,
            LogLevel::Alert,
            LogLevel::Critical,
            LogLevel::Error,
            LogLevel::Warning,
        ]
    }

    /// Notice and above (no debug/info)
    pub fn notices_and_above() -> Vec<LogLevel> {
        vec![
            LogLevel::Emergency,
            LogLevel::Alert,
            LogLevel::Critical,
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Notice,
        ]
    }

    /// All except debug
    pub fn no_debug() -> Vec<LogLevel> {
        vec![
            LogLevel::Emergency,
            LogLevel::Alert,
            LogLevel::Critical,
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Notice,
            LogLevel::Info,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn make_entry(level: LogLevel) -> LogEntry {
        LogEntry::new(Local::now(), level, "test".to_string())
    }

    #[test]
    fn test_filter_by_level() {
        let entries = vec![
            make_entry(LogLevel::Error),
            make_entry(LogLevel::Warning),
            make_entry(LogLevel::Info),
        ];

        let filtered = filter_by_level(&entries, &[LogLevel::Error]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, LogLevel::Error);
    }

    #[test]
    fn test_filter_by_min_level() {
        let entries = vec![
            make_entry(LogLevel::Error),
            make_entry(LogLevel::Warning),
            make_entry(LogLevel::Info),
            make_entry(LogLevel::Debug),
        ];

        let filtered = filter_by_min_level(&entries, LogLevel::Warning);
        assert_eq!(filtered.len(), 2); // Error and Warning
    }

    #[test]
    fn test_count_by_level() {
        let entries = vec![
            make_entry(LogLevel::Error),
            make_entry(LogLevel::Error),
            make_entry(LogLevel::Warning),
            make_entry(LogLevel::Info),
        ];

        let counts = count_by_level(&entries);
        assert_eq!(counts.error, 2);
        assert_eq!(counts.warning, 1);
        assert_eq!(counts.info, 1);
        assert_eq!(counts.total(), 4);
    }
}
