//! mDNS/DNS-SD device discovery service
//!
//! Uses Avahi/mDNS for automatic device discovery on the local network.
//! Compatible with KDE Connect's discovery protocol.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Service type for KDE Connect compatible discovery
pub const SERVICE_TYPE: &str = "_kdeconnect._udp.local.";

/// Default discovery port (UDP)
pub const DISCOVERY_PORT: u16 = 1716;

/// Discovery service for finding devices on the network
pub struct DiscoveryService {
    running: Arc<RwLock<bool>>,
    discovered: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            discovered: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the discovery service
    pub fn start(&self) {
        *self.running.write().unwrap() = true;
        tracing::info!("Discovery service started");

        // In production, this would:
        // 1. Register our service with mDNS
        // 2. Browse for other devices
        // 3. Handle announcements

        self.register_service();
        self.browse_services();
    }

    /// Stop the discovery service
    pub fn stop(&self) {
        *self.running.write().unwrap() = false;
        tracing::info!("Discovery service stopped");
    }

    /// Register our device on the network
    fn register_service(&self) {
        // Create identity packet for broadcasting
        let identity = IdentityPacket {
            device_id: self.get_device_id(),
            device_name: self.get_device_name(),
            device_type: "desktop".to_string(),
            protocol_version: 7,
            incoming_capabilities: vec![
                "kdeconnect.ping".to_string(),
                "kdeconnect.battery".to_string(),
                "kdeconnect.battery.request".to_string(),
                "kdeconnect.clipboard".to_string(),
                "kdeconnect.clipboard.connect".to_string(),
                "kdeconnect.notification".to_string(),
                "kdeconnect.notification.request".to_string(),
                "kdeconnect.share.request".to_string(),
                "kdeconnect.mprisremote".to_string(),
                "kdeconnect.findmyphone.request".to_string(),
                "kdeconnect.sms.messages".to_string(),
            ],
            outgoing_capabilities: vec![
                "kdeconnect.ping".to_string(),
                "kdeconnect.battery".to_string(),
                "kdeconnect.battery.request".to_string(),
                "kdeconnect.clipboard".to_string(),
                "kdeconnect.clipboard.connect".to_string(),
                "kdeconnect.notification".to_string(),
                "kdeconnect.notification.request".to_string(),
                "kdeconnect.share.request".to_string(),
                "kdeconnect.mprisremote".to_string(),
                "kdeconnect.findmyphone.request".to_string(),
                "kdeconnect.sms.request".to_string(),
            ],
            tcp_port: 1716,
        };

        tracing::info!("Registering service: {:?}", identity.device_name);
    }

    /// Browse for KDE Connect compatible devices
    fn browse_services(&self) {
        // In production, this would use mdns-sd crate to:
        // 1. Listen for mDNS announcements
        // 2. Query for _kdeconnect._udp.local. services
        // 3. Resolve service addresses

        tracing::info!("Browsing for services on {}", SERVICE_TYPE);
    }

    /// Get our device ID (persistent across restarts)
    fn get_device_id(&self) -> String {
        // In production, this would be stored in a config file
        // and generated using UUID on first run
        "winux_connect_".to_string() + &uuid::Uuid::new_v4().to_string()[..8]
    }

    /// Get our device name
    fn get_device_name(&self) -> String {
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Winux PC".to_string())
    }

    /// Add a discovered device
    pub fn add_discovered(&self, device: DiscoveredDevice) {
        self.discovered.write().unwrap().insert(device.id.clone(), device);
    }

    /// Remove a discovered device
    pub fn remove_discovered(&self, id: &str) {
        self.discovered.write().unwrap().remove(id);
    }

    /// Get all discovered devices
    pub fn get_discovered(&self) -> Vec<DiscoveredDevice> {
        self.discovered.read().unwrap().values().cloned().collect()
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        *self.running.read().unwrap()
    }

    /// Send UDP broadcast for discovery
    pub fn send_broadcast(&self) -> Result<(), std::io::Error> {
        use std::net::UdpSocket;

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;

        let identity = IdentityPacket {
            device_id: self.get_device_id(),
            device_name: self.get_device_name(),
            device_type: "desktop".to_string(),
            protocol_version: 7,
            incoming_capabilities: vec![],
            outgoing_capabilities: vec![],
            tcp_port: 1716,
        };

        let packet = serde_json::to_string(&identity).unwrap_or_default();
        let broadcast_addr = format!("255.255.255.255:{}", DISCOVERY_PORT);

        socket.send_to(packet.as_bytes(), &broadcast_addr)?;
        tracing::info!("Broadcast sent to {}", broadcast_addr);

        Ok(())
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Device discovered on the network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub ip_address: String,
    pub port: u16,
    pub protocol_version: u32,
    pub capabilities: Vec<String>,
}

/// KDE Connect identity packet
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityPacket {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub protocol_version: u32,
    pub incoming_capabilities: Vec<String>,
    pub outgoing_capabilities: Vec<String>,
    pub tcp_port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_service_creation() {
        let service = DiscoveryService::new();
        assert!(!service.is_running());
    }

    #[test]
    fn test_discovery_service_start_stop() {
        let service = DiscoveryService::new();
        service.start();
        assert!(service.is_running());
        service.stop();
        assert!(!service.is_running());
    }

    #[test]
    fn test_discovered_devices() {
        let service = DiscoveryService::new();

        let device = DiscoveredDevice {
            id: "test_device".to_string(),
            name: "Test Phone".to_string(),
            device_type: "phone".to_string(),
            ip_address: "192.168.1.100".to_string(),
            port: 1716,
            protocol_version: 7,
            capabilities: vec!["kdeconnect.ping".to_string()],
        };

        service.add_discovered(device.clone());
        let devices = service.get_discovered();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Test Phone");

        service.remove_discovered("test_device");
        let devices = service.get_discovered();
        assert!(devices.is_empty());
    }
}
