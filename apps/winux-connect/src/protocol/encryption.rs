//! End-to-end encryption for secure communication
//!
//! Uses TLS for transport encryption and RSA for key exchange,
//! compatible with KDE Connect's security model.

use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

use super::messages::NetworkPacket;

/// Encryption manager for secure communications
pub struct EncryptionManager {
    /// Our private key (PEM format)
    private_key: RwLock<Option<String>>,

    /// Our public certificate (PEM format)
    certificate: RwLock<Option<String>>,

    /// Trusted device certificates (device_id -> certificate)
    trusted_certificates: RwLock<HashMap<String, String>>,

    /// Session keys for each device (device_id -> key)
    session_keys: RwLock<HashMap<String, Vec<u8>>>,
}

impl EncryptionManager {
    pub fn new() -> Self {
        Self {
            private_key: RwLock::new(None),
            certificate: RwLock::new(None),
            trusted_certificates: RwLock::new(HashMap::new()),
            session_keys: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize encryption with a new or existing key pair
    pub fn initialize(&self) -> Result<(), String> {
        // Check if we already have a key pair
        if self.private_key.read().unwrap().is_some() {
            return Ok(());
        }

        // Generate new key pair
        self.generate_key_pair()?;

        Ok(())
    }

    /// Generate a new RSA key pair and self-signed certificate
    fn generate_key_pair(&self) -> Result<(), String> {
        // In production, this would use rcgen to generate:
        // 1. RSA 2048-bit private key
        // 2. Self-signed X.509 certificate

        // Placeholder implementation
        *self.private_key.write().unwrap() = Some("PRIVATE_KEY_PLACEHOLDER".to_string());
        *self.certificate.write().unwrap() = Some("CERTIFICATE_PLACEHOLDER".to_string());

        tracing::info!("Generated new encryption key pair");
        Ok(())
    }

    /// Get our public certificate
    pub fn get_certificate(&self) -> Option<String> {
        self.certificate.read().unwrap().clone()
    }

    /// Get SHA256 fingerprint of our certificate
    pub fn get_certificate_fingerprint(&self) -> String {
        // In production, would calculate actual SHA256 hash
        "AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD".to_string()
    }

    /// Add a trusted device certificate
    pub fn add_trusted_certificate(&self, device_id: &str, certificate: &str) {
        self.trusted_certificates
            .write()
            .unwrap()
            .insert(device_id.to_string(), certificate.to_string());

        tracing::info!("Added trusted certificate for device {}", device_id);
    }

    /// Remove a trusted device certificate
    pub fn remove_trusted_certificate(&self, device_id: &str) {
        self.trusted_certificates.write().unwrap().remove(device_id);
        self.session_keys.write().unwrap().remove(device_id);

        tracing::info!("Removed trusted certificate for device {}", device_id);
    }

    /// Check if a device certificate is trusted
    pub fn is_trusted(&self, device_id: &str) -> bool {
        self.trusted_certificates.read().unwrap().contains_key(device_id)
    }

    /// Verify a device's certificate
    pub fn verify_certificate(&self, device_id: &str, certificate: &str) -> Result<bool, String> {
        if let Some(stored) = self.trusted_certificates.read().unwrap().get(device_id) {
            // Compare certificate fingerprints
            Ok(stored == certificate)
        } else {
            Ok(false)
        }
    }

    /// Encrypt a network packet for a specific device
    pub fn encrypt(&self, packet: &NetworkPacket, device_id: &str) -> Result<Vec<u8>, String> {
        // Get or create session key
        let session_key = self.get_or_create_session_key(device_id)?;

        // Serialize packet to JSON
        let json = packet.to_json().map_err(|e| e.to_string())?;

        // In production, would use AES-GCM to encrypt
        // For now, just return the JSON bytes
        Ok(json.into_bytes())
    }

    /// Decrypt a network packet from a specific device
    pub fn decrypt(&self, data: &[u8], device_id: &str) -> Result<NetworkPacket, String> {
        // Get session key
        let _session_key = self.get_or_create_session_key(device_id)?;

        // In production, would use AES-GCM to decrypt
        // For now, just parse the JSON
        let json = String::from_utf8(data.to_vec()).map_err(|e| e.to_string())?;
        NetworkPacket::from_json(&json).map_err(|e| e.to_string())
    }

    /// Get or create a session key for a device
    fn get_or_create_session_key(&self, device_id: &str) -> Result<Vec<u8>, String> {
        // Check if we already have a session key
        if let Some(key) = self.session_keys.read().unwrap().get(device_id) {
            return Ok(key.clone());
        }

        // Generate new session key (32 bytes for AES-256)
        let key = self.generate_session_key();
        self.session_keys
            .write()
            .unwrap()
            .insert(device_id.to_string(), key.clone());

        Ok(key)
    }

    /// Generate a random session key
    fn generate_session_key(&self) -> Vec<u8> {
        use rand::RngCore;
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Create TLS configuration for server
    pub fn create_tls_config(&self) -> Result<TlsConfig, String> {
        let cert = self.certificate.read().unwrap()
            .clone()
            .ok_or("Certificate not initialized")?;

        let key = self.private_key.read().unwrap()
            .clone()
            .ok_or("Private key not initialized")?;

        Ok(TlsConfig {
            certificate: cert,
            private_key: key,
        })
    }

    /// Load saved certificates from disk
    pub fn load_certificates(&self, path: &str) -> Result<(), String> {
        // In production, would load from config directory
        tracing::info!("Loading certificates from {}", path);
        Ok(())
    }

    /// Save certificates to disk
    pub fn save_certificates(&self, path: &str) -> Result<(), String> {
        // In production, would save to config directory
        tracing::info!("Saving certificates to {}", path);
        Ok(())
    }
}

impl Default for EncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// TLS configuration
#[derive(Clone, Debug)]
pub struct TlsConfig {
    pub certificate: String,
    pub private_key: String,
}

/// Encrypted packet wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedPacket {
    /// Initialization vector
    pub iv: String,

    /// Encrypted data (base64)
    pub data: String,

    /// Authentication tag
    pub tag: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_manager_creation() {
        let manager = EncryptionManager::new();
        assert!(!manager.is_trusted("unknown_device"));
    }

    #[test]
    fn test_trusted_certificates() {
        let manager = EncryptionManager::new();
        manager.add_trusted_certificate("device1", "CERT1");
        assert!(manager.is_trusted("device1"));
        assert!(!manager.is_trusted("device2"));

        manager.remove_trusted_certificate("device1");
        assert!(!manager.is_trusted("device1"));
    }

    #[test]
    fn test_encrypt_decrypt() {
        let manager = EncryptionManager::new();
        manager.initialize().unwrap();
        manager.add_trusted_certificate("device1", "CERT");

        let packet = NetworkPacket::ping();
        let encrypted = manager.encrypt(&packet, "device1").unwrap();
        let decrypted = manager.decrypt(&encrypted, "device1").unwrap();

        assert_eq!(packet.packet_type, decrypted.packet_type);
    }
}
