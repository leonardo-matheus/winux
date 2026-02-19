//! Month view - Grid display of a full month

use gtk4::prelude::*;
use gtk4::{Box, Button, Grid, Label, Orientation, ScrolledWindow, Frame, GestureClick};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, NaiveDate, Datelike, Weekday, Duration};

use crate::data::{CalendarStore, Event};

pub struct MonthView {
    container: Box,
    grid: Grid,
    store: Rc<RefCell<CalendarStore>>,
    current_date: Rc<RefCell<NaiveDate>>,
}

impl MonthView {
    pub fn new(store: Rc<RefCell<CalendarStore>>) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        let grid = Grid::builder()
            .row_homogeneous(true)
            .column_homogeneous(true)
            .row_spacing(1)
            .column_spacing(1)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .vexpand(true)
            .hexpand(true)
            .build();

        let today = Local::now().date_naive();
        let current_date = Rc::new(RefCell::new(today));

        let view = Self {
            container,
            grid,
            store,
            current_date,
        };

        view.build_header();
        view.build_month_grid();
        view
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    fn build_header(&self) {
        // Day headers
        let header_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .build();

        let days = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sab"];
        for day in days {
            let label = Label::builder()
                .label(day)
                .css_classes(vec!["dim-label", "caption"])
                .hexpand(true)
                .build();
            header_box.append(&label);
        }

        self.container.append(&header_box);
    }

    fn build_month_grid(&self) {
        let today = Local::now().date_naive();
        let current = *self.current_date.borrow();

        // Get first day of month
        let first_day = NaiveDate::from_ymd_opt(current.year(), current.month(), 1).unwrap();
        let days_in_month = get_days_in_month(current.year(), current.month());

        // Get the weekday of the first day (0 = Sunday)
        let start_weekday = first_day.weekday().num_days_from_sunday();

        // Previous month days
        let prev_month_days = if current.month() == 1 {
            get_days_in_month(current.year() - 1, 12)
        } else {
            get_days_in_month(current.year(), current.month() - 1)
        };

        let mut day_counter = 1i32;
        let mut next_month_day = 1;

        // Create 6 rows of 7 days
        for row in 0..6 {
            for col in 0..7 {
                let cell_index = row * 7 + col;
                let cell = self.create_day_cell(
                    cell_index,
                    start_weekday,
                    days_in_month,
                    prev_month_days,
                    &mut day_counter,
                    &mut next_month_day,
                    current,
                    today,
                );
                self.grid.attach(&cell, col as i32, row as i32, 1, 1);
            }
        }

        self.container.append(&self.grid);
    }

    fn create_day_cell(
        &self,
        cell_index: u32,
        start_weekday: u32,
        days_in_month: u32,
        prev_month_days: u32,
        day_counter: &mut i32,
        next_month_day: &mut i32,
        current: NaiveDate,
        today: NaiveDate,
    ) -> Frame {
        let frame = Frame::builder()
            .css_classes(vec!["card"])
            .build();

        let cell_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let (day_num, is_current_month) = if cell_index < start_weekday {
            // Previous month
            let day = prev_month_days as i32 - (start_weekday as i32 - cell_index as i32 - 1);
            (day, false)
        } else if *day_counter <= days_in_month as i32 {
            // Current month
            let day = *day_counter;
            *day_counter += 1;
            (day, true)
        } else {
            // Next month
            let day = *next_month_day;
            *next_month_day += 1;
            (day, false)
        };

        // Day number label
        let is_today = is_current_month
            && day_num == today.day() as i32
            && current.month() == today.month()
            && current.year() == today.year();

        let day_label = Label::builder()
            .label(&day_num.to_string())
            .halign(gtk4::Align::End)
            .css_classes(if is_today {
                vec!["accent", "title-4"]
            } else if is_current_month {
                vec!["title-4"]
            } else {
                vec!["dim-label"]
            })
            .build();

        cell_box.append(&day_label);

        // Events container
        let events_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .vexpand(true)
            .build();

        // Get events for this day
        if is_current_month {
            if let Some(date) = NaiveDate::from_ymd_opt(current.year(), current.month(), day_num as u32) {
                let store = self.store.borrow();
                let events = store.get_events_for_date(date);

                for (i, event) in events.iter().take(3).enumerate() {
                    let event_pill = self.create_event_pill(event);
                    events_box.append(&event_pill);
                }

                if events.len() > 3 {
                    let more_label = Label::builder()
                        .label(&format!("+{} mais", events.len() - 3))
                        .css_classes(vec!["caption", "dim-label"])
                        .halign(gtk4::Align::Start)
                        .build();
                    events_box.append(&more_label);
                }
            }
        }

        cell_box.append(&events_box);
        frame.set_child(Some(&cell_box));

        // Click handler
        let gesture = GestureClick::new();
        let store = self.store.clone();
        gesture.connect_released(move |_, _, _, _| {
            // Handle day click - show day detail or create event
        });
        frame.add_controller(gesture);

        frame
    }

    fn create_event_pill(&self, event: &Event) -> Box {
        let pill = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .css_classes(vec!["card"])
            .build();

        // Color dot
        let color_dot = Box::builder()
            .width_request(8)
            .height_request(8)
            .valign(gtk4::Align::Center)
            .build();

        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "box {{ background-color: {}; border-radius: 4px; }}",
            event.color
        );
        css_provider.load_from_string(&css);
        color_dot.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        pill.append(&color_dot);

        // Event title
        let title = Label::builder()
            .label(&event.title)
            .css_classes(vec!["caption"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .halign(gtk4::Align::Start)
            .build();
        pill.append(&title);

        pill
    }

    pub fn set_date(&self, date: NaiveDate) {
        *self.current_date.borrow_mut() = date;
        self.refresh();
    }

    pub fn refresh(&self) {
        // Clear and rebuild grid
        while let Some(child) = self.grid.first_child() {
            self.grid.remove(&child);
        }
        self.build_month_grid();
    }
}

fn get_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
