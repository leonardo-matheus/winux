//! AT-SPI (Assistive Technology Service Provider Interface) integration
//!
//! This module provides integration with AT-SPI2 for screen reader
//! and assistive technology support.

use std::process::Command;
use anyhow::{Result, Context};
use tracing::{info, warn, error};

/// AT-SPI settings paths
const A11Y_APPLICATIONS: &str = "org.gnome.desktop.a11y.applications";

/// AtSpiSettings provides methods for managing assistive technologies
pub struct AtSpiSettings;

impl AtSpiSettings {
    /// Execute gsettings get command
    fn get(schema: &str, key: &str) -> Result<String> {
        let output = Command::new("gsettings")
            .args(["get", schema, key])
            .output()
            .context("Failed to execute gsettings")?;

        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
            Ok(value)
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("gsettings error: {}", err))
        }
    }

    /// Execute gsettings set command
    fn set(schema: &str, key: &str, value: &str) {
        match Command::new("gsettings")
            .args(["set", schema, key, value])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("Set {}.{} = {}", schema, key, value);
                } else {
                    warn!(
                        "Failed to set {}.{}: {}",
                        schema,
                        key,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(e) => {
                error!("Failed to execute gsettings: {}", e);
            }
        }
    }

    // =========================================================================
    // Screen Reader (Orca)
    // =========================================================================

    /// Check if screen reader is enabled
    pub fn is_screen_reader_enabled() -> Result<bool> {
        let value = Self::get(A11Y_APPLICATIONS, "screen-reader-enabled")?;
        Ok(value == "true")
    }

    /// Enable or disable screen reader
    pub fn set_screen_reader_enabled(enabled: bool) {
        Self::set(A11Y_APPLICATIONS, "screen-reader-enabled", if enabled { "true" } else { "false" });
    }

    /// Start Orca screen reader
    pub fn start_orca() {
        info!("Starting Orca screen reader");

        // Kill any existing Orca instance first
        let _ = Command::new("pkill")
            .args(["-f", "orca"])
            .output();

        // Start Orca
        match Command::new("orca")
            .spawn()
        {
            Ok(_) => {
                info!("Orca screen reader started");
            }
            Err(e) => {
                error!("Failed to start Orca: {}", e);

                // Try alternative method using systemd user service
                let _ = Command::new("systemctl")
                    .args(["--user", "start", "orca.service"])
                    .spawn();
            }
        }
    }

    /// Stop Orca screen reader
    pub fn stop_orca() {
        info!("Stopping Orca screen reader");

        // Try graceful shutdown first
        match Command::new("orca")
            .args(["--quit"])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    // Force kill if graceful shutdown fails
                    let _ = Command::new("pkill")
                        .args(["-f", "orca"])
                        .output();
                }
            }
            Err(_) => {
                // Force kill
                let _ = Command::new("pkill")
                    .args(["-f", "orca"])
                    .output();
            }
        }

        // Also stop systemd service if running
        let _ = Command::new("systemctl")
            .args(["--user", "stop", "orca.service"])
            .output();
    }

    /// Check if Orca is running
    pub fn is_orca_running() -> bool {
        match Command::new("pgrep")
            .args(["-f", "orca"])
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Open Orca settings dialog
    pub fn open_orca_settings() {
        info!("Opening Orca settings");

        // Orca settings can be opened with orca -s or orca --setup
        match Command::new("orca")
            .args(["--setup"])
            .spawn()
        {
            Ok(_) => {
                info!("Orca settings opened");
            }
            Err(e) => {
                error!("Failed to open Orca settings: {}", e);

                // Try alternative: open settings via GUI
                let _ = Command::new("orca")
                    .args(["-s"])
                    .spawn();
            }
        }
    }

    /// Set Orca speech rate (words per minute)
    pub fn set_orca_speech_rate(rate: u32) {
        // Orca speech rate is controlled via orca configuration
        // This would typically modify ~/.local/share/orca/user-settings.conf
        // For now, we log the intent
        info!("Setting Orca speech rate to {} wpm", rate);

        // The actual implementation would use orca's Python API or modify config directly
    }

    /// Get Orca speech rate
    pub fn get_orca_speech_rate() -> Result<u32> {
        // Default rate
        Ok(175)
    }

    /// Set Orca voice
    pub fn set_orca_voice(voice: &str) {
        info!("Setting Orca voice to {}", voice);
        // Implementation would modify orca configuration
    }

    /// Get available voices
    pub fn get_available_voices() -> Vec<String> {
        // Query speech-dispatcher for available voices
        match Command::new("spd-say")
            .args(["-L"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .skip(1) // Skip header line
                    .map(|s| s.to_string())
                    .collect()
            }
            Err(_) => {
                vec!["default".to_string()]
            }
        }
    }

    // =========================================================================
    // AT-SPI Registry
    // =========================================================================

    /// Check if AT-SPI is enabled
    pub fn is_atspi_enabled() -> bool {
        // Check if at-spi2-registryd is running
        match Command::new("pgrep")
            .args(["-f", "at-spi2-registryd"])
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Enable AT-SPI (required for screen readers and other ATs)
    pub fn enable_atspi() {
        info!("Enabling AT-SPI");

        // Start the AT-SPI registry daemon
        let _ = Command::new("systemctl")
            .args(["--user", "start", "at-spi-dbus-bus.service"])
            .spawn();

        // Set environment variable for GTK apps
        std::env::set_var("GTK_MODULES", "gail:atk-bridge");
    }

    /// Disable AT-SPI
    pub fn disable_atspi() {
        info!("Disabling AT-SPI");

        let _ = Command::new("systemctl")
            .args(["--user", "stop", "at-spi-dbus-bus.service"])
            .spawn();
    }

    // =========================================================================
    // Speech Dispatcher
    // =========================================================================

    /// Check if speech-dispatcher is running
    pub fn is_speech_dispatcher_running() -> bool {
        match Command::new("pgrep")
            .args(["-f", "speech-dispatcher"])
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Start speech-dispatcher
    pub fn start_speech_dispatcher() {
        info!("Starting speech-dispatcher");

        let _ = Command::new("speech-dispatcher")
            .args(["--spawn"])
            .spawn();
    }

    /// Stop speech-dispatcher
    pub fn stop_speech_dispatcher() {
        info!("Stopping speech-dispatcher");

        let _ = Command::new("pkill")
            .args(["-f", "speech-dispatcher"])
            .spawn();
    }

    /// Test speech output
    pub fn test_speech(text: &str) {
        let _ = Command::new("spd-say")
            .args([text])
            .spawn();
    }

    // =========================================================================
    // Other Assistive Technologies
    // =========================================================================

    /// Check if on-screen keyboard is enabled
    pub fn is_onscreen_keyboard_enabled() -> Result<bool> {
        let value = Self::get(A11Y_APPLICATIONS, "screen-keyboard-enabled")?;
        Ok(value == "true")
    }

    /// Enable/disable on-screen keyboard
    pub fn set_onscreen_keyboard_enabled(enabled: bool) {
        Self::set(A11Y_APPLICATIONS, "screen-keyboard-enabled", if enabled { "true" } else { "false" });

        if enabled {
            // Start on-screen keyboard (e.g., Onboard or GNOME's built-in)
            let _ = Command::new("onboard")
                .spawn()
                .or_else(|_| Command::new("caribou").spawn());
        } else {
            // Stop on-screen keyboard
            let _ = Command::new("pkill")
                .args(["-f", "onboard"])
                .spawn();
            let _ = Command::new("pkill")
                .args(["-f", "caribou"])
                .spawn();
        }
    }

    /// Start on-screen keyboard application
    pub fn start_onscreen_keyboard() {
        info!("Starting on-screen keyboard");

        // Try Onboard first, then Caribou, then GNOME OSK
        if Command::new("onboard").spawn().is_err() {
            if Command::new("caribou").spawn().is_err() {
                let _ = Command::new("gnome-shell-extension-tool")
                    .args(["-e", "screen-keyboard@example.com"])
                    .spawn();
            }
        }
    }

    /// Stop on-screen keyboard
    pub fn stop_onscreen_keyboard() {
        info!("Stopping on-screen keyboard");

        let _ = Command::new("pkill").args(["-f", "onboard"]).spawn();
        let _ = Command::new("pkill").args(["-f", "caribou"]).spawn();
    }
}
