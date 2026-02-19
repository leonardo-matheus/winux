//! NetworkManager D-Bus integration module
//!
//! Provides async interface to NetworkManager via D-Bus for:
//! - Network scanning
//! - Connection management
//! - Device monitoring
//! - VPN control

mod dbus;
mod wifi_scan;

pub use dbus::*;
pub use wifi_scan::*;

use thiserror::Error;

/// Network management errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),

    #[error("NetworkManager not available")]
    NetworkManagerNotAvailable,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Network device state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Unknown,
    Unmanaged,
    Unavailable,
    Disconnected,
    Preparing,
    ConfiguringHardware,
    NeedAuth,
    ConfiguringIP,
    CheckingIP,
    WaitingForSecondaries,
    Activated,
    Deactivating,
    Failed,
}

impl From<u32> for DeviceState {
    fn from(state: u32) -> Self {
        match state {
            0 => DeviceState::Unknown,
            10 => DeviceState::Unmanaged,
            20 => DeviceState::Unavailable,
            30 => DeviceState::Disconnected,
            40 => DeviceState::Preparing,
            50 => DeviceState::ConfiguringHardware,
            60 => DeviceState::NeedAuth,
            70 => DeviceState::ConfiguringIP,
            80 => DeviceState::CheckingIP,
            90 => DeviceState::WaitingForSecondaries,
            100 => DeviceState::Activated,
            110 => DeviceState::Deactivating,
            120 => DeviceState::Failed,
            _ => DeviceState::Unknown,
        }
    }
}

/// WiFi security type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    None,
    WEP,
    WPA,
    WPA2,
    WPA3,
    Enterprise,
}

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Ethernet,
    Wifi,
    Vpn,
    Bridge,
    Bond,
    Vlan,
    Other,
}

impl ConnectionType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "802-3-ethernet" => ConnectionType::Ethernet,
            "802-11-wireless" => ConnectionType::Wifi,
            "vpn" => ConnectionType::Vpn,
            "bridge" => ConnectionType::Bridge,
            "bond" => ConnectionType::Bond,
            "vlan" => ConnectionType::Vlan,
            _ => ConnectionType::Other,
        }
    }
}
