//! Event data structure

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use uuid::Uuid;

/// Calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier
    pub id: String,

    /// Event title
    pub title: String,

    /// Event description
    pub description: Option<String>,

    /// Event location
    pub location: Option<String>,

    /// Start date
    pub start_date: NaiveDate,

    /// Start time (None for all-day events)
    pub start_time: Option<NaiveTime>,

    /// End date
    pub end_date: NaiveDate,

    /// End time (None for all-day events)
    pub end_time: Option<NaiveTime>,

    /// Is all-day event
    pub all_day: bool,

    /// Event color (hex)
    pub color: String,

    /// Calendar ID this event belongs to
    pub calendar_id: String,

    /// Calendar name (for display)
    pub calendar_name: String,

    /// Recurrence rule
    pub recurrence: Option<Recurrence>,

    /// Reminders
    pub reminders: Vec<Reminder>,

    /// Category/tag
    pub category: Option<String>,

    /// Notes
    pub notes: Option<String>,

    /// URL
    pub url: Option<String>,

    /// Created timestamp
    pub created_at: NaiveDateTime,

    /// Last modified timestamp
    pub updated_at: NaiveDateTime,
}

impl Event {
    /// Create a new event
    pub fn new(
        title: String,
        start_date: NaiveDate,
        calendar_id: String,
        calendar_name: String,
        color: String,
    ) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: None,
            location: None,
            start_date,
            start_time: None,
            end_date: start_date,
            end_time: None,
            all_day: true,
            color,
            calendar_id,
            calendar_name,
            recurrence: None,
            reminders: Vec::new(),
            category: None,
            notes: None,
            url: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set event time range
    pub fn with_time(mut self, start: NaiveTime, end: NaiveTime) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self.all_day = false;
        self
    }

    /// Set event description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set event location
    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    /// Set recurrence
    pub fn with_recurrence(mut self, recurrence: Recurrence) -> Self {
        self.recurrence = Some(recurrence);
        self
    }

    /// Add reminder
    pub fn add_reminder(mut self, reminder: Reminder) -> Self {
        self.reminders.push(reminder);
        self
    }

    /// Check if event occurs on a given date
    pub fn occurs_on(&self, date: NaiveDate) -> bool {
        if self.start_date <= date && date <= self.end_date {
            return true;
        }

        // Check recurrence
        if let Some(ref recurrence) = self.recurrence {
            return recurrence.occurs_on(self.start_date, date);
        }

        false
    }

    /// Get duration in minutes
    pub fn duration_minutes(&self) -> i64 {
        if self.all_day {
            return 24 * 60;
        }

        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            let start_mins = start.hour() as i64 * 60 + start.minute() as i64;
            let end_mins = end.hour() as i64 * 60 + end.minute() as i64;
            return end_mins - start_mins;
        }

        0
    }
}

/// Recurrence rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recurrence {
    /// Type of recurrence
    pub recurrence_type: RecurrenceType,

    /// Interval (e.g., every 2 weeks)
    pub interval: u32,

    /// End date (None for infinite)
    pub end_date: Option<NaiveDate>,

    /// Number of occurrences (alternative to end_date)
    pub count: Option<u32>,

    /// Days of week (for weekly recurrence)
    pub days_of_week: Vec<chrono::Weekday>,

    /// Day of month (for monthly recurrence)
    pub day_of_month: Option<u32>,
}

impl Recurrence {
    /// Create daily recurrence
    pub fn daily() -> Self {
        Self {
            recurrence_type: RecurrenceType::Daily,
            interval: 1,
            end_date: None,
            count: None,
            days_of_week: Vec::new(),
            day_of_month: None,
        }
    }

    /// Create weekly recurrence
    pub fn weekly(days: Vec<chrono::Weekday>) -> Self {
        Self {
            recurrence_type: RecurrenceType::Weekly,
            interval: 1,
            end_date: None,
            count: None,
            days_of_week: days,
            day_of_month: None,
        }
    }

    /// Create monthly recurrence
    pub fn monthly(day: u32) -> Self {
        Self {
            recurrence_type: RecurrenceType::Monthly,
            interval: 1,
            end_date: None,
            count: None,
            days_of_week: Vec::new(),
            day_of_month: Some(day),
        }
    }

    /// Create yearly recurrence
    pub fn yearly() -> Self {
        Self {
            recurrence_type: RecurrenceType::Yearly,
            interval: 1,
            end_date: None,
            count: None,
            days_of_week: Vec::new(),
            day_of_month: None,
        }
    }

    /// Set interval
    pub fn with_interval(mut self, interval: u32) -> Self {
        self.interval = interval;
        self
    }

    /// Set end date
    pub fn until(mut self, date: NaiveDate) -> Self {
        self.end_date = Some(date);
        self
    }

    /// Set count
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Check if recurrence occurs on a given date
    pub fn occurs_on(&self, start: NaiveDate, check_date: NaiveDate) -> bool {
        if check_date < start {
            return false;
        }

        if let Some(end) = self.end_date {
            if check_date > end {
                return false;
            }
        }

        match self.recurrence_type {
            RecurrenceType::Daily => {
                let days_diff = (check_date - start).num_days();
                days_diff >= 0 && days_diff % self.interval as i64 == 0
            }
            RecurrenceType::Weekly => {
                let days_diff = (check_date - start).num_days();
                let weeks_diff = days_diff / 7;

                if weeks_diff % self.interval as i64 != 0 {
                    return false;
                }

                if self.days_of_week.is_empty() {
                    check_date.weekday() == start.weekday()
                } else {
                    self.days_of_week.contains(&check_date.weekday())
                }
            }
            RecurrenceType::Monthly => {
                use chrono::Datelike;

                let month_diff = (check_date.year() - start.year()) * 12
                    + (check_date.month() as i32 - start.month() as i32);

                if month_diff < 0 || month_diff % self.interval as i32 != 0 {
                    return false;
                }

                let expected_day = self.day_of_month.unwrap_or(start.day());
                check_date.day() == expected_day
            }
            RecurrenceType::Yearly => {
                use chrono::Datelike;

                let year_diff = check_date.year() - start.year();
                if year_diff < 0 || year_diff % self.interval as i32 != 0 {
                    return false;
                }

                check_date.month() == start.month() && check_date.day() == start.day()
            }
        }
    }
}

/// Recurrence type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceType {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Daily => "Diario",
            Self::Weekly => "Semanal",
            Self::Monthly => "Mensal",
            Self::Yearly => "Anual",
        }
    }
}

/// Reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    /// Time before event
    pub time_before: u32,

    /// Unit of time
    pub unit: ReminderUnit,

    /// Reminder type
    pub method: ReminderMethod,
}

impl Reminder {
    /// Create a new reminder
    pub fn new(time_before: u32, unit: ReminderUnit) -> Self {
        Self {
            time_before,
            unit,
            method: ReminderMethod::Notification,
        }
    }

    /// Get minutes before event
    pub fn minutes_before(&self) -> u32 {
        match self.unit {
            ReminderUnit::Minutes => self.time_before,
            ReminderUnit::Hours => self.time_before * 60,
            ReminderUnit::Days => self.time_before * 60 * 24,
            ReminderUnit::Weeks => self.time_before * 60 * 24 * 7,
        }
    }
}

/// Reminder time unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReminderUnit {
    Minutes,
    Hours,
    Days,
    Weeks,
}

impl ReminderUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minutes => "minutos",
            Self::Hours => "horas",
            Self::Days => "dias",
            Self::Weeks => "semanas",
        }
    }
}

/// Reminder method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReminderMethod {
    Notification,
    Email,
    Sound,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Baixa",
            Self::Medium => "Media",
            Self::High => "Alta",
            Self::Urgent => "Urgente",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Low => "#57e389",
            Self::Medium => "#f9f06b",
            Self::High => "#ff7800",
            Self::Urgent => "#ed333b",
        }
    }
}
