//! XDG Desktop Portal Screencast interface
//!
//! Implements the org.freedesktop.portal.ScreenCast portal for
//! Wayland-native screen capture with proper permissions.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use zbus::zvariant::{OwnedValue, Value};

static REQUEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Stream information from the portal
#[derive(Debug, Clone)]
pub struct PortalStream {
    /// PipeWire node ID
    pub node_id: u32,
    /// Stream width
    pub width: u32,
    /// Stream height
    pub height: u32,
    /// Position X
    pub x: i32,
    /// Position Y
    pub y: i32,
    /// Source type (1 = monitor, 2 = window)
    pub source_type: u32,
}

/// XDG Desktop Portal Screencast interface
pub struct ScreencastPortal {
    connection: zbus::Connection,
}

impl ScreencastPortal {
    /// Create a new portal instance
    pub async fn new() -> Result<Self> {
        let connection = zbus::Connection::session().await
            .map_err(|e| anyhow!("Failed to connect to D-Bus session: {}", e))?;

        Ok(Self { connection })
    }

    /// Get the unique request token and path
    fn get_request_path(&self) -> Result<(String, String)> {
        let token = format!("winux_screencast_{}", REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst));
        let sender_name = self.connection
            .unique_name()
            .ok_or_else(|| anyhow!("No unique name"))?
            .as_str()
            .trim_start_matches(':')
            .replace('.', "_");

        let path = format!(
            "/org/freedesktop/portal/desktop/request/{}/{}",
            sender_name, token
        );

        Ok((token, path))
    }

    /// Create a new screencast session
    pub async fn create_session(&self) -> Result<String> {
        let (handle_token, request_path) = self.get_request_path()?;
        let session_token = format!("winux_session_{}", REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst));

        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("handle_token", Value::from(handle_token));
        options.insert("session_handle_token", Value::from(session_token));

        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await?;

        // Subscribe to response signal
        let response = self.wait_for_response(&request_path).await;

        // Call CreateSession
        let _: zbus::zvariant::OwnedObjectPath = proxy.call("CreateSession", &(options,)).await?;

        // Wait for response
        let (code, results) = response.await?;

        if code != 0 {
            return Err(anyhow!("CreateSession failed with code {}", code));
        }

        // Extract session handle
        let session_handle = results.get("session_handle")
            .and_then(|v| String::try_from(v.clone()).ok())
            .ok_or_else(|| anyhow!("No session handle in response"))?;

        Ok(session_handle)
    }

    /// Select sources for the screencast
    pub async fn select_sources(
        &self,
        session_handle: &str,
        source_types: u32,
        cursor_mode: u32,
        multiple: bool,
    ) -> Result<()> {
        let (handle_token, request_path) = self.get_request_path()?;

        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("handle_token", Value::from(handle_token));
        options.insert("types", Value::from(source_types));
        options.insert("multiple", Value::from(multiple));
        options.insert("cursor_mode", Value::from(cursor_mode));

        // Request persist mode for session restore
        options.insert("persist_mode", Value::from(2u32)); // Persist until explicitly revoked

        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await?;

        // Subscribe to response signal
        let response = self.wait_for_response(&request_path).await;

        // Call SelectSources
        let _: zbus::zvariant::OwnedObjectPath = proxy.call(
            "SelectSources",
            &(session_handle, options),
        ).await?;

        // Wait for response
        let (code, _) = response.await?;

        if code != 0 {
            return Err(anyhow!("SelectSources failed with code {}", code));
        }

        Ok(())
    }

    /// Start the screencast
    pub async fn start(&self, session_handle: &str) -> Result<Vec<PortalStream>> {
        let (handle_token, request_path) = self.get_request_path()?;

        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("handle_token", Value::from(handle_token));

        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await?;

        // Subscribe to response signal
        let response = self.wait_for_response(&request_path).await;

        // Call Start with empty parent window identifier
        let _: zbus::zvariant::OwnedObjectPath = proxy.call(
            "Start",
            &(session_handle, "", options),
        ).await?;

        // Wait for response
        let (code, results) = response.await?;

        if code != 0 {
            return Err(anyhow!("Start failed with code {} (user cancelled or permission denied)", code));
        }

        // Parse streams from response
        let streams = self.parse_streams(&results)?;

        Ok(streams)
    }

    /// Close a session
    pub async fn close_session(&self, session_handle: &str) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            session_handle,
            "org.freedesktop.portal.Session",
        ).await?;

        let _: () = proxy.call("Close", &()).await?;

        Ok(())
    }

    /// Wait for portal response signal
    async fn wait_for_response(&self, request_path: &str) -> impl std::future::Future<Output = Result<(u32, HashMap<String, OwnedValue>)>> {
        let connection = self.connection.clone();
        let request_path = request_path.to_string();

        async move {
            use futures::StreamExt;

            // Create a proxy for the request object
            let proxy = zbus::Proxy::new(
                &connection,
                "org.freedesktop.portal.Desktop",
                &request_path,
                "org.freedesktop.portal.Request",
            ).await?;

            // Create signal stream
            let mut stream = proxy.receive_signal("Response").await?;

            // Wait for response with timeout
            let timeout = tokio::time::Duration::from_secs(120);

            match tokio::time::timeout(timeout, stream.next()).await {
                Ok(Some(signal)) => {
                    let body = signal.body();
                    let (code, results): (u32, HashMap<String, OwnedValue>) = body.deserialize()?;
                    Ok((code, results))
                }
                Ok(None) => Err(anyhow!("Signal stream ended unexpectedly")),
                Err(_) => Err(anyhow!("Timeout waiting for portal response")),
            }
        }
    }

    /// Parse stream information from portal response
    fn parse_streams(&self, results: &HashMap<String, OwnedValue>) -> Result<Vec<PortalStream>> {
        let streams_value = results.get("streams")
            .ok_or_else(|| anyhow!("No streams in response"))?;

        // Streams is an array of (node_id, properties) tuples
        let streams_array: Vec<(u32, HashMap<String, OwnedValue>)> =
            streams_value.clone().try_into()
                .map_err(|e| anyhow!("Failed to parse streams: {:?}", e))?;

        let mut streams = Vec::new();

        for (node_id, props) in streams_array {
            let width = props.get("size")
                .and_then(|v| {
                    let tuple: Option<(i32, i32)> = v.clone().try_into().ok();
                    tuple.map(|(w, _)| w as u32)
                })
                .unwrap_or(1920);

            let height = props.get("size")
                .and_then(|v| {
                    let tuple: Option<(i32, i32)> = v.clone().try_into().ok();
                    tuple.map(|(_, h)| h as u32)
                })
                .unwrap_or(1080);

            let position = props.get("position")
                .and_then(|v| {
                    let tuple: Option<(i32, i32)> = v.clone().try_into().ok();
                    tuple
                })
                .unwrap_or((0, 0));

            let source_type = props.get("source_type")
                .and_then(|v| u32::try_from(v.clone()).ok())
                .unwrap_or(1);

            streams.push(PortalStream {
                node_id,
                width,
                height,
                x: position.0,
                y: position.1,
                source_type,
            });
        }

        Ok(streams)
    }

    /// Get available cursor modes
    pub async fn get_available_cursor_modes(&self) -> Result<u32> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await?;

        let modes: u32 = proxy.get_property("AvailableCursorModes").await?;
        Ok(modes)
    }

    /// Get available source types
    pub async fn get_available_source_types(&self) -> Result<u32> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await?;

        let types: u32 = proxy.get_property("AvailableSourceTypes").await?;
        Ok(types)
    }

    /// Get portal version
    pub async fn get_version(&self) -> Result<u32> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await?;

        let version: u32 = proxy.get_property("version").await?;
        Ok(version)
    }
}

/// Check if the screencast portal is available
pub fn is_portal_available() -> bool {
    // Use blocking connection for simple check
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
                &("org.freedesktop.portal.ScreenCast", "version"),
            );
            return result.is_ok();
        }
    }

    false
}

/// Check if PipeWire is available for actual capture
pub fn is_pipewire_available() -> bool {
    // Check if PipeWire socket exists
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        let socket_path = std::path::Path::new(&runtime_dir).join("pipewire-0");
        return socket_path.exists();
    }
    false
}
