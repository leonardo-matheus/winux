//! Calendar and calendar store

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, NaiveDateTime};
use uuid::Uuid;
use std::collections::HashMap;

use super::{Event, Priority};

/// Calendar information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarInfo {
    /// Unique identifier
    pub id: String,

    /// Calendar name
    pub name: String,

    /// Calendar color (hex)
    pub color: String,

    /// Whether calendar is visible
    pub visible: bool,

    /// Calendar description
    pub description: Option<String>,

    /// Is this the default calendar
    pub is_default: bool,

    /// Sync source (local, caldav, google, etc)
    pub source: CalendarSource,

    /// Remote URL (for CalDAV)
    pub remote_url: Option<String>,

    /// Last sync time
    pub last_sync: Option<NaiveDateTime>,

    /// Read-only calendar
    pub read_only: bool,
}

impl CalendarInfo {
    /// Create a new local calendar
    pub fn new(name: &str, color: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            color: color.to_string(),
            visible: true,
            description: None,
            is_default: false,
            source: CalendarSource::Local,
            remote_url: None,
            last_sync: None,
            read_only: false,
        }
    }

    /// Create a CalDAV calendar
    pub fn caldav(name: &str, color: &str, url: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            color: color.to_string(),
            visible: true,
            description: None,
            is_default: false,
            source: CalendarSource::CalDAV,
            remote_url: Some(url.to_string()),
            last_sync: None,
            read_only: false,
        }
    }

    /// Set as default calendar
    pub fn set_default(mut self) -> Self {
        self.is_default = true;
        self
    }
}

/// Calendar source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarSource {
    Local,
    CalDAV,
    Google,
    ICloud,
    Exchange,
    Subscribed,
}

impl CalendarSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::CalDAV => "CalDAV",
            Self::Google => "Google",
            Self::ICloud => "iCloud",
            Self::Exchange => "Exchange",
            Self::Subscribed => "Inscrito",
        }
    }
}

/// Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: String,

    /// Task title
    pub title: String,

    /// Task description
    pub description: Option<String>,

    /// Due date
    pub due_date: Option<NaiveDate>,

    /// Due time
    pub due_time: Option<chrono::NaiveTime>,

    /// Is completed
    pub completed: bool,

    /// Completion date
    pub completed_at: Option<NaiveDateTime>,

    /// Priority
    pub priority: Priority,

    /// Calendar/list ID
    pub calendar_id: String,

    /// Created timestamp
    pub created_at: NaiveDateTime,

    /// Last modified timestamp
    pub updated_at: NaiveDateTime,

    /// Subtasks
    pub subtasks: Vec<SubTask>,

    /// Tags
    pub tags: Vec<String>,
}

impl Task {
    /// Create a new task
    pub fn new(title: String, calendar_id: String) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: None,
            due_date: None,
            due_time: None,
            completed: false,
            completed_at: None,
            priority: Priority::Medium,
            calendar_id,
            created_at: now,
            updated_at: now,
            subtasks: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Set due date
    pub fn with_due_date(mut self, date: NaiveDate) -> Self {
        self.due_date = Some(date);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Toggle completion
    pub fn toggle_complete(&mut self) {
        self.completed = !self.completed;
        if self.completed {
            self.completed_at = Some(chrono::Local::now().naive_local());
        } else {
            self.completed_at = None;
        }
        self.updated_at = chrono::Local::now().naive_local();
    }

    /// Is overdue
    pub fn is_overdue(&self) -> bool {
        if self.completed {
            return false;
        }
        if let Some(due) = self.due_date {
            return due < chrono::Local::now().date_naive();
        }
        false
    }
}

/// Subtask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

impl SubTask {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            completed: false,
        }
    }
}

/// Calendar store - manages all calendars, events and tasks
#[derive(Debug)]
pub struct CalendarStore {
    /// All calendars
    calendars: Vec<CalendarInfo>,

    /// All events indexed by calendar ID
    events: HashMap<String, Vec<Event>>,

    /// All tasks indexed by calendar ID
    tasks: HashMap<String, Vec<Task>>,
}

impl CalendarStore {
    /// Create a new calendar store
    pub fn new() -> Self {
        Self {
            calendars: Vec::new(),
            events: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Add a calendar
    pub fn add_calendar(&mut self, calendar: CalendarInfo) {
        let id = calendar.id.clone();
        self.calendars.push(calendar);
        self.events.insert(id.clone(), Vec::new());
        self.tasks.insert(id, Vec::new());
    }

    /// Remove a calendar
    pub fn remove_calendar(&mut self, id: &str) {
        self.calendars.retain(|c| c.id != id);
        self.events.remove(id);
        self.tasks.remove(id);
    }

    /// Get all calendars
    pub fn calendars(&self) -> &[CalendarInfo] {
        &self.calendars
    }

    /// Get a calendar by ID
    pub fn get_calendar(&self, id: &str) -> Option<&CalendarInfo> {
        self.calendars.iter().find(|c| c.id == id)
    }

    /// Get the default calendar
    pub fn default_calendar(&self) -> Option<&CalendarInfo> {
        self.calendars.iter().find(|c| c.is_default)
            .or_else(|| self.calendars.first())
    }

    /// Add an event
    pub fn add_event(&mut self, event: Event) {
        let calendar_id = event.calendar_id.clone();
        if let Some(events) = self.events.get_mut(&calendar_id) {
            events.push(event);
        }
    }

    /// Update an event
    pub fn update_event(&mut self, event: Event) {
        let calendar_id = event.calendar_id.clone();
        if let Some(events) = self.events.get_mut(&calendar_id) {
            if let Some(existing) = events.iter_mut().find(|e| e.id == event.id) {
                *existing = event;
            }
        }
    }

    /// Remove an event
    pub fn remove_event(&mut self, calendar_id: &str, event_id: &str) {
        if let Some(events) = self.events.get_mut(calendar_id) {
            events.retain(|e| e.id != event_id);
        }
    }

    /// Get all events for a date
    pub fn get_events_for_date(&self, date: NaiveDate) -> Vec<Event> {
        let mut result = Vec::new();

        for calendar in &self.calendars {
            if !calendar.visible {
                continue;
            }

            if let Some(events) = self.events.get(&calendar.id) {
                for event in events {
                    if event.occurs_on(date) {
                        result.push(event.clone());
                    }
                }
            }
        }

        // Sort by time
        result.sort_by(|a, b| {
            let a_time = a.start_time.unwrap_or(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            let b_time = b.start_time.unwrap_or(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            a_time.cmp(&b_time)
        });

        result
    }

    /// Get events for a date range
    pub fn get_events_for_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<Event> {
        let mut result = Vec::new();

        let mut current = start;
        while current <= end {
            let day_events = self.get_events_for_date(current);
            for event in day_events {
                if !result.iter().any(|e: &Event| e.id == event.id) {
                    result.push(event);
                }
            }
            current += chrono::Duration::days(1);
        }

        result
    }

    /// Add a task
    pub fn add_task(&mut self, task: Task) {
        let calendar_id = task.calendar_id.clone();
        if let Some(tasks) = self.tasks.get_mut(&calendar_id) {
            tasks.push(task);
        }
    }

    /// Update a task
    pub fn update_task(&mut self, task: Task) {
        let calendar_id = task.calendar_id.clone();
        if let Some(tasks) = self.tasks.get_mut(&calendar_id) {
            if let Some(existing) = tasks.iter_mut().find(|t| t.id == task.id) {
                *existing = task;
            }
        }
    }

    /// Remove a task
    pub fn remove_task(&mut self, calendar_id: &str, task_id: &str) {
        if let Some(tasks) = self.tasks.get_mut(calendar_id) {
            tasks.retain(|t| t.id != task_id);
        }
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Vec<&Task> {
        let mut result = Vec::new();

        for calendar in &self.calendars {
            if !calendar.visible {
                continue;
            }

            if let Some(tasks) = self.tasks.get(&calendar.id) {
                result.extend(tasks.iter());
            }
        }

        // Sort by due date, then by priority
        result.sort_by(|a, b| {
            match (a.due_date, b.due_date) {
                (Some(a_date), Some(b_date)) => a_date.cmp(&b_date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        result
    }

    /// Get incomplete tasks
    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.get_all_tasks()
            .into_iter()
            .filter(|t| !t.completed)
            .collect()
    }

    /// Get tasks for a specific date
    pub fn get_tasks_for_date(&self, date: NaiveDate) -> Vec<&Task> {
        self.get_all_tasks()
            .into_iter()
            .filter(|t| t.due_date == Some(date))
            .collect()
    }

    /// Toggle calendar visibility
    pub fn toggle_calendar_visibility(&mut self, id: &str) {
        if let Some(calendar) = self.calendars.iter_mut().find(|c| c.id == id) {
            calendar.visible = !calendar.visible;
        }
    }

    /// Get visible calendars
    pub fn visible_calendars(&self) -> Vec<&CalendarInfo> {
        self.calendars.iter().filter(|c| c.visible).collect()
    }
}

impl Default for CalendarStore {
    fn default() -> Self {
        Self::new()
    }
}
