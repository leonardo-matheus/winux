//! Printer data structures

use serde::{Deserialize, Serialize};

/// Printer status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrinterStatus {
    /// Printer is ready to accept jobs
    Ready,
    /// Printer is currently printing
    Printing,
    /// Printer is offline/unreachable
    Offline,
    /// Printer has an error
    Error(String),
    /// Printer is paused/disabled
    Paused,
}

impl PrinterStatus {
    /// Check if the printer can accept new jobs
    pub fn can_accept_jobs(&self) -> bool {
        matches!(self, PrinterStatus::Ready | PrinterStatus::Printing)
    }

    /// Get a human-readable status string
    pub fn display_string(&self) -> &str {
        match self {
            PrinterStatus::Ready => "Pronta",
            PrinterStatus::Printing => "Imprimindo",
            PrinterStatus::Offline => "Offline",
            PrinterStatus::Error(_) => "Erro",
            PrinterStatus::Paused => "Pausada",
        }
    }

    /// Get an icon name for this status
    pub fn icon_name(&self) -> &str {
        match self {
            PrinterStatus::Ready => "emblem-ok-symbolic",
            PrinterStatus::Printing => "printer-printing-symbolic",
            PrinterStatus::Offline => "network-offline-symbolic",
            PrinterStatus::Error(_) => "dialog-warning-symbolic",
            PrinterStatus::Paused => "media-playback-pause-symbolic",
        }
    }
}

/// Connection type for printers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Internet Printing Protocol
    Ipp,
    /// IPP Everywhere (driverless)
    IppEverywhere,
    /// Line Printer Daemon protocol
    Lpd,
    /// Raw socket (JetDirect/AppSocket)
    Socket,
    /// USB connection
    Usb,
    /// DNS Service Discovery (Bonjour/Avahi)
    Dnssd,
    /// Windows SMB/CIFS sharing
    Smb,
}

impl ConnectionType {
    /// Get display name for the connection type
    pub fn display_name(&self) -> &str {
        match self {
            ConnectionType::Ipp => "IPP",
            ConnectionType::IppEverywhere => "IPP Everywhere",
            ConnectionType::Lpd => "LPD",
            ConnectionType::Socket => "Socket/JetDirect",
            ConnectionType::Usb => "USB",
            ConnectionType::Dnssd => "DNS-SD/Bonjour",
            ConnectionType::Smb => "Windows (SMB)",
        }
    }

    /// Get the default port for this connection type
    pub fn default_port(&self) -> u16 {
        match self {
            ConnectionType::Ipp | ConnectionType::IppEverywhere => 631,
            ConnectionType::Lpd => 515,
            ConnectionType::Socket => 9100,
            ConnectionType::Usb => 0,
            ConnectionType::Dnssd => 631,
            ConnectionType::Smb => 445,
        }
    }

    /// Get URI scheme for this connection type
    pub fn uri_scheme(&self) -> &str {
        match self {
            ConnectionType::Ipp => "ipp",
            ConnectionType::IppEverywhere => "ipps",
            ConnectionType::Lpd => "lpd",
            ConnectionType::Socket => "socket",
            ConnectionType::Usb => "usb",
            ConnectionType::Dnssd => "dnssd",
            ConnectionType::Smb => "smb",
        }
    }

    /// Parse connection type from URI
    pub fn from_uri(uri: &str) -> Option<Self> {
        if uri.starts_with("ipp://") {
            Some(ConnectionType::Ipp)
        } else if uri.starts_with("ipps://") {
            Some(ConnectionType::IppEverywhere)
        } else if uri.starts_with("lpd://") {
            Some(ConnectionType::Lpd)
        } else if uri.starts_with("socket://") {
            Some(ConnectionType::Socket)
        } else if uri.starts_with("usb://") {
            Some(ConnectionType::Usb)
        } else if uri.starts_with("dnssd://") {
            Some(ConnectionType::Dnssd)
        } else if uri.starts_with("smb://") {
            Some(ConnectionType::Smb)
        } else {
            None
        }
    }
}

/// Configured printer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Printer {
    /// Internal name (used for CUPS commands)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Device URI
    pub uri: String,
    /// Current status
    pub status: PrinterStatus,
    /// Whether the printer is enabled
    pub enabled: bool,
    /// Whether this is the default printer
    pub is_default: bool,
    /// Printer location
    pub location: Option<String>,
    /// Make and model
    pub make_model: Option<String>,
    /// Whether the printer is shared
    pub shared: bool,
    /// Capabilities
    pub capabilities: Option<PrinterCapabilities>,
}

impl Printer {
    /// Create a new printer
    pub fn new(
        name: &str,
        description: &str,
        uri: &str,
        status: PrinterStatus,
        enabled: bool,
        is_default: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            uri: uri.to_string(),
            status,
            enabled,
            is_default,
            location: None,
            make_model: None,
            shared: false,
            capabilities: None,
        }
    }

    /// Get the connection type from the URI
    pub fn connection_type(&self) -> Option<ConnectionType> {
        ConnectionType::from_uri(&self.uri)
    }

    /// Check if this is a network printer
    pub fn is_network_printer(&self) -> bool {
        matches!(
            self.connection_type(),
            Some(ConnectionType::Ipp)
                | Some(ConnectionType::IppEverywhere)
                | Some(ConnectionType::Lpd)
                | Some(ConnectionType::Socket)
                | Some(ConnectionType::Dnssd)
                | Some(ConnectionType::Smb)
        )
    }

    /// Check if this is a USB printer
    pub fn is_usb_printer(&self) -> bool {
        matches!(self.connection_type(), Some(ConnectionType::Usb))
    }

    /// Check if printer supports driverless printing
    pub fn supports_driverless(&self) -> bool {
        matches!(
            self.connection_type(),
            Some(ConnectionType::IppEverywhere) | Some(ConnectionType::Dnssd)
        )
    }
}

/// Printer discovered on the network (not yet configured)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrinter {
    /// Printer name/model
    pub name: String,
    /// Address or hostname
    pub address: String,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Manufacturer if known
    pub manufacturer: Option<String>,
    /// Raw URI
    pub uri: Option<String>,
    /// Whether driverless printing is supported
    pub driverless: bool,
}

impl DiscoveredPrinter {
    /// Create a new discovered printer
    pub fn new(
        name: &str,
        address: &str,
        connection_type: ConnectionType,
        manufacturer: Option<&str>,
    ) -> Self {
        let driverless = matches!(
            connection_type,
            ConnectionType::IppEverywhere | ConnectionType::Dnssd
        );

        Self {
            name: name.to_string(),
            address: address.to_string(),
            connection_type,
            manufacturer: manufacturer.map(|s| s.to_string()),
            uri: None,
            driverless,
        }
    }

    /// Parse a discovered printer from a URI
    pub fn from_uri(uri: &str) -> Option<Self> {
        let connection_type = ConnectionType::from_uri(uri)?;

        // Extract name from URI (typically URL-encoded)
        let name = if uri.contains("://") {
            uri.split("://")
                .nth(1)?
                .split('/')
                .next()?
                .replace("%20", " ")
                .split('.')
                .next()?
                .to_string()
        } else {
            return None;
        };

        // Extract address
        let address = if uri.contains("://") {
            uri.split("://").nth(1)?.split('/').next()?.to_string()
        } else {
            return None;
        };

        Some(Self {
            name,
            address: address.clone(),
            connection_type,
            manufacturer: None,
            uri: Some(uri.to_string()),
            driverless: matches!(
                connection_type,
                ConnectionType::IppEverywhere | ConnectionType::Dnssd
            ),
        })
    }

    /// Build a full URI for this printer
    pub fn build_uri(&self) -> String {
        if let Some(uri) = &self.uri {
            return uri.clone();
        }

        match self.connection_type {
            ConnectionType::Ipp => {
                format!("ipp://{}/ipp/print", self.address)
            }
            ConnectionType::IppEverywhere => {
                format!("ipps://{}/ipp/print", self.address)
            }
            ConnectionType::Lpd => {
                format!("lpd://{}/", self.address)
            }
            ConnectionType::Socket => {
                format!("socket://{}", self.address)
            }
            ConnectionType::Dnssd => {
                format!("dnssd://{}._ipp._tcp.local", self.name.replace(' ', "%20"))
            }
            ConnectionType::Smb => {
                format!("smb://{}/{}", self.address, self.name.replace(' ', "%20"))
            }
            ConnectionType::Usb => {
                format!("usb://{}", self.address)
            }
        }
    }
}

/// Printer capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrinterCapabilities {
    /// Supported paper sizes
    pub paper_sizes: Vec<String>,
    /// Supported resolutions (DPI)
    pub resolutions: Vec<String>,
    /// Supports color printing
    pub color: bool,
    /// Supports duplex (double-sided) printing
    pub duplex: bool,
    /// Supported input trays
    pub input_trays: Vec<String>,
    /// Supported output bins
    pub output_bins: Vec<String>,
    /// Supports stapling
    pub staple: bool,
    /// Supports hole punching
    pub punch: bool,
    /// Supports collating
    pub collate: bool,
    /// Maximum copies
    pub max_copies: u32,
}

impl PrinterCapabilities {
    /// Create capabilities with common defaults
    pub fn standard() -> Self {
        Self {
            paper_sizes: vec![
                "A4".to_string(),
                "Letter".to_string(),
                "Legal".to_string(),
                "A3".to_string(),
                "A5".to_string(),
            ],
            resolutions: vec![
                "300dpi".to_string(),
                "600dpi".to_string(),
                "1200dpi".to_string(),
            ],
            color: true,
            duplex: true,
            input_trays: vec!["Auto".to_string(), "Tray1".to_string(), "Manual".to_string()],
            output_bins: vec!["Default".to_string()],
            staple: false,
            punch: false,
            collate: true,
            max_copies: 999,
        }
    }
}
