// World Clock Tab - Display time in multiple timezones

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, FlowBox, SelectionMode};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage};
use chrono::{Local, Timelike, Datelike};
use chrono_tz::Tz;
use std::cell::RefCell;
use std::rc::Rc;

use crate::data::timezone::COMMON_TIMEZONES;
use crate::ui::analog_clock::AnalogClock;
use crate::ui::digital_clock::DigitalClock;

pub fn create_world_clock_tab() -> Box {
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Main clock display
    let clock_container = Box::new(Orientation::Vertical, 12);
    clock_container.set_halign(gtk4::Align::Center);
    clock_container.set_margin_bottom(24);

    // Digital clock (main)
    let digital_clock = DigitalClock::new();
    clock_container.append(digital_clock.widget());

    // Analog clock toggle
    let analog_clock = AnalogClock::new(150);
    analog_clock.widget().set_halign(gtk4::Align::Center);
    analog_clock.widget().set_margin_top(12);
    clock_container.append(analog_clock.widget());

    // Date display
    let date_label = Label::new(None);
    date_label.add_css_class("date-display");
    date_label.set_margin_top(8);
    clock_container.append(&date_label);

    // Local timezone
    let tz_label = Label::new(None);
    tz_label.add_css_class("timezone-label");
    clock_container.append(&tz_label);

    main_box.append(&clock_container);

    // World clocks section
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let page = PreferencesPage::new();

    // World clocks group
    let world_group = PreferencesGroup::builder()
        .title("Cidades")
        .description("Horarios ao redor do mundo")
        .build();

    // Add city button
    let add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Adicionar cidade")
        .build();
    add_button.add_css_class("flat");
    world_group.set_header_suffix(Some(&add_button));

    // Store for city clocks
    let city_clocks: Rc<RefCell<Vec<CityClockRow>>> = Rc::new(RefCell::new(Vec::new()));

    // Add some default cities
    let default_cities = [
        ("Nova York", "America/New_York"),
        ("Londres", "Europe/London"),
        ("Toquio", "Asia/Tokyo"),
        ("Sydney", "Australia/Sydney"),
    ];

    for (city, tz_str) in default_cities {
        let row = create_city_clock_row(city, tz_str);
        world_group.add(&row.widget);
        city_clocks.borrow_mut().push(row);
    }

    page.add(&world_group);
    scrolled.set_child(Some(&page));
    main_box.append(&scrolled);

    // Update clocks every second
    let digital_clock_clone = digital_clock.clone();
    let analog_clock_clone = analog_clock.clone();
    let date_label_clone = date_label.clone();
    let tz_label_clone = tz_label.clone();
    let city_clocks_clone = city_clocks.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        let now = Local::now();

        // Update main clocks
        digital_clock_clone.update();
        analog_clock_clone.update();

        // Update date
        let weekdays = ["Domingo", "Segunda", "Terca", "Quarta", "Quinta", "Sexta", "Sabado"];
        let months = ["Janeiro", "Fevereiro", "Marco", "Abril", "Maio", "Junho",
                      "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro"];
        let weekday = weekdays[now.weekday().num_days_from_sunday() as usize];
        let month = months[now.month0() as usize];
        date_label_clone.set_text(&format!("{}, {} de {} de {}", weekday, now.day(), month, now.year()));

        // Update timezone label
        let tz_name = Local::now().format("%Z").to_string();
        let offset = Local::now().format("%:z").to_string();
        tz_label_clone.set_text(&format!("{} (UTC{})", tz_name, offset));

        // Update city clocks
        for city_clock in city_clocks_clone.borrow().iter() {
            city_clock.update();
        }

        glib::ControlFlow::Continue
    });

    // Add city dialog
    let main_box_clone = main_box.clone();
    add_button.connect_clicked(move |_| {
        show_add_city_dialog(&main_box_clone);
    });

    main_box
}

struct CityClockRow {
    widget: ActionRow,
    time_label: Label,
    date_label: Label,
    offset_label: Label,
    timezone: Tz,
}

impl CityClockRow {
    fn update(&self) {
        let now = chrono::Utc::now().with_timezone(&self.timezone);
        self.time_label.set_text(&now.format("%H:%M").to_string());

        let weekdays_short = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sab"];
        let weekday = weekdays_short[now.weekday().num_days_from_sunday() as usize];
        self.date_label.set_text(&format!("{}, {}/{}", weekday, now.day(), now.month()));

        // Calculate offset from local
        let local_now = Local::now();
        let local_offset = local_now.offset().local_minus_utc() / 3600;
        let city_offset = now.offset().local_minus_utc() / 3600;
        let diff = city_offset - local_offset;

        let offset_text = if diff == 0 {
            "Mesmo horario".to_string()
        } else if diff > 0 {
            format!("+{}h", diff)
        } else {
            format!("{}h", diff)
        };
        self.offset_label.set_text(&offset_text);
    }
}

fn create_city_clock_row(city: &str, tz_str: &str) -> CityClockRow {
    let timezone: Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);

    let row = ActionRow::builder()
        .title(city)
        .build();
    row.add_css_class("world-clock-card");

    // Time display
    let time_box = Box::new(Orientation::Vertical, 2);
    time_box.set_valign(gtk4::Align::Center);

    let time_label = Label::new(Some("00:00"));
    time_label.add_css_class("city-time");
    time_box.append(&time_label);

    let date_label = Label::new(Some(""));
    date_label.add_css_class("city-date");
    time_box.append(&date_label);

    row.add_suffix(&time_box);

    // Offset badge
    let offset_label = Label::new(Some(""));
    offset_label.add_css_class("city-offset");
    offset_label.set_valign(gtk4::Align::Center);
    offset_label.set_margin_start(8);
    row.add_suffix(&offset_label);

    // Delete button
    let delete_btn = Button::from_icon_name("user-trash-symbolic");
    delete_btn.add_css_class("flat");
    delete_btn.set_valign(gtk4::Align::Center);
    row.add_suffix(&delete_btn);

    CityClockRow {
        widget: row,
        time_label,
        date_label,
        offset_label,
        timezone,
    }
}

fn show_add_city_dialog(parent: &Box) {
    let dialog = adw::Dialog::new();
    dialog.set_title("Adicionar Cidade");
    dialog.set_content_width(400);
    dialog.set_content_height(500);

    let toolbar_view = adw::ToolbarView::new();

    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let page = PreferencesPage::new();
    let group = PreferencesGroup::builder()
        .title("Selecione uma cidade")
        .build();

    for (city, region, _tz) in COMMON_TIMEZONES {
        let row = ActionRow::builder()
            .title(*city)
            .subtitle(*region)
            .activatable(true)
            .build();
        row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        group.add(&row);
    }

    page.add(&group);
    scrolled.set_child(Some(&page));
    toolbar_view.set_content(Some(&scrolled));

    dialog.set_child(Some(&toolbar_view));
    dialog.present(Some(parent));
}
