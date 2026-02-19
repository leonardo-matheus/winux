// Font rendering utilities

use gtk4::prelude::*;
use gdk4::RGBA;
use pango::{AttrList, AttrFontDesc, AttrInt, FontDescription, Weight};

use super::FontInfo;

/// Render configuration for font preview
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub font_size: i32,
    pub text_color: RGBA,
    pub background_color: RGBA,
    pub line_height: f64,
    pub letter_spacing: i32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            font_size: 24,
            text_color: RGBA::new(1.0, 1.0, 1.0, 1.0),
            background_color: RGBA::new(0.1, 0.1, 0.1, 1.0),
            line_height: 1.5,
            letter_spacing: 0,
        }
    }
}

/// Font renderer for creating previews
pub struct FontRenderer {
    config: RenderConfig,
}

impl FontRenderer {
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
        }
    }

    pub fn with_config(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Create Pango attributes for a font
    pub fn create_attributes(&self, font: &FontInfo) -> AttrList {
        let attrs = AttrList::new();

        // Font family and style
        let font_desc = FontDescription::from_string(&font.to_pango_string());
        font_desc.set_size(self.config.font_size * pango::SCALE);
        let font_attr = AttrFontDesc::new(&font_desc);
        attrs.insert(font_attr);

        // Letter spacing
        if self.config.letter_spacing != 0 {
            let spacing_attr = AttrInt::new_letter_spacing(
                self.config.letter_spacing * pango::SCALE
            );
            attrs.insert(spacing_attr);
        }

        attrs
    }

    /// Create Pango attributes for a specific size
    pub fn create_sized_attributes(&self, font: &FontInfo, size: i32) -> AttrList {
        let attrs = AttrList::new();

        let font_desc = FontDescription::from_string(&font.to_pango_string());
        font_desc.set_size(size * pango::SCALE);
        let font_attr = AttrFontDesc::new(&font_desc);
        attrs.insert(font_attr);

        attrs
    }

    /// Create attributes for weight preview
    pub fn create_weight_attributes(&self, font: &FontInfo, weight: i32) -> AttrList {
        let attrs = AttrList::new();

        let mut font_desc = FontDescription::from_string(&font.family);
        font_desc.set_size(self.config.font_size * pango::SCALE);
        font_desc.set_weight(Weight::__Unknown(weight));
        let font_attr = AttrFontDesc::new(&font_desc);
        attrs.insert(font_attr);

        attrs
    }

    /// Set font size
    pub fn set_size(&mut self, size: i32) {
        self.config.font_size = size;
    }

    /// Set text color
    pub fn set_text_color(&mut self, color: RGBA) {
        self.config.text_color = color;
    }

    /// Set background color
    pub fn set_background_color(&mut self, color: RGBA) {
        self.config.background_color = color;
    }

    /// Get current config
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Set config
    pub fn set_config(&mut self, config: RenderConfig) {
        self.config = config;
    }
}

impl Default for FontRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample text presets
pub struct SampleText;

impl SampleText {
    pub const ALPHABET_UPPER: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    pub const ALPHABET_LOWER: &'static str = "abcdefghijklmnopqrstuvwxyz";
    pub const NUMBERS: &'static str = "0123456789";
    pub const PUNCTUATION: &'static str = "!@#$%^&*()[]{}|;':\",./<>?";

    pub const PANGRAM_PT: &'static str =
        "A rapida raposa marrom pula sobre o cachorro preguicoso";
    pub const PANGRAM_EN: &'static str =
        "The quick brown fox jumps over the lazy dog";

    pub const LOREM_SHORT: &'static str =
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    pub const LOREM_MEDIUM: &'static str =
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
         Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
    pub const LOREM_LONG: &'static str =
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
         Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
         Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
         nisi ut aliquip ex ea commodo consequat.";

    /// Get full character set for specimen view
    pub fn full_specimen() -> String {
        format!(
            "{}\n{}\n{}\n{}",
            Self::ALPHABET_UPPER,
            Self::ALPHABET_LOWER,
            Self::NUMBERS,
            Self::PUNCTUATION
        )
    }

    /// Get all sample texts
    pub fn all_samples() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Alfabeto", Self::ALPHABET_UPPER),
            ("Minusculas", Self::ALPHABET_LOWER),
            ("Numeros", Self::NUMBERS),
            ("Pontuacao", Self::PUNCTUATION),
            ("Pangrama (PT)", Self::PANGRAM_PT),
            ("Pangrama (EN)", Self::PANGRAM_EN),
            ("Lorem Ipsum", Self::LOREM_SHORT),
        ]
    }
}

/// CSS provider for font styling
pub fn create_font_css(font: &FontInfo, config: &RenderConfig) -> String {
    let rgba_to_css = |c: &RGBA| {
        format!(
            "rgba({}, {}, {}, {})",
            (c.red() * 255.0) as u8,
            (c.green() * 255.0) as u8,
            (c.blue() * 255.0) as u8,
            c.alpha()
        )
    };

    format!(
        r#"
        .font-preview {{
            font-family: "{}";
            font-size: {}pt;
            font-weight: {};
            font-style: {};
            color: {};
            background-color: {};
            padding: 12px;
        }}
        "#,
        font.family,
        config.font_size,
        font.weight,
        if font.is_italic() { "italic" } else { "normal" },
        rgba_to_css(&config.text_color),
        rgba_to_css(&config.background_color)
    )
}
