// Digital Clock Widget - Large digital time display

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct DigitalClock {
    container: Box,
    time_label: Label,
    seconds_label: Label,
    show_seconds: Rc<RefCell<bool>>,
    use_24h: Rc<RefCell<bool>>,
}

impl DigitalClock {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Horizontal, 0);
        container.set_halign(gtk4::Align::Center);

        let time_label = Label::new(Some("00:00"));
        time_label.add_css_class("clock-display");

        let seconds_label = Label::new(Some(":00"));
        seconds_label.add_css_class("clock-seconds");
        seconds_label.set_valign(gtk4::Align::End);
        seconds_label.set_margin_bottom(16);

        container.append(&time_label);
        container.append(&seconds_label);

        Self {
            container,
            time_label,
            seconds_label,
            show_seconds: Rc::new(RefCell::new(true)),
            use_24h: Rc::new(RefCell::new(true)),
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn update(&self) {
        let now = chrono::Local::now();
        let use_24h = *self.use_24h.borrow();
        let show_seconds = *self.show_seconds.borrow();

        if use_24h {
            self.time_label.set_text(&now.format("%H:%M").to_string());
        } else {
            self.time_label.set_text(&now.format("%I:%M").to_string());
        }

        if show_seconds {
            self.seconds_label.set_visible(true);
            self.seconds_label.set_text(&now.format(":%S").to_string());
        } else {
            self.seconds_label.set_visible(false);
        }
    }

    pub fn set_show_seconds(&self, show: bool) {
        *self.show_seconds.borrow_mut() = show;
        self.seconds_label.set_visible(show);
    }

    pub fn set_use_24h(&self, use_24h: bool) {
        *self.use_24h.borrow_mut() = use_24h;
        self.update();
    }
}

impl Default for DigitalClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Medium-sized digital clock for secondary displays
#[derive(Clone)]
pub struct MediumDigitalClock {
    container: Box,
    time_label: Label,
    period_label: Label, // AM/PM
    use_24h: Rc<RefCell<bool>>,
    timezone: Rc<RefCell<Option<String>>>,
}

impl MediumDigitalClock {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Horizontal, 4);
        container.set_halign(gtk4::Align::Center);

        let time_label = Label::new(Some("00:00"));
        time_label.add_css_class("clock-display-medium");

        let period_label = Label::new(Some(""));
        period_label.add_css_class("timezone-label");
        period_label.set_valign(gtk4::Align::End);
        period_label.set_margin_bottom(8);

        container.append(&time_label);
        container.append(&period_label);

        Self {
            container,
            time_label,
            period_label,
            use_24h: Rc::new(RefCell::new(true)),
            timezone: Rc::new(RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn update(&self) {
        let now = if let Some(tz_str) = self.timezone.borrow().as_ref() {
            let tz: chrono_tz::Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);
            chrono::Utc::now().with_timezone(&tz)
        } else {
            chrono::Local::now().with_timezone(&chrono_tz::UTC)
        };

        let use_24h = *self.use_24h.borrow();

        if use_24h {
            self.time_label.set_text(&now.format("%H:%M").to_string());
            self.period_label.set_visible(false);
        } else {
            self.time_label.set_text(&now.format("%I:%M").to_string());
            self.period_label.set_text(&now.format("%p").to_string());
            self.period_label.set_visible(true);
        }
    }

    pub fn set_timezone(&self, timezone: Option<&str>) {
        *self.timezone.borrow_mut() = timezone.map(|s| s.to_string());
        self.update();
    }

    pub fn set_use_24h(&self, use_24h: bool) {
        *self.use_24h.borrow_mut() = use_24h;
        self.update();
    }
}

impl Default for MediumDigitalClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Small digital clock for compact displays
#[derive(Clone)]
pub struct SmallDigitalClock {
    label: Label,
    use_24h: Rc<RefCell<bool>>,
    timezone: Rc<RefCell<Option<String>>>,
}

impl SmallDigitalClock {
    pub fn new() -> Self {
        let label = Label::new(Some("00:00"));
        label.add_css_class("clock-display-small");

        Self {
            label,
            use_24h: Rc::new(RefCell::new(true)),
            timezone: Rc::new(RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &Label {
        &self.label
    }

    pub fn update(&self) {
        let now = if let Some(tz_str) = self.timezone.borrow().as_ref() {
            let tz: chrono_tz::Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);
            chrono::Utc::now().with_timezone(&tz)
        } else {
            chrono::Local::now().with_timezone(&chrono_tz::UTC)
        };

        let use_24h = *self.use_24h.borrow();

        if use_24h {
            self.label.set_text(&now.format("%H:%M").to_string());
        } else {
            self.label.set_text(&now.format("%I:%M %p").to_string());
        }
    }

    pub fn set_timezone(&self, timezone: Option<&str>) {
        *self.timezone.borrow_mut() = timezone.map(|s| s.to_string());
        self.update();
    }

    pub fn set_use_24h(&self, use_24h: bool) {
        *self.use_24h.borrow_mut() = use_24h;
        self.update();
    }
}

impl Default for SmallDigitalClock {
    fn default() -> Self {
        Self::new()
    }
}
