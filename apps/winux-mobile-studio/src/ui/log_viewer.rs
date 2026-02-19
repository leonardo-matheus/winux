// Winux Mobile Studio - Log Viewer Widget
// Copyright (c) 2026 Winux OS Project
//
// A reusable log viewer widget for displaying build logs,
// device logs, and system output with filtering and search

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, Image, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, SearchEntry, TextView, TextBuffer, WrapMode,
    DropDown, StringList,
};
use libadwaita as adw;
use adw::prelude::*;
use std::collections::VecDeque;

const MAX_LOG_LINES: usize = 10000;

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
    pub pid: Option<u32>,
    pub tid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

impl LogLevel {
    pub fn from_char(c: char) -> Self {
        match c {
            'V' => LogLevel::Verbose,
            'D' => LogLevel::Debug,
            'I' => LogLevel::Info,
            'W' => LogLevel::Warning,
            'E' => LogLevel::Error,
            'F' => LogLevel::Fatal,
            _ => LogLevel::Info,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            LogLevel::Verbose => 'V',
            LogLevel::Debug => 'D',
            LogLevel::Info => 'I',
            LogLevel::Warning => 'W',
            LogLevel::Error => 'E',
            LogLevel::Fatal => 'F',
        }
    }

    pub fn css_class(&self) -> &str {
        match self {
            LogLevel::Verbose => "dim-label",
            LogLevel::Debug => "dim-label",
            LogLevel::Info => "",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Fatal => "error",
        }
    }
}

pub struct LogViewer {
    entries: VecDeque<LogEntry>,
    filter_level: LogLevel,
    filter_tag: Option<String>,
    filter_text: Option<String>,
    auto_scroll: bool,
    paused: bool,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_LOG_LINES),
            filter_level: LogLevel::Verbose,
            filter_tag: None,
            filter_text: None,
            auto_scroll: true,
            paused: false,
        }
    }

    pub fn add_entry(&mut self, entry: LogEntry) {
        if self.paused {
            return;
        }

        if self.entries.len() >= MAX_LOG_LINES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn set_filter_level(&mut self, level: LogLevel) {
        self.filter_level = level;
    }

    pub fn set_filter_tag(&mut self, tag: Option<String>) {
        self.filter_tag = tag;
    }

    pub fn set_filter_text(&mut self, text: Option<String>) {
        self.filter_text = text;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn filtered_entries(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                // Filter by level
                if entry.level < self.filter_level {
                    return false;
                }

                // Filter by tag
                if let Some(ref tag) = self.filter_tag {
                    if !entry.tag.contains(tag) {
                        return false;
                    }
                }

                // Filter by text
                if let Some(ref text) = self.filter_text {
                    let text_lower = text.to_lowercase();
                    if !entry.message.to_lowercase().contains(&text_lower)
                        && !entry.tag.to_lowercase().contains(&text_lower)
                    {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Parse a logcat line
    pub fn parse_logcat_line(line: &str) -> Option<LogEntry> {
        // Format: MM-DD HH:MM:SS.mmm PID TID LEVEL TAG: MESSAGE
        // Example: 02-18 10:35:21.123  1234  1234 I MyApp   : Application started

        let parts: Vec<&str> = line.splitn(7, ' ').collect();
        if parts.len() < 7 {
            return None;
        }

        let timestamp = format!("{} {}", parts[0], parts[1]);
        let pid = parts[2].trim().parse().ok();
        let tid = parts[3].trim().parse().ok();
        let level = parts[4].chars().next().map(LogLevel::from_char)?;

        let rest: Vec<&str> = parts[5..].join(" ").splitn(2, ": ").collect();
        let tag = rest.get(0).unwrap_or(&"").trim().to_string();
        let message = rest.get(1).unwrap_or(&"").to_string();

        Some(LogEntry {
            timestamp,
            level,
            tag,
            message,
            pid,
            tid,
        })
    }
}

pub fn create_log_viewer_widget() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    // Toolbar
    let toolbar = create_toolbar();
    container.append(&toolbar);

    // Log list
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Multiple)
        .css_classes(vec!["rich-list"])
        .build();

    // Add sample entries
    let sample_entries = get_sample_log_entries();
    for entry in sample_entries {
        let row = create_log_entry_row(&entry);
        list.append(&row);
    }

    scrolled.set_child(Some(&list));
    container.append(&scrolled);

    // Status bar
    let status_bar = create_status_bar(42, 1000);
    container.append(&status_bar);

    container
}

fn create_toolbar() -> Box {
    let toolbar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    // Search
    let search = SearchEntry::builder()
        .placeholder_text("Filtrar logs...")
        .width_request(200)
        .build();
    toolbar.append(&search);

    // Log level filter
    let level_list = StringList::new(&["Verbose", "Debug", "Info", "Warning", "Error"]);
    let level_filter = DropDown::new(Some(level_list), gtk4::Expression::NONE);
    level_filter.set_selected(2); // Info
    toolbar.append(&level_filter);

    // Tag filter
    let tag_entry = Entry::builder()
        .placeholder_text("Tag...")
        .width_request(100)
        .build();
    toolbar.append(&tag_entry);

    let spacer = Box::builder().hexpand(true).build();
    toolbar.append(&spacer);

    // Actions
    let pause_btn = Button::builder()
        .icon_name("media-playback-pause-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Pausar")
        .build();
    toolbar.append(&pause_btn);

    let scroll_btn = Button::builder()
        .icon_name("go-bottom-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Rolar para o fim")
        .build();
    toolbar.append(&scroll_btn);

    let clear_btn = Button::builder()
        .icon_name("edit-clear-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Limpar")
        .build();
    toolbar.append(&clear_btn);

    let save_btn = Button::builder()
        .icon_name("document-save-symbolic")
        .css_classes(vec!["flat"])
        .tooltip_text("Salvar logs")
        .build();
    toolbar.append(&save_btn);

    toolbar
}

fn create_log_entry_row(entry: &LogEntry) -> ListBoxRow {
    let row_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(2)
        .margin_bottom(2)
        .margin_start(5)
        .margin_end(5)
        .build();

    // Timestamp
    let timestamp_label = Label::builder()
        .label(&entry.timestamp)
        .css_classes(vec!["dim-label", "monospace"])
        .width_chars(18)
        .xalign(0.0)
        .build();
    row_box.append(&timestamp_label);

    // Level indicator
    let level_label = Label::builder()
        .label(&entry.level.to_char().to_string())
        .css_classes(vec!["monospace", entry.level.css_class()])
        .width_chars(1)
        .build();
    row_box.append(&level_label);

    // Tag
    let tag_label = Label::builder()
        .label(&entry.tag)
        .css_classes(vec!["monospace"])
        .width_chars(15)
        .xalign(0.0)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    row_box.append(&tag_label);

    // Message
    let message_label = Label::builder()
        .label(&entry.message)
        .css_classes(vec![entry.level.css_class()])
        .hexpand(true)
        .xalign(0.0)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    row_box.append(&message_label);

    ListBoxRow::builder()
        .child(&row_box)
        .build()
}

fn create_status_bar(showing: usize, total: usize) -> Box {
    let status = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_start(10)
        .margin_end(10)
        .margin_top(3)
        .margin_bottom(3)
        .build();

    let count_label = Label::builder()
        .label(&format!("Mostrando {} de {} entradas", showing, total))
        .css_classes(vec!["dim-label", "caption"])
        .build();
    status.append(&count_label);

    status
}

fn get_sample_log_entries() -> Vec<LogEntry> {
    vec![
        LogEntry {
            timestamp: "02-18 10:35:21.123".to_string(),
            level: LogLevel::Info,
            tag: "MyApp".to_string(),
            message: "Application started".to_string(),
            pid: Some(1234),
            tid: Some(1234),
        },
        LogEntry {
            timestamp: "02-18 10:35:21.145".to_string(),
            level: LogLevel::Debug,
            tag: "MyApp".to_string(),
            message: "Initializing database...".to_string(),
            pid: Some(1234),
            tid: Some(1234),
        },
        LogEntry {
            timestamp: "02-18 10:35:21.500".to_string(),
            level: LogLevel::Warning,
            tag: "MyApp".to_string(),
            message: "Cache miss for resource_123".to_string(),
            pid: Some(1234),
            tid: Some(1234),
        },
        LogEntry {
            timestamp: "02-18 10:35:22.100".to_string(),
            level: LogLevel::Error,
            tag: "MyApp".to_string(),
            message: "Failed to connect to server: timeout".to_string(),
            pid: Some(1234),
            tid: Some(1256),
        },
    ]
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new()
    }
}
