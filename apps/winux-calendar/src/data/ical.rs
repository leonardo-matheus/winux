//! iCal (ICS) parser and exporter

use anyhow::{Result, Context};
use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use std::path::Path;
use std::fs;

use super::{Event, Recurrence, RecurrenceType, CalendarInfo};

/// iCal parser
pub struct ICalParser;

impl ICalParser {
    /// Parse an ICS file
    pub fn parse_file(path: &Path) -> Result<Vec<Event>> {
        let content = fs::read_to_string(path)
            .context("Failed to read ICS file")?;
        Self::parse_string(&content)
    }

    /// Parse ICS content from string
    pub fn parse_string(content: &str) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        let mut in_event = false;
        let mut current_event: Option<EventBuilder> = None;

        for line in content.lines() {
            let line = line.trim();

            if line == "BEGIN:VEVENT" {
                in_event = true;
                current_event = Some(EventBuilder::new());
            } else if line == "END:VEVENT" {
                in_event = false;
                if let Some(builder) = current_event.take() {
                    if let Ok(event) = builder.build() {
                        events.push(event);
                    }
                }
            } else if in_event {
                if let Some(ref mut builder) = current_event {
                    Self::parse_event_property(builder, line);
                }
            }
        }

        Ok(events)
    }

    fn parse_event_property(builder: &mut EventBuilder, line: &str) {
        if let Some(pos) = line.find(':') {
            let (key, value) = line.split_at(pos);
            let value = &value[1..]; // Skip the ':'

            // Handle parameters in key (e.g., DTSTART;VALUE=DATE:20240101)
            let key = key.split(';').next().unwrap_or(key);

            match key {
                "UID" => builder.id = Some(value.to_string()),
                "SUMMARY" => builder.title = Some(Self::unescape(value)),
                "DESCRIPTION" => builder.description = Some(Self::unescape(value)),
                "LOCATION" => builder.location = Some(Self::unescape(value)),
                "DTSTART" => builder.start = Self::parse_datetime(value),
                "DTEND" => builder.end = Self::parse_datetime(value),
                "RRULE" => builder.rrule = Some(value.to_string()),
                "CATEGORIES" => builder.category = Some(value.to_string()),
                "URL" => builder.url = Some(value.to_string()),
                _ => {}
            }
        }
    }

    fn parse_datetime(value: &str) -> Option<(NaiveDate, Option<NaiveTime>)> {
        // Handle date-only format: 20240101
        if value.len() == 8 {
            if let Ok(date) = NaiveDate::parse_from_str(value, "%Y%m%d") {
                return Some((date, None));
            }
        }

        // Handle datetime format: 20240101T120000 or 20240101T120000Z
        let value = value.trim_end_matches('Z');
        if value.len() >= 15 {
            if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S") {
                return Some((dt.date(), Some(dt.time())));
            }
        }

        None
    }

    fn unescape(value: &str) -> String {
        value
            .replace("\\n", "\n")
            .replace("\\,", ",")
            .replace("\\;", ";")
            .replace("\\\\", "\\")
    }
}

/// Event builder for parsing
struct EventBuilder {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    location: Option<String>,
    start: Option<(NaiveDate, Option<NaiveTime>)>,
    end: Option<(NaiveDate, Option<NaiveTime>)>,
    rrule: Option<String>,
    category: Option<String>,
    url: Option<String>,
}

impl EventBuilder {
    fn new() -> Self {
        Self {
            id: None,
            title: None,
            description: None,
            location: None,
            start: None,
            end: None,
            rrule: None,
            category: None,
            url: None,
        }
    }

    fn build(self) -> Result<Event> {
        let title = self.title.unwrap_or_else(|| "Sem titulo".to_string());
        let (start_date, start_time) = self.start.context("Missing start date")?;
        let (end_date, end_time) = self.end.unwrap_or((start_date, start_time));

        let mut event = Event::new(
            title,
            start_date,
            "local".to_string(),
            "Local".to_string(),
            "#3584e4".to_string(),
        );

        if let Some(id) = self.id {
            event.id = id;
        }

        event.end_date = end_date;
        event.description = self.description;
        event.location = self.location;
        event.category = self.category;
        event.url = self.url;

        if let Some(time) = start_time {
            event.start_time = Some(time);
            event.end_time = end_time;
            event.all_day = false;
        }

        if let Some(rrule) = self.rrule {
            if let Some(recurrence) = Self::parse_rrule(&rrule) {
                event.recurrence = Some(recurrence);
            }
        }

        Ok(event)
    }

    fn parse_rrule(rrule: &str) -> Option<Recurrence> {
        let mut freq = None;
        let mut interval = 1u32;
        let mut until = None;
        let mut count = None;

        for part in rrule.split(';') {
            if let Some(pos) = part.find('=') {
                let (key, value) = part.split_at(pos);
                let value = &value[1..];

                match key {
                    "FREQ" => {
                        freq = match value {
                            "DAILY" => Some(RecurrenceType::Daily),
                            "WEEKLY" => Some(RecurrenceType::Weekly),
                            "MONTHLY" => Some(RecurrenceType::Monthly),
                            "YEARLY" => Some(RecurrenceType::Yearly),
                            _ => None,
                        };
                    }
                    "INTERVAL" => {
                        interval = value.parse().unwrap_or(1);
                    }
                    "UNTIL" => {
                        if let Ok(date) = NaiveDate::parse_from_str(&value[..8], "%Y%m%d") {
                            until = Some(date);
                        }
                    }
                    "COUNT" => {
                        count = value.parse().ok();
                    }
                    _ => {}
                }
            }
        }

        freq.map(|recurrence_type| Recurrence {
            recurrence_type,
            interval,
            end_date: until,
            count,
            days_of_week: Vec::new(),
            day_of_month: None,
        })
    }
}

/// iCal exporter
pub struct ICalExporter;

impl ICalExporter {
    /// Export events to ICS format
    pub fn export(events: &[Event], calendar: &CalendarInfo) -> String {
        let mut output = String::new();

        // Calendar header
        output.push_str("BEGIN:VCALENDAR\r\n");
        output.push_str("VERSION:2.0\r\n");
        output.push_str("PRODID:-//Winux//Winux Calendar//EN\r\n");
        output.push_str(&format!("X-WR-CALNAME:{}\r\n", Self::escape(&calendar.name)));
        output.push_str("CALSCALE:GREGORIAN\r\n");
        output.push_str("METHOD:PUBLISH\r\n");

        // Events
        for event in events {
            output.push_str(&Self::export_event(event));
        }

        output.push_str("END:VCALENDAR\r\n");
        output
    }

    /// Export a single event to VEVENT format
    fn export_event(event: &Event) -> String {
        let mut output = String::new();

        output.push_str("BEGIN:VEVENT\r\n");
        output.push_str(&format!("UID:{}\r\n", event.id));
        output.push_str(&format!("SUMMARY:{}\r\n", Self::escape(&event.title)));

        // Start time
        if event.all_day {
            output.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n",
                event.start_date.format("%Y%m%d")));
        } else if let Some(time) = event.start_time {
            let dt = NaiveDateTime::new(event.start_date, time);
            output.push_str(&format!("DTSTART:{}\r\n", dt.format("%Y%m%dT%H%M%S")));
        }

        // End time
        if event.all_day {
            output.push_str(&format!("DTEND;VALUE=DATE:{}\r\n",
                event.end_date.format("%Y%m%d")));
        } else if let Some(time) = event.end_time {
            let dt = NaiveDateTime::new(event.end_date, time);
            output.push_str(&format!("DTEND:{}\r\n", dt.format("%Y%m%dT%H%M%S")));
        }

        // Optional fields
        if let Some(ref desc) = event.description {
            output.push_str(&format!("DESCRIPTION:{}\r\n", Self::escape(desc)));
        }

        if let Some(ref location) = event.location {
            output.push_str(&format!("LOCATION:{}\r\n", Self::escape(location)));
        }

        if let Some(ref category) = event.category {
            output.push_str(&format!("CATEGORIES:{}\r\n", category));
        }

        if let Some(ref url) = event.url {
            output.push_str(&format!("URL:{}\r\n", url));
        }

        // Recurrence
        if let Some(ref recurrence) = event.recurrence {
            output.push_str(&Self::export_rrule(recurrence));
        }

        // Timestamps
        output.push_str(&format!("CREATED:{}\r\n",
            event.created_at.format("%Y%m%dT%H%M%S")));
        output.push_str(&format!("LAST-MODIFIED:{}\r\n",
            event.updated_at.format("%Y%m%dT%H%M%S")));

        output.push_str("END:VEVENT\r\n");
        output
    }

    /// Export recurrence rule
    fn export_rrule(recurrence: &Recurrence) -> String {
        let mut rrule = String::from("RRULE:");

        let freq = match recurrence.recurrence_type {
            RecurrenceType::Daily => "DAILY",
            RecurrenceType::Weekly => "WEEKLY",
            RecurrenceType::Monthly => "MONTHLY",
            RecurrenceType::Yearly => "YEARLY",
        };

        rrule.push_str(&format!("FREQ={}", freq));

        if recurrence.interval > 1 {
            rrule.push_str(&format!(";INTERVAL={}", recurrence.interval));
        }

        if let Some(end) = recurrence.end_date {
            rrule.push_str(&format!(";UNTIL={}", end.format("%Y%m%d")));
        }

        if let Some(count) = recurrence.count {
            rrule.push_str(&format!(";COUNT={}", count));
        }

        rrule.push_str("\r\n");
        rrule
    }

    /// Escape special characters for iCal format
    fn escape(text: &str) -> String {
        text
            .replace('\\', "\\\\")
            .replace(';', "\\;")
            .replace(',', "\\,")
            .replace('\n', "\\n")
    }

    /// Export to file
    pub fn export_to_file(events: &[Event], calendar: &CalendarInfo, path: &Path) -> Result<()> {
        let content = Self::export(events, calendar);
        fs::write(path, content).context("Failed to write ICS file")?;
        Ok(())
    }
}
