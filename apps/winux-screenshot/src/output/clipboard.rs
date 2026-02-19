//! Clipboard functionality for screenshots

use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::Command;

/// Copy an image file to the clipboard
pub fn copy_to_clipboard(path: &Path) -> Result<()> {
    // Try different clipboard methods based on availability

    // Method 1: wl-copy (Wayland)
    if let Ok(output) = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(std::fs::File::open(path)?)
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    // Method 2: xclip (X11)
    if let Ok(output) = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .arg("-t")
        .arg("image/png")
        .arg("-i")
        .arg(path)
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    // Method 3: xsel (X11 alternative)
    if let Ok(output) = Command::new("xsel")
        .arg("--clipboard")
        .arg("--input")
        .stdin(std::fs::File::open(path)?)
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    // Method 4: copyq (cross-platform)
    if let Ok(output) = Command::new("copyq")
        .arg("copy")
        .arg("image/png")
        .arg("-")
        .stdin(std::fs::File::open(path)?)
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    Err(anyhow!(
        "Could not copy to clipboard. Please install wl-copy (Wayland) or xclip (X11)"
    ))
}

/// Copy raw image data to clipboard
pub fn copy_image_data_to_clipboard(data: &[u8], mime_type: &str) -> Result<()> {
    // Method 1: wl-copy (Wayland)
    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg(mime_type)
        .stdin(std::process::Stdio::piped())
        .spawn();

    if let Ok(ref mut child) = child {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            if stdin.write_all(data).is_ok() {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    return Ok(());
                }
            }
        }
    }

    // Method 2: xclip (X11)
    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .arg("-t")
        .arg(mime_type)
        .stdin(std::process::Stdio::piped())
        .spawn();

    if let Ok(ref mut child) = child {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            if stdin.write_all(data).is_ok() {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    return Ok(());
                }
            }
        }
    }

    Err(anyhow!("Could not copy to clipboard"))
}

/// Copy text to clipboard (for sharing URLs, etc.)
pub fn copy_text_to_clipboard(text: &str) -> Result<()> {
    // Method 1: wl-copy (Wayland)
    if let Ok(output) = Command::new("wl-copy")
        .arg(text)
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    // Method 2: xclip (X11)
    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(std::process::Stdio::piped())
        .spawn();

    if let Ok(ref mut child) = child {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            if stdin.write_all(text.as_bytes()).is_ok() {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    return Ok(());
                }
            }
        }
    }

    // Method 3: xsel (X11 alternative)
    let mut child = Command::new("xsel")
        .arg("--clipboard")
        .arg("--input")
        .stdin(std::process::Stdio::piped())
        .spawn();

    if let Ok(ref mut child) = child {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            if stdin.write_all(text.as_bytes()).is_ok() {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    return Ok(());
                }
            }
        }
    }

    Err(anyhow!("Could not copy to clipboard"))
}

/// Check if clipboard tools are available
pub fn check_clipboard_available() -> bool {
    // Check wl-copy
    if Command::new("which")
        .arg("wl-copy")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check xclip
    if Command::new("which")
        .arg("xclip")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check xsel
    if Command::new("which")
        .arg("xsel")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    false
}

/// Get clipboard manager name
pub fn get_clipboard_manager() -> Option<&'static str> {
    if Command::new("which")
        .arg("wl-copy")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("wl-clipboard (Wayland)");
    }

    if Command::new("which")
        .arg("xclip")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("xclip (X11)");
    }

    if Command::new("which")
        .arg("xsel")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("xsel (X11)");
    }

    None
}
