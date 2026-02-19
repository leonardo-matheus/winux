//! Mini calendar widget for sidebar

use gtk4::prelude::*;
use gtk4::{Box, Button, Grid, Label, Orientation, GestureClick};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, NaiveDate, Datelike, Weekday, Duration};

/// Mini calendar widget
pub struct MiniCalendar {
    widget: Box,
    grid: Grid,
    current_date: Rc<RefCell<NaiveDate>>,
    selected_date: Rc<RefCell<NaiveDate>>,
    on_date_selected: Rc<RefCell<Option<Box<dyn Fn(NaiveDate)>>>>,
}

impl MiniCalendar {
    /// Create a new mini calendar
    pub fn new() -> Self {
        let widget = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .build();

        let grid = Grid::builder()
            .row_homogeneous(true)
            .column_homogeneous(true)
            .row_spacing(2)
            .column_spacing(2)
            .build();

        let today = Local::now().date_naive();
        let current_date = Rc::new(RefCell::new(today));
        let selected_date = Rc::new(RefCell::new(today));
        let on_date_selected: Rc<RefCell<Option<Box<dyn Fn(NaiveDate)>>>> =
            Rc::new(RefCell::new(None));

        let calendar = Self {
            widget,
            grid,
            current_date,
            selected_date,
            on_date_selected,
        };

        calendar.build_ui();
        calendar
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.widget
    }

    /// Build the UI
    fn build_ui(&self) {
        // Navigation header
        let header = self.build_header();
        self.widget.append(&header);

        // Day of week headers
        let dow_header = self.build_dow_header();
        self.widget.append(&dow_header);

        // Calendar grid
        self.build_grid();
        self.widget.append(&self.grid);
    }

    /// Build the navigation header
    fn build_header(&self) -> Box {
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let prev_btn = Button::from_icon_name("go-previous-symbolic");
        prev_btn.add_css_class("flat");
        prev_btn.add_css_class("circular");

        let current_date = self.current_date.clone();
        let grid = self.grid.clone();
        let selected_date = self.selected_date.clone();
        let on_date_selected = self.on_date_selected.clone();

        prev_btn.connect_clicked(move |_| {
            let mut date = current_date.borrow_mut();
            if date.month() == 1 {
                *date = NaiveDate::from_ymd_opt(date.year() - 1, 12, 1).unwrap();
            } else {
                *date = NaiveDate::from_ymd_opt(date.year(), date.month() - 1, 1).unwrap();
            }
            drop(date);
            Self::rebuild_grid(&grid, &current_date, &selected_date, &on_date_selected);
        });
        header.append(&prev_btn);

        // Month/Year label
        let current = *self.current_date.borrow();
        let month_label = Label::builder()
            .label(&format!("{} {}",
                get_month_name(current.month()),
                current.year()))
            .css_classes(vec!["title-4"])
            .hexpand(true)
            .build();
        header.append(&month_label);

        let next_btn = Button::from_icon_name("go-next-symbolic");
        next_btn.add_css_class("flat");
        next_btn.add_css_class("circular");

        let current_date = self.current_date.clone();
        let grid = self.grid.clone();
        let selected_date = self.selected_date.clone();
        let on_date_selected = self.on_date_selected.clone();

        next_btn.connect_clicked(move |_| {
            let mut date = current_date.borrow_mut();
            if date.month() == 12 {
                *date = NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap();
            } else {
                *date = NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap();
            }
            drop(date);
            Self::rebuild_grid(&grid, &current_date, &selected_date, &on_date_selected);
        });
        header.append(&next_btn);

        header
    }

    /// Build day of week header
    fn build_dow_header(&self) -> Grid {
        let grid = Grid::builder()
            .row_homogeneous(true)
            .column_homogeneous(true)
            .build();

        let days = ["D", "S", "T", "Q", "Q", "S", "S"];
        for (i, day) in days.iter().enumerate() {
            let label = Label::builder()
                .label(*day)
                .css_classes(vec!["caption", "dim-label"])
                .build();
            grid.attach(&label, i as i32, 0, 1, 1);
        }

        grid
    }

    /// Build the calendar grid
    fn build_grid(&self) {
        Self::rebuild_grid(
            &self.grid,
            &self.current_date,
            &self.selected_date,
            &self.on_date_selected,
        );
    }

    /// Rebuild the calendar grid
    fn rebuild_grid(
        grid: &Grid,
        current_date: &Rc<RefCell<NaiveDate>>,
        selected_date: &Rc<RefCell<NaiveDate>>,
        on_date_selected: &Rc<RefCell<Option<Box<dyn Fn(NaiveDate)>>>>,
    ) {
        // Clear grid
        while let Some(child) = grid.first_child() {
            grid.remove(&child);
        }

        let current = *current_date.borrow();
        let selected = *selected_date.borrow();
        let today = Local::now().date_naive();

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

                let (day_num, is_current_month) = if cell_index < start_weekday {
                    // Previous month
                    let day = prev_month_days as i32 - (start_weekday as i32 - cell_index as i32 - 1);
                    (day, false)
                } else if day_counter <= days_in_month as i32 {
                    // Current month
                    let day = day_counter;
                    day_counter += 1;
                    (day, true)
                } else {
                    // Next month
                    let day = next_month_day;
                    next_month_day += 1;
                    (day, false)
                };

                let date = if is_current_month {
                    NaiveDate::from_ymd_opt(current.year(), current.month(), day_num as u32)
                } else {
                    None
                };

                let is_today = date.map(|d| d == today).unwrap_or(false);
                let is_selected = date.map(|d| d == selected).unwrap_or(false);

                let btn = Button::builder()
                    .label(&day_num.to_string())
                    .width_request(32)
                    .height_request(32)
                    .build();

                btn.add_css_class("flat");

                if !is_current_month {
                    btn.add_css_class("dim-label");
                }

                if is_today {
                    btn.add_css_class("accent");
                }

                if is_selected {
                    btn.add_css_class("suggested-action");
                }

                // Click handler
                if let Some(date) = date {
                    let selected_date_clone = selected_date.clone();
                    let grid_clone = grid.clone();
                    let current_date_clone = current_date.clone();
                    let on_date_selected_clone = on_date_selected.clone();

                    btn.connect_clicked(move |_| {
                        *selected_date_clone.borrow_mut() = date;

                        // Notify callback
                        if let Some(ref callback) = *on_date_selected_clone.borrow() {
                            callback(date);
                        }

                        // Rebuild grid to update selection
                        Self::rebuild_grid(
                            &grid_clone,
                            &current_date_clone,
                            &selected_date_clone,
                            &on_date_selected_clone,
                        );
                    });
                }

                grid.attach(&btn, col as i32, row as i32, 1, 1);
            }
        }
    }

    /// Set the callback for date selection
    pub fn connect_date_selected<F: Fn(NaiveDate) + 'static>(&self, callback: F) {
        *self.on_date_selected.borrow_mut() = Some(Box::new(callback));
    }

    /// Get the selected date
    pub fn selected_date(&self) -> NaiveDate {
        *self.selected_date.borrow()
    }

    /// Set the selected date
    pub fn set_selected_date(&self, date: NaiveDate) {
        *self.selected_date.borrow_mut() = date;

        // Update view to show the month of the selected date
        *self.current_date.borrow_mut() = NaiveDate::from_ymd_opt(
            date.year(),
            date.month(),
            1,
        ).unwrap();

        Self::rebuild_grid(
            &self.grid,
            &self.current_date,
            &self.selected_date,
            &self.on_date_selected,
        );
    }

    /// Go to today
    pub fn go_to_today(&self) {
        let today = Local::now().date_naive();
        self.set_selected_date(today);
    }
}

impl Default for MiniCalendar {
    fn default() -> Self {
        Self::new()
    }
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "Janeiro",
        2 => "Fevereiro",
        3 => "Marco",
        4 => "Abril",
        5 => "Maio",
        6 => "Junho",
        7 => "Julho",
        8 => "Agosto",
        9 => "Setembro",
        10 => "Outubro",
        11 => "Novembro",
        12 => "Dezembro",
        _ => "Desconhecido",
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
