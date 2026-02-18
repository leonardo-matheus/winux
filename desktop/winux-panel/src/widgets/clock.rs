//! Clock Widget
//!
//! A clock widget that displays the current time and date,
//! with an optional calendar popover.

use chrono::{Local, Timelike, Datelike};
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Calendar, Label, MenuButton, Orientation, Popover,
};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::debug;

/// Clock display format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClockFormat {
    /// 12-hour format (e.g., 3:45 PM)
    TwelveHour,
    /// 24-hour format (e.g., 15:45)
    TwentyFourHour,
}

/// Clock widget configuration
#[derive(Debug, Clone)]
pub struct ClockConfig {
    /// Time format
    pub format: ClockFormat,
    /// Show seconds
    pub show_seconds: bool,
    /// Show date
    pub show_date: bool,
    /// Date format string (strftime format)
    pub date_format: String,
    /// Show calendar on click
    pub show_calendar: bool,
    /// Show week numbers in calendar
    pub show_week_numbers: bool,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: ClockFormat::TwentyFourHour,
            show_seconds: false,
            show_date: true,
            date_format: "%a, %b %d".to_string(),
            show_calendar: true,
            show_week_numbers: false,
        }
    }
}

/// Clock widget
pub struct ClockWidget {
    /// The menu button container (for calendar popover)
    button: MenuButton,
    /// Content box
    content: GtkBox,
    /// Time label
    time_label: Label,
    /// Date label
    date_label: Label,
    /// Calendar popover
    calendar_popover: Popover,
    /// Configuration
    config: Rc<RefCell<ClockConfig>>,
}

impl ClockWidget {
    /// Create a new clock widget with default configuration
    pub fn new() -> Self {
        Self::with_config(ClockConfig::default())
    }

    /// Create a new clock widget with custom configuration
    pub fn with_config(config: ClockConfig) -> Self {
        let config = Rc::new(RefCell::new(config));

        // Create content box
        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .valign(Align::Center)
            .build();

        content.add_css_class("clock-widget");

        // Time label
        let time_label = Label::builder()
            .halign(Align::Center)
            .build();

        time_label.add_css_class("clock-time");

        // Date label
        let date_label = Label::builder()
            .halign(Align::Center)
            .build();

        date_label.add_css_class("clock-date");

        content.append(&time_label);

        if config.borrow().show_date {
            content.append(&date_label);
        }

        // Create calendar popover
        let calendar_popover = Self::create_calendar_popover(&config.borrow());

        // Create menu button
        let button = MenuButton::builder()
            .tooltip_text("Date and time")
            .child(&content)
            .popover(&calendar_popover)
            .build();

        button.add_css_class("flat");

        let widget = Self {
            button,
            content,
            time_label,
            date_label,
            calendar_popover,
            config,
        };

        // Initial update
        widget.update_time();

        // Start update timer
        widget.start_timer();

        widget
    }

    /// Create the calendar popover
    fn create_calendar_popover(config: &ClockConfig) -> Popover {
        let popover = Popover::new();

        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        // Full date header
        let now = Local::now();
        let full_date = now.format("%A, %B %d, %Y").to_string();

        let date_header = Label::builder()
            .label(&full_date)
            .halign(Align::Start)
            .build();

        date_header.add_css_class("title-2");

        // Calendar widget
        let calendar = Calendar::builder()
            .show_week_numbers(config.show_week_numbers)
            .build();

        // Events list (placeholder)
        let events_label = Label::builder()
            .label("No events today")
            .halign(Align::Start)
            .margin_top(8)
            .build();

        events_label.add_css_class("dim-label");

        content.append(&date_header);
        content.append(&calendar);
        content.append(&events_label);

        popover.set_child(Some(&content));

        popover
    }

    /// Update the displayed time
    fn update_time(&self) {
        let now = Local::now();
        let config = self.config.borrow();

        // Format time
        let time_str = match (config.format, config.show_seconds) {
            (ClockFormat::TwelveHour, true) => now.format("%I:%M:%S %p").to_string(),
            (ClockFormat::TwelveHour, false) => now.format("%I:%M %p").to_string(),
            (ClockFormat::TwentyFourHour, true) => now.format("%H:%M:%S").to_string(),
            (ClockFormat::TwentyFourHour, false) => now.format("%H:%M").to_string(),
        };

        self.time_label.set_label(&time_str);

        // Format date
        if config.show_date {
            let date_str = now.format(&config.date_format).to_string();
            self.date_label.set_label(&date_str);
        }
    }

    /// Start the update timer
    fn start_timer(&self) {
        let time_label = self.time_label.clone();
        let date_label = self.date_label.clone();
        let config = Rc::clone(&self.config);

        // Calculate interval based on whether we show seconds
        let interval = if config.borrow().show_seconds { 1 } else { 30 };

        gtk4::glib::timeout_add_seconds_local(interval, move || {
            let now = Local::now();
            let cfg = config.borrow();

            // Format time
            let time_str = match (cfg.format, cfg.show_seconds) {
                (ClockFormat::TwelveHour, true) => now.format("%I:%M:%S %p").to_string(),
                (ClockFormat::TwelveHour, false) => now.format("%I:%M %p").to_string(),
                (ClockFormat::TwentyFourHour, true) => now.format("%H:%M:%S").to_string(),
                (ClockFormat::TwentyFourHour, false) => now.format("%H:%M").to_string(),
            };

            time_label.set_label(&time_str);

            // Format date
            if cfg.show_date {
                let date_str = now.format(&cfg.date_format).to_string();
                date_label.set_label(&date_str);
            }

            gtk4::glib::ControlFlow::Continue
        });
    }

    /// Set clock format
    pub fn set_format(&self, format: ClockFormat) {
        self.config.borrow_mut().format = format;
        self.update_time();
    }

    /// Toggle seconds display
    pub fn set_show_seconds(&self, show: bool) {
        self.config.borrow_mut().show_seconds = show;
        self.update_time();
    }

    /// Toggle date display
    pub fn set_show_date(&self, show: bool) {
        let mut config = self.config.borrow_mut();
        config.show_date = show;

        if show {
            if self.date_label.parent().is_none() {
                self.content.append(&self.date_label);
            }
        } else {
            self.content.remove(&self.date_label);
        }

        drop(config);
        self.update_time();
    }

    /// Set date format
    pub fn set_date_format(&self, format: &str) {
        self.config.borrow_mut().date_format = format.to_string();
        self.update_time();
    }

    /// Get the widget
    pub fn widget(&self) -> &MenuButton {
        &self.button
    }
}

impl Default for ClockWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_config_default() {
        let config = ClockConfig::default();
        assert_eq!(config.format, ClockFormat::TwentyFourHour);
        assert!(!config.show_seconds);
        assert!(config.show_date);
    }
}
