//! Local storage for calendar data

use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use std::path::PathBuf;

use crate::data::{CalendarInfo, CalendarSource, Event, Task, Priority, Recurrence, RecurrenceType, Reminder, ReminderUnit, ReminderMethod};

/// Local storage manager
pub struct LocalStorage {
    db_path: PathBuf,
}

impl LocalStorage {
    /// Create a new local storage instance
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Could not find data directory")?
            .join("winux-calendar");

        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("calendar.db");

        let storage = Self { db_path };
        storage.init_database()?;

        Ok(storage)
    }

    /// Get database connection
    fn connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).context("Failed to open database")
    }

    /// Initialize database schema
    fn init_database(&self) -> Result<()> {
        let conn = self.connection()?;

        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS calendars (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                visible INTEGER DEFAULT 1,
                description TEXT,
                is_default INTEGER DEFAULT 0,
                source TEXT NOT NULL,
                remote_url TEXT,
                last_sync TEXT,
                read_only INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                calendar_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_date TEXT NOT NULL,
                start_time TEXT,
                end_date TEXT NOT NULL,
                end_time TEXT,
                all_day INTEGER DEFAULT 1,
                color TEXT NOT NULL,
                category TEXT,
                notes TEXT,
                url TEXT,
                recurrence_type TEXT,
                recurrence_interval INTEGER,
                recurrence_end_date TEXT,
                recurrence_count INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (calendar_id) REFERENCES calendars(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS reminders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL,
                time_before INTEGER NOT NULL,
                unit TEXT NOT NULL,
                method TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                calendar_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                due_date TEXT,
                due_time TEXT,
                completed INTEGER DEFAULT 0,
                completed_at TEXT,
                priority TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (calendar_id) REFERENCES calendars(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS subtasks (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                title TEXT NOT NULL,
                completed INTEGER DEFAULT 0,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_events_calendar ON events(calendar_id);
            CREATE INDEX IF NOT EXISTS idx_events_date ON events(start_date);
            CREATE INDEX IF NOT EXISTS idx_tasks_calendar ON tasks(calendar_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
        "#)?;

        Ok(())
    }

    // Calendar operations

    /// Save a calendar
    pub fn save_calendar(&self, calendar: &CalendarInfo) -> Result<()> {
        let conn = self.connection()?;

        conn.execute(
            r#"INSERT OR REPLACE INTO calendars
               (id, name, color, visible, description, is_default, source, remote_url, last_sync, read_only)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                calendar.id,
                calendar.name,
                calendar.color,
                calendar.visible,
                calendar.description,
                calendar.is_default,
                format!("{:?}", calendar.source),
                calendar.remote_url,
                calendar.last_sync.map(|dt| dt.to_string()),
                calendar.read_only,
            ],
        )?;

        Ok(())
    }

    /// Load all calendars
    pub fn load_calendars(&self) -> Result<Vec<CalendarInfo>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, visible, description, is_default, source, remote_url, last_sync, read_only FROM calendars"
        )?;

        let calendars = stmt.query_map([], |row| {
            let source_str: String = row.get(6)?;
            let source = match source_str.as_str() {
                "Local" => CalendarSource::Local,
                "CalDAV" => CalendarSource::CalDAV,
                "Google" => CalendarSource::Google,
                "ICloud" => CalendarSource::ICloud,
                "Exchange" => CalendarSource::Exchange,
                "Subscribed" => CalendarSource::Subscribed,
                _ => CalendarSource::Local,
            };

            Ok(CalendarInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                visible: row.get(3)?,
                description: row.get(4)?,
                is_default: row.get(5)?,
                source,
                remote_url: row.get(7)?,
                last_sync: row.get::<_, Option<String>>(8)?
                    .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f").ok()),
                read_only: row.get(9)?,
            })
        })?;

        calendars.collect::<Result<Vec<_>, _>>().context("Failed to load calendars")
    }

    /// Delete a calendar
    pub fn delete_calendar(&self, id: &str) -> Result<()> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM calendars WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Event operations

    /// Save an event
    pub fn save_event(&self, event: &Event) -> Result<()> {
        let conn = self.connection()?;

        let (recurrence_type, recurrence_interval, recurrence_end_date, recurrence_count) =
            if let Some(ref r) = event.recurrence {
                (
                    Some(format!("{:?}", r.recurrence_type)),
                    Some(r.interval as i32),
                    r.end_date.map(|d| d.to_string()),
                    r.count.map(|c| c as i32),
                )
            } else {
                (None, None, None, None)
            };

        conn.execute(
            r#"INSERT OR REPLACE INTO events
               (id, calendar_id, title, description, location, start_date, start_time,
                end_date, end_time, all_day, color, category, notes, url,
                recurrence_type, recurrence_interval, recurrence_end_date, recurrence_count,
                created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                       ?15, ?16, ?17, ?18, ?19, ?20)"#,
            params![
                event.id,
                event.calendar_id,
                event.title,
                event.description,
                event.location,
                event.start_date.to_string(),
                event.start_time.map(|t| t.to_string()),
                event.end_date.to_string(),
                event.end_time.map(|t| t.to_string()),
                event.all_day,
                event.color,
                event.category,
                event.notes,
                event.url,
                recurrence_type,
                recurrence_interval,
                recurrence_end_date,
                recurrence_count,
                event.created_at.to_string(),
                event.updated_at.to_string(),
            ],
        )?;

        // Save reminders
        conn.execute("DELETE FROM reminders WHERE event_id = ?1", params![event.id])?;
        for reminder in &event.reminders {
            conn.execute(
                "INSERT INTO reminders (event_id, time_before, unit, method) VALUES (?1, ?2, ?3, ?4)",
                params![
                    event.id,
                    reminder.time_before,
                    format!("{:?}", reminder.unit),
                    format!("{:?}", reminder.method),
                ],
            )?;
        }

        Ok(())
    }

    /// Load events for a calendar
    pub fn load_events(&self, calendar_id: &str) -> Result<Vec<Event>> {
        let conn = self.connection()?;

        let mut stmt = conn.prepare(
            r#"SELECT id, calendar_id, title, description, location, start_date, start_time,
                      end_date, end_time, all_day, color, category, notes, url,
                      recurrence_type, recurrence_interval, recurrence_end_date, recurrence_count,
                      created_at, updated_at
               FROM events WHERE calendar_id = ?1"#
        )?;

        let events = stmt.query_map(params![calendar_id], |row| {
            let recurrence = row.get::<_, Option<String>>(14)?.map(|rt| {
                let recurrence_type = match rt.as_str() {
                    "Daily" => RecurrenceType::Daily,
                    "Weekly" => RecurrenceType::Weekly,
                    "Monthly" => RecurrenceType::Monthly,
                    "Yearly" => RecurrenceType::Yearly,
                    _ => RecurrenceType::Daily,
                };

                Recurrence {
                    recurrence_type,
                    interval: row.get::<_, Option<i32>>(15).unwrap_or(Some(1)).unwrap_or(1) as u32,
                    end_date: row.get::<_, Option<String>>(16).ok().flatten()
                        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                    count: row.get::<_, Option<i32>>(17).ok().flatten().map(|c| c as u32),
                    days_of_week: Vec::new(),
                    day_of_month: None,
                }
            });

            Ok(Event {
                id: row.get(0)?,
                calendar_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                location: row.get(4)?,
                start_date: chrono::NaiveDate::parse_from_str(&row.get::<_, String>(5)?, "%Y-%m-%d")
                    .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                start_time: row.get::<_, Option<String>>(6)?
                    .and_then(|s| chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S%.f").ok()),
                end_date: chrono::NaiveDate::parse_from_str(&row.get::<_, String>(7)?, "%Y-%m-%d")
                    .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                end_time: row.get::<_, Option<String>>(8)?
                    .and_then(|s| chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S%.f").ok()),
                all_day: row.get(9)?,
                color: row.get(10)?,
                calendar_name: String::new(), // Will be filled later
                recurrence,
                reminders: Vec::new(), // Will be loaded separately
                category: row.get(11)?,
                notes: row.get(12)?,
                url: row.get(13)?,
                created_at: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(18)?, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap_or_else(|_| chrono::Local::now().naive_local()),
                updated_at: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(19)?, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap_or_else(|_| chrono::Local::now().naive_local()),
            })
        })?;

        let mut result: Vec<Event> = events.collect::<Result<Vec<_>, _>>()
            .context("Failed to load events")?;

        // Load reminders for each event
        for event in &mut result {
            event.reminders = self.load_reminders(&event.id)?;
        }

        Ok(result)
    }

    /// Load reminders for an event
    fn load_reminders(&self, event_id: &str) -> Result<Vec<Reminder>> {
        let conn = self.connection()?;

        let mut stmt = conn.prepare(
            "SELECT time_before, unit, method FROM reminders WHERE event_id = ?1"
        )?;

        let reminders = stmt.query_map(params![event_id], |row| {
            let unit_str: String = row.get(1)?;
            let unit = match unit_str.as_str() {
                "Minutes" => ReminderUnit::Minutes,
                "Hours" => ReminderUnit::Hours,
                "Days" => ReminderUnit::Days,
                "Weeks" => ReminderUnit::Weeks,
                _ => ReminderUnit::Minutes,
            };

            let method_str: String = row.get(2)?;
            let method = match method_str.as_str() {
                "Notification" => ReminderMethod::Notification,
                "Email" => ReminderMethod::Email,
                "Sound" => ReminderMethod::Sound,
                _ => ReminderMethod::Notification,
            };

            Ok(Reminder {
                time_before: row.get(0)?,
                unit,
                method,
            })
        })?;

        reminders.collect::<Result<Vec<_>, _>>().context("Failed to load reminders")
    }

    /// Delete an event
    pub fn delete_event(&self, id: &str) -> Result<()> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM events WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Task operations

    /// Save a task
    pub fn save_task(&self, task: &Task) -> Result<()> {
        let conn = self.connection()?;

        conn.execute(
            r#"INSERT OR REPLACE INTO tasks
               (id, calendar_id, title, description, due_date, due_time,
                completed, completed_at, priority, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                task.id,
                task.calendar_id,
                task.title,
                task.description,
                task.due_date.map(|d| d.to_string()),
                task.due_time.map(|t| t.to_string()),
                task.completed,
                task.completed_at.map(|dt| dt.to_string()),
                format!("{:?}", task.priority),
                task.created_at.to_string(),
                task.updated_at.to_string(),
            ],
        )?;

        // Save subtasks
        conn.execute("DELETE FROM subtasks WHERE task_id = ?1", params![task.id])?;
        for subtask in &task.subtasks {
            conn.execute(
                "INSERT INTO subtasks (id, task_id, title, completed) VALUES (?1, ?2, ?3, ?4)",
                params![subtask.id, task.id, subtask.title, subtask.completed],
            )?;
        }

        // Save tags
        conn.execute("DELETE FROM tags WHERE task_id = ?1", params![task.id])?;
        for tag in &task.tags {
            conn.execute(
                "INSERT INTO tags (task_id, tag) VALUES (?1, ?2)",
                params![task.id, tag],
            )?;
        }

        Ok(())
    }

    /// Load tasks for a calendar
    pub fn load_tasks(&self, calendar_id: &str) -> Result<Vec<Task>> {
        let conn = self.connection()?;

        let mut stmt = conn.prepare(
            r#"SELECT id, calendar_id, title, description, due_date, due_time,
                      completed, completed_at, priority, created_at, updated_at
               FROM tasks WHERE calendar_id = ?1"#
        )?;

        let tasks = stmt.query_map(params![calendar_id], |row| {
            let priority_str: String = row.get(8)?;
            let priority = match priority_str.as_str() {
                "Low" => Priority::Low,
                "Medium" => Priority::Medium,
                "High" => Priority::High,
                "Urgent" => Priority::Urgent,
                _ => Priority::Medium,
            };

            Ok(Task {
                id: row.get(0)?,
                calendar_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                due_date: row.get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                due_time: row.get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S%.f").ok()),
                completed: row.get(6)?,
                completed_at: row.get::<_, Option<String>>(7)?
                    .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f").ok()),
                priority,
                created_at: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(9)?, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap_or_else(|_| chrono::Local::now().naive_local()),
                updated_at: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(10)?, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap_or_else(|_| chrono::Local::now().naive_local()),
                subtasks: Vec::new(),
                tags: Vec::new(),
            })
        })?;

        let mut result: Vec<Task> = tasks.collect::<Result<Vec<_>, _>>()
            .context("Failed to load tasks")?;

        // Load subtasks and tags for each task
        for task in &mut result {
            task.subtasks = self.load_subtasks(&task.id)?;
            task.tags = self.load_tags(&task.id)?;
        }

        Ok(result)
    }

    /// Load subtasks for a task
    fn load_subtasks(&self, task_id: &str) -> Result<Vec<crate::data::SubTask>> {
        let conn = self.connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, title, completed FROM subtasks WHERE task_id = ?1"
        )?;

        let subtasks = stmt.query_map(params![task_id], |row| {
            Ok(crate::data::SubTask {
                id: row.get(0)?,
                title: row.get(1)?,
                completed: row.get(2)?,
            })
        })?;

        subtasks.collect::<Result<Vec<_>, _>>().context("Failed to load subtasks")
    }

    /// Load tags for a task
    fn load_tags(&self, task_id: &str) -> Result<Vec<String>> {
        let conn = self.connection()?;

        let mut stmt = conn.prepare("SELECT tag FROM tags WHERE task_id = ?1")?;

        let tags = stmt.query_map(params![task_id], |row| {
            row.get(0)
        })?;

        tags.collect::<Result<Vec<_>, _>>().context("Failed to load tags")
    }

    /// Delete a task
    pub fn delete_task(&self, id: &str) -> Result<()> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(())
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create local storage")
    }
}
