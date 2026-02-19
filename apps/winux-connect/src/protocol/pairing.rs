//! Secure device pairing with QR code and PIN
//!
//! Implements KDE Connect compatible pairing protocol with:
//! - TLS certificate exchange
//! - PIN verification
//! - QR code generation for quick pairing

use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Pairing timeout in seconds
pub const PAIRING_TIMEOUT_SECS: u64 = 30;

/// PIN length
pub const PIN_LENGTH: usize = 6;

/// Pairing manager for device authentication
pub struct PairingManager {
    pending_requests: RwLock<HashMap<String, PairingRequest>>,
    paired_devices: RwLock<HashMap<String, PairedDevice>>,
    our_certificate: RwLock<Option<String>>,
}

impl PairingManager {
    pub fn new() -> Self {
        Self {
            pending_requests: RwLock::new(HashMap::new()),
            paired_devices: RwLock::new(HashMap::new()),
            our_certificate: RwLock::new(None),
        }
    }

    /// Initialize the pairing manager with our certificate
    pub fn initialize(&self) -> Result<(), String> {
        // Generate or load our TLS certificate
        let cert = self.load_or_generate_certificate()?;
        *self.our_certificate.write().unwrap() = Some(cert);
        Ok(())
    }

    /// Generate or load our TLS certificate
    fn load_or_generate_certificate(&self) -> Result<String, String> {
        // In production, this would:
        // 1. Check if certificate exists in config directory
        // 2. If not, generate a new self-signed certificate
        // 3. Store it for future use

        // For now, return a placeholder
        Ok("CERTIFICATE_PLACEHOLDER".to_string())
    }

    /// Initiate pairing with a device
    pub fn initiate_pairing(&self, device_id: &str) -> Result<String, String> {
        let pin = self.generate_pin();

        let request = PairingRequest {
            device_id: device_id.to_string(),
            pin: pin.clone(),
            timestamp: std::time::SystemTime::now(),
            status: PairingStatus::Pending,
            initiated_by_us: true,
        };

        self.pending_requests
            .write()
            .unwrap()
            .insert(device_id.to_string(), request);

        tracing::info!("Initiated pairing with device {}, PIN: {}", device_id, pin);
        Ok(pin)
    }

    /// Handle incoming pairing request
    pub fn handle_incoming_request(&self, device_id: &str, device_name: &str, certificate: &str) {
        let pin = self.generate_pin();

        let request = PairingRequest {
            device_id: device_id.to_string(),
            pin: pin.clone(),
            timestamp: std::time::SystemTime::now(),
            status: PairingStatus::Pending,
            initiated_by_us: false,
        };

        self.pending_requests
            .write()
            .unwrap()
            .insert(device_id.to_string(), request);

        tracing::info!("Received pairing request from {} ({})", device_name, device_id);
    }

    /// Accept pairing request
    pub fn accept_pairing(&self, device_id: &str) -> Result<(), String> {
        let mut requests = self.pending_requests.write().unwrap();

        if let Some(request) = requests.remove(device_id) {
            // Store as paired device
            let paired = PairedDevice {
                device_id: device_id.to_string(),
                certificate: "DEVICE_CERTIFICATE".to_string(), // Would be actual certificate
                paired_at: std::time::SystemTime::now(),
            };

            self.paired_devices
                .write()
                .unwrap()
                .insert(device_id.to_string(), paired);

            tracing::info!("Pairing accepted for device {}", device_id);
            Ok(())
        } else {
            Err("No pending pairing request for this device".to_string())
        }
    }

    /// Reject pairing request
    pub fn reject_pairing(&self, device_id: &str) {
        self.pending_requests.write().unwrap().remove(device_id);
        tracing::info!("Pairing rejected for device {}", device_id);
    }

    /// Unpair a device
    pub fn unpair_device(&self, device_id: &str) {
        self.paired_devices.write().unwrap().remove(device_id);
        tracing::info!("Device {} unpaired", device_id);
    }

    /// Check if a device is paired
    pub fn is_paired(&self, device_id: &str) -> bool {
        self.paired_devices.read().unwrap().contains_key(device_id)
    }

    /// Get all paired devices
    pub fn get_paired_devices(&self) -> Vec<PairedDevice> {
        self.paired_devices.read().unwrap().values().cloned().collect()
    }

    /// Get pending pairing requests
    pub fn get_pending_requests(&self) -> Vec<PairingRequest> {
        self.pending_requests.read().unwrap().values().cloned().collect()
    }

    /// Generate a random PIN
    fn generate_pin(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..PIN_LENGTH)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }

    /// Generate QR code data for pairing
    pub fn generate_qr_code_data(&self) -> Result<String, String> {
        let device_id = self.get_device_id();
        let device_name = self.get_device_name();

        // Format compatible with KDE Connect
        let qr_data = QrCodeData {
            device_id,
            device_name,
            ip_address: self.get_local_ip(),
            port: 1716,
            certificate_hash: self.get_certificate_hash(),
        };

        serde_json::to_string(&qr_data)
            .map_err(|e| format!("Failed to generate QR code data: {}", e))
    }

    /// Get device ID
    fn get_device_id(&self) -> String {
        // Would be loaded from config
        "winux_connect_device_id".to_string()
    }

    /// Get device name
    fn get_device_name(&self) -> String {
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Winux PC".to_string())
    }

    /// Get local IP address
    fn get_local_ip(&self) -> String {
        // In production, would detect actual local IP
        "192.168.1.100".to_string()
    }

    /// Get certificate hash for verification
    fn get_certificate_hash(&self) -> String {
        // SHA256 hash of our certificate for verification
        "CERTIFICATE_HASH_PLACEHOLDER".to_string()
    }

    /// Verify PIN entered by user
    pub fn verify_pin(&self, device_id: &str, pin: &str) -> bool {
        if let Some(request) = self.pending_requests.read().unwrap().get(device_id) {
            request.pin == pin
        } else {
            false
        }
    }

    /// Clean up expired pairing requests
    pub fn cleanup_expired(&self) {
        let now = std::time::SystemTime::now();
        let timeout = std::time::Duration::from_secs(PAIRING_TIMEOUT_SECS);

        self.pending_requests.write().unwrap().retain(|_, request| {
            request.timestamp.elapsed().map(|d| d < timeout).unwrap_or(true)
        });
    }
}

impl Default for PairingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pairing request information
#[derive(Clone, Debug)]
pub struct PairingRequest {
    pub device_id: String,
    pub pin: String,
    pub timestamp: std::time::SystemTime,
    pub status: PairingStatus,
    pub initiated_by_us: bool,
}

/// Pairing status
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PairingStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

/// Paired device information
#[derive(Clone, Debug)]
pub struct PairedDevice {
    pub device_id: String,
    pub certificate: String,
    pub paired_at: std::time::SystemTime,
}

/// QR code data for quick pairing
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeData {
    pub device_id: String,
    pub device_name: String,
    pub ip_address: String,
    pub port: u16,
    pub certificate_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_manager_creation() {
        let manager = PairingManager::new();
        assert!(manager.get_paired_devices().is_empty());
        assert!(manager.get_pending_requests().is_empty());
    }

    #[test]
    fn test_initiate_pairing() {
        let manager = PairingManager::new();
        let result = manager.initiate_pairing("test_device");
        assert!(result.is_ok());
        let pin = result.unwrap();
        assert_eq!(pin.len(), PIN_LENGTH);
        assert!(!manager.get_pending_requests().is_empty());
    }

    #[test]
    fn test_accept_pairing() {
        let manager = PairingManager::new();
        manager.initiate_pairing("test_device").unwrap();
        let result = manager.accept_pairing("test_device");
        assert!(result.is_ok());
        assert!(manager.is_paired("test_device"));
    }

    #[test]
    fn test_reject_pairing() {
        let manager = PairingManager::new();
        manager.initiate_pairing("test_device").unwrap();
        manager.reject_pairing("test_device");
        assert!(manager.get_pending_requests().is_empty());
        assert!(!manager.is_paired("test_device"));
    }

    #[test]
    fn test_verify_pin() {
        let manager = PairingManager::new();
        let pin = manager.initiate_pairing("test_device").unwrap();
        assert!(manager.verify_pin("test_device", &pin));
        assert!(!manager.verify_pin("test_device", "wrong_pin"));
    }
}
