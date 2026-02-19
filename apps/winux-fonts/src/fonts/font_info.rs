// Font information structures

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Font category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontCategory {
    All,
    Serif,
    SansSerif,
    Monospace,
    Display,
    Handwriting,
    Symbol,
}

impl Default for FontCategory {
    fn default() -> Self {
        Self::All
    }
}

impl std::fmt::Display for FontCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "Todas"),
            Self::Serif => write!(f, "Serif"),
            Self::SansSerif => write!(f, "Sans-Serif"),
            Self::Monospace => write!(f, "Monoespaco"),
            Self::Display => write!(f, "Display"),
            Self::Handwriting => write!(f, "Cursiva"),
            Self::Symbol => write!(f, "Simbolos"),
        }
    }
}

/// Complete font information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    /// Font family name (e.g., "Noto Sans")
    pub family: String,

    /// Font style (e.g., "Regular", "Bold", "Italic")
    pub style: String,

    /// Path to font file
    pub file_path: PathBuf,

    /// Font weight (100-900, 400 = normal)
    pub weight: i32,

    /// Font slant (0 = normal, 100 = italic, 110 = oblique)
    pub slant: i32,

    /// Font category
    pub category: FontCategory,

    /// Font version
    pub version: String,

    /// Font designer/foundry
    pub designer: String,

    /// License information
    pub license: String,
}

impl FontInfo {
    /// Create a new FontInfo
    pub fn new(family: &str, style: &str, path: PathBuf) -> Self {
        Self {
            family: family.to_string(),
            style: style.to_string(),
            file_path: path,
            weight: 400,
            slant: 0,
            category: FontCategory::SansSerif,
            version: String::new(),
            designer: String::new(),
            license: String::new(),
        }
    }

    /// Get full font name (family + style)
    pub fn full_name(&self) -> String {
        if self.style == "Regular" {
            self.family.clone()
        } else {
            format!("{} {}", self.family, self.style)
        }
    }

    /// Get Pango font description string
    pub fn to_pango_string(&self) -> String {
        let weight = self.weight_name();
        let slant = if self.slant > 0 { " Italic" } else { "" };
        format!("{} {}{}", self.family, weight, slant)
    }

    /// Get weight name from numeric value
    pub fn weight_name(&self) -> &str {
        match self.weight {
            0..=149 => "Thin",
            150..=249 => "Extra-Light",
            250..=349 => "Light",
            350..=449 => "Regular",
            450..=549 => "Medium",
            550..=649 => "Semi-Bold",
            650..=749 => "Bold",
            750..=849 => "Extra-Bold",
            _ => "Black",
        }
    }

    /// Check if font is bold
    pub fn is_bold(&self) -> bool {
        self.weight >= 600 || self.style.to_lowercase().contains("bold")
    }

    /// Check if font is italic
    pub fn is_italic(&self) -> bool {
        self.slant > 0 || self.style.to_lowercase().contains("italic")
    }

    /// Check if font is monospace
    pub fn is_monospace(&self) -> bool {
        self.category == FontCategory::Monospace
    }

    /// Get file extension
    pub fn file_format(&self) -> &str {
        self.file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
    }

    /// Get file size in bytes
    pub fn file_size(&self) -> Option<u64> {
        std::fs::metadata(&self.file_path)
            .map(|m| m.len())
            .ok()
    }

    /// Get human-readable file size
    pub fn file_size_string(&self) -> String {
        match self.file_size() {
            Some(size) => {
                if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                }
            }
            None => "Unknown".to_string(),
        }
    }

    /// Check if font is user-installed (in user directories)
    pub fn is_user_font(&self) -> bool {
        if let Some(home) = dirs::home_dir() {
            let path_str = self.file_path.display().to_string();
            path_str.starts_with(&home.display().to_string())
        } else {
            false
        }
    }
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            family: "Sans".to_string(),
            style: "Regular".to_string(),
            file_path: PathBuf::new(),
            weight: 400,
            slant: 0,
            category: FontCategory::SansSerif,
            version: String::new(),
            designer: String::new(),
            license: String::new(),
        }
    }
}

impl PartialEq for FontInfo {
    fn eq(&self, other: &Self) -> bool {
        self.family == other.family && self.style == other.style
    }
}

impl Eq for FontInfo {}

impl std::hash::Hash for FontInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.family.hash(state);
        self.style.hash(state);
    }
}

/// Character range information
#[derive(Debug, Clone)]
pub struct CharacterRange {
    pub name: String,
    pub start: u32,
    pub end: u32,
}

impl CharacterRange {
    /// Get common Unicode ranges
    pub fn common_ranges() -> Vec<Self> {
        vec![
            Self { name: "Basic Latin".into(), start: 0x0020, end: 0x007F },
            Self { name: "Latin-1 Supplement".into(), start: 0x0080, end: 0x00FF },
            Self { name: "Latin Extended-A".into(), start: 0x0100, end: 0x017F },
            Self { name: "Latin Extended-B".into(), start: 0x0180, end: 0x024F },
            Self { name: "Greek".into(), start: 0x0370, end: 0x03FF },
            Self { name: "Cyrillic".into(), start: 0x0400, end: 0x04FF },
            Self { name: "General Punctuation".into(), start: 0x2000, end: 0x206F },
            Self { name: "Currency Symbols".into(), start: 0x20A0, end: 0x20CF },
            Self { name: "Arrows".into(), start: 0x2190, end: 0x21FF },
            Self { name: "Mathematical Operators".into(), start: 0x2200, end: 0x22FF },
            Self { name: "Box Drawing".into(), start: 0x2500, end: 0x257F },
            Self { name: "Block Elements".into(), start: 0x2580, end: 0x259F },
            Self { name: "Geometric Shapes".into(), start: 0x25A0, end: 0x25FF },
            Self { name: "Miscellaneous Symbols".into(), start: 0x2600, end: 0x26FF },
        ]
    }
}
