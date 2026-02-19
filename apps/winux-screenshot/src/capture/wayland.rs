//! Wayland portal (xdg-desktop-portal) screenshot support
//!
//! Uses the org.freedesktop.portal.Screenshot interface for
//! proper Wayland screenshot support with user permission.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static REQUEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Wayland screenshot capture using xdg-desktop-portal
pub struct WaylandCapture {
    connection: zbus::blocking::Connection,
}

impl WaylandCapture {
    /// Create a new Wayland capture instance
    pub fn new() -> Result<Self> {
        let connection = zbus::blocking::Connection::session()
            .map_err(|e| anyhow!("Failed to connect to D-Bus session: {}", e))?;

        Ok(Self { connection })
    }

    /// Capture the entire screen using the portal
    pub fn capture_screen(&self) -> Result<PathBuf> {
        self.capture_with_options(false)
    }

    /// Capture with interactive selection (region/window)
    pub fn capture_interactive(&self) -> Result<PathBuf> {
        self.capture_with_options(true)
    }

    fn capture_with_options(&self, interactive: bool) -> Result<PathBuf> {
        let request_token = format!("winux_screenshot_{}", REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst));
        let sender_name = self.connection
            .unique_name()
            .ok_or_else(|| anyhow!("No unique name"))?
            .as_str()
            .trim_start_matches(':')
            .replace('.', "_");

        let request_path = format!(
            "/org/freedesktop/portal/desktop/request/{}/{}",
            sender_name, request_token
        );

        // Build options
        let mut options: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
        options.insert("handle_token", request_token.clone().into());
        options.insert("interactive", interactive.into());
        options.insert("modal", false.into());

        // Call Screenshot method
        let proxy = zbus::blocking::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Screenshot",
        )?;

        // Subscribe to Response signal before making the call
        let response_proxy = zbus::blocking::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            &request_path,
            "org.freedesktop.portal.Request",
        )?;

        let _: zbus::zvariant::OwnedObjectPath = proxy.call("Screenshot", &("", options))?;

        // Wait for Response signal
        let mut response_received = false;
        let mut result_uri: Option<String> = None;
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60);

        while !response_received && start_time.elapsed() < timeout {
            // Try to receive the response
            if let Ok(msg) = self.connection.receive_message() {
                if let Some(header) = msg.header() {
                    if header.interface().map(|i| i.as_str()) == Some("org.freedesktop.portal.Request")
                        && header.member().map(|m| m.as_str()) == Some("Response")
                    {
                        // Parse response
                        if let Ok(body) = msg.body().deserialize::<(u32, HashMap<String, zbus::zvariant::OwnedValue>)>() {
                            let (response_code, results) = body;
                            if response_code == 0 {
                                if let Some(uri) = results.get("uri") {
                                    if let Ok(uri_str) = <String as TryFrom<_>>::try_from(uri.clone()) {
                                        result_uri = Some(uri_str);
                                    }
                                }
                            }
                            response_received = true;
                        }
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Close the request if still pending
        let _ = response_proxy.call::<_, ()>("Close", &());

        if let Some(uri) = result_uri {
            // Convert file:// URI to path
            let path = uri.strip_prefix("file://")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(&uri));

            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow!("Screenshot capture was cancelled or failed"))
    }
}

/// Check if xdg-desktop-portal-screenshot is available
pub fn is_portal_available() -> bool {
    if let Ok(connection) = zbus::blocking::Connection::session() {
        let proxy = zbus::blocking::Proxy::new(
            &connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.DBus.Properties",
        );

        if let Ok(proxy) = proxy {
            let result: Result<u32, _> = proxy.call(
                "Get",
                &("org.freedesktop.portal.Screenshot", "version"),
            );
            return result.is_ok();
        }
    }

    false
}
