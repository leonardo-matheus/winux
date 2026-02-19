//! Pomodoro Timer Plugin
//!
//! A productivity timer following the Pomodoro Technique.

use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use winux_shell_plugins::prelude::*;

/// Timer state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TimerState {
    /// Timer is idle/stopped
    Idle,
    /// Work session in progress
    Working,
    /// Short break in progress
    ShortBreak,
    /// Long break in progress
    LongBreak,
    /// Timer is paused
    Paused,
}

impl Default for TimerState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Pomodoro statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PomodoroStats {
    /// Total pomodoros completed today
    pomodoros_today: u32,
    /// Total pomodoros completed all time
    pomodoros_total: u32,
    /// Total work time today (seconds)
    work_time_today: u64,
    /// Current streak (consecutive days with at least one pomodoro)
    streak: u32,
    /// Last activity date
    last_activity: Option<chrono::NaiveDate>,
}

/// Pomodoro configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PomodoroConfig {
    /// Work duration in minutes
    work_duration: u32,
    /// Short break duration in minutes
    short_break: u32,
    /// Long break duration in minutes
    long_break: u32,
    /// Pomodoros before long break
    pomodoros_before_long_break: u32,
    /// Show notifications
    show_notifications: bool,
    /// Play sound on timer end
    play_sound: bool,
    /// Auto-start breaks
    auto_start_breaks: bool,
    /// Auto-start work after breaks
    auto_start_work: bool,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: 25,
            short_break: 5,
            long_break: 15,
            pomodoros_before_long_break: 4,
            show_notifications: true,
            play_sound: true,
            auto_start_breaks: false,
            auto_start_work: false,
        }
    }
}

/// Timer data
#[derive(Debug, Clone)]
struct TimerData {
    state: TimerState,
    remaining_seconds: u32,
    total_seconds: u32,
    pomodoros_completed: u32,
}

impl Default for TimerData {
    fn default() -> Self {
        Self {
            state: TimerState::Idle,
            remaining_seconds: 0,
            total_seconds: 0,
            pomodoros_completed: 0,
        }
    }
}

/// Pomodoro timer plugin
pub struct PomodoroPlugin {
    config: PomodoroConfig,
    timer: Arc<RwLock<TimerData>>,
    stats: Arc<RwLock<PomodoroStats>>,
    paused_state: Option<TimerState>,
}

impl Default for PomodoroPlugin {
    fn default() -> Self {
        Self {
            config: PomodoroConfig::default(),
            timer: Arc::new(RwLock::new(TimerData::default())),
            stats: Arc::new(RwLock::new(PomodoroStats::default())),
            paused_state: None,
        }
    }
}

impl PomodoroPlugin {
    /// Start a work session
    fn start_work(&mut self, ctx: &PluginContext) {
        let mut timer = self.timer.write().unwrap();
        timer.state = TimerState::Working;
        timer.total_seconds = self.config.work_duration * 60;
        timer.remaining_seconds = timer.total_seconds;

        if self.config.show_notifications {
            ctx.show_notification(
                "Pomodoro Started",
                &format!("Focus for {} minutes!", self.config.work_duration),
                Some("alarm-symbolic"),
            );
        }

        log::info!("Pomodoro work session started");
    }

    /// Start a break
    fn start_break(&mut self, ctx: &PluginContext, long: bool) {
        let mut timer = self.timer.write().unwrap();

        if long {
            timer.state = TimerState::LongBreak;
            timer.total_seconds = self.config.long_break * 60;
        } else {
            timer.state = TimerState::ShortBreak;
            timer.total_seconds = self.config.short_break * 60;
        }
        timer.remaining_seconds = timer.total_seconds;

        let break_type = if long { "Long Break" } else { "Short Break" };
        let duration = if long { self.config.long_break } else { self.config.short_break };

        if self.config.show_notifications {
            ctx.show_notification(
                break_type,
                &format!("Take a {} minute break!", duration),
                Some("coffee-symbolic"),
            );
        }

        log::info!("{} started ({} minutes)", break_type, duration);
    }

    /// Stop the timer
    fn stop(&mut self) {
        let mut timer = self.timer.write().unwrap();
        timer.state = TimerState::Idle;
        timer.remaining_seconds = 0;
        timer.total_seconds = 0;
        self.paused_state = None;

        log::info!("Pomodoro timer stopped");
    }

    /// Pause/resume the timer
    fn toggle_pause(&mut self) {
        let mut timer = self.timer.write().unwrap();

        if timer.state == TimerState::Paused {
            // Resume
            if let Some(state) = self.paused_state.take() {
                timer.state = state;
                log::info!("Pomodoro timer resumed");
            }
        } else if timer.state != TimerState::Idle {
            // Pause
            self.paused_state = Some(timer.state);
            timer.state = TimerState::Paused;
            log::info!("Pomodoro timer paused");
        }
    }

    /// Tick the timer (called every second when running)
    fn tick(&mut self, ctx: &PluginContext) {
        let mut timer = self.timer.write().unwrap();

        if timer.state == TimerState::Idle || timer.state == TimerState::Paused {
            return;
        }

        if timer.remaining_seconds > 0 {
            timer.remaining_seconds -= 1;
        }

        if timer.remaining_seconds == 0 {
            match timer.state {
                TimerState::Working => {
                    // Work session completed
                    timer.pomodoros_completed += 1;

                    // Update stats
                    let mut stats = self.stats.write().unwrap();
                    stats.pomodoros_today += 1;
                    stats.pomodoros_total += 1;
                    stats.work_time_today += self.config.work_duration as u64 * 60;
                    stats.last_activity = Some(chrono::Local::now().date_naive());

                    drop(stats);

                    if self.config.show_notifications {
                        ctx.show_notification(
                            "Pomodoro Complete!",
                            &format!(
                                "Great work! You've completed {} pomodoro{}",
                                timer.pomodoros_completed,
                                if timer.pomodoros_completed == 1 { "" } else { "s" }
                            ),
                            Some("emblem-ok-symbolic"),
                        );
                    }

                    // Determine break type
                    let long_break = timer.pomodoros_completed % self.config.pomodoros_before_long_break == 0;
                    timer.state = TimerState::Idle;

                    if self.config.auto_start_breaks {
                        drop(timer);
                        self.start_break(ctx, long_break);
                    }
                }
                TimerState::ShortBreak | TimerState::LongBreak => {
                    // Break completed
                    if self.config.show_notifications {
                        ctx.show_notification(
                            "Break Over!",
                            "Ready to get back to work?",
                            Some("appointment-soon-symbolic"),
                        );
                    }

                    timer.state = TimerState::Idle;

                    if self.config.auto_start_work {
                        drop(timer);
                        self.start_work(ctx);
                    }
                }
                _ => {}
            }
        }
    }

    /// Format seconds as MM:SS
    fn format_time(seconds: u32) -> String {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Get state label
    fn state_label(state: TimerState) -> &'static str {
        match state {
            TimerState::Idle => "Ready",
            TimerState::Working => "Working",
            TimerState::ShortBreak => "Short Break",
            TimerState::LongBreak => "Long Break",
            TimerState::Paused => "Paused",
        }
    }

    /// Get state icon
    fn state_icon(state: TimerState) -> &'static str {
        match state {
            TimerState::Idle => "alarm-symbolic",
            TimerState::Working => "media-record-symbolic",
            TimerState::ShortBreak | TimerState::LongBreak => "coffee-symbolic",
            TimerState::Paused => "media-playback-pause-symbolic",
        }
    }
}

impl Plugin for PomodoroPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "org.winux.pomodoro".into(),
            name: "Pomodoro Timer".into(),
            version: Version::new(1, 0, 0),
            description: "Productivity timer using the Pomodoro Technique".into(),
            authors: vec!["Winux Team".into()],
            homepage: Some("https://winux.org/plugins/pomodoro".into()),
            license: Some("MIT".into()),
            min_api_version: Version::new(1, 0, 0),
            capabilities: vec![PluginCapability::PanelWidget],
            permissions: {
                let mut perms = PermissionSet::new();
                perms.add(Permission::PanelWidgets);
                perms.add(Permission::NotificationsSend);
                perms.add(Permission::Audio);
                perms.add(Permission::OwnData);
                perms
            },
            icon: Some("alarm-symbolic".into()),
            category: Some("Productivity".into()),
            keywords: vec!["pomodoro".into(), "timer".into(), "productivity".into(), "focus".into()],
            ..Default::default()
        }
    }

    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        // Load config
        let config_path = ctx.config_file("config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.config = config;
                }
            }
        }

        // Load stats
        let stats_path = ctx.data_file("stats.json");
        if stats_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&stats_path) {
                if let Ok(stats) = serde_json::from_str(&content) {
                    *self.stats.write().unwrap() = stats;
                }
            }
        }

        // Check if we need to reset daily stats
        let today = chrono::Local::now().date_naive();
        let mut stats = self.stats.write().unwrap();
        if let Some(last) = stats.last_activity {
            if last != today {
                // Reset daily stats
                if last == today.pred_opt().unwrap_or(today) {
                    // Consecutive day - increase streak
                    stats.streak += 1;
                } else {
                    // Streak broken
                    stats.streak = 0;
                }
                stats.pomodoros_today = 0;
                stats.work_time_today = 0;
            }
        }
        drop(stats);

        log::info!("Pomodoro plugin initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        log::info!("Pomodoro plugin shutting down");
        Ok(())
    }

    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        Some(Box::new(PomodoroPanelWidget {
            timer: self.timer.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }))
    }

    fn command_provider(&self) -> Option<Box<dyn CommandProvider>> {
        Some(Box::new(PomodoroCommandProvider {
            timer: self.timer.clone(),
        }))
    }

    fn settings_provider(&self) -> Option<Box<dyn SettingsProvider>> {
        Some(Box::new(PomodoroSettingsProvider {
            config: self.config.clone(),
        }))
    }

    fn wants_updates(&self) -> bool {
        let timer = self.timer.read().unwrap();
        timer.state != TimerState::Idle && timer.state != TimerState::Paused
    }

    fn update_interval(&self) -> u32 {
        1000 // Every second
    }

    fn update(&mut self) -> PluginResult<()> {
        // Note: In a real implementation, we'd need access to ctx here
        // For now, we'll just decrement the timer
        let mut timer = self.timer.write().unwrap();
        if timer.state != TimerState::Idle && timer.state != TimerState::Paused && timer.remaining_seconds > 0 {
            timer.remaining_seconds -= 1;
        }
        Ok(())
    }
}

/// Panel widget for pomodoro
struct PomodoroPanelWidget {
    timer: Arc<RwLock<TimerData>>,
    config: PomodoroConfig,
    stats: Arc<RwLock<PomodoroStats>>,
}

impl PanelWidget for PomodoroPanelWidget {
    fn id(&self) -> &str {
        "pomodoro-timer"
    }

    fn name(&self) -> &str {
        "Pomodoro"
    }

    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    fn size(&self) -> WidgetSize {
        WidgetSize::Small
    }

    fn priority(&self) -> i32 {
        12
    }

    fn state(&self) -> WidgetState {
        let timer = self.timer.read().unwrap();
        let icon = PomodoroPlugin::state_icon(timer.state);

        let label = if timer.state == TimerState::Idle {
            String::new()
        } else {
            PomodoroPlugin::format_time(timer.remaining_seconds)
        };

        WidgetState::with_icon(icon)
            .label(&label)
            .tooltip(&format!(
                "{}\nPomodoros today: {}",
                PomodoroPlugin::state_label(timer.state),
                timer.pomodoros_completed
            ))
            .active(timer.state == TimerState::Working)
    }

    fn build_widget(&self) -> gtk::Widget {
        let timer = self.timer.read().unwrap();

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        hbox.set_valign(gtk::Align::Center);
        hbox.add_css_class("pomodoro-widget");

        // Icon
        let icon_name = PomodoroPlugin::state_icon(timer.state);
        let icon = gtk::Image::from_icon_name(icon_name);
        icon.set_pixel_size(16);
        hbox.append(&icon);

        // Timer display
        if timer.state != TimerState::Idle {
            let time_label = gtk::Label::new(Some(&PomodoroPlugin::format_time(timer.remaining_seconds)));
            time_label.add_css_class("pomodoro-time");

            match timer.state {
                TimerState::Working => time_label.add_css_class("working"),
                TimerState::ShortBreak | TimerState::LongBreak => time_label.add_css_class("break"),
                TimerState::Paused => time_label.add_css_class("paused"),
                _ => {}
            }

            hbox.append(&time_label);
        }

        // Tooltip
        let stats = self.stats.read().unwrap();
        let tooltip = format!(
            "{}\nPomodoros today: {}\nStreak: {} days",
            PomodoroPlugin::state_label(timer.state),
            stats.pomodoros_today,
            stats.streak
        );
        hbox.set_tooltip_text(Some(&tooltip));

        hbox.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        WidgetAction::ShowPopup
    }

    fn popup_config(&self) -> Option<PopupConfig> {
        Some(PopupConfig {
            width: 280,
            height: 380,
            ..Default::default()
        })
    }

    fn build_popup(&self) -> Option<gtk::Widget> {
        let timer = self.timer.read().unwrap();
        let stats = self.stats.read().unwrap();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
        vbox.set_margin_top(16);
        vbox.set_margin_bottom(16);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);
        vbox.add_css_class("pomodoro-popup");

        // Title
        let title = gtk::Label::new(Some("Pomodoro Timer"));
        title.add_css_class("title-3");
        vbox.append(&title);

        // Timer display
        let timer_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        timer_box.set_margin_top(12);
        timer_box.set_margin_bottom(12);
        timer_box.set_halign(gtk::Align::Center);

        let time_display = if timer.state == TimerState::Idle {
            PomodoroPlugin::format_time(self.config.work_duration * 60)
        } else {
            PomodoroPlugin::format_time(timer.remaining_seconds)
        };

        let time_label = gtk::Label::new(Some(&time_display));
        time_label.add_css_class("title-1");
        timer_box.append(&time_label);

        let state_label = gtk::Label::new(Some(PomodoroPlugin::state_label(timer.state)));
        state_label.add_css_class("dim-label");
        timer_box.append(&state_label);

        // Progress bar
        if timer.state != TimerState::Idle {
            let progress = if timer.total_seconds > 0 {
                1.0 - (timer.remaining_seconds as f64 / timer.total_seconds as f64)
            } else {
                0.0
            };
            let progress_bar = gtk::ProgressBar::new();
            progress_bar.set_fraction(progress);
            progress_bar.set_margin_top(8);
            timer_box.append(&progress_bar);
        }

        vbox.append(&timer_box);

        // Controls
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        controls.set_halign(gtk::Align::Center);

        if timer.state == TimerState::Idle {
            let start_button = gtk::Button::with_label("Start Working");
            start_button.add_css_class("suggested-action");
            controls.append(&start_button);
        } else {
            let pause_label = if timer.state == TimerState::Paused { "Resume" } else { "Pause" };
            let pause_button = gtk::Button::with_label(pause_label);
            controls.append(&pause_button);

            let stop_button = gtk::Button::with_label("Stop");
            stop_button.add_css_class("destructive-action");
            controls.append(&stop_button);
        }

        vbox.append(&controls);

        // Quick actions
        let quick_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        quick_box.set_halign(gtk::Align::Center);
        quick_box.set_margin_top(8);

        let short_break_btn = gtk::Button::with_label("Short Break");
        short_break_btn.set_sensitive(timer.state == TimerState::Idle);
        quick_box.append(&short_break_btn);

        let long_break_btn = gtk::Button::with_label("Long Break");
        long_break_btn.set_sensitive(timer.state == TimerState::Idle);
        quick_box.append(&long_break_btn);

        vbox.append(&quick_box);

        // Separator
        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        sep.set_margin_top(12);
        vbox.append(&sep);

        // Stats
        let stats_box = gtk::Box::new(gtk::Orientation::Horizontal, 24);
        stats_box.set_halign(gtk::Align::Center);
        stats_box.set_margin_top(12);

        // Today
        let today_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let today_count = gtk::Label::new(Some(&stats.pomodoros_today.to_string()));
        today_count.add_css_class("title-2");
        today_box.append(&today_count);
        let today_label = gtk::Label::new(Some("Today"));
        today_label.add_css_class("dim-label");
        today_box.append(&today_label);
        stats_box.append(&today_box);

        // Total
        let total_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let total_count = gtk::Label::new(Some(&stats.pomodoros_total.to_string()));
        total_count.add_css_class("title-2");
        total_box.append(&total_count);
        let total_label = gtk::Label::new(Some("Total"));
        total_label.add_css_class("dim-label");
        total_box.append(&total_label);
        stats_box.append(&total_box);

        // Streak
        let streak_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let streak_count = gtk::Label::new(Some(&stats.streak.to_string()));
        streak_count.add_css_class("title-2");
        streak_box.append(&streak_count);
        let streak_label = gtk::Label::new(Some("Streak"));
        streak_label.add_css_class("dim-label");
        streak_box.append(&streak_label);
        stats_box.append(&streak_box);

        vbox.append(&stats_box);

        Some(vbox.upcast())
    }
}

/// Command provider for pomodoro
struct PomodoroCommandProvider {
    timer: Arc<RwLock<TimerData>>,
}

impl CommandProvider for PomodoroCommandProvider {
    fn id(&self) -> &str {
        "pomodoro-commands"
    }

    fn commands(&self) -> Vec<Command> {
        let timer = self.timer.read().unwrap();
        let is_idle = timer.state == TimerState::Idle;
        let is_paused = timer.state == TimerState::Paused;

        let mut commands = vec![];

        if is_idle {
            commands.push(
                Command::new("pomodoro.start", "Start Pomodoro")
                    .with_description("Start a 25-minute work session")
                    .with_icon("media-playback-start-symbolic")
                    .with_category("Pomodoro"),
            );
        } else {
            commands.push(
                Command::new(
                    "pomodoro.pause",
                    if is_paused { "Resume Pomodoro" } else { "Pause Pomodoro" },
                )
                .with_description(if is_paused { "Resume the timer" } else { "Pause the timer" })
                .with_icon(if is_paused {
                    "media-playback-start-symbolic"
                } else {
                    "media-playback-pause-symbolic"
                })
                .with_category("Pomodoro"),
            );

            commands.push(
                Command::new("pomodoro.stop", "Stop Pomodoro")
                    .with_description("Stop and reset the timer")
                    .with_icon("media-playback-stop-symbolic")
                    .with_category("Pomodoro"),
            );
        }

        commands.push(
            Command::new("pomodoro.short_break", "Start Short Break")
                .with_description("Start a 5-minute break")
                .with_icon("coffee-symbolic")
                .with_category("Pomodoro"),
        );

        commands.push(
            Command::new("pomodoro.long_break", "Start Long Break")
                .with_description("Start a 15-minute break")
                .with_icon("weather-clear-symbolic")
                .with_category("Pomodoro"),
        );

        commands
    }

    fn execute(&mut self, command_id: &str, _context: &CommandContext) -> CommandResult {
        match command_id {
            "pomodoro.start" => {
                let mut timer = self.timer.write().unwrap();
                timer.state = TimerState::Working;
                timer.total_seconds = 25 * 60;
                timer.remaining_seconds = timer.total_seconds;
                CommandResult::Message("Pomodoro started! Focus for 25 minutes.".to_string())
            }
            "pomodoro.pause" => {
                let mut timer = self.timer.write().unwrap();
                if timer.state == TimerState::Paused {
                    timer.state = TimerState::Working; // Simplified - should restore actual state
                    CommandResult::Message("Timer resumed".to_string())
                } else {
                    timer.state = TimerState::Paused;
                    CommandResult::Message("Timer paused".to_string())
                }
            }
            "pomodoro.stop" => {
                let mut timer = self.timer.write().unwrap();
                timer.state = TimerState::Idle;
                timer.remaining_seconds = 0;
                CommandResult::Message("Timer stopped".to_string())
            }
            "pomodoro.short_break" => {
                let mut timer = self.timer.write().unwrap();
                timer.state = TimerState::ShortBreak;
                timer.total_seconds = 5 * 60;
                timer.remaining_seconds = timer.total_seconds;
                CommandResult::Message("Short break started! Relax for 5 minutes.".to_string())
            }
            "pomodoro.long_break" => {
                let mut timer = self.timer.write().unwrap();
                timer.state = TimerState::LongBreak;
                timer.total_seconds = 15 * 60;
                timer.remaining_seconds = timer.total_seconds;
                CommandResult::Message("Long break started! Relax for 15 minutes.".to_string())
            }
            _ => CommandResult::Error(format!("Unknown command: {}", command_id)),
        }
    }
}

/// Settings provider for pomodoro
struct PomodoroSettingsProvider {
    config: PomodoroConfig,
}

impl SettingsProvider for PomodoroSettingsProvider {
    fn id(&self) -> &str {
        "pomodoro-settings"
    }

    fn pages(&self) -> Vec<SettingsPage> {
        vec![SettingsPage::new("pomodoro", "Pomodoro Timer", "alarm-symbolic")
            .with_description("Configure pomodoro timer settings")
            .add_group(
                SettingGroup::new("Durations")
                    .add(
                        Setting::slider("work_duration", "Work Duration (minutes)", 15.0, 60.0, 5.0, self.config.work_duration as f64),
                    )
                    .add(
                        Setting::slider("short_break", "Short Break (minutes)", 3.0, 15.0, 1.0, self.config.short_break as f64),
                    )
                    .add(
                        Setting::slider("long_break", "Long Break (minutes)", 10.0, 30.0, 5.0, self.config.long_break as f64),
                    )
                    .add(
                        Setting::slider("pomodoros_before_long_break", "Pomodoros Before Long Break", 2.0, 8.0, 1.0, self.config.pomodoros_before_long_break as f64),
                    )
            )
            .add_group(
                SettingGroup::new("Behavior")
                    .add(
                        Setting::toggle("auto_start_breaks", "Auto-start Breaks", self.config.auto_start_breaks)
                            .with_description("Automatically start breaks after work sessions"),
                    )
                    .add(
                        Setting::toggle("auto_start_work", "Auto-start Work", self.config.auto_start_work)
                            .with_description("Automatically start work after breaks"),
                    )
            )
            .add_group(
                SettingGroup::new("Notifications")
                    .add(
                        Setting::toggle("show_notifications", "Show Notifications", self.config.show_notifications),
                    )
                    .add(
                        Setting::toggle("play_sound", "Play Sound", self.config.play_sound),
                    )
            )]
    }

    fn load(&mut self) -> std::collections::HashMap<String, SettingValue> {
        let mut values = std::collections::HashMap::new();
        values.insert("work_duration".to_string(), SettingValue::Float(self.config.work_duration as f64));
        values.insert("short_break".to_string(), SettingValue::Float(self.config.short_break as f64));
        values.insert("long_break".to_string(), SettingValue::Float(self.config.long_break as f64));
        values.insert("pomodoros_before_long_break".to_string(), SettingValue::Float(self.config.pomodoros_before_long_break as f64));
        values.insert("auto_start_breaks".to_string(), SettingValue::Bool(self.config.auto_start_breaks));
        values.insert("auto_start_work".to_string(), SettingValue::Bool(self.config.auto_start_work));
        values.insert("show_notifications".to_string(), SettingValue::Bool(self.config.show_notifications));
        values.insert("play_sound".to_string(), SettingValue::Bool(self.config.play_sound));
        values
    }

    fn save(&mut self, key: &str, value: SettingValue) -> Result<(), String> {
        match key {
            "work_duration" => {
                if let Some(v) = value.as_float() { self.config.work_duration = v as u32; }
            }
            "short_break" => {
                if let Some(v) = value.as_float() { self.config.short_break = v as u32; }
            }
            "long_break" => {
                if let Some(v) = value.as_float() { self.config.long_break = v as u32; }
            }
            "pomodoros_before_long_break" => {
                if let Some(v) = value.as_float() { self.config.pomodoros_before_long_break = v as u32; }
            }
            "auto_start_breaks" => {
                if let Some(v) = value.as_bool() { self.config.auto_start_breaks = v; }
            }
            "auto_start_work" => {
                if let Some(v) = value.as_bool() { self.config.auto_start_work = v; }
            }
            "show_notifications" => {
                if let Some(v) = value.as_bool() { self.config.show_notifications = v; }
            }
            "play_sound" => {
                if let Some(v) = value.as_bool() { self.config.play_sound = v; }
            }
            _ => return Err(format!("Unknown setting: {}", key)),
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<(), String> {
        self.config = PomodoroConfig::default();
        Ok(())
    }

    fn reset_setting(&mut self, key: &str) -> Result<(), String> {
        let default = PomodoroConfig::default();
        match key {
            "work_duration" => self.config.work_duration = default.work_duration,
            "short_break" => self.config.short_break = default.short_break,
            "long_break" => self.config.long_break = default.long_break,
            "pomodoros_before_long_break" => self.config.pomodoros_before_long_break = default.pomodoros_before_long_break,
            "auto_start_breaks" => self.config.auto_start_breaks = default.auto_start_breaks,
            "auto_start_work" => self.config.auto_start_work = default.auto_start_work,
            "show_notifications" => self.config.show_notifications = default.show_notifications,
            "play_sound" => self.config.play_sound = default.play_sound,
            _ => return Err(format!("Unknown setting: {}", key)),
        }
        Ok(())
    }
}

// Plugin entry point
winux_shell_plugins::declare_plugin!(PomodoroPlugin, PomodoroPlugin::default);
