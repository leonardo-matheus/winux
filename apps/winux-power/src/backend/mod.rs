// Backend module for Winux Power
// Integrates with UPower, Power Profiles Daemon and TLP via D-Bus

mod upower;
mod ppd;
mod tlp;

pub use upower::UPowerClient;
pub use ppd::PowerProfilesDaemon;
pub use tlp::TlpClient;

use std::cell::RefCell;
use std::rc::Rc;

/// Power profile types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerProfile {
    Performance,
    Balanced,
    PowerSaver,
}

impl Default for PowerProfile {
    fn default() -> Self {
        PowerProfile::Balanced
    }
}

impl std::fmt::Display for PowerProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerProfile::Performance => write!(f, "performance"),
            PowerProfile::Balanced => write!(f, "balanced"),
            PowerProfile::PowerSaver => write!(f, "power-saver"),
        }
    }
}

impl From<&str> for PowerProfile {
    fn from(s: &str) -> Self {
        match s {
            "performance" => PowerProfile::Performance,
            "power-saver" => PowerProfile::PowerSaver,
            _ => PowerProfile::Balanced,
        }
    }
}

/// Battery state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Unknown,
    Charging,
    Discharging,
    Empty,
    FullyCharged,
    PendingCharge,
    PendingDischarge,
}

impl Default for BatteryState {
    fn default() -> Self {
        BatteryState::Unknown
    }
}

/// Battery information
#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub percentage: u32,
    pub state: BatteryState,
    pub time_to_empty: u32,  // seconds
    pub time_to_full: u32,   // seconds
    pub energy: f64,         // Wh
    pub energy_full: f64,    // Wh
    pub energy_full_design: f64, // Wh
    pub energy_rate: f64,    // W
    pub voltage: f64,        // V
    pub temperature: f64,    // Celsius
    pub capacity: f64,       // Health percentage
    pub cycle_count: u32,
    pub technology: String,
    pub model: String,
    pub serial: String,
    pub vendor: String,
    pub is_present: bool,
    pub is_rechargeable: bool,
}

/// Device power information
#[derive(Debug, Clone)]
pub struct DevicePowerInfo {
    pub name: String,
    pub device_type: String,
    pub vendor: String,
    pub model: String,
    pub power_consumption: f64, // W
    pub can_suspend: bool,
    pub is_suspended: bool,
}

/// Power statistics entry
#[derive(Debug, Clone)]
pub struct PowerStatEntry {
    pub timestamp: i64,
    pub percentage: u32,
    pub charging: bool,
    pub energy_rate: f64,
}

/// Main power manager that coordinates all backends
pub struct PowerManager {
    upower: UPowerClient,
    ppd: PowerProfilesDaemon,
    tlp: TlpClient,
    battery_info: BatteryInfo,
    current_profile: PowerProfile,
    history: Vec<PowerStatEntry>,
}

impl PowerManager {
    pub fn new() -> Self {
        let upower = UPowerClient::new();
        let ppd = PowerProfilesDaemon::new();
        let tlp = TlpClient::new();

        // Get initial state
        let battery_info = upower.get_battery_info();
        let current_profile = ppd.get_active_profile();

        Self {
            upower,
            ppd,
            tlp,
            battery_info,
            current_profile,
            history: Vec::new(),
        }
    }

    /// Get current battery percentage
    pub fn get_battery_percentage(&self) -> u32 {
        self.upower.get_percentage()
    }

    /// Check if battery is charging
    pub fn is_charging(&self) -> bool {
        matches!(
            self.upower.get_state(),
            BatteryState::Charging | BatteryState::PendingCharge
        )
    }

    /// Get time remaining (minutes)
    pub fn get_time_remaining(&self) -> u32 {
        let info = self.upower.get_battery_info();
        if self.is_charging() {
            info.time_to_full / 60
        } else {
            info.time_to_empty / 60
        }
    }

    /// Get current energy rate (watts)
    pub fn get_energy_rate(&self) -> f64 {
        self.upower.get_energy_rate()
    }

    /// Get full battery information
    pub fn get_battery_info(&self) -> BatteryInfo {
        self.upower.get_battery_info()
    }

    /// Get current power profile
    pub fn get_current_profile(&self) -> PowerProfile {
        self.ppd.get_active_profile()
    }

    /// Set power profile
    pub fn set_profile(&mut self, profile: PowerProfile) {
        self.ppd.set_profile(profile);
        self.current_profile = profile;
    }

    /// Get available power profiles
    pub fn get_available_profiles(&self) -> Vec<PowerProfile> {
        self.ppd.get_profiles()
    }

    /// Check if on AC power
    pub fn is_on_ac(&self) -> bool {
        self.upower.is_on_ac()
    }

    /// Get battery health percentage
    pub fn get_battery_health(&self) -> f64 {
        let info = self.upower.get_battery_info();
        if info.energy_full_design > 0.0 {
            (info.energy_full / info.energy_full_design) * 100.0
        } else {
            100.0
        }
    }

    /// Get TLP status
    pub fn is_tlp_active(&self) -> bool {
        self.tlp.is_active()
    }

    /// Get USB devices with power info
    pub fn get_usb_devices(&self) -> Vec<DevicePowerInfo> {
        self.upower.get_devices()
    }

    /// Get power history
    pub fn get_history(&self, hours: u32) -> Vec<PowerStatEntry> {
        self.upower.get_history(hours)
    }

    /// Record current state to history
    pub fn record_stat(&mut self) {
        let entry = PowerStatEntry {
            timestamp: chrono::Utc::now().timestamp(),
            percentage: self.get_battery_percentage(),
            charging: self.is_charging(),
            energy_rate: self.get_energy_rate(),
        };
        self.history.push(entry);

        // Keep only last 24 hours (assuming 1 entry per minute)
        if self.history.len() > 24 * 60 {
            self.history.remove(0);
        }
    }
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}
