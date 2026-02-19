// Winux Mobile Studio - Devices Module
// Copyright (c) 2026 Winux OS Project

pub mod android_device;
pub mod ios_device;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub os_version: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    AndroidPhysical,
    AndroidEmulator,
    IOSPhysical,
    IOSSimulator,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Connected,
    Disconnected,
    Unauthorized,
    Offline,
    Booting,
}

impl Device {
    pub fn is_android(&self) -> bool {
        matches!(self.device_type, DeviceType::AndroidPhysical | DeviceType::AndroidEmulator)
    }

    pub fn is_ios(&self) -> bool {
        matches!(self.device_type, DeviceType::IOSPhysical | DeviceType::IOSSimulator)
    }

    pub fn is_connected(&self) -> bool {
        self.status == DeviceStatus::Connected
    }
}
