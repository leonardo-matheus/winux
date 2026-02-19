// Stopwatch Tab - Precise timing with laps

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, ListBox, SelectionMode};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct StopwatchState {
    running: bool,
    start_time: Option<Instant>,
    elapsed: Duration,
    laps: Vec<Duration>,
}

impl Default for StopwatchState {
    fn default() -> Self {
        Self {
            running: false,
            start_time: None,
            elapsed: Duration::ZERO,
            laps: Vec::new(),
        }
    }
}

pub fn create_stopwatch_tab() -> Box {
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_margin_top(40);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Stopwatch display
    let display_box = Box::new(Orientation::Vertical, 8);
    display_box.set_halign(gtk4::Align::Center);
    display_box.set_margin_bottom(40);

    // Main time display
    let time_box = Box::new(Orientation::Horizontal, 0);
    time_box.set_halign(gtk4::Align::Center);

    let time_label = Label::new(Some("00:00"));
    time_label.add_css_class("stopwatch-display");

    let ms_label = Label::new(Some(".00"));
    ms_label.add_css_class("clock-milliseconds");
    ms_label.set_valign(gtk4::Align::End);
    ms_label.set_margin_bottom(12);

    time_box.append(&time_label);
    time_box.append(&ms_label);

    display_box.append(&time_box);

    // Lap indicator
    let lap_indicator = Label::new(Some(""));
    lap_indicator.add_css_class("timezone-label");
    lap_indicator.set_margin_top(8);
    display_box.append(&lap_indicator);

    main_box.append(&display_box);

    // Control buttons
    let controls_box = Box::new(Orientation::Horizontal, 20);
    controls_box.set_halign(gtk4::Align::Center);
    controls_box.set_margin_bottom(30);

    // Reset/Lap button
    let reset_btn = Button::new();
    reset_btn.set_size_request(70, 70);
    reset_btn.add_css_class("circular");
    reset_btn.add_css_class("control-button");
    reset_btn.add_css_class("reset");
    let reset_label = Label::new(Some("Zerar"));
    reset_btn.set_child(Some(&reset_label));
    reset_btn.set_sensitive(false);

    // Start/Stop button
    let start_btn = Button::new();
    start_btn.set_size_request(70, 70);
    start_btn.add_css_class("circular");
    start_btn.add_css_class("control-button");
    start_btn.add_css_class("start");
    let start_label = Label::new(Some("Iniciar"));
    start_btn.set_child(Some(&start_label));

    controls_box.append(&reset_btn);
    controls_box.append(&start_btn);
    main_box.append(&controls_box);

    // Laps section
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let page = PreferencesPage::new();

    let laps_group = PreferencesGroup::builder()
        .title("Voltas")
        .build();

    // Laps list
    let laps_list = ListBox::new();
    laps_list.set_selection_mode(SelectionMode::None);
    laps_list.add_css_class("boxed-list");

    // Placeholder when no laps
    let placeholder = Label::new(Some("Nenhuma volta registrada"));
    placeholder.add_css_class("dim-label");
    placeholder.set_margin_top(20);
    placeholder.set_margin_bottom(20);
    laps_list.set_placeholder(Some(&placeholder));

    laps_group.add(&laps_list);
    page.add(&laps_group);
    scrolled.set_child(Some(&page));
    main_box.append(&scrolled);

    // State
    let state: Rc<RefCell<StopwatchState>> = Rc::new(RefCell::new(StopwatchState::default()));

    // Update display
    let time_label_clone = time_label.clone();
    let ms_label_clone = ms_label.clone();
    let lap_indicator_clone = lap_indicator.clone();
    let state_clone = state.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
        let state = state_clone.borrow();
        let elapsed = if state.running {
            if let Some(start) = state.start_time {
                state.elapsed + start.elapsed()
            } else {
                state.elapsed
            }
        } else {
            state.elapsed
        };

        let total_ms = elapsed.as_millis();
        let minutes = (total_ms / 60000) % 60;
        let seconds = (total_ms / 1000) % 60;
        let centiseconds = (total_ms % 1000) / 10;

        if total_ms >= 3600000 {
            let hours = total_ms / 3600000;
            time_label_clone.set_text(&format!("{:02}:{:02}:{:02}", hours, minutes, seconds));
        } else {
            time_label_clone.set_text(&format!("{:02}:{:02}", minutes, seconds));
        }
        ms_label_clone.set_text(&format!(".{:02}", centiseconds));

        // Update lap indicator
        if !state.laps.is_empty() {
            lap_indicator_clone.set_text(&format!("Volta {}", state.laps.len() + 1));
        } else if state.running {
            lap_indicator_clone.set_text("Volta 1");
        } else {
            lap_indicator_clone.set_text("");
        }

        glib::ControlFlow::Continue
    });

    // Start/Stop button handler
    let state_clone = state.clone();
    let start_btn_clone = start_btn.clone();
    let start_label_clone = start_label.clone();
    let reset_btn_clone = reset_btn.clone();
    let reset_label_clone = reset_label.clone();

    start_btn.connect_clicked(move |btn| {
        let mut state = state_clone.borrow_mut();

        if state.running {
            // Stop
            if let Some(start) = state.start_time.take() {
                state.elapsed += start.elapsed();
            }
            state.running = false;
            start_label_clone.set_text("Continuar");
            btn.remove_css_class("stop");
            btn.add_css_class("start");
            reset_label_clone.set_text("Zerar");
        } else {
            // Start
            state.running = true;
            state.start_time = Some(Instant::now());
            start_label_clone.set_text("Parar");
            btn.remove_css_class("start");
            btn.add_css_class("stop");
            reset_btn_clone.set_sensitive(true);
            reset_label_clone.set_text("Volta");
        }
    });

    // Reset/Lap button handler
    let state_clone = state.clone();
    let laps_list_clone = laps_list.clone();
    let reset_btn_weak = reset_btn.downgrade();
    let start_btn_weak = start_btn.downgrade();
    let start_label_weak = start_label.downgrade();
    let reset_label_weak = reset_label.downgrade();

    reset_btn.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();

        if state.running {
            // Record lap
            if let Some(start) = state.start_time {
                let total_elapsed = state.elapsed + start.elapsed();

                // Get lap time (difference from last lap)
                let lap_time = if let Some(last) = state.laps.last() {
                    total_elapsed - *last
                } else {
                    total_elapsed
                };

                let lap_number = state.laps.len() + 1;
                state.laps.push(total_elapsed);

                // Add lap to list
                let row = create_lap_row(lap_number, lap_time, total_elapsed);
                laps_list_clone.prepend(&row);
            }
        } else {
            // Reset
            state.elapsed = Duration::ZERO;
            state.start_time = None;
            state.laps.clear();

            // Clear laps list
            while let Some(row) = laps_list_clone.first_child() {
                laps_list_clone.remove(&row);
            }

            if let Some(reset_btn) = reset_btn_weak.upgrade() {
                reset_btn.set_sensitive(false);
            }
            if let Some(start_btn) = start_btn_weak.upgrade() {
                start_btn.remove_css_class("stop");
                start_btn.add_css_class("start");
            }
            if let Some(start_label) = start_label_weak.upgrade() {
                start_label.set_text("Iniciar");
            }
            if let Some(reset_label) = reset_label_weak.upgrade() {
                reset_label.set_text("Zerar");
            }
        }
    });

    main_box
}

fn create_lap_row(lap_number: usize, lap_time: Duration, total_time: Duration) -> ActionRow {
    let row = ActionRow::builder()
        .title(&format!("Volta {}", lap_number))
        .build();
    row.add_css_class("lap-time");

    // Lap time
    let lap_ms = lap_time.as_millis();
    let lap_min = (lap_ms / 60000) % 60;
    let lap_sec = (lap_ms / 1000) % 60;
    let lap_cs = (lap_ms % 1000) / 10;
    let lap_str = format!("{:02}:{:02}.{:02}", lap_min, lap_sec, lap_cs);

    let lap_label = Label::new(Some(&lap_str));
    lap_label.add_css_class("lap-time");
    row.add_suffix(&lap_label);

    // Total time
    let total_ms = total_time.as_millis();
    let total_min = (total_ms / 60000) % 60;
    let total_sec = (total_ms / 1000) % 60;
    let total_cs = (total_ms % 1000) / 10;
    let total_str = format!("{:02}:{:02}.{:02}", total_min, total_sec, total_cs);

    row.set_subtitle(&format!("Total: {}", total_str));

    row
}
