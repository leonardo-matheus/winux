//! Bluetooth device representation

use serde::{Deserialize, Serialize};

/// Type of Bluetooth device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Headphones or earbuds
    Headphones,
    /// Speaker or soundbar
    Speaker,
    /// Keyboard
    Keyboard,
    /// Mouse or trackpad
    Mouse,
    /// Game controller
    Gamepad,
    /// Mobile phone
    Phone,
    /// Tablet
    Tablet,
    /// Computer or laptop
    Computer,
    /// Smartwatch
    Watch,
    /// Printer
    Printer,
    /// Camera
    Camera,
    /// Unknown device type
    Unknown,
}

impl DeviceType {
    /// Get the icon name for this device type
    pub fn icon_name(&self) -> &'static str {
        match self {
            DeviceType::Headphones => "audio-headphones-symbolic",
            DeviceType::Speaker => "audio-speakers-symbolic",
            DeviceType::Keyboard => "input-keyboard-symbolic",
            DeviceType::Mouse => "input-mouse-symbolic",
            DeviceType::Gamepad => "input-gaming-symbolic",
            DeviceType::Phone => "phone-symbolic",
            DeviceType::Tablet => "tablet-symbolic",
            DeviceType::Computer => "computer-symbolic",
            DeviceType::Watch => "smartwatch-symbolic",
            DeviceType::Printer => "printer-symbolic",
            DeviceType::Camera => "camera-photo-symbolic",
            DeviceType::Unknown => "bluetooth-symbolic",
        }
    }

    /// Get display name for this device type
    pub fn display_name(&self) -> &'static str {
        match self {
            DeviceType::Headphones => "Fones de Ouvido",
            DeviceType::Speaker => "Alto-falante",
            DeviceType::Keyboard => "Teclado",
            DeviceType::Mouse => "Mouse",
            DeviceType::Gamepad => "Controle de Jogo",
            DeviceType::Phone => "Telefone",
            DeviceType::Tablet => "Tablet",
            DeviceType::Computer => "Computador",
            DeviceType::Watch => "Relogio",
            DeviceType::Printer => "Impressora",
            DeviceType::Camera => "Camera",
            DeviceType::Unknown => "Dispositivo",
        }
    }

    /// Parse device type from BlueZ device class
    pub fn from_class(class: u32) -> Self {
        // Bluetooth device class parsing
        // Major device class is bits 8-12
        let major_class = (class >> 8) & 0x1F;
        // Minor device class is bits 2-7
        let minor_class = (class >> 2) & 0x3F;

        match major_class {
            0x01 => DeviceType::Computer,
            0x02 => DeviceType::Phone,
            0x04 => {
                // Audio/Video
                match minor_class {
                    0x01 | 0x02 => DeviceType::Headphones, // Wearable headset / Hands-free
                    0x06 => DeviceType::Headphones,        // Headphones
                    0x07 => DeviceType::Speaker,           // Loudspeaker
                    _ => DeviceType::Speaker,
                }
            }
            0x05 => {
                // Peripheral
                match minor_class & 0x30 {
                    0x10 => DeviceType::Keyboard,
                    0x20 => DeviceType::Mouse,
                    0x30 => DeviceType::Keyboard, // Combo keyboard/mouse
                    _ => match minor_class & 0x0F {
                        0x01 => DeviceType::Gamepad,
                        _ => DeviceType::Unknown,
                    },
                }
            }
            0x06 => DeviceType::Printer, // Imaging
            0x07 => DeviceType::Watch,   // Wearable
            _ => DeviceType::Unknown,
        }
    }

    /// Parse device type from device appearance value (GATT)
    pub fn from_appearance(appearance: u16) -> Self {
        let category = appearance >> 6;
        match category {
            0x03 => DeviceType::Watch,    // Watch
            0x0F => DeviceType::Keyboard, // HID
            0x40 => DeviceType::Gamepad,  // Gaming
            0x41 => DeviceType::Headphones, // Pulse Oximeter (usually wearable)
            0xC1..=0xC4 => DeviceType::Headphones, // Audio Sink/Source
            _ => DeviceType::Unknown,
        }
    }
}

/// Connection state of a device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Connection in progress
    Connecting,
    /// Connected
    Connected,
    /// Disconnecting
    Disconnecting,
}

impl ConnectionState {
    pub fn display_name(&self) -> &'static str {
        match self {
            ConnectionState::Disconnected => "Desconectado",
            ConnectionState::Connecting => "Conectando...",
            ConnectionState::Connected => "Conectado",
            ConnectionState::Disconnecting => "Desconectando...",
        }
    }
}

/// Represents a Bluetooth device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    /// MAC address
    pub address: String,
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: DeviceType,
    /// Whether device is paired
    pub paired: bool,
    /// Whether device is connected
    pub connected: bool,
    /// Battery percentage (if available)
    pub battery: Option<u8>,
    /// Signal strength (RSSI)
    pub rssi: Option<i16>,
    /// Device class (BlueZ)
    pub class: Option<u32>,
    /// Supported UUIDs (services)
    pub uuids: Vec<String>,
    /// Whether device is trusted
    pub trusted: bool,
    /// Whether device is blocked
    pub blocked: bool,
    /// Alias (user-defined name)
    pub alias: Option<String>,
    /// D-Bus object path
    pub object_path: Option<String>,
}

impl BluetoothDevice {
    /// Create a new BluetoothDevice
    pub fn new(
        address: &str,
        name: &str,
        device_type: DeviceType,
        paired: bool,
        connected: bool,
        battery: Option<u8>,
    ) -> Self {
        Self {
            address: address.to_string(),
            name: name.to_string(),
            device_type,
            paired,
            connected,
            battery,
            rssi: None,
            class: None,
            uuids: Vec::new(),
            trusted: false,
            blocked: false,
            alias: None,
            object_path: None,
        }
    }

    /// Get display name (alias if set, otherwise name)
    pub fn display_name(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.name)
    }

    /// Check if device supports A2DP (audio)
    pub fn supports_a2dp(&self) -> bool {
        // A2DP Sink UUID
        self.uuids.iter().any(|uuid| {
            uuid.to_lowercase().contains("110b") || // A2DP Sink
            uuid.to_lowercase().contains("110a")    // A2DP Source
        })
    }

    /// Check if device supports HFP (hands-free)
    pub fn supports_hfp(&self) -> bool {
        // HFP UUID
        self.uuids.iter().any(|uuid| {
            uuid.to_lowercase().contains("111e") || // HFP AG
            uuid.to_lowercase().contains("111f")    // HFP HF
        })
    }

    /// Check if device supports OBEX (file transfer)
    pub fn supports_obex(&self) -> bool {
        self.uuids.iter().any(|uuid| {
            uuid.to_lowercase().contains("1105") || // OBEXObjectPush
            uuid.to_lowercase().contains("1106")    // OBEXFileTransfer
        })
    }

    /// Check if device is an input device (HID)
    pub fn is_input_device(&self) -> bool {
        matches!(
            self.device_type,
            DeviceType::Keyboard | DeviceType::Mouse | DeviceType::Gamepad
        )
    }

    /// Check if device is an audio device
    pub fn is_audio_device(&self) -> bool {
        matches!(
            self.device_type,
            DeviceType::Headphones | DeviceType::Speaker
        )
    }
}

impl Default for BluetoothDevice {
    fn default() -> Self {
        Self {
            address: String::new(),
            name: "Unknown Device".to_string(),
            device_type: DeviceType::Unknown,
            paired: false,
            connected: false,
            battery: None,
            rssi: None,
            class: None,
            uuids: Vec::new(),
            trusted: false,
            blocked: false,
            alias: None,
            object_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_from_class() {
        // Computer
        assert_eq!(DeviceType::from_class(0x100), DeviceType::Computer);
        // Phone
        assert_eq!(DeviceType::from_class(0x200), DeviceType::Phone);
        // Headphones (Audio, Headphones minor class)
        assert_eq!(DeviceType::from_class(0x418), DeviceType::Headphones);
    }

    #[test]
    fn test_device_creation() {
        let device = BluetoothDevice::new(
            "00:11:22:33:44:55",
            "Test Device",
            DeviceType::Headphones,
            true,
            false,
            Some(75),
        );

        assert_eq!(device.address, "00:11:22:33:44:55");
        assert_eq!(device.name, "Test Device");
        assert!(device.paired);
        assert!(!device.connected);
        assert_eq!(device.battery, Some(75));
    }
}
