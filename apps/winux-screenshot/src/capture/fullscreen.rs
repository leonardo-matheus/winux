//! Fullscreen capture implementation

use super::{CaptureResult, generate_filename, get_screenshots_dir, wayland::WaylandCapture};
use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::PathBuf;

/// Capture the entire screen
pub fn capture_fullscreen() -> Result<CaptureResult> {
    // First try Wayland portal (preferred method)
    if let Ok(result) = capture_via_portal() {
        return Ok(result);
    }

    // Fallback to grim (Wayland)
    if let Ok(result) = capture_via_grim() {
        return Ok(result);
    }

    // Fallback to gnome-screenshot
    if let Ok(result) = capture_via_gnome_screenshot() {
        return Ok(result);
    }

    // Fallback to scrot (X11)
    if let Ok(result) = capture_via_scrot() {
        return Ok(result);
    }

    Err(anyhow!("No screenshot tool available. Please install grim, gnome-screenshot, or scrot."))
}

fn capture_via_portal() -> Result<CaptureResult> {
    let wayland = WaylandCapture::new()?;
    let path = wayland.capture_screen()?;
    load_capture_result(path)
}

fn capture_via_grim() -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    let output = Command::new("grim")
        .arg(&path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("grim failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    load_capture_result(path)
}

fn capture_via_gnome_screenshot() -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    let output = Command::new("gnome-screenshot")
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
