//! Window capture implementation

use super::{CaptureResult, generate_filename, get_screenshots_dir, wayland::WaylandCapture};
use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::PathBuf;

/// Capture the active/focused window
pub fn capture_window() -> Result<CaptureResult> {
    // First try Wayland portal (preferred method)
    if let Ok(result) = capture_via_portal() {
        return Ok(result);
    }

    // Fallback to grim + slurp with window geometry (Wayland)
    if let Ok(result) = capture_via_grim_window() {
        return Ok(result);
    }

    // Fallback to gnome-screenshot window mode
    if let Ok(result) = capture_via_gnome_screenshot() {
        return Ok(result);
    }

    // Fallback to scrot (X11)
    if let Ok(result) = capture_via_scrot() {
        return Ok(result);
    }

    Err(anyhow!("No screenshot tool available for window capture."))
}

fn capture_via_portal() -> Result<CaptureResult> {
    let wayland = WaylandCapture::new()?;
    // Portal doesn't have direct window capture, but interactive mode allows selection
    let path = wayland.capture_interactive()?;
    load_capture_result(path)
}

fn capture_via_grim_window() -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    // Get focused window geometry using swaymsg or hyprctl
    let geometry = get_focused_window_geometry()?;

    let output = Command::new("grim")
        .arg("-g")
        .arg(&geometry)
        .arg(&path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("grim failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    load_capture_result(path)
}

fn get_focused_window_geometry() -> Result<String> {
    // Try swaymsg first (for Sway)
    if let Ok(output) = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
    {
        if output.status.success() {
            let json = String::from_utf8_lossy(&output.stdout);
            if let Some(geometry) = parse_sway_focused_geometry(&json) {
                return Ok(geometry);
            }
        }
    }

    // Try hyprctl (for Hyprland)
    if let Ok(output) = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
    {
        if output.status.success() {
            let json = String::from_utf8_lossy(&output.stdout);
            if let Some(geometry) = parse_hyprland_window_geometry(&json) {
                return Ok(geometry);
            }
        }
    }

    // Try wlr-randr for general Wayland
    Err(anyhow!("Could not determine focused window geometry"))
}

fn parse_sway_focused_geometry(json: &str) -> Option<String> {
    // Parse sway tree to find focused window
    // This is a simplified parser - in production, use serde_json
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
        if let Some(rect) = find_focused_rect(&value) {
            return Some(format!(
                "{},{} {}x{}",
                rect.x, rect.y, rect.width, rect.height
            ));
        }
    }
    None
}

struct WindowRect {
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

fn find_focused_rect(value: &serde_json::Value) -> Option<WindowRect> {
    if let Some(focused) = value.get("focused") {
        if focused.as_bool() == Some(true) {
            if let Some(rect) = value.get("rect") {
                return Some(WindowRect {
                    x: rect.get("x")?.as_i64()?,
                    y: rect.get("y")?.as_i64()?,
                    width: rect.get("width")?.as_i64()?,
                    height: rect.get("height")?.as_i64()?,
                });
            }
        }
    }

    // Search in nodes
    if let Some(nodes) = value.get("nodes").and_then(|n| n.as_array()) {
        for node in nodes {
            if let Some(rect) = find_focused_rect(node) {
                return Some(rect);
            }
        }
    }

    // Search in floating_nodes
    if let Some(nodes) = value.get("floating_nodes").and_then(|n| n.as_array()) {
        for node in nodes {
            if let Some(rect) = find_focused_rect(node) {
                return Some(rect);
            }
        }
    }

    None
}

fn parse_hyprland_window_geometry(json: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
        let at = value.get("at")?.as_array()?;
        let size = value.get("size")?.as_array()?;

        let x = at.first()?.as_i64()?;
        let y = at.get(1)?.as_i64()?;
        let width = size.first()?.as_i64()?;
        let height = size.get(1)?.as_i64()?;

        return Some(format!("{},{} {}x{}", x, y, width, height));
    }
    None
}

fn capture_via_gnome_screenshot() -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    let output = Command::new("gnome-screenshot")
        .arg("-w") // Window mode
        .arg("-f")
        .arg(&path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("gnome-screenshot failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    load_capture_result(path)
}

fn capture_via_scrot() -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    let output = Command::new("scrot")
        .arg("-u") // Active window
        .arg(&path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("scrot failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    load_capture_result(path)
}

fn load_capture_result(path: PathBuf) -> Result<CaptureResult> {
    let image = image::open(&path)?;
    let width = image.width();
    let height = image.height();

    Ok(CaptureResult {
        image,
        path,
        width,
        height,
    })
}
