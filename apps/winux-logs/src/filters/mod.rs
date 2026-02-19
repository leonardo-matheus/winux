// Winux Logs - Filters Module
// Copyright (c) 2026 Winux OS Project

pub mod level;
pub mod unit;
pub mod time;
pub mod search;

use crate::sources::{LogEntry, LogLevel};
use chrono::{DateTime, Local};
use regex::Regex;

/// Combined filter state
#[derive(Debug, Clone, Default)]
pub struct FilterState {
    /// Filter by log levels (empty = all levels)
    pub levels: Vec<LogLevel>,

    /// Filter by systemd units (empty = all units)
    pub units: Vec<String>,

    /// Filter by time range
    pub time_from: Option<DateTime<Local>>,
    pub time_to: Option<DateTime<Local>>,

    /// Text search query (supports regex)
    pub search_query: Option<String>,

    /// Whether search is case-sensitive
    pub search_case_sensitive: bool,

    /// Whether search uses regex
    pub search_regex: bool,

    /// Compiled regex (cached)
    #[allow(dead_code)]
    compiled_regex: Option<Regex>,
}

impl FilterState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create filter with level selection
    pub fn with_levels(mut self, levels: Vec<LogLevel>) -> Self {
        self.levels = levels;
        self
    }

    /// Create filter with unit selection
    pub fn with_units(mut self, units: Vec<String>) -> Self {
        self.units = units;
        self
    }

    /// Create filter with time range
    pub fn with_time_range(mut self, from: Option<DateTime<Local>>, to: Option<DateTime<Local>>) -> Self {
        self.time_from = from;
        self.time_to = to;
        self
    }

    /// Create filter with search query
    pub fn with_search(mut self, query: String, case_sensitive: bool, regex: bool) -> Self {
        self.search_query = Some(query);
        self.search_case_sensitive = case_sensitive;
        self.search_regex = regex;
        self.compile_regex();
        self
    }

    /// Compile regex pattern if search uses regex
    fn compile_regex(&mut self) {
        if self.search_regex {
            if let Some(ref query) = self.search_query {
                let pattern = if self.search_case_sensitive {
                    query.clone()
                } else {
                    format!("(?i){}", query)
                };

                self.compiled_regex = Regex::new(&pattern).ok();
            }
        } else {
            self.compiled_regex = None;
        }
    }

    /// Check if a log entry matches all active filters
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Level filter
        if !self.levels.is_empty() && !self.levels.contains(&entry.level) {
            return false;
        }

        // Unit filter
        if !self.units.is_empty() {
            let unit_matches = entry.unit.as_ref()
                .map(|u| self.units.iter().any(|f| u.contains(f)))
                .unwrap_or(false);

            if !unit_matches {
                return false;
            }
        }

        // Time range filter
        if let Some(from) = self.time_from {
            if entry.timestamp < from {
                return false;
            }
        }

        if let Some(to) = self.time_to {
            if entry.timestamp > to {
                return false;
            }
        }

        // Search filter
        if let Some(ref query) = self.search_query {
            if !query.is_empty() {
                let matches = if self.search_regex {
                    if let Some(ref re) = self.compiled_regex {
                        re.is_match(&entry.message)
                    } else {
                        // Invalid regex, fall back to text search
                        self.text_matches(&entry.message, query)
                    }
                } else {
                    self.text_matches(&entry.message, query)
                };

                if !matches {
                    return false;
                }
            }
        }

        true
    }

    fn text_matches(&self, text: &str, query: &str) -> bool {
        if self.search_case_sensitive {
            text.contains(query)
        } else {
            text.to_lowercase().contains(&query.to_lowercase())
        }
    }

    /// Check if any filter is active
    pub fn is_active(&self) -> bool {
        !self.levels.is_empty() ||
        !self.units.is_empty() ||
        self.time_from.is_some() ||
        self.time_to.is_some() ||
        self.search_query.as_ref().map(|q| !q.is_empty()).unwrap_or(false)
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.levels.clear();
        self.units.clear();
        self.time_from = None;
        self.time_to = None;
        self.search_query = None;
        self.compiled_regex = None;
    }

    /// Get a description of active filters
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if !self.levels.is_empty() {
            let level_names: Vec<&str> = self.levels.iter()
                .map(|l| l.display_name())
                .collect();
            parts.push(format!("Niveis: {}", level_names.join(", ")));
        }

        if !self.units.is_empty() {
            parts.push(format!("Units: {}", self.units.join(", ")));
        }

        if self.time_from.is_some() || self.time_to.is_some() {
            let from = self.time_from.map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "inicio".to_string());
            let to = self.time_to.map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "agora".to_string());
            parts.push(format!("Periodo: {} - {}", from, to));
        }

        if let Some(ref query) = self.search_query {
            if !query.is_empty() {
                parts.push(format!("Busca: \"{}\"", query));
            }
        }

        if parts.is_empty() {
            "Sem filtros".to_string()
        } else {
            parts.join(" | ")
        }
    }
}

/// Filter entries with the given filter state
pub fn filter_entries<'a>(entries: &'a [LogEntry], filter: &FilterState) -> Vec<&'a LogEntry> {
    entries.iter()
        .filter(|e| filter.matches(e))
        .collect()
}

/// Quick filter presets
pub mod presets {
    use super::*;

    /// Errors and critical messages only
    pub fn errors_only() -> FilterState {
        FilterState::new().with_levels(vec![
            LogLevel::Emergency,
            LogLevel::Alert,
            LogLevel::Critical,
            LogLevel::Error,
        ])
    }

    /// Warnings and above
    pub fn warnings_and_above() -> FilterState {
        FilterState::new().with_levels(vec![
            LogLevel::Emergency,
            LogLevel::Alert,
            LogLevel::Critical,
            LogLevel::Error,
            LogLevel::Warning,
        ])
    }

    /// Last hour
    pub fn last_hour() -> FilterState {
        use chrono::Duration;
        let now = Local::now();
        FilterState::new().with_time_range(
            Some(now - Duration::hours(1)),
            Some(now),
        )
    }

    /// Today's logs
    pub fn today() -> FilterState {
        let now = Local::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0)
            .and_then(|dt| Local.from_local_datetime(&dt).single());
        FilterState::new().with_time_range(today_start, Some(now))
    }

    /// Current boot only (uses journald boot filter in practice)
    pub fn current_boot() -> FilterState {
        FilterState::new() // Would be handled at journal level
    }
}
