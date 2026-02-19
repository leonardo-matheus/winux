//! Storage and persistence for clipboard history

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use std::path::Path;

use crate::config::Config;
use crate::history::ClipboardHistory;

/// Storage manager for clipboard history
pub struct Storage {
    config: Config,
    encryption_key: Option<[u8; 32]>,
}

impl Storage {
    /// Create a new storage manager
    pub fn new(config: Config) -> Self {
        Self {
            config,
            encryption_key: None,
        }
    }

    /// Initialize storage with optional encryption
    pub fn init(&mut self) -> Result<()> {
        if self.config.encrypt_history {
            self.encryption_key = Some(self.derive_key()?);
        }
        Ok(())
    }

    /// Derive encryption key from machine-specific data
    fn derive_key(&self) -> Result<[u8; 32]> {
        // Use machine ID and username as base for key derivation
        let machine_id = self.get_machine_id()?;
        let username = std::env::var("USER").unwrap_or_else(|_| "default".to_string());

        let password = format!("{}:{}", machine_id, username);
        let salt = SaltString::from_b64("winuxclipboard2024")
            .map_err(|e| anyhow::anyhow!("Invalid salt: {}", e))?;

        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash_bytes = hash.hash.context("No hash output")?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

        Ok(key)
    }

    /// Get machine ID for key derivation
    fn get_machine_id(&self) -> Result<String> {
        // Try to read machine ID from standard locations
        let paths = [
            "/etc/machine-id",
            "/var/lib/dbus/machine-id",
        ];

        for path in paths {
            if let Ok(id) = std::fs::read_to_string(path) {
                return Ok(id.trim().to_string());
            }
        }

        // Fallback to hostname
        Ok(hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "winux-default".to_string()))
    }

    /// Encrypt data
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.encryption_key.context("Encryption not initialized")?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Combine nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// Decrypt data
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.encryption_key.context("Encryption not initialized")?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        if data.len() < 12 {
            anyhow::bail!("Data too short for decryption");
        }

        // Split nonce and ciphertext
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        // Decrypt
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
    }

    /// Load history from disk
    pub fn load_history(&self) -> Result<ClipboardHistory> {
        let path = Config::history_path()?;

        if !path.exists() {
            return Ok(ClipboardHistory::new(self.config.max_history));
        }

        let data = std::fs::read(&path).context("Failed to read history file")?;

        let json = if self.config.encrypt_history && self.encryption_key.is_some() {
            // Decode base64 and decrypt
            let encrypted = BASE64
                .decode(&data)
                .context("Failed to decode history")?;
            let decrypted = self.decrypt(&encrypted)?;
            String::from_utf8(decrypted).context("Invalid UTF-8 in history")?
        } else {
            String::from_utf8(data).context("Invalid UTF-8 in history")?
        };

        let mut history: ClipboardHistory =
            serde_json::from_str(&json).context("Failed to parse history")?;

        history.set_max_items(self.config.max_history);

        Ok(history)
    }

    /// Save history to disk
    pub fn save_history(&self, history: &ClipboardHistory) -> Result<()> {
        let path = Config::history_path()?;
        let json = serde_json::to_string(history).context("Failed to serialize history")?;

        let data = if self.config.encrypt_history && self.encryption_key.is_some() {
            // Encrypt and encode to base64
            let encrypted = self.encrypt(json.as_bytes())?;
            BASE64.encode(encrypted).into_bytes()
        } else {
            json.into_bytes()
        };

        std::fs::write(&path, data).context("Failed to write history file")?;

        Ok(())
    }

    /// Save image to storage
    pub fn save_image(&self, data: &[u8], format: &str) -> Result<String> {
        let images_dir = Config::images_dir()?;

        // Generate unique filename
        let timestamp = chrono::Utc::now().timestamp_millis();
        let mut random = [0u8; 4];
        rand::thread_rng().fill_bytes(&mut random);
        let filename = format!(
            "{}_{}.{}",
            timestamp,
            hex::encode(random),
            format
        );

        let path = images_dir.join(&filename);
        std::fs::write(&path, data).context("Failed to save image")?;

        Ok(path.to_string_lossy().to_string())
    }

    /// Load image from storage
    pub fn load_image(&self, path: &str) -> Result<Vec<u8>> {
        std::fs::read(path).context("Failed to load image")
    }

    /// Delete image from storage
    pub fn delete_image(&self, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            std::fs::remove_file(path).context("Failed to delete image")?;
        }
        Ok(())
    }

    /// Clear all stored data
    pub fn clear_all(&self) -> Result<()> {
        // Clear history file
        let history_path = Config::history_path()?;
        if history_path.exists() {
            std::fs::remove_file(&history_path).context("Failed to delete history")?;
        }

        // Clear images directory
        let images_dir = Config::images_dir()?;
        if images_dir.exists() {
            for entry in std::fs::read_dir(&images_dir)? {
                let entry = entry?;
                if entry.path().is_file() {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        Ok(())
    }

    /// Export history to file
    pub fn export_history(&self, history: &ClipboardHistory, path: &Path) -> Result<()> {
        let json = history.export_json();
        std::fs::write(path, json).context("Failed to export history")?;
        Ok(())
    }

    /// Get storage statistics
    pub fn storage_stats(&self) -> Result<StorageStats> {
        let history_path = Config::history_path()?;
        let images_dir = Config::images_dir()?;

        let history_size = if history_path.exists() {
            std::fs::metadata(&history_path)?.len() as usize
        } else {
            0
        };

        let mut images_size = 0usize;
        let mut image_count = 0usize;

        if images_dir.exists() {
            for entry in std::fs::read_dir(&images_dir)? {
                let entry = entry?;
                if entry.path().is_file() {
                    images_size += entry.metadata()?.len() as usize;
                    image_count += 1;
                }
            }
        }

        Ok(StorageStats {
            history_size,
            images_size,
            image_count,
            total_size: history_size + images_size,
            encrypted: self.config.encrypt_history,
        })
    }
}

/// Storage statistics
#[derive(Debug)]
pub struct StorageStats {
    pub history_size: usize,
    pub images_size: usize,
    pub image_count: usize,
    pub total_size: usize,
    pub encrypted: bool,
}

impl StorageStats {
    pub fn format_total_size(&self) -> String {
        format_size(self.total_size)
    }

    pub fn format_history_size(&self) -> String {
        format_size(self.history_size)
    }

    pub fn format_images_size(&self) -> String {
        format_size(self.images_size)
    }
}

fn format_size(size: usize) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    }
}

// Add hex encoding since we use it
mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// Add hostname fallback
mod hostname {
    use std::ffi::OsString;

    pub fn get() -> std::io::Result<OsString> {
        std::fs::read_to_string("/etc/hostname")
            .map(|s| OsString::from(s.trim()))
            .or_else(|_| Ok(OsString::from("localhost")))
    }
}
