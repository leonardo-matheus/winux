// Winux Logs - Text Search Filter
// Copyright (c) 2026 Winux OS Project

use crate::sources::LogEntry;
use regex::{Regex, RegexBuilder};

/// Search configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// The search query
    pub query: String,
    /// Case-sensitive search
    pub case_sensitive: bool,
    /// Use regex pattern
    pub use_regex: bool,
    /// Search in message only (false = search all fields)
    pub message_only: bool,
    /// Highlight matches
    pub highlight: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            case_sensitive: false,
            use_regex: false,
            message_only: true,
            highlight: true,
        }
    }
}

impl SearchConfig {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            ..Default::default()
        }
    }

    pub fn case_sensitive(mut self, value: bool) -> Self {
        self.case_sensitive = value;
        self
    }

    pub fn use_regex(mut self, value: bool) -> Self {
        self.use_regex = value;
        self
    }

    pub fn message_only(mut self, value: bool) -> Self {
        self.message_only = value;
        self
    }
}

/// Search result with match information
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Index of the entry in the original list
    pub entry_index: usize,
    /// Start positions of matches in the message
    pub match_positions: Vec<(usize, usize)>,  // (start, end)
}

/// Compiled searcher for efficient repeated searches
pub struct Searcher {
    config: SearchConfig,
    regex: Option<Regex>,
}

impl Searcher {
    pub fn new(config: SearchConfig) -> Result<Self, regex::Error> {
        let regex = if config.use_regex {
            Some(
                RegexBuilder::new(&config.query)
                    .case_insensitive(!config.case_sensitive)
                    .build()?
            )
        } else if !config.query.is_empty() {
            // Escape regex special characters for literal search
            let escaped = regex::escape(&config.query);
            Some(
                RegexBuilder::new(&escaped)
                    .case_insensitive(!config.case_sensitive)
                    .build()?
            )
        } else {
            None
        };

        Ok(Self { config, regex })
    }

    /// Check if an entry matches the search
    pub fn matches(&self, entry: &LogEntry) -> bool {
        let regex = match &self.regex {
            Some(r) => r,
            None => return true, // Empty search matches all
        };

        // Search in message
        if regex.is_match(&entry.message) {
            return true;
        }

        // Search in other fields if not message_only
        if !self.config.message_only {
            // Search in unit
            if let Some(ref unit) = entry.unit {
                if regex.is_match(unit) {
                    return true;
                }
            }

            // Search in source
            if regex.is_match(&entry.source) {
                return true;
            }

            // Search in extra fields
            for value in entry.extra_fields.values() {
                if regex.is_match(value) {
                    return true;
                }
            }

            // Search in raw message
            if let Some(ref raw) = entry.raw_message {
                if regex.is_match(raw) {
                    return true;
                }
            }
        }

        false
    }

    /// Find matches with position information
    pub fn find_matches(&self, entry: &LogEntry) -> Option<SearchMatch> {
        let regex = self.regex.as_ref()?;

        let mut positions = Vec::new();

        for m in regex.find_iter(&entry.message) {
            positions.push((m.start(), m.end()));
        }

        if positions.is_empty() && !self.config.message_only {
            // Check other fields
            if let Some(ref unit) = entry.unit {
                if regex.is_match(unit) {
                    positions.push((0, 0)); // Indicate match elsewhere
                }
            }
        }

        if positions.is_empty() {
            None
        } else {
            Some(SearchMatch {
                entry_index: 0, // Will be set by caller
                match_positions: positions,
            })
        }
    }

    /// Get the config
    pub fn config(&self) -> &SearchConfig {
        &self.config
    }
}

/// Filter entries by text search
pub fn filter_by_search(entries: &[LogEntry], query: &str) -> Vec<&LogEntry> {
    if query.is_empty() {
        return entries.iter().collect();
    }

    let config = SearchConfig::new(query);
    let searcher = match Searcher::new(config) {
        Ok(s) => s,
        Err(_) => return entries.iter().collect(), // Invalid regex, return all
    };

    entries.iter()
        .filter(|e| searcher.matches(e))
        .collect()
}

/// Filter entries with regex pattern
pub fn filter_by_regex(entries: &[LogEntry], pattern: &str) -> Result<Vec<&LogEntry>, regex::Error> {
    let config = SearchConfig::new(pattern).use_regex(true);
    let searcher = Searcher::new(config)?;

    Ok(entries.iter()
        .filter(|e| searcher.matches(e))
        .collect())
}

/// Search and return matches with positions
pub fn search_with_positions(
    entries: &[LogEntry],
    config: SearchConfig,
) -> Result<Vec<SearchMatch>, regex::Error> {
    let searcher = Searcher::new(config)?;
    let mut matches = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        if let Some(mut m) = searcher.find_matches(entry) {
            m.entry_index = i;
            matches.push(m);
        }
    }

    Ok(matches)
}

/// Highlight matches in text with Pango markup
pub fn highlight_matches(text: &str, pattern: &str, case_sensitive: bool) -> String {
    if pattern.is_empty() {
        return glib_escape(text);
    }

    let regex = match RegexBuilder::new(&regex::escape(pattern))
        .case_insensitive(!case_sensitive)
        .build()
    {
        Ok(r) => r,
        Err(_) => return glib_escape(text),
    };

    let mut result = String::new();
    let mut last_end = 0;

    for m in regex.find_iter(text) {
        // Add text before match
        result.push_str(&glib_escape(&text[last_end..m.start()]));
        // Add highlighted match
        result.push_str("<span background=\"#FFFF00\" foreground=\"#000000\">");
        result.push_str(&glib_escape(m.as_str()));
        result.push_str("</span>");
        last_end = m.end();
    }

    // Add remaining text
    result.push_str(&glib_escape(&text[last_end..]));

    result
}

/// Escape text for Pango markup
fn glib_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Common search patterns
pub mod patterns {
    /// Error messages
    pub const ERRORS: &str = r"(?i)(error|fail|fatal|exception)";

    /// Warning messages
    pub const WARNINGS: &str = r"(?i)(warn|warning)";

    /// Stack traces
    pub const STACK_TRACE: &str = r"(at 0x[0-9a-f]+|Traceback|panic|SIGSEGV|backtrace)";

    /// IP addresses
    pub const IP_ADDRESS: &str = r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}";

    /// Authentication events
    pub const AUTH_EVENTS: &str = r"(?i)(login|logout|auth|password|session|sudo)";

    /// Network events
    pub const NETWORK_EVENTS: &str = r"(?i)(connect|disconnect|network|interface|dhcp)";

    /// Disk/storage events
    pub const STORAGE_EVENTS: &str = r"(?i)(mount|unmount|disk|drive|partition|filesystem)";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::LogLevel;
    use chrono::Local;

    fn make_entry(message: &str) -> LogEntry {
        LogEntry::new(Local::now(), LogLevel::Info, message.to_string())
    }

    #[test]
    fn test_basic_search() {
        let entries = vec![
            make_entry("Starting service"),
            make_entry("Error: connection failed"),
            make_entry("Service started"),
        ];

        let filtered = filter_by_search(&entries, "error");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].message.contains("Error"));
    }

    #[test]
    fn test_case_insensitive_search() {
        let entries = vec![
            make_entry("ERROR: something failed"),
            make_entry("error: another issue"),
            make_entry("Warning: minor issue"),
        ];

        let filtered = filter_by_search(&entries, "error");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_regex_search() {
        let entries = vec![
            make_entry("Connection from 192.168.1.1"),
            make_entry("Connection from 10.0.0.5"),
            make_entry("No IP address"),
        ];

        let filtered = filter_by_regex(&entries, r"\d+\.\d+\.\d+\.\d+").unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_highlight_matches() {
        let text = "Error: connection failed";
        let highlighted = highlight_matches(text, "error", false);
        assert!(highlighted.contains("<span"));
        assert!(highlighted.contains("Error"));
    }
}
