// Time Picker Widget - Select hours, minutes, seconds

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TimePicker {
    container: Box,
    hours: Rc<RefCell<u32>>,
    minutes: Rc<RefCell<u32>>,
    seconds: Rc<RefCell<u32>>,
    hours_label: Label,
    minutes_label: Label,
    seconds_label: Label,
    show_seconds: bool,
}

impl TimePicker {
    pub fn new() -> Self {
        Self::with_options(true)
    }

    pub fn with_options(show_seconds: bool) -> Self {
        let container = Box::new(Orientation::Horizontal, 8);
        container.set_halign(gtk4::Align::Center);
        container.add_css_class("time-picker");

        let hours = Rc::new(RefCell::new(0u32));
        let minutes = Rc::new(RefCell::new(0u32));
        let seconds = Rc::new(RefCell::new(0u32));

        // Hours column
        let hours_box = Box::new(Orientation::Vertical, 4);
        let hours_up = Button::from_icon_name("go-up-symbolic");
        hours_up.add_css_class("time-picker-button");
        hours_up.add_css_class("circular");

        let hours_label = Label::new(Some("00"));
        hours_label.add_css_class("time-picker-value");

        let hours_down = Button::from_icon_name("go-down-symbolic");
        hours_down.add_css_class("time-picker-button");
        hours_down.add_css_class("circular");

        hours_box.append(&hours_up);
        hours_box.append(&hours_label);
        hours_box.append(&hours_down);

        // Minutes column
        let minutes_box = Box::new(Orientation::Vertical, 4);
        let minutes_up = Button::from_icon_name("go-up-symbolic");
        minutes_up.add_css_class("time-picker-button");
        minutes_up.add_css_class("circular");

        let minutes_label = Label::new(Some("00"));
        minutes_label.add_css_class("time-picker-value");

        let minutes_down = Button::from_icon_name("go-down-symbolic");
        minutes_down.add_css_class("time-picker-button");
        minutes_down.add_css_class("circular");

        minutes_box.append(&minutes_up);
        minutes_box.append(&minutes_label);
        minutes_box.append(&minutes_down);

        // Seconds column
        let seconds_box = Box::new(Orientation::Vertical, 4);
        let seconds_up = Button::from_icon_name("go-up-symbolic");
        seconds_up.add_css_class("time-picker-button");
        seconds_up.add_css_class("circular");

        let seconds_label = Label::new(Some("00"));
        seconds_label.add_css_class("time-picker-value");

        let seconds_down = Button::from_icon_name("go-down-symbolic");
        seconds_down.add_css_class("time-picker-button");
        seconds_down.add_css_class("circular");

        seconds_box.append(&seconds_up);
        seconds_box.append(&seconds_label);
        seconds_box.append(&seconds_down);

        // Separators
        let sep1 = Label::new(Some(":"));
        sep1.add_css_class("time-picker-separator");

        let sep2 = Label::new(Some(":"));
        sep2.add_css_class("time-picker-separator");

        // Assemble
        container.append(&hours_box);
        container.append(&sep1);
        container.append(&minutes_box);

        if show_seconds {
            container.append(&sep2);
            container.append(&seconds_box);
        }

        // Connect signals
        let hours_clone = hours.clone();
        let hours_label_clone = hours_label.clone();
        hours_up.connect_clicked(move |_| {
            let mut h = hours_clone.borrow_mut();
            *h = (*h + 1) % 24;
            hours_label_clone.set_text(&format!("{:02}", *h));
        });

        let hours_clone = hours.clone();
        let hours_label_clone = hours_label.clone();
        hours_down.connect_clicked(move |_| {
            let mut h = hours_clone.borrow_mut();
            *h = if *h == 0 { 23 } else { *h - 1 };
            hours_label_clone.set_text(&format!("{:02}", *h));
        });

        let minutes_clone = minutes.clone();
        let minutes_label_clone = minutes_label.clone();
        minutes_up.connect_clicked(move |_| {
            let mut m = minutes_clone.borrow_mut();
            *m = (*m + 1) % 60;
            minutes_label_clone.set_text(&format!("{:02}", *m));
        });

        let minutes_clone = minutes.clone();
        let minutes_label_clone = minutes_label.clone();
        minutes_down.connect_clicked(move |_| {
            let mut m = minutes_clone.borrow_mut();
            *m = if *m == 0 { 59 } else { *m - 1 };
            minutes_label_clone.set_text(&format!("{:02}", *m));
        });

        let seconds_clone = seconds.clone();
        let seconds_label_clone = seconds_label.clone();
        seconds_up.connect_clicked(move |_| {
            let mut s = seconds_clone.borrow_mut();
            *s = (*s + 1) % 60;
            seconds_label_clone.set_text(&format!("{:02}", *s));
        });

        let seconds_clone = seconds.clone();
        let seconds_label_clone = seconds_label.clone();
        seconds_down.connect_clicked(move |_| {
            let mut s = seconds_clone.borrow_mut();
            *s = if *s == 0 { 59 } else { *s - 1 };
            seconds_label_clone.set_text(&format!("{:02}", *s));
        });

        Self {
            container,
            hours,
            minutes,
            seconds,
            hours_label,
            minutes_label,
            seconds_label,
            show_seconds,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn get_time(&self) -> (u32, u32, u32) {
        (
            *self.hours.borrow(),
            *self.minutes.borrow(),
            *self.seconds.borrow(),
        )
    }

    pub fn set_time(&self, hours: u32, minutes: u32, seconds: u32) {
        *self.hours.borrow_mut() = hours % 24;
        *self.minutes.borrow_mut() = minutes % 60;
        *self.seconds.borrow_mut() = seconds % 60;

        self.hours_label.set_text(&format!("{:02}", hours % 24));
        self.minutes_label.set_text(&format!("{:02}", minutes % 60));
        self.seconds_label.set_text(&format!("{:02}", seconds % 60));
    }

    pub fn get_total_seconds(&self) -> u32 {
        let (h, m, s) = self.get_time();
        h * 3600 + m * 60 + s
    }

    pub fn set_from_seconds(&self, total_seconds: u32) {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        self.set_time(hours, minutes, seconds);
    }
}

impl Default for TimePicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple time picker for alarms (hours and minutes only)
#[derive(Clone)]
pub struct AlarmTimePicker {
    container: Box,
    hours: Rc<RefCell<u32>>,
    minutes: Rc<RefCell<u32>>,
    hours_label: Label,
    minutes_label: Label,
}

impl AlarmTimePicker {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Horizontal, 8);
        container.set_halign(gtk4::Align::Center);
        container.add_css_class("time-picker");

        let hours = Rc::new(RefCell::new(7u32));
        let minutes = Rc::new(RefCell::new(0u32));

        // Hours column
        let hours_box = Box::new(Orientation::Vertical, 4);
        let hours_up = Button::from_icon_name("go-up-symbolic");
        hours_up.add_css_class("time-picker-button");
        hours_up.add_css_class("circular");

        let hours_label = Label::new(Some("07"));
        hours_label.add_css_class("time-picker-value");

        let hours_down = Button::from_icon_name("go-down-symbolic");
        hours_down.add_css_class("time-picker-button");
        hours_down.add_css_class("circular");

        hours_box.append(&hours_up);
        hours_box.append(&hours_label);
        hours_box.append(&hours_down);

        // Separator
        let sep = Label::new(Some(":"));
        sep.add_css_class("time-picker-separator");

        // Minutes column
        let minutes_box = Box::new(Orientation::Vertical, 4);
        let minutes_up = Button::from_icon_name("go-up-symbolic");
        minutes_up.add_css_class("time-picker-button");
        minutes_up.add_css_class("circular");

        let minutes_label = Label::new(Some("00"));
        minutes_label.add_css_class("time-picker-value");

        let minutes_down = Button::from_icon_name("go-down-symbolic");
        minutes_down.add_css_class("time-picker-button");
        minutes_down.add_css_class("circular");

        minutes_box.append(&minutes_up);
        minutes_box.append(&minutes_label);
        minutes_box.append(&minutes_down);

        // Assemble
        container.append(&hours_box);
        container.append(&sep);
        container.append(&minutes_box);

        // Connect signals
        let hours_clone = hours.clone();
        let hours_label_clone = hours_label.clone();
        hours_up.connect_clicked(move |_| {
            let mut h = hours_clone.borrow_mut();
            *h = (*h + 1) % 24;
            hours_label_clone.set_text(&format!("{:02}", *h));
        });

        let hours_clone = hours.clone();
        let hours_label_clone = hours_label.clone();
        hours_down.connect_clicked(move |_| {
            let mut h = hours_clone.borrow_mut();
            *h = if *h == 0 { 23 } else { *h - 1 };
            hours_label_clone.set_text(&format!("{:02}", *h));
        });

        let minutes_clone = minutes.clone();
        let minutes_label_clone = minutes_label.clone();
        minutes_up.connect_clicked(move |_| {
            let mut m = minutes_clone.borrow_mut();
            *m = (*m + 5) % 60; // 5-minute increments
            minutes_label_clone.set_text(&format!("{:02}", *m));
        });

        let minutes_clone = minutes.clone();
        let minutes_label_clone = minutes_label.clone();
        minutes_down.connect_clicked(move |_| {
            let mut m = minutes_clone.borrow_mut();
            *m = if *m < 5 { 55 } else { *m - 5 }; // 5-minute decrements
            minutes_label_clone.set_text(&format!("{:02}", *m));
        });

        Self {
            container,
            hours,
            minutes,
            hours_label,
            minutes_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn get_time(&self) -> (u32, u32) {
        (*self.hours.borrow(), *self.minutes.borrow())
    }

    pub fn set_time(&self, hours: u32, minutes: u32) {
        *self.hours.borrow_mut() = hours % 24;
        *self.minutes.borrow_mut() = minutes % 60;

        self.hours_label.set_text(&format!("{:02}", hours % 24));
        self.minutes_label.set_text(&format!("{:02}", minutes % 60));
    }
}

impl Default for AlarmTimePicker {
    fn default() -> Self {
        Self::new()
    }
}
