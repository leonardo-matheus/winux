//! Agenda view - List of upcoming events

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, ScrolledWindow, Frame, Separator, ListBox, ListBoxRow};
use libadwaita as adw;
use adw::prelude::*;
use adw::ActionRow;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Local, NaiveDate, Datelike, Duration};

use crate::data::{CalendarStore, Event};

pub struct AgendaView {
    container: Box,
    store: Rc<RefCell<CalendarStore>>,
    days_to_show: i64,
}

impl AgendaView {
    pub fn new(store: Rc<RefCell<CalendarStore>>) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        let view = Self {
            container,
            store,
            days_to_show: 30,
        };

        view.build_view();
        view
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    fn build_view(&self) {
        // Header
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(20)
            .margin_end(20)
            .margin_top(16)
            .margin_bottom(8)
            .build();

        let title = Label::builder()
            .label("Proximos Eventos")
            .css_classes(vec!["title-2"])
            .halign(gtk4::Align::Start)
            .build();
        header.append(&title);

        let spacer = Box::builder()
            .hexpand(true)
            .build();
        header.append(&spacer);

        // Filter dropdown
        let filter_label = Label::builder()
            .label("Proximos 30 dias")
            .css_classes(vec!["dim-label"])
            .build();
        header.append(&filter_label);

        self.container.append(&header);

        // Scrollable list
        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        let list_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(20)
            .margin_end(20)
            .margin_bottom(20)
            .build();

        let today = Local::now().date_naive();
        let store = self.store.borrow();

        let mut current_date: Option<NaiveDate> = None;
        let mut has_events = false;

        for day_offset in 0..self.days_to_show {
            let date = today + Duration::days(day_offset);
            let events = store.get_events_for_date(date);

            if !events.is_empty() {
                has_events = true;

                // Date header
                let date_header = self.create_date_header(date, day_offset);
                list_box.append(&date_header);

                // Events for this day
                for event in events {
                    let event_row = self.create_event_row(&event);
                    list_box.append(&event_row);
                }
            }
        }

        if !has_events {
            let empty_state = self.create_empty_state();
            list_box.append(&empty_state);
        }

        scrolled.set_child(Some(&list_box));
        self.container.append(&scrolled);
    }

    fn create_date_header(&self, date: NaiveDate, day_offset: i64) -> Box {
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(if day_offset == 0 { 0 } else { 16 })
            .margin_bottom(8)
            .build();

        // Day number
        let day_box = Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(50)
            .build();

        let day_num = Label::builder()
            .label(&date.day().to_string())
            .css_classes(if day_offset == 0 {
                vec!["title-1", "accent"]
            } else {
                vec!["title-1"]
            })
            .build();
        day_box.append(&day_num);

        let weekday = get_short_weekday(date.weekday());
        let weekday_label = Label::builder()
            .label(weekday)
            .css_classes(vec!["caption", "dim-label"])
            .build();
        day_box.append(&weekday_label);

        header.append(&day_box);

        // Relative date text
        let relative = if day_offset == 0 {
            "Hoje".to_string()
        } else if day_offset == 1 {
            "Amanha".to_string()
        } else {
            let month = get_month_name(date.month());
            format!("{} de {}", date.day(), month)
        };

        let relative_label = Label::builder()
            .label(&relative)
            .css_classes(vec!["title-4"])
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Center)
            .build();
        header.append(&relative_label);

        header
    }

    fn create_event_row(&self, event: &Event) -> Frame {
        let frame = Frame::builder()
            .css_classes(vec!["card"])
            .build();

        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        // Color indicator
        let color_bar = Box::builder()
            .width_request(4)
            .build();

        let css_provider = gtk4::CssProvider::new();
        let css = format!(
            "box {{ background-color: {}; border-radius: 2px; }}",
            event.color
        );
        css_provider.load_from_string(&css);
        color_bar.style_context().add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        row.append(&color_bar);

        // Time column
        let time_box = Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(60)
            .valign(gtk4::Align::Center)
            .build();

        if event.all_day {
            let all_day_label = Label::builder()
                .label("Dia inteiro")
                .css_classes(vec!["caption", "dim-label"])
                .build();
            time_box.append(&all_day_label);
        } else if let Some(start) = event.start_time {
            let start_label = Label::builder()
                .label(&format!("{:02}:{:02}", start.hour(), start.minute()))
                .css_classes(vec!["title-4"])
                .build();
            time_box.append(&start_label);

            if let Some(end) = event.end_time {
                let end_label = Label::builder()
                    .label(&format!("{:02}:{:02}", end.hour(), end.minute()))
                    .css_classes(vec!["caption", "dim-label"])
                    .build();
                time_box.append(&end_label);
            }
        }

        row.append(&time_box);

        // Event info
        let info_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .build();

        let title = Label::builder()
            .label(&event.title)
            .css_classes(vec!["title-4"])
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        info_box.append(&title);

        if let Some(desc) = &event.description {
            let desc_label = Label::builder()
                .label(desc)
                .css_classes(vec!["dim-label"])
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            info_box.append(&desc_label);
        }

        if let Some(location) = &event.location {
            let loc_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(4)
                .build();

            let loc_icon = gtk4::Image::from_icon_name("mark-location-symbolic");
            loc_icon.add_css_class("dim-label");
            loc_box.append(&loc_icon);

            let loc_label = Label::builder()
                .label(location)
                .css_classes(vec!["caption", "dim-label"])
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            loc_box.append(&loc_label);

            info_box.append(&loc_box);
        }

        row.append(&info_box);

        // Calendar name
        let calendar_label = Label::builder()
            .label(&event.calendar_name)
            .css_classes(vec!["caption", "dim-label"])
            .valign(gtk4::Align::Center)
            .build();
        row.append(&calendar_label);

        // Action button
        let action_btn = gtk4::Button::from_icon_name("go-next-symbolic");
        action_btn.add_css_class("flat");
        action_btn.set_valign(gtk4::Align::Center);
        row.append(&action_btn);

        frame.set_child(Some(&row));
        frame
    }

    fn create_empty_state(&self) -> Box {
        let empty = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(80)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let icon = gtk4::Image::from_icon_name("x-office-calendar-symbolic");
        icon.set_pixel_size(64);
        icon.add_css_class("dim-label");
        empty.append(&icon);

        let title = Label::builder()
            .label("Nenhum evento proximo")
            .css_classes(vec!["title-2"])
            .build();
        empty.append(&title);

        let subtitle = Label::builder()
            .label("Clique no botao + para criar um novo evento")
            .css_classes(vec!["dim-label"])
            .build();
        empty.append(&subtitle);

        empty
    }

    pub fn refresh(&self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
        self.build_view();
    }
}

fn get_short_weekday(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "Seg",
        chrono::Weekday::Tue => "Ter",
        chrono::Weekday::Wed => "Qua",
        chrono::Weekday::Thu => "Qui",
        chrono::Weekday::Fri => "Sex",
        chrono::Weekday::Sat => "Sab",
        chrono::Weekday::Sun => "Dom",
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
