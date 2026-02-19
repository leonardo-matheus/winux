//! Upload functionality for sharing screenshots

use anyhow::{Result, anyhow};
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Supported upload services
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadService {
    Imgur,
    // Future services can be added here
}

impl UploadService {
    pub fn name(&self) -> &'static str {
        match self {
            UploadService::Imgur => "Imgur",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            UploadService::Imgur => "web-browser-symbolic",
        }
    }

    pub fn all() -> &'static [UploadService] {
        &[UploadService::Imgur]
    }
}

/// Result of an upload operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub url: String,
    pub delete_url: Option<String>,
    pub thumbnail_url: Option<String>,
}

/// Imgur API response
#[derive(Debug, Deserialize)]
struct ImgurResponse {
    data: ImgurData,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct ImgurData {
    link: String,
    deletehash: Option<String>,
}

/// Upload an image to a service
pub fn upload_image(path: &Path, service: UploadService) -> Result<UploadResult> {
    match service {
        UploadService::Imgur => upload_to_imgur(path),
    }
}

/// Upload to Imgur anonymously
fn upload_to_imgur(path: &Path) -> Result<UploadResult> {
    use std::process::Command;

    // Read image as base64
    let image_data = std::fs::read(path)?;
    let base64_data = base64_encode(&image_data);

    // Imgur anonymous client ID (public, rate-limited)
    let client_id = "3e7a4deb7ac67da";

    // Use curl for the upload
    let output = Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg(format!("Authorization: Client-ID {}", client_id))
        .arg("-F")
        .arg(format!("image={}", base64_data))
        .arg("https://api.imgur.com/3/image")
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Imgur upload failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let response: ImgurResponse = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("Failed to parse Imgur response: {}", e))?;

    if !response.success {
        return Err(anyhow!("Imgur returned error"));
    }

    let delete_url = response.data.deletehash.map(|hash| {
        format!("https://imgur.com/delete/{}", hash)
    });

    Ok(UploadResult {
        url: response.data.link,
        delete_url,
        thumbnail_url: None,
    })
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

/// Upload history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadHistoryEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub service: String,
    pub url: String,
    pub delete_url: Option<String>,
    pub local_path: Option<String>,
}

/// Upload history manager
pub struct UploadHistory {
    entries: Vec<UploadHistoryEntry>,
    history_file: std::path::PathBuf,
}

impl UploadHistory {
    /// Load or create upload history
    pub fn load() -> Self {
        let history_file = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("winux-screenshot")
            .join("upload_history.json");

        let entries = if history_file.exists() {
            std::fs::read_to_string(&history_file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            entries,
            history_file,
        }
    }

    /// Add an entry to history
    pub fn add(&mut self, entry: UploadHistoryEntry) {
        self.entries.push(entry);
        self.save();
    }

    /// Get all entries
    pub fn entries(&self) -> &[UploadHistoryEntry] {
        &self.entries
    }

    /// Save history to disk
    fn save(&self) {
        if let Some(parent) = self.history_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
            std::fs::write(&self.history_file, json).ok();
        }
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }
}
