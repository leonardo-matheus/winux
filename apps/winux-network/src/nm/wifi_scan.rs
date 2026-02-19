//! WiFi network scanning
//!
//! Provides async WiFi scanning with real-time updates

use super::{AccessPoint, NetworkManagerClient, NetworkResult, WifiSecurity};
use async_channel::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// WiFi scanner events
#[derive(Debug, Clone)]
pub enum ScanEvent {
    /// Scan started
    Started,
    /// New access points found
    AccessPointsUpdated(Vec<AccessPoint>),
    /// Scan completed
    Completed,
    /// Scan failed
    Failed(String),
}

/// WiFi scanner for continuous or one-shot scanning
pub struct WifiScanner {
    client: NetworkManagerClient,
    access_points: Arc<RwLock<Vec<AccessPoint>>>,
    is_scanning: Arc<RwLock<bool>>,
}

impl WifiScanner {
    /// Create a new WiFi scanner
    pub async fn new() -> NetworkResult<Self> {
        let client = NetworkManagerClient::new().await?;
        Ok(Self {
            client,
            access_points: Arc::new(RwLock::new(Vec::new())),
            is_scanning: Arc::new(RwLock::new(false)),
        })
    }

    /// Perform a single WiFi scan
    pub async fn scan_once(&self) -> NetworkResult<Vec<AccessPoint>> {
        {
            let mut scanning = self.is_scanning.write().await;
            if *scanning {
                // Already scanning, return cached results
                return Ok(self.access_points.read().await.clone());
            }
            *scanning = true;
        }

        info!("Starting WiFi scan...");

        // Request scan from NetworkManager
        self.client.request_wifi_scan().await?;

        // Wait for scan to complete (in real implementation, listen for StateChanged signal)
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Get access points
        let aps = self.client.get_access_points().await?;

        // Update cached list
        {
            let mut cached = self.access_points.write().await;
            *cached = aps.clone();
        }

        {
            let mut scanning = self.is_scanning.write().await;
            *scanning = false;
        }

        info!("WiFi scan completed. Found {} networks.", aps.len());
        Ok(aps)
    }

    /// Start continuous scanning with event channel
    pub fn scan_continuous(&self) -> Receiver<ScanEvent> {
        let (tx, rx) = async_channel::bounded(10);

        let client = self.client.clone();
        let access_points = Arc::clone(&self.access_points);
        let is_scanning = Arc::clone(&self.is_scanning);

        tokio::spawn(async move {
            loop {
                // Check if we should continue scanning
                {
                    let scanning = is_scanning.read().await;
                    if !*scanning {
                        break;
                    }
                }

                let _ = tx.send(ScanEvent::Started).await;

                // Request scan
                if let Err(e) = client.request_wifi_scan().await {
                    let _ = tx.send(ScanEvent::Failed(e.to_string())).await;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }

                // Wait for scan
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Get results
                match client.get_access_points().await {
                    Ok(aps) => {
                        // Update cache
                        {
                            let mut cached = access_points.write().await;
                            *cached = aps.clone();
                        }
                        let _ = tx.send(ScanEvent::AccessPointsUpdated(aps)).await;
                        let _ = tx.send(ScanEvent::Completed).await;
                    }
                    Err(e) => {
                        let _ = tx.send(ScanEvent::Failed(e.to_string())).await;
                    }
                }

                // Wait before next scan
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });

        rx
    }

    /// Start continuous scanning
    pub async fn start_continuous(&self) {
        let mut scanning = self.is_scanning.write().await;
        *scanning = true;
    }

    /// Stop continuous scanning
    pub async fn stop_continuous(&self) {
        let mut scanning = self.is_scanning.write().await;
        *scanning = false;
    }

    /// Get cached access points
    pub async fn get_cached(&self) -> Vec<AccessPoint> {
        self.access_points.read().await.clone()
    }

    /// Check if currently scanning
    pub async fn is_scanning(&self) -> bool {
        *self.is_scanning.read().await
    }

    /// Find access point by SSID
    pub async fn find_by_ssid(&self, ssid: &str) -> Option<AccessPoint> {
        self.access_points
            .read()
            .await
            .iter()
            .find(|ap| ap.ssid == ssid)
            .cloned()
    }

    /// Get access points sorted by signal strength
    pub async fn get_sorted_by_signal(&self) -> Vec<AccessPoint> {
        let mut aps = self.access_points.read().await.clone();
        aps.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
        aps
    }

    /// Get only secured networks
    pub async fn get_secured_only(&self) -> Vec<AccessPoint> {
        self.access_points
            .read()
            .await
            .iter()
            .filter(|ap| ap.security != WifiSecurity::None)
            .cloned()
            .collect()
    }

    /// Get only open networks
    pub async fn get_open_only(&self) -> Vec<AccessPoint> {
        self.access_points
            .read()
            .await
            .iter()
            .filter(|ap| ap.security == WifiSecurity::None)
            .cloned()
            .collect()
    }
}

/// Parse SSID from raw bytes
pub fn parse_ssid(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).to_string()
}

/// Parse security flags from NM flags
pub fn parse_security(flags: u32, wpa_flags: u32, rsn_flags: u32) -> WifiSecurity {
    if rsn_flags != 0 {
        // Check for WPA3
        if rsn_flags & 0x200 != 0 {
            return WifiSecurity::WPA3;
        }
        return WifiSecurity::WPA2;
    }

    if wpa_flags != 0 {
        return WifiSecurity::WPA;
    }

    if flags & 0x1 != 0 {
        return WifiSecurity::WEP;
    }

    WifiSecurity::None
}

/// Calculate channel from frequency
pub fn frequency_to_channel(freq: u32) -> u32 {
    if freq >= 2412 && freq <= 2484 {
        // 2.4 GHz band
        if freq == 2484 {
            14
        } else {
            (freq - 2407) / 5
        }
    } else if freq >= 5170 && freq <= 5825 {
        // 5 GHz band
        (freq - 5000) / 5
    } else if freq >= 5955 && freq <= 7115 {
        // 6 GHz band (Wi-Fi 6E)
        (freq - 5950) / 5
    } else {
        0
    }
}

/// Get band name from frequency
pub fn frequency_to_band(freq: u32) -> &'static str {
    if freq >= 2412 && freq <= 2484 {
        "2.4 GHz"
    } else if freq >= 5170 && freq <= 5825 {
        "5 GHz"
    } else if freq >= 5955 && freq <= 7115 {
        "6 GHz"
    } else {
        "Unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_to_channel() {
        assert_eq!(frequency_to_channel(2412), 1);
        assert_eq!(frequency_to_channel(2437), 6);
        assert_eq!(frequency_to_channel(2462), 11);
        assert_eq!(frequency_to_channel(5180), 36);
        assert_eq!(frequency_to_channel(5240), 48);
    }

    #[test]
    fn test_frequency_to_band() {
        assert_eq!(frequency_to_band(2437), "2.4 GHz");
        assert_eq!(frequency_to_band(5180), "5 GHz");
        assert_eq!(frequency_to_band(6115), "6 GHz");
    }

    #[test]
    fn test_parse_security() {
        assert_eq!(parse_security(0, 0, 0), WifiSecurity::None);
        assert_eq!(parse_security(1, 0, 0), WifiSecurity::WEP);
        assert_eq!(parse_security(0, 1, 0), WifiSecurity::WPA);
        assert_eq!(parse_security(0, 0, 1), WifiSecurity::WPA2);
        assert_eq!(parse_security(0, 0, 0x200), WifiSecurity::WPA3);
    }
}
