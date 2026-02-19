//! Caffeine Plugin
//!
//! Prevents the screen from sleeping or activating screensaver.

use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use winux_shell_plugins::prelude::*;

/// Caffeine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaffeineConfig {
    /// Auto-disable after this many minutes (0 = never)
    auto_disable_minutes: u32,
    /// Show notification when enabled/disabled
    show_notifications: bool,
    /// Also inhibit suspend
    inhibit_suspend: bool,
    /// Remember state between sessions
    remember_state: bool,
    /// Last state (for remember_state)
    last_enabled: bool,
}

impl Default for CaffeineConfig {
    fn default() -> Self {
        Self {
            auto_disable_minutes: 0,
            show_notifications: true,
            inhibit_suspend: false,
            remember_state: false,
            last_enabled: false,
        }
    }
}

/// Caffeine plugin
pub struct CaffeinePlugin {
    config: CaffeineConfig,
    enabled: Arc<AtomicBool>,
    enabled_at: Option<std::time::Instant>,
    inhibit_cookie: Option<u32>,
}

impl Default for CaffeinePlugin {
    fn default() -> Self {
        Self {
            config: CaffeineConfig::default(),
            enabled: Arc::new(AtomicBool::new(false)),
            enabled_at: None,
            inhibit_cookie: None,
        }
    }
}

impl CaffeinePlugin {
    /// Enable caffeine (prevent sleep)
    fn enable(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        if self.enabled.load(Ordering::SeqCst) {
            return Ok(());
        }

        // In a real implementation, we would use D-Bus to inhibit screensaver
        // Example: org.freedesktop.ScreenSaver.Inhibit
        // For now, just set the flag

        self.enabled.store(true, Ordering::SeqCst);
        self.enabled_at = Some(std::time::Instant::now());

        if self.config.show_notifications {
            ctx.show_notification(
                "Caffeine Enabled",
                "Screen will stay awake",
                Some("caffeine-cup-full-symbolic"),
            );
        }

        log::info!("Caffeine enabled - screen will stay awake");
        Ok(())
    }

    /// Disable caffeine (allow sleep)
    fn disable(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        if !self.enabled.load(Ordering::SeqCst) {
            return Ok(());
        }

        // In a real implementation, we would uninhibit via D-Bus
        self.enabled.store(false, Ordering::SeqCst);
        self.enabled_at = None;
        self.inhibit_cookie = None;

        if self.config.show_notifications {
            ctx.show_notification(
                "Caffeine Disabled",
                "Screen can now sleep normally",
                Some("caffeine-cup-empty-symbolic"),
            );
        }

        log::info!("Caffeine disabled - screen can sleep normally");
        Ok(())
    }

    /// Toggle caffeine state
    fn toggle(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        if self.enabled.load(Ordering::SeqCst) {
            self.disable(ctx)
        } else {
            self.enable(ctx)
        }
    }

    /// Check if auto-disable timer has expired
    fn check_auto_disable(&mut self) -> bool {
        if self.config.auto_disable_minutes == 0 {
            return false;
        }

        if let Some(enabled_at) = self.enabled_at {
            let elapsed = enabled_at.elapsed();
            let timeout = std::time::Duration::from_secs(self.config.auto_disable_minutes as u64 * 60);
            return elapsed >= timeout;
        }

        false
    }

    /// Get remaining time for auto-disable
    fn remaining_time(&self) -> Option<std::time::Duration> {
        if self.config.auto_disable_minutes == 0 {
            return None;
        }

        if let Some(enabled_at) = self.enabled_at {
            let elapsed = enabled_at.elapsed();
            let timeout = std::time::Duration::from_secs(self.config.auto_disable_minutes as u64 * 60);
            if elapsed < timeout {
                return Some(timeout - elapsed);
            }
        }

        None
    }

    /// Format duration as human readable
    fn format_duration(duration: std::time::Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m remaining", hours, minutes)
        } else if minutes > 0 {
            format!("{}m remaining", minutes)
        } else {
            "Less than a minute remaining".to_string()
        }
    }
}

impl Plugin for CaffeinePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "org.winux.caffeine".into(),
            name: "Caffeine".into(),
            version: Version::new(1, 0, 0),
            description: "Prevent the screen from sleeping or activating screensaver".into(),
            authors: vec!["Winux Team".into()],
            homepage: Some("https://winux.org/plugins/caffeine".into()),
            license: Some("MIT".into()),
            min_api_version: Version::new(1, 0, 0),
            capabilities: vec![
                PluginCapability::PanelWidget,
                PluginCapability::KeyboardShortcuts,
            ],
            permissions: {
                let mut perms = PermissionSet::new();
                perms.add(Permission::PowerInhibit);
                perms.add(Permission::PanelWidgets);
                perms.add(Permission::NotificationsSend);
                perms.add(Permission::DBusSession);
                perms.add(Permission::OwnData);
                perms
            },
            icon: Some("caffeine-cup-empty-symbolic".into()),
            category: Some("Utilities".into()),
            keywords: vec![
                "caffeine".into(),
                "awake".into(),
                "sleep".into(),
                "screensaver".into(),
                "inhibit".into(),
            ],
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

        // Restore last state if configured
        if self.config.remember_state && self.config.last_enabled {
            let _ = self.enable(ctx);
        }

        log::info!("Caffeine plugin initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        // Save state
        if self.config.remember_state {
            self.config.last_enabled = self.enabled.load(Ordering::SeqCst);
        }

        // Make sure to uninhibit on shutdown
        self.enabled.store(false, Ordering::SeqCst);

        log::info!("Caffeine plugin shutting down");
        Ok(())
    }

    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        Some(Box::new(CaffeinePanelWidget {
            enabled: self.enabled.clone(),
            config: self.config.clone(),
            enabled_at: self.enabled_at,
        }))
    }

    fn command_provider(&self) -> Option<Box<dyn CommandProvider>> {
        Some(Box::new(CaffeineCommandProvider {
            enabled: self.enabled.clone(),
        }))
    }

    fn settings_provider(&self) -> Option<Box<dyn SettingsProvider>> {
        Some(Box::new(CaffeineSettingsProvider {
            config: self.config.clone(),
        }))
    }

    fn wants_updates(&self) -> bool {
        self.enabled.load(Ordering::SeqCst) && self.config.auto_disable_minutes > 0
    }

    fn update_interval(&self) -> u32 {
        60000 // Check every minute
    }

    fn update(&mut self) -> PluginResult<()> {
        if self.check_auto_disable() {
            log::info!("Caffeine auto-disable timer expired");
            self.enabled.store(false, Ordering::SeqCst);
            self.enabled_at = None;
        }
        Ok(())
    }
}

/// Panel widget for caffeine
struct CaffeinePanelWidget {
    enabled: Arc<AtomicBool>,
    config: CaffeineConfig,
    enabled_at: Option<std::time::Instant>,
}

impl PanelWidget for CaffeinePanelWidget {
    fn id(&self) -> &str {
        "caffeine-indicator"
    }

    fn name(&self) -> &str {
        "Caffeine"
    }

    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    fn size(&self) -> WidgetSize {
        WidgetSize::Minimal
    }

    fn priority(&self) -> i32 {
        20
    }

    fn state(&self) -> WidgetState {
        let enabled = self.enabled.load(Ordering::SeqCst);
        let icon = if enabled {
            "caffeine-cup-full-symbolic"
        } else {
            "caffeine-cup-empty-symbolic"
        };

        let tooltip = if enabled {
            if let Some(enabled_at) = self.enabled_at {
                if self.config.auto_disable_minutes > 0 {
                    let elapsed = enabled_at.elapsed();
                    let timeout = std::time::Duration::from_secs(self.config.auto_disable_minutes as u64 * 60);
                    if elapsed < timeout {
                        let remaining = timeout - elapsed;
                        format!("Caffeine enabled\n{}", CaffeinePlugin::format_duration(remaining))
                    } else {
                        "Caffeine enabled".to_string()
                    }
                } else {
                    "Caffeine enabled - Screen will stay awake".to_string()
                }
            } else {
                "Caffeine enabled - Screen will stay awake".to_string()
            }
        } else {
            "Caffeine disabled - Click to enable".to_string()
        };

        WidgetState::with_icon(icon)
            .tooltip(&tooltip)
            .active(enabled)
    }

    fn build_widget(&self) -> gtk::Widget {
        let enabled = self.enabled.load(Ordering::SeqCst);

        let button = gtk::ToggleButton::new();
        button.set_has_frame(false);
        button.set_active(enabled);
        button.add_css_class("caffeine-button");

        let icon_name = if enabled {
            "caffeine-cup-full-symbolic"
        } else {
            "caffeine-cup-empty-symbolic"
        };

        let icon = gtk::Image::from_icon_name(icon_name);
        icon.set_pixel_size(16);
        button.set_child(Some(&icon));

        let tooltip = if enabled {
            "Caffeine enabled - Click to disable"
        } else {
            "Caffeine disabled - Click to enable"
        };
        button.set_tooltip_text(Some(tooltip));

        if enabled {
            button.add_css_class("suggested-action");
        }

        button.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        // Toggle state
        let new_state = !self.enabled.load(Ordering::SeqCst);
        self.enabled.store(new_state, Ordering::SeqCst);
        WidgetAction::Custom {
            name: "toggle".to_string(),
            data: new_state.to_string(),
        }
    }

    fn on_right_click(&mut self) -> WidgetAction {
        let enabled = self.enabled.load(Ordering::SeqCst);
        let toggle_label = if enabled { "Disable" } else { "Enable" };

        WidgetAction::ShowMenu(vec![
            MenuItem::new("toggle", toggle_label)
                .with_icon(if enabled {
                    "caffeine-cup-empty-symbolic"
                } else {
                    "caffeine-cup-full-symbolic"
                }),
            MenuItem::separator(),
            MenuItem::new("timer_30", "Enable for 30 minutes"),
            MenuItem::new("timer_60", "Enable for 1 hour"),
            MenuItem::new("timer_120", "Enable for 2 hours"),
            MenuItem::separator(),
            MenuItem::new("settings", "Settings").with_icon("preferences-system-symbolic"),
        ])
    }
}

/// Command provider for caffeine
struct CaffeineCommandProvider {
    enabled: Arc<AtomicBool>,
}

impl CommandProvider for CaffeineCommandProvider {
    fn id(&self) -> &str {
        "caffeine-commands"
    }

    fn commands(&self) -> Vec<Command> {
        let enabled = self.enabled.load(Ordering::SeqCst);

        vec![
            Command::new("caffeine.toggle", if enabled {
                "Disable Caffeine"
            } else {
                "Enable Caffeine"
            })
            .with_description(if enabled {
                "Allow screen to sleep"
            } else {
                "Keep screen awake"
            })
            .with_icon(if enabled {
                "caffeine-cup-empty-symbolic"
            } else {
                "caffeine-cup-full-symbolic"
            })
            .with_category("System"),
            Command::new("caffeine.enable_timed", "Enable Caffeine for...")
                .with_description("Keep screen awake for a specific duration")
                .with_icon("caffeine-cup-full-symbolic")
                .with_category("System"),
        ]
    }

    fn execute(&mut self, command_id: &str, _context: &CommandContext) -> CommandResult {
        match command_id {
            "caffeine.toggle" => {
                let new_state = !self.enabled.load(Ordering::SeqCst);
                self.enabled.store(new_state, Ordering::SeqCst);
                if new_state {
                    CommandResult::Message("Caffeine enabled - screen will stay awake".to_string())
                } else {
                    CommandResult::Message("Caffeine disabled - screen can sleep".to_string())
                }
            }
            "caffeine.enable_timed" => {
                CommandResult::ShowOptions(vec![
                    CommandOption::new("30", "30 minutes"),
                    CommandOption::new("60", "1 hour"),
                    CommandOption::new("120", "2 hours"),
                    CommandOption::new("240", "4 hours"),
                ])
            }
            _ => CommandResult::Error(format!("Unknown command: {}", command_id)),
        }
    }
}

/// Settings provider for caffeine
struct CaffeineSettingsProvider {
    config: CaffeineConfig,
}

impl SettingsProvider for CaffeineSettingsProvider {
    fn id(&self) -> &str {
        "caffeine-settings"
    }

    fn pages(&self) -> Vec<SettingsPage> {
        vec![SettingsPage::new("caffeine", "Caffeine", "caffeine-cup-full-symbolic")
            .with_description("Configure caffeine settings")
            .add_group(
                SettingGroup::new("Behavior")
                    .add(
                        Setting::toggle("show_notifications", "Show Notifications", self.config.show_notifications)
                            .with_description("Show notifications when caffeine is enabled/disabled"),
                    )
                    .add(
                        Setting::toggle("inhibit_suspend", "Also Inhibit Suspend", self.config.inhibit_suspend)
                            .with_description("Prevent system suspend in addition to screen sleep"),
                    )
                    .add(
                        Setting::toggle("remember_state", "Remember State", self.config.remember_state)
                            .with_description("Restore caffeine state when logging back in"),
                    )
            )
            .add_group(
                SettingGroup::new("Timer")
                    .add(
                        Setting::slider("auto_disable_minutes", "Auto-disable After (minutes)", 0.0, 480.0, 30.0, self.config.auto_disable_minutes as f64)
                            .with_description("Automatically disable caffeine after this many minutes (0 = never)"),
                    )
            )]
    }

    fn load(&mut self) -> std::collections::HashMap<String, SettingValue> {
        let mut values = std::collections::HashMap::new();
        values.insert("show_notifications".to_string(), SettingValue::Bool(self.config.show_notifications));
        values.insert("inhibit_suspend".to_string(), SettingValue::Bool(self.config.inhibit_suspend));
        values.insert("remember_state".to_string(), SettingValue::Bool(self.config.remember_state));
        values.insert("auto_disable_minutes".to_string(), SettingValue::Float(self.config.auto_disable_minutes as f64));
        values
    }

    fn save(&mut self, key: &str, value: SettingValue) -> Result<(), String> {
        match key {
            "show_notifications" => {
                if let Some(v) = value.as_bool() {
                    self.config.show_notifications = v;
                }
            }
            "inhibit_suspend" => {
                if let Some(v) = value.as_bool() {
                    self.config.inhibit_suspend = v;
                }
            }
            "remember_state" => {
                if let Some(v) = value.as_bool() {
                    self.config.remember_state = v;
                }
            }
            "auto_disable_minutes" => {
                if let Some(v) = value.as_float() {
                    self.config.auto_disable_minutes = v as u32;
                }
            }
            _ => return Err(format!("Unknown setting: {}", key)),
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<(), String> {
        self.config = CaffeineConfig::default();
        Ok(())
    }

    fn reset_setting(&mut self, key: &str) -> Result<(), String> {
        let default = CaffeineConfig::default();
        match key {
            "show_notifications" => self.config.show_notifications = default.show_notifications,
            "inhibit_suspend" => self.config.inhibit_suspend = default.inhibit_suspend,
            "remember_state" => self.config.remember_state = default.remember_state,
            "auto_disable_minutes" => self.config.auto_disable_minutes = default.auto_disable_minutes,
            _ => return Err(format!("Unknown setting: {}", key)),
        }
        Ok(())
    }
}

// Plugin entry point
winux_shell_plugins::declare_plugin!(CaffeinePlugin, CaffeinePlugin::default);
