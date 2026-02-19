// Alarm data structures

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub id: u32,
    pub hour: u32,
    pub minute: u32,
    pub label: String,
    pub enabled: bool,
    pub repeat: RepeatDays,
    pub snooze_minutes: u32,
    pub sound: String,
}

impl Alarm {
    pub fn new() -> Self {
        Self {
            id: 0,
            hour: 7,
            minute: 0,
            label: "Alarme".to_string(),
            enabled: true,
            repeat: RepeatDays::none(),
            snooze_minutes: 10,
            sound: "default".to_string(),
        }
    }

    pub fn time_string(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }
}

impl Default for Alarm {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepeatDays {
    pub sunday: bool,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
}

impl RepeatDays {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn all() -> Self {
        Self {
            sunday: true,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: true,
        }
    }

    pub fn weekdays() -> Self {
        Self {
            sunday: false,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
        }
    }

    pub fn weekend() -> Self {
        Self {
            sunday: true,
            monday: false,
            tuesday: false,
            wednesday: false,
            thursday: false,
            friday: false,
            saturday: true,
        }
    }

    pub fn is_none(&self) -> bool {
        !self.sunday && !self.monday && !self.tuesday && !self.wednesday
            && !self.thursday && !self.friday && !self.saturday
    }

    pub fn is_all(&self) -> bool {
        self.sunday && self.monday && self.tuesday && self.wednesday
            && self.thursday && self.friday && self.saturday
    }

    pub fn is_weekdays(&self) -> bool {
        !self.sunday && self.monday && self.tuesday && self.wednesday
            && self.thursday && self.friday && !self.saturday
    }

    pub fn is_weekend(&self) -> bool {
        self.sunday && !self.monday && !self.tuesday && !self.wednesday
            && !self.thursday && !self.friday && self.saturday
    }

    pub fn should_trigger(&self, weekday: chrono::Weekday) -> bool {
        match weekday {
            chrono::Weekday::Sun => self.sunday,
            chrono::Weekday::Mon => self.monday,
            chrono::Weekday::Tue => self.tuesday,
            chrono::Weekday::Wed => self.wednesday,
            chrono::Weekday::Thu => self.thursday,
            chrono::Weekday::Fri => self.friday,
            chrono::Weekday::Sat => self.saturday,
        }
    }
}

impl fmt::Display for RepeatDays {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            write!(f, "Nunca")
        } else if self.is_all() {
            write!(f, "Todos os dias")
        } else if self.is_weekdays() {
            write!(f, "Dias uteis")
        } else if self.is_weekend() {
            write!(f, "Fim de semana")
        } else {
            let mut days = Vec::new();
            if self.sunday { days.push("Dom"); }
            if self.monday { days.push("Seg"); }
            if self.tuesday { days.push("Ter"); }
            if self.wednesday { days.push("Qua"); }
            if self.thursday { days.push("Qui"); }
            if self.friday { days.push("Sex"); }
            if self.saturday { days.push("Sab"); }
            write!(f, "{}", days.join(", "))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlarmStore {
    pub alarms: Vec<Alarm>,
    next_id: u32,
}

impl AlarmStore {
    pub fn new() -> Self {
        Self {
            alarms: Vec::new(),
            next_id: 1,
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(store) = serde_json::from_str(&content) {
                    return store;
                }
            }
        }
        Self::new()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn add(&mut self, mut alarm: Alarm) -> u32 {
        alarm.id = self.next_id;
        self.next_id += 1;
        self.alarms.push(alarm);
        self.alarms.last().map(|a| a.id).unwrap_or(0)
    }

    pub fn remove(&mut self, id: u32) {
        self.alarms.retain(|a| a.id != id);
    }

    pub fn get(&self, id: u32) -> Option<&Alarm> {
        self.alarms.iter().find(|a| a.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Alarm> {
        self.alarms.iter_mut().find(|a| a.id == id)
    }

    pub fn toggle(&mut self, id: u32) -> Option<bool> {
        if let Some(alarm) = self.get_mut(id) {
            alarm.enabled = !alarm.enabled;
            Some(alarm.enabled)
        } else {
            None
        }
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-clock")
            .join("alarms.json")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub id: u32,
    pub name: String,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl Timer {
    pub fn new(name: &str, hours: u32, minutes: u32, seconds: u32) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            hours,
            minutes,
            seconds,
        }
    }

    pub fn total_seconds(&self) -> u32 {
        self.hours * 3600 + self.minutes * 60 + self.seconds
    }

    pub fn from_seconds(name: &str, total_seconds: u32) -> Self {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        Self::new(name, hours, minutes, seconds)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimerStore {
    pub timers: Vec<Timer>,
    next_id: u32,
}

impl TimerStore {
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            next_id: 1,
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(store) = serde_json::from_str(&content) {
                    return store;
                }
            }
        }
        Self::new()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn add(&mut self, mut timer: Timer) -> u32 {
        timer.id = self.next_id;
        self.next_id += 1;
        self.timers.push(timer);
        self.timers.last().map(|t| t.id).unwrap_or(0)
    }

    pub fn remove(&mut self, id: u32) {
        self.timers.retain(|t| t.id != id);
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("winux-clock")
            .join("timers.json")
    }
}
