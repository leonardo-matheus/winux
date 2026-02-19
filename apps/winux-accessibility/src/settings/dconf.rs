//! dconf/gsettings integration for accessibility settings
//!
//! This module provides integration with GNOME's dconf/gsettings system
//! for reading and writing accessibility settings.

use std::process::Command;
use anyhow::{Result, Context};
use tracing::{info, warn, error};

/// GNOME accessibility schema paths
const A11Y_SCHEMA: &str = "org.gnome.desktop.a11y";
const A11Y_APPLICATIONS: &str = "org.gnome.desktop.a11y.applications";
const A11Y_INTERFACE: &str = "org.gnome.desktop.a11y.interface";
const A11Y_KEYBOARD: &str = "org.gnome.desktop.a11y.keyboard";
const A11Y_MAGNIFIER: &str = "org.gnome.desktop.a11y.magnifier";
const A11Y_MOUSE: &str = "org.gnome.desktop.a11y.mouse";
const WM_PREFERENCES: &str = "org.gnome.desktop.wm.preferences";
const INTERFACE: &str = "org.gnome.desktop.interface";
const PERIPHERALS_MOUSE: &str = "org.gnome.desktop.peripherals.mouse";
const PERIPHERALS_KEYBOARD: &str = "org.gnome.desktop.peripherals.keyboard";
const SOUND: &str = "org.gnome.desktop.sound";

/// DconfSettings provides methods for reading and writing accessibility settings
pub struct DconfSettings;

impl DconfSettings {
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
    // Visual/Seeing Settings
    // =========================================================================

    /// Get high contrast setting
    pub fn get_high_contrast() -> Result<bool> {
        let value = Self::get(A11Y_INTERFACE, "high-contrast")?;
        Ok(value == "true")
    }

    /// Set high contrast setting
    pub fn set_high_contrast(enabled: bool) {
        Self::set(A11Y_INTERFACE, "high-contrast", if enabled { "true" } else { "false" });
    }

    /// Get large text setting
    pub fn get_large_text() -> Result<bool> {
        let value = Self::get(INTERFACE, "text-scaling-factor")?;
        if let Ok(factor) = value.parse::<f64>() {
            Ok(factor >= 1.25)
        } else {
            Ok(false)
        }
    }

    /// Set large text setting
    pub fn set_large_text(enabled: bool) {
        let factor = if enabled { "1.25" } else { "1.0" };
        Self::set(INTERFACE, "text-scaling-factor", factor);
    }

    /// Get text scaling factor
    pub fn get_text_scaling_factor() -> Result<f64> {
        let value = Self::get(INTERFACE, "text-scaling-factor")?;
        value.parse().context("Failed to parse text scaling factor")
    }

    /// Set text scaling factor
    pub fn set_text_scaling_factor(factor: f64) {
        Self::set(INTERFACE, "text-scaling-factor", &factor.to_string());
    }

    /// Get cursor size
    pub fn get_cursor_size() -> Result<i32> {
        let value = Self::get(INTERFACE, "cursor-size")?;
        value.parse().context("Failed to parse cursor size")
    }

    /// Set cursor size
    pub fn set_cursor_size(size: i32) {
        Self::set(INTERFACE, "cursor-size", &size.to_string());
    }

    /// Get reduce animations setting
    pub fn get_reduce_animations() -> Result<bool> {
        let value = Self::get(INTERFACE, "enable-animations")?;
        Ok(value == "false")
    }

    /// Set reduce animations setting
    pub fn set_reduce_animations(reduced: bool) {
        Self::set(INTERFACE, "enable-animations", if reduced { "false" } else { "true" });
    }

    /// Get always show scrollbars
    pub fn get_always_show_scrollbars() -> Result<bool> {
        let value = Self::get(INTERFACE, "overlay-scrolling")?;
        Ok(value == "false")
    }

    /// Set always show scrollbars
    pub fn set_always_show_scrollbars(always: bool) {
        Self::set(INTERFACE, "overlay-scrolling", if always { "false" } else { "true" });
    }

    /// Get cursor blink setting
    pub fn get_cursor_blink() -> Result<bool> {
        let value = Self::get(INTERFACE, "cursor-blink")?;
        Ok(value == "true")
    }

    /// Set cursor blink setting
    pub fn set_cursor_blink(enabled: bool) {
        Self::set(INTERFACE, "cursor-blink", if enabled { "true" } else { "false" });
    }

    /// Get locate pointer setting
    pub fn get_locate_pointer() -> Result<bool> {
        let value = Self::get(INTERFACE, "locate-pointer")?;
        Ok(value == "true")
    }

    /// Set locate pointer setting
    pub fn set_locate_pointer(enabled: bool) {
        Self::set(INTERFACE, "locate-pointer", if enabled { "true" } else { "false" });
    }

    // =========================================================================
    // Hearing Settings
    // =========================================================================

    /// Get visual alerts enabled
    pub fn get_visual_alerts() -> Result<bool> {
        let value = Self::get(WM_PREFERENCES, "visual-bell")?;
        Ok(value == "true")
    }

    /// Set visual alerts enabled
    pub fn set_visual_alerts(enabled: bool) {
        Self::set(WM_PREFERENCES, "visual-bell", if enabled { "true" } else { "false" });
    }

    /// Get visual alerts type
    pub fn get_visual_alerts_type() -> Result<String> {
        Self::get(WM_PREFERENCES, "visual-bell-type")
    }

    /// Set visual alerts type (window, frame, fullscreen)
    pub fn set_visual_alerts_type(alert_type: &str) {
        Self::set(WM_PREFERENCES, "visual-bell-type", &format!("'{}'", alert_type));
    }

    /// Trigger a visual alert for testing
    pub fn trigger_visual_alert() {
        // Use xdotool or similar to trigger bell
        let _ = Command::new("sh")
            .args(["-c", "echo -e '\\a'"])
            .spawn();
    }

    /// Get captions enabled
    pub fn get_captions_enabled() -> Result<bool> {
        // This is typically application-specific, but we store it in a11y settings
        Ok(false) // Default
    }

    /// Set captions enabled
    pub fn set_captions_enabled(_enabled: bool) {
        // Application-specific setting
    }

    /// Get mono audio setting
    pub fn get_mono_audio() -> Result<bool> {
        // Check PulseAudio/PipeWire settings
        let output = Command::new("pactl")
            .args(["get-sink-mute", "@DEFAULT_SINK@"])
            .output();

        // This is a simplified check - actual mono audio requires PulseAudio modules
        Ok(false)
    }

    /// Set mono audio
    pub fn set_mono_audio(enabled: bool) {
        if enabled {
            // Load the mono audio module
            let _ = Command::new("pactl")
                .args(["load-module", "module-remap-sink",
                       "sink_name=mono", "master=@DEFAULT_SINK@",
                       "channels=2", "channel_map=mono,mono"])
                .spawn();
        } else {
            // Unload the module
            let _ = Command::new("pactl")
                .args(["unload-module", "module-remap-sink"])
                .spawn();
        }
    }

    /// Set audio balance (-1.0 left to 1.0 right)
    pub fn set_audio_balance(balance: f64) {
        // Use pactl to set balance
        let left = if balance <= 0.0 { 1.0 } else { 1.0 - balance };
        let right = if balance >= 0.0 { 1.0 } else { 1.0 + balance };

        let _ = Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@",
                   &format!("{}%", (left * 100.0) as i32),
                   &format!("{}%", (right * 100.0) as i32)])
            .spawn();
    }

    /// Get event sounds enabled
    pub fn get_event_sounds() -> Result<bool> {
        let value = Self::get(SOUND, "event-sounds")?;
        Ok(value == "true")
    }

    /// Set event sounds
    pub fn set_event_sounds(enabled: bool) {
        Self::set(SOUND, "event-sounds", if enabled { "true" } else { "false" });
    }

    /// Set input feedback sounds
    pub fn set_input_feedback_sounds(enabled: bool) {
        Self::set(SOUND, "input-feedback-sounds", if enabled { "true" } else { "false" });
    }

    // =========================================================================
    // Keyboard/Typing Settings
    // =========================================================================

    /// Get screen keyboard enabled
    pub fn get_screen_keyboard_enabled() -> Result<bool> {
        let value = Self::get(A11Y_APPLICATIONS, "screen-keyboard-enabled")?;
        Ok(value == "true")
    }

    /// Set screen keyboard enabled
    pub fn set_screen_keyboard_enabled(enabled: bool) {
        Self::set(A11Y_APPLICATIONS, "screen-keyboard-enabled", if enabled { "true" } else { "false" });
    }

    /// Get sticky keys enabled
    pub fn get_sticky_keys() -> Result<bool> {
        let value = Self::get(A11Y_KEYBOARD, "stickykeys-enable")?;
        Ok(value == "true")
    }

    /// Set sticky keys enabled
    pub fn set_sticky_keys(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "stickykeys-enable", if enabled { "true" } else { "false" });
    }

    /// Set sticky keys two-key-off
    pub fn set_sticky_keys_two_key_off(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "stickykeys-two-key-off", if enabled { "true" } else { "false" });
    }

    /// Set sticky keys beep
    pub fn set_sticky_keys_beep(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "stickykeys-modifier-beep", if enabled { "true" } else { "false" });
    }

    /// Get slow keys enabled
    pub fn get_slow_keys() -> Result<bool> {
        let value = Self::get(A11Y_KEYBOARD, "slowkeys-enable")?;
        Ok(value == "true")
    }

    /// Set slow keys enabled
    pub fn set_slow_keys(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "slowkeys-enable", if enabled { "true" } else { "false" });
    }

    /// Get slow keys delay
    pub fn get_slow_keys_delay() -> Result<u32> {
        let value = Self::get(A11Y_KEYBOARD, "slowkeys-delay")?;
        value.parse().context("Failed to parse slow keys delay")
    }

    /// Set slow keys delay (milliseconds)
    pub fn set_slow_keys_delay(delay: u32) {
        Self::set(A11Y_KEYBOARD, "slowkeys-delay", &delay.to_string());
    }

    /// Set slow keys beep on accept
    pub fn set_slow_keys_beep_accept(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "slowkeys-beep-accept", if enabled { "true" } else { "false" });
    }

    /// Get bounce keys enabled
    pub fn get_bounce_keys() -> Result<bool> {
        let value = Self::get(A11Y_KEYBOARD, "bouncekeys-enable")?;
        Ok(value == "true")
    }

    /// Set bounce keys enabled
    pub fn set_bounce_keys(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "bouncekeys-enable", if enabled { "true" } else { "false" });
    }

    /// Get bounce keys delay
    pub fn get_bounce_keys_delay() -> Result<u32> {
        let value = Self::get(A11Y_KEYBOARD, "bouncekeys-delay")?;
        value.parse().context("Failed to parse bounce keys delay")
    }

    /// Set bounce keys delay (milliseconds)
    pub fn set_bounce_keys_delay(delay: u32) {
        Self::set(A11Y_KEYBOARD, "bouncekeys-delay", &delay.to_string());
    }

    /// Set bounce keys beep on reject
    pub fn set_bounce_keys_beep(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "bouncekeys-beep-reject", if enabled { "true" } else { "false" });
    }

    /// Get repeat keys enabled
    pub fn get_repeat_keys() -> Result<bool> {
        let value = Self::get(PERIPHERALS_KEYBOARD, "repeat")?;
        Ok(value == "true")
    }

    /// Set repeat keys enabled
    pub fn set_repeat_keys(enabled: bool) {
        Self::set(PERIPHERALS_KEYBOARD, "repeat", if enabled { "true" } else { "false" });
    }

    /// Get repeat keys delay
    pub fn get_repeat_keys_delay() -> Result<u32> {
        let value = Self::get(PERIPHERALS_KEYBOARD, "delay")?;
        value.parse().context("Failed to parse repeat delay")
    }

    /// Set repeat keys delay
    pub fn set_repeat_keys_delay(delay: u32) {
        Self::set(PERIPHERALS_KEYBOARD, "delay", &delay.to_string());
    }

    /// Get repeat keys interval
    pub fn get_repeat_keys_interval() -> Result<u32> {
        let value = Self::get(PERIPHERALS_KEYBOARD, "repeat-interval")?;
        value.parse().context("Failed to parse repeat interval")
    }

    /// Set repeat keys interval
    pub fn set_repeat_keys_interval(interval: u32) {
        Self::set(PERIPHERALS_KEYBOARD, "repeat-interval", &interval.to_string());
    }

    // =========================================================================
    // Mouse/Pointing Settings
    // =========================================================================

    /// Get mouse keys enabled
    pub fn get_mouse_keys() -> Result<bool> {
        let value = Self::get(A11Y_KEYBOARD, "mousekeys-enable")?;
        Ok(value == "true")
    }

    /// Set mouse keys enabled
    pub fn set_mouse_keys(enabled: bool) {
        Self::set(A11Y_KEYBOARD, "mousekeys-enable", if enabled { "true" } else { "false" });
    }

    /// Get mouse keys max speed
    pub fn get_mouse_keys_max_speed() -> Result<u32> {
        let value = Self::get(A11Y_KEYBOARD, "mousekeys-max-speed")?;
        value.parse().context("Failed to parse mouse keys max speed")
    }

    /// Set mouse keys max speed
    pub fn set_mouse_keys_max_speed(speed: u32) {
        Self::set(A11Y_KEYBOARD, "mousekeys-max-speed", &speed.to_string());
    }

    /// Get mouse keys acceleration time
    pub fn get_mouse_keys_accel_time() -> Result<u32> {
        let value = Self::get(A11Y_KEYBOARD, "mousekeys-accel-time")?;
        value.parse().context("Failed to parse mouse keys accel time")
    }

    /// Set mouse keys acceleration time
    pub fn set_mouse_keys_accel_time(time: u32) {
        Self::set(A11Y_KEYBOARD, "mousekeys-accel-time", &time.to_string());
    }

    /// Get secondary click enabled
    pub fn get_secondary_click_enabled() -> Result<bool> {
        let value = Self::get(A11Y_MOUSE, "secondary-click-enabled")?;
        Ok(value == "true")
    }

    /// Set secondary click enabled
    pub fn set_secondary_click_enabled(enabled: bool) {
        Self::set(A11Y_MOUSE, "secondary-click-enabled", if enabled { "true" } else { "false" });
    }

    /// Get secondary click time
    pub fn get_secondary_click_time() -> Result<f64> {
        let value = Self::get(A11Y_MOUSE, "secondary-click-time")?;
        value.parse().context("Failed to parse secondary click time")
    }

    /// Set secondary click time
    pub fn set_secondary_click_time(time: f64) {
        Self::set(A11Y_MOUSE, "secondary-click-time", &time.to_string());
    }

    /// Get dwell click enabled
    pub fn get_dwell_click_enabled() -> Result<bool> {
        let value = Self::get(A11Y_MOUSE, "dwell-click-enabled")?;
        Ok(value == "true")
    }

    /// Set dwell click enabled
    pub fn set_dwell_click_enabled(enabled: bool) {
        Self::set(A11Y_MOUSE, "dwell-click-enabled", if enabled { "true" } else { "false" });
    }

    /// Get dwell time
    pub fn get_dwell_time() -> Result<f64> {
        let value = Self::get(A11Y_MOUSE, "dwell-time")?;
        value.parse().context("Failed to parse dwell time")
    }

    /// Set dwell time
    pub fn set_dwell_time(time: f64) {
        Self::set(A11Y_MOUSE, "dwell-time", &time.to_string());
    }

    /// Get dwell threshold
    pub fn get_dwell_threshold() -> Result<u32> {
        let value = Self::get(A11Y_MOUSE, "dwell-threshold")?;
        value.parse().context("Failed to parse dwell threshold")
    }

    /// Set dwell threshold
    pub fn set_dwell_threshold(threshold: u32) {
        Self::set(A11Y_MOUSE, "dwell-threshold", &threshold.to_string());
    }

    /// Get dwell mode
    pub fn get_dwell_mode() -> Result<String> {
        Self::get(A11Y_MOUSE, "click-type-window")
    }

    /// Set dwell mode
    pub fn set_dwell_mode(mode: &str) {
        Self::set(A11Y_MOUSE, "dwell-mode", &format!("'{}'", mode));
    }

    /// Get double click time
    pub fn get_double_click_time() -> Result<u32> {
        let value = Self::get(PERIPHERALS_MOUSE, "double-click")?;
        value.parse().context("Failed to parse double click time")
    }

    /// Set double click time
    pub fn set_double_click_time(time: u32) {
        Self::set(PERIPHERALS_MOUSE, "double-click", &time.to_string());
    }

    /// Set pointer speed
    pub fn set_pointer_speed(speed: f64) {
        Self::set(PERIPHERALS_MOUSE, "speed", &speed.to_string());
    }

    /// Set natural scrolling
    pub fn set_natural_scroll(enabled: bool) {
        Self::set(PERIPHERALS_MOUSE, "natural-scroll", if enabled { "true" } else { "false" });
    }

    /// Set left-handed mouse
    pub fn set_left_handed(enabled: bool) {
        Self::set(PERIPHERALS_MOUSE, "left-handed", if enabled { "true" } else { "false" });
    }

    // =========================================================================
    // Magnifier/Zoom Settings
    // =========================================================================

    /// Get magnifier enabled
    pub fn get_magnifier_enabled() -> Result<bool> {
        let value = Self::get(A11Y_APPLICATIONS, "screen-magnifier-enabled")?;
        Ok(value == "true")
    }

    /// Set magnifier enabled
    pub fn set_magnifier_enabled(enabled: bool) {
        Self::set(A11Y_APPLICATIONS, "screen-magnifier-enabled", if enabled { "true" } else { "false" });
    }

    /// Get magnifier factor
    pub fn get_magnifier_factor() -> Result<f64> {
        let value = Self::get(A11Y_MAGNIFIER, "mag-factor")?;
        value.parse().context("Failed to parse magnifier factor")
    }

    /// Set magnifier factor
    pub fn set_magnifier_factor(factor: f64) {
        Self::set(A11Y_MAGNIFIER, "mag-factor", &factor.to_string());
    }

    /// Get magnifier screen position
    pub fn get_magnifier_screen_position() -> Result<String> {
        Self::get(A11Y_MAGNIFIER, "screen-position")
    }

    /// Set magnifier screen position
    pub fn set_magnifier_screen_position(position: &str) {
        Self::set(A11Y_MAGNIFIER, "screen-position", &format!("'{}'", position));
    }

    /// Get magnifier lens size
    pub fn get_magnifier_lens_size() -> Result<u32> {
        let value = Self::get(A11Y_MAGNIFIER, "lens-size")?;
        // lens-size is a (width, height) tuple, get width
        let cleaned = value.trim_matches(|c| c == '(' || c == ')');
        if let Some(width) = cleaned.split(',').next() {
            width.trim().parse().context("Failed to parse lens size")
        } else {
            Ok(300)
        }
    }

    /// Set magnifier lens size
    pub fn set_magnifier_lens_size(size: u32) {
        Self::set(A11Y_MAGNIFIER, "lens-mode", "true");
        // Note: lens-size requires tuple format
    }

    /// Get magnifier lens shape
    pub fn get_magnifier_lens_shape() -> Result<String> {
        Self::get(A11Y_MAGNIFIER, "lens-shape")
    }

    /// Set magnifier lens shape
    pub fn set_magnifier_lens_shape(shape: &str) {
        Self::set(A11Y_MAGNIFIER, "lens-shape", &format!("'{}'", shape));
    }

    /// Get magnifier mouse tracking
    pub fn get_magnifier_mouse_tracking() -> Result<String> {
        Self::get(A11Y_MAGNIFIER, "mouse-tracking")
    }

    /// Set magnifier mouse tracking
    pub fn set_magnifier_mouse_tracking(mode: &str) {
        Self::set(A11Y_MAGNIFIER, "mouse-tracking", &format!("'{}'", mode));
    }

    /// Get magnifier focus tracking
    pub fn get_magnifier_focus_tracking() -> Result<String> {
        Self::get(A11Y_MAGNIFIER, "focus-tracking")
    }

    /// Set magnifier focus tracking
    pub fn set_magnifier_focus_tracking(mode: &str) {
        Self::set(A11Y_MAGNIFIER, "focus-tracking", &format!("'{}'", mode));
    }

    /// Get magnifier caret tracking
    pub fn get_magnifier_caret_tracking() -> Result<String> {
        Self::get(A11Y_MAGNIFIER, "caret-tracking")
    }

    /// Set magnifier caret tracking
    pub fn set_magnifier_caret_tracking(mode: &str) {
        Self::set(A11Y_MAGNIFIER, "caret-tracking", &format!("'{}'", mode));
    }

    /// Get magnifier cross hairs enabled
    pub fn get_magnifier_cross_hairs() -> Result<bool> {
        let value = Self::get(A11Y_MAGNIFIER, "show-cross-hairs")?;
        Ok(value == "true")
    }

    /// Set magnifier cross hairs enabled
    pub fn set_magnifier_cross_hairs(enabled: bool) {
        Self::set(A11Y_MAGNIFIER, "show-cross-hairs", if enabled { "true" } else { "false" });
    }

    /// Get magnifier cross hairs length
    pub fn get_magnifier_cross_hairs_length() -> Result<u32> {
        let value = Self::get(A11Y_MAGNIFIER, "cross-hairs-length")?;
        value.parse().context("Failed to parse cross hairs length")
    }

    /// Set magnifier cross hairs length
    pub fn set_magnifier_cross_hairs_length(length: u32) {
        Self::set(A11Y_MAGNIFIER, "cross-hairs-length", &length.to_string());
    }

    /// Get magnifier cross hairs thickness
    pub fn get_magnifier_cross_hairs_thickness() -> Result<u32> {
        let value = Self::get(A11Y_MAGNIFIER, "cross-hairs-thickness")?;
        value.parse().context("Failed to parse cross hairs thickness")
    }

    /// Set magnifier cross hairs thickness
    pub fn set_magnifier_cross_hairs_thickness(thickness: u32) {
        Self::set(A11Y_MAGNIFIER, "cross-hairs-thickness", &thickness.to_string());
    }

    /// Get magnifier invert lightness
    pub fn get_magnifier_invert_lightness() -> Result<bool> {
        let value = Self::get(A11Y_MAGNIFIER, "invert-lightness")?;
        Ok(value == "true")
    }

    /// Set magnifier invert lightness
    pub fn set_magnifier_invert_lightness(invert: bool) {
        Self::set(A11Y_MAGNIFIER, "invert-lightness", if invert { "true" } else { "false" });
    }

    /// Get magnifier brightness
    pub fn get_magnifier_brightness() -> Result<f64> {
        let value = Self::get(A11Y_MAGNIFIER, "brightness-red")?;
        value.parse().context("Failed to parse brightness")
    }

    /// Set magnifier brightness (applies to all channels)
    pub fn set_magnifier_brightness(brightness: f64) {
        Self::set(A11Y_MAGNIFIER, "brightness-red", &brightness.to_string());
        Self::set(A11Y_MAGNIFIER, "brightness-green", &brightness.to_string());
        Self::set(A11Y_MAGNIFIER, "brightness-blue", &brightness.to_string());
    }

    /// Get magnifier contrast
    pub fn get_magnifier_contrast() -> Result<f64> {
        let value = Self::get(A11Y_MAGNIFIER, "contrast-red")?;
        value.parse().context("Failed to parse contrast")
    }

    /// Set magnifier contrast (applies to all channels)
    pub fn set_magnifier_contrast(contrast: f64) {
        Self::set(A11Y_MAGNIFIER, "contrast-red", &contrast.to_string());
        Self::set(A11Y_MAGNIFIER, "contrast-green", &contrast.to_string());
        Self::set(A11Y_MAGNIFIER, "contrast-blue", &contrast.to_string());
    }
}
