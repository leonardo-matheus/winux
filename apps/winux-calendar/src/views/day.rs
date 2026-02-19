//! Day view - Detailed timeline display of a single day

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, ScrolledWindow, Frame, Separator, GestureClick};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, NaiveDate, NaiveTime, Datelike, Weekday};

use crate::data::{CalendarStore, Event};

pub struct DayView {
    container: Box,
    store: Rc<RefCell<CalendarStore>>,
    current_date: Rc<RefCell<NaiveDate>>,
}

impl DayView {
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
        let current = *self.current_date.borrow();
        let today = Local::now().date_naive();

        // Header with date
        let header = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_top(16)
            .margin_bottom(8)
            .build();

        let weekday = get_weekday_name(current.weekday());
        let month = get_month_name(current.month());

        let date_label = Label::builder()
            .label(&format!("{}, {} de {} de {}",
                weekday,
                current.day(),
                month,
                current.year()))
            .css_classes(if current == today {
                vec!["title-2", "accent"]
            } else {
                vec!["title-2"]
            })
            .halign(gtk4::Align::Start)
            .build();
        header.append(&date_label);

        self.container.append(&header);

        // All-day events section
        let all_day_section = self.build_all_day_section(current);
        self.container.append(&all_day_section);

        // Scrollable timeline
        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        let timeline = self.build_timeline(current);
        scrolled.set_child(Some(&timeline));

        self.container.append(&scrolled);
    }

    fn build_all_day_section(&self, date: NaiveDate) -> Box {
        let section = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_bottom(8)
            .build();

        let label = Label::builder()
            .label("Dia inteiro")
            .css_classes(vec!["caption", "dim-label"])
            .halign(gtk4::Align::Start)
            .margin_bottom(4)
            .build();
        section.append(&label);

        // Get all-day events
        let store = self.store.borrow();
        let events = store.get_events_for_date(date);
        let all_day_events: Vec<_> = events.iter().filter(|e| e.all_day).collect();

        if all_day_events.is_empty() {
            let empty_label = Label::builder()
                .label("Nenhum evento de dia inteiro")
                .css_classes(vec!["dim-label"])
                .halign(gtk4::Align::Start)
                .build();
            section.append(&empty_label);
        } else {
            let events_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .build();

            for event in all_day_events {
                let pill = self.create_all_day_pill(event);
                events_box.append(&pill);
            }

            section.append(&events_box);
        }

        let separator = Separator::new(Orientation::Horizontal);
        separator.set_margin_top(8);
        section.append(&separator);

        section
    }

    fn create_all_day_pill(&self, event: &Event) -> Frame {
        let frame = Frame::builder()
            .css_classes(vec!["card"])
            .build();

        let content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Apply color
        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "frame {{ background-color: alpha({}, 0.3); border-left: 4px solid {}; }}",
            event.color, event.color
        );
        css_provider.load_from_string(&css);
        frame.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let title = Label::builder()
            .label(&event.title)
            .build();
        content.append(&title);

        frame.set_child(Some(&content));
        frame
    }

    fn build_timeline(&self, date: NaiveDate) -> Box {
        let timeline = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .build();

        let store = self.store.borrow();
        let events = store.get_events_for_date(date);

        for hour in 0..24 {
            let hour_row = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(16)
                .build();

            // Time label
            let time_label = Label::builder()
                .label(&format!("{:02}:00", hour))
                .css_classes(vec!["caption", "dim-label"])
                .width_request(50)
                .halign(gtk4::Align::End)
                .valign(gtk4::Align::Start)
                .build();
            hour_row.append(&time_label);

            // Hour slot
            let hour_slot = Box::builder()
                .orientation(Orientation::Vertical)
                .height_request(80)
                .hexpand(true)
                .css_classes(vec!["view"])
                .build();

            // Separator line
            let separator = Separator::new(Orientation::Horizontal);
            hour_slot.append(&separator);

            // Events in this hour
            let hour_events: Vec<_> = events.iter()
                .filter(|e| {
                    !e.all_day && e.start_time.map(|t| t.hour() == hour).unwrap_or(false)
                })
                .collect();

            for event in hour_events {
                let event_card = self.create_event_card(event);
                hour_slot.append(&event_card);
            }

            // Click handler
            let gesture = GestureClick::new();
            gesture.connect_released(move |_, _, _, _| {
                // Create new event at this time
            });
            hour_slot.add_controller(gesture);

            hour_row.append(&hour_slot);
            timeline.append(&hour_row);
        }

        timeline
    }

    fn create_event_card(&self, event: &Event) -> Frame {
        let frame = Frame::builder()
            .margin_top(4)
            .margin_bottom(4)
            .css_classes(vec!["card"])
            .build();

        let content = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Apply event color
        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "frame {{ background-color: alpha({}, 0.2); border-left: 4px solid {}; }}",
            event.color, event.color
        );
        css_provider.load_from_string(&css);
        frame.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Time range
        let time_str = if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
            format!("{:02}:{:02} - {:02}:{:02}",
                start.hour(), start.minute(),
                end.hour(), end.minute())
        } else if let Some(start) = event.start_time {
            format!("{:02}:{:02}", start.hour(), start.minute())
        } else {
            String::new()
        };

        let time_label = Label::builder()
            .label(&time_str)
            .css_classes(vec!["caption", "dim-label"])
            .halign(gtk4::Align::Start)
            .build();
        content.append(&time_label);

        // Title
        let title = Label::builder()
            .label(&event.title)
            .css_classes(vec!["title-4"])
            .halign(gtk4::Align::Start)
            .build();
        content.append(&title);

        // Description
        if let Some(desc) = &event.description {
            let desc_label = Label::builder()
                .label(desc)
                .css_classes(vec!["dim-label"])
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .lines(2)
                .wrap(true)
                .build();
            content.append(&desc_label);
        }

        // Location
        if let Some(location) = &event.location {
            let location_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(4)
                .build();

            let location_icon = gtk4::Image::from_icon_name("mark-location-symbolic");
            location_icon.add_css_class("dim-label");
            location_box.append(&location_icon);

            let location_label = Label::builder()
                .label(location)
                .css_classes(vec!["caption", "dim-label"])
                .build();
            location_box.append(&location_label);

            content.append(&location_box);
        }

        frame.set_child(Some(&content));
        frame
    }

    pub fn set_date(&self, date: NaiveDate) {
        *self.current_date.borrow_mut() = date;
        self.refresh();
    }

    pub fn refresh(&self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
        self.build_view();
    }
}

fn get_weekday_name(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Segunda-feira",
        Weekday::Tue => "Terca-feira",
        Weekday::Wed => "Quarta-feira",
        Weekday::Thu => "Quinta-feira",
        Weekday::Fri => "Sexta-feira",
        Weekday::Sat => "Sabado",
        Weekday::Sun => "Domingo",
    }
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "janeiro",
        2 => "fevereiro",
        3 => "marco",
        4 => "abril",
        5 => "maio",
        6 => "junho",
        7 => "julho",
        8 => "agosto",
        9 => "setembro",
        10 => "outubro",
        11 => "novembro",
        12 => "dezembro",
        _ => "desconhecido",
    }
}
