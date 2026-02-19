//! Protocol layer for KDE Connect compatible communication

mod discovery;
mod pairing;
mod messages;
mod encryption;

pub use discovery::DiscoveryService;
pub use pairing::PairingManager;
pub use messages::{NetworkPacket, PacketType};
pub use encryption::EncryptionManager;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

/// Connection manager for all device communications
pub struct ConnectionManager {
    devices: Arc<RwLock<HashMap<String, Device>>>,
    discovery: DiscoveryService,
    pairing: PairingManager,
    encryption: EncryptionManager,
    running: Arc<RwLock<bool>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            discovery: DiscoveryService::new(),
            pairing: PairingManager::new(),
            encryption: EncryptionManager::new(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start device discovery service
    pub fn start_discovery(&self) {
        *self.running.write().unwrap() = true;
        self.discovery.start();
    }

    /// Stop device discovery service
    pub fn stop_discovery(&self) {
        *self.running.write().unwrap() = false;
        self.discovery.stop();
    }

    /// Get list of discovered devices
    pub fn get_devices(&self) -> Vec<Device> {
        self.devices.read().unwrap().values().cloned().collect()
    }

    /// Get a specific device by ID
    pub fn get_device(&self, id: &str) -> Option<Device> {
        self.devices.read().unwrap().get(id).cloned()
    }

    /// Add or update a device
    pub fn add_device(&self, device: Device) {
        self.devices.write().unwrap().insert(device.id.clone(), device);
    }

    /// Remove a device
    pub fn remove_device(&self, id: &str) {
        self.devices.write().unwrap().remove(id);
    }

    /// Initiate pairing with a device
    pub fn pair_device(&self, device_id: &str) -> Result<String, String> {
        self.pairing.initiate_pairing(device_id)
    }

    /// Accept pairing request from a device
    pub fn accept_pairing(&self, device_id: &str) -> Result<(), String> {
        self.pairing.accept_pairing(device_id)
    }

    /// Reject pairing request from a device
    pub fn reject_pairing(&self, device_id: &str) {
        self.pairing.reject_pairing(device_id);
    }

    /// Send a packet to a device
    pub fn send_packet(&self, device_id: &str, packet: NetworkPacket) -> Result<(), String> {
        if let Some(device) = self.get_device(device_id) {
            if device.status == DeviceStatus::Connected {
                // Encrypt and send
                let encrypted = self.encryption.encrypt(&packet, device_id)?;
                // TODO: Actually send via TCP
                tracing::info!("Sending packet to {}: {:?}", device_id, encrypted);
                Ok(())
            } else {
                Err("Device not connected".to_string())
            }
        } else {
            Err("Device not found".to_string())
        }
    }

    /// Get pairing manager for QR code generation
    pub fn pairing_manager(&self) -> &PairingManager {
        &self.pairing
    }

    /// Get encryption manager
    pub fn encryption_manager(&self) -> &EncryptionManager {
        &self.encryption
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Device information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub battery: Option<u8>,
    pub ip_address: String,
    pub port: u16,
    pub protocol_version: u32,
    pub capabilities: Vec<String>,
}

impl Device {
    pub fn new(
        id: &str,
        name: &str,
        device_type: DeviceType,
        status: DeviceStatus,
        battery: Option<u8>,
        ip_address: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            device_type,
            status,
            battery,
            ip_address: ip_address.to_string(),
            port: 1716, // Default KDE Connect port
            protocol_version: 7,
            capabilities: vec![
                "kdeconnect.ping".to_string(),
                "kdeconnect.battery".to_string(),
                "kdeconnect.clipboard".to_string(),
                "kdeconnect.notification".to_string(),
                "kdeconnect.share".to_string(),
                "kdeconnect.mprisremote".to_string(),
                "kdeconnect.findmyphone".to_string(),
                "kdeconnect.sms".to_string(),
            ],
        }
    }
}

/// Device type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Phone,
    Tablet,
    Desktop,
    Laptop,
    Tv,
    Unknown,
}

impl DeviceType {
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Phone => "phone-symbolic",
            Self::Tablet => "tablet-symbolic",
            Self::Desktop => "computer-symbolic",
            Self::Laptop => "laptop-symbolic",
            Self::Tv => "video-display-symbolic",
            Self::Unknown => "computer-symbolic",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "phone" | "smartphone" => Self::Phone,
            "tablet" => Self::Tablet,
            "desktop" => Self::Desktop,
            "laptop" => Self::Laptop,
            "tv" => Self::Tv,
            _ => Self::Unknown,
        }
    }
}

/// Device connection status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Discovered,
    Pairing,
    Paired,
    Connected,
    Disconnected,
}

impl DeviceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Discovered => "Descoberto",
            Self::Pairing => "Pareando...",
            Self::Paired => "Pareado",
            Self::Connected => "Conectado",
            Self::Disconnected => "Desconectado",
        }
    }
}
