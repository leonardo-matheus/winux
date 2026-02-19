// Winux Logs - Time Filter
// Copyright (c) 2026 Winux OS Project

use crate::sources::LogEntry;
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

/// Time range for filtering
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub from: Option<DateTime<Local>>,
    pub to: Option<DateTime<Local>>,
}

impl TimeRange {
    pub fn new(from: Option<DateTime<Local>>, to: Option<DateTime<Local>>) -> Self {
        Self { from, to }
    }

    /// Check if a timestamp is within this range
    pub fn contains(&self, timestamp: &DateTime<Local>) -> bool {
        if let Some(from) = self.from {
            if *timestamp < from {
                return false;
            }
        }

        if let Some(to) = self.to {
            if *timestamp > to {
                return false;
            }
        }

        true
    }

    /// Duration of the range (if both bounds are set)
    pub fn duration(&self) -> Option<Duration> {
        match (&self.from, &self.to) {
            (Some(from), Some(to)) => Some(*to - *from),
            _ => None,
        }
    }
}

impl Default for TimeRange {
    fn default() -> Self {
        Self { from: None, to: None }
    }
}

/// Filter entries by time range
pub fn filter_by_time_range(entries: &[LogEntry], range: &TimeRange) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|e| range.contains(&e.timestamp))
        .collect()
}

/// Filter entries from a specific point in time until now
pub fn filter_since(entries: &[LogEntry], since: DateTime<Local>) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|e| e.timestamp >= since)
        .collect()
}

/// Filter entries before a specific point in time
pub fn filter_until(entries: &[LogEntry], until: DateTime<Local>) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|e| e.timestamp <= until)
        .collect()
}

/// Preset time ranges
pub mod presets {
    use super::*;

    /// Last N minutes
    pub fn last_minutes(n: i64) -> TimeRange {
        let now = Local::now();
        TimeRange::new(
            Some(now - Duration::minutes(n)),
            Some(now),
        )
    }

    /// Last N hours
    pub fn last_hours(n: i64) -> TimeRange {
        let now = Local::now();
        TimeRange::new(
            Some(now - Duration::hours(n)),
            Some(now),
        )
    }

    /// Last N days
    pub fn last_days(n: i64) -> TimeRange {
        let now = Local::now();
        TimeRange::new(
            Some(now - Duration::days(n)),
            Some(now),
        )
    }

    /// Last 15 minutes
    pub fn last_15_min() -> TimeRange {
        last_minutes(15)
    }

    /// Last hour
    pub fn last_hour() -> TimeRange {
        last_hours(1)
    }

    /// Last 6 hours
    pub fn last_6_hours() -> TimeRange {
        last_hours(6)
    }

    /// Last 24 hours
    pub fn last_24_hours() -> TimeRange {
        last_hours(24)
    }

    /// Today (from midnight)
    pub fn today() -> TimeRange {
        let now = Local::now();
        let midnight = now.date_naive()
            .and_hms_opt(0, 0, 0)
            .and_then(|dt| Local.from_local_datetime(&dt).single());

        TimeRange::new(midnight, Some(now))
    }

    /// Yesterday
    pub fn yesterday() -> TimeRange {
        let now = Local::now();
        let yesterday = now.date_naive() - Duration::days(1);

        let from = yesterday.and_hms_opt(0, 0, 0)
            .and_then(|dt| Local.from_local_datetime(&dt).single());
        let to = yesterday.and_hms_opt(23, 59, 59)
            .and_then(|dt| Local.from_local_datetime(&dt).single());

        TimeRange::new(from, to)
    }

    /// This week (from Monday)
    pub fn this_week() -> TimeRange {
        let now = Local::now();
        let weekday = now.weekday().num_days_from_monday() as i64;
        let monday = now.date_naive() - Duration::days(weekday);

        let from = monday.and_hms_opt(0, 0, 0)
            .and_then(|dt| Local.from_local_datetime(&dt).single());

        TimeRange::new(from, Some(now))
    }

    /// This month
    pub fn this_month() -> TimeRange {
        let now = Local::now();
        let first_of_month = NaiveDate::from_ymd_opt(
            now.year(),
            now.month(),
            1
        );

        let from = first_of_month
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .and_then(|dt| Local.from_local_datetime(&dt).single());

        TimeRange::new(from, Some(now))
    }
}

/// Parse a time string into DateTime
pub fn parse_time_string(s: &str) -> Option<DateTime<Local>> {
    let s = s.trim();

    // Try relative formats first
    if let Some(dt) = parse_relative_time(s) {
        return Some(dt);
    }

    // Try absolute formats
    parse_absolute_time(s)
}

fn parse_relative_time(s: &str) -> Option<DateTime<Local>> {
    let s = s.to_lowercase();
    let now = Local::now();

    // "now"
    if s == "now" || s == "agora" {
        return Some(now);
    }

    // "today" / "hoje"
    if s == "today" || s == "hoje" {
        return now.date_naive().and_hms_opt(0, 0, 0)
            .and_then(|dt| Local.from_local_datetime(&dt).single());
    }

    // "yesterday" / "ontem"
    if s == "yesterday" || s == "ontem" {
        let yesterday = now.date_naive() - Duration::days(1);
        return yesterday.and_hms_opt(0, 0, 0)
            .and_then(|dt| Local.from_local_datetime(&dt).single());
    }

    // "N minutes/hours/days ago" patterns
    let patterns = [
        (r"(\d+)\s*(min|minute|minuto)s?\s*ago", |n: i64| Duration::minutes(n)),
        (r"(\d+)\s*(hour|hora)s?\s*ago", |n: i64| Duration::hours(n)),
        (r"(\d+)\s*(day|dia)s?\s*ago", |n: i64| Duration::days(n)),
        (r"(\d+)\s*(week|semana)s?\s*ago", |n: i64| Duration::weeks(n)),
    ];

    for (pattern, duration_fn) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(&s) {
                if let Some(n) = caps.get(1).and_then(|m| m.as_str().parse::<i64>().ok()) {
                    return Some(now - duration_fn(n));
                }
            }
        }
    }

    None
}

fn parse_absolute_time(s: &str) -> Option<DateTime<Local>> {
    // Full datetime formats
    let datetime_formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%d/%m/%Y %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d/%m/%Y %H:%M",
    ];

    for format in datetime_formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
            if let Some(local) = Local.from_local_datetime(&dt).single() {
                return Some(local);
            }
        }
    }

    // Date-only formats (assume start of day)
    let date_formats = [
        "%Y-%m-%d",
        "%d/%m/%Y",
        "%Y/%m/%d",
    ];

    for format in date_formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, format) {
            if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                if let Some(local) = Local.from_local_datetime(&dt).single() {
                    return Some(local);
                }
            }
        }
    }

    // Time-only formats (assume today)
    let time_formats = [
        "%H:%M:%S",
        "%H:%M",
    ];

    for format in time_formats {
        if let Ok(time) = NaiveTime::parse_from_str(s, format) {
            let date = Local::now().date_naive();
            if let Some(local) = Local.from_local_datetime(&date.and_time(time)).single() {
                return Some(local);
            }
        }
    }

    None
}

/// Format a duration for display
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.num_seconds();

    if total_secs < 60 {
        format!("{} segundos", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        format!("{} minutos", mins)
    } else if total_secs < 86400 {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        if mins > 0 {
            format!("{}h {}min", hours, mins)
        } else {
            format!("{} horas", hours)
        }
    } else {
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{} dias", days)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::LogLevel;

    fn make_entry(minutes_ago: i64) -> LogEntry {
        let timestamp = Local::now() - Duration::minutes(minutes_ago);
        LogEntry::new(timestamp, LogLevel::Info, "test".to_string())
    }

    #[test]
    fn test_filter_by_time_range() {
        let entries = vec![
            make_entry(5),   // 5 min ago
            make_entry(30),  // 30 min ago
            make_entry(120), // 2 hours ago
        ];

        let range = presets::last_hour();
        let filtered = filter_by_time_range(&entries, &range);
        assert_eq!(filtered.len(), 2); // Only entries within last hour
    }

    #[test]
    fn test_parse_relative_time() {
        let now = Local::now();

        let today = parse_time_string("today").unwrap();
        assert_eq!(today.date_naive(), now.date_naive());

        let yesterday = parse_time_string("yesterday").unwrap();
        assert_eq!(yesterday.date_naive(), now.date_naive() - Duration::days(1));
    }

    #[test]
    fn test_parse_absolute_time() {
        let dt = parse_time_string("2026-02-19 10:30:00").unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 19);
    }
}
