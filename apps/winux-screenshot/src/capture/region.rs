//! Region capture implementation

use super::{CaptureResult, generate_filename, get_screenshots_dir, wayland::WaylandCapture};
use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::PathBuf;

/// Region selection for capture
#[derive(Debug, Clone, Copy, Default)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Region {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_points(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x1 - x2).abs();
        let height = (y1 - y2).abs();
        Self { x, y, width, height }
    }

    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    pub fn to_grim_format(&self) -> String {
        format!("{},{} {}x{}", self.x, self.y, self.width, self.height)
    }
}

/// Capture a specific region of the screen
pub fn capture_region() -> Result<CaptureResult> {
    // First try Wayland portal with interactive mode
    if let Ok(result) = capture_via_portal() {
        return Ok(result);
    }

    // Fallback to grim + slurp (Wayland)
    if let Ok(result) = capture_via_grim_slurp() {
        return Ok(result);
    }

    // Fallback to gnome-screenshot area mode
    if let Ok(result) = capture_via_gnome_screenshot() {
        return Ok(result);
    }

    // Fallback to scrot (X11)
    if let Ok(result) = capture_via_scrot() {
        return Ok(result);
    }

    Err(anyhow!("No screenshot tool available for region capture."))
}

/// Capture a specific region without user interaction
pub fn capture_region_direct(region: &Region) -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    // Try grim first
    let output = Command::new("grim")
        .arg("-g")
        .arg(region.to_grim_format())
        .arg(&path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            return load_capture_result(path);
        }
    }

    // Fallback to ImageMagick import
    let output = Command::new("import")
        .arg("-window")
        .arg("root")
        .arg("-crop")
        .arg(format!("{}x{}+{}+{}", region.width, region.height, region.x, region.y))
        .arg(&path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Region capture failed"));
    }

    load_capture_result(path)
}

fn capture_via_portal() -> Result<CaptureResult> {
    let wayland = WaylandCapture::new()?;
    let path = wayland.capture_interactive()?;
    load_capture_result(path)
}

fn capture_via_grim_slurp() -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    // First get the region using slurp
    let slurp_output = Command::new("slurp")
        .arg("-d") // Display dimensions
        .output()?;

    if !slurp_output.status.success() {
        return Err(anyhow!("slurp cancelled or failed"));
    }

    let geometry = String::from_utf8_lossy(&slurp_output.stdout).trim().to_string();

    if geometry.is_empty() {
        return Err(anyhow!("No region selected"));
    }

    // Capture the region with grim
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

fn capture_via_gnome_screenshot() -> Result<CaptureResult> {
    let screenshots_dir = get_screenshots_dir();
    let filename = generate_filename();
    let path = screenshots_dir.join(&filename);

    let output = Command::new("gnome-screenshot")
        .arg("-a") // Area/region mode
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
        .arg("-s") // Select region
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
