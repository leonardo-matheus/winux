//! Week view - Timeline display of a week

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, ScrolledWindow, Grid, Frame, Separator, GestureClick};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, NaiveDate, NaiveTime, Datelike, Weekday, Duration};

use crate::data::{CalendarStore, Event};

pub struct WeekView {
    container: Box,
    store: Rc<RefCell<CalendarStore>>,
    current_date: Rc<RefCell<NaiveDate>>,
}

impl WeekView {
    pub fn new(store: Rc<RefCell<CalendarStore>>) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        let today = Local::now().date_naive();
        let current_date = Rc::new(RefCell::new(today));

        let view = Self {
            container,
            store,
            current_date,
        };

        view.build_view();
        view
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    fn build_view(&self) {
        // Header with day names and dates
        let header = self.build_header();
        self.container.append(&header);

        // Scrollable timeline
        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        let timeline = self.build_timeline();
        scrolled.set_child(Some(&timeline));

        self.container.append(&scrolled);
    }

    fn build_header(&self) -> Box {
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(60)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(8)
            .build();

        let current = *self.current_date.borrow();
        let today = Local::now().date_naive();

        // Get start of week (Sunday)
        let days_from_sunday = current.weekday().num_days_from_sunday();
        let week_start = current - Duration::days(days_from_sunday as i64);

        let day_names = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sab"];

        for i in 0..7 {
            let day = week_start + Duration::days(i);
            let is_today = day == today;

            let day_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(4)
                .hexpand(true)
                .halign(gtk4::Align::Center)
                .build();

            let name_label = Label::builder()
                .label(day_names[i as usize])
                .css_classes(vec!["caption", "dim-label"])
                .build();
            day_box.append(&name_label);

            let date_label = Label::builder()
                .label(&day.day().to_string())
                .css_classes(if is_today {
                    vec!["title-2", "accent"]
                } else {
                    vec!["title-2"]
                })
                .build();
            day_box.append(&date_label);

            header.append(&day_box);
        }

        header
    }

    fn build_timeline(&self) -> Box {
        let timeline = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();

        // Time labels column
        let time_column = Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(50)
            .margin_start(8)
            .build();

        for hour in 0..24 {
            let hour_box = Box::builder()
                .height_request(60)
                .valign(gtk4::Align::Start)
                .build();

            let label = Label::builder()
                .label(&format!("{:02}:00", hour))
                .css_classes(vec!["caption", "dim-label"])
                .valign(gtk4::Align::Start)
                .build();
            hour_box.append(&label);

            time_column.append(&hour_box);
        }

        timeline.append(&time_column);

        // Days columns
        let current = *self.current_date.borrow();
        let days_from_sunday = current.weekday().num_days_from_sunday();
        let week_start = current - Duration::days(days_from_sunday as i64);

        for i in 0..7 {
            let day = week_start + Duration::days(i);
            let day_column = self.build_day_column(day);
            timeline.append(&day_column);
        }

        timeline
    }

    fn build_day_column(&self, date: NaiveDate) -> Box {
        let column = Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .css_classes(vec!["view"])
            .build();

        // Hour slots
        for hour in 0..24 {
            let hour_slot = Box::builder()
                .height_request(60)
                .vexpand(false)
                .css_classes(vec!["card"])
                .build();

            // Add separator at top
            let separator = Separator::new(Orientation::Horizontal);
            separator.add_css_class("dim-label");
            hour_slot.append(&separator);

            // Get events for this hour
            let store = self.store.borrow();
            let events = store.get_events_for_date(date);

            for event in events.iter() {
                if let Some(start_time) = event.start_time {
                    if start_time.hour() == hour {
                        let event_widget = self.create_event_block(event);
                        hour_slot.append(&event_widget);
                    }
                }
            }

            // Click handler for creating events
            let gesture = GestureClick::new();
            let store_clone = self.store.clone();
            let date_clone = date;
            let hour_clone = hour;
            gesture.connect_released(move |_, _, _, _| {
                // Handle click - create new event at this time
            });
            hour_slot.add_controller(gesture);

            column.append(&hour_slot);
        }

        column
    }

    fn create_event_block(&self, event: &Event) -> Frame {
        let frame = Frame::builder()
            .margin_start(2)
            .margin_end(2)
            .margin_top(2)
            .css_classes(vec!["card"])
            .build();

        let content = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // Apply event color
        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "frame {{ background-color: alpha({}, 0.3); border-left: 3px solid {}; }}",
            event.color, event.color
        );
        css_provider.load_from_string(&css);
        frame.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let title = Label::builder()
            .label(&event.title)
            .css_classes(vec!["caption"])
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        content.append(&title);

        if let Some(location) = &event.location {
            let location_label = Label::builder()
                .label(location)
                .css_classes(vec!["caption", "dim-label"])
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            content.append(&location_label);
        }

        frame.set_child(Some(&content));
        frame
    }

    pub fn set_date(&self, date: NaiveDate) {
        *self.current_date.borrow_mut() = date;
        self.refresh();
    }

    pub fn refresh(&self) {
        // Clear and rebuild view
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
        self.build_view();
    }
}
