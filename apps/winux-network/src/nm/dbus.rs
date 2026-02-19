//! NetworkManager D-Bus interface
//!
//! Async interface to NetworkManager using zbus

use super::{ConnectionType, DeviceState, NetworkError, NetworkResult, WifiSecurity};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use zbus::{Connection, proxy};

/// NetworkManager D-Bus interface paths
const NM_SERVICE: &str = "org.freedesktop.NetworkManager";
const NM_PATH: &str = "/org/freedesktop/NetworkManager";

/// WiFi Access Point information
#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub ssid: String,
    pub bssid: String,
    pub frequency: u32,
    pub signal_strength: u8,
    pub security: WifiSecurity,
    pub is_connected: bool,
}

/// Network device information
#[derive(Debug, Clone)]
pub struct NetworkDevice {
    pub name: String,
    pub device_type: DeviceType,
    pub state: DeviceState,
    pub hw_address: String,
    pub ip_address: Option<String>,
    pub object_path: String,
}

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Unknown,
    Ethernet,
    Wifi,
    Unused1,
    Unused2,
    Bt,
    OlpcMesh,
    Wimax,
    Modem,
    Infiniband,
    Bond,
    Vlan,
    Adsl,
    Bridge,
    Generic,
    Team,
    Tun,
    IpTunnel,
    Macvlan,
    Vxlan,
    Veth,
    Macsec,
    Dummy,
    Ppp,
    OvsInterface,
    OvsPort,
    OvsBridge,
    Wpan,
    Lowpan,
    WireGuard,
    WifiP2P,
    Vrf,
}

impl From<u32> for DeviceType {
    fn from(dt: u32) -> Self {
        match dt {
            0 => DeviceType::Unknown,
            1 => DeviceType::Ethernet,
            2 => DeviceType::Wifi,
            3 => DeviceType::Unused1,
            4 => DeviceType::Unused2,
            5 => DeviceType::Bt,
            6 => DeviceType::OlpcMesh,
            7 => DeviceType::Wimax,
            8 => DeviceType::Modem,
            9 => DeviceType::Infiniband,
            10 => DeviceType::Bond,
            11 => DeviceType::Vlan,
            12 => DeviceType::Adsl,
            13 => DeviceType::Bridge,
            14 => DeviceType::Generic,
            15 => DeviceType::Team,
            16 => DeviceType::Tun,
            17 => DeviceType::IpTunnel,
            18 => DeviceType::Macvlan,
            19 => DeviceType::Vxlan,
            20 => DeviceType::Veth,
            21 => DeviceType::Macsec,
            22 => DeviceType::Dummy,
            23 => DeviceType::Ppp,
            24 => DeviceType::OvsInterface,
            25 => DeviceType::OvsPort,
            26 => DeviceType::OvsBridge,
            27 => DeviceType::Wpan,
            28 => DeviceType::Lowpan,
            29 => DeviceType::WireGuard,
            30 => DeviceType::WifiP2P,
            31 => DeviceType::Vrf,
            _ => DeviceType::Unknown,
        }
    }
}

/// Connection profile
#[derive(Debug, Clone)]
pub struct ConnectionProfile {
    pub id: String,
    pub uuid: String,
    pub connection_type: ConnectionType,
    pub autoconnect: bool,
    pub object_path: String,
}

/// Active connection information
#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub id: String,
    pub uuid: String,
    pub connection_type: ConnectionType,
    pub state: u32,
    pub default: bool,
    pub devices: Vec<String>,
}

/// NetworkManager client for D-Bus operations
#[derive(Clone)]
pub struct NetworkManagerClient {
    connection: Connection,
}

impl NetworkManagerClient {
    /// Create a new NetworkManager client
    pub async fn new() -> NetworkResult<Self> {
        let connection = Connection::system().await?;
        Ok(Self { connection })
    }

    /// Check if NetworkManager is running
    pub async fn is_available(&self) -> bool {
        // Try to get NM version to check availability
        self.get_version().await.is_ok()
    }

    /// Get NetworkManager version
    pub async fn get_version(&self) -> NetworkResult<String> {
        let proxy = zbus::fdo::PropertiesProxy::builder(&self.connection)
            .destination(NM_SERVICE)?
            .path(NM_PATH)?
            .build()
            .await?;

        let version: zbus::zvariant::OwnedValue = proxy
            .get(NM_SERVICE, "Version")
            .await?;

        Ok(version.downcast_ref::<str>().unwrap_or("unknown").to_string())
    }

    /// Get all network devices
    pub async fn get_devices(&self) -> NetworkResult<Vec<NetworkDevice>> {
        // This would use proper D-Bus calls in a real implementation
        // For now, return mock data for UI development
        Ok(vec![
            NetworkDevice {
                name: "enp3s0".to_string(),
                device_type: DeviceType::Ethernet,
                state: DeviceState::Activated,
                hw_address: "00:1A:2B:3C:4D:5E".to_string(),
                ip_address: Some("192.168.1.50".to_string()),
                object_path: "/org/freedesktop/NetworkManager/Devices/1".to_string(),
            },
            NetworkDevice {
                name: "wlan0".to_string(),
                device_type: DeviceType::Wifi,
                state: DeviceState::Activated,
                hw_address: "AA:BB:CC:DD:EE:FF".to_string(),
                ip_address: Some("192.168.1.100".to_string()),
                object_path: "/org/freedesktop/NetworkManager/Devices/2".to_string(),
            },
        ])
    }

    /// Get WiFi device path
    pub async fn get_wifi_device(&self) -> NetworkResult<String> {
        let devices = self.get_devices().await?;
        devices
            .into_iter()
            .find(|d| d.device_type == DeviceType::Wifi)
            .map(|d| d.object_path)
            .ok_or(NetworkError::DeviceNotFound("WiFi".to_string()))
    }

    /// Request a WiFi scan
    pub async fn request_wifi_scan(&self) -> NetworkResult<()> {
        let wifi_device = self.get_wifi_device().await?;
        info!("Requesting WiFi scan on device: {}", wifi_device);
        // In real implementation, call RequestScan on the WiFi device interface
        Ok(())
    }

    /// Get available WiFi access points
    pub async fn get_access_points(&self) -> NetworkResult<Vec<AccessPoint>> {
        // This would parse D-Bus responses in a real implementation
        // Mock data for UI development
        Ok(vec![
            AccessPoint {
                ssid: "Casa_5G".to_string(),
                bssid: "00:11:22:33:44:55".to_string(),
                frequency: 5180,
                signal_strength: 95,
                security: WifiSecurity::WPA3,
                is_connected: true,
            },
            AccessPoint {
                ssid: "Vizinho_Net".to_string(),
                bssid: "66:77:88:99:AA:BB".to_string(),
                frequency: 2437,
                signal_strength: 75,
                security: WifiSecurity::WPA2,
                is_connected: false,
            },
            AccessPoint {
                ssid: "Cafe_WiFi".to_string(),
                bssid: "CC:DD:EE:FF:00:11".to_string(),
                frequency: 2412,
                signal_strength: 50,
                security: WifiSecurity::None,
                is_connected: false,
            },
        ])
    }

    /// Connect to a WiFi network
    pub async fn connect_wifi(&self, ssid: &str, password: Option<&str>) -> NetworkResult<()> {
        info!("Connecting to WiFi network: {}", ssid);

        if let Some(pwd) = password {
            debug!("Using WPA authentication");
        }

        // In real implementation:
        // 1. Check if connection profile exists
        // 2. If not, create new connection with AddAndActivateConnection
        // 3. If exists, use ActivateConnection

        Ok(())
    }

    /// Disconnect from current WiFi network
    pub async fn disconnect_wifi(&self) -> NetworkResult<()> {
        let wifi_device = self.get_wifi_device().await?;
        info!("Disconnecting WiFi device: {}", wifi_device);
        // Call Disconnect on device interface
        Ok(())
    }

    /// Get saved connection profiles
    pub async fn get_connections(&self) -> NetworkResult<Vec<ConnectionProfile>> {
        Ok(vec![
            ConnectionProfile {
                id: "Casa_5G".to_string(),
                uuid: "550e8400-e29b-41d4-a716-446655440001".to_string(),
                connection_type: ConnectionType::Wifi,
                autoconnect: true,
                object_path: "/org/freedesktop/NetworkManager/Settings/1".to_string(),
            },
            ConnectionProfile {
                id: "Trabalho VPN".to_string(),
                uuid: "550e8400-e29b-41d4-a716-446655440002".to_string(),
                connection_type: ConnectionType::Vpn,
                autoconnect: false,
                object_path: "/org/freedesktop/NetworkManager/Settings/2".to_string(),
            },
        ])
    }

    /// Get active connections
    pub async fn get_active_connections(&self) -> NetworkResult<Vec<ActiveConnection>> {
        Ok(vec![
            ActiveConnection {
                id: "Casa_5G".to_string(),
                uuid: "550e8400-e29b-41d4-a716-446655440001".to_string(),
                connection_type: ConnectionType::Wifi,
                state: 2, // Activated
                default: true,
                devices: vec!["wlan0".to_string()],
            },
        ])
    }

    /// Delete a saved connection
    pub async fn delete_connection(&self, uuid: &str) -> NetworkResult<()> {
        info!("Deleting connection: {}", uuid);
        // Call Delete on Settings.Connection interface
        Ok(())
    }

    /// Activate a VPN connection
    pub async fn activate_vpn(&self, uuid: &str) -> NetworkResult<()> {
        info!("Activating VPN: {}", uuid);
        Ok(())
    }

    /// Deactivate a VPN connection
    pub async fn deactivate_vpn(&self, uuid: &str) -> NetworkResult<()> {
        info!("Deactivating VPN: {}", uuid);
        Ok(())
    }

    /// Enable/disable networking
    pub async fn set_networking_enabled(&self, enabled: bool) -> NetworkResult<()> {
        info!("Setting networking enabled: {}", enabled);
        Ok(())
    }

    /// Enable/disable WiFi
    pub async fn set_wifi_enabled(&self, enabled: bool) -> NetworkResult<()> {
        info!("Setting WiFi enabled: {}", enabled);
        Ok(())
    }

    /// Check if WiFi is enabled
    pub async fn is_wifi_enabled(&self) -> NetworkResult<bool> {
        Ok(true)
    }

    /// Get primary connection info
    pub async fn get_primary_connection(&self) -> NetworkResult<Option<ActiveConnection>> {
        let connections = self.get_active_connections().await?;
        Ok(connections.into_iter().find(|c| c.default))
    }

    /// Create a hotspot
    pub async fn create_hotspot(&self, ssid: &str, password: &str) -> NetworkResult<()> {
        info!("Creating hotspot with SSID: {}", ssid);
        // Uses AddAndActivateConnection2 with ap mode
        Ok(())
    }

    /// Stop hotspot
    pub async fn stop_hotspot(&self) -> NetworkResult<()> {
        info!("Stopping hotspot");
        Ok(())
    }
}

/// Helper to convert signal strength to icon name
pub fn signal_strength_icon(strength: u8) -> &'static str {
    match strength {
        0..=25 => "network-wireless-signal-weak-symbolic",
        26..=50 => "network-wireless-signal-ok-symbolic",
        51..=75 => "network-wireless-signal-good-symbolic",
        _ => "network-wireless-signal-excellent-symbolic",
    }
}

/// Helper to convert security type to display string
pub fn security_display(security: WifiSecurity) -> &'static str {
    match security {
        WifiSecurity::None => "Aberta",
        WifiSecurity::WEP => "WEP",
        WifiSecurity::WPA => "WPA",
        WifiSecurity::WPA2 => "WPA2",
        WifiSecurity::WPA3 => "WPA3",
        WifiSecurity::Enterprise => "Enterprise",
    }
}
