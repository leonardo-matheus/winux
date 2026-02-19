// Fonts module for Winux Fonts

pub mod fontconfig;
pub mod font_info;
pub mod render;

pub use fontconfig::FontConfig;
pub use font_info::{FontInfo, FontCategory};

use std::collections::HashMap;
use std::path::PathBuf;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Font manager - central hub for font operations
pub struct FontManager {
    fonts: Vec<FontInfo>,
    families: Vec<String>,
    search_filter: String,
    category_filter: FontCategory,
    fontconfig: FontConfig,
    matcher: SkimMatcherV2,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            families: Vec::new(),
            search_filter: String::new(),
            category_filter: FontCategory::All,
            fontconfig: FontConfig::new(),
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Scan system fonts using fontconfig
    pub fn scan_fonts(&mut self) {
        self.fonts = self.fontconfig.get_all_fonts();
        self.update_families();
    }

    /// Update family list from fonts
    fn update_families(&mut self) {
        let mut family_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for font in &self.fonts {
            family_set.insert(font.family.clone());
        }
        self.families = family_set.into_iter().collect();
        self.families.sort();
    }

    /// Get all fonts with current filters applied
    pub fn get_fonts(&self) -> Vec<&FontInfo> {
        self.fonts
            .iter()
            .filter(|f| self.matches_filter(f))
            .collect()
    }

    /// Check if font matches current filters
    fn matches_filter(&self, font: &FontInfo) -> bool {
        // Category filter
        if self.category_filter != FontCategory::All {
            if font.category != self.category_filter {
                return false;
            }
        }

        // Search filter
        if !self.search_filter.is_empty() {
            let search_str = format!("{} {}", font.family, font.style);
            if self.matcher.fuzzy_match(&search_str, &self.search_filter).is_none() {
                return false;
            }
        }

        true
    }

    /// Set search filter
    pub fn set_search_filter(&mut self, query: &str) {
        self.search_filter = query.to_string();
    }

    /// Set category filter
    pub fn set_category_filter(&mut self, category: FontCategory) {
        self.category_filter = category;
    }

    /// Get font count
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Get family count
    pub fn family_count(&self) -> usize {
        self.families.len()
    }

    /// Get all families
    pub fn get_families(&self) -> &[String] {
        &self.families
    }

    /// Count fonts by category
    pub fn count_by_category(&self, category: &FontCategory) -> usize {
        if *category == FontCategory::All {
            return self.fonts.len();
        }
        self.fonts.iter().filter(|f| f.category == *category).count()
    }

    /// Install a font from file
    pub fn install_font(&mut self, path: &PathBuf) -> Result<(), FontInstallError> {
        self.fontconfig.install_font(path)?;
        self.scan_fonts(); // Refresh font list
        Ok(())
    }

    /// Uninstall a font
    pub fn uninstall_font(&mut self, font: &FontInfo) -> Result<(), FontInstallError> {
        self.fontconfig.uninstall_font(font)?;
        self.scan_fonts(); // Refresh font list
        Ok(())
    }

    /// Get font by family and style
    pub fn get_font(&self, family: &str, style: &str) -> Option<&FontInfo> {
        self.fonts
            .iter()
            .find(|f| f.family == family && f.style == style)
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FontInstallError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid font format: {0}")]
    InvalidFormat(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Fontconfig error: {0}")]
    FontconfigError(String),
}
