//! Battery status widget
//!
//! Displays battery level, charging status, and power mode controls.

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, Orientation, ProgressBar};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use sysinfo::System;
use tracing::info;

/// Power mode options
#[derive(Clone, Debug, PartialEq)]
pub enum PowerMode {
    PowerSaver,
    Balanced,
    Performance,
}

impl PowerMode {
    fn label(&self) -> &str {
        match self {
            PowerMode::PowerSaver => "Power Saver",
            PowerMode::Balanced => "Balanced",
            PowerMode::Performance => "Performance",
        }
    }

    fn next(&self) -> PowerMode {
        match self {
            PowerMode::PowerSaver => PowerMode::Balanced,
            PowerMode::Balanced => PowerMode::Performance,
            PowerMode::Performance => PowerMode::PowerSaver,
        }
    }
}

/// Battery information
#[derive(Clone, Debug)]
pub struct BatteryInfo {
    pub percentage: u32,
    pub is_charging: bool,
    pub time_remaining: Option<String>,
    pub power_mode: PowerMode,
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self {
            percentage: 100,
            is_charging: false,
            time_remaining: None,
            power_mode: PowerMode::Balanced,
        }
    }
}

/// Battery status widget
pub struct BatteryWidget {
    container: Box,
    icon: Image,
    percentage_label: Label,
    status_label: Label,
    progress_bar: ProgressBar,
    power_mode_btn: Button,
    battery_info: Rc<RefCell<BatteryInfo>>,
    is_laptop: bool,
}

impl BatteryWidget {
    /// Create a new Battery widget
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 8);
        container.add_css_class("battery-card");

        // Check if this is a laptop with a battery
        let is_laptop = Self::has_battery();

        if !is_laptop {
            // Hide or show minimal content for desktops
            let label = Label::new(Some("Power"));
            label.add_css_class("title-4");
            container.append(&label);

            // Just show power mode selector for desktops
            let power_mode_btn = Button::builder()
                .label("Balanced")
                .build();
            power_mode_btn.add_css_class("power-mode-toggle");
            container.append(&power_mode_btn);

            let power_mode = Rc::new(Cell::new(PowerMode::Balanced));
            let power_mode_clone = power_mode.clone();
            let btn_clone = power_mode_btn.clone();

            power_mode_btn.connect_clicked(move |_| {
                let current = power_mode_clone.get();
                let next = current.next();
                power_mode_clone.set(next.clone());
                btn_clone.set_label(next.label());
                Self::set_power_mode(&next);
            });

            return Self {
                container,
                icon: Image::new(),
                percentage_label: Label::new(None),
                status_label: Label::new(None),
                progress_bar: ProgressBar::new(),
                power_mode_btn,
                battery_info: Rc::new(RefCell::new(BatteryInfo::default())),
                is_laptop: false,
            };
        }

        // Full battery widget for laptops
        let header = Box::new(Orientation::Horizontal, 12);

        // Battery icon
        let icon = Image::from_icon_name("battery-level-80-symbolic");
        icon.add_css_class("battery-icon");
        icon.set_pixel_size(32);
        header.append(&icon);

        // Battery info
        let info_box = Box::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);

        let percentage_label = Label::new(Some("80%"));
        percentage_label.add_css_class("battery-percentage");
        percentage_label.set_halign(gtk::Align::Start);
        info_box.append(&percentage_label);

        let status_label = Label::new(Some("4 hours remaining"));
        status_label.add_css_class("battery-status");
        status_label.set_halign(gtk::Align::Start);
        info_box.append(&status_label);

        header.append(&info_box);

        // Power mode toggle
        let power_mode_btn = Button::builder()
            .label("Balanced")
            .build();
        power_mode_btn.add_css_class("power-mode-toggle");
        header.append(&power_mode_btn);

        container.append(&header);

        // Progress bar
        let progress_bar = ProgressBar::new();
        progress_bar.set_fraction(0.8);
        progress_bar.add_css_class("battery-progress");
        container.append(&progress_bar);

        let battery_info = Rc::new(RefCell::new(BatteryInfo::default()));

        // Power mode toggle handler
        let battery_info_clone = battery_info.clone();
        let power_mode_btn_clone = power_mode_btn.clone();

        power_mode_btn.connect_clicked(move |_| {
            let mut info = battery_info_clone.borrow_mut();
            info.power_mode = info.power_mode.next();
            power_mode_btn_clone.set_label(info.power_mode.label());
            Self::set_power_mode(&info.power_mode);
        });

        let widget = Self {
            container,
            icon,
            percentage_label,
            status_label,
            progress_bar,
            power_mode_btn,
            battery_info,
            is_laptop,
        };

        // Load actual battery status
        widget.refresh_battery_status();
        widget.start_updates();

        widget
    }

    /// Check if the system has a battery
    fn has_battery() -> bool {
        // Check for battery in /sys/class/power_supply/
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let path = entry.path();
                let type_path = path.join("type");
                if let Ok(contents) = std::fs::read_to_string(type_path) {
                    if contents.trim() == "Battery" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Refresh battery status from the system
    fn refresh_battery_status(&self) {
        if !self.is_laptop {
            return;
        }

        // Read battery info from /sys/class/power_supply/BAT0 (or BAT1)
        let battery_paths = [
            "/sys/class/power_supply/BAT0",
            "/sys/class/power_supply/BAT1",
        ];

        for path in &battery_paths {
            let capacity_path = format!("{}/capacity", path);
            let status_path = format!("{}/status", path);

            if let Ok(capacity_str) = std::fs::read_to_string(&capacity_path) {
                if let Ok(percentage) = capacity_str.trim().parse::<u32>() {
                    let is_charging = std::fs::read_to_string(&status_path)
                        .map(|s| s.trim() == "Charging")
                        .unwrap_or(false);

                    self.update_battery_info(BatteryInfo {
                        percentage,
                        is_charging,
                        time_remaining: self.estimate_time_remaining(percentage, is_charging),
                        power_mode: self.battery_info.borrow().power_mode.clone(),
                    });
                    break;
                }
            }
        }
    }

    /// Estimate remaining time (simplified)
    fn estimate_time_remaining(&self, percentage: u32, is_charging: bool) -> Option<String> {
        if is_charging {
            if percentage < 100 {
                // Rough estimate: assume ~1.5 hours to full from 0%
                let remaining_percent = 100 - percentage;
                let hours = remaining_percent as f32 / 100.0 * 1.5;
                if hours < 1.0 {
                    Some(format!("{} min until full", (hours * 60.0) as u32))
                } else {
                    Some(format!("{:.1} hours until full", hours))
                }
            } else {
                Some("Fully charged".to_string())
            }
        } else {
            // Rough estimate: assume 5 hours at 100%
            let hours = percentage as f32 / 100.0 * 5.0;
            if hours < 1.0 {
                Some(format!("{} min remaining", (hours * 60.0) as u32))
            } else {
                Some(format!("{:.1} hours remaining", hours))
            }
        }
    }

    /// Update the displayed battery information
    pub fn update_battery_info(&self, info: BatteryInfo) {
        // Update percentage
        self.percentage_label.set_text(&format!("{}%", info.percentage));
        self.progress_bar.set_fraction(info.percentage as f64 / 100.0);

        // Update icon
        let icon_name = Self::battery_icon(info.percentage, info.is_charging);
        self.icon.set_icon_name(Some(icon_name));

        // Update icon CSS classes for color
        self.icon.remove_css_class("low");
        self.icon.remove_css_class("critical");
        self.icon.remove_css_class("charging");

        if info.is_charging {
            self.icon.add_css_class("charging");
        } else if info.percentage <= 10 {
            self.icon.add_css_class("critical");
        } else if info.percentage <= 20 {
            self.icon.add_css_class("low");
        }

        // Update status text
        if let Some(time) = &info.time_remaining {
            self.status_label.set_text(time);
        } else if info.is_charging {
            self.status_label.set_text("Charging");
        } else {
            self.status_label.set_text("On Battery");
        }

        // Update power mode button
        self.power_mode_btn.set_label(info.power_mode.label());

        *self.battery_info.borrow_mut() = info;
    }

    /// Get the battery icon name based on level and charging state
    fn battery_icon(percentage: u32, charging: bool) -> &'static str {
        let base = match percentage {
            0..=10 => "battery-level-10",
            11..=20 => "battery-level-20",
            21..=30 => "battery-level-30",
            31..=40 => "battery-level-40",
            41..=50 => "battery-level-50",
            51..=60 => "battery-level-60",
            61..=70 => "battery-level-70",
            71..=80 => "battery-level-80",
            81..=90 => "battery-level-90",
            _ => "battery-level-100",
        };

        if charging {
            match percentage {
                0..=10 => "battery-level-10-charging-symbolic",
                11..=20 => "battery-level-20-charging-symbolic",
                21..=30 => "battery-level-30-charging-symbolic",
                31..=40 => "battery-level-40-charging-symbolic",
                41..=50 => "battery-level-50-charging-symbolic",
                51..=60 => "battery-level-60-charging-symbolic",
                61..=70 => "battery-level-70-charging-symbolic",
                71..=80 => "battery-level-80-charging-symbolic",
                81..=90 => "battery-level-90-charging-symbolic",
                _ => "battery-level-100-charged-symbolic",
            }
        } else {
            match percentage {
                0..=10 => "battery-level-10-symbolic",
                11..=20 => "battery-level-20-symbolic",
                21..=30 => "battery-level-30-symbolic",
                31..=40 => "battery-level-40-symbolic",
                41..=50 => "battery-level-50-symbolic",
                51..=60 => "battery-level-60-symbolic",
                61..=70 => "battery-level-70-symbolic",
                71..=80 => "battery-level-80-symbolic",
                81..=90 => "battery-level-90-symbolic",
                _ => "battery-level-100-symbolic",
            }
        }
    }

    /// Set the system power mode
    fn set_power_mode(mode: &PowerMode) {
        let mode_str = match mode {
            PowerMode::PowerSaver => "power-saver",
            PowerMode::Balanced => "balanced",
            PowerMode::Performance => "performance",
        };

        // Try power-profiles-daemon (GNOME/systemd)
        let _ = std::process::Command::new("powerprofilesctl")
            .args(["set", mode_str])
            .spawn();

        // Alternative: TLP
        // let _ = std::process::Command::new("tlp")
        //     .arg(mode_str)
        //     .spawn();

        info!("Power mode set to: {:?}", mode);
    }

    /// Start periodic updates
    fn start_updates(&self) {
        if !self.is_laptop {
            return;
        }

        // Clone what we need for the closure
        let percentage_label = self.percentage_label.clone();
        let status_label = self.status_label.clone();
        let icon = self.icon.clone();
        let progress_bar = self.progress_bar.clone();
        let battery_info = self.battery_info.clone();

        // Update every 30 seconds
        gtk::glib::timeout_add_seconds_local(30, move || {
            // Read current battery status
            let battery_paths = [
                "/sys/class/power_supply/BAT0",
                "/sys/class/power_supply/BAT1",
            ];

            for path in &battery_paths {
                let capacity_path = format!("{}/capacity", path);
                let status_path = format!("{}/status", path);

                if let Ok(capacity_str) = std::fs::read_to_string(&capacity_path) {
                    if let Ok(percentage) = capacity_str.trim().parse::<u32>() {
                        let is_charging = std::fs::read_to_string(&status_path)
                            .map(|s| s.trim() == "Charging")
                            .unwrap_or(false);

                        // Update UI
                        percentage_label.set_text(&format!("{}%", percentage));
                        progress_bar.set_fraction(percentage as f64 / 100.0);
                        icon.set_icon_name(Some(Self::battery_icon(percentage, is_charging)));

                        if is_charging {
                            status_label.set_text("Charging");
                        } else {
                            let hours = percentage as f32 / 100.0 * 5.0;
                            status_label.set_text(&format!("{:.1} hours remaining", hours));
                        }

                        break;
                    }
                }
            }

            gtk::glib::ControlFlow::Continue
        });
    }

    /// Get the widget for adding to containers
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Get current battery percentage
    pub fn percentage(&self) -> u32 {
        self.battery_info.borrow().percentage
    }

    /// Check if charging
    pub fn is_charging(&self) -> bool {
        self.battery_info.borrow().is_charging
    }

    /// Check if this is a laptop
    pub fn is_laptop(&self) -> bool {
        self.is_laptop
    }
}

impl Default for BatteryWidget {
    fn default() -> Self {
        Self::new()
    }
}
