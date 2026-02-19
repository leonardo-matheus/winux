// Timer Tab - Countdown timers with presets

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, ProgressBar};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::ui::time_picker::TimePicker;

#[derive(Clone)]
struct TimerState {
    running: bool,
    paused: bool,
    start_time: Option<Instant>,
    total_duration: Duration,
    remaining: Duration,
    pause_time: Option<Instant>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            running: false,
            paused: false,
            start_time: None,
            total_duration: Duration::from_secs(60),
            remaining: Duration::from_secs(60),
            pause_time: None,
        }
    }
}

pub fn create_timer_tab() -> Box {
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // State
    let state: Rc<RefCell<TimerState>> = Rc::new(RefCell::new(TimerState::default()));

    // Timer display
    let display_box = Box::new(Orientation::Vertical, 16);
    display_box.set_halign(gtk4::Align::Center);
    display_box.set_margin_top(20);
    display_box.set_margin_bottom(20);

    // Progress ring (simulated with progress bar for now)
    let progress = ProgressBar::new();
    progress.set_fraction(1.0);
    progress.set_size_request(250, 8);
    progress.add_css_class("osd");
    display_box.append(&progress);

    // Time display
    let time_label = Label::new(Some("01:00"));
    time_label.add_css_class("stopwatch-display");
    time_label.set_margin_top(20);
    display_box.append(&time_label);

    // Status label
    let status_label = Label::new(Some(""));
    status_label.add_css_class("timezone-label");
    display_box.append(&status_label);

    main_box.append(&display_box);

    // Time picker (hidden when running)
    let picker = TimePicker::new();
    picker.widget().set_margin_bottom(20);
    picker.set_time(0, 1, 0);
    main_box.append(picker.widget());

    // Presets
    let presets_box = Box::new(Orientation::Horizontal, 8);
    presets_box.set_halign(gtk4::Align::Center);
    presets_box.set_margin_bottom(20);

    let presets = [
        ("1 min", 60),
        ("3 min", 180),
        ("5 min", 300),
        ("10 min", 600),
        ("15 min", 900),
        ("30 min", 1800),
    ];

    let picker_clone = picker.clone();
    let state_clone = state.clone();
    for (label, seconds) in presets {
        let btn = Button::with_label(label);
        btn.add_css_class("timer-preset");
        btn.add_css_class("flat");

        let picker_clone = picker_clone.clone();
        let state_clone = state_clone.clone();
        let secs = seconds;
        btn.connect_clicked(move |_| {
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let s = secs % 60;
            picker_clone.set_time(hours, mins, s);

            let mut state = state_clone.borrow_mut();
            state.total_duration = Duration::from_secs(secs as u64);
            state.remaining = state.total_duration;
        });

        presets_box.append(&btn);
    }

    main_box.append(&presets_box);

    // Control buttons
    let controls_box = Box::new(Orientation::Horizontal, 20);
    controls_box.set_halign(gtk4::Align::Center);
    controls_box.set_margin_top(20);
    controls_box.set_margin_bottom(20);

    // Cancel button
    let cancel_btn = Button::new();
    cancel_btn.set_size_request(70, 70);
    cancel_btn.add_css_class("circular");
    cancel_btn.add_css_class("control-button");
    cancel_btn.add_css_class("reset");
    let cancel_label = Label::new(Some("Cancelar"));
    cancel_btn.set_child(Some(&cancel_label));
    cancel_btn.set_sensitive(false);

    // Start/Pause button
    let start_btn = Button::new();
    start_btn.set_size_request(70, 70);
    start_btn.add_css_class("circular");
    start_btn.add_css_class("control-button");
    start_btn.add_css_class("start");
    let start_label = Label::new(Some("Iniciar"));
    start_btn.set_child(Some(&start_label));

    controls_box.append(&cancel_btn);
    controls_box.append(&start_btn);
    main_box.append(&controls_box);

    // Recent timers
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let page = PreferencesPage::new();

    let recent_group = PreferencesGroup::builder()
        .title("Recentes")
        .build();

    // Sample recent timers
    let recent_timers = [
        ("Cha", "5:00"),
        ("Exercicio", "30:00"),
        ("Descanso", "10:00"),
    ];

    for (name, duration) in recent_timers {
        let row = ActionRow::builder()
            .title(name)
            .subtitle(duration)
            .activatable(true)
            .build();

        let use_btn = Button::from_icon_name("media-playback-start-symbolic");
        use_btn.add_css_class("flat");
        use_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&use_btn);

        recent_group.add(&row);
    }

    page.add(&recent_group);
    scrolled.set_child(Some(&page));
    main_box.append(&scrolled);

    // Update timer display
    let time_label_clone = time_label.clone();
    let progress_clone = progress.clone();
    let status_label_clone = status_label.clone();
    let state_clone = state.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        let state = state_clone.borrow();

        if state.running && !state.paused {
            if let Some(start) = state.start_time {
                let elapsed = start.elapsed();
                let remaining = if elapsed >= state.total_duration {
                    Duration::ZERO
                } else {
                    state.total_duration - elapsed
                };

                // Update display
                let total_secs = remaining.as_secs();
                let hours = total_secs / 3600;
                let mins = (total_secs % 3600) / 60;
                let secs = total_secs % 60;

                if hours > 0 {
                    time_label_clone.set_text(&format!("{:02}:{:02}:{:02}", hours, mins, secs));
                } else {
                    time_label_clone.set_text(&format!("{:02}:{:02}", mins, secs));
                }

                // Update progress
                let fraction = remaining.as_secs_f64() / state.total_duration.as_secs_f64();
                progress_clone.set_fraction(fraction);

                // Check if finished
                if remaining == Duration::ZERO {
                    status_label_clone.set_text("Tempo esgotado!");
                    status_label_clone.add_css_class("pulsing");
                    // Trigger notification
                    crate::notifications::alarm_notify::trigger_timer_finished();
                }
            }
        } else if state.paused {
            let total_secs = state.remaining.as_secs();
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;

            if hours > 0 {
                time_label_clone.set_text(&format!("{:02}:{:02}:{:02}", hours, mins, secs));
            } else {
                time_label_clone.set_text(&format!("{:02}:{:02}", mins, secs));
            }
        }

        glib::ControlFlow::Continue
    });

    // Start/Pause button handler
    let state_clone = state.clone();
    let start_btn_clone = start_btn.clone();
    let start_label_clone = start_label.clone();
    let cancel_btn_clone = cancel_btn.clone();
    let picker_clone = picker.clone();
    let time_label_clone2 = time_label.clone();
    let progress_clone2 = progress.clone();
    let status_label_clone2 = status_label.clone();

    start_btn.connect_clicked(move |btn| {
        let mut state = state_clone.borrow_mut();

        if !state.running {
            // Start timer
            let (hours, mins, secs) = picker_clone.get_time();
            let total_secs = hours * 3600 + mins * 60 + secs;

            if total_secs == 0 {
                return;
            }

            state.total_duration = Duration::from_secs(total_secs as u64);
            state.remaining = state.total_duration;
            state.start_time = Some(Instant::now());
            state.running = true;
            state.paused = false;

            start_label_clone.set_text("Pausar");
            btn.remove_css_class("start");
            btn.add_css_class("stop");
            cancel_btn_clone.set_sensitive(true);
            picker_clone.widget().set_visible(false);
            status_label_clone2.set_text("Em execucao");
            status_label_clone2.remove_css_class("pulsing");
        } else if state.paused {
            // Resume
            let pause_duration = if let Some(pause_time) = state.pause_time {
                pause_time.elapsed()
            } else {
                Duration::ZERO
            };

            // Adjust start time to account for pause
            if let Some(ref mut start) = state.start_time {
                *start = Instant::now() - (state.total_duration - state.remaining);
            }

            state.paused = false;
            start_label_clone.set_text("Pausar");
            status_label_clone2.set_text("Em execucao");
        } else {
            // Pause
            if let Some(start) = state.start_time {
                let elapsed = start.elapsed();
                state.remaining = if elapsed >= state.total_duration {
                    Duration::ZERO
                } else {
                    state.total_duration - elapsed
                };
            }
            state.paused = true;
            state.pause_time = Some(Instant::now());
            start_label_clone.set_text("Continuar");
            btn.remove_css_class("stop");
            btn.add_css_class("start");
            status_label_clone2.set_text("Pausado");
        }
    });

    // Cancel button handler
    let state_clone = state.clone();
    let start_btn_weak = start_btn.downgrade();
    let start_label_weak = start_label.downgrade();
    let cancel_btn_weak = cancel_btn.downgrade();
    let picker_weak = picker.widget().downgrade();
    let time_label_weak = time_label.downgrade();
    let progress_weak = progress.downgrade();
    let status_label_weak = status_label.downgrade();

    cancel_btn.connect_clicked(move |_| {
        let mut state = state_clone.borrow_mut();
        state.running = false;
        state.paused = false;
        state.start_time = None;
        state.remaining = state.total_duration;

        if let Some(start_btn) = start_btn_weak.upgrade() {
            start_btn.remove_css_class("stop");
            start_btn.add_css_class("start");
        }
        if let Some(start_label) = start_label_weak.upgrade() {
            start_label.set_text("Iniciar");
        }
        if let Some(cancel_btn) = cancel_btn_weak.upgrade() {
            cancel_btn.set_sensitive(false);
        }
        if let Some(picker) = picker_weak.upgrade() {
            picker.set_visible(true);
        }
        if let Some(time_label) = time_label_weak.upgrade() {
            time_label.set_text("01:00");
        }
        if let Some(progress) = progress_weak.upgrade() {
            progress.set_fraction(1.0);
        }
        if let Some(status_label) = status_label_weak.upgrade() {
            status_label.set_text("");
            status_label.remove_css_class("pulsing");
        }
    });

    main_box
}
