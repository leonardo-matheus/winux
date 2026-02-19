// Fontconfig integration for Winux Fonts
// Uses fc-list, fc-cache, and fc-query commands

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::fs;

use super::{FontInfo, FontCategory, FontInstallError};

/// Fontconfig wrapper
pub struct FontConfig {
    /// Cache of font paths
    font_paths: Vec<PathBuf>,
}

impl FontConfig {
    pub fn new() -> Self {
        Self {
            font_paths: Self::get_font_directories(),
        }
    }

    /// Get standard font directories
    fn get_font_directories() -> Vec<PathBuf> {
        let mut dirs = vec![
            PathBuf::from("/usr/share/fonts"),
            PathBuf::from("/usr/local/share/fonts"),
        ];

        // User fonts directory
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/fonts"));
            dirs.push(home.join(".fonts"));
        }

        // XDG data directories
        if let Ok(xdg_data) = std::env::var("XDG_DATA_DIRS") {
            for dir in xdg_data.split(':') {
                dirs.push(PathBuf::from(dir).join("fonts"));
            }
        }

        dirs
    }

    /// Get all installed fonts using fc-list
    pub fn get_all_fonts(&self) -> Vec<FontInfo> {
        let mut fonts = Vec::new();

        // Run fc-list to get all fonts
        // Format: family:style:file:slant:weight
        let output = Command::new("fc-list")
            .args(["--format", "%{family}|%{style}|%{file}|%{slant}|%{weight}\n"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if let Some(font) = Self::parse_fc_list_line(line) {
                        fonts.push(font);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to run fc-list: {}", e);
                // Return some sample fonts for development
                return Self::get_sample_fonts();
            }
        }

        if fonts.is_empty() {
            return Self::get_sample_fonts();
        }

        // Sort by family then style
        fonts.sort_by(|a, b| {
            match a.family.cmp(&b.family) {
                std::cmp::Ordering::Equal => a.style.cmp(&b.style),
                other => other,
            }
        });

        // Remove duplicates
        fonts.dedup_by(|a, b| a.family == b.family && a.style == b.style);

        fonts
    }

    /// Parse a line from fc-list output
    fn parse_fc_list_line(line: &str) -> Option<FontInfo> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 3 {
            return None;
        }

        let family = parts[0].trim().to_string();
        let style = parts.get(1).map(|s| s.trim()).unwrap_or("Regular").to_string();
        let file = parts.get(2).map(|s| s.trim()).unwrap_or("").to_string();
        let slant = parts.get(3).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        let weight = parts.get(4).and_then(|s| s.trim().parse().ok()).unwrap_or(400);

        if family.is_empty() {
            return None;
        }

        // Determine category based on font characteristics
        let category = Self::detect_category(&family, &style);

        Some(FontInfo {
            family,
            style,
            file_path: PathBuf::from(file),
            weight,
            slant,
            category,
            version: String::new(),
            designer: String::new(),
            license: String::new(),
        })
    }

    /// Detect font category from name and style
    fn detect_category(family: &str, style: &str) -> FontCategory {
        let family_lower = family.to_lowercase();
        let style_lower = style.to_lowercase();

        // Monospace detection
        if family_lower.contains("mono") ||
           family_lower.contains("code") ||
           family_lower.contains("console") ||
           family_lower.contains("courier") ||
           family_lower.contains("fixed") ||
           family_lower.contains("terminal") {
            return FontCategory::Monospace;
        }

        // Serif detection
        if family_lower.contains("serif") && !family_lower.contains("sans") {
            return FontCategory::Serif;
        }

        // Sans-serif detection
        if family_lower.contains("sans") ||
           family_lower.contains("gothic") ||
           family_lower.contains("arial") ||
           family_lower.contains("helvetica") {
            return FontCategory::SansSerif;
        }

        // Display/decorative detection
        if family_lower.contains("display") ||
           family_lower.contains("headline") ||
           family_lower.contains("poster") ||
           family_lower.contains("decorative") {
            return FontCategory::Display;
        }

        // Handwriting/script detection
        if family_lower.contains("script") ||
           family_lower.contains("handwrit") ||
           family_lower.contains("cursive") ||
           family_lower.contains("callig") {
            return FontCategory::Handwriting;
        }

        // Default to serif for fonts with serifs
        if family_lower.contains("times") ||
           family_lower.contains("georgia") ||
           family_lower.contains("garamond") ||
           family_lower.contains("palatino") {
            return FontCategory::Serif;
        }

        // Default based on common font families
        if family_lower.contains("noto") ||
           family_lower.contains("ubuntu") ||
           family_lower.contains("dejavu") ||
           family_lower.contains("liberation") {
            if family_lower.contains("serif") {
                return FontCategory::Serif;
            }
            return FontCategory::SansSerif;
        }

        FontCategory::SansSerif
    }

    /// Get sample fonts for development/fallback
    fn get_sample_fonts() -> Vec<FontInfo> {
        let samples = vec![
            ("Noto Sans", "Regular", FontCategory::SansSerif),
            ("Noto Sans", "Bold", FontCategory::SansSerif),
            ("Noto Sans", "Italic", FontCategory::SansSerif),
            ("Noto Serif", "Regular", FontCategory::Serif),
            ("Noto Serif", "Bold", FontCategory::Serif),
            ("Noto Mono", "Regular", FontCategory::Monospace),
            ("DejaVu Sans", "Regular", FontCategory::SansSerif),
            ("DejaVu Sans", "Bold", FontCategory::SansSerif),
            ("DejaVu Serif", "Regular", FontCategory::Serif),
            ("DejaVu Sans Mono", "Regular", FontCategory::Monospace),
            ("Liberation Sans", "Regular", FontCategory::SansSerif),
            ("Liberation Serif", "Regular", FontCategory::Serif),
            ("Liberation Mono", "Regular", FontCategory::Monospace),
            ("Ubuntu", "Regular", FontCategory::SansSerif),
            ("Ubuntu", "Bold", FontCategory::SansSerif),
            ("Ubuntu Mono", "Regular", FontCategory::Monospace),
            ("Cantarell", "Regular", FontCategory::SansSerif),
            ("Cantarell", "Bold", FontCategory::SansSerif),
            ("Source Code Pro", "Regular", FontCategory::Monospace),
            ("Fira Code", "Regular", FontCategory::Monospace),
            ("JetBrains Mono", "Regular", FontCategory::Monospace),
            ("Inter", "Regular", FontCategory::SansSerif),
            ("Inter", "Bold", FontCategory::SansSerif),
            ("Roboto", "Regular", FontCategory::SansSerif),
            ("Roboto", "Bold", FontCategory::SansSerif),
            ("Open Sans", "Regular", FontCategory::SansSerif),
            ("Lato", "Regular", FontCategory::SansSerif),
            ("Dancing Script", "Regular", FontCategory::Handwriting),
            ("Pacifico", "Regular", FontCategory::Handwriting),
            ("Lobster", "Regular", FontCategory::Display),
            ("Playfair Display", "Regular", FontCategory::Display),
        ];

        samples
            .into_iter()
            .map(|(family, style, category)| FontInfo {
                family: family.to_string(),
                style: style.to_string(),
                file_path: PathBuf::from(format!("/usr/share/fonts/{}/{}.ttf",
                    family.to_lowercase().replace(' ', ""),
                    family.to_lowercase().replace(' ', "-")
                )),
                weight: if style.contains("Bold") { 700 } else { 400 },
                slant: if style.contains("Italic") { 100 } else { 0 },
                category,
                version: "1.0".to_string(),
                designer: String::new(),
                license: "OFL".to_string(),
            })
            .collect()
    }

    /// Query detailed font info using fc-query
    pub fn query_font(&self, path: &PathBuf) -> Option<FontInfo> {
        let output = Command::new("fc-query")
            .args(["--format", "%{family}|%{style}|%{fontversion}|%{foundry}\n"])
            .arg(path)
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()?;
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() < 2 {
            return None;
        }

        Some(FontInfo {
            family: parts[0].trim().to_string(),
            style: parts.get(1).map(|s| s.trim()).unwrap_or("Regular").to_string(),
            file_path: path.clone(),
            weight: 400,
            slant: 0,
            category: FontCategory::SansSerif,
            version: parts.get(2).map(|s| s.trim().to_string()).unwrap_or_default(),
            designer: parts.get(3).map(|s| s.trim().to_string()).unwrap_or_default(),
            license: String::new(),
        })
    }

    /// Install a font to user directory
    pub fn install_font(&self, source: &PathBuf) -> Result<(), FontInstallError> {
        if !source.exists() {
            return Err(FontInstallError::FileNotFound(
                source.display().to_string()
            ));
        }

        // Validate font extension
        let ext = source.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !["ttf", "otf", "woff", "woff2"].contains(&ext.as_str()) {
            return Err(FontInstallError::InvalidFormat(ext));
        }

        // Get user fonts directory
        let fonts_dir = dirs::home_dir()
            .ok_or_else(|| FontInstallError::PermissionDenied("Cannot find home directory".into()))?
            .join(".local/share/fonts");

        // Create directory if it doesn't exist
        fs::create_dir_all(&fonts_dir)?;

        // Copy font file
        let dest = fonts_dir.join(source.file_name().unwrap_or_default());
        fs::copy(source, &dest)?;

        // Update font cache
        self.update_cache()?;

        Ok(())
    }

    /// Uninstall a font
    pub fn uninstall_font(&self, font: &FontInfo) -> Result<(), FontInstallError> {
        // Only allow uninstalling user fonts
        let home = dirs::home_dir()
            .ok_or_else(|| FontInstallError::PermissionDenied("Cannot find home directory".into()))?;

        let user_fonts_path = home.join(".local/share/fonts");
        let old_fonts_path = home.join(".fonts");

        if !font.file_path.starts_with(&user_fonts_path) &&
           !font.file_path.starts_with(&old_fonts_path) {
            return Err(FontInstallError::PermissionDenied(
                "Can only uninstall user fonts".into()
            ));
        }

        // Remove font file
        fs::remove_file(&font.file_path)?;

        // Update font cache
        self.update_cache()?;

        Ok(())
    }

    /// Update font cache using fc-cache
    pub fn update_cache(&self) -> Result<(), FontInstallError> {
        let status = Command::new("fc-cache")
            .arg("-fv")
            .status()
            .map_err(|e| FontInstallError::FontconfigError(e.to_string()))?;

        if !status.success() {
            return Err(FontInstallError::FontconfigError(
                "fc-cache failed".into()
            ));
        }

        Ok(())
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self::new()
    }
}
